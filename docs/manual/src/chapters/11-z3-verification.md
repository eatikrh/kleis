# Z3 Verification

## What is Z3?

[Z3](https://github.com/Z3Prover/z3) is a theorem prover from Microsoft Research. Kleis uses Z3 to:

- **Verify** mathematical statements
- **Find counterexamples** when statements are false
- **Check** that implementations satisfy axioms

## Basic Verification

Use `verify` to check a statement:

```kleis
verify x + y = y + x
-- Result: ✓ Valid (commutativity of addition)

verify x * 0 = 0
-- Result: ✓ Valid

verify x > 0
-- Result: ✗ Invalid
-- Counterexample: x = -1
```

## Verifying Quantified Statements

Z3 handles universal and existential quantifiers:

```kleis
verify ∀ x : ℝ . x + 0 = x
-- Result: ✓ Valid

verify ∀ x : ℝ . x * x ≥ 0
-- Result: ✓ Valid (squares are non-negative)

verify ∃ x : ℝ . x * x = -1
-- Result: ✗ Invalid (no real square root of -1)

verify ∃ x : ℂ . x * x = -1
-- Result: ✓ Valid (x = i works)
```

## Checking Axioms

Verify that definitions satisfy axioms:

```kleis
structure Group(G) {
    operation e : G
    operation mul : G × G → G
    operation inv : G → G
    
    axiom identity : ∀ x : G . mul(e, x) = x
    axiom inverse : ∀ x : G . mul(x, inv(x)) = e
    axiom associative : ∀ x : G . ∀ y : G . ∀ z : G .
        mul(mul(x, y), z) = mul(x, mul(y, z))
}

-- Define integers with addition
implements Group(ℤ) {
    operation e = 0
    operation mul(x, y) = x + y
    operation inv(x) = -x
}

-- Kleis verifies each axiom automatically!
```

## Implication Verification

Prove that premises imply conclusions:

```kleis
-- If x > 0 and y > 0, then x + y > 0
verify (x > 0 ∧ y > 0) → x + y > 0
-- Result: ✓ Valid

-- Triangle inequality
verify (abs(x) ≤ a ∧ abs(y) ≤ b) → abs(x + y) ≤ a + b
-- Result: ✓ Valid
```

## Counterexamples

When verification fails, Z3 provides counterexamples:

```kleis
verify ∀ x : ℝ . x^2 = x
-- Result: ✗ Invalid
-- Counterexample: x = 2 (since 4 ≠ 2)

verify ∀ n : ℕ . n > 0 → n > 1
-- Result: ✗ Invalid
-- Counterexample: n = 1
```

## Timeout and Limits

Complex statements may time out:

```kleis
-- Very complex statement
verify ∀ M : Matrix(100, 100) . det(M * M') ≥ 0
-- Result: ⏱ Timeout (statement too complex)
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
-- Step 1: Define structure
structure Ring(R) {
    operation zero : R
    operation one : R
    operation add : R × R → R
    operation mul : R × R → R
    operation neg : R → R
    
    axiom add_assoc : ∀ a : R . ∀ b : R . ∀ c : R .
        add(add(a, b), c) = add(a, add(b, c))
    // ... more axioms
}

-- Step 2: Implement for integers
implements Ring(ℤ) {
    operation zero = 0
    operation one = 1
    operation add(x, y) = x + y
    operation mul(x, y) = x * y
    operation neg(x) = -x
}

-- Step 3: Auto-verification happens!

-- Step 4: Check additional properties
verify ∀ x : ℤ . mul(x, zero) = zero
-- Result: ✓ Valid
```

## What's Next?

Try the interactive REPL!

→ [Next: The REPL](./12-repl.md)
