# Uninterpreted Functions in Z3 for Abstract Algebraic Operations

**Date:** December 11, 2024  
**Context:** Full Prelude Migration - Z3 Integration  
**Status:** Design Document → Implementation

---

## The Problem

Our prelude defines abstract algebraic structures with axioms:

```kleis
structure Semigroup(S) {
  operation (•) : S × S → S
  
  axiom associativity:
    ∀(x y z : S). (x • y) • z = x • (y • z)
}
```

**Question:** How does Z3 verify this when it doesn't know what `(•)` means?

**Current State:** Z3 translator errors out:
```
Error: "Unsupported arithmetic operation: •"
```

---

## The Solution: Uninterpreted Functions

### What Are Uninterpreted Functions?

**Uninterpreted functions** in Z3 are function symbols that:
- Have a **signature** (types of inputs and outputs)
- Have **NO implementation** (no definition of what they compute)
- Can be reasoned about using **only the axioms** we assert about them

**Example:**
```rust
// Declare an abstract binary operation
let op = FuncDecl::new("•", &[&Sort::int(), &Sort::int()], &Sort::int());

// Z3 knows: (•) : Int × Int → Int
// Z3 doesn't know: What (•) actually computes

// Now assert axioms about it
solver.assert(&forall_xyz(|x, y, z| {
    op.apply(&[&op.apply(&[&x, &y]), &z])._eq(
        &op.apply(&[&x, &op.apply(&[&y, &z])])
    )
}));

// Z3 will reason about (•) using ONLY this axiom!
```

### Why This Is Perfect for Algebraic Structures

**Algebraic structures are ABSTRACT:**
- Semigroup: Any set with an associative operation
- Monoid: Semigroup + identity element
- Group: Monoid + inverse operation

**We don't want to assume `(•)` is addition or multiplication!**

Uninterpreted functions let us say:
- "Here's an operation (•)"
- "Here are its properties (associativity)"  
- "Prove things about it WITHOUT knowing what it does"

This is **exactly** how mathematicians think about algebra!

---

## Research Findings

### From Z3 Rust API (vendor/z3/src/func_decl.rs)

**Creating uninterpreted functions:**
```rust
pub fn new<S: Into<Symbol>>(name: S, domain: &[&Sort], range: &Sort) -> Self
```

**Example:**
```rust
let f = FuncDecl::new("f", &[&Sort::int(), &Sort::real()], &Sort::int());
assert_eq!(f.arity(), 2);
```

**Applying uninterpreted functions:**
```rust
pub fn apply(&self, args: &[&dyn ast::Ast]) -> ast::Dynamic
```

**Example:**
```rust
let x = Int::new_const("x");
let y = Int::new_const("y");
let result = f.apply(&[&x, &y]);  // f(x, y)
```

### From Web Research

**SMT-LIB2 supports slashes in symbols!**
```smt
(declare-fun f/g (Int) Int)
```

So we COULD use `d/dx` as an operation name in Z3, even if our parser doesn't support it yet.

**Special Relations** - Z3 has built-in support for:
- `partial_order`: Reflexive, antisymmetric, transitive (like `≤`)
- `linear_order`: Total ordering
- `transitive_closure`: Computing transitive closure

**Insight:** These are basically **predefined uninterpreted functions with axioms**!

### From Philip Zucker's Article

"To model differentiation in Z3, define derivatives as uninterpreted functions and assert axioms like:
- Linearity: d(f + g) = d(f) + d(g)
- Product rule: d(f × g) = d(f) × g + f × d(g)
- Chain rule: d(f(g)) = d(f)(g) × d(g)

Z3 can then reason about derivatives using only these axioms."

**This is exactly our approach!**

---

## Design: Uninterpreted Function Support

### Architecture

```
Kleis Axiom                      Z3 Solver
    ↓                                ↓
Parser: Extract operations    1. Declare operations as uninterpreted
    ↓                                ↓
Registry: Store axioms        2. Translate axioms to Z3 constraints
    ↓                                ↓
AxiomVerifier                 3. Check satisfiability
    ↓                                ↓
kleis_to_z3()                 4. Return Valid/Invalid/Unknown
    ↓
Declare operation if not known
    ↓
Apply operation in translation
```

### Implementation Plan

#### Phase 1: Detect Unknown Operations

When translating `Operation { name: "•", args }`:

```rust
fn operation_to_z3(&self, name: &str, args: &[Expression]) -> Result<Bool, String> {
    match name {
        // Known built-ins (concrete theories)
        "plus" => { /* use Int::add */ }
        "times" => { /* use Int::mul */ }
        "equals" => { /* use _eq */ }
        
        // Unknown operation - use uninterpreted function
        _ => self.apply_uninterpreted_function(name, args)
    }
}
```

#### Phase 2: Declare Uninterpreted Functions

```rust
fn get_or_declare_operation(&mut self, name: &str, arity: usize) -> FuncDecl {
    // Check cache
    if let Some(decl) = self.declared_ops.get(name) {
        return decl.clone();
    }
    
    // Create new uninterpreted function
    // For now, assume all operations are Int → Int → Int
    let domain = vec![&Sort::int(); arity];
    let func_decl = FuncDecl::new(name, &domain, &Sort::int());
    
    self.declared_ops.insert(name.to_string(), func_decl.clone());
    
    func_decl
}
```

#### Phase 3: Apply in Translation

```rust
fn apply_uninterpreted_function(
    &mut self,
    name: &str,
    args: &[Expression],
    vars: &HashMap<String, Int>
) -> Result<Dynamic, String> {
    // Translate arguments
    let z3_args: Result<Vec<_>, _> = args
        .iter()
        .map(|arg| self.kleis_expr_to_z3_int(arg, vars))
        .collect();
    let z3_args = z3_args?;
    
    // Get or create function declaration
    let func_decl = self.get_or_declare_operation(name, args.len());
    
    // Apply function: op(arg1, arg2, ...)
    let ast_args: Vec<&dyn Ast> = z3_args.iter().map(|a| a as &dyn Ast).collect();
    let result = func_decl.apply(&ast_args);
    
    Ok(result)
}
```

### Type Handling

**Challenge:** Different operations have different types:
- `(•) : S × S → S` (binary operation)
- `inv : S → S` (unary operation)  
- `e : S` (nullary - identity element)

**Solution:** Infer arity from number of arguments, use Int sort for now.

**Future:** Could use Sort::uninterpreted() for proper type polymorphism.

---

## Expected Behavior After Implementation

### Test Case: Semigroup Associativity

**Input:**
```kleis
structure Semigroup(S) {
  operation (•) : S × S → S
  axiom associativity: ∀(x y z : S). (x • y) • z = x • (y • z)
}
```

**Z3 Translation:**
```rust
// Declare (•) as uninterpreted binary operation
let op = FuncDecl::new("•", &[&Sort::int(), &Sort::int()], &Sort::int());

// Create variables
let x = Int::new_const("x");
let y = Int::new_const("y");
let z = Int::new_const("z");

// Build: (x • y) • z
let xy = op.apply(&[&x, &y]);
let lhs = op.apply(&[&xy, &z]);

// Build: x • (y • z)
let yz = op.apply(&[&y, &z]);
let rhs = op.apply(&[&x, &yz]);

// Assert axiom
solver.assert(&lhs._eq(&rhs));

// To verify: assert NOT(axiom) and check UNSAT
solver.assert(&lhs._eq(&rhs).not());
assert_eq!(solver.check(), SatResult::Unsat);  // ✅ Valid!
```

### Test Case: Ring Distributivity

**Input:**
```kleis
axiom left_distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
```

**Z3 Translation:**
```rust
// Declare operations as uninterpreted
let add_op = FuncDecl::new("+", &[&Sort::int(), &Sort::int()], &Sort::int());
let mul_op = FuncDecl::new("×", &[&Sort::int(), &Sort::int()], &Sort::int());

// Build: x × (y + z)
let y_plus_z = add_op.apply(&[&y, &z]);
let lhs = mul_op.apply(&[&x, &y_plus_z]);

// Build: (x × y) + (x × z)
let xy = mul_op.apply(&[&x, &y]);
let xz = mul_op.apply(&[&x, &z]);
let rhs = add_op.apply(&[&xy, &xz]);

// Assert axiom
solver.assert(&lhs._eq(&rhs));
```

**Z3 will treat `(+)` and `(×)` as abstract operations with no built-in meaning!**

---

## Implementation Checklist

### Step 1: Update AxiomVerifier Struct
- ✅ Already has `declared_ops: HashMap<String, FuncDecl>`
- ✅ Already has `#[cfg(feature = "axiom-verification")]` guards

### Step 2: Add Declaration Method
```rust
fn get_or_declare_operation(&mut self, name: &str, arity: usize) -> Result<FuncDecl, String>
```

### Step 3: Update operation_to_z3()
- Check for built-ins first (equals, and, or, etc.)
- Fall back to uninterpreted function for unknowns

### Step 4: Update kleis_expr_to_z3_int()
- Support applying uninterpreted functions
- Return Dynamic (not just Int) to handle general operations

### Step 5: Handle Type Conversions
- Dynamic → Bool for logical operations
- Dynamic → Int for arithmetic operations
- Add helper methods for conversions

### Step 6: Test
- Verify Semigroup associativity (abstract operation)
- Verify Ring distributivity (two abstract operations)
- Verify Monoid identity (with identity element)

---

## Key Design Decisions

### Decision 1: Default to Int Sort

**For abstract operations, use Int sort as default.**

**Why:**
- Simplest approach
- Works for most algebraic structures
- Z3's integer theory is well-optimized

**Future:** Could infer proper sorts from type signatures.

### Decision 2: Lazy Declaration

**Declare operations on-demand when first encountered.**

**Why:**
- Avoids declaring unused operations
- Operations discovered during translation
- Simpler implementation

**Alternative:** Pre-declare all operations from registry (more complex).

### Decision 3: Cache Function Declarations

**Store declared operations in HashMap.**

**Why:**
- FuncDecl creation is relatively expensive
- Same operation used multiple times in axioms
- Consistent identity across uses

### Decision 4: Keep Built-In Theories

**Still use concrete theories for equals, and, or, etc.**

**Why:**
- Z3's built-in theories are more powerful
- Better error messages
- Better performance
- Only use uninterpreted for truly abstract operations

---

## Comparison: Concrete vs Uninterpreted

### Concrete Theory (Current - for equals, and, or)

```rust
"equals" => {
    let left = self.kleis_expr_to_z3_int(&args[0], vars)?;
    let right = self.kleis_expr_to_z3_int(&args[1], vars)?;
    Ok(left._eq(&right))
}
```

**Pros:**
- Uses Z3's built-in equality theory
- Better performance
- More powerful reasoning

**Cons:**
- Only works for operations Z3 knows

### Uninterpreted Function (New - for •, +, ×)

```rust
_ => {
    let func_decl = self.get_or_declare_operation(name, args.len())?;
    let z3_args = translate_all(args)?;
    Ok(func_decl.apply(&z3_args))
}
```

**Pros:**
- Works for ANY operation
- Z3 reasons using axioms only
- Perfect for abstract algebra

**Cons:**
- Less powerful than built-in theories
- Requires axioms for reasoning

---

## Example: Verifying Commutativity

**Axiom:**
```kleis
axiom commutativity: ∀(x y : A). x • y = y • x
```

**Translation:**
```rust
// Declare (•) as uninterpreted
let op = FuncDecl::new("•", &[&Sort::int(), &Sort::int()], &Sort::int());

// Variables
let x = Int::new_const("x");
let y = Int::new_const("y");

// Build: x • y
let xy = op.apply(&[&x, &y]);

// Build: y • x  
let yx = op.apply(&[&y, &x]);

// Assert: x • y = y • x
solver.assert(&xy._eq(&yx));

// To verify, check if NEGATION is unsat
solver.assert(&xy._eq(&yx).not());
assert_eq!(solver.check(), SatResult::Unsat);  // ✅ Valid!
```

**Z3 says:** "For ANY function (•), if we assume nothing else, 
can we find an example where x • y ≠ y • x?"

**Answer:** "Yes! (e.g., matrix multiplication)"

**So commutativity is NOT universally true - it's a constraint!**

---

## Example: Verifying Associativity

**Axiom:**
```kleis
axiom associativity: ∀(x y z : S). (x • y) • z = x • (y • z)
```

**Translation:**
```rust
let op = FuncDecl::new("•", &[&Sort::int(), &Sort::int()], &Sort::int());

let x = Int::new_const("x");
let y = Int::new_const("y");
let z = Int::new_const("z");

// Build: (x • y) • z
let xy = op.apply(&[&x, &y]);
let lhs = op.apply(&[&xy, &z]);

// Build: x • (y • z)
let yz = op.apply(&[&y, &z]);
let rhs = op.apply(&[&x, &yz]);

// Try to find counterexample
solver.assert(&lhs._eq(&rhs).not());
assert_eq!(solver.check(), SatResult::Unsat);  // ✅ Universally valid!
```

**Z3 says:** "For ANY binary operation (•), 
can we find x, y, z where (x • y) • z ≠ x • (y • z)?"

**Answer:** "No! This is a tautology - we can't even construct a counterexample."

**Wait, is that right?** No! Associativity is NOT universal - subtraction is NOT associative!

**The test should find:** `(5 - 3) - 1 = 1` but `5 - (3 - 1) = 3`

**So the test reveals:** We need to NOT assert any other axioms about (•) when checking associativity alone.

---

## Verification Strategy

### For Checking a Single Axiom

**Goal:** Is this axiom universally true for ANY operation?

**Process:**
1. Declare operation(s) as uninterpreted
2. Assert NEGATION of axiom
3. Check SAT
4. If UNSAT → axiom is universal
5. If SAT → axiom is not universal (and that's OK!)

**Example - Associativity:**
```
Result: SAT (counterexample: subtraction)
Interpretation: Associativity is not universal, it's a constraint
Conclusion: Semigroup is a meaningful distinction (not all operations are associative)
```

### For Checking an Implementation

**Goal:** Does a concrete implementation satisfy a structure's axioms?

**Process:**
1. Map operations to concrete Z3 theories (addition → Int::add)
2. Load ALL axioms for that structure
3. Try to find violation
4. If UNSAT → implementation satisfies axioms

**Example - Does ℝ with + form a Monoid?**
```rust
// Use concrete addition (not uninterpreted)
"+" => Int::add(...)

// Assert all Monoid axioms
solver.assert(associativity_axiom);
solver.assert(left_identity_axiom);
solver.assert(right_identity_axiom);

// Check satisfiability
// If consistent → ℝ could be a Monoid (pending identity element check)
```

---

## Implementation Details

### Changes to AxiomVerifier

```rust
impl AxiomVerifier {
    /// Get or declare an operation as uninterpreted function
    fn get_or_declare_operation(
        &mut self, 
        name: &str, 
        arity: usize
    ) -> Result<FuncDecl, String> {
        // Check cache
        if let Some(decl) = self.declared_ops.get(name) {
            return Ok(decl.clone());
        }
        
        // Create uninterpreted function
        // Signature: Int × Int × ... → Int (arity times)
        let domain: Vec<_> = (0..arity).map(|_| &Sort::int()).collect();
        let func_decl = FuncDecl::new(name, &domain, &Sort::int());
        
        println!("   🔧 Declared uninterpreted function: {} (arity {})", name, arity);
        
        // Cache it
        self.declared_ops.insert(name.to_string(), func_decl.clone());
        
        Ok(func_decl)
    }
}
```

### Changes to operation_to_z3

```rust
fn operation_to_z3(
    &mut self,  // Now &mut to allow declaration
    name: &str,
    args: &[Expression],
    vars: &HashMap<String, Int>,
) -> Result<Bool, String> {
    match name {
        // Built-in theories (concrete)
        "equals" | "eq" => { /* concrete equality */ }
        "and" | "logical_and" => { /* concrete boolean logic */ }
        // ... other built-ins
        
        // Unknown operation - uninterpreted function
        _ => {
            let func_decl = self.get_or_declare_operation(name, args.len())?;
            
            // Translate arguments
            let z3_args: Result<Vec<_>, _> = args
                .iter()
                .map(|arg| self.kleis_expr_to_z3_int(arg, vars))
                .collect();
            let z3_args = z3_args?;
            
            // Apply function
            let ast_args: Vec<&dyn Ast> = z3_args.iter().map(|a| a as &dyn Ast).collect();
            let result = func_decl.apply(&ast_args);
            
            // Need to convert Dynamic to Bool for logical context
            // For now, assume it's used in equality context
            Ok(Bool::from_bool(true))  // TODO: Proper type handling
        }
    }
}
```

### Type Handling Challenge

**Problem:** Uninterpreted functions return `Dynamic`, but we need `Bool` or `Int`.

**Solution:** 
```rust
// For operations in equality context: x • y = z
// The equality operator handles the Dynamic → Bool conversion

// For operations as standalone: just (x • y)
// Need to track expected type from context
```

This is complex - might need to refactor `kleis_to_z3` return type.

---

## Simplified First Implementation

### Approach: Two-Phase Translation

**Phase 1: Translate to Dynamic (general)**
```rust
fn kleis_to_z3_dynamic(&self, expr: &Expression) -> Result<Dynamic, String>
```

**Phase 2: Context-specific conversion**
```rust
fn operation_to_z3_bool(&self, ...) -> Result<Bool, String> {
    let dynamic = self.kleis_to_z3_dynamic(...)?;
    // Convert based on operation type
}
```

**Or simpler:** Just handle equals specially:

```rust
"equals" => {
    // Translate both sides as Dynamic
    let left = self.kleis_to_z3_dynamic(&args[0])?;
    let right = self.kleis_to_z3_dynamic(&args[1])?;
    
    // Use Dynamic's _eq method
    Ok(left._eq(&right))
}
```

This way uninterpreted functions return Dynamic, and equals handles the comparison.

---

## Testing Strategy

### Test 1: Basic Uninterpreted Function

```rust
#[test]
fn test_uninterpreted_function_declaration() {
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    
    // Create simple axiom using abstract operation
    let axiom = parse_kleis("∀(x y : S). equals(op(x, y), op(y, x))");
    
    // Should declare 'op' as uninterpreted
    let result = verifier.verify_axiom(&axiom);
    
    // Commutativity is not universal, so should be SAT (find counterexample)
    assert!(matches!(result, Ok(VerificationResult::Invalid { .. })));
}
```

### Test 2: Prelude Associativity

```rust
#[test]
fn test_semigroup_associativity_from_prelude() {
    let checker = TypeChecker::with_stdlib().unwrap();
    let registry = checker.get_structure_registry();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    
    let axiom = registry.get_axioms("Semigroup")[0].1;
    let result = verifier.verify_axiom(axiom);
    
    // Associativity should be universally valid
    assert_eq!(result, Ok(VerificationResult::Valid));
}
```

### Test 3: Ring Distributivity

```rust
#[test]
fn test_ring_distributivity_from_prelude() {
    let checker = TypeChecker::with_stdlib().unwrap();
    let registry = checker.get_structure_registry();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();
    
    let axiom = registry
        .get_axioms("Ring")
        .iter()
        .find(|(name, _)| name == "left_distributivity")
        .unwrap()
        .1;
    
    let result = verifier.verify_axiom(axiom);
    
    // Distributivity should be universally valid
    assert_eq!(result, Ok(VerificationResult::Valid));
}
```

---

## Expected Results

### Which Axioms Should Verify as Universal?

**✅ Universal (UNSAT when negated):**
- Associativity: `(x • y) • z = x • (y • z)` - This is actually NOT universal!
  - Subtraction: `(5-3)-1 = 1` but `5-(3-1) = 3`
  - **Wait...** Let me reconsider this

**Actually, need to clarify our testing goal:**

#### Goal A: Check if axiom is universally true
- Assert NOT(axiom)
- If UNSAT → universally true
- If SAT → not universal (counterexample exists)

**Result:** Most axioms are NOT universal (that's why structures exist!)

#### Goal B: Check if axiom is satisfiable
- Assert axiom
- If SAT → consistent (structure can exist)
- If UNSAT → contradictory (impossible structure)

**Result:** Well-formed axioms should be SAT (structure is possible)

#### Goal C: Check if implementation satisfies axioms
- Use concrete operations (not uninterpreted)
- Assert all structure axioms
- Check consistency

**Result:** Good implementations should be SAT

### What We Should Test

**For abstract axioms (uninterpreted):**
- Test they're **satisfiable** (not contradictory)
- Not trying to prove them universal

**For concrete implementations:**
- Test they **satisfy the axioms**
- Use concrete Z3 theories

---

## Revised Testing Approach

### Test: Axiom Satisfiability

```rust
// Goal: Verify Ring axioms are consistent (not contradictory)
let mut verifier = AxiomVerifier::new(&registry).unwrap();

// Load all Ring axioms with uninterpreted operations
for (name, axiom) in registry.get_axioms("Ring") {
    verifier.assert_axiom(axiom)?;  // Just assert, don't negate
}

// Check if axioms are consistent
match verifier.check_satisfiability() {
    SatResult::Sat => println!("✅ Ring axioms are consistent"),
    SatResult::Unsat => panic!("❌ Ring axioms are contradictory!"),
    SatResult::Unknown => println!("⚠️ Z3 couldn't determine"),
}
```

**This tests:** Can a Ring structure exist at all?

---

## Implementation Priority

### MVP (Minimum Viable Product)

1. ✅ Declare uninterpreted functions on-demand
2. ✅ Apply them in translation
3. ✅ Test satisfiability of structure axioms
4. ✅ Return Dynamic, convert for equality checks

### Future Enhancements

- Sort inference from type signatures
- Optimize common operations
- Quantifier patterns for better instantiation
- Proof term extraction

---

## Code Locations

**To Modify:**
- `src/axiom_verifier.rs`:
  - `get_or_declare_operation()` (new method)
  - `operation_to_z3()` (add fallback case)
  - `kleis_expr_to_z3_dynamic()` (new method, returns Dynamic)
  - Update signatures to `&mut self` where needed

**To Test:**
- `tests/verify_prelude_axioms_test.rs` (update expectations)
- `tests/uninterpreted_functions_test.rs` (new file)

---

## Success Criteria

After implementation:

✅ Semigroup axioms load and verify as satisfiable  
✅ Ring axioms load and verify as satisfiable  
✅ Field axioms load and verify as satisfiable  
✅ VectorSpace axioms load and verify as satisfiable  
✅ Abstract operations `(•)`, `(+)`, `(×)` work in axioms  
✅ All 421 tests still pass  
✅ New integration tests pass

---

## References

**Z3 API:**
- `FuncDecl::new()` - Create uninterpreted function
- `FuncDecl::apply()` - Apply function to arguments
- `Dynamic` type - Generic AST node

**External:**
- Z3 Guide on Special Relations: https://microsoft.github.io/z3guide/docs/theories/Special%20Relations/
- Philip Zucker - Differentiation in Z3: https://www.philipzucker.com/z3_diff/
- SMT-LIB2 standard: Uninterpreted functions section

**Local:**
- `vendor/z3/src/func_decl.rs` - API implementation
- `vendor/z3/tests/lib.rs` - Usage examples
- `src/axiom_verifier.rs` - Our implementation

---

**Ready to implement!** 🚀

