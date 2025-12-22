# Next Session Notes

**Last Updated:** December 21, 2024

---

## ✅ DONE: Matrix Arithmetic Type Inference Fix

**Problem:** `minus(Matrix, Matrix)` was incorrectly returning `Scalar` type.

**Root Cause:** The hardcoded type hierarchy in `type_inference.rs` (lines 1401-1489) checked for Complex, Rational, Scalar, Int, Nat but **never checked for Matrix**. If nothing matched, it defaulted to Scalar.

**Fix:** Added Matrix handling before the default fallback (lines 1474-1485):
```rust
// Check for Matrix - if either arg is Matrix, return that Matrix type
if let Type::Data { constructor, .. } = &t1 {
    if constructor == "Matrix" {
        return Ok(t1.clone());
    }
}
// ... similar for t2
```

**Future Work (TODO #10):** Per ADR-016, all ~400 lines of hardcoded type logic should move to `stdlib/prelude.kleis` structures and be queried from the registry. Current approach works but isn't self-hosting.

---

## ✅ DONE: Equation Editor `let x =` Template

Added `let_simple` template for 2-argument let bindings:
- Button in "Logic & Set Theory" palette
- Template in `std_template_lib/logic.kleist`
- Implemented for ℝ, Matrix, and Bool types in `stdlib/prelude.kleis`

---

## 🎯 Equation Editor: Add `let x =` Template

The equation editor needs a template for let bindings:

```
let x = [value] in [body]
```

This allows users to define local variables in the visual editor.

**Files to modify:**
- `static/index.html` - Add button/template
- Template structure: `Let { pattern: "x", value: Placeholder, body: Placeholder }`

---

## ⚠️ Program Synthesis: Documented Limitation

**The Dream:** `spec → Z3 → program`

**The Reality:** Z3 cannot synthesize recursive programs from grammar. We tried and documented the failure in `feature/program-synthesis` branch.

**What works:**
- Sketch-based synthesis (human provides template, Z3 fills parameters)
- Bounded verification (sort 2-3 elements)
- LLM proposes, Z3 verifies

**Architecture going forward:**
```
LLM → proposes program → Z3 → verifies properties
                              ✓ or counterexample
```

See `docs/vision/VERIFIED_SOFTWARE_DREAM.md` (in abandoned branch) for full analysis.

---

## ✅ DONE: LISP Interpreter in Kleis

- ✅ Parser (recursive descent, S-expressions)
- ✅ Evaluator (arithmetic, lambda, let, letrec)  
- ✅ Recursion: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix
- ✅ `:eval` command for concrete execution
- ❌ `(verify ...)` form — **CANCELLED** (program synthesis doesn't work as envisioned)

---

## ✅ DONE: Type Inference for User-Defined Types

Fixed Dec 21, 2024:
- `:load` now registers data types with TypeChecker
- `:type VNum(42)` → `VNum(Scalar)` ✅
- `:type SAtom("hello")` → `SAtom("hello")` ✅

---

## 📝 Key Learnings (Dec 21, 2024)

1. **Kleis is Turing complete** — proved by implementing LISP interpreter
2. **Data constructors create concrete objects** — not just symbols
3. **Z3 cannot unroll recursion over unbounded ADTs** — fundamental limitation
4. **`:eval` enables execution** — concrete evaluation in Rust
5. **Verification ≠ Synthesis** — Z3 verifies, LLMs synthesize

---

## 🚫 CANCELLED: Implement `(verify ...)` in LISP Interpreter

**Reason:** The program synthesis vision didn't work. Z3 can't evaluate LISP programs symbolically, so `(verify ...)` can't use Z3 the way we hoped.

### What We Have
- ✅ LISP parser (recursive descent, S-expressions)
- ✅ LISP evaluator (arithmetic, comparisons, lambda, let, letrec)
- ✅ Recursion working: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix

### What We Need to Design
1. **How does `(verify expr)` call Z3?**
   - Option A: Translate LISP → Kleis expression → Z3
   - Option B: Direct LISP → Z3 (bypass Kleis translation)
   - Option C: Add Z3 access to Rust evaluator as a built-in

2. **What syntax for quantifiers?**
   - `(forall (x) (= (+ x 0) x))` - LISP-style
   - How to specify types for quantified variables?

3. **Return value on failure?**
   - `VBool(false)` vs `VSym("Counterexample: x = 42")`

### Why This Matters
See `docs/vision/VERIFIED_SOFTWARE_VISION.md` — this enables:
- Programs with embedded proofs
- Design-by-contract with verification
- The path to "correct by construction" software

### Files to Modify
- `docs/grammar/lisp_parser.kleis` - Add verify form
- `src/evaluator.rs` - May need Z3 integration
- `docs/manual/src/appendix/lisp-interpreter.md` - Update with new code

---

## 🎯 PRIORITY: Bourbaki Compliance Roadmap

Based on capability assessment (Dec 19, 2025), here's what's needed to increase Bourbaki coverage from ~15-20% to higher levels.

### Priority 1: Parser Fixes (THE Critical Blocker) 🔧

**⚠️ This constitutes Grammar v0.9** - see `docs/grammar/kleis_grammar_v09.md` (to be created).

**This is the ONLY Rust change needed.** Once fixed, Priorities 2-5 are pure Kleis stdlib code.

| Issue | Current | Target | Effort |
|-------|---------|--------|--------|
| **∀ inside ∧** | `(y > 0) ∧ (∀(x). ...)` fails | Should parse | 1-2 days |
| **Function types in quantifiers** | `∀(f : ℝ → ℝ). ...` fails | Should parse | 1-2 days |
| **→ as implication** | Only `where` works for preconditions | Support `P → Q` | 1 day |

**Impact:** Enables full ε-δ analysis definitions, nested quantifiers.

#### Exact Parser Changes Required

**File:** `src/kleis_parser.rs`

**Change 1: Allow quantifiers as expression operands**

Current behavior: The expression parser treats `∀` and `∃` as statement-level constructs only.

```rust
// In parse_primary() or parse_expression():
// Currently fails when ∀ appears after ∧

// NEEDED: When parsing RHS of ∧/∨/→, allow:
//   - ∀(var : Type). body
//   - ∃(var : Type). body
// as valid primary expressions
```

**Change 2: Parse function types in type annotations**

Current behavior: Type annotations only accept simple types like `ℝ`, `ℕ`, `Set(T)`.

```rust
// In parse_type_annotation():
// Currently: ℝ, ℕ, Set(T), Vector(n, T)
// NEEDED: ℝ → ℝ, (ℝ × ℝ) → ℝ, etc.

// Grammar addition:
// type_annotation ::= simple_type | simple_type '→' type_annotation
```

**Test cases to pass after fix:**
```kleis
// Test 1: Quantifier inside conjunction
structure Test1 {
    axiom nested: (x > 0) ∧ (∀(y : ℝ). y > 0)
}

// Test 2: Function type in quantifier
structure Test2 {
    axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)
}

// Test 3: Epsilon-delta (the real goal)
structure Limits {
    axiom epsilon_delta: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
}
```

### Priority 2: Set Theory in stdlib (Foundation) 📚

Set(T) exists but operations need defining:

```kleis
// Add to stdlib/sets.kleis:
structure SetTheory(X) {
    operation (⊆) : Set(X) × Set(X) → Bool
    operation (∪) : Set(X) × Set(X) → Set(X)
    operation (∩) : Set(X) × Set(X) → Set(X)
    operation 𝒫 : Set(X) → Set(Set(X))
    element ∅ : Set(X)
    
    axiom subset_def: ∀(A B : Set(X)). A ⊆ B ↔ ∀(x : X). in_set(x, A) → in_set(x, B)
    axiom union_def: ∀(A B : Set(X), x : X). in_set(x, A ∪ B) ↔ in_set(x, A) ∨ in_set(x, B)
    axiom power_set_def: ∀(S A : Set(X)). in_set(A, 𝒫(S)) ↔ A ⊆ S
}
```

**Impact:** Enables Bourbaki Vol I (Set Theory foundations).

### Priority 3: Topology in stdlib 🌐

Now verified to be expressible:

```kleis
// Add to stdlib/topology.kleis:
structure TopologicalSpace(X) {
    element tau : Set(Set(X))
    
    axiom empty_open: in_set(∅, tau)
    axiom full_open: in_set(X, tau)
    axiom union_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(union(U, V), tau)
    axiom intersection_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(intersect(U, V), tau)
}

structure Continuous(X, Y) over TopologicalSpace(X), TopologicalSpace(Y) {
    operation f : X → Y
    axiom continuity: ∀(V : Set(Y)). in_set(V, tau_Y) → in_set(preimage(f, V), tau_X)
}
```

**Impact:** Enables Bourbaki Vol III (Topology).

### Priority 4: Analysis Structures 📈

```kleis
// Add to stdlib/analysis.kleis:
structure MetricSpace(X) {
    operation d : X × X → ℝ
    
    axiom non_negative: ∀(x y : X). d(x, y) >= 0
    axiom identity: ∀(x y : X). d(x, y) = 0 ↔ x = y
    axiom symmetry: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}

structure Limit {
    // Requires parser fix for nested quantifiers
    axiom epsilon_delta: ∀(L a : ℝ, epsilon : ℝ) where epsilon > 0.
        ∃(delta : ℝ). delta > 0
}
```

**Impact:** Enables Bourbaki Vol IV (Analysis), after parser fixes.

### Priority 5: ZFC Axioms (Long-term) 🏛️

```kleis
// Add to stdlib/foundations/zfc.kleis:
structure ZFC {
    // Extensionality
    axiom extensionality: ∀(A B : Set). (∀(x). in_set(x, A) ↔ in_set(x, B)) → A = B
    
    // Pairing
    axiom pairing: ∀(a b). ∃(c : Set). in_set(a, c) ∧ in_set(b, c)
    
    // Union
    axiom union: ∀(F : Set(Set)). ∃(U : Set). ∀(x). in_set(x, U) ↔ ∃(A : Set). in_set(A, F) ∧ in_set(x, A)
    
    // Power Set
    axiom power: ∀(A : Set). ∃(P : Set). ∀(B : Set). in_set(B, P) ↔ B ⊆ A
    
    // Infinity (requires ordinals)
    // axiom infinity: ...
}
```

**Impact:** Full foundational rigor, but Z3 verification may struggle with some axioms.

---

## ⚠️ Z3 Capabilities (Clarified Dec 19, 2025)

**Z3 CAN verify (no Kleis implementation needed):**
- Arithmetic: `∀(n : ℕ). n + 0 = n` ✅
- Algebra: `∀(a b : ℝ). (a-b)*(a+b) = a²-b²` ✅
- Logic: De Morgan, distributivity ✅
- Most Bourbaki-style axioms about ℝ, ℂ, topology ✅

**Z3 struggles with:**

| Limitation | Example | Status |
|------------|---------|--------|
| **Structural induction** | `length(xs ++ ys) = length(xs) + length(ys)` | May timeout |
| **Limits/Convergence** | ε-δ proofs with nested quantifiers | May timeout |
| **Type-level arithmetic** | `Vec(m+n)` from `Vec(m) ++ Vec(n)` | Not expressible |

**Key insight:** Bourbaki is mostly continuous math (ℝ, ℂ, topology) where Z3 works well. Structural induction on lists/trees is rare in Bourbaki.

---

## ✅ Recently Completed

### Operator Overloading (Dec 19, 2025)
- Natural arithmetic: `3 + 4*i = complex(3, 4)` ✅
- Type-directed lowering working
- 17 integration tests

### Capability Assessment (Dec 19, 2025)
- Verified Kleis capabilities against Bourbaki
- Found more works than expected (~15-20% not 5%)
- Documented real limitations

---

## 📊 Current Stats

| Metric | Value |
|--------|-------|
| Tests | 663+ passing |
| Commits | 840+ |
| ADRs | 23 |
| Grammar | v0.8 |
| Unique Cloners | 505+ |
| Bourbaki Coverage | ~15-20% (axiomatic) |

---

## 🏗️ Architecture Notes

### Operator Overloading Pipeline

```
Parser → Type Inference → Lowering → Z3 Backend
                              ↓
              Rewrites: plus(ℂ, ℂ) → complex_add
                        times(ℝ, ℂ) → complex_mul(lift, _)
```

### Bourbaki Coverage Path

```
Current: Basic Algebra (Groups, Rings, Fields, Vector Spaces)
    ↓ Priority 1-2 (parser + set theory)
Next: Set Theory foundations
    ↓ Priority 3
Next: Topology (open sets, continuity)
    ↓ Priority 4
Next: Analysis (limits, metric spaces)
    ↓ Priority 5
Long-term: ZFC foundations
    ↓ New backend
Ultimate: Induction, transfinite, category theory
```

---

*See `docs/CAPABILITY_ASSESSMENT.md` for full analysis.*
