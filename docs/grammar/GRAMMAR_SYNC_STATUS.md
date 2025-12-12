# Grammar Synchronization Status

**Date:** December 12, 2024  
**Status:** ✅ SYNCHRONIZED

## File Versions

| File | Version | Last Updated | Status |
|------|---------|--------------|--------|
| `kleis_grammar_v06.ebnf` | v0.6 | Dec 12, 2024 | ✅ Reference |
| `Kleis_v06.g4` | v0.6 | Dec 12, 2024 | ✅ Synced |
| `kleis_grammar_v06.md` | v0.6 | Dec 12, 2024 | ✅ Documented |
| `vscode-kleis/docs/grammar/kleis_grammar_v06.ebnf` | v0.6 | Dec 12, 2024 | ✅ Synced |

---

## Changes Applied (Dec 12, 2024)

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

## Previous Changes (Dec 11, 2024)

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

**v0.6 (Dec 12, 2024):**
- Added `functionDef` to `structureMember`
- Enables derived operations in structures
- Resolves TODO #11

**v0.5.1 (Dec 11, 2024):**
- Added custom operator support (Unicode math symbols)
- Added named operation support

**v0.5 (Dec 8, 2024):**
- Added pattern matching
- Completes ADR-021

---

**Status:** ✅ Grammars are now synchronized as of Dec 12, 2024 (v0.6)

