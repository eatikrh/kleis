# Matrix Dimensions as Dimensional Analysis

**Date:** December 7, 2024  
**Insight:** Matrix type checking is dimensional analysis from physics  
**Impact:** Opens path to compile-time unit verification

---

## The Core Insight

**What we're doing with matrix dimensions is exactly dimensional analysis from physics!**

When we check:
```kleis
Matrix(2, 3) + Matrix(3, 2)  → ❌ ERROR: dimensions must match
Matrix(2, 3) × Matrix(3, 4)  → ✓ Matrix(2, 4)
```

This is **identical** to physics dimensional analysis:
```
Length + Time  → ❌ ERROR: incompatible dimensions
Force × Length → Energy (dimensions compose)
```

---

## The Parallel

### **Physics: Dimensional Analysis**

Physical quantities have dimensions that compose:

| Quantity | Dimension | Example |
|----------|-----------|---------|
| Distance | [L] | 10 meters |
| Time | [T] | 5 seconds |
| Velocity | [L]/[T] | 2 m/s |
| Acceleration | [L]/[T²] | 9.8 m/s² |
| Force | [M][L]/[T²] | 50 Newtons |
| Energy | [M][L²]/[T²] | 100 Joules |

**Rules:**
1. Can only add/subtract same dimensions: [L] + [L] ✓, [L] + [T] ✗
2. Multiplication composes: [L]/[T] × [T] = [L]
3. Powers scale: [L²] = [L] × [L]

---

### **Kleis: Matrix Type Checking**

Matrix types have dimensions that compose:

| Expression | Type | Result |
|------------|------|--------|
| A | Matrix(2, 3) | 2×3 matrix |
| B | Matrix(3, 4) | 3×4 matrix |
| A + A | Matrix(2, 3) | ✓ Same dimensions |
| A + B | ❌ | ✗ Incompatible (2,3) ≠ (3,4) |
| A × B | Matrix(2, 4) | ✓ Inner dimension 3 cancels |
| transpose(A) | Matrix(3, 2) | Dimensions swap |

**Rules:**
1. Can only add/subtract same dimensions: Matrix(m,n) + Matrix(m,n) ✓
2. Multiplication requires inner match: Matrix(m,n) × Matrix(n,p) → Matrix(m,p)
3. Operations transform dimensions: transpose swaps them

---

## The Deep Connection

### **Both Are Compile-Time Safety**

**Physics:**
```c
// Compiler can't catch this:
double velocity = distance + time;  // ← Nonsense! But compiles
```

**With Units:**
```cpp
// Type system catches it:
Quantity<Length> distance = 10_m;
Quantity<Time> time = 5_s;
auto wrong = distance + time;  // ← Compile error! Can't add Length + Time
```

**Kleis:**
```kleis
let A = Matrix(2, 3, ...)
let B = Matrix(3, 2, ...)
A + B  // ← Type error! Can't add Matrix(2,3) + Matrix(3,2)
```

**Both prevent nonsensical operations at compile time!**

---

## This Opens Powerful Possibilities

### **1. Physical Unit Types**

```kleis
structure Quantity(value: ℝ, dimension: Dimension) {
  // Addition: Same dimensions only
  operation plus : Quantity(a, D) → Quantity(b, D) → Quantity(a+b, D)
  
  // Multiplication: Dimensions compose
  operation times : Quantity(a, D1) → Quantity(b, D2) → Quantity(a×b, D1×D2)
  
  // Division: Dimensions divide
  operation divide : Quantity(a, D1) → Quantity(b, D2) → Quantity(a/b, D1/D2)
}

// Define base dimensions
type Length = Dimension(L=1, M=0, T=0)
type Time = Dimension(L=0, M=0, T=1)
type Velocity = Dimension(L=1, M=0, T=-1)  // L/T

// Type-safe physics!
let distance = Quantity(100, Length)      // 100 meters
let time = Quantity(10, Time)             // 10 seconds
let speed = distance / time               // Type: Quantity(10, Velocity) ✓

// Prevent errors:
distance + time  // ❌ Type error: Length ≠ Time
```

---

### **2. Matrix Dimensions ARE Units**

```kleis
// Matrix dimensions are like physical dimensions!
structure Matrix(m: Nat, n: Nat, T) {
  // Addition: "Dimensions" must match
  operation plus : Matrix(m, n, T) → Matrix(m, n, T) → Matrix(m, n, T)
  
  // Multiplication: "Dimensions" compose (inner cancels)
  operation times : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
  //                               ↑ These must match (like units canceling)
}

// Example: Transform compositions
let A = Matrix(2, 3)  // Maps ℝ³ → ℝ²
let B = Matrix(3, 4)  // Maps ℝ⁴ → ℝ³
let C = A × B         // Type: Matrix(2, 4) - Maps ℝ⁴ → ℝ² ✓

// Inner dimension 3 "cancels" like units!
// ℝ³ in B's codomain matches ℝ³ in A's domain
```

---

### **3. Tensor Index Analysis**

```kleis
// Einstein notation with type-checked indices!
structure Tensor(indices: IndexPattern) {
  // Contraction: repeated indices "cancel" (like dimensions)
  operation contract : Tensor([↑μ, ↓μ]) → Scalar
  
  // Product: Indices must be distinct
  operation tensor : Tensor([↑μ]) → Tensor([↑ν]) → Tensor([↑μ, ↑ν])
}

// Type system verifies index patterns!
let g = Tensor([↓μ, ↓ν])      // Metric tensor
let v = Tensor([↑μ])           // Vector
let lower = g × v              // Type: Tensor([↓μ, ↓ν, ↑μ])
let contracted = contract(lower)  // ❌ Type error: ν unmatched!
```

---

## Why This Matters

### **Dimensional Analysis Prevents Bugs**

**Famous example: Mars Climate Orbiter (1999)**
- Cost: $327 million
- Lost because: One team used imperial units, another used metric
- If they had compile-time dimensional analysis: **Would have caught the error!**

### **Our Type System Does This**

```kleis
// Catch matrix dimension errors at "compile" time:
let A = Matrix(2, 3, ...)
let B = Matrix(4, 5, ...)
let C = A × B  // ❌ Type error: inner dimensions don't match (3 ≠ 4)

// Instead of runtime crash:
// RuntimeError: shapes (2,3) and (4,5) not aligned: 3 (dim 1) != 4 (dim 0)
```

**Same benefit: Catch errors before runtime!**

---

## Connection to Dependent Types

Dimensional analysis requires tracking **value-level information in types**:

**Physics:**
```
Quantity(10, Length) ← The dimension (Length) is in the TYPE
```

**Matrices:**
```
Matrix(2, 3, ℝ) ← The dimensions (2, 3) are in the TYPE
```

**This is why matrices need dependent types!**

Dimensions are VALUES (2, 3, n, m) but appear in TYPES (Matrix(2, 3)).

---

## Existing Research

This connection has been explored in programming languages:

### **F# Units of Measure**
```fsharp
[<Measure>] type m   // meters
[<Measure>] type s   // seconds
[<Measure>] type kg  // kilograms

let distance = 100.0<m>
let time = 10.0<s>
let speed = distance / time  // Type: float<m/s> ✓

distance + time  // Compile error: Cannot add m + s
```

### **Rust's `uom` Crate**
```rust
let distance = Length::new::<meter>(100.0);
let time = Time::new::<second>(10.0);
let velocity = distance / time;  // Type: Velocity

distance + time  // Compile error: mismatched types
```

### **Haskell's Dimensional Library**
```haskell
distance :: Length
time :: Time
velocity = distance / time  -- Type: Velocity

distance + time  -- Type error!
```

---

## What Kleis Can Do Uniquely

### **Kleis Advantage: User-Defined Dimensions**

**Physics libraries:** Dimensions are hardcoded (Length, Time, Mass)

**Kleis:** Users can define ANY dimensional structure!

```kleis
// User defines their own "dimensional" structure:
structure Quantity(D: Dimension) {
  operation plus : Quantity(D) → Quantity(D) → Quantity(D)
  operation times : Quantity(D1) → Quantity(D2) → Quantity(D1 × D2)
}

// Or matrices:
structure Matrix(m: Nat, n: Nat) {
  operation times : Matrix(m, n) → Matrix(n, p) → Matrix(m, p)
}

// Or tensors:
structure Tensor(indices: IndexPattern) {
  // User-defined index patterns!
}

// Or database columns:
structure Table(columns: List(ColumnName)) {
  operation join : Table(cols1) → Table(cols2) → Table(cols1 ∪ cols2)
}
```

**Kleis is a META-dimensional-analysis system!**

Users can define their own "dimensions" for ANY domain:
- Physics: Length, Time, Mass
- Matrices: Rows, Cols
- Tensors: Index patterns
- Databases: Column schemas
- Finance: Currency types
- Chemistry: Molar quantities
- etc.

---

## The Research Connection

### **Dependent Types + Dimensions**

This is active research in type theory:

1. **Dependent Types** (Idris, Agda, Coq)
   - Types that depend on values
   - `Vector(n)` where n is a value
   - `Matrix(m, n)` where m, n are values

2. **Dimensional Types** (F#, Rust, Haskell)
   - Track physical units
   - Prevent dimensional errors
   - Compile-time safety

3. **Kleis combines both:**
   - Dependent types for matrices: `Matrix(m, n)`
   - Dimensional checking: Inner dimensions must match
   - User-extensible: Define your own "dimensions"

---

## Future Vision

### **Physics in Kleis:**

```kleis
// Define physical dimensions
structure Physical(L: ℤ, M: ℤ, T: ℤ) {
  operation times : Physical(L1, M1, T1) → Physical(L2, M2, T2) 
                  → Physical(L1+L2, M1+M2, T1+T2)
}

// Define base units
type Length = Physical(1, 0, 0)
type Mass = Physical(0, 1, 0)
type Time = Physical(0, 0, 1)
type Velocity = Physical(1, 0, -1)   // L·T⁻¹
type Force = Physical(1, 1, -2)      // M·L·T⁻²
type Energy = Physical(2, 1, -2)     // M·L²·T⁻²

// Use it:
let d = Quantity(100, Length)        // 100 m
let t = Quantity(10, Time)           // 10 s
let v = d / t                        // Type: Quantity(10, Velocity) ✓

let F = Quantity(50, Force)          // 50 N
let W = F * d                        // Type: Quantity(5000, Energy) ✓ (Force·Length = Energy)

// Prevent nonsense:
d + t  // ❌ Type error: Physical(1,0,0) ≠ Physical(0,0,1)
```

---

## This Should Be an ADR!

This is such a fundamental insight that it deserves its own Architecture Decision Record. Should I create:

**ADR-019: Dimensional Analysis Type System**

This would document:
1. Matrix dimensions as compile-time dimensional analysis
2. Connection to physics dimensional analysis
3. Path to general dimensional/unit types
4. User-defined dimensions for any domain

This could be a **major selling point** for Kleis:

> **"Kleis: The first language with user-extensible dimensional analysis"**

Want me to write this ADR? 🎯
