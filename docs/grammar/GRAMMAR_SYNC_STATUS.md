# Grammar Synchronization Status

**Date:** December 11, 2024  
**Status:** ✅ SYNCHRONIZED

## File Versions

| File | Version | Last Updated | Status |
|------|---------|--------------|--------|
| `kleis_grammar_v05.ebnf` | v0.5 | Dec 10, 2024 | ✅ Reference |
| `Kleis_v05.g4` | v0.5.1 | Dec 11, 2024 | ✅ Synced |

---

## Changes Applied (Dec 11, 2024)

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

### Parser Features (Dec 10 additions)
- ✅ Custom operators (Unicode math symbols)
- ✅ Named operations (identifiers as operators)
- ✅ `element` keyword
- ✅ `where` clauses
- ✅ `over` clauses
- ✅ `extends` clauses
- ✅ Nested structures

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

### All Features Together
```kleis
structure VectorSpace(V) over Field(F) {
  operation (+) : V × V → V
  operation (·) : F × V → V
  element zero_v : V
  
  axiom scalar_identity: ∀(v : V). one · v = v
  axiom vector_identity: ∀(v : V). v + zero_v = v
}

implements VectorSpace(Vector(n)) over Field(ℝ) {
  operation (+) = vector_add
  operation (·) = scalar_mul
  element zero_v = zero_vector(n)
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
1. [ ] Update `kleis_grammar_v05.ebnf`
2. [ ] Update `Kleis_v05.g4` with same changes
3. [ ] Update version/date in both files
4. [ ] Update this GRAMMAR_SYNC_STATUS.md
5. [ ] Test with parser implementation
6. [ ] Commit both files together

---

## Notes

- EBNF is the **reference grammar** (easier to read/maintain)
- G4 is for **ANTLR4 tooling** (can generate parsers)
- Both must stay synchronized for consistency
- Comments already existed in both (no changes needed)

---

**Status:** ✅ Grammars are now synchronized as of Dec 11, 2024

