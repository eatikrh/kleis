# Kleis Evaluation Syntax - Design Proposal

## Status
**Draft** - Proposed based on analysis of SymPy, Mathematica, REDUCE, and Lisp patterns

## Philosophy

Following ADR-002, Kleis maintains strict separation:
- **Evaluation**: Semantic computation, type-aware, minimal, deterministic
- **Simplification**: Optional transformation, cognitive optimization, heuristic
- **Substitution**: Symbolic replacement without evaluation

---

## Core Concepts

### 1. Context (Environment)

Every evaluation happens with respect to a **context**:

```rust
Context {
    bindings: Map<Symbol, Value>,
    types: Map<Symbol, Type>,
    operations: Map<OpName, OpDef>,
}
```

Example:
```
context physics {
    c : Scalar = 299792458      // Speed of light
    G : Scalar                  // Gravitational constant (symbolic)
    ℏ : Scalar = 1.054571e-34  // Planck constant
    E : Field                   // Electric field (typed, unbound)
}
```

---

## Evaluation Operations

### 1. Substitute (Symbolic Replacement)

**Syntax:**
```
expr.substitute(x → value)
expr.substitute_all({x: val1, y: val2, ...})
```

**Semantics:**
- Returns new Expression (still symbolic)
- No type checking required
- No evaluation performed
- Pure syntactic replacement

**Example:**
```
let expr = x² + 2x + 1
expr.substitute(x → y + 1)
// → (y + 1)² + 2(y + 1) + 1  (not expanded)
```

**AST Operation:**
```rust
impl Expression {
    fn substitute(&self, symbol: &str, replacement: &Expression) -> Expression {
        // Tree traversal and replacement
    }
}
```

---

### 2. Eval (Semantic Evaluation)

**Syntax:**
```
expr.eval(context)
expr.eval_to_number(context)  // Force numeric result
```

**Semantics:**
- Type-aware computation
- Respects operation semantics
- May remain symbolic if values unavailable
- Returns typed result

**Examples:**
```
// Pure symbolic
let expr = ∇φ
expr.eval(context)  
// → grad_field(φ) : Field  (not expanded without φ definition)

// With bindings
let expr = E² / (2ε₀)
context = {E: 1000.0 : Scalar, ε₀: 8.854e-12 : Scalar}
expr.eval(context)
// → 5.65e16 : Scalar

// Partial evaluation
let expr = x² + y
context = {x: 2 : Scalar}
expr.eval(context)
// → 4 + y : Scalar  (y remains symbolic)
```

**Type Rules:**
```
eval(a : T₁ + b : T₂) → Result<T₃, TypeError>
  where T₃ = promote(T₁, T₂)

eval(∇φ : Potential) → Field
eval(∫F·dS) → Scalar (if F : Field)
```

---

### 3. Apply (Operation Application)

**Syntax:**
```
operation.apply(args...)
operation.apply_with_context(args..., context)
```

**Semantics:**
- Direct operation invocation
- Type-checked before application
- Returns typed result or error

**Examples:**
```
// Define operation
operation surface_integral : (Field, Surface) → Scalar

// Apply
let field = E
let surface = S₂  // Unit sphere
surface_integral.apply(field, surface)
// → ∫∫_S₂ E·dS : Scalar

// With context
context = {E: electric_field : Field}
surface_integral.apply_with_context(E, S₂, context)
// → symbolic or numeric depending on E binding
```

---

### 4. Simplify (Cognitive Optimization)

**Syntax:**
```
expr.simplify()
expr.simplify_with_rules(ruleset)
expr.expand()
expr.factor()
expr.collect_terms(var)
```

**Semantics:**
- Optional transformation
- Preserves semantic equality
- May use heuristics
- Returns simpler (cognitively) form

**Examples:**
```
let expr = (x + 1)² 
expr.expand()
// → x² + 2x + 1

let expr = x² - 1
expr.factor()
// → (x - 1)(x + 1)

let expr = sin²(x) + cos²(x)
expr.simplify()
// → 1  (trig identity)
```

---

## Advanced Features

### 1. Held Expressions (Deferred Evaluation)

**Syntax:**
```
hold(expr)         // Prevent evaluation
release(held_expr) // Allow evaluation
```

**Use Case:**
```
// Define a symbolic rule without evaluating
let rule = hold(x² → (x + 1)(x - 1))

// Apply when needed
expr.apply_rule(release(rule))
```

---

### 2. Partial Evaluation

**Syntax:**
```
expr.eval_partial(context)
expr.eval_until(depth: usize)
```

**Example:**
```
let expr = ∫₀^∞ e^(-x²) dx / Γ(s)

context = {s: 2}
expr.eval_partial(context)
// → ∫₀^∞ e^(-x²) dx / 2  (Γ(2) = 1 computed, integral remains)
```

---

### 3. Type-Directed Evaluation

**Syntax:**
```
expr.eval_as_type(target_type, context)
expr.coerce_to(target_type)
```

**Example:**
```
let expr = [1, 2, 3]  // Vector or Matrix row?

expr.eval_as_type(Vector, context)
// → Vector₃(1, 2, 3)

expr.eval_as_type(Matrix, context)  
// → Matrix₁ₓ₃([1, 2, 3])
```

---

## Proposed Kleis Syntax

### Expression Methods

```rust
// Core operations
expr.substitute(symbol → replacement) → Expression
expr.substitute_all(bindings: Map) → Expression
expr.eval(context: Context) → Result<Value, EvalError>
expr.simplify() → Expression

// Type operations
expr.typecheck(context: Context) → Result<Type, TypeError>
expr.infer_type(context: Context) → Option<Type>

// Advanced
expr.eval_partial(context: Context) → Expression
expr.apply_operation(op: Operation, context: Context) → Result<Value, Error>
hold(expr) → HeldExpression
release(held) → Expression
```

### Operation Definition

```rust
operation grad {
    signature: Potential → Field
    
    eval(φ: Potential, ctx: Context) → Field {
        // Compute gradient symbolically or numerically
        match φ {
            Symbolic => Field::symbolic(∇φ),
            Numeric(grid) => Field::numeric(compute_gradient(grid)),
        }
    }
}
```

### Context Definition

```
context quantum {
    ℏ : Scalar = 1.054571e-34
    c : Scalar = 299792458
    |ψ⟩ : StateVector  // Type but no value
    Ĥ : Operator       // Type but no value
    
    // Type aliases
    type WaveFunction = ComplexField
    type Operator = Matrix  // Or more sophisticated
}
```

---

## Comparison Table

| Feature | SymPy | Mathematica | REDUCE | Lisp | **Kleis** |
|---------|-------|-------------|---------|------|-----------|
| Substitute | `.subs()` | `/.` | `sub()` | `subst` | `.substitute()` |
| Evaluate | `.evalf()` | `N[]` | `eval` | `eval` | `.eval()` |
| Simplify | `.simplify()` | `Simplify[]` | `simplify` | custom | `.simplify()` |
| Apply | N/A | `Apply[]` | `apply` | `apply` | `.apply_operation()` |
| Hold | N/A | `Hold[]` | N/A | `'expr` | `hold()` |
| Context | dict | `Block[]` | `let` | `let` | `context {}` |
| Types | Limited | Patterns | None | CLOS | **First-class** |

---

## Key Kleis Distinctions

**1. Types Are Central**
```
// Not just: eval(expr, {x: 2})
// But: eval(expr, {x: 2 : Scalar})
```

**2. Multiple Equality Types** (from grammar)
```
define  E = mc²      // Definition (by fiat)
assert  a² + b² == c²  // Algebraic claim
equiv   ∇×E ~ -∂B/∂t   // Structural equivalence
approx  π ≈ 3.14159    // Numerical approximation
```

**3. Explicit Operation Semantics**
```
operation grad {
    signature: Potential → Field
    laws: [linearity, product_rule]
    eval(φ, ctx) { ... }
}
```

**4. Evaluation Control**
```
// Don't evaluate, just check types
expr.typecheck(context) → Result<Type, Error>

// Evaluate symbolically
expr.eval(context) → Expression | Value

// Force numeric
expr.eval_to_number(context) → f64 | Error
```

---

## Recommended Starting Point

**Minimal MVP Syntax:**

```rust
// 1. Substitution (pure syntactic)
expr.substitute(symbol, replacement)

// 2. Evaluation (semantic, type-aware)
expr.eval(context) → Result<Value, Error>
  where Value = Expression | Numeric(f64) | SymbolicField | ...

// 3. Type checking (before eval)
expr.typecheck(context) → Result<Type, TypeError>

// 4. Context definition
let ctx = Context::new()
    .bind("x", Value::Scalar(2.0))
    .bind_type("E", Type::Field)
```

**Rationale:**
- Start simple: substitute, eval, typecheck
- Add simplify later (per ADR-002)
- Add hold/release only if needed
- Context is explicit (not global)

**This gives you:**
- Clean separation (ADR-002 compliant)
- Type-first design (matches grammar)
- Familiar syntax (like SymPy/Mathematica)
- Room to grow (can add `apply`, `hold`, etc.)

---

## Implementation Notes

**For Your Current AST:**
```rust
// Already have
pub enum Expression {
    Const(String),         // Numeric/symbolic constants
    Object(String),        // Variables/symbols
    Placeholder { ... },   // Unknowns
    Operation { name, args },
}

// Add
pub enum Value {
    Scalar(f64),
    Complex(f64, f64),
    Vector(Vec<Value>),
    Matrix(Vec<Vec<Value>>),
    Field(FieldData),
    Symbolic(Expression),  // Unevaluated
}

pub struct Context {
    bindings: HashMap<String, Value>,
    types: HashMap<String, Type>,
}

impl Expression {
    pub fn eval(&self, context: &Context) -> Result<Value, EvalError> {
        match self {
            Expression::Object(name) => {
                context.bindings.get(name)
                    .cloned()
                    .ok_or(EvalError::UnboundSymbol(name))
            }
            Expression::Const(val) => {
                // Parse numeric or keep symbolic
                val.parse::<f64>()
                    .map(Value::Scalar)
                    .unwrap_or(Value::Symbolic(self.clone()))
            }
            Expression::Operation { name, args } => {
                eval_operation(name, args, context)
            }
            _ => Ok(Value::Symbolic(self.clone()))
        }
    }
}
```

---

## Multi-Valued Operations: The ± Problem

### The Challenge

The quadratic formula notation presents a fundamental semantic issue:

```
x = (-b ± √(b² - 4ac)) / (2a)
```

The `±` operator is **syntactic sugar** for two separate equations:
```
x₁ = (-b + √(b² - 4ac)) / (2a)
x₂ = (-b - √(b² - 4ac)) / (2a)
```

**Design question**: How does Kleis represent multi-valued results?

### Option 1: Multi-Valued Type

Extend the `Value` enum to support solution sets:

```rust
pub enum Value {
    Scalar(f64),
    Complex(f64, f64),
    Vector(Vec<Value>),
    Matrix(Vec<Vec<Value>>),
    MultiValue(Vec<Value>),  // For ± results
    Set(HashSet<Value>),      // For solution sets
    Field(FieldData),
    Symbolic(Expression),
}
```

Evaluation semantics:
```rust
Operation("plus_minus", [a, b]) → MultiValue([
    eval(a) + eval(b),
    eval(a) - eval(b)
])
```

**Issue**: What does `x = MultiValue([3, 2])` mean?
- Is `x` a set `{2, 3}`?
- Is it two separate bindings `x₁=2, x₂=3`?
- Is it a constraint `x ∈ {2, 3}`?

### Option 2: Solver Expansion

Treat equations with `±` as **constraints to solve**, not direct assignments:

```rust
solve(x, equation_with_pm) → Context {
    x: MultiValue([root1, root2])
}
```

The `±` operator only has meaning in a **solving context**, not general evaluation.

### Option 3: Indexed Expansion

Parser/evaluator automatically expands:
```
x = expr with ±
```
Into:
```
x₁ = expr with +
x₂ = expr with -
```

Returns a `Context` with multiple bindings.

### Recommended Approach

**Two-phase interpretation**:

1. **Parsing**: Recognize `\pm` as `Operation("plus_minus", [a, b])`
2. **Evaluation**:
   - Direct eval: `plus_minus(a, b) → MultiValue([a+b, a-b])`
   - In equation context: Recognize as constraint, invoke solver
   - Type system: `x: Scalar | MultiValue<Scalar>`

### Parser Requirements

To support `x = \frac{-b \pm \sqrt{...}}{2a}`:

1. Add `\pm` token recognition
2. Parse as binary operator: `plus_minus(left, right)`
3. Handle precedence (binds tighter than `+`, looser than unary `-`)
4. Extend equation solver to detect and expand multi-valued operations

### Future Work

- Define semantics for `MultiValue` in arithmetic (distribute? error? symbolic?)
- Extend type system to track solution multiplicity
- Add `solve()` operation that returns all roots/solutions
- Consider constraint satisfaction vs direct evaluation

**Status**: Design exploration - awaiting decision on Kleis equation semantics

---

## Next Steps

1. **Implement minimal eval** - Just `substitute`, `eval`, `typecheck`
2. **Add Context struct** - Bindings + type information
3. **Test with physics examples** - E = mc², F = ma, etc.
4. **Design multi-valued semantics** - Decide on ± operator behavior
5. **Add simplifier later** - Per ADR-002, keep it separate
6. **Extend as needed** - `apply`, `hold`, etc. when use cases emerge

This gives Kleis a **clean, type-first evaluation model** that's familiar to users of existing CAS systems but more principled about types and evaluation control.

---

**Would you like me to implement a prototype `eval()` function to test this design?**

