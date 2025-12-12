# Z3 vs Evaluator: Two Different Purposes for Functions

**Date:** December 12, 2024  
**Critical Question:** Do "functions as axioms" work when we need results?

---

## 🎯 The Key Insight: Two Separate Systems

Kleis has **TWO** systems that need to know about functions:

### 1. **Evaluator** - Symbolic Computation
**Purpose:** Expand/substitute function calls  
**Input:** `double(5)` where `define double(x) = x + x`  
**Output:** `5 + 5` (symbolic expansion)  
**Location:** `src/evaluator.rs`

### 2. **Z3 Axiom Verifier** - Logical Reasoning
**Purpose:** Prove properties about functions  
**Input:** Prove `∀x. double(x) + y = y + double(x)` (commutativity)  
**Output:** Valid/Invalid/Unknown  
**Location:** `src/axiom_verifier.rs`

---

## 🔍 Current Architecture

### Evaluator - Already Works! ✅

```rust
// src/evaluator.rs:115
pub fn apply_function(&self, name: &str, args: Vec<Expression>) -> Result<Expression, String> {
    let closure = self.functions.get(name)?;
    
    // Build substitution: param → arg
    let mut subst = HashMap::new();
    for (param, arg) in closure.params.iter().zip(args.iter()) {
        subst.insert(param.clone(), arg.clone());
    }
    
    // Substitute in body: double(5) → 5 + 5
    Ok(self.substitute(&closure.body, &subst))
}
```

**What it does:**
- `double(5)` → expands to → `5 + 5` ✅
- `minus(a, b)` → expands to → `a + negate(b)` ✅
- This is **symbolic** - doesn't compute `10`, just expands

**Where functions come from:**
- Loaded from top-level `define` statements ✅
- **NOT** loaded from structure `define` statements ❌ ← This is a gap!

### Z3 - Missing Function Support! ❌

**Currently:** Treats all operations as uninterpreted
```rust
// When Z3 sees: minus(x, y)
// It declares: (declare-fun minus (Int Int) Int)
// It knows NOTHING about the definition!
```

**Problem:**
```kleis
define (-)(x, y) = x + negate(y)

axiom test: ∀(a b : R). (a - b) + b = a

// Z3 can't prove this! It doesn't know (-) is defined in terms of (+) and negate
```

---

## 🎯 Your Question: Do Axioms Work for Getting Results?

**Answer:** Yes and No - it depends on what "results" means!

### Use Case 1: Symbolic Reasoning (Z3's Job)

**Goal:** Prove properties about functions

```kleis
structure Ring(R) {
  define (-)(x, y) = x + negate(y)
  
  axiom inverse_property: ∀(x : R). (x - x) = zero
}

// Question: Is this axiom valid?
```

**With "Functions as Axioms" approach:**
```smt
; Z3 knows:
∀(x y). minus(x, y) = plus(x, negate(y))    [from define]
∀(x). plus(x, negate(x)) = zero              [Ring axiom]

; Can Z3 prove: minus(x, x) = zero?
; Answer: YES! ✅
; 
; Proof:
;   minus(x, x) = plus(x, negate(x))    [by definition]
;               = zero                   [by Ring inverse axiom]
```

✅ **Functions as axioms WORK for logical reasoning!**

### Use Case 2: Symbolic Expansion (Evaluator's Job)

**Goal:** Expand function calls symbolically

```kleis
define double(x) = x + x

// Expression: double(y + 1)
// Want: (y + 1) + (y + 1)
```

**This is NOT Z3's job!** This is the Evaluator's job:
```rust
// Evaluator.apply_function()
double(y + 1)
→ substitute x := (y + 1) in body (x + x)
→ (y + 1) + (y + 1)
```

✅ **Evaluator handles expansion via substitution!**

### Use Case 3: Concrete Numeric Results

**Goal:** Compute `double(5) = 10`

**Neither system does this!**
- **Evaluator:** Returns `5 + 5` (symbolic)
- **Z3:** Doesn't compute, just proves

**For concrete numeric results, you'd need:**
- A separate arithmetic evaluator
- Or call built-in functions (Rust implementations)
- Or use Z3's model extraction (get a satisfying assignment)

---

## 🎯 The Real Answer to Your Question

**Q: Would functions defined as axioms handle the case where we use results?**

**A: YES, but with clarification:**

### Scenario 1: Using Function Results in Proofs ✅

```kleis
define area(w, h) = w * h

axiom rectangle: ∀(w h). area(w, h) = area(h, w)  // commutativity

// Z3 needs to:
// 1. Know what area(w, h) means
// 2. Use that to prove commutativity
```

**Functions as Axioms:** ✅ **PERFECT for this!**
```smt
∀(w h). area(w, h) = w * h    [axiom from define]
∀(w h). w * h = h * w          [arithmetic axiom]

Prove: area(w, h) = area(h, w)
→ w * h = h * w                [expand area]
→ true                         [by commutativity of *]
```

### Scenario 2: Using Function Results in Expressions ✅

```kleis
define square(x) = x * x
define sum_of_squares(a, b) = square(a) + square(b)

axiom pythagoras: ∀(a b c). c * c = sum_of_squares(a, b)
```

**Functions as Axioms:** ✅ **Works through transitive equality!**
```smt
∀x. square(x) = x * x
∀a b. sum_of_squares(a, b) = square(a) + square(b)

Z3 can chain these definitions automatically!
```

### Scenario 3: Computing Numeric Results ❌ (Different system)

```rust
// Want: double(5) → 10 (actual number)
```

**This is NOT what Z3 does!**  
**This is what the Evaluator does:**
```rust
evaluator.apply_function("double", vec![Const("5")])
→ Expression::Operation { name: "plus", args: [Const("5"), Const("5")] }
→ Still symbolic! Would need arithmetic evaluator to get 10
```

---

## 🔧 Complete Solution: Hybrid Approach

**We need BOTH systems working:**

### 1. Evaluator (Symbolic Expansion) - Already works for top-level!

```rust
// evaluator.rs - Already implemented!
pub fn apply_function(&self, name: &str, args: Vec<Expression>) -> Result<Expression, String> {
    // Substitutes parameters with arguments
    // double(5) → 5 + 5
}
```

**Gap:** Doesn't load functions from structures yet!

### 2. Z3 (Logical Reasoning) - Needs implementation!

```rust
// axiom_verifier.rs - Need to add!
fn load_function_as_axiom(&mut self, func_def: &FunctionDef) -> Result<(), String> {
    // Translates: define f(x) = body
    // To Z3: ∀x. f(x) = body
}
```

**Gap:** Doesn't handle function definitions at all yet!

---

## ✅ Answer: Yes, Axioms Work!

**Your concern:** "We might use function results for other operations"

**Examples where this works:**

### Example 1: Derived Operation Chain
```kleis
structure Ring(R) {
  operation (+) : R × R → R
  operation negate : R → R
  
  define (-)(x, y) = x + negate(y)
  define diff_squared(a, b) = (a - b) * (a - b)
  
  axiom: ∀(a b). diff_squared(a, b) >= zero
}
```

**Z3 handling:**
```smt
∀(x y). minus(x, y) = plus(x, negate(y))
∀(a b). diff_squared(a, b) = times(minus(a, b), minus(a, b))

; When proving the axiom, Z3 can expand:
diff_squared(a, b) 
= times(minus(a, b), minus(a, b))           [by definition]
= times(plus(a, negate(b)), plus(a, negate(b)))  [by definition]
```

✅ **Z3 chains definitions transitively!**

### Example 2: Function Results in Other Axioms
```kleis
define double(x) = x + x

axiom: ∀(x y). double(x + y) = double(x) + double(y)
```

**Z3 handling:**
```smt
∀x. double(x) = plus(x, x)

Prove: ∀(x y). double(plus(x, y)) = plus(double(x), double(y))

LHS: double(plus(x, y)) 
   = plus(plus(x, y), plus(x, y))        [expand double]
   
RHS: plus(double(x), double(y))
   = plus(plus(x, x), plus(y, y))        [expand double]
   = plus(x, plus(x, plus(y, y)))        [associativity]
   = plus(plus(x, y), plus(x, y))        [rearrange]
   
LHS = RHS ✅
```

---

## 🎯 Recommendation: Functions as Axioms + Evaluator

**Use BOTH approaches:**

### For Z3 (Proving):
✅ **Option 1: Functions as Axioms**
- Declare function as uninterpreted
- Assert `∀ params. f(params) = body` as axiom
- Z3 uses this for proofs

### For Evaluator (Expansion):
✅ **Substitution (already implemented!)**
- Store function body
- Substitute parameters when called
- Returns symbolic expansion

### Integration Points:

**1. In axiom_verifier.rs:**
```rust
StructureMember::FunctionDef(func_def) => {
    self.load_function_as_axiom(func_def)?;  // Z3 knows it
}
```

**2. In evaluator.rs:**
```rust
// Already works for top-level, just need to also load from structures:
impl Evaluator {
    pub fn load_structure_functions(&mut self, structure: &StructureDef) {
        for member in &structure.members {
            if let StructureMember::FunctionDef(func_def) = member {
                self.load_function_def(func_def)?;  // Evaluator knows it
            }
        }
    }
}
```

---

## ✅ Conclusion

**Your concern is valid and important!** But the answer is:

**YES, functions as axioms work for using results in other operations:**
- ✅ Z3 can chain function definitions transitively
- ✅ Z3 uses definitions automatically in proofs
- ✅ Function results can be used in other axioms/operations
- ✅ Evaluator handles symbolic expansion independently

**The key:** We need to integrate with BOTH systems:
1. **Z3** ← Functions as axioms (for proving)
2. **Evaluator** ← Functions as closures (for expansion)

Both are needed, both work, and they complement each other!

---

**Shall I implement the integration for both Z3 and Evaluator?**

