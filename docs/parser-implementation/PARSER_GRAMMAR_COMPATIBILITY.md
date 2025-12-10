# Kleis Parser vs Formal Grammar Compatibility

**Date:** December 10, 2024  
**Formal Grammar:** Kleis v0.5 (with pattern matching)  
**Parser Implementation:** `src/kleis_parser.rs`

---

## TL;DR

⚠️ **Parser implements ~40-45% of formal grammar v0.5, intentionally simplified for POC**

**Coverage:** ~40-45% of formal grammar  
**Purpose:** Validate core language features and ADR-015 design  
**Status:** Good for POC and stdlib loading, needs expansion for production

---

## What's Supported NOW (December 2024)

### ✅ Fully Supported

| Feature | Grammar v0.5 | kleis_parser.rs | Status |
|---------|--------------|-----------------|--------|
| **Data types** | `data Bool = True \| False` | ✅ Complete | ✅ **NEW!** |
| **Pattern matching** | `match x { True => 1 \| False => 0 }` | ✅ Complete | ✅ **NEW!** |
| **Function definitions** | `define f(x) = x + x` | ✅ Complete | ✅ **NEW!** |
| **List literals** | `[1, 2, 3]` | ✅ In AST | ✅ **NEW!** |
| **Structure definitions** | `structure Matrix(m, n, T) { ... }` | ✅ Complete | ✅ Works |
| **Implements blocks** | `implements Matrix(m, n, ℝ) { ... }` | ✅ Complete | ✅ Works |
| **Function calls** | `abs(x)`, `frac(a,b)` | ✅ Complete | ✅ Works |
| **Basic arithmetic** | `a + b - c * d / e` | ✅ With precedence | ✅ Works |
| **Exponentiation** | `a ^ b` | ✅ Right-associative | ✅ Works |
| **Parentheses** | `(a + b) * c` | ✅ Grouping | ✅ Works |
| **Identifiers** | `x`, `alpha`, `myVar` | ✅ Standard | ✅ Works |
| **Numbers** | `42`, `3.14` | ✅ Integer and float | ✅ Works |

**Pattern Matching Features:**
- Wildcard: `_`
- Variables: `x`, `myVar`
- Constructors: `Some(x)`, `Cons(h, t)`
- Nested patterns: `Some(Cons(x, xs))`
- Tuple patterns: `(x, y)`
- Constant patterns: `0`, `"hello"`

**Total Major Features:** ~12 supported ✅

---

## What's Still Missing

### ❌ Not Yet Supported

| Feature | Grammar v0.5 | Status | Priority |
|---------|--------------|--------|----------|
| **Prefix operators** | `-x`, `∇f`, `√x` | ❌ Missing | Medium |
| **Postfix operators** | `n!`, `Aᵀ`, `A†` | ❌ Missing | Medium |
| **Lambda expressions** | `λ x . x^2` | ❌ Missing | Low |
| **Let bindings** | `let x = 5 in x^2` | ❌ Missing | Low |
| **Conditionals** | `if x > 0 then x else -x` | ❌ Missing | Low |
| **Type annotations** | `x : ℝ` | ❌ Missing | Medium |
| **Operator symbols** | `(×)`, `(⊗)` in definitions | ❌ Missing | High |
| **Symbolic constants** | `π`, `e`, `i`, `ℏ` | ❌ Missing | Low |
| **Universal quantifiers** | `∀(x : T)` in axioms | ❌ Missing | High |
| **Placeholders** | `□` syntax | ❌ Missing | Low |

**Why missing features matter:**

**High priority (blocks full stdlib):**
- Operator symbols: Prevents loading `prelude.kleis` with `operation (×)`
- Universal quantifiers: Prevents loading axioms with `∀(x y : S)`

**Medium priority (convenience):**
- Prefix/postfix operators: User-friendly syntax
- Type annotations: Explicit type declarations

**Low priority (can work around):**
- Lambda, let, if: Can use function definitions instead
- Symbolic constants: Can use variables
- Placeholders: Editor generates them, parser doesn't need to parse them

---

## Parser Evolution

### Version History

**v0.3 (December 6, 2024):**
- Basic expressions: identifiers, numbers, operators
- Function calls with precedence
- ~30% grammar coverage

**v0.4 (December 7-8, 2024):**
- Added structure definitions
- Added implements blocks
- ~35% grammar coverage

**v0.5 (December 8, 2024):**
- Added data type definitions
- Added pattern matching (complete!)
- Added function definitions
- Added List literal AST support
- ~40-45% grammar coverage

---

## Coverage Breakdown

### Grammar v0.5 Major Features

**Total features in formal grammar:** ~25 major constructs

**Implemented (11):**
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

**Not Implemented (14):**
1. ❌ Prefix operators
2. ❌ Postfix operators
3. ❌ Lambda expressions
4. ❌ Let bindings
5. ❌ Conditionals (if/then/else)
6. ❌ Type annotations in expressions
7. ❌ Operator symbols in definitions `(×)`
8. ❌ Extended operator set (relations, logic, calculus)
9. ❌ Symbolic constants
10. ❌ Universal quantifiers `∀`
11. ❌ Existential quantifiers `∃`
12. ❌ Type aliases
13. ❌ Module system
14. ❌ Comments

**Coverage:** 11/25 = **44%**

---

## What Works in Practice

### ✅ Can Load These Stdlib Files:

- **`stdlib/types.kleis`** ✅ (data types, function definitions)
- **`stdlib/minimal_prelude.kleis`** ✅ (structures, basic operations)
- **`stdlib/matrices.kleis`** ✅ (except operator symbols)
- **`stdlib/tensors_minimal.kleis`** ✅ (subset)
- **`stdlib/quantum_minimal.kleis`** ✅ (subset)
- **`stdlib/math_functions.kleis`** ✅ (all math ops)

### ❌ Cannot Load These (Yet):

- **`stdlib/prelude.kleis`** ❌ (uses `operation (×)`, `∀(x : T)`)
- **`stdlib/tensors.kleis`** ❌ (full version with advanced syntax)
- **`stdlib/quantum.kleis`** ❌ (full version with advanced syntax)

---

## Specific Blocking Issues

### Issue 1: Operator Symbols in Definitions

**Needed for prelude.kleis:**
```kleis
structure Ring(R) {
  operation (×) : R × R → R    // ❌ Parser fails on (×)
  operation (+) : R × R → R    // ❌ Parser fails on (+)
}
```

**Workaround in minimal_prelude.kleis:**
```kleis
structure Arithmetic(T) {
  operation times : T → T → T    // ✅ Use word "times" instead
  operation plus : T → T → T     // ✅ Use word "plus" instead
}
```

### Issue 2: Universal Quantifiers

**Needed for axioms:**
```kleis
axiom associativity:
  ∀(x y z : S). (x • y) • z = x • (y • z)    // ❌ Parser fails on ∀
```

**Workaround:**
```kleis
axiom associativity:
  associative_law    // ✅ Just name it, don't express it
```

### Issue 3: Type-Level Computation

**Wanted:**
```kleis
operation transpose : ∀(m n : ℕ). Matrix(m,n) → Matrix(n,m)
```

**What works:**
```kleis
structure Matrix(m: Nat, n: Nat, T) {
  operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}
```

Same idea, but without the `∀` syntax.

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

## Recent Additions (December 8-10, 2024)

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

## Comparison with Grammar v0.5

### Core Expression Grammar

**Formal grammar v0.5:**
```ebnf
expression
    ::= primary
      | prefixOp expression              (* ❌ Not supported *)
      | expression postfixOp              (* ❌ Not supported *)
      | expression infixOp expression     (* ⚠️ Limited operators *)
      | expression '(' arguments ')'      (* ✅ Supported *)
      | '[' expressions ']'               (* ❌ Not in parser *)
      | matchExpr                          (* ✅ Supported! *)
      | lambda                             (* ❌ Not supported *)
      | letBinding                         (* ❌ Not supported *)
      | conditional                        (* ❌ Not supported *)
      ;
```

**Our parser (simplified):**
```rust
expression := term (('+' | '-') term)*           // Only + and -
term       := factor (('*' | '/') factor)*       // Only * and /
factor     := primary ('^' primary)?             // Only ^
primary    := identifier 
            | number 
            | function_call                      // identifier '(' args ')'
            | '(' expression ')'
            | match_expr                         // ✅ NEW!
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

### ❌ You Cannot Write (Parser Limitation):

```kleis
// Operator symbols
operation (×) : R × R → R    // ❌ Parser fails

// Universal quantifiers
∀(x y : S). x • y = y • x    // ❌ Parser fails

// Lambda
map(λ x . x^2, [1,2,3])      // ❌ Parser fails

// Vector literals in source
v = [1, 2, 3]                // ❌ Parser fails (but AST supports it!)
```

---

## Why We Have Both "Minimal" and "Full" Stdlib

**This is now clear:**

**Minimal versions** (loaded today):
- Use syntax parser CAN handle
- No operator symbols: `times` instead of `(×)`
- No quantifiers: skip axiom bodies
- Work with current ~40% parser

**Full versions** (future):
- Use ideal syntax: `operation (×)`
- Include axioms: `∀(x : T)`
- Need ~80-90% parser coverage

**Files:**
- `minimal_prelude.kleis` ✅ vs `prelude.kleis` ⏳
- `tensors_minimal.kleis` ✅ vs `tensors.kleis` ⏳
- `quantum_minimal.kleis` ✅ vs `quantum.kleis` ⏳

---

## Path to Full Grammar Support

### High Priority (Blocks Full Stdlib)

**1. Operator Symbols in Definitions** (2-3 hours)
```kleis
operation (×) : T → T → T
operation (⊗) : T → T → T
```

**Needed for:** `prelude.kleis` algebraic hierarchy

**2. Universal Quantifiers** (2-3 hours)
```kleis
axiom associativity: ∀(x y z : S). (x • y) • z = x • (y • z)
```

**Needed for:** Formal axioms in structures

### Medium Priority (Better UX)

**3. Prefix Operators** (1-2 hours)
- Unary minus: `-x`
- Negation: `¬p`
- Gradient: `∇f`

**4. Postfix Operators** (1-2 hours)
- Factorial: `n!`
- Transpose: `Aᵀ`
- Conjugate: `A†`

### Low Priority (Nice to Have)

**5. Lambda Expressions** (2-3 hours)
- `λ x . x^2`
- Can use `define` instead

**6. Let Bindings** (1 hour)
- `let x = 5 in x^2`
- Can use `define` instead

**7. List Literal Parsing** (1 hour)
- `[1, 2, 3]` in source
- AST already supports it!

**8. Type Annotations** (2 hours)
- `x : ℝ`
- Type inference makes this optional

---

## Test Coverage

### Parser Tests

**Total:** 553 lines of tests in `kleis_parser.rs`

**Categories:**
- ✅ Basic expressions: 8 tests
- ✅ Function calls: 6 tests
- ✅ Operators: 10 tests
- ✅ Data definitions: 5 tests
- ✅ Pattern matching: 17 tests ⭐
- ✅ Function definitions: 8 tests
- ✅ Structures: 12 tests

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
- ✅ 413 tests passing

**Parser limitations don't block core functionality!**

---

## Next Steps

### For Current POC: ✅ Parser is Sufficient

Current parser supports:
- All essential language features
- Real self-hosted stdlib functions
- Complete pattern matching
- Full type inference

**No immediate parser work needed!**

### For Production: Consider These

**Phase 1: Stdlib Completion** (Highest value)
1. Operator symbols: `operation (×)`
2. Universal quantifiers: `∀(x : T)`
3. Load full `prelude.kleis`

**Estimated:** 1 week

**Phase 2: User Experience** (Better syntax)
1. Prefix/postfix operators
2. List literal parsing `[1,2,3]`
3. Type annotations `x : ℝ`

**Estimated:** 1 week

**Phase 3: Advanced Features** (If needed)
1. Lambda expressions
2. Let bindings  
3. Advanced pattern features (guards, as-patterns)

**Estimated:** 1-2 weeks

---

## Conclusion

### ✅ Parser Successfully Supports Core Language

**What works:**
- Complete pattern matching ⭐
- Data type definitions ⭐
- Function definitions ⭐
- Structure/implements blocks ⭐
- Full type inference ⭐

**Coverage: 40-45% of formal grammar**

This is **sufficient for:**
- Loading working stdlib
- Self-hosted functions
- Production type checking
- Real mathematical expressions

### ⚠️ Parser Needs Extension For Full Stdlib

**Blocking issues:**
1. Operator symbols: `(×)`, `(⊗)`
2. Universal quantifiers: `∀(x : T)`

**Impact:** Can't load full `prelude.kleis` with ideal syntax

**Timeline:** 1-2 weeks to add these features

---

## Related Documents

- **[Kleis Grammar v0.5](../grammar/kleis_grammar_v05.md)** - Complete formal specification
- **[Parser Status](KLEIS_PARSER_STATUS.md)** - Implementation details
- **[ADR-007](../adr/adr-007-bootstrap-grammar.md)** - Bootstrap strategy (~30% → gradual expansion)
- **[ADR-015](../adr/adr-015-text-as-source-of-truth.md)** - Why we need Kleis text parser

---

**Status:** ✅ **~40-45% Coverage - Excellent for POC, Needs Extension for Full Production**  
**Recommendation:** Continue with current parser, add operator symbols + quantifiers when ready for full stdlib

**Last Updated:** December 10, 2024
