# ADR-019: Dimensional Type Checking

**Date:** December 7, 2024  
**Status:** Accepted  
**Context:** Type system implementation, matrix dimension checking  
**Related:** ADR-014 (Hindley-Milner), ADR-016 (Operations in Structures)

---

## Summary

Matrix dimension checking in Kleis is **dimensional analysis** from physics, applied to type checking. This insight opens a path to general compile-time dimensional/unit verification for any domain.

---

## Context

### **What We Built**

During type system implementation, we realized our matrix type checking:

```kleis
Matrix(2, 3) × Matrix(3, 4) → Matrix(2, 4) ✓
Matrix(2, 3) × Matrix(4, 5) → ❌ Type error: inner dimensions don't match
```

Is **exactly** the same as dimensional analysis in physics:

```
Force × Distance → Energy ✓
Force + Distance → ❌ Dimensional error: incompatible dimensions
```

---

## The Parallel

| Physics Dimensional Analysis | Kleis Matrix Type Checking |
|------------------------------|----------------------------|
| Physical quantities have dimensions | Matrices have dimensions (m, n) |
| [Length], [Time], [Mass] | Matrix(m, n), Vector(n) |
| Can't add Length + Time | Can't add Matrix(2,3) + Matrix(3,2) |
| Velocity = Length/Time | Matrix(m,p) = Matrix(m,n) × Matrix(n,p) |
| Dimensions compose/cancel | Dimensions match/transform |
| Catches errors at compile time | Catches errors at type check time |

---

## Decision

**We recognize matrix dimension checking as dimensional analysis and design the type system to support general dimensional checking.**

### **Immediate (Already Implemented):**

1. **Matrix dimensions are type-level**
   ```kleis
   type Matrix2x3 = Matrix(2, 3, ℝ)
   type Matrix3x4 = Matrix(3, 4, ℝ)
   ```

2. **Operations check dimensional compatibility**
   ```kleis
   structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
     operation multiply : Matrix(m, p, T)
   }
   // Inner dimension n must match on both inputs
   ```

3. **Type errors are dimension errors**
   ```
   ❌ Matrix multiplication: inner dimensions must match!
      Left: 2×3
      Right: 4×5
      Cannot multiply: 3 ≠ 4
   ```

---

### **Future (Generalization):**

**Extend to arbitrary dimensional structures:**

```kleis
// Define dimensional structure
structure Dimensional(dimensions: DimVector) {
  operation times : Dimensional(d1) → Dimensional(d2) → Dimensional(d1 + d2)
  operation divide : Dimensional(d1) → Dimensional(d2) → Dimensional(d1 - d2)
  operation plus : Dimensional(d) → Dimensional(d) → Dimensional(d)
}

// Physics dimensions (L, M, T exponents)
type Length = Dimensional([1, 0, 0])
type Mass = Dimensional([0, 1, 0])
type Time = Dimensional([0, 0, 1])
type Velocity = Dimensional([1, 0, -1])   // L¹·T⁻¹
type Force = Dimensional([1, 1, -2])      // M¹·L¹·T⁻²
type Energy = Dimensional([2, 1, -2])     // M¹·L²·T⁻²

// Use it:
implements Dimensional(Length) {
  element meter : Quantity(1, Length)
  element kilometer : Quantity(1000, Length)
}

implements Dimensional(Force) {
  element newton : Quantity(1, Force)
}

// Type-safe physics:
let F = Quantity(50, Force)           // 50 N
let d = Quantity(10, Length)          // 10 m
let W = F * d                         // Type: Quantity(500, Energy) ✓

// Compiler prevents nonsense:
F + d  // ❌ Type error: Force([1,1,-2]) ≠ Length([1,0,0])
```

---

## Consequences

### **Positive**

1. **Compile-Time Safety**
   - Catch dimensional errors before runtime
   - No more "unit mismatch" bugs
   - Mars Climate Orbiter wouldn't have crashed 🚀

2. **Self-Documenting Code**
   ```kleis
   operation kinetic_energy : Mass → Velocity → Energy
   // Type signature shows dimensional relationship!
   ```

3. **User-Extensible**
   - Not just physics - ANY domain with "dimensions"
   - Matrices: row/col dimensions
   - Databases: column schemas
   - Finance: currency types
   - Chemistry: molar quantities
   - Biology: concentration units

4. **Unique Feature**
   - F#, Rust, Haskell have dimensional analysis
   - **But only for hardcoded dimensions**
   - Kleis: **User-defined dimensions** for any domain

---

### **Challenges**

1. **Parser Support Needed**
   - Need to parse dimension parameters properly
   - Need to handle dimension expressions (m+n, 2×m)
   - Requires Phase 2 parser extension

2. **Unification Gets Complex**
   - Dimension variables: Matrix(m, n) where m, n are unknown
   - Dimension constraints: m = n (square matrices)
   - Dimension arithmetic: Matrix(m, n) × Matrix(n, p) → Matrix(m, p)

3. **Performance**
   - Dimension checking adds overhead
   - Must be efficient for large programs
   - May need caching/memoization

4. **Error Messages**
   - Must be clear about dimensional mismatches
   - Show which dimensions don't match
   - Suggest fixes when possible

---

## Implementation Status

### **✅ Already Working**

- **Matrix dimensions checked** - Inner dimensions must match for multiplication
- **Addition requires same dimensions** - Can't add 2×3 + 3×2
- **Type errors show dimensions** - "Cannot multiply: 3 ≠ 4"
- **Transpose swaps dimensions** - Matrix(m, n) → Matrix(n, m)

### **⚠️ Limitations**

- **Dimension extraction is hacky** - Defaults to 2×2 for non-constants
- **No dimension variables** - Can't track Matrix(n, n) where n is unknown
- **Only matrices** - Not generalized to other dimensional structures

### **🔮 Future**

- **Dimension expressions** - Track Matrix(n, m+1) properly
- **General dimensional structures** - Physics units, tensor indices, etc.
- **Dimension inference** - Infer dimensions from operations
- **Dimension arithmetic** - Symbolic dimension manipulation

---

## Examples

### **What Works Now**

```kleis
// Concrete dimensions:
let A = Matrix(2, 3, 1, 2, 3, 4, 5, 6)  // Type: Matrix(2, 3)
let B = Matrix(3, 4, ...)                // Type: Matrix(3, 4)
let C = A × B                            // Type: Matrix(2, 4) ✓

// Dimension mismatch:
A × A  // ❌ Type error: 3 ≠ 2 (inner dimensions don't match)
```

### **What Will Work (Phase 2)**

```kleis
// Variable dimensions:
let n = 3
let A = Matrix(n, n, ...)                // Type: Matrix(n, n)
let B = transpose(A)                     // Type: Matrix(n, n)
let C = A × B                            // Type: Matrix(n, n) ✓
```

### **What Will Work (Phase 3)**

```kleis
// General dimensional analysis:
let force = Quantity(50, Newton)
let distance = Quantity(10, Meter)
let work = force * distance              // Type: Quantity(500, Joule) ✓

force + distance  // ❌ Type error: Newton ≠ Meter
```

---

## Research Connections

### **Type Theory**

- **Dependent Types:** Types depending on values (Idris, Agda, Coq)
- **Refinement Types:** Types with predicates (LiquidHaskell)
- **Dimension Types:** Types tracking physical dimensions (F#)

**Kleis combines all three!**

### **Physics**

- **Dimensional Analysis:** Buckingham π theorem
- **Unit Systems:** SI units, natural units, Planck units
- **Tensor Calculus:** Einstein notation, index gymnastics

**Kleis can formalize all of these!**

### **Programming Languages**

- **F# Units of Measure:** Hardcoded dimensional analysis
- **Rust uom:** Type-safe units library
- **Haskell dimensional:** Dimensional analysis library
- **Ada:** Range types (similar concept)

**Kleis is more general:** User-defined dimensions for ANY domain!

---

## Use Cases Beyond Physics

### **1. Financial Types**

```kleis
structure Money(currency: Currency) {
  operation plus : Money(C) → Money(C) → Money(C)
  operation convert : Money(C1) → Rate(C1, C2) → Money(C2)
}

let usd = Money(100, USD)
let eur = Money(80, EUR)
usd + eur  // ❌ Type error: Cannot add USD + EUR
```

### **2. Database Schemas**

```kleis
structure Table(schema: Schema) {
  operation join : Table(s1) → Table(s2) → Table(merge(s1, s2))
  operation select : Table(s) → Columns(c) → Table(project(s, c))
}

// Type system verifies schema compatibility!
```

### **3. Graph Types**

```kleis
structure Graph(nodes: Nat, edges: Nat) {
  operation add_edge : Graph(n, e) → Graph(n, e+1)
  operation add_node : Graph(n, e) → Graph(n+1, e)
}
```

### **4. Tensor Dimensions**

```kleis
structure Tensor(shape: List(Nat)) {
  operation reshape : Tensor(s1) → Tensor(s2) where product(s1) = product(s2)
  operation contract : Tensor([..., n, n, ...]) → Tensor([..., ...])
}
```

---

## Vision

**Kleis as the Universal Dimensional Type System:**

> Any domain with "dimensions" (physics, matrices, tensors, units, currencies, schemas) can be expressed in Kleis with compile-time dimensional checking.

**The type system becomes a meta-framework for dimensional reasoning.**

---

## References

### **Academic**

- Buckingham π theorem (dimensional analysis)
- Kennedy, A. "Types for Units-of-Measure" (F# inspiration)
- McBride, C. "Faking It: Simulating Dependent Types" (Haskell approach)

### **Implementation**

- F# Units of Measure: https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/units-of-measure
- Rust uom: https://github.com/iliekturtles/uom
- Haskell dimensional: https://hackage.haskell.org/package/dimensional

### **Kleis Docs**

- `src/type_inference.rs:238-290` - Matrix constructor (current hack)
- `docs/session-2024-12-07/MATRIX_CONSTRUCTOR_ISSUE.md` - Technical debt
- `docs/theory/DIMENSIONAL_ANALYSIS_ANALOGY.md` - This insight

---

## Next Steps

1. **Phase 1:** Document this insight (done ✓)
2. **Phase 2:** Implement dimension expressions for matrices
3. **Phase 3:** Generalize to arbitrary dimensional structures
4. **Future:** Paper on "User-Extensible Dimensional Type Systems"

---

## Conclusion

**What started as "matrix type checking" is actually a general framework for dimensional reasoning.**

This positions Kleis as:
- ✅ More than a symbolic math system
- ✅ A general dimensional type system
- ✅ Applicable to physics, engineering, finance, data science, etc.
- ✅ User-extensible (not hardcoded dimensions)

**This is a major differentiator and research contribution.** 🎯

---

**Status:** Accepted  
**Implementation:** Partial (matrices only)  
**Full implementation:** Phase 2-3  
**Research potential:** HIGH

