# Kleis Grammar v0.99 - Kinded Type Parameters

**Date:** 2026-02-13  
**Based on:** v0.98 (Parametric Types in Quantifiers)  
**EBNF:** `kleis_grammar_v098.ebnf` (grammar rules unchanged; semantics updated)

## Summary

v0.99 documents **kinded type parameters** for structures, data types, and type aliases. This enables explicit kinds like `Type`, `Nat`, and higher‑kind arrows (`Type → Type`) in type parameter lists.

## Motivation

Kinds make it possible to distinguish:
- `T : Type` (a concrete type)
- `n : Nat` (a dimension parameter)
- `M : Type → Type` (a type constructor)

## New Syntax (v0.99)

```kleis
structure Functor(F : Type → Type) {
    operation fmap : (A → B) → F(A) → F(B)
}
```

```kleis
structure Matrix(m: Nat, n: Nat, T: Type) {
    operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}
```

## Grammar Change

```ebnf
typeParam ::= identifier [ ":" kind ] ;
typeAliasParam ::= identifier [ ":" kind ] ;

kind ::= "Type"
       | "Nat"
       | "String"
       | kind "→" kind
       ;
```

## Version History

- **v0.99 (2026-02-13)**: Kinded type parameters (Type, Nat, Type → Type)
- **v0.98 (2026-01-09)**: Parametric types in quantifiers




