# Idea: Kleis Program Templates

**Date:** December 13, 2024  
**Status:** 💡 Captured - Not Yet Implemented  
**Priority:** Future consideration

## The Idea

Extend the Kleis renderer beyond expressions to generate **complete Kleis programs**.

Templates would have placeholders for:
- Structure definitions
- Function definitions
- Axioms
- Implementations
- Proofs

## Example Vision

```kleis
// Template: Algebraic Structure
structure □(□) {
    operation □ : □ × □ → □
    
    axiom □: ∀(□ : □). □
}

implements □(□) {
    operation □ = □
}
```

User fills in the boxes → generates valid Kleis program.

## Why It's Cool

1. **Visual Specification Design** - Build formal specs visually
2. **Consistent Structure** - Templates enforce correct syntax
3. **Learning Tool** - Discover structure patterns by example
4. **Rapid Prototyping** - Quick iteration on algebraic structures

## Why It's Scary

1. **Code Generation Scope** - Generating programs, not just expressions
2. **Semantic Complexity** - Placeholders need type awareness
3. **Validation Depth** - Must check generated code is well-formed
4. **Meta-Level Mixing** - Blurs editor ↔ language boundary

## Potential Templates

| Template | Purpose |
|----------|---------|
| `structure_basic` | Simple structure with operation + axiom |
| `structure_extends` | Structure extending another |
| `implements_block` | Implementation for a type |
| `theorem_proof` | Statement + proof skeleton |
| `data_type` | Algebraic data type with variants |

## Technical Considerations

- Would need **structural placeholders** in program AST
- Parsing ↔ Rendering would be bidirectional
- Type inference would need to work with incomplete programs
- Could enable "fill in the proof" workflows

## Related Patterns

- **Literate Programming** (Knuth) - Programs as documents
- **Proof Assistants** (Coq, Lean) - Tactic templates
- **Visual DSLs** - Scratch, Blockly for domain languages

---

**Action:** Captured for future consideration. Don't implement yet.

