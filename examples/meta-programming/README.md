# Meta-Programming Examples

**These examples demonstrate Kleis's most powerful capability: self-hosting and language definition.**

---

## 🌟 Why This Matters

Most specification languages can describe data structures. **Kleis can describe itself.**

This isn't just a party trick - it proves:
1. **Kleis is expressive enough** to define programming language semantics
2. **The grammar is consistent** - no hidden magic in the parser
3. **Users can define their own DSLs** within Kleis

---

## Files

### `kleis_in_kleis.kleis` (26KB)
**Kleis defining its own grammar and semantics.**

```kleis
-- The Kleis language, defined in Kleis
data KleisExpr = 
    | KConst(value)
    | KVar(name)  
    | KOp(name, args : List(KleisExpr))
    | KLambda(param, body : KleisExpr)
    | KApply(func : KleisExpr, arg : KleisExpr)
    ...
```

This is **self-hosting**: the language is its own metalanguage.

### `lisp_parser.kleis` (20KB)
**A complete LISP parser written in Kleis.**

Parses S-expressions:
```kleis
-- Tokenizer for LISP
data Token = LParen | RParen | Symbol(String) | Number(ℤ)

-- Parser produces LISP AST
data SExpr = Atom(String) | List(List(SExpr))
```

### `lisp_in_kleis.kleis` (17KB)
**LISP semantics and evaluation in Kleis.**

Defines LISP operations:
```kleis
-- LISP primitives
operation car : SExpr → SExpr
operation cdr : SExpr → SExpr
operation cons : SExpr × SExpr → SExpr

-- Evaluation
operation eval : SExpr × Environment → SExpr
```

---

## The Power of Meta-Programming

With these primitives, you can:

1. **Define your own DSLs** - Create domain-specific notation
2. **Verify language properties** - Prove your DSL has certain guarantees
3. **Translate between formalisms** - LISP ↔ Kleis ↔ Z3
4. **Bootstrap new languages** - Use Kleis as a language laboratory

---

## Related ADRs

- **ADR-003:** Self-Hosting Strategy
- **ADR-007:** Bootstrap Grammar
- **ADR-008:** Bootstrap Grammar Boundary

