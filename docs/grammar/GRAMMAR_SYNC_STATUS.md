# Grammar Synchronization Status

**Date:** December 29, 2025  
**Status:** 🆕 v0.95 Released (Big Operator Syntax)

## Current Version: v0.95

### What's New in v0.95

**Big Operator Syntax** - Summation, product, integral, and limit:

```kleis
-- Summation: Σ(from, to, body)
Σ(1, n, λ i . f(i))

-- Product: Π(from, to, body)
Π(1, n, λ i . g(i))

-- Integral: ∫(lower, upper, body, var)
∫(0, 1, λ x . x * x, x)

-- Limit: lim(var, target, body)
lim(x, 0, sin(x) / x)
```

### v0.94 - N-ary Product Types

```kleis
operation f : A × B × C × D → E   -- Multi-factor types
```

### v0.93 - Example Blocks

```kleis
example "complex arithmetic" {
    let z1 = Complex(1, 2)
    assert(z1.re = 1)
}
```

## File Versions

| File | Version | Last Updated | Status |
|------|---------|--------------|--------|
| `kleis_grammar_v095.ebnf` | v0.95 | Dec 29, 2025 | ✅ Current |
| `kleis_grammar_v095.md` | v0.95 | Dec 29, 2025 | ✅ Documented |
| `kleis_grammar_v094.ebnf` | v0.94 | Dec 26, 2025 | ✅ Previous |
| `kleis_grammar_v093.ebnf` | v0.93 | Dec 24, 2025 | ✅ Previous |
| `kleis_grammar_v092.ebnf` | v0.92 | Dec 22, 2025 | ✅ Previous |
| `kleis_grammar_v08.ebnf` | v0.8 | Dec 18, 2025 | ✅ Legacy |
| `vscode-kleis/docs/grammar/` | v0.95 | Dec 29, 2025 | ✅ Synced |

---

## Changes Applied (Dec 18, 2025)

### Grammar v0.8 - Advanced Pattern Matching

**ADDED** - Pattern Guards:
```kleis
match x {
    n if n < 0 => "negative"
    n if n > 0 => "positive"
    _ => "zero"
}
```

**ADDED** - As-Patterns (Alias Binding):
```kleis
match list {
    Cons(h, t) as whole => process(h, t, whole)
    Nil => empty
}
```

**ADDED** - Let Destructuring:
```kleis
let Point(x, y) = p in x^2 + y^2
let Some(Pair(a, b)) = opt in a + b
```

**Grammar Changes:**
```ebnf
// Pattern guards
matchCase ::= pattern [ "if" guardExpression ] "=>" expression

// As-patterns
pattern ::= basePattern [ "as" identifier ]

// Let destructuring
letBinding ::= "let" pattern [ typeAnnotation ] "=" expression "in" expression
```

**No breaking changes** - v0.8 is fully backward compatible with v0.7.

---

## Changes Applied (Dec 13, 2025)

### Grammar v0.7 - Mathematica-Style Calculus (BREAKING CHANGE)

**REMOVED** - Old derivative notation:
```kleis
// No longer valid Kleis:
∂f/∂x
df/dx  
∂²f/∂x∂y
```

**ADDED** - Mathematica-style derivatives:
```kleis
D(f, x)         // Partial derivative
D(f, x, y)      // Mixed partial
Dt(f, x)        // Total derivative (chain rule)
```

**ADDED** - Limit notation:
```kleis
Limit(f, x, a)  // lim_{x→a} f
```

**ADDED** - Function-call alternatives for calculus:
```kleis
Sum(expr, i, 1, n)        // Alternative to Σ_{i=1}^{n}
Product(expr, i, 1, n)    // Alternative to Π_{i=1}^{n}
Integrate(f, x, a, b)     // Alternative to ∫_a^b f dx
```

**Rationale:**
- Function-call syntax is unambiguous to parse
- Follows Mathematica conventions
- Structural editor renders visual ∂f/∂x → D(f, x) for verification

---

## Changes Applied (Dec 12, 2025)

### Grammar v0.6 - Functions in Structures

**Added to `structureMember`:**
```ebnf
structureMember
    ::= operationDecl
      | elementDecl
      | axiomDecl
      | nestedStructure
      | supportsBlock
      | notationDecl
      | functionDef        (* v0.6: Derived operations *)
      ;
```

**Enables:**
- Default implementations of derived operations
- Example: `define (-)(x, y) = x + negate(y)` in Ring structure
- Example: `define (/)(x, y) = x × inverse(y)` in Field structure
- Reduces boilerplate in `implements` blocks

**Resolves:** TODO #11 from parser implementation

---

## Previous Changes (Dec 11, 2025)

### 1. Named Operations Support

**Added to `operatorSymbol`:**
```antlr
operatorSymbol
    : '(' infixOp ')'     // Infix as function: (+), (×)
    | infixOp
    | prefixOp
    | postfixOp
    | IDENTIFIER          // ← NEW: Named operations
    ;
```

**Enables:**
- `operation transpose : Matrix(m,n) → Matrix(n,m)`
- `operation inverse : G → G`
- `operation dot : Vector(n) × Vector(n) → ℝ`

### 2. Custom Operator Support

**Added lexer rule:**
```antlr
CUSTOM_OPERATOR
    : [\u2200-\u22FF]      // Mathematical Operators block
    | [\u2A00-\u2AFF]      // Supplemental Mathematical Operators
    | [\u27C0-\u27EF]      // Miscellaneous Mathematical Symbols-A
    | [\u2980-\u29FF]      // Miscellaneous Mathematical Symbols-B
    ;
```

**Added to `arithmeticOp`:**
```antlr
arithmeticOp
    : '+' | '-' | '×' | '/' | '·' | '*' | '^'
    | '⊗' | '∘' | '∗'
    | CUSTOM_OPERATOR     // ← NEW: User-defined operators
    ;
```

**Enables:**
- `operation (•) : S × S → S` (bullet operator)
- `operation (⊕) : V × V → V` (direct sum)
- `operation (⊙) : R × R → R` (Hadamard product)
- Any Unicode math symbol!

---

## Feature Parity

Both grammars now support:

### Core Language Features
- ✅ Algebraic data types (`data`)
- ✅ Pattern matching (`match`)
- ✅ Structure definitions
- ✅ Implementations (`implements`)
- ✅ Axioms with quantifiers
- ✅ Type system with polymorphism

### Structure Features
- ✅ Custom operators (Unicode math symbols)
- ✅ Named operations (identifiers as operators)
- ✅ `element` keyword
- ✅ `where` clauses
- ✅ `over` clauses
- ✅ `extends` clauses
- ✅ Nested structures
- ✅ **Function definitions in structures (v0.6)**

### Comments
- ✅ Line comments: `// comment`
- ✅ Block comments: `/* comment */`

---

## Testing

Both grammars should accept the same input. Test cases:

### Custom Operators
```kleis
structure Semigroup(S) {
  operation (•) : S × S → S
  axiom associativity: ∀(x y z : S). (x • y) • z = x • (y • z)
}
```

### Named Operations
```kleis
structure Matrix(m: Nat, n: Nat, T) {
  operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
  operation inverse : Matrix(n, n, T) → Matrix(n, n, T)
}
```

### All Features Together (v0.6)
```kleis
structure Ring(R) {
  operation (+) : R × R → R
  operation (×) : R × R → R
  operation negate : R → R
  element zero : R
  element one : R
  
  // Derived operation (v0.6 feature)
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
  
  axiom left_distributivity:
    ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
}

implements Ring(ℤ) {
  operation (+) = builtin_add
  operation (×) = builtin_mul
  operation negate = builtin_negate
  // (-) inherited from structure's default implementation!
}
```

---

## Maintenance

**Going forward:**
- ✅ Update EBNF first (it's the reference)
- ✅ Sync G4 immediately (don't wait 2 days!)
- ✅ Update this status file
- ✅ Test both grammars with same input

**Checklist for grammar changes:**
1. [ ] Update `kleis_grammar_vXX.ebnf` (create new version)
2. [ ] Update `Kleis_vXX.g4` with same changes (create new version)
3. [ ] Create `kleis_grammar_vXX.md` documentation
4. [ ] Update version/date in all files
5. [ ] Sync to `vscode-kleis/docs/grammar/`
6. [ ] Update this GRAMMAR_SYNC_STATUS.md
7. [ ] Test with parser implementation
8. [ ] Run quality gates (fmt, clippy, test)
9. [ ] Commit all files together

---

## Notes

- EBNF is the **reference grammar** (easier to read/maintain)
- G4 is for **ANTLR4 tooling** (can generate parsers)
- Both must stay synchronized for consistency
- Comments already existed in both (no changes needed)

---

## Version History

**v0.93 (Dec 24, 2025):**
- ADDED: `example` blocks for executable documentation
- ADDED: `assert()` statement for verification
- ADDED: REPL `:debug` command (runtime feature)
- Entry points for step-through debugging
- No breaking changes from v0.92

**v0.92 (Dec 22, 2025):**
- ADDED: Type-level arithmetic expressions (2*n, n+1, etc.)
- ADDED: Dimension expressions in type parameters
- Enables dependent dimension types
- No breaking changes from v0.91

**v0.91 (Dec 22, 2025):**
- ADDED: Parameterized type aliases `type Name(params) = Type`
- ADDED: Tuple types `(A, B)` syntax
- No breaking changes from v0.8

**v0.8 (Dec 18, 2025):**
- ADDED: Pattern guards - `n if n < 0 => "negative"`
- ADDED: As-patterns - `Cons(h, t) as whole`
- ADDED: Let destructuring - `let Point(x, y) = p in ...`
- Full Z3 integration for all new features
- No breaking changes from v0.7

**v0.7 (Dec 13, 2025): BREAKING**
- REMOVED: `∂f/∂x` and `df/dx` derivative notation
- ADDED: Mathematica-style `D(f, x)`, `Dt(f, x)`
- ADDED: `Limit(f, x, a)` for limits
- ADDED: Function-call alternatives for Sum, Product, Integrate
- Removed `∂` from prefixOp (no longer standalone prefix)

**v0.6 (Dec 12, 2025):**
- Added `functionDef` to `structureMember`
- Enables derived operations in structures
- Resolves TODO #11

**v0.5.1 (Dec 11, 2025):**
- Added custom operator support (Unicode math symbols)
- Added named operation support

**v0.5 (Dec 8, 2025):**
- Added pattern matching
- Completes ADR-021

---

**Status:** ⚠️ v0.8 EBNF complete, G4 pending sync

