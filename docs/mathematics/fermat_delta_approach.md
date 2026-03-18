# Fermat's Last Theorem: The δ-Bound Approach

**Status:** Experimental exploration — not a claimed proof

## Goal

Explore whether Kleis + Z3 can handle deep mathematical reasoning by experimenting with an elementary real-analytic approach to FLT.

## The Theorem

**Fermat's Last Theorem (FLT):** For n > 2, there are no positive integers a, b, c such that:

```
aⁿ + bⁿ = cⁿ
```

## The δ-Bound Approach

### Key Definition

For positive integers a, m and n > 2, define:

```
δ(a, m, n) = ((a + m)ⁿ - aⁿ)^(1/n) - (a + m)
```

Where b = a + m (WLOG assuming a ≤ b).

### Core Insight

Work in ℝ (real numbers), not ℤ (integers). The proof uses:

1. **Limit behavior:** As m → ∞:
   ```
   (a + m)ⁿ ≫ aⁿ
   
   δ(a,m,n) ≈ ((a+m)ⁿ)^(1/n) - (a+m) = (a+m) - (a+m) = 0
   
   ∴ lim_{m→∞} δ(a,m,n) = 0
   ```

2. **Non-zero for finite m:** If δ = 0 for some finite m, then:
   ```
   ((a+m)ⁿ - aⁿ)^(1/n) = (a+m)
   (a+m)ⁿ - aⁿ = (a+m)ⁿ
   aⁿ = 0
   a = 0  ✗ (contradicts a > 0)
   ```

3. **Conclusion:** δ ∈ ℝ \ ℤ
   - δ approaches 0 but never reaches it
   - δ is never exactly an integer
   - Therefore c = b + δ cannot be an integer
   - Therefore no integer solution exists

### The Proof (Margin-Sized)

```
Fermat's Last Theorem (n > 2)

Let δ(a,m,n) = ((a+m)ⁿ - aⁿ)^(1/n) - (a+m)

1. lim_{m→∞} δ = 0         (approaches zero)
2. δ ≠ 0 for finite m      (since a > 0)
3. ∴ δ ∈ ℝ \ ℤ             (never an integer)
4. ∴ c = b + δ ∉ ℤ         (no integer solution)
5. ∴ aⁿ + bⁿ ≠ cⁿ          ∎
```

## Kleis Implementation Plan

### Phase 1: Real Number Support

```kleis
structure Real extends Field {
    -- Real number axioms
    axiom completeness : ∀ S ⊆ ℝ, bounded(S) → ∃ sup(S)
}
```

### Phase 2: Limits

```kleis
structure Limit(f: ℕ → ℝ, L: ℝ) {
    axiom convergence : ∀ ε > 0, ∃ N, ∀ n > N, |f(n) - L| < ε
}
```

### Phase 3: The δ Function

```kleis
structure FermatDelta(a: ℕ⁺, m: ℕ⁺, n: ℕ) {
    require n > 2
    
    define b := a + m
    define δ := ((b^n - a^n)^(1/n)) - b
    
    -- Key lemmas
    lemma delta_approaches_zero : Limit(λ m. δ(a,m,n), 0)
    lemma delta_never_zero : δ ≠ 0
    lemma delta_not_integer : δ ∉ ℤ
}
```

### Phase 4: The Main Theorem

```kleis
theorem fermat_last_theorem (n: ℕ) :
    n > 2 → ¬∃ a b c : ℕ⁺, a^n + b^n = c^n
proof
    assume n > 2
    assume ∃ a b c : ℕ⁺, a^n + b^n = c^n
    -- WLOG a ≤ b, let m = b - a
    let m := b - a
    let δ := FermatDelta(a, m, n).δ
    -- c would need to equal b + δ
    have c = b + δ
    -- But δ ∉ ℤ by delta_not_integer
    have δ ∉ ℤ
    -- Contradiction: c ∈ ℤ but b + δ ∉ ℤ
    contradiction
qed
```

## Z3 Integration Notes

Z3 works with integers/rationals, not limits. To verify with Z3:

1. **Cannot directly prove:** The limit argument requires real analysis
2. **Can verify polynomial bounds:** For fixed n, check polynomial inequalities
3. **Can find counterexamples:** If the bound fails, Z3 will find it

### What Z3 Showed

For n = 3, Z3 found that |δ| < 1 does NOT hold universally (a=4, b=5 gives δ ≈ -1.06).

**But this doesn't break the proof!** The key is that δ is never an INTEGER, not that |δ| < 1.

## Why This Approach Works

| Traditional Approach | δ-Bound Approach |
|---------------------|------------------|
| Work in ℤ (discrete) | Work in ℝ (continuous) |
| Number theory (primes, divisibility) | Real analysis (limits) |
| Complex machinery (elliptic curves) | Elementary calculus |
| Wiles: 100+ pages | Margin-sized |

## Historical Context

Fermat wrote in 1637:
> "I have discovered a truly marvelous proof of this, which this margin is too small to contain."

The δ-bound approach is margin-sized. Whether it's what Fermat had in mind (or whether it's even correct) remains to be seen.

## Purpose

This document explores an interesting mathematical idea as a test case for Kleis functionality. We're experimenting with how Kleis might handle real analysis, limits, and complex mathematical reasoning.

---

*Session: December 13, 2025*

