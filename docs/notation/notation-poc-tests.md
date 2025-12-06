# Notation System - Proof of Concept Tests

**Date:** December 6, 2024  
**Purpose:** Validate design decisions with concrete test cases  
**Status:** Proposed Tests

---

## Overview

These tests validate our design decisions:
1. ✅ Text is source of truth (stored in `.kleis` files)
2. ✅ `frac(a,b)` vs `a/b` are same semantics, different display
3. ✅ Explicit forms required: `abs(x)`, `card(S)`, `norm(v)` (not `|x|`)

---

## POC Test 1: Basic Parsing and Rendering

### Input: Kleis Text File

**File:** `test1.kleis`
```kleis
// Simple arithmetic with different display modes
define inline_result = (a + b) / (c + d)
define display_result = frac(a + b, c + d)

// Summation
define sum_total = Σ_{i=0}^{n} f(i)

// Explicit forms for absolute value
define distance: ℝ = abs(x - y)
```

### Expected AST

```typescript
[
  {
    type: "function_definition",
    name: "inline_result",
    body: {
      type: "division",
      displayStyle: "inline",
      left: { type: "binary_op", operator: "+", left: "a", right: "b" },
      right: { type: "binary_op", operator: "+", left: "c", right: "d" }
    }
  },
  {
    type: "function_definition",
    name: "display_result",
    body: {
      type: "division",
      displayStyle: "fraction",
      numerator: { type: "binary_op", operator: "+", left: "a", right: "b" },
      denominator: { type: "binary_op", operator: "+", left: "c", right: "d" }
    }
  },
  {
    type: "function_definition",
    name: "sum_total",
    body: {
      type: "summation",
      variable: "i",
      lowerBound: 0,
      upperBound: "n",
      body: { type: "application", function: "f", arguments: ["i"] }
    }
  },
  {
    type: "function_definition",
    name: "distance",
    typeAnnotation: { type: "primitive", name: "ℝ" },
    body: {
      type: "abs",
      argument: {
        type: "binary_op",
        operator: "-",
        left: { type: "identifier", name: "x" },
        right: { type: "identifier", name: "y" }
      }
    }
  }
]
```

### Expected Visual Rendering

**Inline Result:**
```
inline_result = (a + b) / (c + d)
```

**Display Result:**
```
                 a + b
display_result = ─────
                 c + d
```

**Summation:**
```
              n
              Σ   f(i)
             i=0
```

**Distance:**
```
distance: ℝ = |x - y|
```

### Test Assertions

```typescript
test("POC Test 1: Basic Parsing and Rendering", () => {
  const text = fs.readFileSync("test1.kleis", "utf-8");
  const ast = parse(text);
  
  // Check AST structure
  assert.equal(ast.length, 4);
  assert.equal(ast[0].body.displayStyle, "inline");
  assert.equal(ast[1].body.displayStyle, "fraction");
  assert.equal(ast[2].body.type, "summation");
  assert.equal(ast[3].body.type, "abs");
  
  // Check rendering
  const html = renderToHTML(ast);
  assert.includes(html, "<div class='inline-division'>");
  assert.includes(html, "<div class='display-fraction'>");
  assert.includes(html, "∑"); // Summation symbol
  assert.includes(html, "|x - y|"); // Visual absolute value
});
```

**Result:** ✅ Pass if parser generates correct AST and renderer displays appropriately

---

## POC Test 2: Visual Editor Text Generation

### User Interaction Sequence

1. User opens visual equation editor
2. User clicks "Fraction" button
3. Visual editor shows fraction template: `□/□`
4. User types `a + b` in numerator
5. User types `c + d` in denominator
6. User saves

### Expected Generated Text

```kleis
frac(a + b, c + d)
```

**NOT:**
```kleis
(a + b) / (c + d)
```

### User Interaction Sequence 2

1. User clicks "Absolute Value" button
2. Visual editor shows: `|□|`
3. User types `x - y`
4. User saves

### Expected Generated Text

```kleis
abs(x - y)
```

**NOT:**
```kleis
|x - y|
```

### Test Code

```typescript
test("POC Test 2: Visual Editor Generates Canonical Text", () => {
  const editor = new VisualEquationEditor();
  
  // Test fraction
  editor.insertConstruct("fraction");
  editor.fillSlot("numerator", parse("a + b"));
  editor.fillSlot("denominator", parse("c + d"));
  const text1 = editor.generateText();
  
  assert.equal(text1, "frac(a + b, c + d)");
  
  // Test absolute value
  editor.clear();
  editor.insertConstruct("absolute_value");
  editor.fillSlot("argument", parse("x - y"));
  const text2 = editor.generateText();
  
  assert.equal(text2, "abs(x - y)");
});
```

**Result:** ✅ Pass if visual editor always generates explicit, canonical text

---

## POC Test 3: Git Diff Readability

### Scenario: User Refactors Code

**Before:** `algebra.kleis`
```kleis
define ratio = frac(numerator, denominator)
define distance = abs(x - y)
define set_size = card(S)
```

**After:** User changes display mode and operations
```kleis
define ratio = numerator / denominator
define distance = abs(x + y)
define set_size = abs(S)  // BUG: should be card()
```

### Expected Git Diff

```diff
diff --git a/algebra.kleis b/algebra.kleis
index abc123..def456 100644
--- a/algebra.kleis
+++ b/algebra.kleis
@@ -1,3 +1,3 @@
-define ratio = frac(numerator, denominator)
-define distance = abs(x - y)
-define set_size = card(S)
+define ratio = numerator / denominator
+define distance = abs(x + y)
+define set_size = abs(S)
```

### Analysis of Diff Readability

**Line 1:** ✅ Clear - changed from display fraction to inline division
**Line 2:** ✅ Clear - changed `-` to `+` in abs()
**Line 3:** ✅ Clear - changed `card(S)` to `abs(S)` (likely a bug, easily spotted!)

### Test Code

```typescript
test("POC Test 3: Git Diff is Human Readable", () => {
  const before = `define ratio = frac(numerator, denominator)
define distance = abs(x - y)
define set_size = card(S)`;

  const after = `define ratio = numerator / denominator
define distance = abs(x + y)
define set_size = abs(S)`;

  const diff = createGitDiff(before, after);
  
  // Diff should show actual code changes, not JSON structure
  assert.notIncludes(diff, '"type":');
  assert.notIncludes(diff, '{');
  assert.includes(diff, '-define ratio = frac(numerator, denominator)');
  assert.includes(diff, '+define ratio = numerator / denominator');
  
  // User can easily spot the bug: card(S) -> abs(S)
  assert.includes(diff, '-define set_size = card(S)');
  assert.includes(diff, '+define set_size = abs(S)');
});
```

**Result:** ✅ Pass if diffs are readable and changes are obvious

---

## POC Test 4: Round-Trip Preservation

### Test: Visual → Text → Parse → Render → Visual

**Step 1:** User creates in visual editor
- Adds fraction with `a + b` over `c + d`
- Visual displays as stacked fraction

**Step 2:** Visual editor generates text
```kleis
frac(a + b, c + d)
```

**Step 3:** Save to file and reload

**Step 4:** Parse text back to AST
```typescript
{
  type: "division",
  displayStyle: "fraction",
  numerator: { type: "binary_op", ... },
  denominator: { type: "binary_op", ... }
}
```

**Step 5:** Render AST back to visual
- Should display as stacked fraction again

**Step 6:** User edits in text editor
```kleis
(a + b) / (c + d)
```

**Step 7:** Parse and render
- Should display as inline division (not stacked)

### Test Code

```typescript
test("POC Test 4: Round-Trip Preservation", () => {
  // Visual to text
  const visualExpr = createVisualFraction(
    parse("a + b"),
    parse("c + d")
  );
  const text1 = generateText(visualExpr);
  assert.equal(text1, "frac(a + b, c + d)");
  
  // Text to AST
  const ast1 = parse(text1);
  assert.equal(ast1.displayStyle, "fraction");
  
  // AST to visual
  const rendered1 = render(ast1);
  assert.equal(rendered1.type, "stacked-fraction");
  
  // Modify text
  const text2 = "(a + b) / (c + d)";
  const ast2 = parse(text2);
  assert.equal(ast2.displayStyle, "inline");
  
  // Render modified
  const rendered2 = render(ast2);
  assert.equal(rendered2.type, "inline-division");
  
  // No information loss
  assert.deepEqual(
    simplify(ast1),
    simplify(ast2)
  ); // Semantically equivalent
});
```

**Result:** ✅ Pass if round-trip preserves both semantics and display intent

---

## POC Test 5: Type-Based Error Detection

### Scenario: User Makes Type Error with Explicit Forms

**File:** `errors.kleis`
```kleis
// Correct: abs of a number
x: ℝ = 5.0
y = abs(x)

// ERROR: abs of a set
S: Set(ℤ) = {1, 2, 3}
n = abs(S)

// Correct: cardinality of a set
m = card(S)

// ERROR: card of a number
p = card(x)
```

### Expected Type Errors

**Line 6:**
```
Type Error: abs() expects Number, got Set(ℤ)
  Line 6: n = abs(S)
              ^^^^^
  Note: Did you mean card(S) for set cardinality?
```

**Line 12:**
```
Type Error: card() expects Set(T), got ℝ
  Line 12: p = card(x)
               ^^^^^^^
  Note: Did you mean abs(x) for absolute value?
```

### Test Code

```typescript
test("POC Test 5: Explicit Forms Enable Better Error Messages", () => {
  const code = `
    x: ℝ = 5.0
    S: Set(ℤ) = {1, 2, 3}
    n = abs(S)
    p = card(x)
  `;
  
  const ast = parse(code);
  const errors = typeCheck(ast);
  
  assert.equal(errors.length, 2);
  
  // First error
  assert.equal(errors[0].line, 4);
  assert.equal(errors[0].message, "abs() expects Number, got Set(ℤ)");
  assert.includes(errors[0].suggestion, "card(S)");
  
  // Second error
  assert.equal(errors[1].line, 5);
  assert.equal(errors[1].message, "card() expects Set(T), got ℝ");
  assert.includes(errors[1].suggestion, "abs(x)");
});
```

**Compare to ambiguous `|x|` approach:**
```
Type Error: Type mismatch in delimited expression
  Line 4: n = |S|
              ^^^
  Expected: Number, got: Set(ℤ)
```
Much less helpful!

**Result:** ✅ Pass if explicit forms enable clear, actionable error messages

---

## POC Test 6: Visual Rendering Distinction

### Input: Mixed Display Styles

**File:** `display.kleis`
```kleis
// All semantically equivalent to division
inline1 = a / b
inline2 = (x + y) / z
fraction1 = frac(a, b)
fraction2 = frac(x + y, z)

// All use absolute value
abs1 = abs(x)
abs2 = abs(x + y)
```

### Expected Visual Rendering (HTML)

```html
<!-- Inline divisions -->
<span class="inline-expr">
  inline1 = a / b
</span>

<span class="inline-expr">
  inline2 = (x + y) / z
</span>

<!-- Display fractions -->
<div class="display-expr">
  fraction1 = <div class="fraction">
    <div class="numerator">a</div>
    <div class="denominator">b</div>
  </div>
</div>

<div class="display-expr">
  fraction2 = <div class="fraction">
    <div class="numerator">x + y</div>
    <div class="denominator">z</div>
  </div>
</div>

<!-- Absolute values render with bars -->
<span class="inline-expr">
  abs1 = <span class="abs">|x|</span>
</span>

<span class="inline-expr">
  abs2 = <span class="abs">|x + y|</span>
</span>
```

### Test Code

```typescript
test("POC Test 6: Renderer Respects Display Style", () => {
  const code = `
    inline1 = a / b
    fraction1 = frac(a, b)
    abs1 = abs(x)
  `;
  
  const ast = parse(code);
  const html = renderToHTML(ast);
  
  // Inline division
  const inline = findNode(html, "inline1");
  assert.includes(inline.className, "inline-expr");
  assert.includes(inline.textContent, "/");
  
  // Display fraction
  const fraction = findNode(html, "fraction1");
  assert.includes(fraction.className, "display-expr");
  assert.isNotNull(fraction.querySelector(".fraction .numerator"));
  assert.isNotNull(fraction.querySelector(".fraction .denominator"));
  
  // Absolute value renders with bars
  const abs = findNode(html, "abs1");
  assert.includes(abs.textContent, "|x|");
  assert.notIncludes(abs.textContent, "abs(x)"); // Function name hidden in display
});
```

**Result:** ✅ Pass if renderer produces visually distinct but semantically correct output

---

## POC Test 7: Parser Rejects Ambiguous Forms

### Input: Code Using Disallowed Notation

**File:** `bad_syntax.kleis`
```kleis
// This should be REJECTED by parser
x = |5|           // ERROR: Use abs(5)
n = |{1,2,3}|     // ERROR: Use card({1,2,3})
len = ||v||       // ERROR: Use norm(v)

// This should be ACCEPTED
y = abs(5)
m = card({1,2,3})
length = norm(v)
```

### Expected Parser Errors

**Line 2:**
```
Syntax Error: Ambiguous notation '|...|' not allowed in text
  Line 2: x = |5|
              ^^^
  Use explicit form: abs(5)
```

**Line 3:**
```
Syntax Error: Ambiguous notation '|...|' not allowed in text
  Line 3: n = |{1,2,3}|
              ^^^^^^^^^
  Use explicit form: card({1,2,3})
```

**Line 4:**
```
Syntax Error: Double delimiter '||...||' not allowed in text
  Line 4: len = ||v||
                ^^^^^
  Use explicit form: norm(v)
```

### Test Code

```typescript
test("POC Test 7: Parser Rejects Ambiguous Notation", () => {
  // Test that |x| is rejected
  assert.throws(() => {
    parse("x = |5|");
  }, /Ambiguous notation.*not allowed/);
  
  assert.throws(() => {
    parse("n = |{1,2,3}|");
  }, /Ambiguous notation.*not allowed/);
  
  assert.throws(() => {
    parse("len = ||v||");
  }, /Double delimiter.*not allowed/);
  
  // Test that explicit forms are accepted
  assert.doesNotThrow(() => {
    parse("y = abs(5)");
    parse("m = card({1,2,3})");
    parse("length = norm(v)");
  });
});
```

**Result:** ✅ Pass if parser enforces explicit notation in text

---

## POC Test 8: Formatter Consistency

### Input: Inconsistently Formatted Code

**File:** `unformatted.kleis`
```kleis
define x=frac(a,b)
define y =abs(  x-y  )
define z= Σ_{i=0}^{n}f(i)
```

### Expected Formatted Output

```kleis
define x = frac(a, b)
define y = abs(x - y)
define z = Σ_{i=0}^{n} f(i)
```

### Formatting Rules

1. Spaces around `=`
2. Space after commas in function calls
3. Spaces around binary operators
4. No spaces in subscript/superscript notation
5. Space before function arguments

### Test Code

```typescript
test("POC Test 8: Auto-Formatter Produces Canonical Style", () => {
  const unformatted = `define x=frac(a,b)
define y =abs(  x-y  )
define z= Σ_{i=0}^{n}f(i)`;

  const formatted = format(unformatted);
  
  const expected = `define x = frac(a, b)
define y = abs(x - y)
define z = Σ_{i=0}^{n} f(i)`;

  assert.equal(formatted, expected);
  
  // Formatting should be idempotent
  assert.equal(format(formatted), formatted);
});
```

**Result:** ✅ Pass if formatter produces consistent, git-friendly output

---

## POC Test 9: Integration Test - Complete Workflow

### Scenario: User Creates, Edits, Collaborates

**Step 1:** Alice creates equation in visual editor
- Creates: `distance = abs(x - y)`
- Visual editor generates and saves: `distance = abs(x - y)`

**Step 2:** Alice commits to git
```bash
$ git add equations.kleis
$ git commit -m "Add distance formula"
```

**Step 3:** Bob clones repo and edits in text editor
```kleis
// Bob changes it
distance = abs(x + y)
```

**Step 4:** Bob commits
```bash
$ git add equations.kleis
$ git commit -m "Fix distance formula sign"
```

**Step 5:** Alice pulls changes
```bash
$ git pull
$ git log -p equations.kleis
```

**Expected Git Log:**
```diff
commit abc123...
Author: Bob <bob@example.com>
Date:   Fri Dec 6 2024

    Fix distance formula sign
    
diff --git a/equations.kleis b/equations.kleis
--- a/equations.kleis
+++ b/equations.kleis
@@ -1 +1 @@
-distance = abs(x - y)
+distance = abs(x + y)
```

**Step 6:** Alice opens in visual editor
- Visual editor parses: `abs(x + y)`
- Displays: `distance = |x + y|`
- Alice sees Bob's change visually

### Test Code

```typescript
test("POC Test 9: Complete Workflow", async () => {
  // Alice creates in visual editor
  const visualEditor = new VisualEquationEditor();
  visualEditor.insertConstruct("absolute_value");
  visualEditor.fillSlot("argument", parse("x - y"));
  const text1 = visualEditor.generateText();
  
  fs.writeFileSync("equations.kleis", `distance = ${text1}`);
  
  // Simulate git commit
  const commit1 = gitCommit("equations.kleis");
  
  // Bob edits in text
  fs.writeFileSync("equations.kleis", "distance = abs(x + y)");
  const commit2 = gitCommit("equations.kleis");
  
  // Check diff
  const diff = gitDiff(commit1, commit2);
  assert.includes(diff, "-distance = abs(x - y)");
  assert.includes(diff, "+distance = abs(x + y)");
  
  // Alice opens in visual editor
  const text2 = fs.readFileSync("equations.kleis", "utf-8");
  const ast = parse(text2);
  const rendered = render(ast);
  
  // Should display |x + y| visually
  assert.equal(rendered.displayText, "|x + y|");
  assert.equal(ast.body.type, "abs");
  assert.equal(ast.body.argument.operator, "+");
});
```

**Result:** ✅ Pass if entire workflow (visual → text → git → text → visual) works seamlessly

---

## POC Test 10: Performance - Large File Parsing

### Scenario: Parse Large Kleis File

**File:** `large.kleis` (1000 equations)
```kleis
define eq_1 = frac(a_1, b_1)
define eq_2 = frac(a_2, b_2)
...
define eq_1000 = frac(a_1000, b_1000)
```

### Performance Requirements

- Parse time: < 100ms for 1000 equations
- Memory: < 50MB for AST
- Render time: < 200ms to HTML

### Test Code

```typescript
test("POC Test 10: Performance at Scale", () => {
  // Generate large file
  const lines = [];
  for (let i = 1; i <= 1000; i++) {
    lines.push(`define eq_${i} = frac(a_${i}, b_${i})`);
  }
  const largeFile = lines.join("\n");
  
  // Test parsing performance
  const parseStart = performance.now();
  const ast = parse(largeFile);
  const parseTime = performance.now() - parseStart;
  
  assert.isBelow(parseTime, 100, "Parse time should be < 100ms");
  assert.equal(ast.length, 1000);
  
  // Test memory usage
  const memory = process.memoryUsage().heapUsed;
  assert.isBelow(memory, 50 * 1024 * 1024, "Memory should be < 50MB");
  
  // Test rendering performance
  const renderStart = performance.now();
  const html = renderToHTML(ast);
  const renderTime = performance.now() - renderStart;
  
  assert.isBelow(renderTime, 200, "Render time should be < 200ms");
});
```

**Result:** ✅ Pass if performance is acceptable for typical use cases

---

## Summary of POC Tests

| Test | Focus | Success Criteria |
|------|-------|-----------------|
| 1 | Basic parsing/rendering | Correct AST, visual display |
| 2 | Visual editor output | Generates canonical text |
| 3 | Git diff readability | Human-readable diffs |
| 4 | Round-trip preservation | No information loss |
| 5 | Error messages | Clear, actionable errors |
| 6 | Visual rendering | Respects display styles |
| 7 | Parser enforcement | Rejects ambiguous notation |
| 8 | Formatter | Consistent, canonical style |
| 9 | Integration workflow | End-to-end collaboration |
| 10 | Performance | Acceptable speed/memory |

---

## Implementation Checklist

To validate these POC tests, implement:

- [ ] **Parser** with explicit notation (`abs`, `card`, `norm`, `frac`)
- [ ] **AST** with `displayStyle` metadata
- [ ] **Renderer** that uses traditional math notation in visual display
- [ ] **Visual Editor** that generates canonical text
- [ ] **Type Checker** that validates types and suggests corrections
- [ ] **Formatter** that enforces canonical style
- [ ] **Test Suite** that runs all 10 POC tests

---

## Next Steps

1. **Implement prototype parser** for subset of Kleis grammar
2. **Run POC Test 1** (basic parsing) to validate approach
3. **Implement simple visual renderer** (HTML or terminal)
4. **Run POC Test 2** (visual editor text generation)
5. **Validate git diff readability** with real examples
6. Iterate based on findings

**Success metric:** All 10 POC tests pass ✅

