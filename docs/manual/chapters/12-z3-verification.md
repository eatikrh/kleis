# Chapter 12: Z3 and Theorem Proving

[← Previous: Implements](11-implements.md) | [Back to Contents](../index.md) | [Next: The REPL →](13-repl.md)

---

## What is Z3?

**Z3** is a theorem prover from Microsoft Research. It can:

- Determine if mathematical statements are true or false
- Find counterexamples when statements are false
- Solve constraints and equations

Kleis uses Z3 to **verify your axioms** — checking that your mathematical structures are consistent and your claims are valid.

---

## Axiom Verification

When you define axioms, Kleis can verify them:

```kleis
structure Commutative(S) {
    operation (*) : S → S → S
    
    axiom commutativity : ∀(x y : S). x * y = y * x
}

// Kleis + Z3 will check: is this axiom consistent?
// Can we find a model where it holds?
```

### What Z3 Checks

1. **Satisfiability** — Can the axioms all be true simultaneously?
2. **Validity** — Is a statement true in all models?
3. **Counterexamples** — If false, what's a counterexample?

---

## Verification Results

### Verified ✓

```kleis
structure Group(G) {
    operation (*) : G → G → G
    operation e : G
    operation inv : G → G
    
    axiom identity : ∀(x : G). x * e = x
    axiom inverse : ∀(x : G). x * inv(x) = e
}

// Result: ✓ Axioms are consistent
```

### Counterexample Found

```kleis
structure Broken(B) {
    operation f : B → B
    
    // This is false in general!
    axiom wrong : ∀(x : B). f(f(x)) = x
}

// Result: ✗ Counterexample found
// f(x) = x + 1 doesn't satisfy f(f(x)) = x
```

### Timeout / Unknown

Some problems are too hard:

```kleis
// Very complex axiom system...
// Result: ? Unknown (timeout after 10s)
```

---

## Using Verification

### In the REPL

```kleis
>>> verify Group(ℤ)
Verifying Group(ℤ)...
✓ identity: verified
✓ inverse: verified  
✓ associativity: verified
All axioms satisfied!
```

### In Files

```kleis
// my_theory.kleis

structure MyStructure(M) {
    operation f : M → M
    axiom idempotent : ∀(x : M). f(f(x)) = f(x)
}

// Run: kleis verify my_theory.kleis
```

---

## How Z3 Works (Simplified)

1. **Translation**: Kleis converts axioms to SMT-LIB format
2. **Solving**: Z3 searches for models/counterexamples
3. **Result**: SAT (satisfiable), UNSAT (unsatisfiable), or UNKNOWN

### Example Translation

```kleis
// Kleis:
axiom comm : ∀(x y : R). x + y = y + x

// SMT-LIB:
(assert (forall ((x R) (y R)) (= (+ x y) (+ y x))))
```

---

## Proving Theorems

You can ask Z3 to prove derived facts:

```kleis
structure Group(G) { /* ... */ }

// Theorem: In a group, the identity is unique
theorem unique_identity :
    ∀(e1 e2 : G). 
        (∀(x : G). e1 * x = x) ∧ (∀(x : G). e2 * x = x) 
        ⟹ e1 = e2
```

Kleis will attempt to prove this from the group axioms.

---

## Verification Strategies

### 1. Start Simple

```kleis
// First verify basic properties
structure Magma(M) {
    operation (*) : M → M → M
}
// ✓ Trivially satisfiable
```

### 2. Add Axioms Incrementally

```kleis
// Then add associativity
structure Semigroup(S) extends Magma(S) {
    axiom assoc : ∀(x y z : S). (x*y)*z = x*(y*z)
}
// ✓ Still satisfiable
```

### 3. Check Consequences

```kleis
// Verify that expected theorems hold
theorem left_cancel :
    ∀(a b c : G). a*b = a*c ⟹ b = c
// ✓ Follows from group axioms
```

---

## Common Issues

### 1. Underdetermined Systems

```kleis
structure Weak(W) {
    operation f : W → W
    // No axioms!
}
// Everything is consistent, but useless
```

**Fix:** Add meaningful axioms.

### 2. Contradictory Axioms

```kleis
structure Broken(B) {
    operation x : B
    axiom a1 : x = x
    axiom a2 : x ≠ x  // Contradiction!
}
// ✗ Unsatisfiable
```

**Fix:** Remove contradictions.

### 3. Non-Terminating Verification

```kleis
// Highly nonlinear axioms
axiom hard : ∀(x : ℝ). sin(x)^2 + cos(x)^2 = 1
// May timeout
```

**Fix:** Simplify or accept unknown result.

---

## Best Practices

### 1. Verify Early, Verify Often

Don't write 1000 lines then verify. Verify as you go.

### 2. Use Meaningful Names

```kleis
// Good
axiom associativity : ∀(x y z : G). (x*y)*z = x*(y*z)

// Bad
axiom a1 : ∀(x y z : G). (x*y)*z = x*(y*z)
```

### 3. Document What You're Proving

```kleis
// Theorem: Every group element has a unique inverse
// Proof: Assume x*y = e and x*z = e. Then...
theorem unique_inverse : ...
```

### 4. Start with Known Structures

Base your work on well-studied mathematical structures (groups, rings, fields) where you know the axioms are consistent.

---

## Exercises

1. **Verify** that the integers under addition form a group

2. **Write** an inconsistent structure and observe Z3's response

3. **Prove** that in any monoid, `e * e = e`

4. **Check** if `∀(x : ℝ). x * x ≥ 0` can be verified

<details>
<summary>Solutions</summary>

```kleis
// 1.
implements Group(ℤ) {
    define (+)(a, b) = a + b
    define e = 0
    define inv(x) = -x
}
// >>> verify Group(ℤ)
// ✓ All axioms verified

// 2.
structure Inconsistent(I) {
    operation x : I
    operation y : I
    axiom bad : x = y ∧ x ≠ y
}
// ✗ Unsatisfiable - contradiction found

// 3.
theorem identity_idempotent : ∀(M : Monoid). e * e = e
// Proof: e * e = e by right identity axiom
// ✓ Verified

// 4.
// This requires nonlinear arithmetic
// Z3 can verify: x * x ≥ 0 for all real x
// ✓ Verified (squares are non-negative)
```

</details>

---

## Summary

- Z3 is a theorem prover that verifies mathematical statements
- Kleis uses Z3 to check axiom consistency
- Results: verified, counterexample found, or unknown
- Verify incrementally as you develop structures
- Z3 handles first-order logic well; nonlinear math is harder

---

[← Previous: Implements](11-implements.md) | [Back to Contents](../index.md) | [Next: The REPL →](13-repl.md)

