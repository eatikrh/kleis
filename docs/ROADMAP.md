# Kleis Roadmap

*Extracted from NEXT_SESSION.md on Feb 26, 2026.*
*Items that are still relevant for future development.*

---

## Open Bugs

## 🐛 Known Issue: Exponentiation Operator (^) for Complex Numbers

**Discovered:** December 19, 2024  
**Status:** Open - workaround available

### The Problem

The `^` operator for exponentiation crashes or misbehaves with complex numbers in Z3:

```
λ> :sat ∃(z : ℂ). z^2 = -1
thread 'main' panicked at vendor/z3/src/func_decl.rs:224:18
```

**Also:** Superscript notation `z²` is parsed as a variable name, not `power(z, 2)`.

### Root Cause

- `translate_power` in `src/solvers/z3/translators/arithmetic.rs` only handles `Int^Int`
- For all other types, it falls back to uninterpreted function
- **No `complex_power` implementation exists** (unlike `complex_add`, `complex_mul`, etc.)
- `power` is NOT in `DISPATCHABLE_OPS` in `type_mapping.rs`

### Workaround

Use explicit multiplication:
```kleis
:sat ∃(z : ℂ). z * z = complex(-1, 0)
✅ Satisfiable: z = -i
```

### Fix Options

**Option 1: Add `complex_power` to Z3 backend**
- For integer exponents, expand to repeated multiplication: `z^3 = z * z * z`
- Add to `translate_operation` dispatch in `backend.rs`

**Option 2: Axiomatic definition** (preferred, aligns with Kleis philosophy)
```kleis
structure Power(T) over Monoid(T) {
    operation power : T × ℕ → T
    axiom power_zero : ∀(x : T). power(x, 0) = e
    axiom power_succ : ∀(x : T)(n : ℕ). power(x, n + 1) = x * power(x, n)
}

implements Power(ℂ) {
    operation power = complex_power  // Rust builtin
}
```

**Option 3: Parser enhancement**
- Lex `z²` (superscript) as `power(z, 2)`
- Desugar `x^n` to `power(x, n)` before type inference

### Priority

**Medium** - workaround exists (`z * z`), but syntax should work eventually.

### Files to Modify

- `src/solvers/z3/backend.rs` - Add `complex_power` case
- `src/solvers/z3/translators/arithmetic.rs` - Update `translate_power`
- `src/solvers/z3/type_mapping.rs` - Add `power` to `DISPATCHABLE_OPS` if using type dispatch

---

## 🔴 Tech Debt: Hardcoded Type Annotation Parsing

### Problem

`type_inference.rs` has `parse_type_annotation()` (lines 1017-1080) that parses type 
annotation strings like `"Matrix(3, 3, ℝ)"`. It **hardcodes** type names instead of 
querying the registry.

**Location:** `src/type_inference.rs` lines 1017-1080

```rust
fn parse_type_annotation(&self, annotation: &str) -> Type {
    match annotation.trim() {
        "ℝ" | "Real" => return Type::scalar(),    // Hardcoded
        "ℂ" | "Complex" => /* hardcoded */,
        // ...
    }
    
    match type_name {
        "Matrix" => /* hardcoded parsing */,       // Should query registry
        "Vector" => /* hardcoded parsing */,       // Should query registry
        // ...
    }
}
```

Also: convenience constructors `Type::matrix()`, `Type::pmatrix()`, etc. at lines 2087-2131.

### Impact

- Works fine because Matrix/Vector ARE defined in stdlib
- But violates ADR-016 (operations/types should come from structures, not Rust)
- Adding new parametric types requires Rust code changes

### Solution

Query registry for known parametric types:
1. Get list of parametric structures from registry
2. Parse type args based on structure's parameter list
3. Remove hardcoded type name matching

### Workaround

Works today - just not self-hosting. Low priority.

---

## Future Enhancements

## 🔧 FUTURE: Set Operators as Infix Syntax (Grammar v0.97)

**Added:** January 5, 2026

### Current State

Set operators require function-call syntax:
```kleis
in_set(x, S)        // instead of x ∈ S
subset(A, B)        // instead of A ⊆ B
proper_subset(A, B) // instead of A ⊂ B
```

### Proposed Enhancement

Add infix operators to the grammar:

```ebnf
binaryOp ::= ... existing operators ...
           | "∈" | "∉" | "⊆" | "⊂"  // NEW: Set operators
```

### Implementation

1. **Grammar v0.97**: Add set operators to `binaryOp` production
2. **Parser**: Add to `try_parse_infix_operator()`:
   ```rust
   '∈' => Some("in_set".to_string()),
   '∉' => Some("not_in_set".to_string()),
   '⊆' => Some("subset".to_string()),
   '⊂' => Some("proper_subset".to_string()),
   ```
3. **Precedence**: Same as comparison operators (level 6)
4. **Tests**: Add to `tests/test_operators.kleis`
5. **Documentation**: Update `grammar.md`

### Why This Was Removed

Set operators existed in grammar v03-v08 but were removed. Possibly:
- Z3 set theory was added later than the grammar
- Function-call syntax was simpler for initial implementation
- No pressing need at the time

### Effort Estimate

~1 hour: Simple parser addition, well-defined semantics, existing function implementations.

---

## 🔧 FUTURE: User-Implementable Unicode Operators

**Added:** January 7, 2026

### Current Limitation

Unicode operators like `•`, `⊗`, `⊕`, `∘` are **syntactic only**:
- They parse as infix: `a • b` → `•(a, b)`
- But they **cannot be computed** — they stay symbolic forever
- Users cannot define implementations for them

### Why Users Can't Implement Them

| Approach | Result |
|----------|--------|
| `define •(a, b) = a * b` | ❌ Parse error — `•` not a valid identifier |
| `operation • : T × T → T` in structure | ❌ Parse error — same reason |
| Define `dot` and hope `•` uses it | ❌ No connection — `•` stays symbolic |

### Proposed Solutions

**Option 1: Add Built-in Aliases**

Add common operators to `evaluator.rs`:
```rust
"•" | "dot" | "inner" => self.builtin_dot_product(args),
"∘" | "compose" => self.builtin_compose(args),
"⊗" | "tensor" => self.builtin_tensor_product(args),
```

**Option 2: Operator Mapping in Structures**

Allow structures to map operators to implementations:
```kleis
structure VectorSpace(V) {
    operation dot : V × V → ℝ
    infix • = dot   // NEW: operator alias
}
```

**Option 3: Parser-Level Rewriting**

Make parser rewrite `a • b` → `dot(a, b)` based on registered mappings.

### Current Documentation

The operators appendix now correctly states these limitations. See:
`docs/manual/src/appendix/operators.md` — "Custom Mathematical Operators" section.

### Effort Estimate

- Option 1: ~2 hours (add builtins, implement semantics)
- Option 2: ~4 hours (parser + evaluator changes)
- Option 3: ~6 hours (complex parser rewriting)


## 🔧 FUTURE: Externalize Configuration (Ports, Timeouts)

**Added:** January 5, 2026

### Current State

Several configuration values are hardcoded in Rust:

| Setting | Current Value | Location |
|---------|---------------|----------|
| Z3 solver timeout | 30 seconds | `src/solvers/z3/backend.rs` |
| LSP server port | stdio | `src/bin/kleis.rs` |
| DAP server port | dynamic | `src/bin/kleis.rs` |
| Equation Editor server port | 3000 | `src/bin/server.rs` |

### Proposed Solution

1. **Configuration file** (e.g., `kleis.toml` or `.kleisrc`):
   ```toml
   [solver]
   backend = "z3"           # future: "cvc5", "lean", etc.
   timeout_seconds = 30
   
   [server]
   port = 3000
   
   [lsp]
   trace = "off"            # "off", "messages", "verbose"
   ```

2. **Environment variable overrides**:
   ```bash
   KLEIS_Z3_TIMEOUT=60 kleis test file.kleis
   KLEIS_SERVER_PORT=8080 kleis server
   ```

3. **Command-line flags**:
   ```bash
   kleis test --timeout 60 file.kleis
   kleis server --port 8080
   ```

### Why This Matters

- **Z3 timeout**: Some proofs need more time; users can't adjust
- **Ports**: Docker/Kubernetes deployments may require specific ports
- **Future solvers**: When adding CVC5, Lean, etc., need backend selection
- **Development vs Production**: Different settings for different environments

### Implementation Plan

1. Add `kleis.toml` parser (use `toml` crate)
2. Check env vars with `std::env::var()`
3. CLI flags via `clap` (already used)
4. Priority: CLI > env > config file > defaults

### Effort Estimate

~2-3 hours for basic implementation.

---

## 🔧 FUTURE: Code Organization & Technical Debt

### Overview

The codebase has grown significantly and needs modularization. Key issues:

1. **~~`evaluator.rs` is 9,325 lines~~** — **DONE** via PR #137. Split into `src/evaluator/` with 7 modules (mod.rs, builtins.rs, helpers.rs, lapack.rs, plotting.rs, substitute.rs, verification.rs)
2. **Hardcoded types in Rust** — violates ADR-016 (types should come from stdlib)
3. **57 TODOs/FIXMEs across src/** — need systematic resolution
4. **Deprecated/duplicate code** — `src/dap.rs` marked deprecated

### Priority 1: Modularize `evaluator.rs`

**DONE (PR #137).** Evaluator split into 7 modules under `src/evaluator/`.

**Proposed structure:**
```
src/evaluator/
├── mod.rs           # Re-exports, Evaluator struct (~200 lines)
├── core.rs          # Loading, bindings, basic operations (~400 lines)
├── eval.rs          # Main evaluation logic (~400 lines)
├── substitution.rs  # substitute(), pattern matching (~150 lines)
├── lambda.rs        # β-reduction, α-conversion, free vars (~800 lines)
├── examples.rs      # Example blocks, assert, Z3 verification (~400 lines)
├── concrete.rs      # Concrete evaluation (~200 lines)
└── builtins/
    ├── mod.rs       # apply_builtin dispatch (~200 lines)
    ├── arithmetic.rs # +, -, *, /, pow, etc. (~600 lines)
    ├── string.rs    # String operations (~300 lines)
    ├── list.rs      # List operations (~400 lines)
    ├── plotting.rs  # diagram, plot, bar, etc. (~1500 lines!)
    ├── typst.rs     # export_typst, render_to_typst (~600 lines)
    └── matrix.rs    # Matrix operations (~400 lines)
```

**Benefit:** No file over 1500 lines. Clear separation of concerns.

### Priority 2: Remove Hardcoded Types (ADR-016)

| Hardcoded in Rust | Should Be in stdlib |
|-------------------|---------------------|
| `Type::matrix()`, `Type::pmatrix()`, etc. | `stdlib/types.kleis` |
| `"Scalar"`, `"Vector"`, `"Complex"` literals | Data registry lookups |
| Matrix dimension checking in Rust | Structure axioms in Kleis |

**Files affected:**
- `src/type_inference.rs` (35 occurrences of Scalar/Matrix/Vector)
- `src/type_context.rs` (8 occurrences)

**Target:** Type inference queries registry, doesn't hardcode type names.

### Priority 3: Clean Up TODOs

| File | TODOs | Notable Issues |
|------|-------|----------------|
| `src/math_layout/mod.rs` | 11 | Layout system incomplete |
| `src/render.rs` | 8 | Rendering edge cases |
| `src/type_inference.rs` | 7 | ADR-016 migration notes |
| `src/math_layout/typst_adapter.rs` | 7 | Typst integration |
| `src/bin/server.rs` | 4 | Server cleanup |

**Total:** 57 TODOs across 19 files

### Priority 4: Remove Deprecated Code

| File | Status | Action |
|------|--------|--------|
| `src/dap.rs` | Marked `#[deprecated]` | Delete after confirming `kleis server` works |
| `src/bin/debug.rs` vs `src/bin/commands/debug.rs` | Duplicate? | Consolidate |

### Estimated Effort

| Task | Sessions |
|------|----------|
| Modularize evaluator.rs | 2-3 |
| Remove hardcoded types | 1-2 |
| Clean up TODOs | 1-2 |
| Remove deprecated code | 0.5 |
| **Total** | **5-8 sessions** |

### Related ADRs

- **ADR-016:** Operations in Structures (types from stdlib, not Rust)
- **ADR-014:** Hindley-Milner Type System
- **ADR-021:** Data types (future)

---

## 🎯 NEXT: Transcendental Functions (sin, cos, log, exp, etc.)

### The Gap

Kleis currently handles:
- ✅ Verification (Z3)
- ✅ Numerical calculations (arithmetic)
- ✅ Plotting (Lilaq/Typst)

But lacks **transcendental functions** for scientific computing:

```kleis
// These don't work yet:
let y = sin(x)      // ❌
let z = exp(-t)     // ❌
plot(xs, map(cos, xs))  // ❌
```

### Implementation Plan

**Use Rust's `std::f64`** — no external dependencies needed!

| Function | Rust Implementation | Notes |
|----------|---------------------|-------|
| `sin(x)` | `x.sin()` | Radians |
| `cos(x)` | `x.cos()` | Radians |
| `tan(x)` | `x.tan()` | Radians |
| `asin(x)` | `x.asin()` | Returns radians |
| `acos(x)` | `x.acos()` | Returns radians |
| `atan(x)` | `x.atan()` | Returns radians |
| `atan2(y, x)` | `y.atan2(x)` | 2-argument arctangent |
| `sinh(x)` | `x.sinh()` | Hyperbolic |
| `cosh(x)` | `x.cosh()` | Hyperbolic |
| `tanh(x)` | `x.tanh()` | Hyperbolic |
| `exp(x)` | `x.exp()` | e^x |
| `log(x)` | `x.ln()` | Natural log |
| `log10(x)` | `x.log10()` | Base-10 log |
| `log2(x)` | `x.log2()` | Base-2 log |
| `sqrt(x)` | `x.sqrt()` | Square root |
| `pow(x, y)` | `x.powf(y)` | x^y |
| `abs(x)` | `x.abs()` | Absolute value |
| `floor(x)` | `x.floor()` | Round down |
| `ceil(x)` | `x.ceil()` | Round up |
| `round(x)` | `x.round()` | Round to nearest |

**Accuracy:** All functions are IEEE 754 compliant, < 1-2 ULP accuracy (same as NumPy, MATLAB, Julia).

### Files to Modify

1. **`src/evaluator.rs`** — Add `builtin_sin`, `builtin_cos`, etc.
2. **`stdlib/prelude.kleis`** — Declare operations with types:
   ```kleis
   operation sin : ℝ → ℝ
   operation cos : ℝ → ℝ
   operation exp : ℝ → ℝ
   operation log : ℝ → ℝ
   // etc.
   ```
3. **`examples/math/transcendental.kleis`** — Test examples
4. **`docs/manual/`** — Document in reference

### Example Usage (After Implementation)

```kleis
example "damped oscillation" {
    let t = [0, 0.1, 0.2, 0.3, 0.4, 0.5]
    let y = [exp(negate(0)) * cos(0),
             exp(negate(0.1)) * cos(0.1),
             exp(negate(0.2)) * cos(0.2),
             exp(negate(0.3)) * cos(0.3),
             exp(negate(0.4)) * cos(0.4),
             exp(negate(0.5)) * cos(0.5)]
    plot(t, y, "Damped Oscillation")
}
```

### Priority

**High** — Needed for scientific plotting and numerical examples.

---

## 🎯 FUTURE: Big Operators as Unified Binders (Dec 28, 2024)

### Unifying Slogan

**Σ/Π/∫/lim are big operators. Big operators are binders.**

### Binder Structure

Every binder has:
1. **Bound variable** — the index/parameter being abstracted
2. **Domain specification** — how it ranges (set, interval, filter, approach)
3. **Body** — the expression being computed
4. **Semantics** — algebra/topology that gives meaning

### Current Binders in Kleis

| Binder | Syntax | Bound Var | Domain | Body |
|--------|--------|-----------|--------|------|
| `∀` | `∀(x : T). P(x)` | x | type T | P(x) |
| `∃` | `∃(x : T). P(x)` | x | type T | P(x) |
| `λ` | `λ x . e` | x | implicit | e |
| `let` | `let x = v in e` | x | singleton | e |
| `match` | `match e { P => b }` | pattern vars | scrutinee | b |

### Proposed Big Operator Syntax (Future)

Harmonize with existing binders:

```kleis
// Sum: Σ(i : ℤ, 1 ≤ i ≤ n). f(i)
// Prod: Π(i : ℤ, i ∈ S). g(i)
// Integral: ∫(x : ℝ, a ≤ x ≤ b). h(x) dx
// Limit: lim(x → a). f(x)
```

Or simpler prefix form:
```kleis
Σ(i = 1..n) f(i)
Π(i ∈ S) g(i)
∫(x ∈ [a,b]) h(x)
lim(x → a) f(x)
```

### ✅ IMPLEMENTED: Sugar Syntax (Dec 28, 2024)

**Parser now supports Unicode big operator syntax:**

```kleis
// Summation: Σ(from, to, body) → sum_bounds(body, from, to)
Σ(1, n, λ i . f(i))

// Product: Π(from, to, body) → prod_bounds(body, from, to)
Π(1, n, λ i . f(i))

// Integral: ∫(lower, upper, body, var) → int_bounds(body, lower, upper, var)
∫(0, 1, λ x . x * x, x)

// Limit: lim(var, target, body) → lim(body, var, target)
lim(x, 0, sin(x) / x)
```

**Also supports simple prefix forms:**
```kleis
Σx    // → Sum(x)
∫f    // → Integrate(f)
```

### Kleis Renderer (Round-Trip)

The Kleis renderer outputs parseable syntax:
- `sum_bounds(body, from, to)` → `Σ(from, to, body)`
- `prod_bounds(body, from, to)` → `Π(from, to, body)`
- `int_bounds(body, lower, upper, var)` → `∫(lower, upper, body, var)`
- `lim(body, var, target)` → `lim(var, target, body)`

### 🏗️ ARCHITECTURE: BigOp as First-Class Binders (v2.0 Target)

**ChatGPT's Design Proposal:**

```rust
// Dedicated AST node (like Quantifier)
Expression::BigOp {
    op: BigOpKind,              // Sum | Prod | Integral | Limit | Sup | Inf
    binder: (String, Option<TypeExpr>),  // (var, type)
    domain: DomainExpr,         // Range(a,b) | Set(S) | Filter(P) | Approach(x→a)
    body: Box<Expression>,
    annotations: HashMap<String, Expression>,  // measure, differential, etc.
}

// DomainExpr variants
enum DomainExpr {
    Range { from: Expr, to: Expr },           // 1..n, a..b
    Set(Expr),                                // S, {1,2,3}
    Filter { domain: Expr, predicate: Expr }, // i ∈ ℤ where P(i)
    Approach { var: String, target: Expr },   // x → a, x → ∞
}
```

**Why This Is More Correct:**

1. **Binder visibility** — Bound variable explicit in AST, not hidden inside lambda
2. **Type checking** — Clear bound variable type annotation
3. **Pattern matching** — Match on `BigOp` variant, not function name
4. **Rendering** — Direct access to binder for pretty-printing (subscript/superscript)
5. **Alpha-equivalence** — Proper variable renaming without lambda inspection
6. **Domain clarity** — Range vs Set vs Filter vs Approach are distinct

**Comparison:**

| Aspect | Current (v0.95) | ChatGPT (v2.0 target) |
|--------|-----------------|----------------------|
| Implementation | ✅ Done, works now | Requires AST + parser + evaluator changes |
| Binder visibility | Hidden inside lambda | Explicit in AST |
| Type checking | Lambda body inference | Clear bound variable type |
| Rendering | Reconstruct from lambda | Direct access to binder |
| Pattern matching | Match on function name | Match on BigOp variant |
| Semantic clarity | "Function with lambda" | "Binder-like operator" |

**Current Approach (v0.95) — Pragmatic Stepping Stone:**

- ✅ Works now
- ✅ Integrates with existing parser/evaluator
- ✅ Can be refactored later without breaking user code
- ✅ Surface syntax (`Σ(1, n, body)`) stays the same

**Recommendation:**

Document ChatGPT's design as the "proper" architecture for v2.0. The current
implementation is a pragmatic stepping stone that:
1. Validates the surface syntax design
2. Provides working semantics for users
3. Can be upgraded to first-class binders when resources allow

**Migration Path:**

1. v0.95 (current): Functions + lambdas, `Σ(from, to, body)` syntax
2. v2.0 (future): `Expression::BigOp` AST node, same surface syntax
3. Users: No code changes required — surface syntax unchanged

### Z3 Limitation

Z3 is first-order — cannot quantify over functions. Higher-order axioms are **specifications**, not Z3-verifiable. See `stdlib/bigops.kleis` for documented semantics.

### Files Created/Updated

- `stdlib/bigops.kleis` — Big operator declarations with equation-editor-compatible names
- `examples/calculus/sum_examples.kleis` — 4 tests
- `examples/calculus/integral_examples.kleis` — 3 tests
- `src/kleis_parser.rs` — Parser for Σ, Π, ∫, lim
- `src/render.rs` — Updated Kleis templates for round-trip

**7/7 examples pass.**

### Parser Tests Added

- `test_parse_sum_sugar` — Σ(1, 10, x) → sum_bounds(x, 1, 10)
- `test_parse_product_sugar` — Π(1, n, f(i)) → prod_bounds(...)
- `test_parse_integral_sugar` — ∫(0, 1, x, x) → int_bounds(x, 0, 1, x)
- `test_parse_limit_sugar` — lim(x, 0, f(x)) → lim(f(x), x, 0)
- `test_parse_sum_prefix` — Σx → Sum(x)
- `test_parse_integral_prefix` — ∫x → Integrate(x)

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

## 🎯 Equation Editor: Set Type Templates

The Equation Editor should support Set operations with proper type inference.

**Current Status:**
- ✅ REPL can infer Set types: `insert(5, empty_set)` → `Set(Int)`
- ❌ Equation Editor doesn't have Set operation templates

**Needed templates:**
- `in_set(x, S)` - membership test (x ∈ S)
- `union(A, B)` - set union (A ∪ B)
- `intersect(A, B)` - intersection (A ∩ B)
- `difference(A, B)` - difference (A \ B)
- `subset(A, B)` - subset test (A ⊆ B)
- `empty_set` - empty set (∅)
- `singleton(x)` - singleton set ({x})
- `insert(x, S)` - add element

**Files to modify:**
- `static/index.html` - Add buttons to palette
- `std_template_lib/sets.kleist` - Template definitions
- `src/render_editor.rs` - Rendering templates
- `patternfly-editor/` - PatternFly integration

**Leave for future branch:** `feature/equation-editor-sets`

---

## Bourbaki Compliance Roadmap

## 🎯 PRIORITY: Bourbaki Compliance Roadmap

Based on capability assessment (Dec 19, 2025), here's what's needed to increase Bourbaki coverage from ~15-20% to higher levels.

### Priority 1: Parser Fixes ✅ COMPLETE (Grammar v0.9)

**Status: DONE** (Dec 22, 2025) - All parser issues resolved!

| Issue | Status | Verified By |
|-------|--------|-------------|
| **∀ inside ∧** | ✅ Works | `tests/grammar_v09_test.rs::test_quantifier_in_conjunction` |
| **Function types in quantifiers** | ✅ Works | `tests/grammar_v09_test.rs::test_function_type_with_nested_quantifier` |
| **→ as implication** | ✅ Works | Used throughout axiom definitions |
| **ε-δ limit definition** | ✅ Works | `tests/grammar_v09_test.rs::test_epsilon_delta_limit` |

**Impact:** Full ε-δ analysis definitions, nested quantifiers, and function types in quantifiers all work.

**Next Steps:** Priorities 2-5 are pure Kleis stdlib code (no more Rust changes needed).

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

## Lessons & Limitations

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
- `examples/meta-programming/lisp_parser.kleis` - Add verify form
- `src/evaluator.rs` - May need Z3 integration
- `docs/manual/src/appendix/lisp-interpreter.md` - Update with new code

---

## Ideas

## 💡 IDEA: Paper Review Rules as Kleis Policy

**Origin:** The POT arXiv paper went through ~6 rounds of peer review (by the author), each catching substantive issues. The review process surfaced implicit quality rules that could be formalized as Kleis policies — enabling Z3-backed verification of scientific papers.

**Rules that emerged from the review:**

1. **Dimensional Consistency** — Every axiom must be scale-free. No magic constants (e.g., `r > 1`) that depend on unit choice.
2. **Physical Honesty** — If a plot contradicts intuition, the paper must explain why. Don't hide assumptions (e.g., uniform-density core).
3. **Ontological Precision** — Distinguish measured quantities (baryonic mass) from computed quantities (projected mass). Never conflate the two.
4. **Concrete Grounding** — Abstract axioms are necessary but not sufficient. Numerical results must trace to an explicit kernel/function.
5. **Counter-theory Acknowledgment** — If replacing Theory X, acknowledge whether the dominant theory (e.g., GR) also fails in the same regime.
6. **Presentation as Rigor** — Formatting errors are conceptual errors in disguise. No tooling artifacts in output.
7. **Intellectual Sovereignty** — Don't bind to labels ("open source") that constrain future decisions.

**Potential formalization:** These are axiomatizable as structural checks on a document AST. Z3 could verify, e.g., that every axiom used in a numerical section has a concrete instantiation, or that every claim about Theory X references a counter-theory. A `paper_review_policy.kleis` could enforce these during paper generation.


## IDEA: Data-Driven Policy Action Registry

**Problem:** Adding a new policy action type (e.g., `git_pr_create`) currently
requires editing Rust code in `src/mcp/policy.rs` and `src/mcp/protocol.rs` —
the action-to-function dispatch and the JSON schema enum are both hardcoded.
This means every new action type is a code change + recompile.

**Inspiration:** The Z3 backend uses `src/solvers/z3/capabilities.toml` to
declare its capabilities declaratively. The policy system should follow the
same pattern.

**Proposed design:** A `policy_actions.toml` (or similar) that declares:
- Action name (e.g., `git_pr_create`)
- Parameters and their types (e.g., `branch: String`)
- Which Kleis functions to call (`check_git_pr_create`, `before_git_pr_create`)
- The mapping from JSON fields to function arguments

The Rust dispatcher would read this file at startup and dynamically build the
action routing — no code changes needed to add new action types. The policy
`.kleis` file already defines the `check_*` / `before_*` functions; the TOML
just tells the MCP server how to wire JSON requests to those functions.

**Concrete trigger:** We wanted to add a `git_pr_create` rule (agent must
inform user before creating a PR) but realized it required touching Rust code
in three places. That's wrong — policy should be entirely in Kleis + config.

---

*See `docs/CAPABILITY_ASSESSMENT.md` for full analysis.*

