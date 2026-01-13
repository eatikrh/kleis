# Symbolic Computation

Kleis isn't just about verification — it can *compute* with symbolic expressions. The crown jewel is `symbolic_diff.kleis`: a complete symbolic differentiation engine written in pure Kleis.

## The Expression AST

Mathematical expressions are represented as an algebraic data type:

```kleis
data Expression = 
    ENumber(value : ℝ)
  | EVariable(name : String)
  | EOperation(name : String, args : List(Expression))
```

This simple structure represents *any* mathematical expression:

| Expression | Representation |
|------------|---------------|
| `5` | `ENumber(5)` |
| `x` | `EVariable("x")` |
| `x + y` | `EOperation("plus", [EVariable("x"), EVariable("y")])` |
| `sin(x²)` | `EOperation("sin", [EOperation("power", [EVariable("x"), ENumber(2)])])` |

### Helper Constructors

For readability, helper functions build expressions:

```kleis
// Value constructors
define num(n) = ENumber(n)
define var(x) = EVariable(x)

// Operation constructors (e_ prefix avoids builtin conflicts)
define e_add(a, b) = EOperation("plus", Cons(a, Cons(b, Nil)))
define e_mul(a, b) = EOperation("times", Cons(a, Cons(b, Nil)))
define e_sub(a, b) = EOperation("minus", Cons(a, Cons(b, Nil)))
define e_div(a, b) = EOperation("divide", Cons(a, Cons(b, Nil)))
define e_pow(a, b) = EOperation("power", Cons(a, Cons(b, Nil)))
define e_neg(a) = EOperation("negate", Cons(a, Nil))
define e_sin(a) = EOperation("sin", Cons(a, Nil))
define e_cos(a) = EOperation("cos", Cons(a, Nil))
define e_sqrt(a) = EOperation("sqrt", Cons(a, Nil))
define e_exp(a) = EOperation("exp", Cons(a, Nil))
define e_ln(a) = EOperation("ln", Cons(a, Nil))
```

## Symbolic Differentiation

The `diff` function computes derivatives by pattern matching on the AST:

```kleis
define diff(e, var_name) = match e {
    // Constant rule: d/dx(c) = 0
    ENumber(_) => num(0)
    
    // Variable rule: d/dx(x) = 1, d/dx(y) = 0 if y ≠ x
    EVariable(name) => if str_eq(name, var_name) then num(1) else num(0)
    
    // Operation rules
    EOperation(op_name, args) => diff_op(op_name, args, var_name)
}
```

### Differentiation Rules

Each operation has its own differentiation rule:

```kleis
define diff_op(op_name, args, var_name) = match op_name {
    // Sum rule: d/dx(f + g) = df/dx + dg/dx
    "plus" => match args {
        Cons(f, Cons(g, Nil)) => e_add(diff(f, var_name), diff(g, var_name))
        | _ => num(0)
    }
    
    // Product rule: d/dx(f × g) = f'g + fg'
    "times" => match args {
        Cons(f, Cons(g, Nil)) => 
            e_add(e_mul(diff(f, var_name), g), e_mul(f, diff(g, var_name)))
        | _ => num(0)
    }
    
    // Chain rule for sin: d/dx(sin(f)) = cos(f) × f'
    "sin" => match args {
        Cons(f, Nil) => e_mul(e_cos(f), diff(f, var_name))
        | _ => num(0)
    }
    
    // Chain rule for cos: d/dx(cos(f)) = -sin(f) × f'
    "cos" => match args {
        Cons(f, Nil) => e_neg(e_mul(e_sin(f), diff(f, var_name)))
        | _ => num(0)
    }
    
    // Power rule: d/dx(f^n) = n × f^(n-1) × f'
    "power" => match args {
        Cons(f, Cons(ENumber(n), Nil)) => 
            e_mul(e_mul(num(n), e_pow(f, num(n - 1))), diff(f, var_name))
        | _ => num(0)
    }
    
    // Exponential rule: d/dx(e^f) = e^f × f'
    "exp" => match args {
        Cons(f, Nil) => e_mul(e_exp(f), diff(f, var_name))
        | _ => num(0)
    }
    
    // Logarithm rule: d/dx(ln(f)) = f'/f
    "ln" => match args {
        Cons(f, Nil) => e_div(diff(f, var_name), f)
        | _ => num(0)
    }
    
    // Square root: d/dx(√f) = f'/(2√f)
    "sqrt" => match args {
        Cons(f, Nil) => e_div(diff(f, var_name), e_mul(num(2), e_sqrt(f)))
        | _ => num(0)
    }
    
    | _ => num(0)  // Unknown operation
}
```

## Algebraic Simplification

Raw differentiation produces verbose expressions. The `simplify` function cleans them up:

```kleis
define simplify(e) = match e {
    // Base cases
    ENumber(n) => ENumber(n)
    EVariable(x) => EVariable(x)
    
    // Simplify operations recursively
    EOperation(op, args) => simplify_op(op, list_map(simplify, args))
}

define simplify_op(op, args) = match op {
    "plus" => match args {
        // 0 + x = x
        Cons(ENumber(0), Cons(x, Nil)) => x
        // x + 0 = x
        Cons(x, Cons(ENumber(0), Nil)) => x
        // n + m = (n+m)
        Cons(ENumber(n), Cons(ENumber(m), Nil)) => ENumber(n + m)
        | _ => EOperation("plus", args)
    }
    
    "times" => match args {
        // 0 × x = 0
        Cons(ENumber(0), _) => num(0)
        Cons(_, Cons(ENumber(0), Nil)) => num(0)
        // 1 × x = x
        Cons(ENumber(1), Cons(x, Nil)) => x
        // x × 1 = x
        Cons(x, Cons(ENumber(1), Nil)) => x
        // n × m = (n×m)
        Cons(ENumber(n), Cons(ENumber(m), Nil)) => ENumber(n * m)
        | _ => EOperation("times", args)
    }
    
    "power" => match args {
        // x^0 = 1
        Cons(_, Cons(ENumber(0), Nil)) => num(1)
        // x^1 = x
        Cons(x, Cons(ENumber(1), Nil)) => x
        | _ => EOperation("power", args)
    }
    
    | _ => EOperation(op, args)
}
```

## Example: Differentiating x²

```kleis
example "derivative of x squared" {
    let expr = e_pow(var("x"), num(2)) in      // x²
    let deriv = diff(expr, "x") in              // d/dx(x²)
    let simplified = simplify(deriv) in
    // Result: 2x (after simplification)
    simplified
}
```

Step-by-step:
1. `e_pow(var("x"), num(2))` = `EOperation("power", [EVariable("x"), ENumber(2)])`
2. Power rule: `e_mul(e_mul(num(2), e_pow(var("x"), num(1))), diff(var("x"), "x"))`
3. Variable rule: `diff(var("x"), "x")` = `num(1)`
4. Simplify: `2 × x^1 × 1` → `2 × x`

## Example: Chain Rule

```kleis
example "derivative of sin(x²)" {
    let expr = e_sin(e_pow(var("x"), num(2))) in   // sin(x²)
    let deriv = diff(expr, "x") in
    simplify(deriv)
    // Result: cos(x²) × 2x
}
```

## Substitution

The `subst` function replaces variables with values:

```kleis
define subst(e, var_name, value) = match e {
    ENumber(n) => ENumber(n)
    EVariable(name) => if str_eq(name, var_name) then value else e
    EOperation(op, args) => EOperation(op, list_map(λ a . subst(a, var_name, value), args))
}
```

**Usage:**

```kleis
example "evaluate at x=2" {
    let expr = e_pow(var("x"), num(2)) in   // x²
    let at_2 = subst(expr, "x", num(2)) in  // 2²
    simplify(at_2)                           // 4
}
```

## Application: Cartan Geometry

The symbolic differentiation engine powers the Cartan geometry computation. See the [Cartan Geometry appendix](../appendix/cartan-geometry.md) for details.

```kleis
// Compute exterior derivative of a 1-form
define d0(f) = [
    simplify(diff(f, "t")),
    simplify(diff(f, "r")),
    simplify(diff(f, "theta")),
    simplify(diff(f, "phi"))
]

// Schwarzschild metric factor
let f = e_sub(num(1), e_div(e_mul(num(2), M), var("r"))) in
// d(f) = [0, 2M/r², 0, 0]
d0(f)
```

## Coordinate-Specific Derivatives

For physics applications, we define coordinate-specific differentiation:

```kleis
define diff_t(e) = diff(e, "t")
define diff_r(e) = diff(e, "r")
define diff_theta(e) = diff(e, "theta")
define diff_phi(e) = diff(e, "phi")
```

These are used in the exterior derivative for differential forms:

```kleis
// (dω)_μν = ∂ω_ν/∂x^μ - ∂ω_μ/∂x^ν
define d1(omega) = 
    let w0 = nth(omega, 0) in
    let w1 = nth(omega, 1) in
    let w2 = nth(omega, 2) in
    let w3 = nth(omega, 3) in
    [
        [num(0),
         simplify(e_sub(diff_t(w1), diff_r(w0))),
         simplify(e_sub(diff_t(w2), diff_theta(w0))),
         simplify(e_sub(diff_t(w3), diff_phi(w0)))],
        // ... (antisymmetric matrix)
    ]
```

## Why This Matters

Traditional computer algebra systems (Mathematica, Maple) implement differentiation in their host language. In Kleis:

1. **Differentiation is pure Kleis** — no Rust builtins needed
2. **Rules are explicit** — you can read and verify them
3. **Extensible** — add new rules for new operations
4. **Verified** — axioms can be checked by Z3

This is **code as mathematics** — the implementation *is* the specification.

## What's Next

See how symbolic computation enables physics simulations:

→ [Next: Physics Applications](./25-physics-applications.md)

