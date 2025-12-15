# ADR-015: Text as Source of Truth for Kleis Notation

**Date:** December 6, 2025  
**Status:** Accepted  
**Related:** ADR-009 (WYSIWYG Editor), ADR-010 (Inline Editing), ADR-011 (Notebook Environment)

---

## Context

Kleis supports dual content editing modes:
1. **Kleis code** with Unicode mathematical notation (computational)
2. **Visual equation editor** with WYSIWYG interface (presentational)
3. **Rich media** (tables, images, graphs) in notebooks

This creates a fundamental design question: **What is the canonical representation?**

### The Problem

- If we store AST (as JSON/binary): Git diffs become unreadable
- If we store visual layout metadata: Version control becomes complex
- If we allow ambiguous notation: Parsing requires type context
- If we have two different "formats": Tools diverge, collaboration suffers

### Requirements

1. **Git-friendly**: Diffs must be human-readable for collaboration
2. **Round-trip**: Visual ↔ Text ↔ Visual without information loss
3. **Unambiguous**: Parsing should not require type information
4. **Beautiful**: Visual display should use traditional mathematical notation
5. **Unified**: One source of truth for both editing modes

---

## Decision

We adopt **three interconnected design principles**:

### 1. Text is Source of Truth

**Kleis code stored as Unicode text in `.kleis` files.**

- AST is derived (parsed from text)
- Visual representation is rendered from AST
- Git tracks text, not JSON or binary formats
- All metadata (types, display hints) inferred or embedded in text syntax

**Flow:**
```
.kleis file (text) → Parser → AST → Type Checker → Renderer → Visual Display
                       ↑                                           ↓
                       └────── Visual Editor generates text ───────┘
```

### 2. Display Mode via Syntax, Same Semantics

**Different text forms indicate display preference, but represent the same operation.**

Examples:
```kleis
// Display mode (stacked fraction)
result = frac(a + b, c + d)

// Inline mode (division operator)
result = (a + b) / (c + d)
```

Both are semantically equivalent (division), but:
- Parser recognizes `frac()` vs `/` 
- AST includes `displayStyle` metadata
- Renderer respects display preference
- Git diffs show stylistic intent clearly

### 3. Explicit Forms Required in Text

**Text syntax requires explicit function names for traditionally ambiguous notation.**

Problem: `|x|` could mean absolute value, cardinality, or norm.

Solution:
```kleis
// Text syntax (explicit, unambiguous)
abs(x)      // Absolute value: Number → Number
card(S)     // Cardinality: Set(T) → ℕ
norm(v)     // Norm: Vector(n) → ℝ

// Visual rendering (traditional notation)
|x|         // Rendered with single bars
|S|         // Rendered with single bars
‖v‖         // Rendered with double bars
```

Visual editor workflow:
1. User clicks "absolute value" button
2. Visual displays: |□|
3. Behind the scenes, generates text: `abs(x)`
4. When saved, file contains: `abs(x)`
5. When reopened, renders as: |x|

---

## Consequences

### Positive

#### 1. Git Compatibility ✅
```diff
- distance = abs(x - y)
+ distance = abs(x + y)
```
Human-readable diffs show actual code changes, not JSON structure.

#### 2. Clear Error Messages ✅
```
Error: card() expects Set(T), got Real
  Line 5: n = card(x)
          
Suggestion: Did you mean abs(x)?
```
Explicit forms enable actionable error messages.

#### 3. Single-Pass Parsing ✅
No type context needed to parse. Faster, simpler parser.

#### 4. Tool Compatibility ✅
Standard development tools work: git, grep, diff, text editors, IDEs.

#### 5. Visual Beauty Preserved ✅
Display still uses traditional notation. Users see |x|, not `abs(x)`.

#### 6. Seamless Collaboration ✅
Multiple authors can work together using git merge, PR reviews, etc.

### Negative

#### 1. Text is Less "Mathematical" ⚠️
`abs(x)` instead of `|x|` in source code.

**Mitigation:** Visual display compensates. Users primarily interact with visual form.

#### 2. Multiple Syntaxes for Same Operation ⚠️
`frac(a,b)` and `a/b` both exist.

**Mitigation:** Intent is clear. Git diffs show stylistic choice explicitly.

#### 3. Visual Editor Constrained ⚠️
Must generate canonical text forms.

**Mitigation:** Ensures consistency across tools. Palette-based UI guides users.

---

## Implementation

### Standard Library Additions Required

**Good news:** The Kleis v0.3 grammar ALREADY supports these via function application syntax!

**No grammar changes needed.** These are standard function calls:

```kleis
// Standard library definitions (stdlib/core.kleis)

// Absolute value
operation abs : ℝ → ℝ
axiom abs_non_negative: ∀ (x : ℝ) . abs(x) ≥ 0

// Cardinality
operation card : ∀T. Set(T) → ℕ
axiom card_empty: card(∅) = 0

// Norm
operation norm : ∀(n : ℕ). Vector(n) → ℝ
define norm(v) = √(dot(v, v))

// Fraction (display mode division)
operation frac : ℝ × ℝ → ℝ
define frac(a, b) = a / b
```

**Existing grammar already supports:**
- `Σ_{i=0}^{n}` - summation syntax
- `∫_{a}^{b}` - integral syntax  
- `∀ x . P(x)` - universal quantification
- `∃ x . Q(x)` - existential quantification
- Function application: `f(x)` - covers abs, card, norm, frac

**Note:** Grammar impact analysis completed and integrated into this document.

### What Needs Implementation

1. **Standard Library** (`stdlib/core.kleis`)
   - Type signatures for `abs`, `card`, `norm`, `frac`
   - Axioms defining their properties
   - Implementation where needed

2. **Renderer Updates**
   - Recognize these functions
   - Display with traditional notation:
     - `abs(x)` → |x|
     - `card(S)` → |S|
     - `norm(v)` → ‖v‖
     - `frac(a,b)` → stacked fraction

3. **Type Checker**
   - Load stdlib signatures
   - Validate function calls
   - Generate helpful error messages

**Grammar:** ✅ Already complete! No changes needed.

### AST Structure

```typescript
interface DivisionNode extends BaseNode {
  type: "division";
  displayStyle: "fraction" | "inline";  // Derived from syntax
  numerator: Expression;
  denominator: Expression;
}

interface AbsNode extends BaseNode {
  type: "abs";
  argument: Expression;
}

interface CardinalityNode extends BaseNode {
  type: "cardinality";
  argument: Expression;
}

interface NormNode extends BaseNode {
  type: "norm";
  argument: Expression;
}
```

### Parser Behavior

**The v0.3 grammar already handles everything correctly!**

**Currently Accepted** (no changes needed):
- ✅ `abs(x)`, `card(S)`, `norm(v)` - parsed as function calls
- ✅ `frac(a, b)` - parsed as function call
- ✅ `a / b` - parsed as division operator
- ✅ Unicode symbols: `Σ`, `∫`, `∀`, `∃`, etc.

**Currently Rejected** (which is what we want):
- ❌ `|x|` - no delimiter syntax in grammar (perfect!)
- ❌ `||v||` - no double delimiter (perfect!)

**Parser sees these as:**
```
abs(x)  →  FunctionApplication(identifier="abs", args=[x])
frac(a,b) → FunctionApplication(identifier="frac", args=[a,b])
a / b   →  BinaryOp(operator="/", left=a, right=b)
```

Type checker then validates based on stdlib type signatures.

### Renderer Behavior

Maps explicit text forms to traditional visual notation:

| Text Syntax | Visual Display | Type |
|-------------|----------------|------|
| `abs(x)` | \|x\| | Number → Number |
| `card(S)` | \|S\| | Set(T) → ℕ |
| `norm(v)` | ‖v‖ | Vector(n) → ℝ |
| `frac(a,b)` | Stacked fraction | Division (display) |
| `a / b` | Inline division | Division (inline) |

### Visual Editor Implementation

Construct palette generates canonical text:

```typescript
class VisualEditor {
  insertConstruct(type: string) {
    switch(type) {
      case "absolute_value":
        return generateText("abs", [slot("argument")]);
      case "fraction":
        return generateText("frac", [slot("numerator"), slot("denominator")]);
      case "summation":
        return generateText("Σ", {
          subscript: slot("lower"),
          superscript: slot("upper"),
          body: slot("expression")
        });
    }
  }
}
```

---

## Validation

### Test Cases

See [notation-mapping-tests.md](../notation/notation-mapping-tests.md) for 11 detailed test cases.

### Proof-of-Concept Tests

See [notation-poc-tests.md](../notation/notation-poc-tests.md) for 10 POC tests including:
- Basic parsing and rendering
- Visual editor text generation
- Git diff readability
- Round-trip preservation
- Type-based error detection

### Success Criteria

1. ✅ Users can create equations in visual editor with beautiful notation
2. ✅ Git diffs are human-readable and show actual changes
3. ✅ Text and visual editors produce identical semantics
4. ✅ Type errors have clear, actionable messages
5. ✅ Round-trip (visual ↔ text ↔ visual) preserves all information
6. ✅ Collaboration works seamlessly (git merge, review, etc.)
7. ✅ Performance is acceptable (< 100ms parse, < 200ms render)

---

## Grammar Impact Analysis

### Summary

**Good news:** The Kleis v0.3 grammar ALREADY supports everything this ADR requires!

**Changes needed:** Standard library additions, NOT grammar changes.

### Current Grammar Support

The existing grammar (via function application syntax) already parses:
- `abs(x)` ✅
- `card(S)` ✅
- `norm(v)` ✅
- `frac(a, b)` ✅

These are standard function calls handled by:
```antlr
expression : expression '(' arguments ')'   // Function application
```

The grammar does NOT support (which is what we want):
- `|x|` ❌ No delimiter operators
- `||v||` ❌ No double delimiters

**Perfect!** The grammar already rejects ambiguous notation.

### What Actually Needs Implementation

**1. Standard Library Definitions** (`stdlib/core.kleis`):

```kleis
@library("kleis.core")
@version("1.0.0")

// Absolute value
operation abs : ℝ → ℝ
operation abs : ℂ → ℝ    // Overload for complex
axiom abs_non_negative: ∀ (x : ℝ) . abs(x) ≥ 0
axiom abs_triangle: ∀ (x y : ℝ) . abs(x + y) ≤ abs(x) + abs(y)

// Cardinality
operation card : ∀T. Set(T) → ℕ
axiom card_empty: card(∅) = 0
axiom card_union: ∀ (A B : Set(T)) . 
    A ∩ B = ∅ ⇒ card(A ∪ B) = card(A) + card(B)

// Norm (already partially defined in docs)
operation norm : ∀(n : ℕ). Vector(n) → ℝ
define norm(v) = √(dot(v, v))
axiom norm_non_negative: ∀ (v : Vector(n)) . norm(v) ≥ 0
axiom norm_triangle: ∀ (u v : Vector(n)) . norm(u + v) ≤ norm(u) + norm(v)

// Fraction (display mode division)
operation frac : ℝ × ℝ → ℝ
define frac(a, b) = a / b
// Note: Semantically identical to /, signals display mode to renderer
```

**2. Renderer Updates**:

```typescript
function renderFunctionCall(func: string, args: Expression[]): string {
  switch(func) {
    case 'abs':
    case 'card':
      return `|${renderExpr(args[0])}|`;
    
    case 'norm':
      return `‖${renderExpr(args[0])}‖`;
    
    case 'frac':
      return renderFraction(args[0], args[1]);  // Stacked
    
    default:
      return `${func}(${args.map(renderExpr).join(', ')})`;
  }
}
```

**3. Type Checker Integration**:

Load stdlib signatures and validate:
```kleis
S: Set(ℤ) = {1, 2, 3}
n = abs(S)  // Error: abs expects Number, got Set(ℤ)
```

### Implementation Status

| Feature | Grammar | Stdlib | Renderer | Status |
|---------|---------|--------|----------|--------|
| `abs(x)` | ✅ Works | ⬜ Need | ⬜ Need | Ready to implement |
| `card(S)` | ✅ Works | ⬜ Need | ⬜ Need | Ready to implement |
| `norm(v)` | ✅ Works | ⚠️ Partial | ⬜ Need | Ready to implement |
| `frac(a,b)` | ✅ Works | ⬜ Need | ⬜ Need | Ready to implement |
| Reject `\|x\|` | ✅ Already | N/A | N/A | Complete! |

---

## Document Authoring: Inline Equations

**Related:** [ADR-012: Document Authoring](adr-012-document-authoring.md)

### Context

Kleis notebooks (ADR-011, ADR-012) include text cells with embedded equations:

```kleis
The energy [E = ½mv²] depends on velocity.
```

### Text Representation

**Per this ADR, inline equations use canonical Kleis text:**

```kleis
// Stored in .kleis file
The absolute value [abs(x)] is always non-negative.
                    ^^^^^^
                    Canonical form

The fraction [frac(a, b)] represents division.
              ^^^^^^^^^^
              Display mode

The cardinality [card(S)] gives set size.
                 ^^^^^^^
                 Explicit function
```

### Visual Display

**Renderer displays with traditional notation:**

```
The absolute value |x| is always non-negative.
                   ^^
                   Rendered beautifully

The fraction a/b represents division.
             ─── (stacked)
```

### Workflow

1. **User edits** in visual editor with structural editing
2. **Visual editor generates** canonical text: `abs(x)`
3. **Saved to file** as: `[abs(x)]`
4. **Git tracks** explicit text changes
5. **When reopened**, parser reads `abs(x)`, renderer displays `|x|`

### Benefits for Document Authoring

```diff
# Someone changes equation in paper
- The magnitude [abs(F)] gives force strength.
+ The magnitude [norm(F)] gives force strength.
```

**Git diff is crystal clear:** Changed from absolute value to norm!

This is why text representation matters for document collaboration.

---

## References

### Design Documents
- [content-editing-paradigm.md](../notation/content-editing-paradigm.md) - Design discussion
- [notation-mapping-tests.md](../notation/notation-mapping-tests.md) - Test cases
- [notation-poc-tests.md](../notation/notation-poc-tests.md) - Validation tests

### Related ADRs
- [ADR-009](adr-009-wysiwyg-structural-editor.md) - WYSIWYG Structural Editor
- [ADR-010](adr-010-inline-editing.md) - Inline Editing
- [ADR-011](adr-011-notebook-environment.md) - Notebook Environment
- [ADR-012](adr-012-document-authoring.md) - Document Authoring (uses these text conventions for inline equations)
- [ADR-014](adr-014-hindley-milner-type-system.md) - Type System

### Implementation
- [notation-poc-tests.md](../notation/notation-poc-tests.md) - 10 proof-of-concept test specifications
- **Executable POC:** `cargo run --bin test_adr015_poc` - ✅ **PASSING** - Validates core design

---

## Alternatives Considered

### Alternative 1: Store AST as JSON

**Rejected because:**
- Git diffs become unreadable
- Version control shows JSON structure changes, not semantic changes
- Collaboration becomes difficult
- Standard text tools don't work

**Example diff:**
```diff
  {
    "type": "application",
-   "function": "abs",
+   "function": "card",
    "argument": {"type": "identifier", "name": "x"}
  }
```
vs our approach:
```diff
- result = abs(x)
+ result = card(x)
```

### Alternative 2: Allow Ambiguous `|x|` Notation

**Rejected because:**
- Requires type-directed parsing (two-pass)
- Error messages are unclear ("type mismatch in delimited expression")
- Git diffs don't show operation changes clearly
- More complex parser implementation

### Alternative 3: Store Both Text and AST

**Rejected because:**
- Synchronization complexity
- Which is source of truth?
- Git tracks redundant information
- Merge conflicts become ambiguous

### Alternative 4: LaTeX as Source Format

**Rejected because:**
- LaTeX is presentational, not computational
- Difficult to parse unambiguously
- Not designed for executable mathematics
- Tooling would be complex

---

## Migration Path

### For Existing Code

If Kleis already has files using ambiguous notation:

1. **Parse with warnings:**
   ```
   Warning: |x| is ambiguous, will be rejected in future
   Suggestion: Use abs(x)
   ```

2. **Provide migration tool:**
   ```bash
   kleis migrate --fix-ambiguous old.kleis new.kleis
   ```

3. **Grace period:** Accept both forms with warnings for one release

4. **Strict mode:** Reject ambiguous forms

### For Visual Editor

If visual editor already generates different text:

1. **Update text generation** to use explicit forms
2. **Keep visual display** unchanged (users see no difference)
3. **Re-save files** automatically updates format

---

## Future Considerations

### Auto-Completion for Unicode

Palette-assisted editor with:
- Type `\sum` → suggests `Σ`
- Type `\int` → suggests `∫`
- Type `\forall` → suggests `∀`

### Language Server Protocol (LSP)

Standard IDE integration:
- Syntax highlighting
- Auto-completion
- Error checking
- Go-to-definition

### Formatter

Like `prettier` or `gofmt`:
- Consistent spacing
- Canonical ordering
- Idempotent formatting

### Collaborative Editing

Real-time collaboration (like Overleaf):
- Operational Transform on text
- Visual updates propagate
- Conflict resolution

---

## Decision Status

**Status:** Accepted ✅

**Date:** December 6, 2025

**Decision Makers:** Design session participants

**Next Steps:**
1. Implement POC Test 1 (Basic Parsing & Rendering)
2. Validate design with working code
3. Iterate based on findings

---

## Appendix: Canonical Forms Quick Reference

| Concept | Text Syntax | Visual Display |
|---------|-------------|----------------|
| Absolute value | `abs(x)` | \|x\| |
| Cardinality | `card(S)` | \|S\| |
| Norm | `norm(v)` | ‖v‖ |
| Display fraction | `frac(a, b)` | a/b (stacked) |
| Inline division | `a / b` | a / b |
| Summation | `Σ_{i=0}^{n} f(i)` | Σ with bounds |
| Product | `Π_{i=0}^{n} f(i)` | Π with bounds |
| Integral | `∫_{a}^{b} f(x) dx` | ∫ with bounds |
| Universal | `∀ x ∈ S . P(x)` | ∀x ∈ S : P(x) |
| Existential | `∃ x ∈ S . P(x)` | ∃x ∈ S : P(x) |

---

**Last Updated:** December 6, 2025

