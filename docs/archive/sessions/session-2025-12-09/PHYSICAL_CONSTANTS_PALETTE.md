# Physical Constants in Palette - Design Note

**Date:** December 9, 2025  
**Context:** Universal constants finding from Einstein equations  
**Status:** Design requirement identified

---

## The Requirement

**Physical constants need palette entries with proper types.**

Constants are NOT just numeric values in .kleis files - they are:
- **Typed entities** in the type system
- **Unit-aware quantities** (value + unit + dimension)
- **Context-dependent** (Lambda means different things)
- **User-facing** (need UI access)

---

## Why Palette Entries?

### The Palette is the Semantic Source

**Principle established today:**
> "Semantic info comes from Structural editing using the Palette" (not LaTeX)

**This applies to constants too:**
- User clicks "Cosmological Constant Λ" button
- Creates: `Constant { name: "Lambda_cosmo", type: PhysicalConstant(...) }`
- NOT just: `Object("\\Lambda")` (notation only)

**The palette choice IS the semantic information.**

### Constants are Domain-Specific

**Different palette tabs should have different constants:**

**Physics Tab:**
- Λ (Cosmological constant) - 1.089×10⁻⁵² m⁻²
- G (Gravitational constant) - 6.674×10⁻¹¹ m³ kg⁻¹ s⁻²
- c (Speed of light) - 299,792,458 m s⁻¹
- ℏ (Reduced Planck) - 1.055×10⁻³⁴ J·s
- kB (Boltzmann) - 1.381×10⁻²³ J·K⁻¹

**Quantum Tab:**
- e (Elementary charge) - 1.602×10⁻¹⁹ C
- me (Electron mass) - 9.109×10⁻³¹ kg
- mp (Proton mass) - 1.673×10⁻²⁷ kg
- α (Fine structure) - 1/137.036
- ℏ (Planck) - different context than physics!

**Optics Tab:**
- λ (Wavelength) - user-specified, unit: m
- ν (Frequency) - user-specified, unit: Hz
- n (Refractive index) - dimensionless

**Context (palette tab) determines meaning!**

---

## Design Proposal

### Palette Entry Structure

**For each constant, create a button:**

```javascript
// In static/index.html, Physics tab
{
    symbol: "Λ",
    name: "Cosmological Constant",
    tooltip: "Λ ≈ 1.089×10⁻⁵² m⁻²",
    ast: {
        Constant: {
            name: "Lambda_cosmological",
            value: 1.089e-52,
            unit: "m^-2",
            type: {
                Data: {
                    type_name: "PhysicalConstant",
                    args: [
                        { NatValue: /* encoded value */ },
                        { StringValue: "m^-2" }
                    ]
                }
            }
        }
    }
}
```

**When user clicks:**
1. Inserts Constant node (not just Object)
2. Type system knows: `PhysicalConstant(1.089e-52, "m^-2")`
3. Can validate dimensional consistency
4. No ambiguity about which "lambda"!

### AST Extension Needed

**Add to `src/ast.rs`:**

```rust
pub enum Expression {
    // ... existing variants ...
    
    /// Physical constant with type, value, and unit
    Constant {
        name: String,           // "Lambda_cosmological"
        value: Option<f64>,     // Numeric value (if known)
        unit: String,           // "m^-2"
        dimension: Vec<i32>,    // [L, M, T] exponents: [-2, 0, 0]
    },
}
```

**The value is optional!** For symbolic computation, we only need:
- Type (PhysicalConstant)
- Unit (m⁻²)
- Dimensional structure ([−2, 0, 0])

**Numeric values are only needed for:**
- Numerical evaluation
- Plotting
- Concrete calculations

**For type checking and symbolic manipulation, we don't need them!**

---

## Numeric Values: Where to Keep Them?

### Option 1: Separate Numeric Registry

```kleis
// In stdlib/physics_values.kleis (separate from types!)
numeric_values {
  Lambda_cosmological = 1.089e-52
  G_Newton = 6.674e-11
  c_light = 299792458
}
```

**Loaded separately** from type definitions. Type checking doesn't need values!

### Option 2: External Database

```json
// physics_constants.json
{
  "Lambda_cosmological": {
    "value": 1.089e-52,
    "unit": "m^-2",
    "uncertainty": 0.024e-52,
    "source": "Planck 2018"
  }
}
```

**Loaded at runtime** only when numerical evaluation is needed.

### Option 3: Compile-Time Constants (Future)

```kleis
const Lambda : PhysicalConstant = {
  value: 1.089e-52,
  unit: "m^-2",
  dimension: [-2, 0, 0]
}
```

**Embedded in AST** for constants that are known at compile time.

### Recommended: Hybrid Approach

**For TYPE CHECKING:**
- Constants have types and units (from palette/stdlib)
- No numeric values needed
- Fast symbolic manipulation

**For EVALUATION:**
- Load numeric values on demand
- From external database or registry
- Only when computing concrete results

**Separation of concerns:**
- Type system: Units and dimensional structure
- Evaluator: Numeric values
- Clean architecture!

---

## Implementation Plan

### Phase 1: Palette Entries (Next Session)

**Add to static/index.html, Physics tab:**

```javascript
const physicsConstants = {
    // GR Constants
    lambda_cosmo: {
        symbol: 'Λ',
        tooltip: 'Cosmological constant (≈1.089×10⁻⁵² m⁻²)',
        ast: { Constant: { name: 'Lambda_cosmological', unit: 'm^-2' } }
    },
    kappa_einstein: {
        symbol: 'κ',
        tooltip: 'Einstein constant 8πG/c⁴',
        ast: { Constant: { name: 'kappa_Einstein', unit: 'm^-1 kg^-1 s^2' } }
    },
    // Universal Constants
    G: {
        symbol: 'G',
        tooltip: 'Gravitational constant (6.674×10⁻¹¹ m³ kg⁻¹ s⁻²)',
        ast: { Constant: { name: 'G_Newton', unit: 'm^3 kg^-1 s^-2' } }
    },
    c: {
        symbol: 'c',
        tooltip: 'Speed of light (299,792,458 m s⁻¹)',
        ast: { Constant: { name: 'c_light', unit: 'm s^-1' } }
    }
};
```

### Phase 2: AST Support

Add `Expression::Constant` variant with unit info.

### Phase 3: Type Definitions

```kleis
structure PhysicalConstant(unit: String, dimension: List(Int)) {
  operation to_real : ℝ
  operation get_unit : String
}
```

### Phase 4: Numeric Values (Optional)

Separate file or database for actual numeric values, loaded only when needed.

---

## Benefits

**With constant palette entries:**

1. **No ambiguity** - "Λ (cosmological)" is different from "λ (wavelength)"
2. **Type-checkable** - Dimensional analysis validates equations
3. **User-friendly** - Click button, get properly typed constant
4. **Extensible** - Users can add their own constants
5. **Scoped** - Different tabs have different contexts
6. **Unit-aware** - Type system enforces dimensional consistency

**Einstein equations will then type-check as:**
```
Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ) ✓
```

**With full dimensional validation!**

---

## Example Workflow

**User building Einstein equations:**

1. Click "Einstein Tensor" → `einstein(?, ?, ?)`
2. Fill in metric tensor
3. Click "+" button → `plus(...)`
4. Click "Λ (Cosmological Constant)" → Inserts typed constant! 
5. Click "×" button → `scalar_multiply(...)`
6. Click metric tensor
7. Click "=" 
8. Click "κ (Einstein Constant)" → Inserts typed constant!
9. Click "×"
10. Fill in stress-energy tensor

**Result:** Properly typed equation with dimensional validation! ✅

**No manual constant declaration needed** - palette provides it!

---

## Related Design Decisions

**This connects to:**
- ADR-009: WYSIWYG Structural Editor (palette is semantic)
- ADR-019: Dimensional Type Checking (units as types)
- ADR-015: Text as Source of Truth (but semantics from palette!)

**New requirement identified:**
- **Constants should be first-class in the palette**
- **Physical quantities need unit-aware types**
- **Numeric values are optional (for type checking)**

---

## Questions for Next Session

1. How should `Expression::Constant` be structured?
2. Where do numeric values live (if ever needed)?
3. How do users add custom constants to palette?
4. Should units use String or structured type?
5. How to handle unit conversions (SI ↔ CGS ↔ natural)?

---

## The Vision

**Eventually, Kleis palette should have:**

```
[Constants Tab]
┌─────────────┬─────────────┬─────────────┐
│ G (6.67e-11)│ c (2.99e8)  │ ℏ (1.05e-34)│
│  Gravity    │   Light     │   Planck    │
│  [m³kg⁻¹s⁻²]│   [m·s⁻¹]   │   [J·s]     │
└─────────────┴─────────────┴─────────────┘
```

**Click button → Properly typed constant in your equation!**

No more "what's kappa again?" - the tooltip and type tell you everything.

---

**This architectural insight came from investigating type inference.** 🎯

**The type system revealed what the design should be!** 🎊

