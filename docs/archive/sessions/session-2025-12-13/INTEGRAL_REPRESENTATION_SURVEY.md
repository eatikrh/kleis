# Survey: How Math Software Represents Definite Integrals

## Summary

**Key Finding:** Major mathematical software systems do NOT treat definite integrals as distinct types. Instead, they treat integration as:

1. **An operation/function** that maps (integrand, bounds) → value
2. **A symbolic expression** that can remain unevaluated

This is an important insight for Kleis's design.

---

## System-by-System Analysis

### 1. Mathematica

**Representation:** Expression/Operation (not a type)

```mathematica
(* Definite integral - evaluates if possible *)
Integrate[x^2, {x, 0, 1}]   (* Returns: 1/3 *)

(* Unevaluated integral - remains symbolic *)
Integrate[f[x], {x, a, b}]  (* Returns: Integrate[f[x], {x, a, b}] *)
```

**Internal Structure:**
- `Integrate` is a function head
- Arguments: `[integrand, {variable, lower, upper}]`
- Result type: Expression (same type as input)
- No distinct "Integral" type

**Philosophy:** Integration is an operation that transforms expressions. When it can't compute symbolically, it returns the unevaluated `Integrate[...]` expression.

**Source:** [Wolfram Documentation](https://www.wolfram.com/language/fast-introduction-for-math-students/integrals/)

---

### 2. Coq (Rocq)

**Representation:** Function returning ℝ (not a type)

```coq
(* From Coq.Reals.RiemannInt *)
Definition RiemannInt (f : R -> R) (a b : R) (pr : Riemann_integrable f a b) : R.
```

**Key Points:**
- `RiemannInt` is a **function** that takes:
  - `f : R -> R` (the integrand)
  - `a b : R` (bounds)
  - `pr : Riemann_integrable f a b` (proof that f is integrable)
- Returns: `R` (a real number)
- No distinct "Integral" type - it's a function to ℝ

**Philosophy:** The integral is defined as a function from (integrable functions × bounds) to real numbers. The **proof of integrability** is a required argument.

**Source:** [Coq Standard Library - RiemannInt](https://rocq-prover.org/doc/V8.16.0/stdlib/Coq.Reals.RiemannInt.html)

---

### 3. Lean (mathlib)

**Representation:** Function returning value (not a type)

```lean
-- From mathlib MeasureTheory.Integral.IntervalIntegral
def intervalIntegral (f : ℝ → E) (μ : Measure ℝ) (a b : ℝ) : E :=
  ∫ x in Ioc a b, f x ∂μ - ∫ x in Ioc b a, f x ∂μ
```

**Key Points:**
- `intervalIntegral` is a **function** that returns type `E` (the codomain of f)
- Uses Bochner integral under the hood
- No distinct "Integral" type

**Notation:** Lean uses `∫ x in a..b, f x` which is notation for `intervalIntegral f volume a b`

**Philosophy:** Integration is an operation defined via measure theory. The integral of `f : ℝ → E` is an element of `E`.

**Source:** [mathlib docs - IntervalIntegral](https://leanprover-community.github.io/mathlib_docs/measure_theory/integral/interval_integral.html)

---

### 4. Isabelle/HOL

**Representation:** Function with properties (not a type)

```isabelle
definition integral :: "('a ⇒ real) ⇒ 'a set ⇒ real" where
  "integral f A = (THE r. (f has_integral r) A)"
```

**Key Points:**
- `integral` is a function: `(function, set) → real`
- Uses definite description (`THE`) to pick the unique value
- `has_integral` is a predicate relating functions to their integral values

**Philosophy:** Integration is defined through predicates and definite descriptions rather than as a distinct type.

---

### 5. SymPy (Python)

**Representation:** Expression Class (closest to a "type")

```python
from sympy import Integral, Symbol, sin

x = Symbol('x')

# Create unevaluated integral
expr = Integral(sin(x), (x, 0, pi))

# expr is an Integral object (a subclass of Expr)
print(type(expr))  # <class 'sympy.integrals.integrals.Integral'>

# Evaluate it
result = expr.doit()  # Returns: 2
```

**Key Points:**
- `Integral` IS a class (closest to a "type" in this survey)
- Inherits from `Expr` (expression base class)
- Stores: `function`, `limits` (list of (var, lower, upper) tuples)
- Has `.doit()` method to evaluate
- Can remain unevaluated as a symbolic expression

**This is the closest to what Kleis might want!**

**Source:** [SymPy Documentation - Integrals](https://docs.sympy.org/latest/modules/integrals/integrals.html)

---

### 6. SageMath

**Representation:** Symbolic Function Classes

```python
from sage.symbolic.integration.integral import DefiniteIntegral

# SageMath has DefiniteIntegral and IndefiniteIntegral classes
# These are symbolic function classes, not types in the type-theoretic sense
```

**Key Points:**
- Has `DefiniteIntegral` class for representation
- More about symbolic manipulation than type theory
- Similar approach to SymPy

**Source:** [SageMath Documentation](https://doc.sagemath.org/html/en/reference/calculus/sage/symbolic/integration/integral.html)

---

## The Two Paradigms

### Paradigm A: Integration as Function (Coq, Lean, Isabelle)

```
∫ : (Function × Bounds × Proof) → Value
```

- Integral is a **function** that produces a value
- Common in proof assistants
- The "integral" is the result, not the expression
- Requires proof of integrability

**Pro:** Clean type theory, composable
**Con:** Can't represent "unevaluated integral" as an object

### Paradigm B: Integration as Expression (Mathematica, SymPy, SageMath)

```
Integral(f, x, a, b) : Expression
```

- Integral is an **expression/object** that can remain unevaluated
- Common in computer algebra systems
- The integral IS an object you can manipulate
- `.doit()` or `evaluate()` to get the value

**Pro:** Can represent and manipulate unevaluated integrals
**Con:** Less type-theoretically clean

---

## Implications for Kleis

### Option 1: Follow Proof Assistants (Coq/Lean approach)

```kleis
// Integration as a function
operation integrate : (ℝ → ℝ) → ℝ → ℝ → ℝ

// Usage
integrate(f, 0, 1) = value
```

**Pros:**
- Cleaner type theory
- Matches ADR-016 (operations in structures)
- Z3 can reason about the result

**Cons:**
- Can't represent "the integral from 0 to 1" as an object
- Hard to express FTC as "D(∫f) = f" without unevaluated integrals

### Option 2: Follow CAS Systems (SymPy approach)

```kleis
// Integral as a structure
structure DefiniteIntegral(F, var: Variable, lower: ℝ, upper: ℝ) {
    field integrand : F
    operation evaluate : Self → ℝ
    axiom FTC2: ...
}
```

**Pros:**
- Can represent unevaluated integrals
- Matches visual/editor representation
- Can have axioms about the integral expression itself

**Cons:**
- More complex type system
- Not how proof assistants do it

### Option 3: Hybrid (Recommended)

```kleis
// Both an operation (for computation) and a constructor (for representation)

// The operation (returns value)
operation integral : (ℝ → ℝ) → ℝ → ℝ → ℝ

// The expression constructor (for axioms and display)
data IntegralExpr = MkIntegral(integrand: Expr, var: Variable, lower: ℝ, upper: ℝ)

// Relationship
axiom integral_eval: ∀(f, a, b). 
    integral(f, a, b) = evaluate(MkIntegral(f, x, a, b))
```

**Pros:**
- Clean computation via operation
- Rich representation via data type
- Can translate visual AST to `IntegralExpr`
- Z3 can work with both

---

---

## Critical Insight: Bounds Are Expressions

A key observation: even "definite" integrals may have **symbolic bounds**:

```latex
∫₀ⁿ x dx           % upper bound is variable n
∫ₐᵇ f(x) dx        % both bounds are symbolic
∫₀^∞ e^(-x) dx     % upper bound is infinity
∫₀^{x²} f(t) dt    % upper bound is expression x²
∫₀^{g(y)} f(x) dx  % upper bound depends on another variable
```

### Implications for Type Design

**Wrong approach:**
```kleis
structure DefiniteIntegral(F, var: Variable, lower: ℝ, upper: ℝ)
                                             ^^^        ^^^
                                             Too restrictive!
```

**Correct approach:**
```kleis
structure Integral(F) {
    field integrand : F
    field variable : Variable
    field lower : Expr    -- Can be: number, symbol, or expression
    field upper : Expr    -- Can be: number, symbol, ∞, or expression
}
```

### The Spectrum of Integrals

| Type | Lower | Upper | Example |
|------|-------|-------|---------|
| Indefinite | — | — | `∫ x² dx` |
| Definite (numeric) | `ℝ` | `ℝ` | `∫₀¹ x² dx` |
| Definite (symbolic) | `Expr` | `Expr` | `∫ₐᵇ f(x) dx` |
| Improper | `ℝ` | `∞` | `∫₀^∞ e^(-x) dx` |
| Parametric | `ℝ` | `Expr` | `∫₀^t f(x) dx` (FTC!) |

### The Fundamental Theorem Connection

This matters especially for FTC Part 1:

```
F(t) = ∫₀ᵗ f(x) dx    →    F'(t) = f(t)
```

Here `t` is both:
- The **upper bound** of the integral
- The **variable of differentiation** for F

The upper bound MUST be an expression, not just a real number.

### Visual Editor Implication

When the user types `\int_{a}^{b} f(x) dx`:
- `a` and `b` are parsed as **expressions** (Object nodes)
- They might be numbers, variables, or complex expressions
- The translation must preserve this

```
\int_{0}^{n} x dx  
    ↓ parse
sub(sup(\int, Object("n")), Object("0")) * ...
    ↓ translate  
Integral {
    integrand: Object("x"),
    variable: Object("x"),
    lower: Const("0"),      -- Expression: a constant
    upper: Object("n")      -- Expression: a variable
}
```

---

## Recommendation for Kleis

Based on this survey:

1. **Keep `operation Integrate`** as it exists in `derivatives.kleis`
   - This is the Coq/Lean approach for computation

2. **Add `data IntegralExpr`** for representation
   - This enables translation from visual AST
   - Allows axioms about integral expressions themselves
   - **Bounds must be `Expr`, not `ℝ`** to handle symbolic bounds

3. **Define the relationship** between them
   - When bounds are concrete: `evaluate(IntegralExpr(...)) = Integrate(...)`
   - When bounds are symbolic: remains as expression for manipulation

**Proposed definition:**

```kleis
// Integral expression - bounds are expressions (can be symbolic)
data IntegralExpr = MkIntegral {
    integrand : Expr,
    variable  : Variable,
    lower     : Expr,      -- Not ℝ! Can be symbol or expression
    upper     : Expr       -- Not ℝ! Can be symbol, ∞, or expression
}

// Evaluation (when bounds are concrete)
operation evaluate : IntegralExpr → Option(ℝ)

// For FTC: derivative of integral with variable upper bound
axiom FTC1: ∀(f : Expr, x t : Variable, a : Expr).
    D(MkIntegral(f, x, a, Object(t)), t) = substitute(f, x, Object(t))
```

This hybrid approach:
- Maintains compatibility with proof assistant conventions
- Enables the visual editor → Kleis translation
- Gives Z3 something concrete to reason about

---

## References

1. Wolfram Mathematica - [Integrals Documentation](https://www.wolfram.com/language/fast-introduction-for-math-students/integrals/)
2. Coq - [RiemannInt Module](https://rocq-prover.org/doc/V8.16.0/stdlib/Coq.Reals.RiemannInt.html)
3. Lean mathlib - [IntervalIntegral](https://leanprover-community.github.io/mathlib_docs/measure_theory/integral/interval_integral.html)
4. SymPy - [Integral Class](https://docs.sympy.org/latest/modules/integrals/integrals.html)
5. SageMath - [Symbolic Integration](https://doc.sagemath.org/html/en/reference/calculus/sage/symbolic/integration/integral.html)
6. Formalizing Calculus in Coq - [MDPI Paper](https://www.mdpi.com/2227-7390/9/12/1377)
7. Lean Change of Variables - [arXiv](https://arxiv.org/abs/2207.12742)

