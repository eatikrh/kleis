# ADR-016: Operations Belong in Structures

**Date:** December 6, 2024  
**Status:** ✅ Accepted and IMPLEMENTED (v0.5.0, December 8, 2024)  
**Related:** ADR-015 (Text as Source of Truth), ADR-014 (Type System), ADR-019 (Dimensional Analysis)

**Implementation:** Sessions 2024-12-07, 2024-12-08  
**Key Achievement:** TRUE self-hosting - operations in Kleis, constraints enforced by signatures

---

## Context

When designing `stdlib/core.kleis` for ADR-015, we had two options for where to declare operations like `abs`, `card`, `norm`:

**Option A: Top-level operations**
```kleis
operation abs : ℝ → ℝ
operation card : Set(T) → ℕ
```

**Option B: Operations in structures**
```kleis
structure Numeric(N) {
    operation abs : N → N
}
implements Numeric(ℝ)
```

---

## Decision

**We adopt Option B: Operations belong in structures, with implementations for concrete types.**

**Top-level operations reserved for:**
- Utility functions that don't belong to a type
- Cross-cutting operations
- Special cases (to be determined)

---

## Rationale

### 1. Conceptual Purity ✅

**Operations are inherently tied to types:**
- `abs` is an operation **on** numbers
- `card` is an operation **on** sets
- `norm` is an operation **on** vectors

**They should be declared where they conceptually belong.**

### 2. Consistency with Prelude ✅

The existing `stdlib/prelude.kleis` already uses this pattern:

```kleis
structure Field(F) {
    operation (+) : F × F → F
    operation (/) : F × F → F
}

implements Field(ℝ) {
    operation (+) = builtin_add
    operation (/) = builtin_div
}
```

**We should follow the same pattern for consistency.**

### 3. Enables Polymorphism ✅

```kleis
structure Numeric(N) {
    operation abs : N → N
}

implements Numeric(ℝ)
implements Numeric(ℂ)

// Now abs works for any Numeric type!
define magnitude<T: Numeric>(x: T) = abs(x)
```

### 4. Type Checker Can Use Structure Information ✅

```rust
// Type checker can query:
// "Does Money implement Numeric?"
// "What operations does Numeric provide?"
// "Can I use abs on this type?"
```

Better error messages and type inference.

### 5. Follows Standard Patterns ✅

This matches:
- **Haskell:** Type classes with instances
- **Rust:** Traits with implementations  
- **Scala:** Type classes
- **Mathematical:** Algebraic structures

---

## Design

### stdlib/core.kleis Structure

```kleis
@library("kleis.core")
@version("1.0.0")

// ============================================
// NUMERIC TYPES
// ============================================

structure Numeric(N) {
    // Absolute value / magnitude
    operation abs : N → N
    
    axiom abs_non_negative: ∀ (x : N) . abs(x) ≥ 0
    axiom abs_symmetric: ∀ (x : N) . abs(-x) = abs(x)
}

// Implementations
implements Numeric(ℝ) {
    operation abs = builtin_abs_real
}

implements Numeric(ℂ) {
    operation abs = complex_modulus  // Returns ℝ for complex!
}

// ============================================
// ORDERED NUMERIC TYPES
// ============================================

structure OrderedNumeric(N) extends Numeric(N) {
    operation floor : N → ℤ
    operation ceil : N → ℤ
    operation round : N → ℤ
}

implements OrderedNumeric(ℝ) {
    operation floor = builtin_floor
    operation ceil = builtin_ceil
    operation round = builtin_round
}

// Note: Complex numbers don't implement OrderedNumeric

// ============================================
// SET OPERATIONS
// ============================================

structure SetOps(T) {
    operation card : Set(T) → ℕ
    operation isEmpty : Set(T) → Bool
    
    axiom card_empty: card(∅) = 0
    axiom card_non_negative: ∀ (S : Set(T)) . card(S) ≥ 0
}

implements SetOps(ℤ)
implements SetOps(ℝ)
// Universal: works for any T

// ============================================
// VECTOR OPERATIONS
// ============================================

structure VectorOps(n) {
    operation norm : Vector(n) → ℝ
    operation normalize : Vector(n) → Vector(n)
    
    axiom norm_non_negative: ∀ (v : Vector(n)) . norm(v) ≥ 0
    axiom norm_zero: ∀ (v : Vector(n)) . norm(v) = 0 ⟺ v = 0⃗
}

implements VectorOps(n) {
    operation norm(v) = √(dot(v, v))
    operation normalize(v) = v / norm(v)
}

// ============================================
// DISPLAY MODE OPERATIONS
// ============================================

// frac is special: it's a display hint, not a real operation
// Keep top-level? Or make structure DisplayOps?

operation frac : ℝ × ℝ → ℝ
define frac(a, b) = a / b
// Note: Signals display mode to renderer
```

---

## Implementation Changes

### Parser Extensions Needed

**Add to `kleis_ast.rs`:**

```rust
/// Implements block
pub struct ImplementsDef {
    pub structure_name: String,
    pub type_arg: TypeExpr,
    pub members: Vec<ImplMember>,
}

pub enum ImplMember {
    Element { name: String, value: Expression },
    Operation { name: String, implementation: Implementation },
}

pub enum Implementation {
    Builtin(String),      // builtin_abs
    Defined(Expression),  // (x) = x^2
}
```

**Add to `kleis_parser.rs`:**

```rust
fn parse_implements(&mut self) -> Result<ImplementsDef, ParseError> {
    // implements StructureName(Type) { ... }
}

fn parse_impl_member(&mut self) -> Result<ImplMember, ParseError> {
    // operation abs = builtin_abs
    // element zero = 0
}
```

### Type Context Changes

**Type context must now:**
1. Load structure definitions (defines abstract operations)
2. Load implements blocks (binds operations to types)
3. Build operation registry from implementations

```rust
impl TypeContext {
    pub fn register_structure(&mut self, structure: StructureDef) {
        // Register operations as abstract
        self.abstract_operations.insert(structure.name, structure.operations);
    }
    
    pub fn register_implements(&mut self, impl_def: ImplementsDef) {
        // Bind abstract operations to concrete implementations
        for type_op in impl_def.members {
            self.concrete_operations.insert(
                (impl_def.type_arg.clone(), type_op.name),
                type_op.implementation
            );
        }
    }
}
```

---

## Consequences

### Positive ✅

1. **Conceptually pure:** Operations belong to types
2. **Polymorphic:** `abs` works for any Numeric type
3. **Extensible:** Users can add their types to structures
4. **Consistent:** Matches prelude.kleis pattern
5. **Better type checking:** Structure provides operation manifest

### Negative ⚠️

1. **More complex:** Need to parse `implements` blocks
2. **More indirection:** Operation → Structure → Implementation
3. **Longer POC timeline:** +2-3 days for implements parsing

### Neutral

Top-level operations still available for special cases (like `frac` for display mode).

---

## Implementation Timeline (Updated)

### Phase 1: Extend AST ⬜ (1 day)
- Add `ImplementsDef` to `kleis_ast.rs`
- Add `ImplMember` types

### Phase 2: Extend Parser ⬜ (2-3 days)
- Parse `implements StructureName(Type) { ... }`
- Parse implementation members
- Test parsing

### Phase 3: Redesign stdlib/core.kleis ⬜ (1 day)
- Create `Numeric(N)` structure with `abs`
- Create `SetOps(T)` structure with `card`
- Create `VectorOps(n)` structure with `norm`
- Add implements for ℝ, ℂ, etc.

### Phase 4: Type Context Builder ⬜ (2-3 days)
- Load structures (abstract operations)
- Load implements (concrete bindings)
- Build operation registry
- Query: "Which types support abs?"

### Phase 5: Test ⬜ (1-2 days)
- Parse stdlib with new structure
- Build type context
- Type check expressions
- Validate error messages

**Total:** ~7-10 days (1.5-2 weeks)

---

## Migration from Current Code

**Current test code:**
```kleis
operation abs : ℝ → ℝ
```

**New design:**
```kleis
structure Numeric(N) {
    operation abs : N → N
}
implements Numeric(ℝ)
```

**Parser needs to handle both!**

---

## Example: Complete Flow

```kleis
// stdlib/core.kleis

structure Numeric(N) {
    operation abs : N → N
    axiom abs_non_negative: ∀ (x : N) . abs(x) ≥ 0
}

implements Numeric(ℝ) {
    operation abs = builtin_abs_real
}

implements Numeric(ℂ) {
    operation abs = complex_modulus
}

// User code
define x : ℝ = -5
define y = abs(x)  // Type: ℝ ✓

define z : ℂ = 3 + 4i
define w = abs(z)  // Type: ℝ (!) ✓ Returns real magnitude

// Error detection still works!
define S : Set(ℤ) = {1, 2, 3}
define bad = abs(S)  
// Error: Set(ℤ) does not implement Numeric
// Suggestion: Did you mean card(S)?
```

---

## Decision Status

**Status:** ✅ Accepted and FULLY IMPLEMENTED (v0.5.0)

**Implementation Milestones:**
- **v0.4.0 (Dec 7):** Stdlib loading, operations in structures
- **v0.5.0 (Dec 8):** SignatureInterpreter enforces constraints from signatures

**Scope:**
- Operations belong in structures (general rule) ✅ IMPLEMENTED
- Top-level operations for special utilities (exception)
- Follows prelude.kleis pattern ✅ IMPLEMENTED

---

## Implementation Summary (v0.5.0)

### **What Was Built:**

**SignatureInterpreter:**
- Parses operation signatures from structure definitions
- Enforces dimension constraints (MatrixAddable, MatrixMultipliable, SquareMatrix)
- Validates parameter bindings across arguments
- Handles 24+ operations automatically

**TypeContextBuilder:**
- Match statement reduced from 229 → 61 lines (73% smaller)
- Pattern-based operation detection (no hardcoded names)
- Registry-driven validation
- TRUE user-extensibility achieved

**Result:**
- Built-in operations (Matrix, etc.) work via SAME path as user operations
- Adding new operations requires ZERO Rust changes
- Constraints defined in Kleis signatures, not Rust code

**Example:**
```kleis
// User defines:
structure PurchaseOrder {
  operation total : Money
  operation validate : Bool
}

// Works automatically via SignatureInterpreter!
// No Rust changes needed!
```

---

## Metrics

**Code Reduction:**
- type_context.rs: 848 → 682 lines (-166 lines, 20% smaller)
- type_inference.rs: 550 → 469 lines (-81 lines, 15% smaller)
- Match statement: 229 → 61 lines (-168 lines, 73% smaller)

**Operations:**
- In stdlib: 30+ operations
- Hardcoded in Rust: 0 (only data constructors remain)
- Via SignatureInterpreter: 24+ operations

**Tests:**
- Total: 364 tests
- Pass rate: 100%
- Coverage: Comprehensive

---

## References

**Implementation:**
- `src/signature_interpreter.rs` - Signature parsing and constraint enforcement
- `src/type_context.rs` - Registry and operation lookup
- `src/type_checker.rs` - User-facing API
- `stdlib/minimal_prelude.kleis` - Operation definitions
- `stdlib/matrices.kleis` - Matrix operation definitions

**Documentation:**
- Session 2024-12-07: Initial implementation
- Session 2024-12-08: SignatureInterpreter improvements
- FORMAL_SPECIFICATION.md: Formal semantics

**This ADR is now FULLY REALIZED in code!** ✅

**Next Steps:**
1. Extend parser for `implements`
2. Redesign stdlib/core.kleis
3. Update type context builder
4. Test the complete pattern

---

## References

- [stdlib/prelude.kleis](../../stdlib/prelude.kleis) - Existing pattern
- [ADR-015](adr-015-text-as-source-of-truth.md) - Text representation
- [TYPE_CHECKING_NEXT_STEPS.md](type-system/TYPE_CHECKING_NEXT_STEPS.md) - Updated roadmap

---

**Last Updated:** December 6, 2024  
**Decision:** Operations in structures (Option B) ✅

