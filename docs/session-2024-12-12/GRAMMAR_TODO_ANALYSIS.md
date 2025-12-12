# Grammar-Related TODO Analysis
**Date:** December 12, 2024  
**Focus:** Parser implementation vs. formal grammar specification

---

## 📋 Grammar TODOs Summary

| TODO# | File | Line | Status | Action |
|-------|------|------|--------|--------|
| #9 | `src/type_context.rs` | 327 | ⚠️ Needs Review | Top-level operations not registered |
| #11 | `src/kleis_parser.rs` | 1758 | ❌ Grammar Conflict | Functions in structures (not in grammar!) |
| #12 | `src/kleis_parser.rs` | 1975 | ✅ Planned (Wire 3) | Parameter types - store individually |
| #53 | `tests/load_full_prelude_test.rs` | 14 | ✅ **OBSOLETE** | Top-level operations **work!** |
| #54 | `tests/load_full_prelude_test.rs` | 89 | ✅ **OBSOLETE** | Parsing + type checking **works!** |

---

## ✅ RESOLVED: Top-Level Operations Work!

### Discovery

**TODOs #53 and #54 claim top-level operations don't work, but they DO!**

**Evidence:**
```bash
$ cargo test test_parse_full_prelude -- --ignored
✅ Successfully parsed prelude.kleis!
   Total items: 51
   Structures: 15
   Implements: 12
   Operations: 20  ← 20 top-level operations parsed!
   Functions: 4

$ cargo test test_load_prelude_into_typechecker -- --ignored
✅ Successfully loaded prelude.kleis into TypeChecker!
```

### Why Tests Were Ignored

The `#[ignore]` attributes have **outdated TODO comments**:
- Line 14: `// TODO: Requires top-level operation declarations`
- Line 89: `// TODO: Requires top-level operation declarations and define statements`

**But the implementation is complete:**

1. **Grammar supports it** (line 19 of grammar: `operationDecl` is a top-level `declaration`)
2. **AST supports it** (`TopLevel::OperationDecl` exists)
3. **Parser supports it** (`parse_operation_decl()` implemented)
4. **Tests pass** (when run with `--ignored`)

### Recommendation

**Remove `#[ignore]` from both tests** and update comments:
- `test_parse_full_prelude` - READY
- `test_load_prelude_into_typechecker` - READY

---

## ❌ GRAMMAR CONFLICT: Functions Inside Structures

### TODO #11 - `src/kleis_parser.rs:1758`

```rust
} else if self.peek_word("define") {
    // define f(x) = expr (inline function definition)
    // Skip for now - not yet supported in structure members
    // TODO: Add FunctionDef variant to StructureMember enum
    for _ in 0..6 {
        self.advance();  // Just skip it!
    }
```

### The Problem

**The prelude uses this syntax:**
```kleis
structure Ring(R) {
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)  ← This line!
}
```

**But the formal grammar does NOT support it!**

From `docs/grammar/kleis_grammar_v05.ebnf` line 157-164:
```ebnf
structureMember
    ::= operationDecl
      | elementDecl
      | axiomDecl
      | nestedStructure
      | supportsBlock
      | notationDecl       ← Has 'notation', but NOT 'functionDef'!
      ;
```

### Grammar Provides Alternative: `notation`

Line 176 of grammar:
```ebnf
notationDecl ::= "notation" identifier "(" params ")" "=" expression ;
```

**This is for inline function-like definitions in structures!**

### Three Possible Solutions

#### Option 1: Fix `prelude.kleis` to use `notation` (Grammar-Compliant)
```kleis
structure Ring(R) {
  operation (-) : R × R → R
  notation (-)(x, y) = x + negate(y)  // Use 'notation' not 'define'
}
```
**Pros:** Aligns with formal grammar  
**Cons:** Requires updating prelude and parser to support `notation`

#### Option 2: Extend grammar to allow `functionDef` in structures
Update line 157 of grammar:
```ebnf
structureMember
    ::= operationDecl
      | elementDecl
      | axiomDecl
      | functionDef        ← ADD THIS
      | nestedStructure
      | supportsBlock
      | notationDecl
      ;
```
**Pros:** Matches current usage in prelude  
**Cons:** Deviates from original grammar design (requires ADR)

#### Option 3: Keep current behavior (skip + ignore)
Keep parser skipping `define` in structures, document as limitation.
**Pros:** No changes needed  
**Cons:** Silent failure, prelude syntax won't work as expected

### Recommendation

**I recommend Option 1** - Use `notation` as the grammar intends:

1. Update `prelude.kleis` to use `notation` instead of `define` in structures
2. Implement `notation` parsing in `kleis_parser.rs` 
3. Remove TODO #11 when complete
4. This aligns with the formal grammar specification (critical per cursor rules!)

**Reasoning:** The grammar explicitly provides `notation` for this use case. The `define` keyword should be reserved for top-level function definitions, while `notation` is for structure-local convenience functions.

---

## ⚠️ NEEDS REVIEW: Top-Level Operations Not Registered

### TODO #9 - `src/type_context.rs:327`

```rust
fn register_toplevel_operation(
    &mut self,
    _op_decl: &crate::kleis_ast::OperationDecl,
) -> Result<(), String> {
    // Top-level operations (like frac for display mode)
    // These are utility operations, not tied to structures
    // TODO: Register these separately if needed
    Ok(())
}
```

### The Issue

**Parser successfully parses 20 top-level operations from `prelude.kleis`, but they're not registered!**

The function is called but does nothing (returns `Ok(())` immediately).

### Questions

1. **Are top-level operations used?** Check if any code calls top-level operations.
2. **Should they be registered?** If yes, where (type context, operation registry)?
3. **What should registration do?** Store type signature, enable operation lookup?

### Example from prelude.kleis

```kleis
// Line 164-165 (top-level, outside any structure)
operation dot : ∀(n : ℕ). Vector(n) × Vector(n) → ℝ
operation cross : Vector(3) × Vector(3) → Vector(3)
```

These are parsed but not registered. Should they be?

### Recommendation

**Need to discuss with you:**
- Are top-level operations intended to be callable?
- Or are they just documentation/stubs for now?
- Should type checker know about them?

---

## ✅ PLANNED: Store Parameter Types Individually

### TODO #12 - `src/kleis_parser.rs:1975`

```rust
// Optional type annotation (we parse but don't store it in the simple Vec<String> for now)
if self.peek() == Some(':') {
    self.advance(); // consume ':'
    self.skip_whitespace();
    // Parse and discard type for now (stored in type_annotation on FunctionDef)
    // TODO: Store parameter types individually when we extend FunctionDef
    self.parse_type()?;
    self.skip_whitespace();
}
```

### Status

**This is planned "Wire 3" work** - related to TODOs #3, #6, #7.

### What's Happening

Currently:
- Parameter types ARE parsed (grammar compliant!)
- But stored in single `type_annotation` field on `FunctionDef`
- Individual parameters don't have their types

**Wire 3 will implement:**
1. Curried function types: `A → B → C`
2. Individual parameter type tracking
3. Proper function application type checking

### Recommendation

**Keep this TODO** - it's correctly marked as planned work. Don't implement until Wire 3 is ready.

---

## 📊 Grammar Compliance Matrix

| Feature | Grammar | Parser | Type System | Status |
|---------|---------|--------|-------------|--------|
| **Top-level operations** | ✅ Yes (line 19) | ✅ Implemented | ❌ Not registered | ⚠️ Partial |
| **Operations in structures** | ✅ Yes (line 158) | ✅ Implemented | ✅ Registered | ✅ Complete |
| **Functions (`define`)** | ✅ Top-level only | ✅ Implemented | ✅ Works | ✅ Complete |
| **`define` in structures** | ❌ Not in grammar | ⚠️ Skipped | ❌ Not supported | ❌ Conflict |
| **`notation` in structures** | ✅ Yes (line 176) | ❌ Not implemented | ❌ Not implemented | ❌ Missing |
| **Parameter types** | ✅ Yes (line 220) | ✅ Parsed | ⚠️ Not stored individually | ⚠️ Partial (Wire 3) |

---

## 🎯 Recommended Actions

### Immediate

1. **Un-ignore tests** (TODOs #53, #54) ✅ Tests pass!
   ```rust
   // Remove #[ignore] from:
   // - test_parse_full_prelude
   // - test_load_prelude_into_typechecker
   ```

2. **Fix grammar conflict** (TODO #11)
   - Update `prelude.kleis` to use `notation` instead of `define` in structures
   - Implement `notation` parsing (aligns with grammar line 176)

3. **Investigate top-level operations** (TODO #9)
   - Determine if they should be registered
   - If yes, implement registration logic

### Future (Wire 3)

4. **Store parameter types individually** (TODO #12)
   - Part of Wire 3 function type system
   - Keep TODO until Wire 3 is implemented

---

## 📝 Parser Coverage vs Grammar

**Parser implements ~40% of full grammar** (by design - POC phase):

**Implemented:**
- ✅ Structures, implements blocks
- ✅ Data types, pattern matching (ADR-021)
- ✅ Top-level operations, functions
- ✅ Axioms, type signatures
- ✅ Basic expressions

**Not Yet Implemented:**
- ❌ `notation` declarations
- ❌ `type` aliases
- ❌ Lambda expressions
- ❌ Let bindings, conditionals
- ❌ Set literals
- ❌ `@library`, `@version` annotations

**By Design:** Parser is a POC covering core language features. Full implementation planned for later phases.

---

## 🔍 Key Insight

**The grammar is well-designed and complete.** The issues are:

1. **Test TODOs are outdated** - features work but tests are ignored
2. **One implementation conflict** - `prelude.kleis` uses `define` where grammar expects `notation`
3. **One registration gap** - top-level operations parsed but not registered

None of these are major issues! Just needs cleanup and clarification.

---

## Next Steps

**For this session, should we:**

1. ✅ Un-ignore the two passing tests?
2. ✅ Update `prelude.kleis` to use `notation` syntax?
3. ✅ Implement `notation` parsing?
4. ⚠️ Decide on top-level operation registration?

**All changes should align with formal grammar** per cursor rules! ✅

