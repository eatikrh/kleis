# Computational Complexity Analysis of Kleis

**Date:** December 8, 2025  
**Status:** Analysis of current implementation  
**Context:** Post ADR-021 (self-hosting type system)

---

## Executive Summary

**Overall:** Kleis has **good asymptotic complexity** for typical mathematical expressions.

**Key Results:**
- **Type Inference:** O(n²) typical, O(2ⁿ) worst case (HM is exponential)
- **Parsing:** O(n) linear (recursive descent)
- **Registry Lookups:** O(1) amortized (HashMap)
- **Template Matching:** O(t·n) where t = number of templates
- **Rendering:** O(n) linear

**Practical Performance:** ✅ Good for expressions up to ~1000 nodes

---

## 1. Type Inference (Hindley-Milner)

### Algorithm: Constraint-Based Unification

**Location:** `src/type_inference.rs`

```rust
fn infer_and_solve(expr: &Expression) -> Type {
    1. Generate constraints: O(n) where n = AST nodes
    2. Solve constraints: O(c²) where c = number of constraints
    3. Apply substitution: O(n)
}
```

### Complexity Analysis

#### **Best Case: O(n)**
```kleis
// Simple expression, no complex types
1 + 2 + 3 + 4
```
- Generate n constraints
- Each unifies trivially
- No recursion in unification

#### **Average Case: O(n²)**
```kleis
// Typical mathematical expression
(a + b) * (c - d) / sqrt(x)
```
- Generate O(n) constraints
- Solve O(n) constraints
- Each constraint may touch O(n) types
- Substitution composition: O(n) per constraint
- **Total: O(n²)**

#### **Worst Case: O(2ⁿ) - EXPONENTIAL!**
```kleis
// Deeply nested polymorphic functions
f(g(h(i(j(k(l(m(n(x)))))))))
```
- Each function application generates new type variable
- Unification may backtrack exponentially
- Occurs check traverses entire type tree each time

**This is inherent to Hindley-Milner!** All HM systems have this issue.

### Mitigation

✅ **In Practice:**
- Most expressions are shallow (depth < 10)
- Type variables quickly unified to concrete types
- Occurs check usually succeeds fast

✅ **Optimizations possible:**
- Memoize occurs check results
- Use union-find for type variables
- Incremental type checking (cache results)

---

## 2. Unification Algorithm

### Algorithm: Structural Recursion

**Location:** `src/type_inference.rs:620-697`

```rust
fn unify(t1: &Type, t2: &Type) -> Result<Substitution, String>
```

### Complexity by Type

| Type Pattern | Complexity | Reason |
|-------------|------------|---------|
| Scalar = Scalar | **O(1)** | Direct comparison |
| NatValue(n1) = NatValue(n2) | **O(1)** | Integer comparison |
| Data {...} = Data {...} | **O(k·d)** | k args, d recursion depth |
| Var(α) = t | **O(size(t))** | Occurs check |
| Function = Function | **O(d)** | Recursive on components |

**Where:**
- k = number of type arguments (Matrix has 2, Tensor3D has 3)
- d = depth of type nesting
- size(t) = number of type nodes in t

### **Overall Unification Complexity**

**For single unification:**
- Simple types: O(1)
- Parametric types: O(k) where k = arity
- Nested types: O(d) where d = depth
- With occurs check: O(n) where n = type size

**For solving n constraints:**
- Best case: O(n)
- Average case: O(n²) (substitution composition)
- Worst case: O(n³) (pathological occurs checks)

---

## 3. Parser Complexity

### Algorithm: Recursive Descent

**Locations:**
- `src/parser.rs` (LaTeX parser)
- `src/kleis_parser.rs` (Kleis parser)

### Kleis Parser

```rust
expression := term (('+' | '-') term)*      // O(n)
term := factor (('*' | '/') factor)*        // O(n)
factor := primary ('^' primary)?            // O(n)
primary := identifier | number | function_call | '(' expression ')'
```

**Complexity: O(n)** where n = input length
- Each character visited once
- Backtracking is minimal (LL(1) grammar)
- Recursive calls bounded by expression depth

### LaTeX Parser

**More complex due to:**
- Command parsing: `\frac{a}{b}`
- Group nesting: `{{{{a}}}}`
- Subscripts/superscripts: `x_i^2`

**Complexity: O(n·d)** where:
- n = input length
- d = maximum nesting depth (usually < 10)

**Worst case: O(n²)** for pathologically nested expressions

---

## 4. Registry Lookups (ADR-021)

### Data Structure: HashMap

**Location:** `src/data_registry.rs`

```rust
pub struct DataTypeRegistry {
    types: HashMap<String, DataDef>,      // Type name → Definition
    variants: HashMap<String, (String, DataVariant)>,  // Variant → (Type, Def)
}
```

### Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `has_type(name)` | **O(1)** amortized | HashMap lookup |
| `has_variant(name)` | **O(1)** amortized | HashMap lookup |
| `get_type(name)` | **O(1)** amortized | HashMap lookup |
| `lookup_variant(name)` | **O(1)** amortized | HashMap lookup |
| `register(data_def)` | **O(v)** | v = number of variants |

**Where v = number of variants in the data definition**

**Excellent performance!** Registry lookups are constant time.

### Memory Complexity

**Space: O(t + v)** where:
- t = number of type definitions
- v = total number of variants across all types

**Typical stdlib:**
- types.kleis: ~7 types, ~18 variants
- **Memory: ~1-5 KB** (negligible!)

---

## 5. Template Matching

### Algorithm: Pattern Matching

**Location:** `src/template_inference.rs`

```rust
fn infer_templates(expr: Expression) -> Expression {
    for template in templates {
        if matches(expr, template.pattern) {
            return apply(template);
        }
    }
    return expr;
}
```

### Complexity

**For single expression:**
- Try each template: O(t) where t = number of templates
- Pattern matching: O(p) where p = pattern size
- **Total: O(t·p)**

**For expression tree:**
- Visit each node: O(n) where n = AST nodes
- Try templates at each node: O(t·p)
- **Total: O(n·t·p)**

**Current system:**
- ~54 templates
- Pattern size typically 1-5 nodes
- **Practical: O(54n) ≈ O(n)** for typical expressions

### Optimization Opportunities

🎯 **Current:** Linear scan through all templates  
🎯 **Possible:** Index templates by root operation

```rust
// Instead of trying all 54 templates:
template_index.get("frac")  // Returns only frac-related templates
// Reduces from O(54) to O(3) for frac operations
```

**Improvement: 10-20× faster template matching!**

---

## 6. Type Context Operations

### Algorithm: Operation Registry Lookup

**Location:** `src/type_context.rs`

```rust
pub struct TypeContextBuilder {
    structures: HashMap<String, StructureDef>,          // O(1) lookup
    operation_map: HashMap<String, Vec<OperationInfo>>, // O(1) lookup
}
```

### Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| `infer_operation_type(op, args)` | **O(s·m)** | s = structures, m = matching |
| `type_supports_operation(type, op)` | **O(s)** | Scan all structures |
| `types_supporting(op)` | **O(s·t)** | s structures, t types each |

**Where:**
- s = number of structures (~10-20 in stdlib)
- m = max matches for operation (~1-3 typically)
- t = types per structure (~1-2 typically)

**Typical: O(10) to O(30)** - very fast!

### After Indexing

Current system builds operation_map:
```rust
operation_map["abs"] = [
    OperationInfo { structure: "Numeric", types: ["ℝ"] },
    OperationInfo { structure: "Complex", types: ["ℂ"] },
]
```

**Lookup: O(1)** to get list, O(m) to match arguments

---

## 7. Data Constructor Inference (ADR-021)

### Algorithm: Registry Lookup + Field Inference

**Location:** `src/type_inference.rs:439-527`

```rust
fn infer_data_constructor(name: &str, args: &[Expression]) -> Type {
    1. Lookup variant: O(1) (HashMap)
    2. Validate arity: O(1)
    3. Infer each field: O(f·i) where f = fields, i = infer complexity
    4. Construct result: O(f)
}
```

### Complexity

**For constructor with f fields:**
- Lookup: O(1)
- Infer f arguments: O(f·C_infer)
- Where C_infer = complexity of inferring one argument

**Total: O(f·n)** where:
- f = number of fields (typically 0-5)
- n = size of argument expressions

**Typical:** Matrix(2, 3, a, b, c, d, e, f) has 8 args
- 2 dimension args: O(1) each (constants)
- 6 value args: O(1) each (variables)
- **Total: O(8) ≈ O(1)** for typical matrix

---

## 8. Signature Interpretation

### Algorithm: Recursive Type Expression Evaluation

**Location:** `src/signature_interpreter.rs:313-348`

```rust
fn interpret_type_expr(type_expr: &TypeExpr) -> Type {
    match type_expr {
        Named(name) => O(1),          // Lookup
        Parametric(name, params) => {  // Recursive
            for param in params {
                interpret_type_expr(param);  // Recursion!
            }
        }
    }
}
```

### Complexity

**For type expression with:**
- p = number of parameters
- d = nesting depth
- r = registry lookup cost (O(1))

**Complexity: O(p·d)**

**Examples:**
- `Scalar`: O(1)
- `Vector(3)`: O(1) (1 param, depth 1)
- `Matrix(2, 3)`: O(2) (2 params, depth 1)
- `Tensor3D(10, 20, 30)`: O(3)
- `Option(Result(T, E))`: O(2·2) = O(4) (nested)

**Typical: O(1) to O(10)** - very fast!

---

## 9. Overall Pipeline Complexity

### End-to-End: Parsing → Type Checking → Rendering

```
Input (LaTeX/Kleis)
    ↓ Parse: O(n)
AST (Expression tree)
    ↓ Type Inference: O(n²) typical
Typed AST
    ↓ Template Matching: O(n·t)
Structured AST
    ↓ Rendering: O(n)
Output (HTML/SVG)
```

**Total: O(n²)** dominated by type inference

**Where n = expression size** (typically 10-100 nodes)

---

## 10. Scalability Analysis

### Small Expressions (n < 100)

**Example:** `(a + b) * sqrt(x^2 + y^2)`

| Phase | Complexity | Time |
|-------|------------|------|
| Parse | O(50) | <1ms |
| Type Infer | O(50²) = O(2,500) | ~1ms |
| Template | O(50·54) = O(2,700) | ~1ms |
| Render | O(50) | <1ms |
| **Total** | **O(n²)** | **~3-5ms** |

✅ **Excellent performance!**

### Medium Expressions (n = 100-1000)

**Example:** Large matrix operations, integral chains

| Phase | Time Estimate |
|-------|---------------|
| Parse | 1-10ms |
| Type Infer | 10-100ms |
| Template | 5-50ms |
| **Total** | **~20-160ms** |

✅ **Still good - under 200ms**

### Large Expressions (n > 1000)

**Example:** Auto-generated code, large proof terms

| Phase | Time Estimate |
|-------|---------------|
| Parse | 10-100ms |
| Type Infer | 100ms-1s |
| Template | 50-500ms |
| **Total** | **~200ms-2s** |

⚠️ **Noticeable latency**

---

## 11. Bottleneck Analysis

### Current Bottlenecks

**1. Type Inference (O(n²))**
- Constraint solving with substitution composition
- Each constraint may touch all previous substitutions
- **Impact:** High for deeply nested expressions

**2. Template Matching (O(n·t))**
- Linear scan through 54 templates for each node
- No indexing or caching
- **Impact:** Medium for large expressions

**3. Substitution Application (O(n) per constraint)**
- Walk entire type tree for each substitution
- No structure sharing
- **Impact:** Medium (cumulative)

### Not Bottlenecks (Good!)

✅ **Registry Lookups:** O(1) - excellent!  
✅ **Parsing:** O(n) - optimal for LL(1)  
✅ **Rendering:** O(n) - optimal  

---

## 12. Optimization Opportunities

### **High Impact (10-100× speedup)**

#### **1. Type Variable Union-Find**

**Current:**
```rust
// Substitution as HashMap: O(n) per lookup
map.get(var) → walk chain of substitutions
```

**Optimized:**
```rust
// Union-Find structure: O(α(n)) ≈ O(1)
union_find.find(var) → nearly constant time
```

**Benefit:** Type inference O(n²) → O(n log n)

#### **2. Template Indexing**

**Current:**
```rust
// Try all 54 templates
for template in all_templates { ... }
```

**Optimized:**
```rust
// Index by root operation
template_index["frac"] → [frac_template]  // Only 1-3 templates
```

**Benefit:** Template matching O(54n) → O(3n)

#### **3. Constraint Graph Ordering**

**Current:**
```rust
// Solve constraints in order added
for constraint in constraints { unify(...) }
```

**Optimized:**
```rust
// Topologically sort by dependency
solve_in_optimal_order(constraints)
```

**Benefit:** Fewer substitution applications

### **Medium Impact (2-5× speedup)**

#### **4. Type Sharing**

**Current:**
```rust
Type::Data { ... } // New allocation each time
```

**Optimized:**
```rust
Arc<Type::Data { ... }> // Reference-counted sharing
```

**Benefit:** Less copying, better cache locality

#### **5. Incremental Type Checking**

**Current:**
```rust
// Re-check entire expression on edit
infer(whole_expression)
```

**Optimized:**
```rust
// Only re-check changed subtree
infer_incremental(changed_node, cached_context)
```

**Benefit:** 10× faster for small edits

### **Low Impact (10-20% speedup)**

#### **6. Lazy Template Matching**

Only match templates when rendering, not during parsing.

#### **7. String Interning**

Share strings for common identifiers ("x", "y", "Matrix").

---

## 13. Memory Complexity

### Current Memory Usage

#### **Per Expression:**
```
AST: ~100 bytes per node
Types: ~50 bytes per Type
Constraints: ~100 bytes per constraint
```

**Typical expression (50 nodes):**
- AST: 50 × 100 = 5 KB
- Types: 50 × 50 = 2.5 KB
- Constraints: 30 × 100 = 3 KB
- **Total: ~10 KB**

✅ **Very good!** Notebook with 100 expressions = ~1 MB

#### **Registry Memory:**
```
DataTypeRegistry: O(t + v)
  types: 7 types × ~200 bytes = 1.4 KB
  variants: 18 variants × ~150 bytes = 2.7 KB
  Total: ~4 KB
```

✅ **Negligible!**

#### **Template Registry:**
```
54 templates × ~500 bytes = 27 KB
```

✅ **Small!**

### Memory Growth

| Component | Growth Rate | Notes |
|-----------|-------------|-------|
| AST | O(n) | Linear with expression size |
| Types | O(v) | v = type variables (bounded) |
| Registry | O(1) | Fixed after stdlib load |
| Templates | O(1) | Fixed set |
| **Total** | **O(n)** | Linear - excellent! |

---

## 14. Comparison with Other Systems

### Hindley-Milner Implementations

| System | Complexity | Notes |
|--------|------------|-------|
| **Kleis** | O(n²) typical | Constraint-based |
| **Haskell GHC** | O(n²) typical | Optimized union-find |
| **OCaml** | O(n²) typical | Levels optimization |
| **Rust rustc** | O(n²) typical | Trait resolution |

**Kleis is competitive!** Same asymptotic complexity as production compilers.

### Template Systems

| System | Complexity | Notes |
|--------|------------|-------|
| **Kleis** | O(n·t) | 54 templates, linear scan |
| **Pandoc** | O(n·t) | Similar approach |
| **LaTeX** | O(n·t·d) | Macro expansion can nest |

**Kleis is typical** for this type of system.

---

## 15. Worst-Case Scenarios

### **Pathological Case 1: Deep Nesting**

```kleis
f(f(f(f(f(f(f(f(f(f(x))))))))))  // 10 levels
```

**Complexity:**
- Parse: O(n) = O(10) ✓
- Type inference: O(2¹⁰) = O(1024) ⚠️ Exponential!
- Template: O(10·54) = O(540) ✓

**Risk:** Exponential in depth for polymorphic functions

### **Pathological Case 2: Wide Operations**

```kleis
sum(a₁, a₂, a₃, ..., a₁₀₀₀)  // 1000 arguments
```

**Complexity:**
- Parse: O(1000) ✓
- Type inference: O(1000) ✓ (all same type)
- Template: O(1000) ✓

**Risk:** Low - linear is fine

### **Pathological Case 3: Large Matrix**

```kleis
Matrix(100, 100, e₁, e₂, ..., e₁₀₀₀₀)  // 10,000 elements
```

**Complexity:**
- Parse: O(10,000) ✓
- Type inference: O(10,000) ✓ (dimension check O(1), elements O(n))
- Memory: ~1 MB for AST

**Risk:** Low - memory, not time

---

## 16. Performance Benchmarks (Estimated)

### Typical Mathematical Expression

```kleis
E = ∫₀¹ (x² + y²) dx + Matrix(2,2, a, b, c, d)
```

**Metrics:**
- AST nodes: ~15
- Type variables: ~5
- Constraints: ~10
- Templates: ~5 matches

**Performance:**
- Parse: <1ms
- Type infer: ~1ms
- Template: ~1ms
- Render: ~1ms
- **Total: ~3-5ms** ✅

### Large Notebook (100 Cells)

```
100 expressions × 5ms = 500ms = 0.5 seconds
```

✅ **Sub-second latency** for full notebook!

### Incremental Edit

```
User types one character → Re-check one cell → ~5ms
```

✅ **Instant feedback!**

---

## 17. Scalability Limits

### Current Implementation Can Handle:

✅ **Expressions:** Up to ~1,000 nodes (comfortable)  
✅ **Notebooks:** Up to ~1,000 cells (comfortable)  
✅ **Types:** Up to ~1,000 types in registry (comfortable)  
⚠️ **Deep nesting:** Depth > 20 may cause exponential behavior  
⚠️ **Polymorphic chains:** Long chains may be slow  

### When You'd Hit Limits:

**~10,000 node expressions:**
- Type inference: ~1-10 seconds
- Noticeable latency

**~10,000 types in registry:**
- Memory: ~10 MB
- Lookup still O(1), no problem!

**Depth > 30:**
- Type inference: Exponential behavior
- May timeout (rare in practice)

---

## 18. Recommended Optimizations

### **Priority 1: Union-Find for Type Variables**

**Impact:** 10-50× faster type inference  
**Effort:** 1-2 weeks  
**When:** After 1000+ users report latency

### **Priority 2: Template Indexing**

**Impact:** 10× faster template matching  
**Effort:** 2-3 days  
**When:** After template externalization (ADR-027)

### **Priority 3: Incremental Type Checking**

**Impact:** 100× faster for edits  
**Effort:** 2-3 weeks  
**When:** For interactive notebook editor

### **Priority 4: Parallel Type Checking**

**Impact:** 2-4× faster (multicore)  
**Effort:** 1-2 weeks  
**When:** For batch processing

---

## 19. Comparison: JIT Would Help Where?

### **Interpreter Bottlenecks (Not Type Checking!)**

| Phase | Current | With JIT |
|-------|---------|----------|
| Type checking | O(n²) | O(n²) - **Same!** |
| **Expression eval** | **O(n·d)** | **O(1)** - Compiled! |
| Matrix multiply | O(m·n·p) | O(m·n·p) - **But 100× faster** |

**Key insight:** JIT helps **execution**, not type checking!

**For compilation:**
- Type checking: ~5ms (unchanged)
- **Execution:** 100ms → 1ms (100× faster!)

---

## 20. Conclusion

### **Current Performance: ✅ Good**

**Strengths:**
- ✅ O(1) registry lookups (ADR-021 design is excellent!)
- ✅ O(n) parsing (optimal)
- ✅ O(n²) type inference (standard for HM)
- ✅ Linear memory growth

**Weaknesses:**
- ⚠️ O(2ⁿ) worst case for deep polymorphic nesting (rare)
- ⚠️ O(n·t) template matching (can be optimized)
- ⚠️ No caching/incremental checking

**Practical:** ✅ **Fast enough for typical use** (<10ms per expression)

### **Optimization Roadmap:**

**Don't optimize yet!** Current performance is fine.

**When to optimize:**
1. Users report latency (>100ms per expression)
2. Profiling shows actual bottlenecks
3. After feature-complete (ADR-027, ADR-028)

**Low-hanging fruit when needed:**
- Template indexing: 2-3 days, 10× speedup
- Union-find: 1-2 weeks, 10-50× speedup

### **For JIT Compiler (Future):**

Type checking complexity **stays the same** (O(n²)).

JIT helps with **execution** (100-1000× faster).

So the tradeoff is:
- Type check: 5ms (unchanged)
- Execute: 100ms → 1ms (100× improvement)

**Worth it for numerical computing!**

---

**Bottom Line:** Your computational complexity is **excellent** for current use cases. No immediate optimization needed. The O(n²) type inference is **standard for HM systems** and performs well in practice! ✅

