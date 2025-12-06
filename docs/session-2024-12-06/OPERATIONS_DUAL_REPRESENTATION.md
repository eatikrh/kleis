# Operations: Dual Representation Question

**Date:** December 6, 2024  
**Status:** Design Discussion  
**Question:** Where do operations "belong"?

---

## The Conceptual Tension

### Current Approach (Two Ways)

**1. Top-level operations:**
```kleis
operation abs : ℝ → ℝ
operation card : Set(T) → ℕ
operation norm : Vector(n) → ℝ
```

**2. Operations inside structures:**
```kleis
structure Field(F) {
    operation (+) : F × F → F
    operation (/) : F × F → F
}

implements Field(ℝ) {
    operation (+) = builtin_add
}
```

### The Question

**Conceptually, operations are part of types:**
- `abs` is an operation **on** ℝ
- `card` is an operation **on** Set(T)
- `norm` is an operation **on** Vector(n)

**So should they be declared top-level or inside type definitions?**

---

## What the Formal Grammar Says

Looking at `Kleis_v03.g4` and `stdlib/prelude.kleis`:

### Grammar Supports BOTH

```antlr
declaration
    : structureDef          // Can contain operations
    | operationDecl         // Top-level operations
    | ...
    ;

structureMember
    : operationDecl         // Operations inside structures
    | ...
    ;
```

**So the grammar intentionally allows both!**

---

## How stdlib/prelude.kleis Uses This

### Pattern 1: Operations Inside Structures (Abstract)

```kleis
structure Field(F) {
    operation (+) : F × F → F
    operation (×) : F × F → F
    operation (/) : F × F → F
    operation negate : F → F
    operation inverse : F → F
}
```

**These are abstract!** They define what a Field **must have**, not the implementation.

### Pattern 2: Implements Binds to Concrete Types

```kleis
implements Field(ℝ) {
    element zero = 0
    element one = 1
    operation (+) = builtin_add
    operation (×) = builtin_mul
    operation negate(x) = -x
    operation inverse(x) = 1/x
}
```

**This binds the abstract operations to actual implementations.**

### Pattern 3: Top-Level Operations (Utilities)

Looking at the grammar and docs, top-level operations are for:
- Operations that don't belong to a single type
- Cross-cutting operations
- Utility functions

**Examples:**
```kleis
// Dot product: operates on TWO vectors
operation dot : ∀n. Vector(n) × Vector(n) → ℝ

// Determinant: operates on matrix, returns scalar
operation det : ∀n. Matrix(n,n) → ℝ
```

---

## The Design Pattern (From Prelude)

### For Algebraic Operations (like +, ×)

**Use structure + implements:**

```kleis
// Step 1: Define structure
structure Field(F) {
    operation (+) : F × F → F
}

// Step 2: Implement for concrete type
implements Field(ℝ) {
    operation (+) = builtin_add
}
```

**Benefit:** Polymorphism! Any type that implements Field gets (+).

### For Type-Specific Operations (like abs, norm)

**Two options:**

**Option A: Top-level (current approach)**
```kleis
operation abs : ℝ → ℝ
operation norm : Vector(n) → ℝ
```

**Option B: Inside a structure (conceptually better?)**
```kleis
structure RealNumbers {  // Define ℝ as a structure
    operation abs : ℝ → ℝ
    operation sqrt : ℝ → ℝ
    operation floor : ℝ → ℤ
}

// But ℝ is primitive, so this feels weird...
```

---

## The Fundamental Issue: ℝ is Primitive

### The Problem

```kleis
// ℝ is built into the type system (Tier 1: hardcoded)
// So where do its operations live?

// Option A: Top-level
operation abs : ℝ → ℝ

// Option B: In a structure (but ℝ is not user-defined!)
structure ??? {
    operation abs : ℝ → ℝ  // Doesn't make sense
}
```

**Tension:** ℝ is special (primitive), but conceptually it HAS operations.

---

## Three Possible Approaches

### Approach 1: Top-Level for Primitives ✅ (Current)

```kleis
// Primitives (ℝ, ℂ, ℤ, ℕ) are hardcoded
// Their operations declared top-level

operation abs : ℝ → ℝ
operation card : Set(T) → ℕ
operation norm : Vector(n) → ℝ
```

**Pros:**
- Simple and direct
- Works immediately
- Clear what type each operates on (from signature)

**Cons:**
- Operations separated from their types
- Not as "object-oriented"
- Doesn't use structure system for primitives

---

### Approach 2: Implicit Structure for Primitives

```kleis
// Conceptually, ℝ HAS a structure
// (Even though it's primitive in implementation)

structure RealNumber {
    // This is ℝ
    operation abs : ℝ → ℝ
    operation floor : ℝ → ℤ
    operation ceil : ℝ → ℤ
}

// The type ℝ "is" RealNumber
type ℝ = RealNumber
```

**Pros:**
- Conceptually cleaner
- Operations belong to their types
- Consistent with user-defined types

**Cons:**
- ℝ is primitive (hardcoded in Rust), creating a structure feels artificial
- Extra indirection
- Type alias might be confusing

---

### Approach 3: Implements for Primitives (Like prelude.kleis does!)

**Looking at the actual prelude.kleis:**

```kleis
// Field is abstract
structure Field(F) {
    operation (+) : F × F → F
    operation (/) : F × F → F
}

// ℝ implements Field!
implements Field(ℝ) {
    operation (+) = builtin_add
    operation (/) = builtin_div
}
```

**So primitives DO get operations via implements!**

But what about `abs`? It's specific to ℝ (not every Field has abs).

**Solution: Extension operations**

```kleis
// Core Field operations (via implements)
implements Field(ℝ) {
    operation (+) = ...
    operation (/) = ...
}

// ℝ-specific operations (top-level)
operation abs : ℝ → ℝ
operation floor : ℝ → ℤ
operation ceil : ℝ → ℤ
```

**This is the current pattern!**

---

## What The Prelude Actually Does

Looking at `stdlib/prelude.kleis`:

### Algebraic Operations: Inside Structures + Implements

```kleis
structure Field(F) { operation (+) : ... }
implements Field(ℝ) { operation (+) = builtin_add }
```

### Utility/Specific Operations: Top-Level

```kleis
operation dot : ∀n. Vector(n) × Vector(n) → ℝ
operation det : ∀n. Matrix(n,n) → ℝ
```

**Pattern:**
- **Polymorphic/abstract operations** → Inside structures
- **Type-specific operations** → Top-level

---

## Proposed Resolution

### For stdlib/core.kleis (ADR-015)

**Keep top-level for now:**

```kleis
// These are ℝ-specific, not polymorphic
operation abs : ℝ → ℝ
operation floor : ℝ → ℤ
operation ceil : ℝ → ℤ

// These are type-specific utilities
operation card : ∀T. Set(T) → ℕ
operation norm : ∀n. Vector(n) → ℝ
```

**Rationale:**
1. Simple and clear
2. Signature shows which type it operates on
3. Not trying to make ℝ into a structure (it's primitive)
4. Follows pattern from prelude for utility operations

### For User-Defined Types

**Use structures + operations:**

```kleis
structure Money {
    amount : ℝ
    currency : String
    
    // Operations belong inside!
    operation abs : Money → ℝ
    operation (+) : Money × Money → Money
}
```

**Benefit:** Operations clearly belong to the type.

### For Polymorphic Operations

**Use structures + implements (prelude pattern):**

```kleis
structure Monoid(M) {
    operation (+) : M × M → M
}

implements Monoid(ℝ)
implements Monoid(Money)  // User can add!
```

---

## The Dual Representation Serves Different Purposes

### Top-Level Operations

**For:**
- Operations on primitives (ℝ, ℤ, ℕ)
- Utility functions that don't "belong" to one type
- Cross-type operations (dot, det)

**Example:**
```kleis
operation abs : ℝ → ℝ        // ℝ is primitive, can't modify it
operation dot : Vector × Vector → ℝ  // Operates on two vectors
```

### Operations Inside Structures

**For:**
- Defining abstract interfaces (Monoid, Group, Field)
- User-defined types with their operations
- Polymorphic operation signatures

**Example:**
```kleis
structure Money {
    operation (+) : Money × Money → Money  // Belongs to Money
}
```

---

## Comparison to Other Languages

### Haskell: Type Classes (Similar to Our Structures)

```haskell
-- Type class (abstract)
class Num a where
    (+) :: a -> a -> a
    abs :: a -> a

-- Instance (concrete)
instance Num Double where
    (+) = primAddDouble
    abs = primAbsDouble
```

**Our equivalent:**
```kleis
structure Numeric(N) {
    operation (+) : N × N → N
    operation abs : N → N
}

implements Numeric(ℝ) {
    operation (+) = builtin_add
    operation abs = builtin_abs
}
```

**So `abs` COULD be part of a Numeric structure!**

### Python/OOP: Methods on Classes

```python
class float:
    def __abs__(self): ...
    def __add__(self, other): ...
```

**Our equivalent would be:**
```kleis
structure RealNumber {
    operation abs : ℝ → ℝ
    operation (+) : ℝ × ℝ → ℝ
}
```

---

## Recommendation

### For ADR-015 / Current POC: Keep Top-Level ✅

**Rationale:**
1. Simple to parse and implement
2. Clear signatures show types
3. Works for POC validation
4. Primitives (ℝ, ℕ) don't have structure definitions

### For Future: Three-Tier System

**Tier 1: Algebraic operations (in structures)**
```kleis
structure Numeric(N) {
    operation (+) : N × N → N
    operation abs : N → N
}

implements Numeric(ℝ)
implements Numeric(ℂ)
```

**Tier 2: Type-specific operations (in user structures)**
```kleis
structure Money {
    operation convert : Money × Currency → Money
}
```

**Tier 3: Utility operations (top-level)**
```kleis
operation dot : Vector(n) × Vector(n) → ℝ  // Cross-type
```

---

## Your Point is Valid!

**You're right:** Operations ARE conceptually part of types.

**The current approach (top-level)** is:
- ⚠️ Conceptually less pure
- ✅ Pragmatic for POC
- ✅ Works with primitive types
- ⚠️ Doesn't leverage structure system

**Better approach (future):**
```kleis
// Define abstract structure for numeric types
structure Numeric(N) {
    operation abs : N → N
    operation floor : N → ℤ
}

// ℝ implements it
implements Numeric(ℝ) {
    operation abs = builtin_abs
    operation floor = builtin_floor
}

// Now abs is polymorphic!
// Works for any Numeric type
```

---

## Question for You

Should we redesign stdlib/core.kleis to use structures instead of top-level operations?

**Option A: Keep top-level (current)**
```kleis
operation abs : ℝ → ℝ
```
- Pros: Simple, works now
- Cons: Not conceptually pure

**Option B: Use structures**
```kleis
structure Numeric(N) {
    operation abs : N → N
}
implements Numeric(ℝ)
```
- Pros: Conceptually cleaner, enables polymorphism
- Cons: More complex, need to implement `implements` parsing

**What's your preference?** This affects how we design the stdlib!

