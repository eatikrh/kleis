# NEXT SESSION: Validate Kleis Examples in Mathematics Documentation

**Priority:** High  
**Type:** Foundational Quality Work  
**Estimated Time:** 2-3 hours  
**Status:** Ready to start

---

## The Problem

**Found:** 55 Kleis code examples across 12 .tex files in `docs/mathematics/`

**Issue:** These examples may not conform to official Kleis grammar v0.5

**Why This Matters:**
- These are **foundational teaching materials**
- Future users learn Kleis syntax from these examples
- Wrong syntax in documentation teaches wrong patterns
- Like publishing a math textbook with incorrect proofs

**This is not throwaway documentation - this is the foundation of mathematics in Kleis!**

---

## What We Found

### Files with Kleis Examples

```
📄 bourbaki-style_foundational_appendix.tex       - 5 examples
📄 cathetory-theoretic_guide_to_kleis.tex         - 5 examples
📄 even_higher_algebraid_structures.tex           - 3 examples
📄 higher_algebraic_structures.tex                - 4 examples
📄 how_rust_code_implements_these.tex             - 8 examples
📄 kleis_fo_rust_and_java_programmers...tex       - 7 examples
📄 kleis_language_specification.tex               - 9 examples
📄 lie_algebras_and_tensor_algebras.tex           - 2 examples
📄 lie_groups_and_the_exponential_map.tex         - 3 examples
📄 magma_semigroup_monoid.tex                     - 1 example
📄 mathematicians_guide_to_kleis.tex              - 5 examples
📄 matrix_algebras.tex                            - 3 examples

Total: 55 Kleis code examples
```

### Sample Example (from lie_algebras_and_tensor_algebras.tex)

```kleis
structure LieAlgebra(g) over Field(F) {
  operation bracket : g × g → g
  axiom antisymmetry:
    ∀(x y : g). bracket(x,y) = - bracket(y,x)
  axiom jacobi:
    ∀(x y z : g).
      bracket(x, bracket(y,z))
    + bracket(y, bracket(z,x))
    + bracket(z, bracket(x,y)) = 0
}
```

**Potential issues:**
- Unary minus: `= - bracket(y,x)` (prefix operator on function call)
- Multi-line expressions in axioms
- Operator precedence in Jacobi identity

---

## The Task

### Goal

**Every Kleis example in documentation must:**
1. ✅ Parse correctly with current parser
2. ✅ Conform to official grammar (kleis_grammar_v05.ebnf)
3. ✅ Be mathematically correct
4. ✅ Demonstrate proper Kleis idioms

### Process

**For each of 55 examples:**

1. **Extract** from .tex file
2. **Parse** with check_parser
3. **If fails:**
   - Identify syntax error
   - Check against official grammar
   - Fix to conform to grammar
   - Re-test until passes
4. **Update** .tex file with corrected example
5. **Verify** mathematical correctness
6. **Document** any grammar limitations discovered

### Tooling

**Script created:** `scripts/validate_kleis_in_docs.py`

**Usage:**
```bash
python3 scripts/validate_kleis_in_docs.py
# Shows: Which files have examples, which pass/fail
```

**Fix workflow:**
```bash
# 1. Extract example
# 2. Save to temp file
echo "structure Example..." > /tmp/test.kleis

# 3. Test parse
cargo run --bin check_parser /tmp/test.kleis

# 4. Fix syntax errors
# 5. Update .tex file
# 6. Re-run validator
```

---

## Common Issues to Expect

### 1. Unary Minus in Expressions

**May not parse:**
```kleis
bracket(x,y) = - bracket(y,x)
```

**Fix (if needed):**
```kleis
bracket(x,y) = negate(bracket(y,x))
```
Or check if prefix minus is actually supported.

### 2. Complex Type Expressions

**May not parse:**
```kleis
operation tensor : V × V → V⊗V
```

**Fix:**
```kleis
operation tensor : V → V → TensorProduct(V, V)
```

### 3. Multi-line Axioms

**Should work:**
```kleis
axiom jacobi:
  ∀(x y z : g).
    bracket(x, bracket(y,z))
  + bracket(y, bracket(z,x))
  + bracket(z, bracket(x,y)) = 0
```

**But verify parser handles newlines in expressions.**

### 4. Unicode Symbols

**May not work:**
```kleis
⊗, ⊕, ∇, ∂
```

**Check grammar** - which Unicode operators are actually supported?

### 5. Higher-Order Types

**May not parse:**
```kleis
operation map : (A → B) → List(A) → List(B)
```

**Known limitation:** Parenthesized function types not yet supported.

---

## Success Criteria

After task complete:

✅ All 55 examples parse correctly  
✅ All examples conform to official grammar  
✅ All examples are mathematically correct  
✅ Validator script shows 55/55 passing  
✅ Any grammar limitations documented  
✅ PDF versions regenerated for all .tex files

---

## Why This Is Important

### 1. Teaching Future Users

These examples are how people **learn Kleis syntax**.

**If examples are wrong:**
- Users learn incorrect patterns
- Bug reports about "Kleis doesn't work"
- Confusion about what's actually supported

**If examples are correct:**
- Clear learning path
- Confidence in documentation
- Professional impression

### 2. Specification by Example

**Grammar (EBNF):** Formal, complete, hard to read  
**Examples:** Concrete, understandable, easy to learn

**Both must agree!** Examples are executable specification.

### 3. Quality Signal

**Publishing Kleis with broken examples says:**
- "We don't test our own documentation"
- "Examples are aspirational, not real"
- Amateur hour

**Publishing with verified examples says:**
- "Every example has been tested"
- "This is production quality"
- Professional work

### 4. Foundation for LLM Training

If Kleis becomes popular, these examples will be:
- Scraped by search engines
- Used in LLM training data
- Reference for code generation

**Wrong examples → LLMs learn wrong syntax**  
**Correct examples → LLMs learn right syntax**

---

## Approach

### Phase 1: Discovery (30 min)

Run validator on all files:
```bash
python3 scripts/validate_kleis_in_docs.py > validation_report.txt
```

Categorize failures:
- Unary minus issues
- Unicode operator issues  
- Type expression issues
- Unknown issues

### Phase 2: Fix Common Patterns (1 hour)

If many examples fail the same way:
- Fix the pattern once
- Apply to all similar examples
- Re-validate

### Phase 3: Fix Unique Cases (1 hour)

Handle special cases:
- Advanced type expressions
- Complex axioms
- Edge cases in grammar

### Phase 4: Verification (30 min)

```bash
# All should pass
python3 scripts/validate_kleis_in_docs.py

# Regenerate PDFs
cd docs/mathematics
for f in *.tex; do
  xelatex wrapper_$f  # (create wrappers as needed)
done
```

---

## Expected Outcomes

### Findings

**We'll discover:**
- Which grammar features are actually used in practice
- Which features documentation assumes but parser doesn't support
- Gaps between grammar specification and implementation

**Document these!** They inform parser development priorities.

### Fixes

**We'll update:**
- 55 Kleis examples to conform to grammar
- Possibly some .tex prose explaining syntax
- Script to validate on each commit

### Process Improvement

**We'll add:**
- Quality gate: Validate doc examples before commit
- CI check: Ensure examples parse
- README: How to add new examples correctly

---

## Relation to Full Prelude Migration

**Today's work enables this task:**
- Parser now supports quantifiers: `∀(x : T)`
- Parser now supports operator symbols: `(×)`, `(•)`
- Parser now supports quantified types
- Grammar coverage: 65%

**Many examples will NOW parse** that didn't before!

**But some still won't** - and that's what we need to fix.

---

## Tools Created

### 1. Validation Script

**File:** `scripts/validate_kleis_in_docs.py`

**Features:**
- Extracts Kleis blocks from LaTeX verbatim environments
- Tests each with check_parser
- Reports pass/fail with line numbers
- Summary statistics

### 2. Quality Gate Addition (TODO)

**Add to `.cursorrules`:**
```
Before committing .tex files with Kleis examples:
→ Run: python3 scripts/validate_kleis_in_docs.py
→ Must show 100% passing
→ Fix any failures before commit
```

---

## Deliverables

After session complete:

1. ✅ All 55 examples parse correctly
2. ✅ Validation script shows 55/55 passing
3. ✅ Quality gate added to .cursorrules
4. ✅ Grammar gaps documented (if any found)
5. ✅ PDFs regenerated with corrected examples
6. ✅ README updated with validation process

---

## Open Questions

1. **Should we update grammar** if examples use features not in v0.5?
   - Or update examples to match grammar?
   - Decision: Match grammar (it's the spec)

2. **What if grammar doesn't support something examples need?**
   - Document as limitation
   - Note for future grammar extension
   - Or use alternative syntax that works

3. **Should validation be in CI?**
   - Yes! Prevent bad examples from merging
   - Add to `.github/workflows/ci.yml`

---

## Success Metrics

**Before:**
- 55 examples, unknown parse status
- No validation process
- Documentation might teach wrong syntax

**After:**
- 55 examples, all verified to parse
- Automated validation in CI
- Documentation guaranteed correct

**This transforms documentation from "hopefully correct" to "provably correct"!**

---

## Notes

**This is foundational work** - not flashy, but essential for quality.

**Mathematical documentation quality matters** - Kleis aims to be the language for rigorous mathematics. The documentation must be rigorous too!

**One-time investment** - Once fixed, these examples are stable. Math doesn't change!

---

**Created:** December 11, 2024  
**For Session:** December 12+  
**Status:** Ready to begin  
**Priority:** High (quality/correctness)

---

## Quick Start for Next Session

```bash
cd /Users/eatik_1/Documents/git/cee/kleis

# Run validator (will need to fix PATH issue first)
python3 scripts/validate_kleis_in_docs.py

# Start with first file
vim docs/mathematics/lie_algebras_and_tensor_algebras.tex

# Test example
echo "structure LieAlgebra..." > /tmp/test.kleis
cargo run --bin check_parser /tmp/test.kleis

# Fix, repeat for all 55 examples
```

**Let's make the documentation as rigorous as the language!** 📚

