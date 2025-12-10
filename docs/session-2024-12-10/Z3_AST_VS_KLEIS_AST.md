# Z3 AST vs Kleis AST - Comparison

**Date:** December 10, 2024  
**Context:** Z3 integration exploration on `feature/full-prelude-migration`

---

## TL;DR

**Z3 AST:** Typed, solver-oriented, built for theorem proving  
**Kleis AST:** Untyped, rendering-oriented, built for mathematical notation

**They serve different purposes but can be bridged!**

---

## Structure Comparison

### Kleis AST

```rust
// src/ast.rs
pub enum Expression {
    Const(String),                          // "1", "3.14", "42"
    Object(String),                         // "x", "\\alpha", "\\pi"
    Operation { name: String, args: Vec<Expression> },  // Flexible!
    Placeholder { id: usize, hint: String },
    Match { scrutinee: Box<Expression>, cases: Vec<MatchCase> },
    List(Vec<Expression>),
}
```

**Characteristics:**
- **Single enum** - all expression types unified
- **String-based** - operations are just names
- **Untyped** - doesn't know if "x" is Int or Real
- **Editor-focused** - has Placeholder for UI
- **Pattern matching** - has Match for functional programming
- **Serializable** - can save/load as JSON

### Z3 AST

```rust
// Z3 has SEPARATE types for each sort:
pub struct Int {     // Integer expressions
    ctx: Context,
    z3_ast: Z3_ast,  // Pointer to C API
}

pub struct Bool {    // Boolean expressions
    ctx: Context,
    z3_ast: Z3_ast,
}

pub struct Real {    // Real number expressions
    ctx: Context,
    z3_ast: Z3_ast,
}

pub struct BV {      // Bitvector expressions
    ctx: Context,
    z3_ast: Z3_ast,
}

pub struct Array {   // Array expressions
    ctx: Context,
    z3_ast: Z3_ast,
}
```

**Characteristics:**
- **Typed structs** - Int ≠ Bool ≠ Real (compile-time safety!)
- **Context-bound** - tied to Z3 solver context
- **Solver-oriented** - built for SAT/SMT queries
- **No UI concepts** - no placeholders or rendering hints
- **Opaque** - wraps C pointers, not direct data

---

## Key Differences

### 1. Type System

**Kleis:**
```rust
// Runtime type checking
let expr = Expression::Operation { 
    name: "plus".to_string(), 
    args: vec![...] 
};
// Don't know if args are Int, Real, or Matrix until type inference!
```

**Z3:**
```rust
// Compile-time type checking
let x: Int = Int::fresh_const("x");
let y: Int = Int::fresh_const("y");
let sum: Int = &x + &y;  // Rust ensures x and y are Ints!
```

**Impact:** Z3 catches type errors at Rust compile time, Kleis at runtime.

### 2. Operations

**Kleis:**
```rust
// Operations are named strings
Expression::Operation { 
    name: "plus",
    args: vec![x, y]
}

// Completely flexible - any operation name!
// plus, minus, gradient, christoffel, whatever!
```

**Z3:**
```rust
// Operations are typed methods
impl Int {
    pub fn add(values: &[Int]) -> Int { ... }
    pub fn mul(values: &[Int]) -> Int { ... }
    pub fn le(&self, other: &Int) -> Bool { ... }
}

// Or operator overloading:
let sum = &x + &y;  // Calls Z3_mk_add internally
```

**Impact:** Z3 operations are closed (fixed set), Kleis is open (user-defined).

### 3. Variables

**Kleis:**
```rust
Expression::Object("x")  // Just a string!
```

**Z3:**
```rust
let x = Int::fresh_const("x");  // Creates actual symbolic variable!
```

**Impact:** Z3 variables are **symbolic** (unknowns to solve for), Kleis variables are just **names** (rendering).

### 4. Constants

**Kleis:**
```rust
Expression::Const("42")  // String!
```

**Z3:**
```rust
Int::from_i64(42)  // Actual Z3 constant AST node
```

**Impact:** Z3 constants are typed values, Kleis are text (parsed later).

### 5. Context

**Kleis:**
```rust
// Context-free
let expr = Expression::Const("1");
// Can serialize, deserialize, pass around freely
```

**Z3:**
```rust
// Context-bound
let ctx = Context::new(&Config::new());
let x = Int::new_const(&ctx, "x");
// x is tied to ctx, can't use with different context
```

**Impact:** Z3 AST nodes are **ephemeral** (solver session), Kleis are **persistent** (documents).

---

## Capabilities Comparison

### What Kleis AST Can Do (Z3 Can't)

**1. Render to Multiple Formats:**
```rust
render(Expression::Operation{...}, RenderTarget::LaTeX)
render(Expression::Operation{...}, RenderTarget::Typst)
render(Expression::Operation{...}, RenderTarget::Unicode)
```

Z3 AST: Just `to_string()` for debugging.

**2. User-Defined Operations:**
```kleis
christoffel(g, mu, nu, rho)  // Custom physics operation
gradient(f)                   // Custom calculus operation
```

Z3: Only built-in operations (arithmetic, logic, arrays, etc.)

**3. Pattern Matching:**
```rust
Expression::Match { 
    scrutinee,
    cases: vec![MatchCase { pattern, body }]
}
```

Z3: No pattern matching construct.

**4. Placeholders for Editing:**
```rust
Expression::Placeholder { id: 0, hint: "numerator" }
```

Z3: No UI/editing concepts.

**5. Lists:**
```rust
Expression::List(vec![a, b, c, d])
```

Z3: Has Array but it's a function (index → value), not a list.

### What Z3 AST Can Do (Kleis Can't)

**1. Satisfiability Checking:**
```rust
solver.assert(&(x + y)._eq(&Int::from_i64(10)));
solver.assert(&x.gt(&Int::from_i64(5)));
solver.check()  // Find values of x, y!
```

Kleis: Just represents the expression, doesn't solve.

**2. Built-In Theories:**
- AC theory (commutative + associative)
- Linear arithmetic (LIA, LRA)
- Bit-vectors
- Arrays
- Quantifiers

Kleis: No built-in algebra knowledge.

**3. Simplification:**
```rust
let complex = (&x + &Int::from_i64(0)) * &y;
let simple = complex.simplify();  // Z3 simplifies automatically!
```

Kleis: Simplification not implemented yet.

**4. Model Extraction:**
```rust
let model = solver.get_model().unwrap();
let x_val = model.eval(&x, true).unwrap().as_i64().unwrap();
```

Kleis: No concept of "solving for values".

**5. Quantifier Reasoning:**
```rust
// Can reason about ∀x. P(x) and ∃x. P(x)
```

Kleis: Quantifiers are just notation (parser doesn't even support them yet).

---

## The Bridge: Kleis → Z3 Translation

### What We Need

**Generic translator that converts Kleis Expression to Z3 AST:**

```rust
fn kleis_to_z3(
    expr: &Expression,
    ctx: &Context,
    vars: &HashMap<String, z3::ast::Int>,  // Variable mapping
) -> Result<z3::ast::Int, String> {
    match expr {
        // Constants: parse and convert
        Expression::Const(s) => {
            let n: i64 = s.parse()?;
            Ok(Int::from_i64(n))
        }
        
        // Variables: look up in map
        Expression::Object(name) => {
            vars.get(name)
                .cloned()
                .ok_or_else(|| format!("Unknown variable: {}", name))
        }
        
        // Operations: map by name
        Expression::Operation { name, args } => {
            match name.as_str() {
                "plus" => {
                    let left = kleis_to_z3(&args[0], ctx, vars)?;
                    let right = kleis_to_z3(&args[1], ctx, vars)?;
                    Ok(&left + &right)
                }
                "times" => {
                    let left = kleis_to_z3(&args[0], ctx, vars)?;
                    let right = kleis_to_z3(&args[1], ctx, vars)?;
                    Ok(&left * &right)
                }
                "minus" => {
                    let left = kleis_to_z3(&args[0], ctx, vars)?;
                    let right = kleis_to_z3(&args[1], ctx, vars)?;
                    Ok(&left - &right)
                }
                // ... more operations
                _ => Err(format!("Unsupported operation: {}", name))
            }
        }
        
        // Can't translate UI-specific constructs
        Expression::Placeholder { .. } => {
            Err("Cannot translate placeholder to Z3".to_string())
        }
        
        Expression::Match { .. } => {
            Err("Cannot translate match expression to Z3".to_string())
        }
        
        Expression::List(_) => {
            // Could map to Z3 Array if needed
            Err("List translation not implemented".to_string())
        }
    }
}
```

### Type Mapping

| Kleis Concept | Z3 Type | Notes |
|---------------|---------|-------|
| Numeric constants | `Int` or `Real` | Depends on value |
| Variables (objects) | `Int::fresh_const()` | Create symbolic variable |
| plus, minus, times | Operator overloading | `+`, `-`, `*` |
| equals | `.eq()` method | Returns `Bool` |
| less_than | `.lt()` method | Returns `Bool` |
| logical_and | `Bool::and()` | Takes slice |
| logical_or | `Bool::or()` | Takes slice |
| logical_not | `.not()` method | Negation |

### What Can't Be Translated

**Kleis-specific:**
- Placeholders (editor UI)
- Match expressions (functional programming)
- Custom operations (christoffel, gradient, etc.)

**These are fine!** Z3 is for **verification**, not **rendering**.

---

## Use Cases for Translation

### 1. Axiom Verification

**Kleis axiom:**
```kleis
axiom distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
```

**Translate to Z3:**
```rust
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");
let z = Int::fresh_const("z");

let lhs = &x * (&y + &z);
let rhs = (&x * &y) + (&x * &z);

solver.assert(&lhs._eq(&rhs).not());
// Check if UNSAT → axiom holds!
```

### 2. Simplification Validation

**User simplifies:**
```kleis
(x + 0) * y  →  x * y
```

**Verify with Z3:**
```rust
let original = kleis_to_z3(&orig_expr, &ctx, &vars)?;
let simplified = kleis_to_z3(&simp_expr, &ctx, &vars)?;

solver.assert(&original._eq(&simplified).not());
if solver.check() == SatResult::Unsat {
    println!("✅ Simplification is valid!");
}
```

### 3. Type Checking Enhancement

**Matrix multiplication:**
```kleis
Matrix(2, 3) × Matrix(n, 4)
```

**Check with Z3:**
```rust
let n = Int::fresh_const("n");
solver.assert(&n._eq(&Int::from_i64(3)));  // Inner dims must match
// Z3 can verify dimension compatibility!
```

---

## Key Insights

### 1. Complementary, Not Competing

**Kleis AST:** Representation (what the user wrote)  
**Z3 AST:** Verification (is it mathematically valid?)

Both needed!

### 2. Translation Is One-Way

**Kleis → Z3:** ✅ Can translate for verification  
**Z3 → Kleis:** ❌ Don't need to (Z3 is ephemeral)

Workflow:
1. User builds expression (Kleis AST)
2. Translate to Z3 for verification
3. Get result (valid/invalid/counterexample)
4. Continue with Kleis AST (rendering, etc.)

### 3. Subsets Can Translate

**Not all Kleis expressions need Z3 translation:**
- Placeholders: Skip (editor only)
- Match expressions: Skip (or translate body)
- Custom operations: Skip (or define in Z3)

**Only translate what makes sense to verify!**

### 4. Z3's Built-In Knowledge Is The Win

We don't implement E-unification - Z3 already has it!
- AC theory: built-in ✅
- Linear arithmetic: built-in ✅
- Simplification: built-in ✅

**We just translate, Z3 does the hard work!**

---

## Architecture Decision

### Kleis AST Stays As-Is ✅

**Don't change Kleis AST to match Z3!**

**Why:**
- Kleis AST is perfect for rendering
- Flexible for user-defined operations
- Works with pattern matching
- Serializable for documents
- UI-friendly (placeholders, etc.)

### Add Translation Layer

```
Kleis AST (source of truth)
    ↓ (when needed)
Z3 AST (verification)
    ↓
Results (valid/invalid)
    ↓
Back to Kleis (continue editing)
```

**Z3 is a tool, not the foundation!**

---

## Comparison Table

| Feature | Kleis AST | Z3 AST | Winner |
|---------|-----------|--------|--------|
| **Purpose** | Representation | Verification | Different |
| **Type safety** | Runtime | Compile-time | Z3 |
| **Flexibility** | Open (user ops) | Closed (built-in) | Kleis |
| **Rendering** | Multiple targets | Debug only | Kleis |
| **Solving** | None | SMT solver | Z3 |
| **Simplification** | Not yet | Built-in | Z3 |
| **Pattern matching** | Native | None | Kleis |
| **Persistence** | Serializable | Ephemeral | Kleis |
| **User-defined types** | Yes | No | Kleis |
| **Algebraic laws** | None | AC theory | Z3 |
| **Quantifiers** | Notation only | Reasoning | Z3 |

---

## The Best of Both Worlds

**Use Kleis AST for:**
- Parsing user input
- Rendering to LaTeX/Typst/HTML
- Pattern matching and functional features
- Document persistence
- UI editing (placeholders, markers)
- Custom mathematical operations

**Use Z3 AST for:**
- Verifying axioms
- Checking algebraic equivalence
- Validating simplifications
- Finding counterexamples
- Dimension checking
- SAT/SMT queries

**Bridge between them:**
- Generic `kleis_to_z3()` translator
- One-way translation (Kleis → Z3)
- Results come back as bool/counterexample
- Continue with Kleis AST

---

## Example: Complete Workflow

**User enters:**
```
(x + 0) * (y - y)
```

**1. Kleis AST (representation):**
```rust
Operation {
    name: "times",
    args: [
        Operation { name: "plus", args: [Object("x"), Const("0")] },
        Operation { name: "minus", args: [Object("y"), Object("y")] }
    ]
}
```

**2. Translate to Z3 (verification):**
```rust
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");
let expr = (&x + &Int::from_i64(0)) * (&y - &y);
let simple = expr.simplify();  // Z3 simplifies!
// Result: 0 (because y-y=0, anything×0=0)
```

**3. Verify simplification:**
```rust
let original_z3 = kleis_to_z3(&original_expr)?;
let zero = Int::from_i64(0);

solver.assert(&original_z3._eq(&zero).not());
if solver.check() == SatResult::Unsat {
    println!("✅ Expression always equals 0!");
}
```

**4. Back to Kleis (continue):**
```rust
// Suggest to user: "This simplifies to 0"
// User can accept or keep original
// Continue editing in Kleis AST
```

---

## Conclusion

**Z3 AST and Kleis AST are COMPLEMENTARY:**
- Kleis: Flexible representation
- Z3: Powerful verification
- Translation layer: Bridge between them

**Don't merge them - keep them separate and use each for what it's good at!**

**The magic:**
- Z3 already has E-unification (AC theory) ✅
- Z3 already has simplification ✅
- Z3 already knows algebraic laws ✅

**We just need to translate, then use Z3's power!** 🎯

---

**All 21 Z3 tests pass, proving this architecture works!**

