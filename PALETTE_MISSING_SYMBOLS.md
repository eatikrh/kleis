# Missing Palette Symbols - TODO

## Status
Symbols used in type system documentation but not yet in palette

## Priority: High
These symbols are essential for expressing the formal axioms documented in `KLEIS_TYPE_SYSTEM.md` and `KLEIS_TYPE_UX.md`

---

## Logical Connectives (Critical)

### Missing from Palette
| Symbol | LaTeX | Name | Used In | Priority |
|--------|-------|------|---------|----------|
| ∧ | `\land` or `\wedge` | AND/Conjunction | All axioms | **HIGH** |
| ∨ | `\lor` or `\vee` | OR/Disjunction | Axioms, case analysis | **HIGH** |
| ¬ | `\neg` or `\lnot` | NOT/Negation | Axioms, constraints | **HIGH** |
| ⟹ | `\implies` or `\Longrightarrow` | Implies | Axioms, theorems | **HIGH** |
| ⟺ | `\iff` or `\Longleftrightarrow` | If and only if | Definitions | **HIGH** |
| ∴ | `\therefore` | Therefore | Proofs | Medium |
| ∵ | `\because` | Because | Proofs | Medium |

### Currently Available
| Symbol | LaTeX | Name |
|--------|-------|------|
| → | `\to` | Arrow |
| ⇒ | `\Rightarrow` | Double arrow |
| ∀ | `\forall` | For all |
| ∃ | `\exists` | Exists |

---

## Set Theory Symbols

### Missing from Palette
| Symbol | LaTeX | Name | Used In | Priority |
|--------|-------|------|---------|----------|
| ∉ | `\notin` | Not element of | Constraints | **HIGH** |
| ∅ | `\emptyset` or `\varnothing` | Empty set | Set theory | **HIGH** |
| ⊆ | `\subseteq` | Subset or equal | Set relations | Medium |
| ⊊ | `\subsetneq` | Proper subset | Set relations | Low |
| ⊇ | `\supseteq` | Superset or equal | Set relations | Medium |
| ⊋ | `\supsetneq` | Proper superset | Set relations | Low |
| ∖ | `\setminus` | Set difference | Field axioms (F\{0}) | **HIGH** |
| 𝒫 | `\mathcal{P}` | Power set | Advanced set theory | Low |

### Currently Available
| Symbol | LaTeX | Name |
|--------|-------|------|
| ∈ | `\in` | Element of |
| ⊂ | `\subset` | Subset |
| ∪ | `\cup` | Union |
| ∩ | `\cap` | Intersection |

---

## Number Sets

### Missing from Palette
| Symbol | LaTeX | Name | Priority |
|--------|-------|------|----------|
| ℕ | `\mathbb{N}` | Natural numbers | **HIGH** |
| ℤ | `\mathbb{Z}` | Integers | **HIGH** |
| ℚ | `\mathbb{Q}` | Rational numbers | **HIGH** |
| ℝ | `\mathbb{R}` | Real numbers | **HIGH** |
| ℂ | `\mathbb{C}` | Complex numbers | **HIGH** |
| ℍ | `\mathbb{H}` | Quaternions | Low |
| 𝔽 | `\mathbb{F}` | Generic field | Medium |

**Note:** Some might already be accessible via text input, but need explicit palette buttons.

---

## Relation Symbols

### Missing from Palette
| Symbol | LaTeX | Name | Priority |
|--------|-------|------|----------|
| ≢ | `\not\equiv` | Not equivalent | Medium |
| ≔ or := | `\coloneqq` | Definition | **HIGH** |
| ≐ | `\doteq` | Approaches | Low |
| ∼ | `\sim` | Similar to | Medium |
| ≃ | `\simeq` | Asymptotic to | Low |
| ≅ | `\cong` | Congruent to | Medium |
| ∝ | `\propto` | Proportional | Medium |

### Currently Available
| Symbol | LaTeX | Name |
|--------|-------|------|
| = | `=` | Equals |
| ≠ | `\neq` | Not equals |
| < | `<` | Less than |
| > | `>` | Greater than |
| ≤ | `\leq` | Less or equal |
| ≥ | `\geq` | Greater or equal |
| ≈ | `\approx` | Approximately |
| ≡ | `\equiv` | Equivalent |

---

## Function/Mapping Symbols

### Missing from Palette
| Symbol | LaTeX | Name | Used In | Priority |
|--------|-------|------|---------|----------|
| ↦ | `\mapsto` | Maps to | Function definitions | **HIGH** |
| λ | `\lambda` | Lambda | Function literals | **HIGH** |
| ∘ | `\circ` | Composition | Function composition | **HIGH** |
| ⊕ | `\oplus` | Direct sum | Linear algebra | Medium |

**Note:** λ is in Greek tab, but needs to be in a Function/Logic tab for discoverability.

---

## Special Operators

### Missing from Palette
| Symbol | LaTeX | Name | Used In | Priority |
|--------|-------|------|---------|----------|
| ⊤ | `\top` | Top/True | Logic | Medium |
| ⊥ | `\bot` | Bottom/False | Logic | Medium |
| □ | `\Box` | Necessity (modal) | Modal logic | Low |
| ◊ | `\Diamond` | Possibility (modal) | Modal logic | Low |
| ⊢ | `\vdash` | Proves/Entails | Proof theory | Medium |
| ⊨ | `\models` | Models/Satisfies | Model theory | Low |

---

## Recommended Palette Additions

### New Tab: "Logic & Proofs"
```
Logic Tab:
- ∧ (and)
- ∨ (or)
- ¬ (not)
- ⟹ (implies)
- ⟺ (iff)
- ⊤ (true)
- ⊥ (false)
- ⊢ (proves)
- ∴ (therefore)
- ∵ (because)
```

### Expand "Logic & Sets" Tab
Currently has: <, >, ≤, ≥, ≈, ≡, ∈, ⊂, ∪, ∩, →, ⇒, ∀, ∃

**Add:**
- ∧, ∨, ¬, ⟹, ⟺ (logical connectives)
- ∉, ∅, ∖ (set operations)
- ℕ, ℤ, ℚ, ℝ, ℂ (number sets)
- ≔ (definition equals)
- ↦ (maps to)
- ∘ (composition)

### Function Tab (New or Merge with Basics)
- λ (lambda)
- ↦ (maps to)
- ∘ (composition)
- f⁻¹ (inverse template)

---

## Implementation Plan

### Phase 1: Critical Logic Symbols (Immediate)
1. Add to "Logic & Sets" tab:
   - `∧` (and) - Button: "∧ And"
   - `∨` (or) - Button: "∨ Or"
   - `¬` (not) - Button: "¬ Not"
   - `⟹` (implies) - Button: "⟹ Implies"
   - `⟺` (iff) - Button: "⟺ Iff"

2. Add to same tab:
   - `∖` (set minus) - Button: "∖ Minus"
   - `∅` (empty set) - Button: "∅ Empty"
   - `≔` (def equals) - Button: "≔ Define"

### Phase 2: Number Sets (High Priority)
Add to "Greek" tab or create "Special Symbols" tab:
   - `ℕ` - Button: "ℕ Naturals"
   - `ℤ` - Button: "ℤ Integers"
   - `ℚ` - Button: "ℚ Rationals"
   - `ℝ` - Button: "ℝ Reals"
   - `ℂ` - Button: "ℂ Complex"

### Phase 3: Function Symbols (Medium Priority)
Add to "Basics" tab:
   - `↦` (mapsto) - Button: "↦ Maps to"
   - `∘` (compose) - Button: "∘ Compose"
   - `λ` - Move from Greek to here or duplicate

### Phase 4: Proof Symbols (Low Priority)
Add to "Logic & Sets" tab or new "Proofs" tab:
   - `∴` (therefore)
   - `∵` (because)
   - `⊢` (proves)

---

## Backend Support

Most of these symbols should already work as `Object` nodes in the AST. Need to verify:

```bash
# Test if backend can render these
curl -X POST http://localhost:3000/api/parse \
  -H "Content-Type: application/json" \
  -d '{"latex": "\\forall x. P(x) \\land Q(x) \\implies R(x)"}'
```

If parser handles them, just need frontend palette buttons.

---

## Testing Checklist

After adding each symbol:
- [ ] Symbol button exists in palette
- [ ] Clicking inserts correct LaTeX
- [ ] LaTeX renders correctly in preview
- [ ] Symbol works in structural mode
- [ ] Backend parser recognizes it
- [ ] Type system can handle it in axioms

---

**Status:** Documentation uses advanced symbols; palette needs updating to match.

**Impact:** Without these symbols, users cannot express formal axioms in the editor that are shown in the documentation.

**Priority:** Phase 1 (logic symbols) should be implemented ASAP to enable axiom writing.

