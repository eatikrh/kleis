# Type Checking UX Design

**Date:** December 2024  
**Status:** Design Specification  
**Consolidated from:** INCREMENTAL_TYPE_CHECKING.md, USER_DEFINED_TYPES.md

---

## Table of Contents

1. [Core Principle](#core-principle)
2. [Five Type States](#five-type-states)
3. [Visual Feedback](#visual-feedback)
4. [Context Management](#context-management)
5. [User-Defined Types](#user-defined-types)
6. [API Design](#api-design)
7. [Frontend Integration](#frontend-integration)

---

## Core Principle

**Type checking should guide, not block the user.**

### ❌ Bad: Blocking Type Checking
```
User types: x + 
[EDITOR BLOCKS: "Type error: incomplete expression"]
```

### ✅ Good: Non-Intrusive Type Checking
```
User types: x + 
[Yellow underline: "Incomplete, expected second argument"]
User can continue typing!
```

---

## Five Type States

### 🔴 Error - Clear Violation
**Definite type error that needs fixing**

```kleis
A + 3  // where A : Matrix(3,3)
```

**Feedback:**
- 🔴 Red underline
- Tooltip: "Cannot add Matrix(3,3) to Scalar"
- Suggestion: "Did you mean A + 3·I?"

**State:** `TypeState::Error { message, suggestion }`

### 🟡 Incomplete - Has Placeholders
**Valid so far, but unfilled slots**

```kleis
x + □
```

**Feedback:**
- 🟡 Yellow underline on placeholder
- Tooltip: "Type: α + α → α (fill placeholder)"

**State:** `TypeState::Incomplete { partial_type, missing }`

### 🟢 Polymorphic - Valid but Generic
**Type-checks with type variables**

```kleis
x + y  // no context
```

**Feedback:**
- 🟢 Green checkmark
- Tooltip: "Type: α + α → α (polymorphic)"
- Info: "Valid for any Numeric type"

**State:** `TypeState::Polymorphic { type_scheme, variables }`

### 🔵 Concrete - Fully Resolved
**Type-checks with specific types**

```kleis
1 + 2
```

**Feedback:**
- 🔵 Blue checkmark
- Tooltip: "Type: ℝ + ℝ → ℝ"

**State:** `TypeState::Concrete { ty }`

### ⚪ Unknown - No Context
**No type information available**

```kleis
x  // undefined
```

**Feedback:**
- ⚪ Gray dot
- Tooltip: "Type: unknown (no context)"

**State:** `TypeState::Unknown`

---

## Visual Feedback

### HTML/CSS Implementation

```html
<div class="expression" data-type-state="polymorphic">
  <span class="indicator">🟢</span>
  <span class="math">x + y</span>
  <span class="type-tooltip">Type: α + α → α</span>
</div>
```

```css
.expression[data-type-state="error"] {
  border-bottom: 2px solid red;
}

.expression[data-type-state="incomplete"] {
  border-bottom: 2px dashed yellow;
}

.expression[data-type-state="polymorphic"] {
  border-bottom: 1px solid green;
}

.expression[data-type-state="concrete"] {
  border-bottom: 1px solid blue;
}

.type-tooltip {
  display: none;
  position: absolute;
  background: #333;
  color: white;
  padding: 4px 8px;
}

.expression:hover .type-tooltip {
  display: block;
}
```

### User Workflow Examples

**Example 1: Building Step-by-Step**
```
Step 1: User clicks "+" → □ + □
  🟡 "Type: α + α → α (fill placeholders)"

Step 2: User fills "x" → x + □
  🟡 "Type: α + α → α (fill second operand)"

Step 3: User fills "1" → x + 1
  🟢 "Type: ℝ + ℝ → ℝ" (if no context)
  or 🔵 "Type: ℝ" (if x : ℝ in context)
```

**Example 2: Type Error**
```
User types: A + 3 (where A : Matrix(3,3))
  🔴 "Cannot add Matrix(3,3) to Scalar"
  💡 "Did you mean A + 3·I?"

User fixes: A + 3·I
  🔵 "Type: Matrix(3,3)"
```

---

## Context Management

### EditorTypeContext

```rust
pub struct EditorTypeContext {
    /// Built-in mathematical types
    builtin_types: HashMap<String, Type>,
    
    /// User-defined types
    user_types: HashMap<String, TypeDefinition>,
    
    /// Variable bindings
    variables: HashMap<String, Type>,
    
    /// Type inference engine
    inference: TypeInference,
}
```

### Initialization

```rust
impl EditorTypeContext {
    pub fn with_defaults() -> Self {
        let mut ctx = Self::new();
        
        // Built-in types
        ctx.add_type("ℝ", Type::Scalar);
        ctx.add_type("ℂ", Type::Complex);
        ctx.add_type("Vector", Type::Parametric(1));
        ctx.add_type("Matrix", Type::Parametric(2));
        
        // Built-in operations
        ctx.bind_operation("+", PolymorphicOp {
            signature: "T → T → T",
            constraint: "Numeric(T)",
        });
        
        ctx
    }
}
```

### When to Run Type Checking

**Real-Time (Fast):**
- On every keystroke (debounced 300ms)
- Check for obvious errors
- Update state indicators

**On-Demand (Thorough):**
- User clicks "Verify Types"
- Before save/export
- Full axiom verification

---

## User-Defined Types

### Mathematical Types

```kleis
structure Complex {
  real : ℝ
  imag : ℝ
}

define (+) : Complex × Complex → Complex
define (×) : Complex × Complex → Complex

// Type inference works!
define z = Complex { real: 3, imag: 4 }
define w = Complex { real: 1, imag: 2 }
define sum = z + w
// Inferred: sum : Complex ✓
```

### Business Types

```kleis
structure Money {
  amount : ℝ
  currency : String
  axiom non_negative: amount ≥ 0
}

structure LineItem {
  sku : String
  quantity : ℕ
  price : Money
  lineTotal : Money
  
  axiom line_total_correct:
    lineTotal.amount = quantity × price.amount
}

structure PurchaseOrder {
  orderId : String
  items : List(LineItem)
  total : Money
  
  axiom has_items: length(items) > 0
  axiom total_correct: total = Σᵢ items[i].lineTotal
}

// Type inference:
define order : PurchaseOrder = ...
define total_amount = order.total
// Inferred: total_amount : Money ✓

// Type error:
define bad = order.total + order.orderId
// 🔴 "Cannot add Money to String" ✗
```

### Physics Types

```kleis
structure Particle {
  mass : Mass
  position : Vector(ℝ³)
  velocity : Vector(ℝ³)
  charge : Charge
  
  axiom mass_positive: mass > 0
}

structure Force {
  magnitude : ℝ
  direction : Vector(ℝ³)
  axiom direction_normalized: |direction| = 1
}
```

### Why This Works

**Haskell's type system is domain-agnostic:**
- Mathematical types: Matrix, Vector
- Business types: PurchaseOrder, Invoice
- Physics types: Particle, Force
- **Same type inference algorithm for all!**

### Type Classes Work Universally

```kleis
// Monoid on numbers
implements Monoid(ℝ, +, 0)

// Monoid on lists
implements Monoid(List(T), append, [])

// Monoid on purchase orders!
implements Monoid(PurchaseOrder, combine, emptyOrder)
```

---

## API Design

### `/api/type_check` (Fast, Real-Time)

**Request:**
```json
{
  "expression": { "Operation": { "name": "plus", "args": [...] } },
  "context": {
    "x": "Scalar",
    "A": "Matrix(3,3)"
  }
}
```

**Response:**
```json
{
  "state": "polymorphic",
  "type": "α + α → α",
  "variables": ["α"],
  "message": "🟢 Type: α + α → α (polymorphic)",
  "info": "Valid for any Numeric type"
}
```

### `/api/verify_axioms` (Thorough, On-Demand)

**Request:**
```json
{
  "type_name": "PurchaseOrder",
  "instance": {
    "orderId": "12345",
    "items": [...],
    "total": {"amount": 100, "currency": "USD"}
  }
}
```

**Response:**
```json
{
  "valid": false,
  "violations": [
    {
      "axiom": "total_correct",
      "message": "total (100) ≠ sum of items (150)",
      "expected": 150,
      "actual": 100
    }
  ]
}
```

---

## Frontend Integration

```javascript
class TypeChecker {
  constructor() {
    this.context = new Map();
  }
  
  async checkExpression(ast) {
    const response = await fetch('/api/type_check', {
      method: 'POST',
      body: JSON.stringify({
        expression: ast,
        context: Object.fromEntries(this.context)
      })
    });
    
    const result = await response.json();
    this.updateUI(result);
  }
  
  updateUI(result) {
    // Update visual indicator
    const indicator = document.getElementById('type-indicator');
    indicator.textContent = this.getIndicator(result.state);
    indicator.style.color = this.getColor(result.state);
    
    // Update tooltip
    const tooltip = document.getElementById('type-tooltip');
    tooltip.textContent = result.message;
    
    // Show suggestion
    if (result.suggestion) {
      this.showSuggestion(result.suggestion);
    }
  }
  
  getIndicator(state) {
    return {
      'error': '🔴',
      'incomplete': '🟡',
      'polymorphic': '🟢',
      'concrete': '🔵',
      'unknown': '⚪'
    }[state];
  }
}

// Debounced real-time checking
let typeCheckTimeout;
function onExpressionChange(ast) {
  clearTimeout(typeCheckTimeout);
  typeCheckTimeout = setTimeout(() => {
    typeChecker.checkExpression(ast);
  }, 300);
}
```

---

## Implementation Plan

### Phase 1: Core States ✅
- TypeState enum
- Basic type checking
- POC complete

### Phase 2: Visual Feedback (Week 1)
- Colored indicators
- Hover tooltips
- Suggestion display

### Phase 3: Context UI (Week 2)
- Variable binding UI
- Type definition UI
- Import management

### Phase 4: User Types (Week 3)
- Record type editor
- Axiom editor
- Instance verification UI

---

## Success Criteria

✅ < 100ms type checking for real-time feedback  
✅ < 5% false positive rate  
✅ Visual feedback integrated smoothly  
✅ 90%+ users find it helpful (not annoying)  
✅ Works for math AND business types  

---

**This makes type checking helpful, not intrusive!** 🎯

