# Kleis Grammar v0.98 - Parametric Types in Quantifiers

**Date:** 2026-01-09  
**Based on:** v0.97 (ASCII Logical Operators)  
**EBNF:** `kleis_grammar_v098.ebnf`

## Summary

v0.98 enables **parametric types** (types with arguments) in universal/existential quantifier type annotations. This is essential for differential geometry axioms where we need to quantify over tensors of specific rank.

## Inherited from v0.97

v0.98 includes all features from v0.97:
- **ASCII logical operators**: `and`, `or`, `not` work everywhere (not just let bindings)
- These are **reserved keywords** (cannot be used as identifiers)

| Unicode | ASCII | Description |
|---------|-------|-------------|
| `∧` | `and` | Logical conjunction |
| `∨` | `or` | Logical disjunction |
| `¬` | `not` | Logical negation |

## Motivation

The ability to use parametric types in quantifiers was **already implemented** but undocumented. This grammar version clarifies and officially documents this capability:

```kleis
// ✅ WORKS (always did, now documented)
axiom ricci_symmetric : ∀ R : Tensor(0, 2, dim, ℝ) .
    ∀ μ : Nat . ∀ ν : Nat .
    component(R, μ, ν) = component(R, ν, μ)

// ✅ Simple types (worked since early versions)
axiom simple_example : ∀ x : Nat . x = x
```

This capability is essential for properly axiomatizing differential geometry tensors - specifying that a quantified variable is a (0,2)-tensor vs a (1,3)-tensor.

## New Syntax (v0.98)

```kleis
// Parametric types in quantifiers now work:
axiom ricci_symmetric : ∀ R : Tensor(0, 2, dim, ℝ) .
    ∀ μ : Nat . ∀ ν : Nat .
    component(R, μ, ν) = component(R, ν, μ)

// Matrix types:
axiom matrix_add_commutative : ∀ A : Matrix(m, n, ℝ) . ∀ B : Matrix(m, n, ℝ) .
    matrix_add(A, B) = matrix_add(B, A)

// Function types (already worked in v0.97):
axiom derivative_linear : ∀ f : ℝ → ℝ . ∀ g : ℝ → ℝ .
    D(plus(f, g), x) = plus(D(f, x), D(g, x))

// Nested parametric types:
axiom block_matrix : ∀ M : Matrix(2, 2, Matrix(3, 3, ℝ)) .
    transpose(transpose(M)) = M
```

## Grammar Change

### Previous (v0.97)

```ebnf
quantifierType
    ::= identifier                           (* Simple: Nat, ℝ, Bool *)
      | identifier "(" typeArgs ")"          (* Parametric: Vector(n) *)
      | "(" quantifierType ")"               (* Grouped *)
      | quantifierType "→" quantifierType    (* Function type *)
      ;

typeArgs
    ::= (* handled by nested paren matching *)
      ;
```

**Problem:** The parser used simple character-by-character paren matching for `typeArgs`, which didn't properly handle the grammar. The issue was in tokenization, not the EBNF itself.

### New (v0.98)

```ebnf
quantifierType
    ::= simpleQuantifierType
      | simpleQuantifierType "→" quantifierType    (* Function type, right-associative *)
      ;

simpleQuantifierType
    ::= identifier                                 (* Simple: Nat, ℝ, Bool *)
      | identifier "(" typeArgList ")"             (* Parametric: Tensor(0,2,dim,ℝ) *)
      | "(" quantifierType ")"                     (* Grouped/tuple *)
      ;

typeArgList
    ::= typeArg { "," typeArg }
      ;

typeArg
    ::= expression                                 (* Nat values, type references, etc. *)
      | quantifierType                             (* Nested types *)
      ;
```

## Key Improvements

1. **Proper type argument parsing**: Instead of character-level paren matching, use recursive descent with proper expression parsing
2. **Nested types**: `Matrix(2, 2, Matrix(3, 3, ℝ))` parses correctly
3. **Mixed arguments**: `Tensor(0, 2, dim, ℝ)` handles both numeric and type arguments

## Parser Implementation

**Discovery:** The parser already supported parametric types in quantifiers via the existing `parse_type_annotation_for_quantifier` function in `src/kleis_parser.rs`. This function uses recursive paren matching to handle nested types like `Tensor(0, 2, dim, ℝ)`.

v0.98 adds explicit tests to verify this capability:
- `test_parse_quantifier_parametric_type_no_paren`
- `test_parse_quantifier_parametric_type_with_paren`
- `test_parse_quantifier_matrix_type`

The feature was already working but was previously undocumented.

## Examples

### Differential Geometry

```kleis
structure RicciTensor(dim: Nat) {
    // Now works with parametric tensor types
    axiom ricci_symmetric : ∀ R : Tensor(0, 2, dim, ℝ) .
        ∀ μ : Nat . ∀ ν : Nat .
        component(R, μ, ν) = component(R, ν, μ)
    
    axiom ricci_from_riemann : ∀ Riem : Tensor(1, 3, dim, ℝ) .
        ∀ μ : Nat . ∀ ν : Nat .
        component(ricci(Riem), μ, ν) = contract(Riem, 0, 2, μ, ν)
}
```

### Matrix Algebra

```kleis
structure MatrixAxioms(m: Nat, n: Nat) {
    axiom add_comm : ∀ A : Matrix(m, n, ℝ) . ∀ B : Matrix(m, n, ℝ) .
        matrix_add(A, B) = matrix_add(B, A)
    
    axiom transpose_involutive : ∀ A : Matrix(m, n, ℝ) .
        transpose(transpose(A)) = A
}
```

### Complex Types

```kleis
structure ComplexMatrixAxioms(n: Nat) {
    // ComplexMatrix is a pair (real_part, imag_part)
    axiom dagger_involutive : ∀ M : ComplexMatrix(n, n) .
        dagger(dagger(M)) = M
}
```

## Backward Compatibility

| Aspect | Compatible? |
|--------|-------------|
| Simple types (`∀ x : Nat`) | ✅ Unchanged |
| Function types (`∀ f : ℝ → ℝ`) | ✅ Unchanged |
| Parametric types (`∀ T : Tensor(0,2,dim,ℝ)`) | ✅ **Now works** |
| Existing stdlib axioms | ✅ All pass |

## Impact on Other Components

| Component | Changes Needed |
|-----------|----------------|
| Parser (`kleis_parser.rs`) | ✅ Modified |
| Type checker | None - already handles parametric types |
| Z3 backend | None - type annotations are metadata |
| vscode-kleis | ✅ Update syntax highlighting |
| Documentation | ✅ Update grammar references |

## Version History

- **v0.98 (2026-01-09)**: Parametric types in quantifiers
- **v0.97 (2026-01-05)**: ASCII logical operators (`and`, `or`, `not`)
- **v0.96 (2026-01-01)**: Named arguments for function calls
- **v0.95 (2025-12-29)**: Big operator syntax (Σ, Π, ∫, lim)
- **v0.94 (2025-12-26)**: N-ary product types
- **v0.93 (2025-12-24)**: Example blocks and assert statements

