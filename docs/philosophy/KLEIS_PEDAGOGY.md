# Kleis Pedagogy: Breaking the Engineering Mindset

> **Session insight from Dec 22, 2024**
> 
> "We engineers are not comfortable with this level of abstraction. We start with IEEE legos, then everything has to be concrete — we cannot break that jail cell."

## The Core Problem

Kleis is mathematics, not programming. But readers will approach it with programmer instincts:

| Engineer asks: | Mathematician says: |
|----------------|---------------------|
| "What IS G?" | "G is just a name" |
| "How is mul implemented?" | "mul is just an arrow: G × G → G" |
| "Where is the data stored?" | "What data?" |
| "But what type is it?" | "Types emerge from implements" |

These questions are **the bars of the jail cell.** They prevent understanding Kleis.

## The Kleis Philosophy

### 1. Names First, Not Types

```kleis
// We don't start with "ℤ is the integers"
// We start with:

G           // a name (nothing more)
mul         // an arrow: G × G → G
identity    // an arrow: Unit → G
```

**There is no a priori knowledge.** No IEEE floats. No memory layout. Just names and arrows.

### 2. Arrows Define Everything

This is Category Theory's core insight:
- Objects are opaque (just labels)
- Morphisms (arrows) are all that matter
- Properties come from how arrows compose

```kleis
structure Group(G) {
    operation mul : G × G → G
    element identity : G
    axiom assoc: ∀(a b c : G). mul(mul(a,b),c) = mul(a,mul(b,c))
}
```

**What is a Group?** Just this pattern of arrows and laws. Nothing more.

### 3. Types Emerge, Not Define

In programming: Define type → Add behavior → Hope it forms a pattern
In Kleis: Define structure → Types are witnesses that fit the pattern

```kleis
// ℤ doesn't "exist" as a Platonic entity
// ℤ is just something that witnesses the Group structure:
implements Group(ℤ) {
    operation mul = builtin_add
    element identity = 0
}
```

### 4. Structure-First, Not Type-First

```
Programming (bottom-up):          Kleis (top-down):
─────────────────────────         ──────────────────
    Patterns                          Structures
        ↑ discover                        ↓ define
    Interfaces                        Arrows (operations)
        ↑ abstract                        ↓ require  
    Methods                           Laws (axioms)
        ↑ add                             ↓ constrain
    Types                             Witnesses (implements)
        ↑ create                          ↓ instantiate
    Data                              Concrete types emerge
```

## Manual Structure (Proposed)

### Part I: The Mindset Shift

1. **Introduction: Why This Isn't a Programming Book**
   - The engineer's jail cell
   - What we must unlearn
   - "No a priori knowledge"

2. **Names and Arrows**
   - Names are just labels (no inherent meaning)
   - Arrows relate names
   - That's all there is

3. **Structures as Patterns**
   - A structure is a pattern of arrows
   - Not "a set with operations"
   - Just: arrows + composition laws

### Part II: Building Up

4. **Axioms: Laws About Arrows**
   - How arrows compose
   - What we can prove from structure alone

5. **Implements: Witnessing a Structure**
   - "ℤ fits the Group pattern"
   - Not "ℤ IS a group" — ℤ witnesses Group

6. **Consequences: What Falls Out**
   - Theorems proved once, valid everywhere
   - The power of abstraction-first

### Part III: Connecting to Reality

7. **The Bridge to Computation**
   - Now we can talk about builtins
   - IEEE floats as witnesses
   - LAPACK as implementations

8. **Why This Matters**
   - Prove once, use everywhere
   - Abstraction prevents bugs
   - Mathematics as the ultimate type system

## Key Phrases to Use

- "Names, not types"
- "Arrows, not methods"
- "Witnesses, not instances"
- "Structures, not classes"
- "No a priori knowledge"
- "All there is are arrows and how they compose"

## Key Phrases to Avoid

- "G is a type" → "G is a name"
- "Create a Group" → "Define the Group structure"
- "ℤ is a Group" → "ℤ witnesses Group"
- "Implement the interface" → "Provide a witness"

## The Goal

> "We really want to knock over the pre-established walls."

The manual should not be a happy-go-lucky tutorial. It should be a **paradigm shift document** that acknowledges:

1. This will feel uncomfortable
2. Your instincts will fight you
3. That discomfort is the jail cell breaking
4. Once through, mathematics opens up

---

*This document captures insights from a session on Dec 22, 2024, discussing the philosophical foundations of Kleis and how to teach them.*

