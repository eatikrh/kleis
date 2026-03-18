# Kleis Parser vs Formal Grammar Compatibility

**Date:** January 1, 2026 (updated Feb 26, 2026)  
**Formal Grammar:** Kleis v0.99 (mature)  
**Parser Implementation:** `src/kleis_parser.rs`  
**Branch:** `main`

---

## TL;DR

✅ **The Kleis grammar is mature at v0.99. The parser implements the full grammar.**

**Coverage:** Grammar is mature — all major features implemented  
**Status:** Quantifiers, pattern matching, let bindings, lambdas, data types, example blocks, imports, and/or/not, big operators (Σ/Π/∫/lim), named arguments, parametric types in quantifiers — all working  
**Tests:** 850+ passing tests

---

## What's Supported NOW (December 16, 2025)

### ✅ Fully Supported

| Feature | Grammar v0.6 | kleis_parser.rs | Status |
|---------|--------------|-----------------|--------|
| **Data types** | `data Bool = True \| False` | ✅ Complete | ✅ Works |
| **Pattern matching** | `match x { True => 1 \| False => 0 }` | ✅ Complete | ✅ Works |
| **Function definitions** | `define f(x) = x + x` | ✅ Complete | ✅ Works |
| **Type aliases** | `type Name = Type` | ✅ Complete | ✅ **NEW Dec 16!** |
| **Parenthesized types** | `(ℝ → ℝ) → ℝ` | ✅ Complete | ✅ **NEW Dec 16!** |
| **List literals** | `[1, 2, 3]` | ✅ In AST | ✅ Works |
| **Structure definitions** | `structure Matrix(m, n, T) { ... }` | ✅ Complete | ✅ Works |
| **Implements blocks** | `implements Matrix(m, n, ℝ) { ... }` | ✅ Complete | ✅ Works |
| **Function calls** | `abs(x)`, `frac(a,b)` | ✅ Complete | ✅ Works |
| **Basic arithmetic** | `a + b - c * d / e` | ✅ With precedence | ✅ Works |
| **Exponentiation** | `a ^ b` | ✅ Right-associative | ✅ Works |
| **Parentheses** | `(a + b) * c` | ✅ Grouping | ✅ Works |
| **Identifiers** | `x`, `alpha`, `myVar` | ✅ Standard | ✅ Works |
| **Numbers** | `42`, `3.14` | ✅ Integer and float | ✅ Works |
| **Universal quantifiers** | `∀(x : M). body` | ✅ Complete | ✅ **NEW!** |
| **Existential quantifiers** | `∃(x : M). body` | ✅ Complete | ✅ **NEW!** |
| **Operator symbols** | `operation (+) : R → R → R` | ✅ Complete | ✅ **NEW!** |
| **Logical operators** | `∧`, `∨`, `¬`, `⟹` | ✅ With precedence | ✅ **NEW!** |
| **Comparisons** | `=`, `<`, `>`, `≤`, `≥`, `≠` | ✅ Complete | ✅ **NEW!** |
| **Where clauses** | `implements Foo(T) where Bar(T)` | ✅ Complete | ✅ **NEW!** |
| **Nested structures** | `structure additive : Group(R) { ... }` | ✅ Complete | ✅ **NEW!** |
| **Extends keyword** | `structure Monoid(M) extends Semigroup(M)` | ✅ Complete | ✅ **NEW!** |
| **Define with operators** | `define (-)(x, y) = x + negate(y)` | ✅ Complete | ✅ Works |
| **Custom operators** | `operation (•) : S → S → S` | ✅ Complete | ✅ Works |
| **Functions in structures** | `define (-)` inside structure | ✅ Complete | ✅ **v0.6!** |
| **Comments** | `// line`, `/* block */` | ✅ Complete | ✅ Works |
| **Axiom verification** | Z3 theorem proving | ✅ Working | ✅ Works |
| **Conditionals** | `if x > 0 then x else 0` | ✅ Complete | ✅ **NEW Dec 13!** |
| **Let bindings** | `let x = 5 in x + x` | ✅ Complete | ✅ Dec 13 |
| **Typed let bindings** | `let x : ℝ = 5 in x^2` | ✅ Complete | ✅ **NEW Dec 17!** |
| **Type ascription** | `(a + b) : ℝ` | ✅ Complete | ✅ **NEW Dec 17!** |
| **Postfix operators** | `n!`, `Aᵀ`, `A†` | ✅ Complete | ✅ **NEW Dec 17!** |

**Pattern Matching Features:**
- Wildcard: `_`
- Variables: `x`, `myVar`
- Constructors: `Some(x)`, `Cons(h, t)`
- Nested patterns: `Some(Cons(x, xs))`
- Tuple patterns: `(x, y)`
- Constant patterns: `0`, `"hello"`

**Quantifier Features (NEW!):**
- Universal: `∀(x : M). body` or `forall(x : M). body`
- Existential: `∃(x : M). body` or `exists(x : M). body`
- Multiple variables: `∀(x y z : R). body`
- Type annotations: `x : M`, `y : Nat`

**Logical Operators (NEW!):**
- Conjunction: `p ∧ q` (AND)
- Disjunction: `p ∨ q` (OR)
- Negation: `¬p` (NOT, prefix)
- Implication: `p ⟹ q` (IMPLIES)
- Proper precedence chain

**Total Major Features:** ~30 supported ✅ (+3 from Dec 17: typed let bindings, type ascription, postfix operators; +2 from Dec 16: type aliases, parenthesized types; +2 from Dec 13: conditionals, let bindings; +12 from Dec 10-11 sessions: quantifiers, logic, where clauses, nested structures, extends, define operators, custom operators, comments)

---

## What's Still Missing

### ❌ Not Yet Supported

| Feature | Grammar v0.6 | Status | Priority |
|---------|--------------|--------|----------|
| ~~**Prefix operators (general)**~~ | ~~`-x`, `∇f`, `√x`~~ | ✅ **Partial Dec 13!** | ~~Medium~~ |
| ~~**Postfix operators**~~ | ~~`n!`, `Aᵀ`, `A†`~~ | ✅ **DONE Dec 17!** | ~~Medium~~ |
| ~~**Lambda expressions**~~ | ~~`λ x . x^2`~~ | ✅ **DONE Dec 17!** | ~~Low~~ |
| ~~**Let bindings**~~ | ~~`let x = 5 in x^2`~~ | ✅ **DONE Dec 13!** | ~~Low~~ |
| ~~**Conditionals**~~ | ~~`if x > 0 then x else -x`~~ | ✅ **DONE Dec 13!** | ~~Low~~ |
| **Type annotations** | `let x : ℝ`, `define f(x: ℝ)` | ✅ Complete | ✅ Dec 17 |
| **Symbolic constants** | `π`, `e`, `i`, `ℏ` | ✅ N/A | Defined in stdlib |
| **Placeholders** | `□` syntax | N/A | N/A - Editor only |
| **Summation/Product** | `Σ`, `Π` notation | ❌ Missing | Low |

### ✅ Calculus Operators (NEW Dec 13!)

| Operator | Syntax | Z3 Translation | Status |
|----------|--------|----------------|--------|
| **Power** | `x ^ n` | Z3 Power for integers | ✅ Works |
| **Square root** | `sqrt(x)` | Z3 Real arithmetic | ✅ Works |
| **Absolute value** | `abs(x)` | Z3 `If(x >= 0, x, -x)` | ✅ Works |
| **Gradient** | `∇f` | Uninterpreted function | ✅ Works |
| **Partial derivative** | `D(f, x)` | Uninterpreted function | ✅ Works |
| **Total derivative** | `Dt(f, x)` | Uninterpreted function | ✅ Works |
| **Integral** | `∫f`, `Integrate(f, x)` | Uninterpreted function | ✅ Works |
| **Double integral** | `∬f`, `DoubleIntegral(f, x, y)` | Uninterpreted function | ✅ Works |
| **Triple integral** | `∭f`, `TripleIntegral(f, x, y, z)` | Uninterpreted function | ✅ Works |
| **Line integral** | `∮f`, `LineIntegral(F, curve)` | Uninterpreted function | ✅ Works |
| **Surface integral** | `∯f`, `SurfaceIntegral(F, surface)` | Uninterpreted function | ✅ Works |

---

## Design Philosophy: Why Kleis Doesn't Need Certain Features

### Why No Abstract Types?

**Other languages:** Abstract types hide implementation details ("you don't need to know what this is").

**Kleis philosophy:** Kleis is about **finding hiding violations**, not enabling them. Everything is transparent so Z3 can verify properties.

```kleis
-- Kleis way: Full transparency
structure Ring(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(x y : R). x + y = y + x
}
-- Z3 sees everything. No hidden assumptions.
```

**Kleis doesn't do trust. Kleis does proof.**

### Why No List Comprehensions?

**Other languages:** `[x*2 | x <- [1..10], even x]` generates a list.

**Kleis approach:** Define **typed constructor functions** with axioms - works for any type:

```kleis
-- Instead of list comprehension syntax:
operation eye : ℕ → Matrix(n, n, ℝ)
operation range : ℕ × ℕ → Set(ℕ)
operation filter : Set(T) × (T → Bool) → Set(T)

-- Properties verified by Z3:
axiom eye_identity: ∀(n : ℕ, M : Matrix(n, n, ℝ)). M × eye(n) = M
```

**Benefits over list comprehensions:**
- Works for any type (Matrix, Set, Vector, not just List)
- Axioms attached to define behavior
- Composable: `List(Matrix(3, 3, ℝ))` just works
- Z3 can verify properties

### Why No Parameterized Structure Dependencies?

**Other languages:** Functors, type class constraints (`class (Eq a, Ord a) => ...`)

**Kleis has:**
- `extends` for single inheritance
- `over` for field action
- Nested structures for composition
- Imports for using operations

```kleis
structure Ring(R) {
    structure additive : AbelianGroup(R) { ... }
    structure multiplicative : Monoid(R) { ... }
}
```

These mechanisms handle algebraic hierarchies without needing structure-typed parameters.

### The Kleis Contract

| Traditional Languages | Kleis |
|----------------------|-------|
| Hide implementation | **Expose everything** |
| Generate values | **State properties** |
| Trust through encapsulation | **Trust through verification** |
| Compute results | **Verify claims** |

**Kleis: Mathematical notation → Z3 verification. Clean and focused.**

### ✅ Z3 Integration Features

**Derivative notation follows Mathematica convention:**
- `D(f, x)` - Partial derivative ∂f/∂x
- `Dt(f, x)` - Total derivative df/dx

**Why missing features matter:**

**High priority (blocks full prelude):**
- ~~Operator symbols~~ ✅ **DONE!**
- ~~Universal quantifiers~~ ✅ **DONE!**
- ~~`where` clauses~~ ✅ **DONE!**
- **`extends` keyword:** Needed for structure inheritance
- **`element` keyword:** Needed to distinguish constants from operations
- **Nested structures:** Needed for Ring/Field hierarchy

**Medium priority (convenience):**
- Prefix/postfix operators: User-friendly syntax (¬ works, need -, ∇, √)
- Type annotations: Explicit type declarations

**Low priority (can work around):**
- ~~Lambda: Can use function definitions instead~~ ✅ **IMPLEMENTED Dec 17!**
- ~~Let, if: Can use function definitions instead~~ ✅ **IMPLEMENTED Dec 13!**
- Symbolic constants: Can use variables
- Placeholders: Editor generates them, parser doesn't need to parse them

---

## Parser Evolution

### Version History

**v0.3 (December 6, 2025):**
- Basic expressions: identifiers, numbers, operators
- Function calls with precedence
- ~30% grammar coverage

**v0.4 (December 7-8, 2025):**
- Added structure definitions
- Added implements blocks
- ~35% grammar coverage

**v0.5 (December 8, 2025 - Morning):**
- Added data type definitions
- Added pattern matching (complete!)
- Added function definitions
- Added List literal AST support
- ~40-45% grammar coverage

**v0.5.1 (December 10, 2025 - Evening):** ✨ **Z3 Integration**
- Added universal quantifiers (`∀`, `∃`)
- Added existential quantifiers
- Added operator symbols in declarations
- Added logical operators (`∧`, `∨`, `¬`, `⟹`)
- Added comparison operators (`=`, `<`, `>`, `≤`, `≥`, `≠`)
- Created axiom verifier (`src/axiom_verifier.rs`)
- **~52% grammar coverage** (+12 percentage points!)

**v0.5.2 (December 10, 2025 - Late Evening):** ✨ **Where Clauses**
- Added where clause support to implements blocks
- Syntax: `implements Foo(T) where Bar(T) { ... }`
- Integrated with Z3 (constrained axioms available)
- Recursive constraint loading
- **~55% grammar coverage** (+3 percentage points!)

**v0.5.3 (December 10, 2025 - Very Late Evening):** ✨ **Nested Structures**
- Added nested structure support (compositional algebra!)
- Syntax: `structure Ring(R) { structure additive : Group(R) { ... } }`
- Integrated with Z3 (nested axioms/identities available)
- Arbitrary nesting depth supported
- **~58% grammar coverage** (+3 percentage points!)

**v0.5.4 (December 10, 2025 - Ultra Late Evening):** ✨ **Extends Keyword**
- Added structure inheritance with extends
- Syntax: `structure Monoid(M) extends Semigroup(M) { ... }`
- Integrated with Z3 (parent axioms automatically loaded!)
- Transitive inheritance (4+ levels working)
- **~60% grammar coverage** (+2 percentage points!)

**v0.5.5 (December 10, 2025 - Final):** ✨ **Define with Operators**
- Enabled operator names in define statements
- Syntax: `define (-)(x, y) = x + negate(y)`
- Works with all operators: +, -, ×, ⊗, ∘
- One-line change (parse_identifier → parse_operation_name)
- **~60% grammar coverage** (refinement)

**v0.5.6 (December 11, 2025):** 🔧 **Quality & Documentation**
- Fixed Z3 dependency analysis bug (nullary operations like `e`, `zero`, `one` now found)
- All 5/5 Z3 proof tests pass - mathematical rigor achieved! ✅
- Created `kleis_doc` tool: generates HTML/Markdown docs from .kleis files
- Synchronized G4 grammar with EBNF (added custom operators, named operations)
- Documented comment support: `//` line and `/* */` block comments fully work
- Updated compatibility doc with custom operators and comments
- 426+ tests passing (421 library + 5 Z3 proof tests)

**v0.6.2 (December 17, 2025):** ✨ **Type Ascription** (Haskell-style)
- Added `Expression::Ascription` variant for expression-level type annotations
- Syntax: `(a + b) : ℝ`, `v : Vector(3)`, `M : Matrix(3, 3, ℝ)`
- Parser recognizes `: Type` at end of expressions (lowest precedence)
- 7 new parser tests for type ascription
- Updated all pattern matches across codebase (16 files)
- Updated documentation in `docs/guides/LET_BINDINGS.md`

**v0.6.1 (December 17, 2025):** ✨ **Typed Let Bindings**
- Added optional type annotations to let bindings: `let x : ℝ = 5 in x^2`
- Implemented `Display` for `TypeExpr` for type-to-string conversion
- Added `let_binding_typed()` helper function to AST
- Updated pretty printer to output `let x : T = e in body`
- 7 new parser tests for typed let bindings
- Added comprehensive documentation: `docs/guides/LET_BINDINGS.md`

**v0.6.0 (December 13, 2025):** ✨ **Control Flow Constructs**
- Added `if/then/else` conditionals with Z3 `ite` translation
- Added `let x = value in body` bindings with context extension
- Pure functional semantics: let bindings extend variable context
- Proper variable shadowing in nested let bindings
- 20+ new parser tests for conditionals and let bindings
- 13 new integration tests for Z3 verification
- **~65% grammar coverage** (+5 percentage points!)
- 487+ unit tests + integration tests passing

**v0.7.0 (December 16, 2025):** ✨ **Type System Enhancements**
- Added `type Name = Type` aliases with full normalization
- Added parenthesized types: `(ℝ → ℝ) → ℝ` for higher-order functions
- Pretty printer extended for all TopLevel constructs
- Round-trip test: parse → print → parse with 17/17 tests passing
- REPL reports type alias count on `:load`
- **~70% grammar coverage** (+5 percentage points!)
- 521+ unit tests passing

---

## Coverage Breakdown

### Grammar v0.5 Major Features

**Total features in formal grammar:** ~25 major constructs

**Implemented (27):** ⭐ **+1 from Dec 17 (typed let bindings); +2 from Dec 13 (if/then/else, let bindings)**
1. ✅ Basic expressions (identifiers, numbers)
2. ✅ Infix operators with precedence
3. ✅ Function calls
4. ✅ Parentheses/grouping
5. ✅ Data type definitions
6. ✅ Pattern matching (all pattern types)
7. ✅ Function definitions
8. ✅ Structure definitions
9. ✅ Implements blocks
10. ✅ List literals (AST level)
11. ✅ Type expressions
12. ✅ **Universal quantifiers `∀`** ⭐
13. ✅ **Existential quantifiers `∃`** ⭐
14. ✅ **Operator symbols in definitions `(×)`** ⭐
15. ✅ **Logical operators (`∧`, `∨`, `¬`, `⟹`)** ⭐
16. ✅ **Comparison operators** ⭐
17. ✅ **Where clauses (`where Constraint(T)`)** ⭐
18. ✅ **Nested structures (compositional algebra)** ⭐
19. ✅ **Extends keyword (structure inheritance)** ⭐
20. ✅ **Define with operators (`define (-)(x,y)`)** ⭐
21. ✅ **Custom operators (`•`, `⊗`, `⊕`, etc.)** ⭐
22. ✅ **Comments (`//` and `/* */`)** ✅
23. ✅ **Axiom verification (Z3)** ⭐
24. ✅ **Generic constraint verification** ⭐
25. ✅ **Conditionals (`if x > 0 then x else 0`)** ⭐ **NEW Dec 13!**
26. ✅ **Let bindings (`let x = 5 in x + x`)** ⭐ Dec 13
27. ✅ **Typed let bindings (`let x : ℝ = 5 in x^2`)** ⭐ **NEW Dec 17!**
28. ✅ **Postfix operators (`n!`, `Aᵀ`, `A†`)** ⭐ **NEW Dec 17!**
29. ✅ **Type aliases (`type Name = Type`)** ⭐ Dec 16

**Not Implemented (1):**
1. ❌ Summation/Product notation (`Σ`, `Π`)

**Recently Implemented:**
30. ✅ **Lambda expressions (`λ x . x^2`)** ⭐ Dec 17

**Note:** Prefix operators are well-supported: `-`, `¬`, `∇`, `∫`, `∬`, `∭`, `∮`, `∯` (8 total)

**Previously marked missing, now implemented:**
- ~~Let bindings~~ ✅ **DONE Dec 13!**
- ~~Conditionals (if/then/else)~~ ✅ **DONE Dec 13!**
- ~~Postfix operators~~ ✅ **DONE Dec 17!**
- ~~Type aliases~~ ✅ **DONE Dec 16!**

**Major Feature Coverage:** 30/32 = **94%** of major constructs  
**Overall Grammar Coverage:** **~80%** (prefix operators more complete than previously documented)

---

## What Works in Practice

### ✅ Can Load These Stdlib Files:

- **`stdlib/types.kleis`** ✅ (data types, function definitions)
- **`stdlib/minimal_prelude.kleis`** ✅ (structures, basic operations)
- **`stdlib/matrices.kleis`** ✅ (now with operator symbols!)
- **`stdlib/tensors_minimal.kleis`** ✅ (subset)
- **`stdlib/quantum_minimal.kleis`** ✅ (subset)
- **`stdlib/math_functions.kleis`** ✅ (all math ops)

### ⚠️ Partially Supported:

- **`stdlib/prelude.kleis`** ⚠️ (operator symbols ✅, quantifiers ✅, where clauses ✅, but needs `extends`, `element`, nested structures)
- **`stdlib/tensors.kleis`** ⚠️ (most syntax works, may need minor adjustments)
- **`stdlib/quantum.kleis`** ⚠️ (most syntax works, may need minor adjustments)

### ❌ Remaining Blocker:

- **`where` clauses** - Needed for generic constraints like `where Semiring(T)`

---

## Specific Blocking Issues

### ~~Issue 1: Operator Symbols in Definitions~~ ✅ **SOLVED!**

**Now works in parser:**
```kleis
structure Ring(R) {
  operation (×) : R × R → R    // ✅ Parser handles (×)
  operation (+) : R × R → R    // ✅ Parser handles (+)
}
```

**Z3 Integration Bonus:** Built-in Z3 support for arithmetic operators!

### ~~Issue 2: Universal Quantifiers~~ ✅ **SOLVED!**

**Now works in parser:**
```kleis
axiom associativity:
  ∀(x y z : S). (x • y) • z = x • (y • z)    // ✅ Parser handles ∀
```

**Z3 Integration:** Axioms are now **verifiable** with theorem prover!

### ~~Issue 3: `where` Clauses~~ ✅ **SOLVED!**

**Now works in parser:**
```kleis
implements MatrixMultipliable(m, n, p, T) 
  where Semiring(T) {    // ✅ Parser now supports 'where'!
    operation multiply = builtin_matrix_multiply
  }
```

**Z3 Integration:** Constrained structure axioms are **automatically loaded** for verification!

### Issue 4: Structure Inheritance ⚠️ **REMAINING BLOCKER**

**Needed for structure hierarchy:**
```kleis
structure Monoid(M) extends Semigroup(M) {  // ✅ Parser now supports 'extends'!
    element e : M
}
```

**Status:** This (plus `element` and nested structures) blocks loading full `prelude.kleis`


---

## Why Parser Is Simplified

### Design Decision (ADR-015, ADR-007)

**Goal:** Bootstrap with ~30-40% of grammar, expand gradually.

**Benefits:**
- ✅ Validates core design decisions
- ✅ Loads working stdlib (minimal versions)
- ✅ Type system works with real code
- ✅ Easy to understand and test
- ✅ Can ship POC without full parser

**Trade-off:**
- ⚠️ Can't load full prelude.kleis yet
- ⚠️ Users must use workarounds (times vs ×)
- ⚠️ Documentation shows ideal syntax parser can't handle

---

## Z3 Theorem Proving Integration (December 10, 2025 - Evening) 🎯

### Major Achievement: Axioms Are Now Verifiable!

**Before:**
```kleis
// axiom identity: forall x. x + 0 = x  // Just a comment
```

**After:**
```kleis
axiom identity: ∀(x : M). x + 0 = x
// Z3 verifies: ✅ VALID!
```

### What Works:

1. **Parse axioms with quantifiers:**
   ```kleis
   axiom commutativity: ∀(x y : R). x + y = y + x
   axiom associativity: ∀(x y z : R). (x + y) + z = x + (y + z)
   axiom distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
   ```

2. **Verify with Z3 theorem prover:**
   - ✅ Commutativity: VERIFIED
   - ✅ Associativity: VERIFIED  
   - ✅ Distributivity: VERIFIED
   - ❌ Invalid axioms: COUNTEREXAMPLE FOUND

3. **Query axioms programmatically:**
   ```rust
   let axioms = registry.get_axioms("Ring");
   for (name, expr) in axioms {
       let result = verifier.verify_axiom(expr)?;
   }
   ```

### Implementation:

- **New module:** `src/axiom_verifier.rs` (generic Kleis → Z3 translator)
- **AST support:** `Expression::Quantifier` with `QuantifierKind`
- **58 new tests** added (all passing!)
- **Feature flag:** Z3 as default feature (can disable with `--no-default-features`)

### Test Results:

- **434+ tests total** on current branch ✅
- **Axiom integration tests:** 10 tests ✅
- **Logical operator tests:** 12 tests ✅
- **Quantifier parsing tests:** 7 tests ✅
- **Operator symbol tests:** 7 tests ✅
- **Structure loading tests:** 3 tests ✅
- **Multi-level structure tests:** 5 tests ✅
- **Where clause parsing tests:** 10 tests ✅ **NEW!**
- **Where constraint Z3 tests:** 3 tests ✅ **NEW!**
- **Library tests:** 421 tests ✅

---

## 💡 Key Discovery: Identity Elements Work Without `element` Keyword!

**We discovered:** The `element` keyword is NOT required for identity elements!

**Instead of:**
```kleis
structure Ring(R) {
    element zero : R    // Needs 'element' keyword?
    element one : R
}
```

**We can use:**
```kleis
structure Ring(R) {
    operation zero : R    // Nullary operation = identity element!
    operation one : R     // No arrows = constant!
    operation plus : R → R → R
}
```

**AxiomVerifier automatically detects:**
```rust
let is_nullary = !matches!(type_signature, TypeExpr::Function(..));
if is_nullary {
    // This is an identity element!
    identity_elements.insert(name, z3_const);
}
```

**This works in all our tests!** Group/Ring/Field identity elements all work without `element` keyword.

**Impact:** One less parser feature needed for full prelude! 🎉

---

## 💡 Axiom Notation Flexibility: Mathematical vs Function Style

**You can write axioms TWO ways - both work identically!**

### Mathematical Notation (Beautiful!) ⭐ Recommended

```kleis
structure Ring(R) {
    operation plus : R → R → R
    operation times : R → R → R
    
    axiom commutativity: ∀(x y : R). x + y = y + x
    axiom associativity: ∀(x y z : R). (x + y) + z = x + (y + z)
    axiom distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
}
```

### Function Notation (Explicit)

```kleis
structure Ring(R) {
    operation plus : R → R → R
    operation times : R → R → R
    
    axiom commutativity: ∀(x y : R). equals(plus(x, y), plus(y, x))
    axiom associativity: ∀(x y z : R). equals(plus(plus(x, y), z), plus(x, plus(y, z)))
    axiom distributivity: ∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))
}
```

### How It Works

**Parser converts both to the same AST:**

```
Input:  x + y = y + x
Parses: Operation { name: "equals", args: [
          Operation { name: "plus", args: [x, y] },
          Operation { name: "plus", args: [y, x] }
        ]}

Input:  equals(plus(x, y), plus(y, x))
Parses: (exact same AST!)
```

**Z3 receives identical representation either way!**

### Which to Use?

**Mathematical notation:**
- ✅ More readable
- ✅ Matches textbooks
- ✅ Easier to write
- ✅ **Recommended for users!**

**Function notation:**
- ✅ More explicit
- ✅ Useful for debugging
- ✅ Shows exact operation names
- ✅ Useful in tests

**Both verify identically with Z3!**

### Supported Operators in Axioms

- `+` → `plus`
- `-` → `minus`
- `×` → `times`
- `/` → `divide`
- `=` → `equals`
- `<`, `>`, `≤`, `≥` → comparisons
- `∧`, `∨`, `¬`, `⟹` → logical operators

**All work in both infix and function notation!**

---

## Recent Additions (December 8-10, 2025)

### Pattern Matching (Complete!)

```kleis
define not(b) = match b {
  True => False
  | False => True
}

define head(list) = match list {
  Nil => None
  | Cons(h, _) => Some(h)
}
```

**All pattern types work:**
- ✅ Wildcard: `_`
- ✅ Variables: `x`
- ✅ Constructors: `Some(x)`, `Cons(h, t)`
- ✅ Nested: `Some(Cons(x, xs))`
- ✅ Tuples: `(x, y)`
- ✅ Constants: `0`, `"hello"`

**Tests:** 17 pattern parsing tests, all passing ✅

### List Literals (AST Level)

```rust
Expression::List(Vec<Expression>)
```

Used for:
- `Matrix(2, 2, [a, b, c, d])` ✅
- `Piecewise(2, [expr1, expr2], [cond1, cond2])` ✅

**Not yet:** Parser doesn't parse `[1,2,3]` text → but AST supports it!

---

## Comparison with Grammar v0.6

### Core Expression Grammar

**Formal grammar v0.6:**
```ebnf
expression
    ::= primary
      | prefixOp expression              (* ❌ Not supported *)
      | expression postfixOp              (* ❌ Not supported *)
      | expression infixOp expression     (* ✅ Supported! *)
      | expression '(' arguments ')'      (* ✅ Supported *)
      | '[' expressions ']'               (* ✅ Supported! *)
      | matchExpr                          (* ✅ Supported! *)
      | lambda                             (* ✅ NEW Dec 17! *)
      | letBinding                         (* ✅ NEW Dec 13! *)
      | conditional                        (* ✅ NEW Dec 13! *)
      ;
```

**Our parser (simplified):**
```rust
expression := term (('+' | '-') term)*           // Full arithmetic
term       := factor (('*' | '/') factor)*       // With precedence
factor     := primary ('^' primary)?             // Right-associative
primary    := identifier 
            | number 
            | function_call                      // identifier '(' args ')'
            | '(' expression ')'
            | match_expr                         // ✅ Pattern matching
            | conditional                        // ✅ if/then/else NEW!
            | let_binding                        // ✅ let x = v in body NEW!
            | '[' expressions ']'                // ✅ List literals
```

---

## What This Means in Practice

### ✅ You Can Write (Works Today):

```kleis
data Bool = True | False

define not(b) = match b {
  True => False
  | False => True
}

structure Matrix(m: Nat, n: Nat, T) {
  operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}

implements Matrix(m, n, ℝ) {
  operation transpose = builtin_transpose
}
```

All of this **parses and type-checks** today! ✅

### ✅ Now Supported (As of Dec 10, 2025):

```kleis
// Operator symbols - NOW WORKS! ✅
operation (×) : R × R → R

// Universal quantifiers - NOW WORKS! ✅
axiom commutativity: ∀(x y : S). x • y = y • x

// Logical operators - NOW WORKS! ✅
axiom identity: ∀(x : M). (x ∧ True) ⟹ x
```

### ✅ Now Supported (Dec 17, 2024):

```kleis
// Lambda expressions - NOW SUPPORTED!
map(λ x . x^2, [1,2,3])      // ✅ Works!
λ x y . x + y                 // ✅ Multiple parameters
λ (x : ℝ) . x^2               // ✅ With type annotations

// Vector literals in source
v = [1, 2, 3]                // ❌ Not yet supported (but AST supports it!)

// where clauses
implements Foo(T) where Bar(T) { ... }  // ❌ Not yet supported
```

---

## Why We Have Both "Minimal" and "Full" Stdlib

**Status Update (Dec 10, 2025):**

**Minimal versions** (works on main branch):
- Use syntax parser CAN handle
- No operator symbols: `times` instead of `(×)`
- No quantifiers: skip axiom bodies
- Work with ~45% parser

**Full versions** (works on feature branch! 🎉):
- ✅ Use ideal syntax: `operation (×)` - **NOW WORKS!**
- ✅ Include axioms: `∀(x : T)` - **NOW WORKS!**
- ✅ Logical operators: `∧`, `∨`, `¬`, `⟹` - **NOW WORKS!**
- ⚠️ Still needs: `where` clauses for full prelude

**Files:**
- `minimal_prelude.kleis` ✅ (works on all branches)
- `matrices.kleis` ✅ (works with operator symbols)
- `prelude.kleis` ⏳ (needs `extends`, `element`, nested structures)
- `tensors.kleis` ⏳ (needs `extends`, `element`)
- `quantum.kleis` ⏳ (needs `extends`, `element`)

---

## Path to Full Grammar Support

### ✅ Recently Completed (Dec 10, 2025)

**1. Operator Symbols in Definitions** ✅ **DONE!**
```kleis
operation (×) : T → T → T
operation (⊗) : T → T → T
```

**Status:** Implemented in Phase 1.2 of Z3 integration

**2. Universal Quantifiers** ✅ **DONE!**
```kleis
axiom associativity: ∀(x y z : S). (x • y) • z = x • (y • z)
```

**Status:** Implemented in Phase 1.1 of Z3 integration

**3. Logical Operators** ✅ **DONE!**
- Conjunction: `∧`, Disjunction: `∨`, Negation: `¬`, Implication: `⟹`

**Status:** Implemented in Phase 2.1 of Z3 integration

**4. Where Clauses** ✅ **DONE!**
```kleis
implements MatrixMultipliable(m, n, p, T) where Semiring(T) {
  operation multiply = builtin_matrix_multiply
}
```

**Status:** Implemented in Phase 3.1 with full Z3 integration!

### High Priority (Current Blockers for Full Prelude)

**1. `extends` Keyword** (~3-4 hours)
```kleis
structure Monoid(M) extends Semigroup(M) {
  element e : M
}
```

**Needed for:** Structure inheritance hierarchy in `prelude.kleis`

**2. `define` with Operators** (~2-3 hours)
```kleis
define (-)(x, y) = x + negate(y)
```

**Needed for:** Defining operations with operator syntax

**Notes on features that work already:**

✅ **`element` keyword:** Not required! Nullary operations work:
```kleis
operation zero : R  // Nullary operation = identity element
```
AxiomVerifier detects them automatically!

✅ **Nested structures:** ✅ IMPLEMENTED!
```kleis
structure Ring(R) {
  structure additive : AbelianGroup(R) { ... }
  structure multiplicative : Monoid(R) { ... }
}
```
Fully integrated with Z3! Axioms from nested structures available!

### Medium Priority (Better UX)

**2. General Prefix Operators** (1-2 hours)
- Unary minus: `-x`
- ✅ Negation: `¬p` - **DONE!**
- Gradient: `∇f`
- Square root: `√x`

**3. Postfix Operators** (1-2 hours)
- Factorial: `n!`
- Transpose: `Aᵀ`
- Conjugate: `A†`

### Low Priority (Nice to Have)

~~**4. Lambda Expressions** (2-3 hours)~~ ✅ **DONE Dec 17!**
- ~~`λ x . x^2`~~
- ~~Can use `define` instead~~

~~**5. Let Bindings**~~ ✅ **DONE Dec 13!**
- `let x = 5 in x^2`
- Full Z3 integration with context extension
- Proper variable shadowing support

~~**6. Conditionals**~~ ✅ **DONE Dec 13!**
- `if x > 0 then x else 0`
- Translates to Z3's `ite` construct
- Works in function definitions

**7. List Literal Parsing** ✅ Already supported!
- `[1, 2, 3]` in source
- AST and parser both support it!

**8. Type Annotations** (2 hours)
- `x : ℝ`
- Type inference makes this optional

---

## Test Coverage

### Parser Tests

**Total:** 628 tests on `feature/full-prelude-migration` branch ✅  
**Comparison:** 565 tests on `main` branch

**Key Test Categories:**
- ✅ Library tests (src/lib.rs): 420 tests
- ✅ Basic expressions: 8 tests
- ✅ Function calls: 6 tests
- ✅ Operators: 10 tests
- ✅ Data definitions: 5 tests
- ✅ Pattern matching: 17 tests
- ✅ Function definitions: 8 tests
- ✅ Structures: 12 tests
- ✅ **Quantifier parsing: 7 tests** ⭐ NEW!
- ✅ **Operator symbols: 7 tests** ⭐ NEW!
- ✅ **Logical operators: 11 tests** ⭐ NEW!
- ✅ **Axiom integration: 10 tests** ⭐ NEW!
- ✅ **Registry queries: 5 tests** ⭐ NEW!
- ✅ **Z3 foundation: ~21 tests** ⭐ NEW!
- ✅ **Plus 100+ additional integration tests** ✅

**Growth:** +63 tests from main branch (565 → 628)  
**All passing!** ✅

---

## Real-World Usage

### What Works Today

**Self-hosting functions in stdlib:**
```kleis
define not(b) = match b { True => False | False => True }
define head(list) = match list { Nil => None | Cons(h, _) => Some(h) }
define getOrDefault(opt, default) = match opt { None => default | Some(x) => x }
```

**9 functions loaded and callable!** ✅

**Type definitions loaded:**
```kleis
data Bool = True | False
data Option(T) = None | Some(value: T)
data List(T) = Nil | Cons(head: T, tail: List(T))
```

**Complete pattern matching working in production!** ✅

### What We Load Successfully

**TypeChecker::with_stdlib() loads:**
1. `types.kleis` (265 lines) ✅
2. `minimal_prelude.kleis` (127 lines) ✅
3. `matrices.kleis` (127 lines) ✅
4. `tensors_minimal.kleis` (56 lines) ✅
5. `quantum_minimal.kleis` (47 lines) ✅
6. `math_functions.kleis` (87 lines) ✅

**Total: 709 lines of Kleis code loaded and type-checked!** ✅

---

## Incompatibility Impact

### Medium Impact

**Can't express ideal signatures:**
```kleis
// Ideal (from formal grammar):
operation (×) : ∀(m n p : ℕ, T). Matrix(m,n,T) × Matrix(n,p,T) → Matrix(m,p,T)

// What works (current parser):
structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
  operation multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
}
```

**Same semantics, less elegant syntax.**

### Low Impact

**Most features work fine:**
- ✅ Type system fully functional
- ✅ Pattern matching complete
- ✅ Self-hosting functions work
- ✅ Parametric polymorphism works
- ✅ Axiom verification with Z3
- ✅ 628 tests passing (feature branch)

**Parser limitations don't block core functionality!**

---

## Next Steps

### ✅ Phase 1 & 2 Complete! (Dec 10, 2025)

**Completed in Z3 Integration Branch:**
- ✅ Operator symbols: `operation (×)`
- ✅ Universal quantifiers: `∀(x : T)`
- ✅ Logical operators: `∧`, `∨`, `¬`, `⟹`
- ✅ Z3 theorem prover integration
- ✅ Axiom verification working

**Branch:** `feature/full-prelude-migration` (628 tests passing)

### ~~Phase 3: Where Clauses~~ ✅ **COMPLETE!**

**Completed Work:**
1. ✅ `where` clauses (3 hours) - Generic constraints working!
2. ✅ Z3 integration (2 hours) - Constrained axioms available to verifier
3. ✅ ADR-022 (already on main) - Z3 architecture documented

**Total:** 5 hours (exactly as estimated!)

### Phase 4: Full Prelude (Future Work)

**Remaining for full prelude:**
1. `extends` keyword (3-4 hours) - Structure inheritance
2. `define` with operators (2-3 hours) - Operator definitions

**Total:** ~5-7 hours additional work (reduced from 8-11!)

**Completed (not blockers anymore):**
- ✅ `element` keyword - Nullary operations work the same way!
- ✅ Nested structures - IMPLEMENTED! Compositional algebra works!

**We're getting close to full prelude!** Only 2 features remain!

### Future Enhancements (Lower Priority)

**User Experience Improvements:**
1. General prefix operators (unary minus, gradient)
2. Postfix operators (factorial, transpose)
3. List literal parsing `[1,2,3]`
4. Type annotations `x : ℝ`

**Advanced Features:**
1. ~~Lambda expressions~~ ✅ **DONE Dec 17!**
2. ~~Let bindings~~ ✅ **DONE Dec 13!**
3. Advanced pattern features (guards, as-patterns)

---

## Conclusion

### ✅ Parser Successfully Supports Core Language + Theorem Proving

**What works (Dec 10, 2025):**
- Complete pattern matching ⭐
- Data type definitions ⭐
- Function definitions ⭐
- Structure/implements blocks ⭐
- Full type inference ⭐
- **Operator symbols in declarations** ⭐ NEW!
- **Universal & existential quantifiers** ⭐ NEW!
- **Logical operators with proper precedence** ⭐ NEW!
- **Z3 theorem prover integration** ⭐ NEW!

**Coverage: ~65% of formal grammar** (up from 60%)

This is **sufficient for:**
- Loading working stdlib
- Self-hosted functions
- Production type checking
- Real mathematical expressions
- **Verifying axioms with Z3 theorem prover** ⭐
- **Checking mathematical properties formally** ⭐
- **Control flow in function definitions (if/then/else, let)** ⭐ **NEW!**

### ✅ Major Extensions Complete (Dec 10, 2025)

**Recently Implemented:**
1. ✅ Operator symbols: `(×)`, `(⊗)` - **DONE!**
2. ✅ Universal quantifiers: `∀(x : T)` - **DONE!**
3. ✅ Logical operators: `∧`, `∨`, `¬`, `⟹` - **DONE!**
4. ✅ Z3 theorem prover integration - **DONE!**

### ✅ All Core Features Implemented!

**Completed (Dec 10, 2025):**
1. ✅ `extends` keyword - Structure inheritance **DONE!**
2. ✅ `define` with operators - Operator definitions **DONE!**
3. ✅ Nested structures - Compositional algebra **DONE!**
4. ✅ Where clauses - Generic constraints **DONE!**
5. ✅ Custom operators - Unicode math symbols **DONE!**

**Already worked:**
- ✅ Nullary operations work: `operation zero : R` (no arrows = identity element)
- ✅ Comments: `//` and `/* */` fully supported

**Remaining for full prelude.kleis:**
- ⚠️ Top-level operation declarations: `operation dot : ∀(n : ℕ). Vector(n) → ℝ`
- ⚠️ Top-level define statements (not critical for Z3)

**Timeline:** Full prelude support ~2-3 hours (top-level syntax only)

---

## Related Documents

- **[Kleis Grammar v0.96](../grammar/kleis_grammar_v096.md)** - Complete formal specification (CURRENT)
- **[Kleis Grammar v0.6](../grammar/archive/kleis_grammar_v06.md)** - Historical specification
- **[Parser Status](../archive/parser-implementation-KLEIS_PARSER_STATUS.md)** - Implementation details (archived, historical)
- **[ADR-007](../adr/adr-007-bootstrap-grammar.md)** - Bootstrap strategy (~30% → gradual expansion)
- **[ADR-015](../adr/adr-015-text-as-source-of-truth.md)** - Why we need Kleis text parser

---

**Status:** ✅ **~70% Coverage - Complete Algebraic Type System with Theorem Proving + Control Flow + Calculus**  
**Recommendation:** Production ready with calculus operators!

**Current Branch:** `feature/calculus-operators` (500+ tests passing)  
**Main Branch:** `main` (Phase 1, 2, 3 merged)

**Phase Status:**
- ✅ Phase 1 & 2: Z3 integration - MERGED to main
- ✅ Phase 3: Where clauses + nested structures + extends + define operators - COMPLETE!
- ✅ Phase 4: Calculus operators (power, sqrt, abs, derivatives, integrals) - COMPLETE!
- ⚠️ Full prelude: Only product type syntax remains (minor: S × S → R vs S → S → R)

**Features Implemented Dec 13 (calculus branch):**
- Power operator (`^`) with Z3 translation
- `sqrt` and `abs` functions
- Gradient prefix operator (`∇f`)
- Partial/Total derivatives (`D(f, x)`, `Dt(f, x)`) - Mathematica style
- Integral operators (`∫`, `∬`, `∭`, `∮`, `∯`)
- Integrable structure with FTC axiom
- Round-trip tested with all examples

**Last Updated:** December 16, 2025 (Added type aliases, parenthesized types, round-trip test)
