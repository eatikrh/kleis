# Beta Reduction Implementation Plan

## Overview

Beta reduction (β-reduction) is the core computational step in lambda calculus, representing function application. This document outlines the implementation plan for adding beta reduction to Kleis lambdas.

## What is Beta Reduction?

**Formal Rule:**
```
(λ x . E) A  →β  E[x := A]
```

Where:
- `(λ x . E)` is a lambda with parameter `x` and body `E`
- `A` is the argument being applied
- `E[x := A]` is the body with all occurrences of `x` replaced by `A`

**Example:**
```kleis
(λ x . x + 1)(5)  →β  5 + 1  →  6
```

## Current State (After `feature/lambda-expressions`)

| Feature | Status |
|---------|--------|
| Parse `λ x . body` | ✅ |
| Store in AST | ✅ |
| Pretty print | ✅ |
| Z3 translation | ✅ |
| **Apply `(λ x . body)(arg)`** | ❌ Returns lambda as-is |

## Goals

1. **Full beta reduction**: `(λ x . x + 1)(5) → 6`
2. **Partial application**: `(λ x y . x + y)(3) → λ y . 3 + y`
3. **Nested reduction**: `(λ x . λ y . x + y)(3)(4) → 7`
4. **Alpha conversion**: Avoid variable capture
5. **Reduction strategy**: Normal order (lazy) by default

---

## Implementation Phases

### Phase 1: Basic Substitution

**File:** `src/evaluator.rs`

```rust
/// Perform beta reduction: (λ x . body)(arg) → body[x := arg]
fn beta_reduce(&self, lambda: &Expression, arg: &Expression) -> Result<Expression, String> {
    match lambda {
        Expression::Lambda { params, body } => {
            if params.is_empty() {
                return Ok((**body).clone());
            }
            
            let param = &params[0];
            
            // Substitute first param with arg
            let reduced_body = self.substitute_var(body, &param.name, arg);
            
            if params.len() == 1 {
                // Fully applied - return reduced body
                Ok(reduced_body)
            } else {
                // Partial application - return new lambda with remaining params
                Ok(Expression::Lambda {
                    params: params[1..].to_vec(),
                    body: Box::new(reduced_body),
                })
            }
        }
        _ => Err("Cannot beta-reduce non-lambda".to_string()),
    }
}
```

**Tests:**
```rust
#[test]
fn test_beta_reduce_simple() {
    // (λ x . x + 1)(5) → 5 + 1
    let lambda = Expression::Lambda {
        params: vec![LambdaParam { name: "x".to_string(), type_annotation: None }],
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        }),
    };
    let result = evaluator.beta_reduce(&lambda, &Expression::Const("5".to_string()));
    // Should be: Operation { name: "plus", args: [Const("5"), Const("1")] }
}
```

### Phase 2: Detection in Evaluator

**Detect lambda application pattern:**

```rust
// In eval() function
Expression::Operation { name, args } if is_lambda_application(name, args) => {
    // f(arg) where f is a lambda
    let lambda = &args[0];
    let arg = &args[1];
    self.beta_reduce(lambda, arg)
}

fn is_lambda_application(name: &str, args: &[Expression]) -> bool {
    // Direct lambda application: (λ x . body)(arg)
    matches!(&args[0], Expression::Lambda { .. }) && args.len() >= 1
}
```

**Also handle:**
```rust
// Named function that resolves to lambda
Expression::Operation { name, args } => {
    // Look up 'name' in environment
    if let Some(Expression::Lambda { .. }) = self.lookup(name) {
        // Reduce
    }
}
```

### Phase 3: Alpha Conversion (Variable Capture Avoidance)

**The Problem:**
```kleis
(λ x . λ y . x + y)(y)
// Naive substitution: λ y . y + y  ← WRONG! 'y' captured!
// Correct: λ z . y + z  (rename inner 'y' to 'z' first)
```

**Solution:**
```rust
/// Rename bound variable to avoid capture
fn alpha_convert(&self, expr: &Expression, old_name: &str, new_name: &str) -> Expression {
    match expr {
        Expression::Lambda { params, body } => {
            let new_params: Vec<_> = params.iter().map(|p| {
                if p.name == old_name {
                    LambdaParam { name: new_name.to_string(), ..p.clone() }
                } else {
                    p.clone()
                }
            }).collect();
            
            let new_body = self.alpha_convert(body, old_name, new_name);
            Expression::Lambda { params: new_params, body: Box::new(new_body) }
        }
        Expression::Object(name) if name == old_name => {
            Expression::Object(new_name.to_string())
        }
        // Recurse for other expression types...
        _ => expr.clone(),
    }
}

/// Check if substitution would cause variable capture
fn would_capture(&self, body: &Expression, var: &str, arg: &Expression) -> bool {
    let free_in_arg = self.free_variables(arg);
    let bound_in_body = self.bound_variables(body);
    free_in_arg.intersection(&bound_in_body).any(|v| v == var)
}

/// Get free variables in expression
fn free_variables(&self, expr: &Expression) -> HashSet<String> {
    // Implementation...
}

/// Get bound variables in expression
fn bound_variables(&self, expr: &Expression) -> HashSet<String> {
    // Implementation...
}
```

### Phase 4: Reduction Strategy

**Normal Order (Leftmost-Outermost First):**
- Always finds normal form if it exists
- Lazy evaluation semantics
- May re-evaluate arguments multiple times

```rust
fn reduce_normal_order(&self, expr: &Expression) -> Expression {
    match expr {
        // If it's a redex (lambda application), reduce it
        Expression::Operation { args, .. } if is_redex(expr) => {
            let reduced = self.beta_reduce(&args[0], &args[1]);
            self.reduce_normal_order(&reduced) // Continue reducing
        }
        // Otherwise, reduce leftmost sub-expression
        Expression::Operation { name, args } => {
            // Try to reduce first arg, then second, etc.
        }
        // Lambda: reduce body
        Expression::Lambda { params, body } => {
            Expression::Lambda {
                params: params.clone(),
                body: Box::new(self.reduce_normal_order(body)),
            }
        }
        _ => expr.clone(),
    }
}
```

**Applicative Order (Arguments First):**
- Eager evaluation semantics
- More efficient when args are always needed
- May not terminate when normal order would

```rust
fn reduce_applicative_order(&self, expr: &Expression) -> Expression {
    // Reduce arguments first, then apply
}
```

### Phase 5: Termination Safety

**The Ω Combinator Problem:**
```kleis
define omega = λ x . x(x)
omega(omega)  // Infinite loop!
```

**Solution: Fuel/Step Limit**
```rust
fn reduce_with_fuel(&self, expr: &Expression, fuel: usize) -> Result<Expression, String> {
    if fuel == 0 {
        return Err("Reduction limit exceeded (possible infinite loop)".to_string());
    }
    
    match self.step(expr)? {
        Some(reduced) => self.reduce_with_fuel(&reduced, fuel - 1),
        None => Ok(expr.clone()), // Normal form reached
    }
}

// Default fuel limit
const DEFAULT_REDUCTION_FUEL: usize = 1000;
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/evaluator.rs` | Add `beta_reduce()`, `alpha_convert()`, reduction strategies |
| `src/evaluator.rs` | Modify `eval()` to detect and reduce lambda applications |
| `src/ast.rs` | Possibly add `Application` variant for explicit application |
| `src/type_inference.rs` | Infer function types during reduction |
| `src/bin/repl.rs` | Add `:reduce` command for step-by-step reduction |

---

## Test Cases

### Basic Reduction
```kleis
(λ x . x)(5)                    // → 5 (identity)
(λ x . x + 1)(5)                // → 6
(λ x . x * x)(3)                // → 9
```

### Partial Application
```kleis
(λ x y . x + y)(3)              // → λ y . 3 + y
(λ x y . x + y)(3)(4)           // → 7
```

### Nested Lambdas
```kleis
(λ f . f(0))(λ x . x + 1)       // → 1
```

### Variable Capture
```kleis
(λ x . λ y . x + y)(y)          // → λ z . y + z (not λ y . y + y!)
```

### Non-Termination Detection
```kleis
define omega = λ x . x(x)
omega(omega)                     // → Error: Reduction limit exceeded
```

### Church Numerals (Stretch Goal)
```kleis
define zero = λ f x . x
define succ = λ n f x . f(n(f)(x))
define one = succ(zero)          // → λ f x . f(x)
define two = succ(one)           // → λ f x . f(f(x))
```

---

## Success Criteria

- [ ] `(λ x . x + 1)(5)` evaluates to `6` in REPL
- [ ] Partial application works: `(λ x y . x + y)(3)` returns `λ y . 3 + y`
- [ ] Alpha conversion prevents variable capture
- [ ] Infinite loops are detected and reported
- [ ] All existing tests still pass
- [ ] Z3 verification still works with reduced expressions

---

## Timeline Estimate

| Phase | Effort |
|-------|--------|
| Phase 1: Basic substitution | 2-3 hours |
| Phase 2: Detection in evaluator | 1-2 hours |
| Phase 3: Alpha conversion | 2-3 hours |
| Phase 4: Reduction strategy | 2-3 hours |
| Phase 5: Termination safety | 1-2 hours |
| Testing & edge cases | 2-3 hours |
| **Total** | **10-16 hours** |

---

## References

- [Lambda Calculus on Wikipedia](https://en.wikipedia.org/wiki/Lambda_calculus)
- [Beta Reduction - OpenDSA](https://opendsa-server.cs.vt.edu/OpenDSA/Books/PL/html/BetaReduction.html)
- [A Gentle Introduction to Lambda Calculus](https://lucasfcosta.com/2018/08/04/lambda-calculus-2.html)
- [nLab: Lambda Calculus](https://ncatlab.org/nlab/show/lambda+calculus)

---

## Notes

- Start with normal order reduction (matches Haskell semantics)
- Consider adding `:trace` REPL command to show reduction steps
- Church encoding of data types is a stretch goal
- Integration with Z3 may need special handling for reduced expressions

