# Operator Overloading Implementation Plan

> **Status: ✅ IMPLEMENTED** (Dec 2024)  
> Type-directed semantic lowering is working. Natural syntax like `3 + 4*i` evaluates correctly.
> 17 integration tests pass. See `src/lowering.rs` for implementation.

**Original Status**: Planned  
**Priority**: High  
**Estimated Effort**: 4-5 sessions  
**Created**: December 2024  

## Goal

Enable natural arithmetic syntax for complex numbers (and future numeric types):

```kleis
// What users want to write:
define z = 3 + 4*i
define sum = z1 + z2
define product = z1 * z2
define quotient = z1 / z2

// Instead of:
define z = complex(3, 4)
define sum = complex_add(z1, z2)
define product = complex_mul(z1, z2)
define quotient = complex_div(z1, z2)
```

## Architecture: Semantic Lowering Pass

The solution adds a **type-directed lowering pass** between type inference and backend translation:

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. PARSER                                                       │
│    "3 + 4*i" → Operation { name: "plus", args: [3, times(4, i)] }│
│    Purely syntactic, no type knowledge                          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. TYPE INFERENCE (existing)                                    │
│    Annotates each AST node with its inferred type               │
│    3 : ℝ, 4 : ℝ, i : ℂ, times(4, i) : ℂ, plus(...) : ℂ         │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. SEMANTIC LOWERING (NEW)                                      │
│    Rewrites generic operators to type-specific operations:      │
│                                                                 │
│    plus(a : ℂ, b : ℂ) → complex_add(a, b)                       │
│    plus(a : ℝ, b : ℂ) → complex_add(lift(a), b)                 │
│    times(a : ℂ, b : ℂ) → complex_mul(a, b)                      │
│    neg(a : ℂ) → neg_complex(a)                                  │
│                                                                 │
│    where lift(r : ℝ) = complex(r, 0)                            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. BACKEND (Z3, Evaluator, Renderer)                            │
│    Receives lowered AST with explicit operations                │
│    No special-casing needed - complex_add already works         │
└─────────────────────────────────────────────────────────────────┘
```

## Why This Approach?

| Alternative | Pros | Cons |
|-------------|------|------|
| **Parser sugar** (`a+b*i` pattern) | Simple | Only handles literals, not general expressions |
| **Backend special-casing** | Quick | Inconsistent across backends, duplicated logic |
| **Full type classes** | Principled | Major language feature, months of work |
| **Semantic lowering** ✅ | Correct, extensible, consistent | Requires typed AST |

Semantic lowering is the sweet spot: type-correct, works everywhere, and provides a foundation for future type classes.

## Implementation Phases

### Phase 1: Typed AST (1 session)

**Goal**: Annotate AST nodes with their inferred types.

**Files to modify**:
- `src/ast.rs` — Add `TypedExpression` enum or type annotations to `Expression`
- `src/type_inference.rs` — Return typed AST instead of just `Type`

**Design Decision**: Two options:

**Option A: Separate TypedExpression enum**
```rust
// New enum that mirrors Expression but includes types
pub enum TypedExpression {
    Object { name: String, ty: Type },
    Const { value: String, ty: Type },
    Operation { name: String, args: Vec<TypedExpression>, ty: Type },
    // ... etc
}
```

**Option B: Annotate existing Expression**
```rust
// Wrapper that pairs Expression with Type
pub struct TypedExpr {
    pub expr: Expression,
    pub ty: Type,
    pub children: Vec<TypedExpr>,  // Typed sub-expressions
}
```

**Recommendation**: Option A is cleaner but requires more code. Option B is more incremental. Start with B, refactor to A later if needed.

**Deliverables**:
- [ ] TypedExpr struct or TypedExpression enum
- [ ] `TypeInference::infer_typed()` method that returns typed AST
- [ ] Unit tests for typed AST construction

### Phase 2: Lowering Module (1-2 sessions)

**Goal**: Transform typed AST by rewriting generic operators to type-specific ones.

**New file**: `src/lowering.rs`

```rust
//! Semantic Lowering Pass
//!
//! Transforms typed AST by rewriting generic operators to type-specific operations.
//! This enables operator overloading without language-level type classes.

use crate::ast::{Expression, Type};
use crate::typed_ast::TypedExpr;

pub struct SemanticLowering;

impl SemanticLowering {
    /// Lower a typed expression to an untyped expression with explicit operations
    pub fn lower(&self, expr: &TypedExpr) -> Expression {
        match &expr.expr {
            Expression::Operation { name, args } => {
                self.lower_operation(name, &expr.children, &expr.ty)
            }
            Expression::Object(name) => Expression::Object(name.clone()),
            Expression::Const(value) => Expression::Const(value.clone()),
            // ... handle other cases
        }
    }
    
    fn lower_operation(
        &self, 
        name: &str, 
        args: &[TypedExpr], 
        result_ty: &Type
    ) -> Expression {
        // First, recursively lower arguments
        let lowered_args: Vec<Expression> = args.iter()
            .map(|a| self.lower(a))
            .collect();
        
        // Then, check if we need to rewrite based on types
        match (name, args.get(0).map(|a| &a.ty), args.get(1).map(|a| &a.ty)) {
            // Binary operators with complex operands
            ("plus", Some(Type::Complex), Some(Type::Complex)) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: lowered_args,
                }
            }
            ("plus", Some(Type::Real), Some(Type::Complex)) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }
            ("plus", Some(Type::Complex), Some(Type::Real)) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }
            // Similar for minus, times, divide...
            
            // Unary operators
            ("neg", Some(Type::Complex), None) => {
                Expression::Operation {
                    name: "neg_complex".to_string(),
                    args: lowered_args,
                }
            }
            
            // Default: keep operation as-is
            _ => Expression::Operation {
                name: name.to_string(),
                args: lowered_args,
            }
        }
    }
    
    /// Lift a real expression to complex: r → complex(r, 0)
    fn lift_to_complex(&self, expr: &Expression) -> Expression {
        Expression::Operation {
            name: "complex".to_string(),
            args: vec![expr.clone(), Expression::Const("0".to_string())],
        }
    }
}
```

**Operator Mapping Table**:

| Operator | Arg Types | Lowered To |
|----------|-----------|------------|
| `plus` | ℂ × ℂ | `complex_add` |
| `plus` | ℝ × ℂ | `complex_add(lift, _)` |
| `plus` | ℂ × ℝ | `complex_add(_, lift)` |
| `minus` | ℂ × ℂ | `complex_sub` |
| `minus` | ℝ × ℂ | `complex_sub(lift, _)` |
| `minus` | ℂ × ℝ | `complex_sub(_, lift)` |
| `times` | ℂ × ℂ | `complex_mul` |
| `times` | ℝ × ℂ | `complex_mul(lift, _)` |
| `times` | ℂ × ℝ | `complex_mul(_, lift)` |
| `divide` | ℂ × ℂ | `complex_div` |
| `divide` | ℝ × ℂ | `complex_div(lift, _)` |
| `divide` | ℂ × ℝ | `complex_div(_, lift)` |
| `neg` (unary) | ℂ | `neg_complex` |
| `conj` | ℂ | `conj` (unchanged) |

**Deliverables**:
- [ ] `src/lowering.rs` module
- [ ] `SemanticLowering` struct with `lower()` method
- [ ] All operator cases implemented
- [ ] Unit tests for each lowering rule

### Phase 3: Integration (1 session)

**Goal**: Wire the lowering pass into all code paths.

**Integration Points**:

1. **REPL** (`src/bin/repl.rs`)
   ```rust
   // Before:
   let expr = parse(input)?;
   backend.verify(&expr)?;
   
   // After:
   let expr = parse(input)?;
   let typed = type_inference.infer_typed(&expr)?;
   let lowered = lowering.lower(&typed);
   backend.verify(&lowered)?;
   ```

2. **Kleis CLI** (`src/main.rs`)
   - Same pattern: parse → infer types → lower → check

3. **AxiomVerifier** (`src/axiom_verifier.rs`)
   - May need to lower axioms before loading

4. **Evaluator** (`src/evaluator.rs`)
   - Could also benefit from lowering for symbolic evaluation

**Deliverables**:
- [ ] REPL uses lowering before verification
- [ ] CLI uses lowering before checking
- [ ] Documentation updated

### Phase 4: Testing (1 session)

**Goal**: Comprehensive tests for all operator/type combinations.

**Test Cases**:

```rust
#[test]
fn test_complex_addition_both_complex() {
    // complex(1,2) + complex(3,4) = complex(4,6)
    verify("complex(1,2) + complex(3,4) = complex(4,6)");
}

#[test]
fn test_complex_addition_real_left() {
    // 3 + complex(1,2) = complex(4,2)
    verify("3 + complex(1,2) = complex(4,2)");
}

#[test]
fn test_complex_addition_real_right() {
    // complex(1,2) + 3 = complex(4,2)
    verify("complex(1,2) + 3 = complex(4,2)");
}

#[test]
fn test_complex_literal_sugar() {
    // 3 + 4*i = complex(3,4)
    verify("3 + 4*i = complex(3,4)");
}

#[test]
fn test_complex_multiplication() {
    // i * i = -1
    verify("i * i = complex(-1,0)");
}

#[test]
fn test_mixed_expression() {
    // (1 + 2*i) * (3 + 4*i) = -5 + 10*i
    verify("(1 + 2*i) * (3 + 4*i) = complex(-5,10)");
}

#[test]
fn test_symbolic_with_operators() {
    // ∀(z : ℂ). z + complex(0,0) = z
    verify("∀(z : ℂ). z + complex(0,0) = z");
}
```

**Deliverables**:
- [ ] `tests/operator_overloading.rs` with comprehensive tests
- [ ] All existing tests still pass
- [ ] Manual examples work with new syntax

## Type Hierarchy for Future Extension

The lowering pass should be extensible for future numeric types:

```
         Scalar
        /      \
      ℂ        Bool
      |
      ℝ
      |
      ℤ
      |
      ℕ
```

**Lifting rules** (implicit coercion):
- ℕ → ℤ → ℝ → ℂ

When operands have different types, lift the "smaller" type:
```
ℕ + ℂ → lift ℕ to ℂ → complex_add
ℤ + ℝ → lift ℤ to ℝ → real_add (just "plus")
```

## Future Extensions

Once the lowering infrastructure exists, we can add:

1. **Matrix arithmetic**: `A + B`, `A * B` for matrices
2. **Vector arithmetic**: `v + w`, `λ * v` for vectors  
3. **Quaternions**: `q1 * q2` for quaternion multiplication
4. **Tensor operations**: Automatic index contraction

Each requires:
1. Add types to type system
2. Add lowering rules
3. Implement underlying operations

## Success Criteria

- [ ] `3 + 4*i` evaluates to `complex(3, 4)`
- [ ] `z1 + z2` works for complex variables
- [ ] Mixed real/complex arithmetic works
- [ ] All existing tests pass
- [ ] REPL, CLI, and verification all work
- [ ] No performance regression

## Open Questions

1. **Should we support `a + bi` syntax in parser?**
   - Currently: `a + b*i` works after lowering
   - Alternative: Parser-level pattern for `a + bi` (juxtaposition)
   - Decision: Defer; `a + b*i` is sufficient

2. **What about comparison operators?**
   - `z1 = z2` already works (structural equality)
   - `z1 < z2` doesn't make sense for complex (not ordered)
   - Decision: Leave comparisons as-is for now

3. **Error messages for type mismatches?**
   - Current: Type inference errors
   - Enhanced: "Cannot add Bool and Complex"
   - Decision: Improve error messages as separate work

## References

- `stdlib/complex.kleis` — Complex number axioms and conventions
- `src/solvers/z3/backend.rs` — Hybrid datatype implementation
- `docs/manual/src/chapters/14-complex-numbers.md` — User documentation
- Haskell type classes — Inspiration for future full type classes

