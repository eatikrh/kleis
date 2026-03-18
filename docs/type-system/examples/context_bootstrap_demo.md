# Type Context Bootstrap Demo

**Shows how type context is populated and used**

---

## Scenario: User Opens Editor and Types `a + b`

### Step 1: Server Starts

```
Initializing Kleis type system...
✓ Core types loaded (Scalar, Bool, String, Nat, Vector, Matrix)
✓ Standard library loaded: stdlib/prelude.kleis (15ms)
  - Loaded: Monoid, Group, Ring, Field structures
  - Loaded: 47 operations
  - Loaded: Numeric(ℝ), Numeric(ℂ), VectorSpace implementations
✓ Type system ready

Context now contains:
  Types: [ℝ, ℂ, ℤ, ℕ, Vector(n), Matrix(m,n), List(T)]
  Structures: [Semigroup, Monoid, Group, AbelianGroup, Ring, Field, VectorSpace]
  Operations: [+, -, ×, /, ∂, ∫, ∇, dot, cross, det, trace, ...]
  Constants: [π, e, i, φ, √2]
```

### Step 2: User Opens Editor

**Initial context has everything from stdlib/prelude.kleis**

```
Browser → GET /api/type_context/summary

Response:
{
  "types": ["ℝ", "ℂ", "ℤ", "ℕ", "Vector", "Matrix", ...],
  "structures": ["Monoid", "Group", "Ring", "Field", ...],
  "operations_count": 47,
  "ready": true
}
```

### Step 3: User Clicks "+" Button

**Editor inserts:** `□ + □`

```
Browser → POST /api/type_check
{
  "expression": {
    "Operation": {
      "name": "plus",
      "args": [
        {"Placeholder": {"id": 0, "hint": "left"}},
        {"Placeholder": {"id": 1, "hint": "right"}}
      ]
    }
  },
  "context": {}
}

Response:
{
  "state": "incomplete",
  "type": "α + α → α",
  "message": "🟡 Type: α (fill placeholders)",
  "info": "Addition available for: ℝ, ℂ, Vector(n), Matrix(m,n)"
}
```

**UI shows:**
```
Expression: □ + □
🟡 Type: α (incomplete)
Hint: "Fill placeholders. Addition works for: ℝ, ℂ, Vector, Matrix"
```

### Step 4: User Fills First Placeholder with "a"

**Editor updates:** `a + □`

```
Browser → POST /api/type_check
{
  "expression": {
    "Operation": {
      "name": "plus",
      "args": [
        {"Object": "a"},
        {"Placeholder": {"id": 1, "hint": "right"}}
      ]
    }
  },
  "context": {}
}

Response:
{
  "state": "incomplete",
  "type": "α + α → α",
  "message": "🟡 Type: α (fill second operand)",
  "info": "Variable 'a' has inferred type α. Fill second placeholder with same type."
}
```

**UI shows:**
```
Expression: a + □
🟡 Type: α (incomplete)
Hint: "Fill second operand. Type will match 'a'"
```

### Step 5: User Fills Second Placeholder with "b"

**Editor updates:** `a + b`

```
Browser → POST /api/type_check
{
  "expression": {
    "Operation": {
      "name": "plus",
      "args": [
        {"Object": "a"},
        {"Object": "b"}
      ]
    }
  },
  "context": {}
}

Response:
{
  "state": "polymorphic",
  "type": "∀α. Numeric(α) ⇒ α",
  "variables": ["α"],
  "message": "🟢 Type: α where Numeric(α) (polymorphic)",
  "info": "Valid for any type implementing Numeric: ℝ, ℂ, Vector(n), Matrix(m,n)",
  "possible_types": ["ℝ", "ℂ", "Vector(n)", "Matrix(m,n)", "Polynomial"]
}
```

**UI shows:**
```
Expression: a + b
🟢 Type: α where Numeric(α)
Possible types: ℝ, ℂ, Vector(n), Matrix(m,n), Polynomial
Info: "Polymorphic - valid for any Numeric type"
```

### Step 6: User Adds Context (Defines a)

**User types:** `define a : Vector(3)`

**Context updated:** `{a: Vector(3)}`

```
Browser → POST /api/type_check
{
  "expression": { ... same ... },
  "context": {
    "a": "Vector(3)"
  }
}

Response:
{
  "state": "polymorphic",
  "type": "Vector(3) + Vector(3) → Vector(3)",
  "message": "🟢 Type: Vector(3) (b inferred as Vector(3))",
  "info": "Both operands must be Vector(3)"
}
```

**UI shows:**
```
Expression: a + b
🔵 Type: Vector(3)
Info: "a : Vector(3), b : Vector(3) (inferred)"
```

---

## What Enables This?

### 1. Standard Library Loaded at Startup

**File:** `stdlib/prelude.kleis`

Contains:
```kleis
structure Numeric(T) {
  operation (+) : T × T → T
  ...
}

implements Numeric(ℝ)
implements Numeric(ℂ)
implements Numeric(Vector(n))
```

### 2. Operation Registry Built

```rust
OperationRegistry {
  "+": [
    (ℝ, Numeric(ℝ)),
    (ℂ, Numeric(ℂ)),
    (Vector(n), Numeric(Vector(n))),
    (Matrix(m,n), Numeric(Matrix(m,n))),
  ]
}
```

### 3. Type Inference Queries Registry

```rust
// When seeing: a + b
query_types_supporting("+") → [ℝ, ℂ, Vector(n), Matrix(m,n)]
generate_constraint: a ∈ {ℝ, ℂ, Vector(n), Matrix(m,n)}
```

---

## Another Example: Multiplication Ambiguity

### User Types: `v × w`

**Initial (no context):**
```
Query: types_supporting("×")
Result: [ℝ, ℂ, Vector(n), Matrix(m,n)]

But (×) means different things:
- Scalar × Scalar → Scalar
- Vector × Vector → Scalar (dot product)
- Matrix × Matrix → Matrix
- Matrix × Vector → Vector

UI shows:
🟡 Multiple interpretations:
  1. Scalar multiplication (v,w : ℝ)
  2. Dot product (v,w : Vector(n))
  3. Matrix multiply (v,w : Matrix)
  
Add context to disambiguate.
```

**User adds:** `define v : Vector(3)`

```
Query: (×) : Vector(3) × ? → ?
Rules: Vector × Vector → Scalar (dot)
       Vector × Scalar → Vector (scalar mul)

UI shows:
🟢 Type: Vector(3) × α → β
Constraints: (α=Vector(3) ∧ β=Scalar) ∨ (α=Scalar ∧ β=Vector(3))

If w : Vector(3) → dot product → Scalar
If w : Scalar → scalar mul → Vector(3)
```

---

## Summary

### Answer to "How do we populate context?"

**Three tiers:**

1. **Hardcoded (Rust):** Primitive types only
   ```rust
   ctx.register("Scalar");
   ctx.register("Vector(n)");
   ```

2. **Standard Library (Kleis):** Everything else
   ```kleis
   // stdlib/prelude.kleis
   structure Monoid(M) { ... }
   implements Numeric(ℝ) { ... }
   ```

3. **User Workspace (Kleis):** Custom types
   ```kleis
   // workspace/user_types.kleis
   structure PurchaseOrder { ... }
   ```

### How it works for `a + b`

1. ✅ Editor loads → stdlib/prelude.kleis parsed
2. ✅ Context knows: `+` available for `[ℝ, ℂ, Vector, Matrix]`
3. ✅ User types `a + b`
4. ✅ Type system queries: "which types support `+`?"
5. ✅ Result: `α where Numeric(α)` (polymorphic)
6. ✅ User adds context → specializes to concrete type

---

**Yes! Use Kleis code for the standard library - it's self-hosting and visible to users!** 🎯

**Files created:**
- `stdlib/prelude.kleis` - Standard library (actual Kleis code!)
- `docs/type-system/TYPE_CONTEXT_BOOTSTRAP.md` - Bootstrap strategy
- `docs/type-system/OPERATION_BASED_TYPE_INFERENCE.md` - How queries work
