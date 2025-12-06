# Implementation Next Steps

**Date:** December 6, 2024  
**Status:** Ready to Implement  
**Formal Decision:** [ADR-015: Text as Source of Truth](adr-015-text-as-source-of-truth.md)  
**Prerequisites:** Read [ADR-015](adr-015-text-as-source-of-truth.md) or [notation-design-summary.md](notation-design-summary.md)

---

## 🎯 Quick Start: Implement POC Test 1

**✅ ALREADY VALIDATED!** Run: `cargo run --bin test_adr015_poc`

The existing Kleis AST supports ADR-015's design:
- `abs(x)` → `Operation { name: "abs", args: [Object("x")] }` ✅
- `frac(a,b)` → `Operation { name: "frac", args: [...] }` ✅
- Nested expressions work ✅

**What remains:**
1. **Standard library** with type signatures for abs, card, norm, frac
2. **Renderer updates** to display with traditional notation (|x|, ‖v‖, etc.)
3. **Type checker** integration for helpful error messages

**Expected time:** 1-2 days for full implementation

**Success criteria:** Full POC Test 1 (with rendering) passes

---

## 📋 Implementation Phases

### Phase 1: Parser & Renderer (Week 1)
**Priority:** HIGH  
**Validates:** Core design decisions

**Tasks:**
- [ ] Extend Kleis parser to support:
  - `abs(expr)` - absolute value
  - `card(expr)` - cardinality
  - `norm(expr)` - norm
  - `frac(expr, expr)` - display fraction
  - Keep existing: `Σ`, `∫`, `∀`, `∃`
- [ ] Generate AST with `displayStyle` field
- [ ] Build HTML renderer that:
  - Renders `abs(x)` as |x|
  - Renders `frac(a,b)` as stacked fraction
  - Renders `a/b` as inline division
- [ ] Write unit tests for parsing
- [ ] Run POC Test 1

**File locations:**
```
src/
├── parser/
│   ├── kleis-parser.ts       # Extend this
│   └── ast-types.ts           # Add displayStyle field
└── renderer/
    └── html-renderer.ts       # Create this
```

**Test:**
```typescript
// Input text
const code = "result = frac(a + b, c + d)";

// Parse
const ast = parse(code);
assert.equal(ast.body.type, "division");
assert.equal(ast.body.displayStyle, "fraction");

// Render
const html = renderToHTML(ast);
assert.includes(html, '<div class="fraction">');
```

---

### Phase 2: Visual Editor Integration (Week 2)
**Priority:** HIGH  
**Validates:** Visual → Text workflow

**Tasks:**
- [ ] Build visual equation editor UI (or extend existing)
- [ ] Add construct palette:
  - Fraction button → generates `frac(a, b)`
  - Absolute value → generates `abs(x)`
  - Summation → generates `Σ_{i=0}^{n}`
- [ ] Implement text generation from visual constructs
- [ ] Ensure output is canonical (formatted consistently)
- [ ] Run POC Test 2

**UI mockup:**
```
┌─────────────────────────────────┐
│ [≡] Kleis Editor                │
├─────────────────────────────────┤
│ Palette:                        │
│ [∑] [∫] [frac] [|x|] [√] [∀]   │
├─────────────────────────────────┤
│ Visual Display:                 │
│   a + b                         │
│   ─────                         │
│   c + d                         │
├─────────────────────────────────┤
│ Generated Text:                 │
│ frac(a + b, c + d)             │
└─────────────────────────────────┘
```

**Test:**
```typescript
const editor = new VisualEditor();
editor.insertConstruct("fraction");
editor.fillSlot("numerator", "a + b");
editor.fillSlot("denominator", "c + d");

const text = editor.generateText();
assert.equal(text, "frac(a + b, c + d)");
```

---

### Phase 3: Type System Integration (Week 3)
**Priority:** MEDIUM  
**Validates:** Error messages

**Tasks:**
- [ ] Implement basic type checker
- [ ] Add type signatures for:
  - `abs: Number → Number`
  - `card: Set(T) → ℕ`
  - `norm: Vector(n) → ℝ`
- [ ] Generate helpful error messages
- [ ] Suggest corrections (abs vs card)
- [ ] Run POC Test 5

**Example:**
```kleis
// Input
S: Set(ℤ) = {1, 2, 3}
n = abs(S)  // Type error!

// Expected error message
Type Error: abs() expects Number, got Set(ℤ)
  Line 2: n = abs(S)
              ^^^^^
Suggestion: Did you mean card(S) for set cardinality?
```

**Test:**
```typescript
const code = `
  S: Set(ℤ) = {1, 2, 3}
  n = abs(S)
`;

const ast = parse(code);
const errors = typeCheck(ast);

assert.equal(errors.length, 1);
assert.includes(errors[0].message, "abs() expects Number");
assert.includes(errors[0].suggestion, "card(S)");
```

---

### Phase 4: Formatting & Tooling (Week 4)
**Priority:** MEDIUM  
**Validates:** Git diff readability

**Tasks:**
- [ ] Build auto-formatter (like `prettier` or `gofmt`)
- [ ] Define canonical style rules:
  - Spaces around `=`
  - Space after commas
  - Spaces around binary operators
- [ ] Make formatter idempotent
- [ ] Add "format on save" option
- [ ] Run POC Test 8

**Formatter rules:**
```typescript
const rules = {
  spacesAroundEquals: true,      // a = b, not a=b
  spaceAfterComma: true,          // frac(a, b), not frac(a,b)
  spacesAroundOperators: true,    // x + y, not x+y
  noSpaceInSubscript: true,       // _{i=0}, not _{ i = 0 }
};
```

**Test:**
```typescript
const unformatted = "define x=frac(a,b)";
const formatted = format(unformatted);
assert.equal(formatted, "define x = frac(a, b)");

// Idempotent
assert.equal(format(formatted), formatted);
```

---

### Phase 5: Integration & Testing (Week 5)
**Priority:** HIGH  
**Validates:** Complete workflow

**Tasks:**
- [ ] Create sample `.kleis` files
- [ ] Test visual editor → text → save workflow
- [ ] Test text editor → parse → render workflow
- [ ] Test git diff readability with real examples
- [ ] Run all 10 POC tests
- [ ] Performance testing (large files)
- [ ] Document any issues found

**Integration test:**
```typescript
test("Complete workflow", async () => {
  // 1. Create in visual editor
  const editor = new VisualEditor();
  editor.insertConstruct("fraction");
  const text = editor.generateText();
  
  // 2. Save to file
  fs.writeFileSync("test.kleis", text);
  
  // 3. Simulate git commit
  const commit1 = gitCommit("test.kleis");
  
  // 4. Edit in text editor
  const modified = text.replace("frac", "inline division");
  fs.writeFileSync("test.kleis", modified);
  
  // 5. Check diff readability
  const diff = gitDiff(commit1, "HEAD");
  assert.notIncludes(diff, "type:");  // No JSON!
  assert.includes(diff, "-frac(");    // Shows actual change
  
  // 6. Reopen in visual editor
  const reopened = parse(fs.readFileSync("test.kleis"));
  const rendered = render(reopened);
  // Should display correctly
});
```

---

## 🛠️ Technical Requirements

### Parser Requirements
- Support Unicode mathematical symbols: `Σ`, `∫`, `∀`, `∃`, etc.
- Parse explicit forms: `abs()`, `card()`, `norm()`, `frac()`
- **Reject** ambiguous forms: `|x|`, `||v||`
- Generate AST with source positions
- Include `displayStyle` metadata where relevant

### AST Structure
```typescript
interface BaseNode {
  type: string;
  sourcePosition: { line: number; column: number };
}

interface DivisionNode extends BaseNode {
  type: "division";
  displayStyle: "fraction" | "inline";
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

### Renderer Requirements
- Input: AST
- Output: HTML (or other format)
- Respect `displayStyle` hints
- Use Unicode/MathML/CSS for display
- Render `abs(x)` as |x| visually
- Render `frac(a,b)` as stacked fraction

### Visual Editor Requirements
- Construct palette with common operations
- Template-based input (fill slots)
- Generate canonical Kleis text
- Preview both visual and text forms
- Validate expressions before saving

---

## 📊 Success Metrics

**MVP Complete When:**
- ✅ All 10 POC tests pass
- ✅ Can create equation in visual editor
- ✅ Can save as `.kleis` text file
- ✅ Git diff is readable
- ✅ Can reopen and edit in text editor
- ✅ Can reopen and edit in visual editor
- ✅ Type errors have clear messages
- ✅ Auto-formatter works

**Performance Targets:**
- Parse 1000 equations: < 100ms
- Render to HTML: < 200ms
- Visual editor latency: < 50ms
- Memory usage: < 50MB for typical files

---

## 🚧 Known Challenges

### Challenge 1: Parser Complexity
Unicode mathematical symbols require careful lexer design.

**Solution:** Use existing Unicode parsing libraries, or extend ANTLR4 lexer.

### Challenge 2: Type Inference
Full Hindley-Milner inference is complex.

**Solution:** Start with simple type annotations, add inference gradually.

### Challenge 3: Visual Rendering
Beautiful mathematical layout is hard.

**Solution:** Use MathML or CSS flexbox. Consider KaTeX/MathJax for rendering.

### Challenge 4: Editor Performance
Rich visual editors can be slow.

**Solution:** Virtual scrolling, incremental parsing, lazy rendering.

---

## 📚 Reference Documents

**Must Read:**
1. [ADR-015](adr-015-text-as-source-of-truth.md) - ⭐ Complete architectural decision
2. [notation-poc-tests.md](notation-poc-tests.md) - What to implement (10 tests)

**Deep Dive:**
3. [notation-mapping-tests.md](notation-mapping-tests.md) - Detailed test cases (11 tests)
4. [content-editing-paradigm.md](content-editing-paradigm.md) - Design discussion history

**Grammar:**
6. [grammar/Kleis_v03.g4](grammar/Kleis_v03.g4) - ANTLR4 grammar
7. [grammar/kleis_grammar_v03.ebnf](grammar/kleis_grammar_v03.ebnf) - EBNF spec

---

## 💬 Getting Help

**Questions about design decisions?**
- Review the design documents
- Check test cases for examples
- Open discussion issue

**Implementation questions?**
- Check existing parser code
- Review POC test pseudocode
- Ask specific technical questions

**Found a problem with the design?**
- Document the issue with example
- Propose alternative approach
- We can revise if needed

---

## ✅ Checklist for Starting

Before you begin implementation:

- [ ] Read [ADR-015](adr-015-text-as-source-of-truth.md) (formal decision record)
- [ ] Understand the 3 key decisions:
  - [ ] Text is source of truth
  - [ ] Display modes via syntax
  - [ ] Explicit forms required
- [ ] Review POC Test 1 requirements
- [ ] Set up development environment
- [ ] Locate existing parser code
- [ ] Understand current AST structure

**Ready?** Start with Phase 1: Parser & Renderer 🚀

---

**Last Updated:** December 6, 2024

