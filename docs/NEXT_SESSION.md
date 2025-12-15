# Next Session: Equation Editor & Kleis Grammar Alignment

## Priority Work Items

### 1. Three-Rung Ladder Architecture
Clarify and implement the separation between:
- **Rung 1: Equation Editor** - User-facing UI for building mathematical expressions
- **Rung 2: Kleis Renderer** - Visual rendering of Kleis AST to human-readable notation
- **Rung 3: Kleis Language** - The formal language with its grammar and semantics

### 2. Kleis Grammar v0.7 Alignment
- Review `docs/grammar/kleis_grammar_v07.ebnf` ✓ (exists)
- Ensure parser, renderer, and editor all conform to official grammar
- Document any deviations with rationale

### 3. Z3 Backend Testing
- Verify that grammar v0.7 expressions translate correctly to Z3
- Test edge cases: quantifiers, matrices, operations
- Ensure round-trip: Editor → AST → Z3 → Result → Renderer

### 4. Kleis Renderer vs Editor Differences
- What the **Editor** produces (AST from user interaction)
- What the **Renderer** displays (visual representation of AST)
- What **Kleis** accepts (grammar-conforming text/AST)
- Ensure bidirectional consistency: Editor → Kleis → Renderer → Editor

## Context from Previous Session
- Z3 integration complete (verify + satisfiability)
- Matrix operations working
- Power/exponentiation fixed
- Branch: `feature/kleis-renderer`

## Questions to Answer
- [ ] Is grammar v0.7 the current official version?
- [ ] What are the key differences from v0.5?
- [ ] Where does the 3-rung separation break down currently?

---
*Created: Dec 14, 2024*

