# Kleis: Symbolic vs Numeric Evaluation

**Date:** December 8, 2025  
**Question:** "Can Kleis add numbers?"  
**Short Answer:** It depends what you mean by "add"!

---

## The Reality Check

### The Question That Matters

> **"Everything a functional language can do" - can Kleis add numbers?**

Let me test:

```rust
let expr = parse_kleis("1 + 2").unwrap();
// Result: Operation { name: "plus", args: [Const("1"), Const("2")] }

// Can we evaluate this to get 3?
// ... ?
```

**The answer reveals something important about Kleis!**

---

## What Kleis Actually Does

### Kleis is SYMBOLIC

**Input:** `1 + 2`

**What Kleis does:**
1. **Parse:** ✅ → `Operation { name: "plus", args: [Const("1"), Const("2")] }`
2. **Type-check:** ✅ → `Scalar + Scalar → Scalar`
3. **Render:** ✅ → LaTeX: `1 + 2` or Unicode: `1 + 2`
4. **Evaluate to 3:** ❌ **NOT IMPLEMENTED!**

**Why?** Kleis stores constants as `Const(String)`, not `Const(f64)`!

```rust
pub enum Expression {
    Const(String),  // ← Stored as string!
    Object(String),
    Operation { name: String, args: Vec<Expression> },
    ...
}
```

---

## The Design Decision (ADR-002)

### Evaluation vs. Simplification

**From ADR-002:**

> "Kleis maintains strict separation between:
> - **Expression Evaluation**: Semantic computation (symbolic)
> - **Simplification**: Optional transformation (numeric)"

**Philosophy:**
- **Evaluation:** Preserve symbolic structure
- **Numeric computation:** Separate concern

### What This Means

**Kleis evaluates SYMBOLICALLY:**
```kleis
// Symbolic evaluation
∇(f + g) → ∇f + ∇g  // Distributes gradient
d/dx(x²) → 2x        // Symbolic differentiation

// NOT numeric:
1 + 2 → plus(1, 2)   // Stays symbolic
```

**Kleis does NOT evaluate numerically (yet):**
```kleis
1 + 2 → 3            // ❌ Not implemented
sin(π/2) → 1         // ❌ Not implemented
√4 → 2               // ❌ Not implemented
```

---

## What Kleis CAN Do

### 1. Symbolic Manipulation ✅

```kleis
// Pattern matching on expressions
define simplify(expr) = match expr {
  plus(Const("0"), e) => e
  plus(e, Const("0")) => e
  times(Const("1"), e) => e
  times(e, Const("1")) => e
  times(Const("0"), _) => Const("0")
  ...
}

// Result:
simplify(plus(x, Const("0"))) → x  // Symbolic simplification!
```

**Works:** Symbolic rewriting ✅

### 2. Type Checking ✅

```kleis
// Type inference
1 + 2     → Scalar + Scalar → Scalar  ✅
x + y     → α + α → α                 ✅
Matrix(2,3) + Matrix(2,3) → Matrix(2,3)  ✅

// Dimension checking
Matrix(2, 3) + Matrix(3, 2) → Error: dimension mismatch  ✅
```

**Works:** Type safety ✅

### 3. Pattern Matching ✅

```kleis
match Some(Const("5")) {
  None => Const("0")
  Some(x) => x
}
// Result: Const("5")  ✅ (symbolic binding works!)
```

**Works:** Pattern matching on symbolic expressions ✅

---

## What Kleis CANNOT Do (Yet)

### Numeric Evaluation ❌

```kleis
1 + 2 → ?
// Expected: Const("3")
// Actual: Operation { name: "plus", args: [Const("1"), Const("2")] }
// Status: Stays symbolic, not evaluated to 3
```

**Why not?**
- `Const(String)` not `Const(f64)`
- No numeric interpreter
- By design (ADR-002: symbolic-first)

---

## The Honest Answer

### "Can Kleis add numbers?"

**Symbolic addition:** ✅ YES
```kleis
plus(1, 2) → Operation { name: "plus", args: [...] }
// Represents the CONCEPT of addition
```

**Numeric addition:** ❌ NO (not yet)
```kleis
plus(1, 2) → 3
// Would require numeric interpreter
```

### "Everything a functional language can do?"

**More accurate statement:**

**"Everything a SYMBOLIC functional language can do"**

**Can do:**
- ✅ Pattern matching (symbolic destructuring)
- ✅ Recursion (symbolic computation)
- ✅ Type inference (symbolic types)
- ✅ ADTs (symbolic data)

**Can't do (yet):**
- ❌ Numeric computation
- ❌ String manipulation
- ❌ IO operations
- ❌ Side effects

---

## Comparison to Other Systems

### Haskell
```haskell
1 + 2  -- Evaluates to 3 (numeric)
```
**Numeric:** ✅  
**Symbolic:** ⚠️ (needs libraries)

### Mathematica
```mathematica
1 + 2  (* Evaluates to 3 *)
x + 0  (* Simplifies to x *)
D[x^2, x]  (* Symbolic: 2x *)
```
**Numeric:** ✅  
**Symbolic:** ✅

### Kleis v0.5
```kleis
1 + 2  // Stays as plus(1, 2)
x + 0  // Can simplify to x with pattern matching
d/dx(x²)  // Symbolic: not yet implemented
```
**Numeric:** ❌  
**Symbolic:** ✅ (with pattern matching!)

---

## What Would Numeric Evaluation Require?

### To Make `1 + 2 → 3` Work

**Step 1: Change Const representation** (~50 lines)
```rust
pub enum Expression {
    Const(f64),  // ← Change from String to f64
    // OR
    Const(Value),  // Where Value = enum { Num(f64), Str(String), ... }
    ...
}
```

**Step 2: Add numeric evaluator** (~200 lines)
```rust
fn eval_numeric(expr: &Expression) -> Result<f64, String> {
    match expr {
        Expression::Const(n) => Ok(n.parse()?),
        Expression::Operation { name, args } => match name.as_str() {
            "plus" => Ok(eval_numeric(&args[0])? + eval_numeric(&args[1])?),
            "times" => Ok(eval_numeric(&args[0])? * eval_numeric(&args[1])?),
            "power" => Ok(eval_numeric(&args[0])?.powf(eval_numeric(&args[1])?)),
            ...
        }
        Expression::Object(_) => Err("Cannot evaluate symbolic variable"),
        ...
    }
}
```

**Step 3: Pattern matching numeric results** (~100 lines)
```rust
// Enable: match (eval(1+1), eval(2*3)) { (2, 6) => ... }
```

**Total effort:** ~350 lines, ~4 hours

**Decision needed:** Should Kleis do this?

---

## The Design Philosophy Question

### Two Paths Forward

**Path A: Symbolic-Only (Current)**

**Kleis as:**
- Symbolic mathematics system
- Type checker
- Notation engine
- Paper authoring tool

**Evaluation means:**
- Symbolic manipulation
- Pattern matching on expressions
- Type checking

**1 + 2 stays as:** `plus(1, 2)` (symbolic)

**Advantages:**
- Clean separation (ADR-002)
- No numeric accuracy issues
- Focus on symbolic reasoning
- Simpler implementation

---

**Path B: Symbolic + Numeric (Hybrid)**

**Kleis as:**
- Symbolic mathematics system
- Numeric calculator
- Interactive computing environment
- Full CAS (Computer Algebra System)

**Evaluation means:**
- Symbolic when needed
- Numeric when possible
- Smart reduction

**1 + 2 evaluates to:** `3` (numeric)

**Advantages:**
- More like Mathematica
- Interactive experimentation
- Immediate feedback
- Practical computations

---

## What Users Expect

### For Mathematical Reasoning (Original Goal)

**Symbolic is enough:**
```kleis
// Prove properties
axiom: ∀x. x + 0 = x
// Type-check algorithms
define gaussElim(M) = ...
// Generate LaTeX
render(expr) → "x + 0"
```

**Numeric not needed!**

### For Interactive Computing (Extended Goal)

**Numeric is essential:**
```kleis
// Calculate actual values
> 1 + 2
3
> sin(π/2)
1.0
> Matrix(2, 2, 1, 0, 0, 1) × Matrix(2, 2, 2, 3, 4, 5)
Matrix(2, 2, 2, 3, 4, 5)
```

**Without this, not a calculator!**

---

## Current State: Symbolic Pattern Matching

### What DOES Work

**Pattern matching on symbolic expressions:**
```kleis
match plus(Const("1"), Const("2")) {
  plus(a, b) => times(a, b)  // Binds a=Const("1"), b=Const("2")
}
// Result: times(Const("1"), Const("2"))
```

**This works!** ✅

**Symbolic simplification via patterns:**
```kleis
define simplify(expr) = match expr {
  plus(Const("0"), e) => e
  plus(Const(a), Const(b)) => Const(toString(parseInt(a) + parseInt(b)))  // Numeric!
  ...
}
```

**Could work** if we add numeric primitives!

---

## The Corrected Statement

### What I Should Have Said

**BEFORE (too broad):**
> "Everything a functional language can do"

**AFTER (accurate):**
> "Everything a **symbolic** functional language can do"

### What Kleis v0.5 Actually Can Do

✅ **Symbolic computation:**
- Parse expressions
- Type-check expressions
- Pattern match on expressions
- Transform expressions
- Render expressions

❌ **Numeric computation (yet):**
- Evaluate `1 + 2` to `3`
- Calculate `sin(π/2)` to `1.0`
- Compute matrix products numerically

---

## The Good News

### Pattern Matching Works Symbolically!

**You CAN implement:**

```kleis
// Symbolic arithmetic
define evalSymbolic(expr) = match expr {
  plus(Const(a), Const(b)) => Const(addStrings(a, b))  // If we add this primitive
  plus(Const("0"), e) => e
  plus(e, Const("0")) => e
  times(Const("0"), _) => Const("0")
  times(Const("1"), e) => e
  ...
}
```

**Pattern matching enables symbolic evaluation!**

Even without numeric evaluation, you can:
- Simplify expressions
- Apply identities
- Transform symbolically
- Verify algebraically

---

## Roadmap to Numeric Evaluation

### Phase 1: Symbolic (Current) ✅

**What works:**
```kleis
parse("1 + 2") → plus(1, 2)    ✅
typeCheck(plus(1, 2)) → Scalar  ✅
match plus(a, b) { ... }        ✅
```

### Phase 2: Numeric Primitives (4 hours)

**Add numeric operations:**
```rust
fn eval_primitive(op: &str, args: &[f64]) -> f64 {
    match op {
        "plus" => args[0] + args[1],
        "times" => args[0] * args[1],
        ...
    }
}
```

**Result:**
```kleis
eval(plus(1, 2)) → 3.0  ✅
```

### Phase 3: Mixed Symbolic/Numeric (8 hours)

**Smart evaluation:**
```kleis
1 + 2 → 3           // Numeric when possible
x + 0 → x           // Symbolic when needed
sin(π/2) → 1.0      // Numeric constants
sin(x) → sin(x)     // Symbolic variables
```

**Result:** Mathematica-like behavior!

---

## The Honest Capabilities Assessment

### What Kleis v0.5 CAN Do

**Symbolic Computation:**
- ✅ Parse mathematical expressions
- ✅ Type-check with dimension safety
- ✅ Pattern match on structure
- ✅ Transform symbolically
- ✅ Render to LaTeX/Unicode
- ✅ Implement symbolic algorithms
- ✅ Verify type correctness

**Example:**
```kleis
// Can implement symbolic differentiation
define diff(expr, x) = match expr {
  Const(_) => Const("0")
  Var(y) if y == x => Const("1")
  plus(e1, e2) => plus(diff(e1, x), diff(e2, x))
  ...
}

// Result: diff(x², x) → 2x  (symbolic!)
```

### What Kleis v0.5 CANNOT Do (Yet)

**Numeric Computation:**
- ❌ Evaluate `1 + 2` to `3`
- ❌ Calculate `sin(3.14159)` to `~0`
- ❌ Compute matrix products numerically
- ❌ Solve equations numerically

**Example:**
```kleis
// Can represent, but not compute:
1 + 2  // Stays as plus(1, 2), not 3
```

---

## The Architecture Decision

### Why Symbolic-First?

**From ADR-002: Evaluation vs Simplification**

**Design principle:**
> "Evaluation must preserve semantics exactly, avoid unnecessary transformations"

**Rationale:**
1. **Mathematical correctness** - No floating point errors
2. **Symbolic reasoning** - Keep structure visible
3. **Type safety** - Dimensions preserved
4. **LaTeX generation** - Show original form

**Trade-off:**
- Gain: Perfect symbolic manipulation
- Cost: No numeric calculation (yet)

---

## Corrected Capability Statement

### What I Said

> "Everything a functional language can do"

### What I SHOULD Have Said

> "Everything a **symbolic-first** functional language can do:
> - Pattern matching ✅
> - Type inference ✅
> - Recursion ✅
> - ADTs ✅
> - Symbolic manipulation ✅
> 
> Plus unique features:
> - Type-level dimensions ✅
> - LaTeX rendering ✅
> - Self-hosting type system ✅
>
> Missing (by design):
> - Numeric evaluation ❌
> - String operations ❌
> - IO/Effects ❌"

---

## Turing Completeness Re-Assessment

### Is Kleis Turing Complete?

**Technically:** ⚠️ **Depends on definition**

**Can compute anything:** ✅ Yes (has recursion + pattern matching)

**Can EVALUATE anything numerically:** ❌ No (no numeric evaluator)

**Example:**
```kleis
// Can represent Fibonacci
define fib(n) = match n {
  0 => 0
  1 => 1
  _ => fib(n-1) + fib(n-2)
}

// Can type-check: ✅ fib : ℕ → ℕ

// Can evaluate symbolically: ✅
// fib(3) → fib(2) + fib(1) → fib(1) + fib(0) + fib(1) → ...

// Can evaluate numerically: ❌
// fib(3) → 2  (not implemented)
```

**Turing complete for SYMBOLIC computation!**  
**Not (yet) Turing complete for NUMERIC computation.**

---

## Comparison to Other Systems

### Mathematica / Maple
```mathematica
1 + 2           (* → 3, numeric *)
x + 0           (* → x, symbolic *)
D[x^2, x]       (* → 2x, symbolic *)
Integrate[x, x] (* → x²/2, symbolic *)
```
**Hybrid:** Numeric + Symbolic ✅

### SymPy (Python)
```python
1 + 2           # → 3, numeric
x + 0           # → x, symbolic
diff(x**2, x)   # → 2*x, symbolic
```
**Hybrid:** Numeric + Symbolic ✅

### Coq / Agda / Lean
```coq
1 + 2           (* → 3, computed by normalization *)
x + 0           (* → x, by simplification *)
```
**Proof assistants:** Symbolic evaluation via normalization

### Kleis v0.5
```kleis
1 + 2           // → plus(1, 2), stays symbolic
x + 0           // → plus(x, 0), can simplify with patterns
// diff(x², x)  // Not yet implemented
```
**Pure symbolic:** Symbolic only (for now)

---

## The Path Forward

### To Add Numeric Evaluation

**Option A: Built-in Numeric Primitives** (Simple)

Add to pattern_matcher.rs:
```rust
impl PatternMatcher {
    pub fn eval_numeric(&self, expr: &Expression) -> Result<f64, String> {
        match expr {
            Expression::Const(s) => s.parse().map_err(|_| "Not a number"),
            Expression::Operation { name, args } => match name.as_str() {
                "plus" => Ok(self.eval_numeric(&args[0])? + self.eval_numeric(&args[1])?),
                "times" => Ok(self.eval_numeric(&args[0])? * self.eval_numeric(&args[1])?),
                ...
            }
        }
    }
}
```

**Effort:** ~200 lines, 2-3 hours

---

**Option B: Numeric Evaluation via Pattern Matching** (Self-Hosted!)

Define in Kleis:
```kleis
// Evaluation rules
define evalNum : Expr → Option(ℝ)
define evalNum(expr) = match expr {
  Const(n) => parseNumber(n)
  plus(e1, e2) => match (evalNum(e1), evalNum(e2)) {
    (Some(a), Some(b)) => Some(a + b)
    _ => None
  }
  times(e1, e2) => match (evalNum(e1), evalNum(e2)) {
    (Some(a), Some(b)) => Some(a * b)
    _ => None
  }
  _ => None
}
```

**Requires:**
- Primitive operations as Kleis functions (`parseNumber`, `+`, `*`)
- Numeric type (ℝ as actual numbers)

**Effort:** ~500 lines, 8-10 hours

---

## The Design Question

### Should Kleis Add Numeric Evaluation?

**Arguments FOR:**
- More useful interactively
- More like Mathematica
- Can compute actual results
- Better user experience

**Arguments AGAINST:**
- Scope creep (Kleis is for REASONING, not computing)
- Floating point complexity
- Not the core mission
- Symbolic is the strength

**My take:** 

**Core mission:** Mathematical REASONING (symbolic)  
**Nice to have:** Mathematical COMPUTING (numeric)

**Recommendation:** Keep symbolic-first, add numeric as optional feature later.

---

## What This Means Practically

### Can You Use Kleis for Real Work? YES!

**What works well:**
- ✅ Type-checking mathematical expressions
- ✅ Verifying dimension safety
- ✅ Generating LaTeX for papers
- ✅ Defining domain-specific types
- ✅ Pattern matching on structures
- ✅ Symbolic reasoning

**What doesn't work (yet):**
- ❌ Computing numeric answers
- ❌ Interactive calculation
- ❌ Plotting results

**Use Kleis for:**
- Research papers (symbolic notation)
- Type-safe algorithm specification
- Domain modeling with types
- Proof sketches

**Don't use Kleis for:**
- Numeric computation (use Python/Julia)
- Data analysis (use R/MATLAB)
- Performance computing (use C++/Rust)

---

## The Corrected Vision

### Kleis v0.5 is:

**EXCELLENT at:**
- ✅ Symbolic functional programming
- ✅ Type-safe mathematical reasoning
- ✅ Pattern matching on types
- ✅ Self-hosting type systems
- ✅ LaTeX generation
- ✅ Domain modeling

**NOT DESIGNED for:**
- ❌ Numeric computation
- ❌ Performance computing
- ❌ Data processing
- ❌ General-purpose programming

**It's a SPECIALIZED functional language for mathematical REASONING!**

---

## Bottom Line

### The Question: "Can Kleis add numbers?"

**Parsing:** ✅ `1 + 2` parses to `plus(1, 2)`  
**Type-checking:** ✅ `Scalar + Scalar → Scalar`  
**Symbolic representation:** ✅ `plus(1, 2)` as structure  
**Numeric evaluation:** ❌ `plus(1, 2) → 3` not implemented  

### The Honest Answer

**Kleis can represent addition symbolically.**  
**Kleis cannot compute sums numerically (yet).**

This is **by design** (ADR-002: symbolic-first philosophy).

### Is This a Problem?

**For Kleis's mission (mathematical reasoning):** ✅ NO  
**For being a calculator:** ❌ YES

**But Kleis isn't trying to be a calculator!**

It's trying to be a **notation system** and **type checker** for mathematics.

For that purpose, symbolic is enough.

---

## The Takeaway

**My statement "everything a functional language can do" was imprecise.**

**More accurate:**

**"Kleis v0.5 is a complete SYMBOLIC functional language that can:**
- ✅ Pattern match on symbolic expressions
- ✅ Type-check with dimension safety
- ✅ Implement symbolic algorithms
- ✅ Self-host its type system
- ✅ Generate publication-ready LaTeX

**But it's NOT (yet) a numeric calculator."**

**For its actual mission (mathematical reasoning and notation), it's complete!**

For numeric computation? That's a different (optional) feature. 🎯

---

**Thank you for the reality check!** This is an important clarification about what Kleis is designed for.
