# Z3 Verification

## What is Z3?

[Z3](https://github.com/Z3Prover/z3) is a theorem prover from Microsoft Research. Kleis uses Z3 to:

- **Verify** mathematical statements
- **Find counterexamples** when statements are false
- **Check** that implementations satisfy axioms

## Basic Verification

Use `verify` to check a statement:

```kleis
axiom commutativity : ∀(x : ℝ)(y : ℝ). x + y = y + x
// Z3 verifies: ✓ Valid

axiom zero_annihilates : ∀(x : ℝ). x * 0 = 0
// Z3 verifies: ✓ Valid

axiom all_positive : ∀(x : ℝ). x > 0
// Z3 finds counterexample: x = -1
```

## Verifying Quantified Statements

Z3 handles universal and existential quantifiers:

```kleis
axiom additive_identity : ∀(x : ℝ). x + 0 = x
// Z3 verifies: ✓ Valid

axiom squares_nonnegative : ∀(x : ℝ). x * x ≥ 0
// Z3 verifies: ✓ Valid (squares are non-negative)

axiom no_real_sqrt_neg1 : ∃(x : ℝ). x * x = -1
// Z3: ✗ Invalid (no real square root of -1)

axiom complex_sqrt_neg1 : ∃(x : ℂ). x * x = -1
// Z3 verifies: ✓ Valid (x = i works)
```

## Checking Axioms

Verify that definitions satisfy axioms:

```kleis
structure Group(G) {
    e : G
    operation mul : G × G → G
    operation inv : G → G
    
    axiom identity : ∀(x : G). mul(e, x) = x
    axiom inverse : ∀(x : G). mul(x, inv(x)) = e
    axiom associative : ∀(x : G)(y : G)(z : G).
        mul(mul(x, y), z) = mul(x, mul(y, z))
}

// Define integers with addition
implements Group(ℤ) {
    element e = 0
    operation mul = builtin_add
    operation inv = builtin_negate
}

// Kleis verifies each axiom automatically!
```

## Implication Verification

Prove that premises imply conclusions:

```kleis
// If x > 0 and y > 0, then x + y > 0
axiom sum_positive : ∀(x : ℝ)(y : ℝ). (x > 0 ∧ y > 0) → x + y > 0
// Z3 verifies: ✓ Valid

// Triangle inequality
axiom triangle_ineq : ∀(x : ℝ)(y : ℝ)(a : ℝ)(b : ℝ).
    (abs(x) ≤ a ∧ abs(y) ≤ b) → abs(x + y) ≤ a + b
// Z3 verifies: ✓ Valid
```

## Counterexamples

When verification fails, Z3 provides counterexamples:

```kleis
axiom square_equals_self : ∀(x : ℝ). x^2 = x
// Z3: ✗ Invalid, Counterexample: x = 2 (since 4 ≠ 2)

axiom positive_greater_than_one : ∀(n : ℕ). n > 0 → n > 1
// Z3: ✗ Invalid, Counterexample: n = 1
```

## Timeout and Limits

Complex statements may time out:

```kleis
// Very complex statement
verify ∀ M : Matrix(100, 100) . det(M * M') ≥ 0
// Result: ⏱ Timeout (statement too complex)
```

## What Z3 Can and Cannot Do

### Z3 Excels At:
- Linear arithmetic
- Boolean logic
- Array reasoning
- Simple quantifiers
- Algebraic identities

### Z3 Struggles With:
- Non-linear real arithmetic (undecidable in general)
- Very deep quantifier nesting
- Transcendental functions (sin, cos, exp)
- Infinite structures

## Practical Workflow

1. **Write structure with axioms**
2. **Implement operations**
3. **Kleis auto-verifies** axioms are satisfied
4. **Use `verify`** for additional properties
5. **Examine counterexamples** when verification fails

```kleis
// Step 1: Define structure
structure Ring(R) {
    zero : R
    one : R
    operation add : R × R → R
    operation mul : R × R → R
    operation neg : R → R
    
    axiom add_assoc : ∀(a : R)(b : R)(c : R).
        add(add(a, b), c) = add(a, add(b, c))
}

// Step 2: Implement for integers
implements Ring(ℤ) {
    element zero = 0
    element one = 1
    operation add = builtin_add
    operation mul = builtin_mul
    operation neg = builtin_negate
}

// Step 3: Auto-verification happens!

// Step 4: Check additional properties
axiom mul_zero : ∀(x : ℤ). mul(x, zero) = zero
// Z3 verifies: ✓ Valid
```

## What's Next?

Try the interactive REPL!

→ [Next: The REPL](./12-repl.md)
