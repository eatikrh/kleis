# Grammar v0.6 - Adding Function Definitions to Structures
**Date:** December 12, 2025  
**Related TODO:** #11 - `src/kleis_parser.rs:1758`  
**Decision:** Add `functionDef` to `structureMember` production

---

## 📋 Problem Statement

**Current Situation:**
- Grammar v0.5 does NOT allow `define` (function definitions) inside structures
- `prelude.kleis` USES `define` inside structures for derived operations
- Parser currently **skips** these silently (TODO #11)

**Example from `prelude.kleis`:**
```kleis
structure Ring(R) {
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)  // ← This is useful but not in grammar!
}

structure Field(F) extends Ring(F) {
  operation (/) : F × F → F
  define (/)(x, y) = x × inverse(y)  // ← Same here
}
```

---

## 🎯 Use Case Analysis

### Pattern: Declared Operations with Derived Implementations

**Ring Structure:**
```kleis
structure Ring(R) {
  // Addition and multiplication are abstract (defined by implements blocks)
  operation (+) : R × R → R
  operation (×) : R × R → R
  operation negate : R → R
  
  // Subtraction is DERIVED from addition and negation
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)  // Default implementation!
}
```

**Why this is valuable:**
1. **Type signature declared** - `operation (-) : R × R → R` specifies the interface
2. **Default implementation** - `define (-)(x, y) = ...` provides a standard definition
3. **Can be overridden** - `implements` blocks can provide custom implementations
4. **Reduces boilerplate** - Don't need to implement (-) for every Ring

### Comparison with `notation`

**Grammar v0.5 provides `notation`:**
```ebnf
notationDecl ::= "notation" identifier "(" params ")" "=" expression ;
```

**But `notation` is different:**
- `notation` is syntactic sugar (display/input convenience)
- `define` provides actual operational semantics
- `define` can be type-checked and verified

**Example distinction:**
```kleis
structure Matrix(m, n, T) {
  operation transpose : Matrix(n, m, T)
  
  // notation for syntactic convenience (rendering)
  notation T() = transpose
  
  // define for semantic behavior (computation)
  define identity() = Matrix(n, n, diagonal_ones)
}
```

---

## ✅ Decision: Add `functionDef` to Grammar v0.6

### Rationale

1. **Already in use** - `prelude.kleis` uses this pattern effectively
2. **Semantically sound** - Derived operations are a core algebraic concept
3. **Type-safe** - Function definitions can be type-checked against operation signatures
4. **Extensible** - Allows default implementations that can be specialized
5. **Parser ready** - Just needs to stop skipping (TODO #11)

### Grammar Change

**Before (v0.5):**
```ebnf
structureMember
    ::= operationDecl
      | elementDecl
      | axiomDecl
      | nestedStructure
      | supportsBlock
      | notationDecl
      ;
```

**After (v0.6):**
```ebnf
structureMember
    ::= operationDecl
      | elementDecl
      | axiomDecl
      | nestedStructure
      | supportsBlock
      | notationDecl
      | functionDef        (* NEW v0.6: Derived operations *)
      ;
```

---

## 🔄 Semantic Interpretation

### How `define` in structures should work:

1. **Scope:** Function is scoped to the structure (like an operation)
2. **Binding:** Available within the structure and in `implements` blocks
3. **Override:** `implements` blocks can override with custom implementations
4. **Type checking:** Must match the operation signature if one exists

**Example:**
```kleis
structure Ring(R) {
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
}

implements Ring(ℤ) {
  // Can use default (-) OR override:
  operation (-) = builtin_int_subtract  // Custom implementation
}
```

---

## 📝 Implementation Checklist

### Grammar Updates

- [ ] Update `docs/grammar/kleis_grammar_v05.ebnf` → `kleis_grammar_v06.ebnf`
- [ ] Update `docs/grammar/Kleis_v05.g4` → `Kleis_v06.g4`
- [ ] Update `docs/grammar/kleis_grammar_v05.md` → `kleis_grammar_v06.md`
- [ ] Update `vscode-kleis/docs/grammar/kleis_grammar_v05.ebnf` → `kleis_grammar_v06.ebnf`
- [ ] Add changelog entry documenting the change

### Parser Updates

- [ ] Update `src/kleis_ast.rs` - Add `FunctionDef` variant to `StructureMember` enum
- [ ] Update `src/kleis_parser.rs` - Parse `functionDef` instead of skipping (line 1758)
- [ ] Remove TODO #11 comment

### Type System Updates

- [ ] Update `src/type_context.rs` - Register function definitions from structures
- [ ] Ensure function definitions can be type-checked against operation signatures

### Tests

- [ ] Verify `prelude.kleis` parses correctly (Ring, Field structures)
- [ ] Add test for function definitions in structures
- [ ] Verify VSCode extension highlights correctly

### Documentation

- [ ] Update grammar documentation with v0.6 changelog
- [ ] Document the pattern in a design doc or ADR if needed
- [ ] Update examples to use this feature

---

## 🎨 VSCode Syntax Highlighter

**Already works!** ✅

The syntax highlighter already includes `define` as a keyword:
```json
"match": "\\b(data|structure|implements|operation|element|axiom|supports|notation|define|let|in|if|then|else|match|type|object|const|narrow|over|extends|verify|where)\\b"
```

No VSCode changes needed! The highlighter will work out of the box.

---

## 🔍 Related Features

### Future Extensions

1. **Guards on function definitions:**
   ```kleis
   define abs(x) where x ≥ 0 = x
   define abs(x) where x < 0 = -x
   ```

2. **Multiple definitions (dispatch):**
   ```kleis
   define factorial(0) = 1
   define factorial(n) = n × factorial(n - 1)
   ```

3. **Type-directed implementations:**
   ```kleis
   define zero : T where Ring(T) = T.additive.zero
   ```

These are for future versions, but the v0.6 foundation supports them.

---

## 📊 Impact Analysis

### Affected Components

| Component | Impact | Status |
|-----------|--------|--------|
| Grammar (EBNF) | One line added | Update needed |
| Grammar (ANTLR) | One line added | Update needed |
| Grammar (MD) | Documentation | Update needed |
| VSCode Grammar | Sync with main | Copy file |
| Parser AST | Add enum variant | Update needed |
| Parser | Parse instead of skip | Update needed |
| Type Context | Register functions | Update needed |
| VSCode Highlighter | None (already works) | ✅ Ready |
| Prelude | None (already uses it) | ✅ Ready |

### Compatibility

- **Backward compatible:** Existing structures without `define` still work
- **Forward compatible:** New structures can use `define` for derived operations
- **No breaking changes:** This is purely additive

---

## ✅ Approval

**Decision:** Proceed with Grammar v0.6 update.

**Justification:**
1. Feature is already in use (prelude.kleis)
2. Semantically sound (derived operations are standard in algebra)
3. Minimal implementation cost (parser just needs to stop skipping)
4. No breaking changes (additive only)
5. VSCode support already present

**Next Step:** Update grammar files and implement parser support.

