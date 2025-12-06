# Notation Mapping Test Cases

**Date:** December 6, 2024  
**Purpose:** Test the one-to-one mapping hypothesis between Kleis text and visual representations  
**Status:** Working Document

---

## Testing Methodology

### Source of Truth: TEXT ✅

**Design Decision:** Kleis code is stored as text in `.kleis` files (for git diff readability).

**This means:**
- Text format is canonical and must be well-defined
- AST and visual representation are derived from text
- Git diffs show actual code changes
- Canonical forms are CRITICAL for consistent diffs

**Round-trip flow:**
```
Visual Editor → Generate Canonical Text → Save
    ↑                                        ↓
    └─── Parse ← Type Check ← Parse ─────────┘
```

### Test Criteria

For each test case, we examine:

1. **Kleis Text** - The Unicode text representation (SOURCE OF TRUTH)
2. **Visual Form** - The 2D typeset appearance (DERIVED)
3. **AST** - The abstract syntax tree structure (DERIVED)
4. **Round-trip** - Can we go Visual → Text → Parse → Render → Same Visual?
5. **Ambiguities** - Are there multiple interpretations?
6. **Git Diff** - Will changes be readable in version control?

**Result Categories:**
- ✅ **BIJECTIVE** - Perfect 1:1 mapping
- ⚠️ **CANONICAL** - 1:1 only with canonical form rules
- ❌ **AMBIGUOUS** - Multiple interpretations exist

---

## Test Case 1: Simple Summation

### Kleis Text
```kleis
Σ_{i=0}^{n} f(i)
```

### Visual Form
```
  n
  Σ   f(i)
 i=0
```

### AST
```typescript
{
  type: "summation",
  variable: { name: "i", type: "bound" },
  lowerBound: { type: "number", value: 0 },
  upperBound: { type: "identifier", name: "n" },
  body: {
    type: "application",
    function: { type: "identifier", name: "f" },
    arguments: [{ type: "identifier", name: "i" }]
  }
}
```

### Round-trip Analysis
1. Visual → AST: Unambiguous structure from 2D layout
2. AST → Text: Deterministic serialization
3. Text → AST: Parser produces same structure
4. AST → Visual: Renderer produces same layout

**Result:** ✅ **BIJECTIVE**

---

## Test Case 2: Nested Summation

### Kleis Text
```kleis
Σ_{i=0}^{m} Σ_{j=0}^{n} a_{i,j}
```

### Visual Form
```
 m   n
 Σ   Σ   aᵢⱼ
i=0 j=0
```

### AST
```typescript
{
  type: "summation",
  variable: { name: "i", type: "bound" },
  lowerBound: { type: "number", value: 0 },
  upperBound: { type: "identifier", name: "m" },
  body: {
    type: "summation",
    variable: { name: "j", type: "bound" },
    lowerBound: { type: "number", value: 0 },
    upperBound: { type: "identifier", name: "n" },
    body: {
      type: "subscript",
      base: { type: "identifier", name: "a" },
      subscripts: [
        { type: "identifier", name: "i" },
        { type: "identifier", name: "j" }
      ]
    }
  }
}
```

### Issues
- Subscript notation: `a_{i,j}` vs `a_i_j` vs `a[i,j]`
- Visual: Can show as aᵢⱼ or a_{i,j} or matrix element notation

**Result:** ⚠️ **CANONICAL** - Need to define standard for multi-index subscripts

**Proposed Canon:** `a_{i,j}` with comma for multiple subscripts

---

## Test Case 3: Fraction vs Division

### Variant A: Fraction

#### Kleis Text
```kleis
frac(a + b, c + d)
```

#### Visual Form
```
a + b
─────
c + d
```

#### AST
```typescript
{
  type: "fraction",
  numerator: {
    type: "binary_op",
    operator: "+",
    left: { type: "identifier", name: "a" },
    right: { type: "identifier", name: "b" }
  },
  denominator: {
    type: "binary_op",
    operator: "+",
    left: { type: "identifier", name: "c" },
    right: { type: "identifier", name: "d" }
  }
}
```

### Variant B: Inline Division

#### Kleis Text
```kleis
(a + b) / (c + d)
```

#### Visual Form (inline)
```
(a + b) / (c + d)
```

#### AST
```typescript
{
  type: "binary_op",
  operator: "/",
  left: {
    type: "binary_op",
    operator: "+",
    left: { type: "identifier", name: "a" },
    right: { type: "identifier", name: "b" }
  },
  right: {
    type: "binary_op",
    operator: "+",
    left: { type: "identifier", name: "c" },
    right: { type: "identifier", name: "d" }
  }
}
```

### Analysis

**Problem:** Are these semantically equivalent or different?
- **Mathematical equivalence:** Yes, both compute the same value
- **Structural equivalence:** No, different AST nodes
- **Visual equivalence:** No, rendered differently

**Question:** Should visual editor allow both? Or only one?

**Options:**
1. **Treat as equivalent** - Normalize during parsing (`frac(a,b)` → `a / b`)
2. **Treat as distinct** - Different display styles, same semantics
3. **Context-dependent** - Display vs inline mode

**Result:** ⚠️ **CANONICAL** - Need display mode hints in AST

**Proposed Solution (Given TEXT is source of truth):**

The text must distinguish between these forms:
```kleis
frac(a + b, c + d)      // Explicit fraction (display mode)
(a + b) / (c + d)       // Division operator (inline mode)
```

Git diff example:
```diff
- result = frac(a + b, c + d)
+ result = (a + b) / (c + d)
```

This is readable! User can see: "changed from fraction notation to division operator"

**Alternative:** Could make them equivalent and use comment hints:
```kleis
// @display-mode: block
result = (a + b) / (c + d)
```

But this is less clear in diffs and adds comment clutter.

**Recommendation:** Use `frac(a, b)` for display fractions, `/` for inline division. They map to different visual forms but same computational semantics.

---

## Test Case 4: Definite Integral

### Kleis Text
```kleis
∫_{a}^{b} f(x) dx
```

### Visual Form
```
 b
 ⌠
 ⎮  f(x) dx
 ⌡
 a
```

### AST
```typescript
{
  type: "integral",
  integralType: "definite",
  lowerBound: { type: "identifier", name: "a" },
  upperBound: { type: "identifier", name: "b" },
  integrand: {
    type: "application",
    function: { type: "identifier", name: "f" },
    arguments: [{ type: "identifier", name: "x" }]
  },
  variable: { type: "identifier", name: "x" }
}
```

### Issues
- Order: `dx` comes after integrand (standard math notation)
- Could also write: `∫_{x=a}^{b} f(x) dx` (explicit variable)

**Result:** ⚠️ **CANONICAL** - Prefer `∫_{a}^{b} f(x) dx` without explicit variable in bounds

---

## Test Case 5: Universal Quantification

### Kleis Text
```kleis
∀ x ∈ ℝ . x^2 ≥ 0
```

### Visual Form
```
∀x ∈ ℝ : x² ≥ 0
```

### AST
```typescript
{
  type: "forall",
  variables: [{
    name: "x",
    domain: { type: "type", name: "ℝ" }
  }],
  proposition: {
    type: "binary_op",
    operator: "≥",
    left: {
      type: "binary_op",
      operator: "^",
      left: { type: "identifier", name: "x" },
      right: { type: "number", value: 2 }
    },
    right: { type: "number", value: 0 }
  }
}
```

### Issues
- Separator: `.` vs `:` vs `,` after quantifier
- Domain: `∈ ℝ` vs `: ℝ` (element-of vs type annotation)

**Grammar shows:**
```ebnf
varDecl ::= identifier [ ":" type ]
          | identifier "∈" type
          | identifier "∈" expression
```

So three forms are allowed! This is ambiguous.

**Result:** ⚠️ **CANONICAL** - Need to pick one standard form

**Proposed Canon:** 
- `x ∈ ℝ` for set/domain membership
- `x : T` for type annotation
- Use `.` as separator (follows logic notation)

---

## Test Case 6: Matrix Notation

### Kleis Text
```kleis
[[1, 2], [3, 4]]
```

### Visual Form
```
⎡ 1  2 ⎤
⎢      ⎥
⎣ 3  4 ⎦
```

### AST
```typescript
{
  type: "matrix",
  rows: 2,
  cols: 2,
  elements: [
    [
      { type: "number", value: 1 },
      { type: "number", value: 2 }
    ],
    [
      { type: "number", value: 3 },
      { type: "number", value: 4 }
    ]
  ]
}
```

### Issues
- Bracket style: square brackets, parentheses, vertical bars?
- Spacing: alignment in visual form
- Type: Matrix vs nested lists?

**Alternative notations:**
```kleis
[[1, 2], [3, 4]]           // Nested lists
matrix(2, 2, [1,2,3,4])    // Constructor with dimensions
⎡1 2; 3 4⎦                 // MATLAB-style semicolons
```

**Result:** ⚠️ **CANONICAL** - Nested list notation is least ambiguous for parsing

**Visual rendering decision:** Parser infers it's a matrix from type system, renders accordingly

---

## Test Case 7: Absolute Value vs Cardinality vs Norm

### Three Interpretations

#### Variant A: Absolute Value
```kleis
|x|
```
Visual: |x|, Context: x is a number
AST: `{ type: "abs", argument: { type: "identifier", name: "x" } }`

#### Variant B: Cardinality
```kleis
|S|
```
Visual: |S|, Context: S is a set
AST: `{ type: "cardinality", argument: { type: "identifier", name: "S" } }`

#### Variant C: Norm
```kleis
||v||
```
Visual: ‖v‖, Context: v is a vector
AST: `{ type: "norm", argument: { type: "identifier", name: "v" } }`

### Analysis

**Problem:** Same text notation, different semantics based on type!

**Result:** ❌ **AMBIGUOUS WITHOUT TYPE CONTEXT**

**DECISION: Require Explicit Forms in Text** ✅

Given that **text is source of truth** and must produce readable git diffs, we require explicit function names:

```kleis
// Text syntax (explicit)
y = abs(x)      // Absolute value
n = card(S)     // Cardinality  
len = norm(v)   // Norm

// Visual rendering (uses traditional notation)
y = |x|         // Rendered with single bars
n = |S|         // Rendered with single bars (context clear from types)
len = ‖v‖       // Rendered with double bars
```

**AST:**
```typescript
// Text generates explicit nodes
{ type: "abs", argument: { type: "identifier", name: "x" } }
{ type: "cardinality", argument: { type: "identifier", name: "S" } }
{ type: "norm", argument: { type: "identifier", name: "v" } }
```

**Git diff example:**
```diff
- magnitude = abs(x)
+ magnitude = card(S)
```
Clear! The operation changed, not just the variable.

**Visual editor workflow:**
1. User clicks "absolute value" button in palette
2. Visual editor generates text: `abs(x)`
3. Parser creates explicit AST node: `{ type: "abs", ... }`
4. Renderer displays with traditional notation: `|x|`

**Benefits:**
- ✅ Unambiguous parsing (no type context needed)
- ✅ Clear git diffs
- ✅ Better error messages
- ✅ Visual display can still use traditional mathematical notation

---

## Test Case 8: Function Application vs Multiplication

### Kleis Text Options
```kleis
f(x)      // Explicit function application
f x       // Juxtaposition (Haskell-style)
f·x       // Explicit dot product
f × x     // Cross product
```

### Visual Forms
```
f(x)      // Parentheses
f x       // Space (could be multiplication!)
f·x       // Dot
f × x     // Cross
```

### AST Differences
```typescript
// Function application
{ type: "application", function: "f", arguments: ["x"] }

// Multiplication
{ type: "binary_op", operator: "*", left: "f", right: "x" }

// Dot product
{ type: "dot_product", left: "f", right: "x" }

// Cross product
{ type: "cross_product", left: "f", right: "x" }
```

### Analysis

**Result:** ⚠️ **CANONICAL** - Use explicit notation for each case

**Proposed Canon:**
- `f(x)` - function application (always use parentheses)
- `f * x` - scalar multiplication
- `f · x` - dot product
- `f × x` - cross product
- Juxtaposition `f x` - NOT SUPPORTED (too ambiguous)

---

## Test Case 9: Superscripts and Exponentiation

### Variant A: Exponentiation
```kleis
x^2
```
Visual: x²
AST: `{ type: "power", base: "x", exponent: 2 }`

### Variant B: Transpose
```kleis
A^T    or    A^†    or    Aᵀ
```
Visual: Aᵀ or A†
AST: `{ type: "transpose", argument: "A" }`

### Variant C: Derivative Order
```kleis
f^{(n)}
```
Visual: f⁽ⁿ⁾
AST: `{ type: "derivative", function: "f", order: "n" }`

### Analysis

**Problem:** Superscript notation is overloaded!

**Current grammar helps:**
```antlr
postfixOp : '!' | '†' | '*' | 'ᵀ' | '^T'
```

So `^T` is a postfix operator, not superscript.

**Result:** ⚠️ **CANONICAL** - Use:
- `^` for exponentiation: `x^2`
- Postfix operators for matrix operations: `A^T` or `Aᵀ`
- Explicit syntax for derivatives: `D^n(f)` or `d^n f/dx^n`

---

## Test Case 10: Piecewise Functions

### Kleis Text
```kleis
f(x) = {
  x^2      if x ≥ 0,
  -x^2     if x < 0
}
```

### Visual Form
```
       ⎧  x²     if x ≥ 0
f(x) = ⎨
       ⎩ -x²     if x < 0
```

### AST
```typescript
{
  type: "function_definition",
  name: "f",
  parameters: ["x"],
  body: {
    type: "piecewise",
    cases: [
      {
        condition: {
          type: "binary_op",
          operator: "≥",
          left: { type: "identifier", name: "x" },
          right: { type: "number", value: 0 }
        },
        expression: {
          type: "power",
          base: { type: "identifier", name: "x" },
          exponent: { type: "number", value: 2 }
        }
      },
      {
        condition: {
          type: "binary_op",
          operator: "<",
          left: { type: "identifier", name: "x" },
          right: { type: "number", value: 0 }
        },
        expression: {
          type: "unary_op",
          operator: "-",
          operand: {
            type: "power",
            base: { type: "identifier", name: "x" },
            exponent: { type: "number", value: 2 }
          }
        }
      }
    ]
  }
}
```

### Issues
- Block structure with braces
- Condition placement: `if` before or after expression?
- Comma separators between cases

**Result:** ✅ **BIJECTIVE** - Structure is clear and unambiguous

---

## Test Case 11: Multi-line Expressions

### Kleis Text
```kleis
x = a + b + c
    + d + e
```

### Visual Form
```
x = a + b + c
  + d + e
```

### Issues
- Line continuation: explicit or implicit?
- Indentation matters?
- Operator position: start or end of line?

**Result:** ⚠️ **CANONICAL** - Need line continuation rules

**Proposed Canon:**
- Trailing operator continues to next line: `a + b +`
- Or explicit continuation character: `a + b \`
- Leading operator on continuation line: `+ c` (requires operator on previous line end)

This is a **parsing issue**, not a visual/text mapping issue.

---

## Test Case 12: Git Diff Readability

Since **text is the source of truth**, git diffs must be human-readable. Let's test some examples:

### Example A: Changing Summation Bounds

```diff
- sum_total = Σ_{i=0}^{n} f(i)
+ sum_total = Σ_{i=1}^{n-1} f(i)
```

**Analysis:** ✅ Clear! User can see bounds changed from `[0,n]` to `[1,n-1]`

### Example B: Changing Integral Type

```diff
- area = ∫_{0}^{∞} f(x) dx
+ area = ∫_{-∞}^{∞} f(x) dx
```

**Analysis:** ✅ Clear! Bounds changed, infinity symbol is readable

### Example C: Refactoring Quantifiers

```diff
- axiom commutative: ∀ x y . x + y = y + x
+ axiom commutative: ∀ (x y : ℝ) . x + y = y + x
```

**Analysis:** ✅ Clear! Added type annotation to variables

### Example D: Matrix Definition

```diff
- A = [[1, 2], [3, 4]]
+ A = [[1, 2, 0], 
+      [3, 4, 0]]
```

**Analysis:** ⚠️ Readable, but formatting matters. Need consistent style.

### Example E: If visual editor stored AST as JSON

```diff
  {
    "type": "summation",
    "variable": {"name": "i", "type": "bound"},
-   "lowerBound": {"type": "number", "value": 0},
+   "lowerBound": {"type": "number", "value": 1},
    "upperBound": {"type": "identifier", "name": "n"},
    ...
  }
```

**Analysis:** ❌ Terrible! User has to read JSON structure to understand change.

### Conclusion

**Text-based storage makes diffs meaningful:**
- Changes to mathematical expressions are visible
- Unicode symbols render correctly in modern git tools
- Structural changes (like adding type annotations) are clear
- Much better than JSON AST diffs

**Requirements for good diffs:**
1. Consistent formatting/style (whitespace, line breaks)
2. Meaningful symbol choices (not abbreviations)
3. Logical grouping (related code on nearby lines)
4. Comments where needed (but not for metadata)

---

## Summary of Findings

### ✅ Bijective (No Issues)
1. Simple summation
2. Piecewise functions
3. Most quantified propositions (with canonical form)

### ⚠️ Requires Canonical Forms
1. **Subscripts:** `a_{i,j}` (comma-separated)
2. **Fractions:** Need display mode in AST (`frac(a,b)` vs `a / b`)
3. **Integrals:** Prefer `∫_{a}^{b} f(x) dx` format
4. **Quantifiers:** Use `.` separator, `∈` for domains, `:` for types
5. **Matrices:** Use nested list notation
6. **Function application:** Always use parentheses `f(x)`
7. **Superscripts:** Context-dependent (exponent vs transpose)

### ❌ Ambiguous Without Additional Context
1. **Absolute value / Cardinality / Norm:** `|x|` - needs type information
2. **Implicit multiplication vs application:** Requires parsing rules

---

## Recommendations

### 1. Text Format is King

Since text is source of truth, prioritize:
- **Readability:** Code should read naturally
- **Consistent formatting:** Define standard style (like `gofmt` or `prettier`)
- **Meaningful symbols:** Use actual Unicode, not ASCII approximations
- **Git-friendly:** Changes should produce clean, understandable diffs

### 2. Grammar Extensions Needed

Add to grammar:
```ebnf
(* Explicit fraction construct for display mode *)
fraction ::= "frac" "(" expression "," expression ")" ;

(* Piecewise functions *)
piecewise ::= "{" case { "," case } "}" ;
case ::= expression "if" expression ;

(* Avoid display mode hints - use different text forms instead *)
(* Example: frac(a,b) vs a/b, not a/b with @display annotation *)
```

### 3. AST Enrichment

Add fields to AST nodes (derived, not stored):
```typescript
interface BaseASTNode {
  type: string;
  sourcePosition: { line: number, column: number };  // From parser
  inferredType?: Type;                               // From type checker
  // NO display mode stored - inferred from syntax!
}
```

**Key principle:** AST is ephemeral, derived from text. Don't store metadata that should be in text.

### 3. Canonical Form Document

Create `docs/canonical-notation.md` specifying:
- Preferred text representation for each construct
- Visual rendering rules
- Normalization rules for equivalent forms

### 4. Round-Trip Testing

Implement test suite:
```typescript
function testRoundTrip(visualInput: VisualExpression) {
  const ast1 = visualToAST(visualInput);
  const text = astToText(ast1);
  const ast2 = textToAST(text);
  const visual = astToVisual(ast2);
  
  assert(deepEqual(ast1, ast2), "AST preservation");
  assert(visualEqual(visualInput, visual), "Visual preservation");
}
```

### 5. Parser Strategy

For ambiguous cases:
1. **Parser creates generic node** (e.g., `delimited`, `juxtaposition`)
2. **Type checker resolves** to specific operation
3. **Visual renderer uses type info** to display correctly

This allows text → AST → visual to work even with type-dependent notation.

---

## Conclusion

**The one-to-one mapping is achievable with these constraints:**

1. ✅ **TEXT IS SOURCE OF TRUTH** - Store `.kleis` files as Unicode text for git compatibility
2. ✅ Define canonical text forms for all constructs (critical for consistent diffs)
3. ✅ Different text forms for different visual layouts (`frac(a,b)` vs `a/b`)
4. ✅ Use type information to resolve ambiguities (like `|x|`)
5. ✅ Visual editor generates explicit, canonical text
6. ✅ Text parser accepts canonical forms (and potentially synonyms for convenience)

**Key insights:**

1. **Visual editor advantage:** Users make explicit choices ("fraction" vs "division"), generating unambiguous canonical text.

2. **Git-driven design:** Since text is source of truth, canonical forms must prioritize:
   - Human readability in diffs
   - Consistent formatting
   - Meaningful Unicode symbols

3. **No metadata in AST storage:** Types, display hints, etc. are derived during parsing/type-checking, not stored in files.

4. **Round-trip guarantee:**
   ```
   Visual Editor → Canonical Text → .kleis file → Git
                                         ↓
   Parse → AST → Type Check → Render → Visual Display
   ```

**Next steps:**
1. ✅ Decision made: Text is source of truth
2. Create `docs/canonical-notation.md` - formal specification of text forms
3. Implement automatic formatter (like `prettier` or `gofmt`) for consistent style
4. Implement round-trip tests for each construct
5. Design palette UI that generates canonical text
6. Test git diff readability with real examples

