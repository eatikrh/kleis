# Kleis Grammar v0.99 - Kinded Type Parameters

**Date:** 2026-02-13  
**Based on:** v0.98 (Parametric Types in Quantifiers)  
**EBNF:** `kleis_grammar_v098.ebnf` (grammar rules unchanged; semantics updated)

## Summary

v0.99 documents **kinded type parameters** for structures, data types, and type aliases. This enables explicit kinds like `Type`, `Nat`, and higher‑kind arrows (`Type → Type`) in type parameter lists.

This is a **documentation + semantic** release: the syntax was already present in EBNF, but kinds were previously treated as opaque strings.

## Motivation

We need to represent type constructors (e.g., `M : Type → Type`) to support higher‑kinded types and structure parameters like:

```kleis
structure Functor(F : Type → Type) {
    operation fmap : (A → B) → F(A) → F(B)
}
```

Kinds make it possible to distinguish:
- `T : Type` (a concrete type)
- `n : Nat` (a dimension parameter)
- `M : Type → Type` (a type constructor)

## New Syntax (v0.99)

### Kinded type params in structures

```kleis
structure Matrix(m: Nat, n: Nat, T: Type) {
    operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}
```

### Higher‑kinded params

```kleis
structure Monad(M : Type → Type) {
    operation unit : A → M(A)
    operation bind : M(A) → (A → M(B)) → M(B)
}
```

### Kinded type params in type aliases

```kleis
type Endo(F : Type) = F → F
type FunctorF(F : Type → Type) = F
```

## Grammar Change

The EBNF already allowed `kind` in `typeParam` and `typeAliasParam`, but v0.99 **formalizes the intended meaning**:

```ebnf
typeParam ::= identifier [ ":" kind ] ;
typeAliasParam ::= identifier [ ":" kind ] ;

kind ::= "Type"
       | "Nat"
       | "String"
       | kind "→" kind
       ;
```

## Notes

- This version **does not** introduce new surface syntax beyond kind annotations.
- Type‑level equality constraints (e.g., `n = m`) remain semantic checks; no new grammar is added for them.

## Version History

- **v0.99 (2026-02-13)**: Kinded type parameters (Type, Nat, Type → Type)
- **v0.98 (2026-01-09)**: Parametric types in quantifiers
- **v0.97 (2026-01-05)**: ASCII logical operators




