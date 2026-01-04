# The Origin of Kleis: From Cosmology to Universal Verification

**A Development Story for the Anti "Mythical Man-Month"**

---

## The Beginning: Equations That Needed Checking

It started with Projected Ontology Theory (POT).

I was working on integral transformations from Hilbert Ontology (ℋₒₙₜ) to ℝ⁴ 
spacetime. The equations lived in `examples/ontology/revised/`. I had ideas 
about the codomain of the modal flow in ℋₒₙₜ, but I had no way to verify 
whether the equations I was writing down actually made mathematical sense.

```
Π[ψ](x) = ∫_ℋ K(x,m) ψ(m) dm
```

Did the types match? Were the dimensions consistent? Was I accidentally 
adding a scalar to a tensor? LaTeX would render it beautifully. But LaTeX 
couldn't tell me if it was *valid*.

**I needed type checking for my equations.**

---

## The Crystallization: HARD_PROBLEMS_AHEAD.md

After building the structural equation editor, I confronted what the project
really needed to become. Not just an equation editor, but a complete language
with:

- Type inference (Hindley-Milner level complexity)
- Polymorphic dispatch (same symbol, different operations)
- Context management (scoping, imports)
- Evaluation engine
- User extensibility

The document `docs/reference/HARD_PROBLEMS_AHEAD.md` estimated this would take
4-5 years of additional work.

**It took months.**

---

## The Methodology: Why It Worked

### 1. NEXT_SESSION.md as Persistent Memory

LLMs have no memory between sessions. But I needed continuity across hundreds
of sessions building a complex system.

Solution: `docs/NEXT_SESSION.md` - a living document that carries context.
At the start of each session, the LLM reads it. At the end, we update it.
The document grew to 2,800+ lines - the complete state of the project.

**No GitHub issues. No ticket fragmentation. One flowing document.**

### 2. Rules Enforcement (.cursorrules)

Hard-won lessons, codified:
- "Never push without explicit permission"
- "Read code before documenting"
- "Check grammar before parser changes"
- "Run quality gates before commit"

The LLM doesn't remember between sessions, but the rules file does.

### 3. ADRs as Decision Memory

29 Architecture Decision Records documenting *why* things are the way they are.
Not just code, but reasoning. When the LLM asks "should we do X?", I can 
point to ADR-014 and say "we decided this already, here's why."

### 4. Domain Teaching

The LLM learned:
- Tensor notation and index gymnastics
- Typst rendering pipeline
- Z3 SMT solving
- Projected Ontology Theory
- Hindley-Milner type inference

It became a specialized collaborator, not a generic code generator.

### 5. Research-Driven Development

The equations I needed to verify drove every feature:
- Christoffel symbols → tensor template system
- Dimensional consistency → type checking
- "Does this make sense?" → Z3 verification

**Actual research need prevented abstraction drift.**

---

## Avoiding the Mythical Man-Month Pitfalls

| Brooks' Pitfall | Traditional Team | Our LLM Collaboration |
|-----------------|------------------|----------------------|
| **Brooks' Law** (more people = later) | 10 devs = chaos | 1 human + LLM = focused |
| **Communication Overhead** | n(n-1)/2 paths | 1 path: human ↔ LLM |
| **Conceptual Integrity** | Design by committee | Human owns vision |
| **Second-System Effect** | "Let's generalize!" | Research kept it concrete |
| **Plan to Throw One Away** | Emotional attachment | LLM has no ego - rewrite freely |
| **Documentation Lag** | "We'll document later" | Same session: code + docs |

---

## The New Dynamics with LLM Collaboration

### Challenges Unique to LLMs

| Dynamic | The Problem | The Solution |
|---------|-------------|--------------|
| **The Dimentia** | Forgets between sessions | `NEXT_SESSION.md` |
| **The Python Bias** | Defaults to common patterns | "Use Kleis, not Python!" |
| **Over-Eagerness** | Suggests "improvements" | `.cursorrules` constraints |
| **The Push Problem** | Doesn't understand workflows | Explicit rule enforcement |
| **Agreement Bias** | Tends to agree | Push back, challenge |

### What's Different from Human Collaboration

- **No ego** - LLM doesn't defend old code
- **Always available** - 3am debugging sessions work
- **No schedule conflicts** - Focus on the work
- **Infinite patience** - Explain the same thing 50 times
- **But... no grounding** - Humans ask "is this useful?"

---

## The Proof

What was built in ~1 month:

**Core Language:**
- Complete evaluator (9,325 lines in `evaluator.rs`)
- Hindley-Milner type inference (ADR-014)
- Pattern matching
- Algebraic data types (ADR-021)
- Z3 integration for verification (ADR-022)

**Tooling:**
- DAP Debugger (full VS Code integration)
- LSP Server (hover, go-to-definition, diagnostics)
- Jupyter Kernel
- REPL with provenance tracking

**Rendering:**
- 157 equation templates (`.kleist` files)
- 5 render targets (Unicode, LaTeX, HTML, Typst, Kleis)
- Equation Editor (WYSIWYG, type-checking)

**Document Generation:**
- MIT thesis template
- UofM Rackham dissertation template
- arXiv paper template
- Pure Kleis → Typst → PDF pipeline

**Architecture:**
- 29 ADRs documenting decisions
- Grammar versioned to v0.96
- Self-hosting strategy (Kleis defines itself)

---

## The Irony

"The Mythical Man-Month" is about how adding more people makes projects later.

Kleis was built in about a month, by one person with an LLM.

The anti-Brooks. The anti-mythical. The proof that the methodology works.

---

## Why This Matters

Most people using LLMs for code:
1. Ask for a function
2. Copy-paste
3. Move on

This was different:
1. Build persistent context system
2. Develop collaboration rules
3. Maintain conceptual integrity across hundreds of sessions
4. Ship a complete programming language

**This isn't "coding with AI." This is a new development methodology.**

And Kleis - a universal verification substrate that grew from one researcher's
need to check cosmological equations - is the proof.

---

## For the Book

Key themes to explore:

1. **Origin in genuine need** - Not "let's build a language" but "I need to verify my physics"
2. **The crystallization moment** - HARD_PROBLEMS_AHEAD.md
3. **Persistent context** - NEXT_SESSION.md as shared brain
4. **Rule enforcement** - Teaching the collaboration itself
5. **Domain expertise transfer** - LLM learning cosmology
6. **Avoiding Brooks' pitfalls** - And discovering new ones
7. **The proof** - What was actually built, how fast

Working title ideas:
- "The Unmythical Month"
- "One Person, One LLM, One Language"
- "From Cosmology to Code"
- "The NEXT_SESSION Methodology"

---

*Written January 2026, documenting work from late 2024 through early 2026.*

*The equations in `examples/ontology/revised/` can now be verified.*

