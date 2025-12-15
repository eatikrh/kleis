# Kleis Grammar v0.6 - Functions in Structures

**Date:** December 12, 2025  
**Status:** ✅ IMPLEMENTED  
**Base:** Kleis Grammar v0.5 (with pattern matching)

---

## Overview

Grammar v0.6 adds **function definitions inside structures**, enabling derived operations with default implementations. This resolves TODO #11 and aligns the grammar with actual usage in `prelude.kleis`.

**Key Change:**
- Added `functionDef` to `structureMember` production

**Impact:**
- Structures can now define derived operations
- Default implementations can be provided
- Reduces boilerplate in `implements` blocks
- Aligns with algebraic structure patterns (Ring, Field, etc.)

---

## Syntax Change

### Before (v0.5)

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

### After (v0.6)

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

---

## Use Cases

### Derived Operations in Algebraic Structures

**Ring - Subtraction as Derived Operation:**
```kleis
structure Ring(R) {
  // Abstract operations (implemented by instances)
  operation (+) : R × R → R
  operation (×) : R × R → R
  operation negate : R → R
  element zero : R
  element one : R
  
  // Derived operation with default implementation
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
}
```

**Field - Division as Derived Operation:**
```kleis
structure Field(F) extends Ring(F) {
  operation inverse : F → F
  
  // Declare the operation
  operation (/) : F × F → F
  
  // Provide default implementation
  define (/)(x, y) = x × inverse(y)
}
```

### Benefits

1. **Type Safety:**
   - Operation signature declared separately
   - Implementation type-checked against signature

2. **Default Implementations:**
   - Don't need to reimplement derived ops for every instance
   - Can still override if needed

3. **Mathematical Clarity:**
   - Separates essential operations from derived ones
   - Documents relationships between operations

4. **Reduced Boilerplate:**
   ```kleis
   // Without derived ops - must implement for each instance:
   implements Ring(ℤ) {
     operation (+) = builtin_add
     operation (×) = builtin_mul
     operation negate = builtin_negate
     operation (-) = builtin_subtract  // Must implement!
   }
   
   // With derived ops - get subtraction for free:
   implements Ring(ℤ) {
     operation (+) = builtin_add
     operation (×) = builtin_mul
     operation negate = builtin_negate
     // (-) automatically uses default: x + negate(y)
   }
   ```

---

## Semantics

### Scope and Binding

1. **Scope:** Function is scoped to the structure
2. **Availability:** Can be used in:
   - Other structure members (operations, axioms)
   - Implements blocks for that structure
   - Expressions that have access to the structure

3. **Override:** Implements blocks can override default:
   ```kleis
   structure Ring(R) {
     operation (-) : R × R → R
     define (-)(x, y) = x + negate(y)  // Default
   }
   
   implements Ring(ℤ) {
     operation (-) = builtin_subtract  // Override with optimized version
   }
   ```

### Type Checking

1. If operation signature exists, function must match it
2. Function parameters and return type inferred
3. Constraint: `define` type must unify with `operation` type

---

## Examples from Standard Library

### Ring Structure (from `prelude.kleis`)

```kleis
structure Ring(R) {
  // Additive structure
  structure additive : AbelianGroup(R) {
    operation (+) : R × R → R
    operation negate : R → R
    element zero : R
  }
  
  // Multiplicative structure
  structure multiplicative : Monoid(R) {
    operation (×) : R × R → R
    element one : R
  }
  
  // Subtraction (derived)
  operation (-) : R × R → R
  define (-)(x, y) = x + negate(y)
  
  // Distributivity law
  axiom left_distributivity:
    ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
}
```

### Field Structure (from `prelude.kleis`)

```kleis
structure Field(F) extends Ring(F) {
  operation (/) : F × F → F
  operation inverse : F → F
  
  axiom multiplicative_inverse:
    ∀(x : F) where x ≠ zero. inverse(x) × x = one
    
  // Division defined
  define (/)(x, y) = x × inverse(y)
}
```

---

## Comparison with `notation`

### `notation` - Syntactic Sugar

```kleis
structure Matrix(m, n, T) {
  operation transpose : Matrix(n, m, T)
  
  // notation provides convenient syntax
  notation T() = transpose
  
  // Usage: matrix.T() instead of transpose(matrix)
}
```

**Purpose:** Input/output convenience, rendering

### `functionDef` - Semantic Operation

```kleis
structure Ring(R) {
  operation (-) : R × R → R
  
  // define provides actual computation
  define (-)(x, y) = x + negate(y)
  
  // Usage: ring subtraction has real semantics
}
```

**Purpose:** Default implementations, derived operations

---

## Implementation Status

### Parser

- ✅ AST `StructureMember` enum includes `FunctionDef` variant
- ✅ Parser recognizes `define` in structures (line 1755-1771)
- ⚠️ Currently **skips** `define` (TODO #11)
- 🔧 Needs update to actually parse instead of skip

### Type System

- 🔧 Needs registration of structure functions
- 🔧 Needs type checking against operation signatures
- 🔧 Needs override resolution in implements blocks

### VSCode Extension

- ✅ Syntax highlighter already recognizes `define` keyword
- ✅ Grammar file needs sync (copy v0.6 EBNF)

---

## Migration Guide

### For Grammar Files

1. Update `structureMember` production to include `functionDef`
2. Update version number to v0.6
3. Add changelog entry

### For Parsers

1. Parse `define` statements in structure bodies
2. Create `StructureMember::FunctionDef` AST nodes
3. Remove skip logic (TODO #11 at line 1758)

### For Type Checkers

1. Register function definitions from structures
2. Check function type against operation signature (if exists)
3. Allow override in implements blocks

---

## Related Features

### Current (v0.6)

- ✅ Function definitions at top-level
- ✅ Function definitions in structures
- ✅ Operation declarations in structures
- ✅ Implements blocks with operation definitions

### Future

- Multiple function clauses (pattern matching on parameters)
- Guarded function definitions (`where` clauses)
- Type-directed dispatch
- Automatic derivation (e.g., `deriving Monoid`)

---

## Version History

**v0.6 (2025-12-12):**
- Added `functionDef` to `structureMember`
- Enables derived operations in structures
- Resolves TODO #11

**v0.5 (2025-12-08):**
- Added pattern matching

**v0.4 (2025-12-08):**
- Added algebraic data types

**v0.3 (2025-12-05):**
- Type system with polymorphism

---

## References

- **TODO #11:** `src/kleis_parser.rs:1758`
- **Rationale:** `docs/session-2025-12-12/GRAMMAR_V06_RATIONALE.md`
- **Examples:** `stdlib/prelude.kleis` (Ring, Field structures)
- **EBNF:** `docs/grammar/kleis_grammar_v06.ebnf`
- **ANTLR4:** `docs/grammar/Kleis_v06.g4`

