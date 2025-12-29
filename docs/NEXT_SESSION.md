# Next Session Notes

**Last Updated:** December 29, 2024

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

## ✅ DONE: Bitvector Theory Examples (Dec 27, 2024)

### Summary

Created `examples/bitvectors/` directory with comprehensive bitvector theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `bv_types.kleis` | Type/operation declarations for 8-bit bitvectors |
| `bv_axioms.kleis` | Formal axioms (bitwise logic, arithmetic, zero/ones, comparisons, shifts) |
| `bv_examples.kleis` | Axiom verification tests (39 examples) |

### Operations Supported

**Bitwise Logic:**
- `bvand`, `bvor`, `bvxor`, `bvnot`

**Arithmetic (modular):**
- `bvadd`, `bvsub`, `bvmul`, `bvneg`
- `bvudiv`, `bvurem` (unsigned division/remainder)
- `bvsdiv` (signed division)

**Shifts:**
- `bvshl` (left shift)
- `bvlshr` (logical right shift)
- `bvashr` (arithmetic right shift)

**Comparisons:**
- `bvult`, `bvule` (unsigned)
- `bvslt`, `bvsle` (signed)

**Constants:**
- `bv_zero` (0x00)
- `bv_ones` (0xFF)

### Results

**39/39 examples pass** using Z3's built-in decidable bitvector theory (QF_BV).

---

## ✅ DONE: String Theory Examples (Dec 27, 2024)

### Summary

Created `examples/strings/` directory with comprehensive string theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `string_types.kleis` | Type/operation declarations for strings |
| `string_axioms.kleis` | Formal axioms (basics, concatenation, containment, substring, indexOf, conversion) |
| `string_examples.kleis` | Full axiom verification tests (20 examples) |
| `string_simple_examples.kleis` | Direct Z3 string operation tests (20 examples) |

### Operations Supported

- `concat(s, t)` - String concatenation
- `strlen(s)` - String length
- `empty_string` - Empty string constant ""
- `contains(s, sub)` - Substring containment
- `hasPrefix(s, p)` - Prefix check
- `hasSuffix(s, suf)` - Suffix check
- `substr(s, start, len)` - Substring extraction
- `indexOf(s, sub, start)` - Find index of substring
- `replace(s, old, new)` - Replace first occurrence
- `charAt(s, i)` - Get character at index
- `strToInt(s)` / `intToStr(n)` - String-integer conversion

### Results

**40/40 examples pass** using Z3's built-in decidable string theory (QF_SLIA).

---

## ✅ DONE: Set Theory Examples (Dec 27, 2024)

### Summary

Created `examples/sets/` directory with comprehensive set theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `set_types.kleis` | Type/operation declarations for sets |
| `set_axioms.kleis` | Formal axioms (membership, operations, algebra laws, subsets) |
| `set_empty_examples.kleis` | Empty set behavior tests (10 examples) |
| `set_examples.kleis` | Full axiom verification tests (22 examples) |
| `set_simple_examples.kleis` | Direct Z3 set operation tests (23 examples) |

### Key Fixes

1. **Added `iff` operator handling** - `↔` (biconditional) now maps to Z3 boolean equality
2. **Added `dynamic_to_set()` helper** - Properly converts Dynamic set variables to Z3 Set type
3. **Added `empty_set` special case** - Recognizes `empty_set` as a nullary operation returning the empty set

### Results

**55/55 examples pass** using Z3's built-in decidable set theory.

---

## ✅ DONE: Z3 Built-in Arithmetic Mapping (Dec 27, 2024)

### Summary

Fixed critical performance issue where Z3 was hanging on quantified axioms. The root cause was using **uninterpreted functions** for arithmetic operations instead of Z3's **built-in decidable arithmetic**.

### Problem

Kleis files defined operations like `rat_add`, `rat_mul`, `flow_add`, etc. These were being translated to Z3 as **uninterpreted functions**:

```smt2
; SLOW - Uninterpreted function
(forall ((a Real) (b Real)) (= (rat_mul a b) (rat_mul b a)))
```

With quantifiers, Z3 uses E-matching to instantiate universally quantified axioms. This can be:
- Slow (exponential search)
- Incomplete (may return "unknown")
- Prone to matching loops

### Solution

Map common arithmetic operation names to Z3's **built-in arithmetic**:

```smt2
; FAST - Built-in multiplication
(forall ((a Real) (b Real)) (= (* a b) (* b a)))
```

Z3's built-in Real/Integer arithmetic is **decidable**, so quantified formulas complete instantly.

### Operations Mapped

| User Operation | Z3 Built-in |
|---------------|-------------|
| `plus`, `add`, `rat_add` | `+` |
| `minus`, `subtract`, `rat_sub` | `-` |
| `times`, `multiply`, `rat_mul` | `*` |
| `negate`, `rat_neg` | unary `-` |
| `rat_inv`, `inv`, `reciprocal` | `1/x` |
| `rat_div`, `divide` | `/` |
| `rat_lt`, `rat_gt`, `rat_le`, `rat_ge` | `<`, `>`, `<=`, `>=` |

### Results

- `examples/rationals/rational_examples.kleis`: 18/18 pass (was hanging)
- `examples/ontology/revised/*.kleis`: All complete quickly
- Z3 verification is now practical for field axioms

### Key Insight

**Use built-in arithmetic when possible, uninterpreted functions only when necessary.**

- Field theory (rationals, reals): Use built-in
- Custom abstract algebra (groups over user-defined types): Use uninterpreted
- Mixed: Map operations to built-in where types match

---

## ✅ DONE: Import Registry Fix (Dec 27, 2024)

### Summary

Fixed critical issue where `kleis test` command wasn't loading imported structures and their operations into the registry for Z3 verification.

### Problem

When running `kleis test` on a file that imports another file:
1. The imported file's structures were NOT loaded into the evaluator
2. Operations from imported structures weren't in the registry
3. Z3 fell back to declaring all operations as untyped (`flow_add → Int`)
4. Implements blocks weren't loaded

### Fixes

1. **Added `load_imports_recursive` in `kleis.rs`**
   - `run_test` now recursively loads all imports before the main file
   - Imported structures, operations, and data types are now in the evaluator

2. **Added `implements_blocks` to Evaluator**
   - New field: `implements_blocks: Vec<ImplementsDef>`
   - Loaded in `load_program_with_file`
   - Added to registry in `build_registry`

3. **Registry now complete**

   | TopLevel Variant | Loaded in Evaluator | Added to Registry |
   |-----------------|---------------------|-------------------|
   | `StructureDef` | ✅ | ✅ |
   | `OperationDecl` | ✅ | ✅ |
   | `DataDef` | ✅ | ✅ |
   | `FunctionDef` | ✅ | N/A |
   | `ImplementsDef` | ✅ | ✅ |
   | `TypeAlias` | ✅ | ✅ |

### Tests

Added `tests/import_registry_test.rs` with 5 tests:
- `test_imported_structures_in_registry`
- `test_standalone_structures_no_import_needed`
- `test_structure_registry_has_operations`
- `test_multiple_structures_operations_accessible`
- `test_implements_blocks_in_registry`

---

## ✅ DONE: Z3 Backend Major Fixes (Dec 27, 2024)

### Summary

Fixed multiple critical bugs in Z3 axiom verification:

1. **Quantifier Translation Bug (CRITICAL)**
   - `translate_quantifier` was NOT wrapping axiom bodies in `forall_const`
   - Axioms like `∀(G a b). apply_kernel(G, a) = apply_kernel(G, b) → equiv(G, a, b)` were being asserted as just the implication body WITHOUT the quantifier
   - Z3 treated quantified variables as free constants, making all reflexivity proofs fail
   - **Fix:** `translate_quantifier` now uses `z3::ast::forall_const()` and `exists_const()` properly

2. **Typed Function Declarations**
   - Previously: All uninterpreted functions declared with `Int` domain
   - Now: Looks up operation signatures from registry and declares with proper sorts
   - `flow_smul : ℂ × Flow → Flow` now declares `Complex × Int → Int` in Z3
   - Added `get_operation_signature()` to `StructureRegistry`
   - Added top-level operations storage to `StructureRegistry` and `Evaluator`

3. **Complex Type Bound Variables**
   - `fresh_complex_const` was constructing Complex values instead of bound variables
   - This caused sort mismatches when applying functions in quantified contexts
   - **Fix:** Use `Dynamic::fresh_const(name, &complex_sort)` for proper Z3 bound variables

4. **AssertResult::Unknown Handling (CRITICAL)**
   - Previously: Unknown was treated as Passed (optimistic!)
   - **Fix:** Unknown now correctly fails with "Assertion unknown: ..." message
   - Z3 timeouts and inconclusive results are no longer falsely reported as success

5. **Z3 Timeout**
   - Added 30-second timeout to prevent infinite hangs on complex quantified axioms
   - Set via `solver.set_params()` with `timeout: 30000`

### Future Enhancement: Configurable Timeout

The 30-second timeout is currently hardcoded. It should be configurable per-assertion:

```kleis
// Option A: Assert-level timeout
assert reflexivity: equiv(f, f) timeout 60s
assert quick_check: x = x timeout 1s
assert default: y = y  // uses default 30s

// Option B: Example block timeout
example "complex proofs" timeout 120s {
    assert reflexivity: equiv(f, f)
}
```

**Implementation would require:**
1. Grammar change to support `timeout Ns` clause
2. Parser update to parse timeout value
3. AST field for optional timeout
4. Evaluator to pass timeout to backend
5. Backend method to set timeout per-assertion

**Priority:** Low (current 30s default works for most cases)

### Future Enhancement: Parameterized Structures with Structure Dependencies

Current limitation: Structures can only extend other structures, not take them as parameters.

**Proposed syntax:**
```kleis
structure AdmissibleKernel(
  G        : GreenKernel,
  FlowAlg  : FlowAlgebra,
  FieldAlg : FieldR4Algebra
) {
  // Local shorthands using dot notation
  define flow_add  = FlowAlg.flow_add
  define field_add = FieldAlg.field_add

  // Axioms referencing parameter structures
  axiom linearity:
    ∀(α : ℂ, f g : FieldR4).
      G.apply(field_add(f,g)) = field_add(G.apply(f), G.apply(g))
}
```

**Features needed:**
1. Structure parameters with structure types (`G : GreenKernel`)
2. Dot notation for accessing operations (`FlowAlg.flow_add`)
3. Local `define` inside structure body
4. Parameter structures as first-class values

**Implementation would require:**
1. Grammar extension for structure parameters with types
2. Parser support for dot notation in expressions
3. Name resolution for structure member access
4. Type checker updates for structure-typed parameters

**Priority:** Medium (enables cleaner POT formalization)

### Test Results

- All 755 unit tests pass
- POT examples verify correctly (when Z3 completes in time)
- Examples that timeout are correctly reported as "unknown" (failed)

---

## ✅ DONE: Z3 Enhanced Registry Integration (Dec 27, 2024)

### Summary

Extended Z3 backend to leverage `data_types` and `type_aliases` from the StructureRegistry for enhanced verification capabilities.

### What Was Implemented

1. **Z3 ADT Declaration from Registry**
   - New method: `declare_data_types_from_registry()`
   - Converts Kleis `data` declarations into Z3 algebraic data types
   - Automatic constructor distinctness: `Mass ≠ EM ≠ Spin ≠ Color`
   - New field: `declared_data_types: HashMap<String, DatatypeSort>`

2. **Type Alias Resolution**
   - New method: `resolve_type_alias(&TypeExpr) -> TypeExpr`
   - Resolves type aliases before Z3 sort mapping
   - Supports parameterized alias substitution

3. **Enhanced `type_name_to_sort`**
   - Now checks declared data types first
   - Then checks type aliases from registry
   - Falls back to built-in primitives

4. **Registry Iterator Methods**
   - Added `data_types()` iterator
   - Added `type_aliases()` iterator
   - Added `data_type_count()` and `type_alias_count()`

### Complete Registry Support Table

| TopLevel Variant | Loaded in Evaluator | Added to Registry | Used by Z3 |
|-----------------|---------------------|-------------------|------------|
| `StructureDef` | ✅ | ✅ | ✅ (operations, axioms) |
| `OperationDecl` | ✅ | ✅ | ✅ (typed declarations) |
| `DataDef` | ✅ | ✅ | ✅ (Z3 ADT, distinctness) |
| `FunctionDef` | ✅ | N/A | N/A |
| `ImplementsDef` | ✅ | ✅ | ⏳ (verification planned) |
| `TypeAlias` | ✅ | ✅ | ✅ (sort resolution) |

### Benefits of Z3 Using Registry Data Types

1. **Automatic Constructor Distinctness** - Z3 knows `Mass ≠ EM` without explicit axioms
2. **Exhaustiveness Checking** - Z3 can verify pattern matching covers all cases
3. **Accessor Functions** - Fields accessible in Z3 reasoning
4. **No Hardcoding** - User-defined data types get first-class Z3 support
5. **Inductive Reasoning** - For recursive types like `List(T)`

### Benefits of Z3 Using Type Aliases

1. **Consistent Sort Resolution** - `type Scalar = ℝ` always resolves to Real sort
2. **Semantic Type Names** - Write axioms using domain-meaningful names
3. **Parameterized Resolution** - `type Matrix(m, n) = ...` can be resolved

### New Tests

Added to `tests/z3_backend_fixes_test.rs`:
- `test_data_types_registered_in_registry`
- `test_type_aliases_registered_in_registry`
- `test_z3_declares_data_types_from_registry` (with axiom-verification feature)
- `test_z3_resolves_type_aliases` (with axiom-verification feature)
- `test_z3_data_type_constructor_distinctness`
- `test_registry_iteration_methods`

### Documentation

Updated ADR-022 (Z3 Integration) with new "Enhanced Registry Integration" section documenting:
- Benefits of data types → Z3 ADTs
- Benefits of type alias resolution
- Implementation plan and impact assessment

---

## 🎯 POT Formalization: Admissible Kernel Class (Next Steps)

### Current Status (Dec 27, 2024)

The POT formalization in `examples/ontology/revised/` is now **airtight**:
- ✅ Option A refactor complete: all projection is kernel-parameterized
- ✅ `apply_kernel(G, ψ)` is the canonical operation (no implicit kernel)
- ✅ `equiv(G, a, b)` and `in_nullspace(G, a)` are definitional (bidirectional)
- ✅ Field extensionality via `field_at` + `field_ext`
- ✅ No "hidden context" leakage

### Next Move: Minimal Admissible Kernel Class (v0)

Pin down constraints on valid kernels that are:
1. Expressible in Kleis today (no integrals needed)
2. Not so strong it hard-codes known physics
3. Strong enough to generate falsifiable constraints

#### 1) Algebraic Admissibility

**(K1) Linearity over flows** — superposition must survive projection:
```kleis
axiom kernel_linear_add: ∀(G : GreenKernel, a b : Flow).
    apply_kernel(G, flow_add(a, b)) = field_add(apply_kernel(G, a), apply_kernel(G, b))

axiom kernel_linear_smul: ∀(G : GreenKernel, α : ℂ, a : Flow).
    apply_kernel(G, flow_smul(α, a)) = field_smul(α, apply_kernel(G, a))
```

**(K2) Zero preservation** — zero flow projects to zero field:
```kleis
axiom kernel_zero: ∀(G : GreenKernel).
    apply_kernel(G, flow_zero) = field_zero
```

**Status:** K1 already implemented (`project_lin_add`, `project_lin_smul`). K2 needs adding.

#### 2) Observational Equivalence Compatibility

**(K3) Equivalence respects kernel action** — already have via `equiv_elim`/`equiv_intro`.

#### 3) Regularity / Locality (Weak, Falsifiable)

**(K4) Event-local determinacy via probes**:
```kleis
// Residues depend only on local probe at the event
operation probe : GreenKernel × Flow × Event → ℝ

axiom residue_local: ∀(G : GreenKernel, ψ1 ψ2 : Flow, e : Event, c : Channel).
    probe(G, ψ1, e) = probe(G, ψ2, e) → residue(apply_kernel(G, ψ1), e, c) = residue(apply_kernel(G, ψ2), e, c)
```

This keeps "physics local-ish" without hardcoding PDEs.

#### 4) Dimensional Well-Typedness

**(K5) Units constraint** — residues must output quantities with declared units:
```kleis
// Mass channel returns Quantity(kg), Charge returns Quantity(C), etc.
// Prevents "mass in bananas" from being a legal model
```

This requires deciding if Kleis should have a units system (future work).

### Falsifiable Claim Patterns

Once `AdmissibleKernel(G)` exists:

**Pattern A: Invariants**
```kleis
// For all admissible kernels, conserved channels satisfy constraint C
∀(G : AdmissibleKernel). conservation_law(G) → constraint(G)
```

**Pattern B: Geometry Emergence**
```kleis
// For all admissible kernels with symmetry S, induced metric has property P
∀(G : AdmissibleKernel). has_symmetry(G, S) → metric_property(apply_kernel(G, _), P)
```

These are falsifiable because P can be tested against observation.

### Files

- `examples/ontology/revised/pot_core_kernel_projection.kleis` — core formalization
- `examples/ontology/revised/pot_foundations_kernel_projection.kleis` — postulates
- `examples/ontology/revised/spacetime_type_kernel_projection.kleis` — spacetime types

---

---

## ✅ DONE: DAP Debugger Fully Working! (Dec 26, 2024)

### What Works
- ✅ Cross-file debugging (VS Code opens imported files)
- ✅ Correct line numbers for ALL operation types (arithmetic, logical, comparison)
- ✅ Breakpoints work in both main and imported files
- ✅ Variables panel shows AST expressions (symbolic representation!)
- ✅ Stack frames tracked correctly
- ✅ Step over, step into, step out all work
- ✅ **assert() uses Z3 for symbolic verification!**

### Key Insight: DAP as a Window to Kleis Internals
The debugger shows variables as **AST expressions**, not evaluated values:
```
doubled = Operation { name: "plus", args: [Object("x"), Object("x")], span: ... }
x = Const("10")
```

This is **exactly right** for a symbolic mathematics system! Variables hold
symbolic expressions that can be passed to Z3 for verification.

### Fixes Applied (Dec 26, 2024)
1. **Skip expressions without spans** - No more line 1 spurious stops
2. **Parser span capture at START** - Fixed 8 parsing functions to capture span
   before parsing, not after (parse_arithmetic, parse_term, parse_factor,
   parse_comparison, parse_conjunction, parse_disjunction, parse_implication,
   parse_biconditional, parse_where_term)
3. **Fixed double pop_frame bug** - Removed redundant pop_frame() call
4. **Custom operator spans** - Fixed parse_where_term

### Future Ideas

#### 1. Eval Command in Debug Panel
Add ability to evaluate an AST expression to a concrete value during debugging.
The infrastructure exists (`evaluator.eval()`).

#### 2. Extend `example` Block Grammar
Current grammar only allows: `let`, `assert`, expressions.

**Could add:**
```kleis
example "test" {
    define local_fn(x) = x + 1   // Local function definition
    let y = local_fn(5)
    assert(y = 6)
}
```

**Pros:** Self-contained test cases, useful for testing helpers
**Cons:** `example` is for testing not defining; functions can be top-level

#### 3. ✅ Wire assert() to Z3 - DONE! (Dec 26, 2024)
**IMPLEMENTED!** `assert()` in example blocks now uses Z3 for symbolic verification:

```kleis
structure CommutativeRing(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(a b : R). a + b = b + a
}

example "test commutativity" {
    assert(x + y = y + x)  // ✅ Z3 verifies this using the commutativity axiom!
}
```

**How it works:**
1. `eval_assert()` checks if expressions are symbolic (`is_symbolic()`)
2. If symbolic → calls `verify_with_z3()` using `AxiomVerifier`
3. Z3 loads structure axioms and verifies/disproves the assertion
4. Results: `Verified`, `Disproved { counterexample }`, or `Unknown`

**Test cases added:**
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom  
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

---

---

## ✅ DONE: Type Promotion (Lift) Implemented (Dec 26, 2024)

### What Was Fixed

The type checker now correctly promotes types through the `Promotes` structure.

**Before:** `:type 1 + sin(x)` → `Int` ❌
**After:** `:type 1 + sin(x)` → `Scalar` ✅

### Bugs Fixed

1. **OperationRegistry.merge() missing fields**
   - Added merge for `structure_extends` and `type_promotions`
   - Without this, promotions registered in stdlib weren't available to type checker

2. **Unicode type names not normalized when registering**
   - `implements Promotes(ℕ, ℤ)` was registering as `("ℕ", "ℤ")`
   - But `has_promotion` and `find_common_supertype` normalize to `("Nat", "Int")`
   - Fix: Normalize in `register_implements` before storing

3. **Top-level operations not registered**
   - Operations like `operation sin : ℝ → ℝ` were ignored (TODO stub)
   - Added `toplevel_operation_types` to `OperationRegistry`
   - Type inference now queries these for function return types

4. **Added type_expr_to_type helper**
   - Converts `TypeExpr` to `Type` for return type extraction
   - Handles Function, Named, Parametric, Product, ForAll, DimExpr

### Test Results

All 8 type promotion tests pass:
- `:type sin(x) = Scalar` ✅ (was `Var(TypeVar(0))`)
- `:type 1 + sin(x) = Scalar` ✅ (was `Int`)
- `:type (1 + sin(x)) / 2 = Scalar` ✅ (was `Int`)
- `:type 1 + 3.14 = Scalar` ✅
- Promotions registered: Nat→Int, Int→Scalar, etc. ✅

### Files Modified
- `src/type_context.rs` - Major fixes to registry and type lookup
- `tests/type_promotion_test.rs` - New test file with 8 tests

---

## ✅ DONE: First-Class Function Types Implemented (Dec 26, 2024)

### What Was Implemented

Added `Type::Function(Box<Type>, Box<Type>)` variant to the type system:

```rust
pub enum Type {
    // ...
    /// Function type: A → B
    Function(Box<Type>, Box<Type>),
    // ...
}
```

### Files Modified
- `src/type_inference.rs` - Added Function variant, updated unify(), occurs(), apply()
- `src/type_context.rs` - Updated type_expr_to_type() and interpret_toplevel_operation_type()
- `tests/function_type_test.rs` - New test file with 9 tests

### What Works Now
- **Display:** `sin : Scalar → Scalar` displays correctly with arrow
- **Unification:** Function types unify properly (same domains/codomains)
- **Occurs check:** Prevents infinite types like `α = α → ℝ`
- **Higher-order functions:** Can represent `(T → U) → List(T) → List(U)`
- **Curried functions:** Can represent `ℝ → ℝ → ℝ`

### ✅ Product Types - DONE

Product types now have proper support with `Type::Product(Vec<Type>)` variant.

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

## ✅ DONE: N-ary Product Types (Grammar v0.94)

Parser now supports n-ary product types:

```kleis
operation mass_at : GreenKernel × Flow × Event → ℝ  // ✅ Works!
```

**Implementation:** `src/kleis_parser.rs` lines 1609-1635
- `parse_product_type()` is right-associative
- `A × B × C × D` flattens into `TypeExpr::Product([A, B, C, D])`
- `×` binds tighter than `→`

**✅ DONE:** `Type::Product(Vec<Type>)` variant added - full product type support in type inference

---

## ✅ DONE: assert() Uses Z3 Verification (Dec 26, 2024)

**Implemented!** `assert()` in example blocks now uses Z3 for symbolic verification.

### Changes Made
- Added `is_symbolic()` to detect if expressions contain unbound variables
- Added `verify_with_z3()` to call `AxiomVerifier.verify_axiom()`
- Modified `eval_equality_assert()` to try Z3 when expressions are symbolic
- Added `AssertResult::Verified` and `AssertResult::Disproved` variants

### Tests Added (`tests/crossfile_debug_test.rs`)
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

### How It Works
```kleis
assert(x + y = y + x)   // ✅ Z3 verifies via commutativity axiom
assert(x + y = y + y)   // ❌ Z3 disproves: "Counterexample: y!1 -> 1, x!0 -> 0"
assert(4 = 4)           // ✅ Concrete equality (no Z3 needed)
```

---

## ✅ DONE: Thread-Safe AST Cache (ADR-025)

**See:** `docs/adr/adr-025-debugger-shared-context.md`

Implemented thread-safe AST cache shared between LSP and DAP:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Thread-Safe AST Cache                         │
│     Arc<RwLock<HashMap<PathBuf, CachedDocument>>>               │
│                                                                  │
│  CachedDocument {                                                │
│    source: String,                                               │
│    program: Option<Program>,  // The AST                         │
│    imports: HashSet<PathBuf>, // Dependencies                    │
│    dirty: bool,               // Needs re-parse?                 │
│  }                                                               │
└─────────────────────────────────────────────────────────────────┘
           ↑                              ↑
           │ write                        │ read (or write if miss)
           │                              │
    ┌──────┴───────┐               ┌──────┴───────┐
    │     LSP      │               │     DAP      │
    │  (Thread 1)  │               │  (Thread 2)  │
    │              │               │              │
    │  Evaluator   │               │  Evaluator   │
    │  (own copy)  │               │  (own copy)  │
    └──────────────┘               └──────────────┘
```

**Key features:**
- LSP updates cache when documents change
- DAP reads from cache (or parses and caches if missing/dirty)
- Cascade invalidation: dirty files propagate to dependents
- Each thread has its own `Evaluator` (because `RefCell` is not `Sync`)

---

## ✅ DONE: DAP Line Number Issues FIXED! (Dec 26, 2024)

### What Was Fixed

1. **Parser span capture at START of operations** - Fixed 8 parsing functions
2. **Skip expressions without spans** - No more line 1 spurious stops
3. **Custom operator spans** - Fixed parse_where_term

### Current State (ALL WORKING!)

| Component | Status |
|-----------|--------|
| Parser populates `SourceSpan` with file path | ✅ |
| `ExampleStatement` carries location | ✅ |
| Evaluator calls `on_eval_start()` for every expression | ✅ |
| `DapDebugHook` exists with channel-based communication | ✅ |
| DAP returns stack traces with file paths | ✅ |
| VS Code shows debugger UI | ✅ |
| DAP wires hook to evaluator | ✅ |
| Cross-file debugging (file switching) | ✅ |
| **Line numbers accurate in cross-file stepping** | ✅ FIXED! |

### Architecture (from `REPL_ENHANCEMENTS.md`)

```
┌─────────────────────────────────────────────────────────────┐
│                     kleis server                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │   LSP       │◄─►│  Shared     │◄─►│   DAP       │       │
│  │  Handler    │   │  Context    │   │  Handler    │       │
│  └─────────────┘   │ - Evaluator │   └─────────────┘       │
│                    │ - Types     │                          │
│                    │ - Structs   │                          │
│                    └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

**Key Design Points:**
- **RefCell** ensures zero overhead when not debugging (hook is `None`)
- **DapDebugHook** blocks in evaluator thread, communicates via channels
- **DapDebugController** held by DAP server, sends commands, receives events
- **DO NOT change RefCell** - it's there for a purpose!

### Implementation Plan

#### Step 1: Update `DapState` to Hold Controller

```rust
struct DapState {
    // ... existing fields ...
    
    /// Controller for channel-based communication with DebugHook
    controller: Option<DapDebugController>,
    
    /// Handle to evaluation thread
    eval_thread: Option<std::thread::JoinHandle<()>>,
    
    /// Parsed program (for finding example blocks)
    program: Option<Program>,
}
```

#### Step 2: Wire `launch` Handler

1. Parse file with `parse_kleis_program_with_file(source, canonical_path)`
2. Find first `ExampleBlock` to debug
3. Create `DapDebugHook` + `DapDebugController` via `DapDebugHook::new()`
4. Store controller in `DapState`
5. **Don't start evaluation yet** (wait for `configurationDone`)

#### Step 3: Wire `setBreakpoints` Handler

1. Create `Breakpoint { file, line, enabled: true }` for each
2. Store in `DapState.breakpoints`
3. Will be added to hook before evaluation starts

#### Step 4: Wire `configurationDone` Handler

1. Lock evaluator, set hook: `evaluator.set_debug_hook(hook)`
2. Spawn evaluation thread:
   ```rust
   thread::spawn(move || {
       evaluator.eval_example_block(&example);
       // Send terminated when done
   });
   ```
3. Wait for first `StopEvent` from `controller.event_rx`
4. Send `stopped` event to VS Code

#### Step 5: Wire Step Commands

| DAP Command | DebugAction |
|-------------|-------------|
| `next` | `StepOver` |
| `stepIn` | `StepInto` |
| `stepOut` | `StepOut` |
| `continue` | `Continue` |

1. Send via `controller.command_tx.send(action)`
2. Wait for `StopEvent` from `controller.event_rx`
3. Update `current_file` and `current_line` from event
4. Send `stopped` event to VS Code

#### Step 6: Wire `stackTrace` Handler

- Get stack from `StopEvent.stack`
- Store latest stack in `DapState`
- Return frames with `source.path` (absolute paths)

#### Step 7: Wire `variables` Handler

- Get bindings from top stack frame
- Return as DAP variables

#### Step 8: Handle Evaluation Complete

- Add `Terminated` variant to `StopEvent` (or use channel close)
- Send `terminated` event to VS Code

### Why This Works for Cross-File Debugging

The evaluator calls `on_eval_start` with whatever `SourceLocation` the AST has.
When stepping into a function from an imported file, the AST node has that file's path.
The hook receives it, checks breakpoints, sends stop event with the correct file.
**No per-construct hardcoding needed.**

---

## 🧠 CRITICAL ARCHITECTURE: SharedContext AST Cache

### The Insight

**LSP already parses every file the user has open.** It re-parses on every edit.
DAP should NOT parse files separately — it should use the SAME cached AST.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              SharedContext.documents                         │
│                                                              │
│   HashMap<PathBuf, CachedDocument>                          │
│                                                              │
│   "/path/to/main.kleis"    → AST (parsed by LSP on open)    │
│   "/path/to/helper.kleis"  → AST (parsed by LSP on open)    │
│   "/path/to/stdlib/prelude" → AST (parsed by DAP if needed) │
│                                                              │
└─────────────────────────────────────────────────────────────┘
        ↑                              ↑
   LSP updates on edit             DAP reads (parses only if missing)
```

### The Rule

1. **DAP checks cache first** before parsing any file
2. **If found** → use it (FREE, already parsed by LSP)
3. **If not found** → parse, then ADD to cache for future use
4. **Both LSP and DAP use the same cache**

### Cache Invalidation (CRITICAL)

**When a file changes, all files that IMPORT it must be evicted from cache.**

Example:
```
main.kleis imports helper.kleis
helper.kleis imports stdlib/prelude.kleis

If stdlib/prelude.kleis changes:
  → Evict helper.kleis (imports stdlib)
  → Evict main.kleis (imports helper which imports stdlib)
```

This requires **dependency tracking**:
```rust
struct CachedDocument {
    ast: Program,
    imports: Vec<PathBuf>,        // Files this doc imports
    imported_by: Vec<PathBuf>,    // Files that import this doc (reverse)
}
```

When file X changes:
1. Evict X from cache
2. For each file that imports X, recursively evict

### Performance Impact

| Without Cache | With Cache |
|---------------|------------|
| Debug start: parse file (50ms) | 0ms (already parsed) |
| Step into import: parse (50ms) | 0ms if open in editor |
| Edit during debug: parse twice | Parse once (LSP only) |

### Why This Matters

> **The user's editor IS the source of truth.**
> LSP sees what user sees. DAP uses what LSP sees.
> No stale ASTs. No duplicate parsing.

### The Algorithm (Classic Incremental Compilation)

This is the same algorithm used by `make`, `cargo`, Webpack, and TypeScript.

**1. Build Dependency Graph (on parse):**
```rust
fn on_parse(file: &Path, ast: &Program) {
    for import_path in ast.imports() {
        // Forward edge: file imports import_path
        cache[file].imports.push(import_path);
        // Reverse edge: import_path is imported_by file
        cache[import_path].imported_by.push(file);
    }
}
```

**2. Invalidation (on file change) — propagate UP the tree:**
```rust
fn invalidate(file: &Path) {
    if let Some(doc) = cache.remove(file) {
        // Recursively invalidate all dependents
        for dependent in doc.imported_by {
            invalidate(&dependent);
        }
    }
}
```

**3. Lazy Re-parse (on demand) — parse dependencies FIRST:**
```rust
fn get_ast(file: &Path) -> &Program {
    if cache.contains(file) {
        return &cache[file].ast;
    }
    
    // Parse the file
    let ast = parse(file);
    
    // Ensure all imports are in cache first (topological order)
    for import_path in ast.imports() {
        get_ast(&import_path);  // Recursive
    }
    
    // Store and return
    cache.insert(file, CachedDocument { ast, ... });
    &cache[file].ast
}
```

**Visual Example:**
```
stdlib/prelude.kleis CHANGES
         ↓ invalidate
    helper.kleis (imports stdlib) → EVICTED
         ↓ invalidate  
    main.kleis (imports helper) → EVICTED

Later, when DAP needs main.kleis:
    get_ast(main.kleis)
        → get_ast(helper.kleis)  // dependency first
            → get_ast(stdlib/prelude.kleis)  // leaf first
            ← parse stdlib, cache it
        ← parse helper, cache it
    ← parse main, cache it
```

**Key Properties:**
- Parse each file at most once per change
- Dependencies parsed before dependents (topological order)
- Lazy: only re-parse when actually needed
- Minimal work: only affected files re-parsed

### Files to Modify

| File | Changes |
|------|---------|
| `src/bin/kleis.rs` | Update `DapState`, wire handlers |
| `src/debug.rs` | Add `Terminated` event (if needed) |

### Technical Debt to Address

**1. Consolidate DAP Implementations**
- `src/dap.rs` — Library version (marked `#[deprecated]`)
- `src/bin/kleis.rs` — Used by `kleis server` (the active one)
- **Action:** Remove `src/dap.rs` after confirming `kleis server` works end-to-end

**2. Review DebugHook Implementations**
We have 3 implementations in `src/debug.rs`:
- `NoOpDebugHook` — Zero overhead when not debugging (KEEP)
- `InteractiveDebugHook` — For REPL `:debug` command (KEEP for REPL)
- `DapDebugHook` — For VS Code DAP integration (KEEP for DAP)

**Action:** After wiring is complete, review if `InteractiveDebugHook` and `DapDebugHook` can share more code or if the separation is justified.

**3. Squash Commits Before Merging**
The `feature/debugger-dap` branch has 63+ incremental commits. Before merging to `main`, squash into logical commits:
- "Add example blocks and assert to grammar (v0.93)"
- "Implement REPL :debug command"  
- "Add DAP infrastructure for VS Code debugging"
- "Add source location tracking to parser"
- "Wire DAP to evaluator with DapDebugHook"

**Command:** `git rebase -i origin/main` then squash/fixup related commits.

### Test Plan

1. Set breakpoint in `examples/debug_main.kleis` on line 8
2. Set breakpoint in `examples/debug_helper.kleis` on line 6
3. Start debugging `debug_main.kleis`
4. Should stop at line 8
5. Step over to line 11 (`let doubled = double(x)`)
6. Step into → should jump to `debug_helper.kleis` line 6
7. Step out → should return to `debug_main.kleis`

### Key Documents

1. **`docs/plans/REPL_ENHANCEMENTS.md`** — Master plan, Phase 6 (Debugging)
2. **`docs/plans/EXPRESSION_SPANS.md`** — Future: spans on all Expressions
3. **`src/debug.rs`** — DebugHook trait and DapDebugHook implementation

---

## ✅ Current Debugger Status (ALL WORKING!)

| Feature | Status |
|---------|--------|
| Launch/attach | ✅ |
| Breakpoints (set) | ✅ |
| Breakpoints (hit) | ✅ Real, wired to evaluator |
| Breakpoints in imported files | ✅ Works! |
| Step in/over/out | ✅ Real evaluation |
| Continue | ✅ Real evaluation |
| Stack trace | ✅ Correct file paths |
| Variables | ✅ Shows AST expressions |
| Cross-file (file switching) | ✅ Works |
| Cross-file (line numbers) | ✅ **FIXED!** All operations correct |
| assert() with Z3 | ✅ **NEW!** Symbolic verification |

### Files to Review

- `src/bin/kleis.rs` — Unified binary (DAP implementation here)
- `src/debug.rs` — DebugHook trait and DapDebugHook
- `src/evaluator.rs` — Calls debug hooks at key points
- `vscode-kleis/src/extension.ts` — VS Code integration

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

## ✅ DONE: LISP Interpreter Uses stdlib Ordering Operations

The LISP interpreter (`docs/grammar/lisp_parser.kleis`) already:
1. ✅ Imports `stdlib/prelude.kleis`
2. ✅ Uses `le`, `lt`, `gt`, `ge`, `eq` from stdlib `Ordered(T)` structure

No changes needed - this was already working correctly.

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
| Tests | 755+ passing |
| Commits | 850+ |
| ADRs | 25 |
| Grammar | v0.93 |
| Unique Cloners | 505+ |
| Bourbaki Coverage | ~15-20% (axiomatic) |
| DAP Debugger | ✅ Fully working! |
| Z3 Assert Verification | ✅ Implemented! |

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
