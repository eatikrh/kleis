# Dependent Types: Currency Example

**Date:** December 6, 2024  
**Question:** How to express: "Can't add Money with different currencies"  
**Answer:** Make currency part of the type!

---

## The Problem

```kleis
structure Money {
    amount : ℝ
    currency : String
}

operation (+) : Money × Money → Money

// This would allow:
usd = Money { amount: 10, currency: "USD" }
eur = Money { amount: 5, currency: "EUR" }
total = usd + eur  // ❌ Should be ERROR! Can't add USD + EUR
```

**Problem:** `Money` type doesn't track currency, so type checker can't catch this error!

---

## Solution 1: Phantom Type Parameter (Recommended)

### Make Currency Part of the Type

```kleis
// Currency is a type parameter!
structure Money(C) {
    amount : ℝ
    currency : C  // C is the currency type, not a string
}

// Define currency types
structure USD { }
structure EUR { }
structure JPY { }

// Addition only works for SAME currency
structure Additive(A) {
    operation (+) : A × A → A
}

implements Additive(Money(C)) {
    operation (+)(m1, m2) = Money(C) {
        amount: m1.amount + m2.amount,
        currency: m1.currency  // Same currency type!
    }
}

// Now usage:
define usd1 : Money(USD) = Money { amount: 10, currency: USD }
define usd2 : Money(USD) = Money { amount: 5, currency: USD }
define total_usd = usd1 + usd2  // ✅ OK: Money(USD) + Money(USD) → Money(USD)

define eur1 : Money(EUR) = Money { amount: 10, currency: EUR }
define bad = usd1 + eur1  // ❌ TYPE ERROR!
// Error: Cannot add Money(USD) to Money(EUR)
// Types don't match: USD ≠ EUR
```

**Key:** The currency becomes a **type parameter**, not a value!

---

## Solution 2: Dependent Types (Advanced)

### Value-Dependent Types

```kleis
// Type depends on the VALUE of currency
structure Money {
    currency : Currency  // This is a value
    amount : ℝ
    
    // Type refinement: two Money values have same currency
    axiom same_currency_for_add:
        ∀ (m1 m2 : Money) .
            (m1 + m2) requires (m1.currency = m2.currency)
}

operation (+) : Money × Money → Money
    where currency(left) = currency(right)
    
// Now:
usd1 = Money { currency: USD, amount: 10 }
usd2 = Money { currency: USD, amount: 5 }
total = usd1 + usd2  // ✅ OK: currencies match

eur1 = Money { currency: EUR, amount: 10 }
bad = usd1 + eur1  // ❌ ERROR!
// Type check: currency(usd1) = USD, currency(eur1) = EUR
// Constraint: USD = EUR fails!
```

**This requires dependent type system** - types can depend on values.

---

## Solution 3: Separate Types Per Currency (Simple)

### Each Currency is Its Own Type

```kleis
structure USD_Money {
    amount : ℝ
}

structure EUR_Money {
    amount : ℝ
}

structure JPY_Money {
    amount : ℝ
}

// Each has its own addition
implements Additive(USD_Money)
implements Additive(EUR_Money)
implements Additive(JPY_Money)

// Now:
usd1 : USD_Money = { amount: 10 }
usd2 : USD_Money = { amount: 5 }
total = usd1 + usd2  // ✅ OK: both USD_Money

eur1 : EUR_Money = { amount: 10 }
bad = usd1 + eur1  // ❌ TYPE ERROR!
// Error: Cannot add USD_Money to EUR_Money
```

**Pros:** Simple, catches errors at type level  
**Cons:** Repetitive, can't write generic code

---

## Solution 4: Runtime Check in Axiom (Compromise)

### Type System + Runtime Verification

```kleis
structure Money {
    amount : ℝ
    currency : String
}

operation (+) : Money × Money → Money

// Axiom enforces constraint
axiom add_same_currency:
    ∀ (m1 m2 : Money) .
        (m1 + m2).currency = m1.currency ∧
        m1.currency = m2.currency

// Type checker can't prevent mixing currencies
// But axiom verification will catch it:
usd1 = Money { amount: 10, currency: "USD" }
eur1 = Money { amount: 5, currency: "EUR" }
bad = usd1 + eur1  
// ⚠️ Type checks but AXIOM VIOLATION!
// Warning: axiom add_same_currency may be violated
```

**Pros:** Flexible, can still type check  
**Cons:** Runtime check, not compile-time safety

---

## Comparison

| Approach | Type Safety | Complexity | When to Use |
|----------|-------------|------------|-------------|
| **Phantom types** | ✅ Compile-time | Medium | **Recommended** |
| **Dependent types** | ✅ Compile-time | High | If needed elsewhere |
| **Separate types** | ✅ Compile-time | Low | Quick solution |
| **Runtime axioms** | ⚠️ Runtime | Low | When type system can't express constraint |

---

## Recommended: Phantom Types

### How to Implement in Kleis

**Grammar already supports this!**

```kleis
structure Money(C) {  // C is type parameter
    amount : ℝ
}

// Specific currency types
structure USD { }
structure EUR { }

// Money(USD) and Money(EUR) are DIFFERENT types!
```

**Type system sees:**
```
Money(USD) ≠ Money(EUR)  // Different types!

Money(USD) + Money(USD) → Money(USD)  ✅
Money(USD) + Money(EUR) → TYPE ERROR  ❌
```

---

## Full Example: Phantom Types

```kleis
// Currency types (empty structures used as type tags)
structure USD { }
structure EUR { }
structure JPY { }

// Money parameterized by currency
structure Money(C) {
    amount : ℝ
}

// Addition only for same currency
structure Additive(A) {
    operation (+) : A × A → A
}

implements Additive(Money(C)) {
    // C is the same in both arguments!
    operation (+)(m1 : Money(C), m2 : Money(C)) -> Money(C) = 
        Money(C) { amount: m1.amount + m2.amount }
}

// Conversion between currencies
operation convert : ∀C₁ C₂. Money(C₁) × ExchangeRate(C₁, C₂) → Money(C₂)

// Usage:
define usd1 : Money(USD) = Money { amount: 100.0 }
define usd2 : Money(USD) = Money { amount: 50.0 }
define total_usd = usd1 + usd2  // ✅ Type: Money(USD)

define eur1 : Money(EUR) = Money { amount: 100.0 }
define bad = usd1 + eur1  // ❌ TYPE ERROR!
// Error: Cannot add Money(USD) to Money(EUR)
// Type mismatch: USD ≠ EUR

// Correct way to mix currencies:
define rate : ExchangeRate(EUR, USD) = ...
define eur_as_usd = convert(eur1, rate)  // Money(EUR) → Money(USD)
define correct = usd1 + eur_as_usd  // ✅ OK: both Money(USD)
```

**Benefits:**
1. ✅ Type system prevents currency mixing
2. ✅ Forces explicit conversion
3. ✅ No runtime checks needed
4. ✅ Generic code possible: `total<C>(items: List(Money(C))) → Money(C)`

---

## Implementation in Current System

### The grammar already supports type parameters!

```antlr
// From Kleis_v03.g4 line 45
structureDef
    : 'structure' IDENTIFIER '(' typeParams ')'
```

**We have this in the grammar!**

**Our parser currently skips them** (we parse `structure Numeric(N)` but ignore `N`).

### To Fully Support

**Need to:**
1. Store type parameters in StructureDef
2. Use them in type checking
3. Unify type parameters (USD ≠ EUR)

**Estimated:** 1-2 days to add proper type parameter support

---

## Quick Fix for POC

### For now, use axiom approach:

```kleis
structure Money {
    amount : ℝ
    currency : String
    
    axiom add_requires_same_currency:
        ∀ (m1 m2 : Money) .
            valid(m1 + m2) ⟹ m1.currency = m2.currency
}
```

**Type system won't prevent it, but documentation/verification can check it.**

---

## Recommendation

**For Money specifically:** Use phantom types

```kleis
structure Money(C) { amount : ℝ }
structure USD { }
structure EUR { }

// Money(USD) and Money(EUR) are different types!
// Type system prevents mixing automatically
```

**Benefit:** Leverages existing type parameter support in grammar.

**Implementation:** Need to properly parse and use type parameters (not skipping them).

---

**Status:** ⚠️ **Need Type Parameter Support**  
**Workaround:** Use axioms for now  
**Proper Solution:** Phantom types (1-2 days work)  
**Grammar:** ✅ Already supports it!

