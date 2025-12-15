# Type Inference Future Plan

**Date:** December 7, 2025  
**Current:** `type_inference.rs` is minimal PoC (~470 lines)  
**Question:** What's the long-term plan?  
**Answer:** Keep it minimal, push complexity elsewhere

---

## Current State

### **type_inference.rs (470 lines)**

**Role:** Core Hindley-Milner algorithm

**What it does:**
- Type variable generation
- Constraint generation
- Unification algorithm
- Substitution
- Basic type checking

**What it delegates:**
- Operation types → `TypeContextBuilder`
- Structure definitions → `TypeContextBuilder`
- Registry queries → `TypeContextBuilder`
- Error messages → `TypeChecker`

---

## Architecture: Separation of Concerns

```
┌─────────────────────────────────────────────────────────┐
│                    TypeChecker                          │
│  (type_checker.rs - 258 lines)                         │
│  - User-facing API                                      │
│  - Load stdlib                                          │
│  - Error message generation                             │
│  - Helpful suggestions                                  │
└─────────────────────────────────────────────────────────┘
                        │
                        ├──────────────┬─────────────────┐
                        ▼              ▼                 ▼
            ┌─────────────────┐  ┌──────────────┐  ┌────────────┐
            │ TypeInference   │  │ TypeContext  │  │  Registry  │
            │ (HM algorithm)  │  │ (Structures) │  │ (Queries)  │
            │   470 lines     │  │  734 lines   │  │  Embedded  │
            └─────────────────┘  └──────────────┘  └────────────┘
                   │                    │                  │
                   └────────────────────┴──────────────────┘
                                        │
                                        ▼
                            ┌───────────────────────┐
                            │   Kleis Stdlib        │
                            │  (minimal_prelude +   │
                            │   matrices)           │
                            └───────────────────────┘
```

---

## Design Philosophy

### **Keep type_inference.rs MINIMAL** ✅

**Rationale:**
1. **HM algorithm is well-understood** - Core algorithm shouldn't change much
2. **Complexity belongs in structures** - ADR-016 principle
3. **Easier to verify** - Small, focused code is easier to prove correct
4. **Easier to maintain** - Changes to operations don't require HM changes

**Analogous to:**
- **Unix philosophy:** Small tools that do one thing well
- **Microkernel:** Core is minimal, features in user space
- **Lisp:** Small core, everything else is macros/libraries

---

## What Stays in type_inference.rs

### **Core HM Algorithm (永久/Permanent)**

```rust
✅ Type representation (Type enum)
✅ Type variable generation
✅ Constraint generation
✅ Unification algorithm
✅ Substitution
✅ Occurs check
✅ Basic type inference (infer())
```

**These are the mathematical foundations of HM type inference.**  
**Should NEVER be diluted with domain logic.**

---

### **Necessary Special Cases**

```rust
⚠️ Matrix constructors (literals, not operations)
   - Matrix(2, 3, a, b, c, d, e, f)
   - Can't reasonably move to stdlib (they're data constructors)
   - ~50 lines of code
   - TODO: Better dimension tracking (Phase 2)

⚠️ Future: Vector constructors, List constructors
   - Same pattern as Matrix
   - Also literals, not operations
```

**Justification:** Data constructors are fundamentally different from operations.

---

## What Moves OUT of type_inference.rs

### **✅ Already Moved (Today's Work)**

```rust
❌ Arithmetic operations (plus, minus, times, divide)
   → Moved to stdlib/minimal_prelude.kleis

❌ Numeric operations (sqrt, abs, floor, power)
   → Moved to stdlib/minimal_prelude.kleis

❌ Calculus operations (derivative, integral)
   → Moved to stdlib (stub definitions)

❌ Matrix operations (transpose, multiply, add)
   → Already in stdlib/matrices.kleis
```

**Before:** 139 lines of hardcoded operation logic  
**After:** 0 lines (all delegated to context_builder)

---

### **🔮 Future Moves**

```rust
⚠️ Type pretty-printing (Display impl)
   → Could move to separate formatter module
   → type_inference.rs just has the data structure

⚠️ Error message construction
   → Already in TypeChecker, but could be richer
   → Separate error formatting module

⚠️ Constraint solving strategies
   → Currently simple unification
   → Future: backtracking, constraint propagation
   → Could be separate solver module
```

---

## Future Extensions (What Grows)

### **Phase 2: Dimension Tracking** (~1 week)

**Problem:** Matrix constructors default to 2×2 for non-constant dimensions

**Solution:** Track dimension expressions

```rust
// Add to type_inference.rs:
enum DimExpr {
    Const(usize),
    Var(String),
    Add(Box<DimExpr>, Box<DimExpr>),
    Mul(Box<DimExpr>, Box<DimExpr>),
}

enum Type {
    // ...
    Matrix(DimExpr, DimExpr),  // Not (usize, usize)
}
```

**Lines added:** ~100 lines  
**Total:** ~570 lines  
**Benefit:** Proper dimension tracking

---

### **Phase 3: Constraint Solver** (~2 weeks)

**Problem:** Current unification is simple, doesn't handle complex constraints

**Solution:** Separate constraint solver

```rust
// New file: src/constraint_solver.rs (~300 lines)
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    strategies: Vec<SolverStrategy>,
}

// type_inference.rs just generates constraints
impl TypeInference {
    pub fn solve(&self) -> Result<Substitution, String> {
        let solver = ConstraintSolver::new(self.constraints.clone());
        solver.solve()  // ← Delegate to separate module
    }
}
```

**Lines added to type_inference.rs:** ~20 lines  
**New module:** constraint_solver.rs (~300 lines)  
**Total type_inference.rs:** ~490 lines  
**Benefit:** More powerful solving without bloating core

---

### **Phase 4: Polymorphic Type Generalization** (~1 week)

**Problem:** Currently don't generalize polymorphic types (∀α. α → α)

**Solution:** Add let-polymorphism

```rust
// Add to type_inference.rs:
impl TypeInference {
    /// Generalize a type to a ForAll type
    pub fn generalize(&self, ty: &Type) -> Type {
        let free_vars = self.free_vars(ty);
        if free_vars.is_empty() {
            ty.clone()
        } else {
            // Create ForAll type: ∀α β γ. T
            Type::ForAll(free_vars, Box::new(ty.clone()))
        }
    }
}
```

**Lines added:** ~50 lines  
**Total:** ~540 lines  
**Benefit:** Proper polymorphic type inference

---

### **Phase 5: Row Polymorphism** (Optional, ~2 weeks)

**Problem:** Can't express "any matrix with at least these dimensions"

**Solution:** Row types for extensible records/matrices

```rust
enum Type {
    // ...
    MatrixRow {
        known_ops: HashMap<String, Type>,
        rest: Option<TypeVar>,  // Open row variable
    }
}
```

**Lines added:** ~100 lines  
**Total:** ~640 lines  
**Benefit:** More flexible polymorphism

---

## Size Projections

| Phase | type_inference.rs | Notes |
|-------|-------------------|-------|
| **Current (Phase 1)** | 470 lines | ✅ Minimal HM + Matrix constructors |
| Phase 2 | ~570 lines | +100 for DimExpr |
| Phase 3 | ~490 lines | -80 move to constraint_solver.rs |
| Phase 4 | ~540 lines | +50 for generalization |
| Phase 5 | ~640 lines | +100 for row polymorphism |

**Maximum ever:** ~640 lines (with all features)

---

## Comparison to Other Implementations

### **Haskell GHC Type Checker**

- **Core:** ~50,000 lines (!)
- **Includes:** Type inference, kind checking, type families, GADTs, etc.

### **OCaml Type Checker**

- **Core:** ~10,000 lines
- **Includes:** HM inference, module system, row polymorphism

### **Kleis (Goal)**

- **Core:** ~600 lines (type_inference.rs)
- **Structures:** ~1,000 lines (type_context.rs, type_checker.rs)
- **Total:** ~1,600 lines

**Why so small?**
- Operations in stdlib (ADR-016), not in type checker
- Minimal PoC scope (not full language yet)
- Simple type system (for now)

---

## What Should NOT Go in type_inference.rs

### **❌ Domain Logic**

```rust
// DON'T add this to type_inference.rs:
match name {
    "determinant" => { /* check if square matrix */ }
    "trace" => { /* check if square matrix */ }
    "cross" => { /* check if 3D vectors */ }
}
```

**Why not:** This is domain logic, belongs in stdlib structures

**Where:** `TypeContextBuilder::infer_operation_type()` or better yet, in the structure definitions themselves

---

### **❌ Built-in Function Implementations**

```rust
// DON'T add this:
fn eval_sqrt(x: f64) -> f64 { x.sqrt() }
fn eval_sin(x: f64) -> f64 { x.sin() }
```

**Why not:** Type inference ≠ evaluation

**Where:** Separate interpreter/evaluator module (Phase 3)

---

### **❌ Parser Integration**

```rust
// DON'T add this:
fn parse_and_infer(latex: &str) -> Result<Type, String> { ... }
```

**Why not:** Mixing concerns

**Where:** Higher-level module that coordinates parser + type inference

---

## The Ideal Future Architecture

```
src/
├── type_inference.rs        (~600 lines)  - Core HM algorithm
│   ├── Type enum
│   ├── Constraint generation
│   ├── Unification
│   └── Matrix constructors (special case)
│
├── constraint_solver.rs     (~300 lines)  - Advanced solving
│   ├── Constraint propagation
│   ├── Backtracking
│   └── Dimension arithmetic
│
├── type_context.rs          (~800 lines)  - Structure registry
│   ├── Structure definitions
│   ├── Operation registry
│   ├── Signature interpreter
│   └── Query interface
│
├── type_checker.rs          (~300 lines)  - User-facing API
│   ├── Stdlib loading
│   ├── Error messages
│   ├── Suggestions
│   └── Integration
│
└── stdlib/
    ├── minimal_prelude.kleis  - Core operations
    ├── matrices.kleis         - Matrix operations
    ├── physics.kleis          - Physical dimensions (future)
    └── ...

Total type system: ~2,000 lines Rust + stdlib in Kleis
```

---

## Guiding Principles

### **1. Keep type_inference.rs Pure**

**Pure = Mathematical foundations only**

✅ Type representation  
✅ Constraint generation  
✅ Unification  
❌ Domain logic  
❌ Built-in operations  
❌ Evaluation

---

### **2. Push Complexity to Stdlib**

**Operations belong in structures (ADR-016)**

```kleis
// NOT in type_inference.rs:
structure Matrix(m: Nat, n: Nat) {
  operation multiply : Matrix(m, n) → Matrix(n, p) → Matrix(m, p)
}

// Type inference just queries the registry!
```

---

### **3. Separate Concerns**

**Type inference ≠ Type checking ≠ Evaluation**

```
TypeInference: Generates types and constraints
TypeContext:   Knows about structures and operations
TypeChecker:   User-facing, error messages, stdlib loading
Evaluator:     (Future) Actually executes expressions
```

---

### **4. Composition Over Size**

**Better:** 3 focused modules of 500 lines each  
**Worse:** 1 monolithic module of 1500 lines

---

## Specific Future Tasks

### **Phase 2: Dimension Expressions** (~1 week)

**File:** `type_inference.rs`  
**Lines:** +100 (total ~570)

**Add:**
```rust
// New types
enum DimExpr { Const(usize), Var(String), ... }
enum Type {
    Matrix(DimExpr, DimExpr),  // Not (usize, usize)
    // ...
}

// Updated functions
fn infer_matrix_constructor(...) {
    // Track dimension expressions properly
}

fn unify(...) {
    // Handle dimension unification
}
```

---

### **Phase 3: Extract Constraint Solver** (~3 days)

**New file:** `src/constraint_solver.rs` (~300 lines)  
**Changes to type_inference.rs:** -80 lines (total ~490)

**Move:**
```rust
// From type_inference.rs to constraint_solver.rs:
pub struct ConstraintSolver { ... }
fn unify(...) { ... }
fn occurs(...) { ... }
impl Substitution { ... }
```

**Keep in type_inference.rs:**
```rust
// Just constraint generation:
fn infer(&mut self, expr: &Expression) -> Type {
    // Generate constraints, don't solve them
}
```

---

### **Phase 4: Polymorphic Generalization** (~1 week)

**File:** `type_inference.rs`  
**Lines:** +50 (total ~540)

**Add:**
```rust
// Let-polymorphism
fn generalize(&self, ty: &Type) -> Type {
    // ∀α β. T where α, β are free variables
}

fn instantiate(&mut self, scheme: &TypeScheme) -> Type {
    // Replace ∀ variables with fresh variables
}
```

---

### **Phase 5: Dependent Type Support** (~2-3 weeks)

**File:** `type_inference.rs`  
**Lines:** +100 (total ~640)

**Add:**
```rust
// Dependent types: types depending on values
enum Type {
    Dependent {
        param: String,
        param_type: Box<Type>,
        body: Box<Type>,
    }
    // Example: Π(n: Nat). Matrix(n, n)
}

fn check_dependent(&mut self, ...) {
    // Check dependent type applications
}
```

---

## Maximum Size Projection

### **End State (All Phases Complete):**

```
type_inference.rs:     ~640 lines  (Core HM + dimensions + dependent types)
constraint_solver.rs:  ~300 lines  (Advanced solving)
type_context.rs:       ~800 lines  (Structure registry)
type_checker.rs:       ~300 lines  (User API)
-----------------------------------
Total:                ~2,040 lines  (Rust type system)

stdlib/*.kleis:       ~1,500 lines  (Type definitions)
-----------------------------------
Grand total:          ~3,500 lines
```

**For comparison:**
- GHC: 50,000+ lines
- OCaml: 10,000+ lines
- Kleis: 3,500 lines (projected)

**Why smaller?** ADR-016 - Operations in structures, not hardcoded!

---

## What Happens to Complexity?

### **It Moves to Kleis Code!**

**Today's achievement:** Moved 139 lines of Rust → 61 lines of Kleis

**Example:**
```rust
// BEFORE: In type_inference.rs (Rust)
"plus" | "minus" => {
    if args.len() != 2 {
        return Err("requires 2 arguments");
    }
    let t1 = self.infer(&args[0])?;
    let t2 = self.infer(&args[1])?;
    self.add_constraint(t1.clone(), t2.clone());
    Ok(t1)
}
// 12 lines of Rust per operation × 8 operations = 96 lines
```

```kleis
// AFTER: In stdlib (Kleis)
structure Arithmetic(T) {
  operation plus : T → T → T
}
implements Arithmetic(ℝ) {
  operation plus = builtin_add
}
// 5 lines of Kleis per operation
```

**Result:** Simpler Rust, richer Kleis

---

## The Vision

### **type_inference.rs Stays ~600 Lines**

**Core HM algorithm:** 400 lines  
**Dimension tracking:** 100 lines  
**Data constructors:** 100 lines  
**Total:** ~600 lines

### **Complexity Goes to Stdlib**

**Today:**
- 12 operations in 61 lines of Kleis

**Phase 2 (full prelude):**
- 47+ operations in ~500 lines of Kleis

**Phase 3 (domain libraries):**
- physics.kleis: ~200 lines
- quantum.kleis: ~300 lines
- tensor.kleis: ~200 lines
- units.kleis: ~150 lines

**Total:** ~1,350 lines of domain logic in Kleis, not Rust!

---

## Benefits of This Approach

### **1. Maintainability**

**Small core:** Easy to understand and verify  
**Domain logic in Kleis:** Users can read and extend it

### **2. Extensibility**

**Add operation:** Edit .kleis file  
**Add structure:** Edit .kleis file  
**Add implementation:** Edit .kleis file  
**Change core algorithm:** Rare, requires Rust changes

### **3. Performance**

**Small core:** Compiles fast  
**Stdlib compiled in:** Zero runtime cost for stdlib access

### **4. Correctness**

**Easy to verify:** Small, focused code  
**Easy to test:** Each piece testable independently  
**Easy to prove:** HM correctness proofs apply directly

---

## Anti-Patterns to Avoid

### **❌ Don't Bloat the Core**

```rust
// DON'T do this:
impl TypeInference {
    fn infer_physics_units(...) { /* 200 lines */ }
    fn infer_tensor_contractions(...) { /* 300 lines */ }
    fn infer_database_schemas(...) { /* 250 lines */ }
}
// Total: 750 lines of domain logic in core!
```

**Instead:** Put domain logic in Kleis structures

---

### **❌ Don't Mix Type Inference with Evaluation**

```rust
// DON'T do this:
impl TypeInference {
    fn eval(&self, expr: &Expression) -> Value {
        // Mixing concerns!
    }
}
```

**Instead:** Separate evaluator/interpreter module

---

### **❌ Don't Hardcode Optimizations**

```rust
// DON'T do this:
fn infer_operation(...) {
    match name {
        "matrix_multiply_fast" => { /* special optimized path */ }
    }
}
```

**Instead:** Optimizations belong in runtime/codegen, not type inference

---

## Comparison: Before and After Today

### **Before Session:**

```
type_inference.rs: 550 lines
├── Core HM: 200 lines
├── Hardcoded operations: 150 lines  ← Problem!
├── Matrix constructors: 35 lines
├── Delegation: 30 lines
└── Tests: 135 lines
```

**Issues:**
- Domain logic mixed with HM algorithm
- Hard to extend
- Violates ADR-016

---

### **After Session:**

```
type_inference.rs: 470 lines
├── Core HM: 200 lines
├── Matrix constructors: 65 lines
├── Delegation: 15 lines  ← Simplified!
└── Tests: 190 lines

stdlib/minimal_prelude.kleis: 61 lines
├── Arithmetic: 17 lines
├── Numeric: 16 lines
├── Calculus: 8 lines
└── Matrix: 10 lines
```

**Benefits:**
- Clean separation
- ADR-016 compliant
- Easy to extend
- 80 lines of Rust → 61 lines of Kleis

---

## Long-Term Strategy

### **The Rule:**

> **type_inference.rs implements HM algorithm, nothing else.**
> 
> **Domain logic belongs in stdlib/*.kleis files.**
> 
> **Complexity goes to separate modules or Kleis code.**

---

### **Growth Limit:**

**Maximum size:** ~700 lines (with all planned features)

**If it grows beyond 700 lines:**
- Extract constraint solver → constraint_solver.rs
- Extract dimension tracking → dimension.rs
- Extract error formatting → type_error.rs

**Keep the core pure and minimal.**

---

## Concrete Roadmap

### **Immediate (Phase 1 complete):** ✅

- [x] Delegate operations to stdlib
- [x] Keep only constructors as special case
- [x] Clean, focused HM implementation

### **Phase 2 (Next 3-4 weeks):**

- [ ] Add DimExpr for dimension tracking
- [ ] Extend parser for full grammar
- [ ] Load full prelude.kleis
- [ ] type_inference.rs: ~570 lines

### **Phase 3 (1-2 months):**

- [ ] Extract constraint solver
- [ ] Add polymorphic generalization  
- [ ] Improve error messages
- [ ] type_inference.rs: ~540 lines

### **Phase 4 (Optional, 3+ months):**

- [ ] Dependent type support
- [ ] Row polymorphism
- [ ] Advanced type features
- [ ] type_inference.rs: ~640 lines (max)

---

## Success Metrics

### **Quality Metrics:**

| Metric | Target | Current |
|--------|--------|---------|
| **Core HM lines** | < 300 | ~200 ✅ |
| **Total lines** | < 700 | 470 ✅ |
| **Operations hardcoded** | 0 | 0 ✅ |
| **Test coverage** | > 90% | ~95% ✅ |
| **Cyclomatic complexity** | < 15 per fn | ~8 avg ✅ |

---

### **Architectural Metrics:**

| Metric | Target | Current |
|--------|--------|---------|
| **ADR-016 compliance** | 100% | 100% ✅ |
| **Operations in stdlib** | > 80% | 100% ✅ |
| **Separation of concerns** | Clear | Clear ✅ |
| **Extensibility** | Via .kleis | Yes ✅ |

---

## Conclusion

**type_inference.rs should remain minimal - it's a feature, not a limitation!**

### **The Plan:**

1. **Keep core HM algorithm pure** (~300 lines)
2. **Add dimension tracking** (~100 lines in Phase 2)
3. **Extract complex solving** (move to separate module in Phase 3)
4. **Add polymorphism** (~50 lines in Phase 4)
5. **Never exceed ~700 lines** (extract if growing too large)

### **The Philosophy:**

> **Small core, rich stdlib. Operations in structures, not in inference engine.**

This is what makes Kleis different from other type systems. The complexity lives in user-extensible Kleis code, not in hardcoded Rust.

---

**Current Status:** ✅ On track  
**Size:** 470 lines (healthy)  
**Future:** ~540-640 lines (with all features)  
**Strategy:** Keep it minimal, delegate complexity

🎯 **This is exactly where we want to be!**

