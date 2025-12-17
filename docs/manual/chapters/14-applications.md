# Chapter 14: Building Applications

[← Previous: The REPL](13-repl.md) | [Back to Contents](../index.md)

---

## File Organization

A typical Kleis project structure:

```
my-project/
├── src/
│   ├── core.kleis      # Core definitions
│   ├── algebra.kleis   # Algebraic structures
│   └── theorems.kleis  # Theorems and proofs
├── tests/
│   └── test_algebra.kleis
├── examples/
│   └── demo.kleis
└── README.md
```

---

## Writing Kleis Files

### File Structure

```kleis
// algebra.kleis - Algebraic structures

// 1. Data types first
data Bool = True | False
data Option(T) = None | Some(T)

// 2. Structures
structure Monoid(M) {
    operation (*) : M → M → M
    operation e : M
    
    axiom assoc : ∀(x y z : M). (x * y) * z = x * (y * z)
    axiom identity : ∀(x : M). e * x = x ∧ x * e = x
}

// 3. Implementations
implements Monoid(ℤ) {
    define (*)(a, b) = a + b
    define e = 0
}

// 4. Functions
define fold(f, init, xs) =
    match xs {
        Nil => init
        Cons(h, t) => f(h, fold(f, init, t))
    }
```

### Comments and Documentation

```kleis
/*
 * Module: LinearAlgebra
 * 
 * Provides vector and matrix operations for
 * n-dimensional spaces over real numbers.
 */

// Vector dot product
// Returns the scalar product of two vectors
define dot(u : Vector(n), v : Vector(n)) : ℝ =
    sum(zipWith((*), u, v))
```

---

## Real-World Example: Physics

```kleis
// physics.kleis - Classical mechanics

// Constants
define c : ℝ = 299792458         // Speed of light (m/s)
define G : ℝ = 6.67430e-11       // Gravitational constant

// Structures
structure Particle {
    mass : ℝ
    position : Vector(3)
    velocity : Vector(3)
}

// Kinetic energy
define kinetic_energy(p : Particle) : ℝ =
    0.5 * p.mass * dot(p.velocity, p.velocity)

// Gravitational force between two particles
define gravity(p1 : Particle, p2 : Particle) : Vector(3) =
    let r = p2.position - p1.position in
    let dist = norm(r) in
    let magnitude = G * p1.mass * p2.mass / (dist * dist) in
    scale(magnitude / dist, r)

// Equations of motion
axiom newton_second_law :
    ∀(p : Particle). ∀(F : Vector(3)).
        p.acceleration = scale(1/p.mass, F)
```

---

## Example: Abstract Algebra

```kleis
// groups.kleis - Group theory

structure Group(G) {
    operation (*) : G → G → G
    operation e : G
    operation inv : G → G
    
    axiom assoc : ∀(a b c : G). (a * b) * c = a * (b * c)
    axiom identity : ∀(a : G). e * a = a ∧ a * e = a
    axiom inverse : ∀(a : G). a * inv(a) = e ∧ inv(a) * a = e
}

// Derived theorems
theorem left_cancel : ∀(G : Group). ∀(a b c : G).
    a * b = a * c ⟹ b = c

theorem right_cancel : ∀(G : Group). ∀(a b c : G).
    b * a = c * a ⟹ b = c

theorem inv_inv : ∀(G : Group). ∀(a : G).
    inv(inv(a)) = a

theorem inv_product : ∀(G : Group). ∀(a b : G).
    inv(a * b) = inv(b) * inv(a)

// Subgroups
structure Subgroup(H, G) where Group(G) {
    // H is a subset of G
    operation embed : H → G
    
    axiom closed : ∀(h1 h2 : H). 
        embed(h1 * h2) = embed(h1) * embed(h2)
}
```

---

## Example: Type-Safe Units

```kleis
// units.kleis - Dimensional analysis

// Base dimensions
data Dimension = Length | Mass | Time | Current | Temperature

// Unit type with dimension
structure Unit(D : Dimension) {
    value : ℝ
}

// Smart constructors
define meters(x : ℝ) : Unit(Length) = Unit { value = x }
define kilograms(x : ℝ) : Unit(Mass) = Unit { value = x }
define seconds(x : ℝ) : Unit(Time) = Unit { value = x }

// Operations preserve dimensions
define add_length(a : Unit(Length), b : Unit(Length)) : Unit(Length) =
    Unit { value = a.value + b.value }

// This would be a type error:
// add_length(meters(5), seconds(3))  // ✗ Type error!

// Derived units
define velocity(d : Unit(Length), t : Unit(Time)) : ℝ =
    d.value / t.value  // Returns m/s as scalar

// Physics formulas
define kinetic_energy_safe(m : Unit(Mass), v : ℝ) : ℝ =
    0.5 * m.value * v * v  // kg * (m/s)² = Joules
```

---

## Testing

```kleis
// test_groups.kleis

// Test integer group
define test_integer_identity() =
    let result = 0 + 5 in
    assert(result = 5, "Identity should work")

define test_integer_inverse() =
    let x = 5 in
    let result = x + (-x) in
    assert(result = 0, "Inverse should give identity")

// Run all tests
define run_tests() = 
    test_integer_identity();
    test_integer_inverse();
    print("All tests passed!")
```

---

## Best Practices

### 1. Start with Types

Define your types and structures before writing functions:

```kleis
// Good: Clear type structure
data Result(T, E) = Ok(T) | Err(E)

define safe_div(a : ℝ, b : ℝ) : Result(ℝ, String) =
    if b = 0 then Err("Division by zero")
    else Ok(a / b)
```

### 2. Use Meaningful Names

```kleis
// Good
define gravitational_force(mass1, mass2, distance) = ...

// Bad
define f(m1, m2, d) = ...
```

### 3. Document Axioms

```kleis
// Axiom: The integral of a constant function
// ∫[a,b] c dx = c(b-a)
axiom integral_constant :
    ∀(c a b : ℝ). integral(const(c), a, b) = c * (b - a)
```

### 4. Verify Incrementally

```kleis
// Step 1: Define and verify Monoid
// Step 2: Extend to Group, verify
// Step 3: Add theorems, prove
```

### 5. Keep Structures Focused

```kleis
// Good: Focused structures
structure Additive(A) { operation (+) : A → A → A }
structure Multiplicative(M) { operation (*) : M → M → M }
structure Ring(R) extends Additive(R), Multiplicative(R) { ... }

// Bad: Kitchen sink structure
structure Everything(X) {
    // 50 operations and 100 axioms...
}
```

---

## Summary

- Organize code into focused files
- Define types → structures → implementations → functions
- Use meaningful names and documentation
- Test and verify as you develop
- Follow mathematical conventions

---

## What's Next?

You've completed the Kleis tutorial! You now know:

- ✅ Basic syntax and expressions
- ✅ Types and type annotations
- ✅ Functions and pattern matching
- ✅ Algebraic data types
- ✅ Structures and axioms
- ✅ Implements, where, extends
- ✅ Z3 verification
- ✅ Practical application development

**Continue learning:**
- Explore the standard library (`stdlib/`)
- Read the architecture decisions (`docs/adr/`)
- Check example files in `examples/`
- Join the community on GitHub

Happy verifying! 🔑

---

[← Previous: The REPL](13-repl.md) | [Back to Contents](../index.md)

