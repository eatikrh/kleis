# Bit-Vectors

Kleis provides support for **bit-vectors**—fixed-width sequences of bits. Bit-vectors are essential for hardware verification, cryptography, and low-level systems programming.

## Bourbaki Definition

Following Bourbaki's rigorous style, a bit-vector of width n is defined as:

> A **bit-vector** of width n is a mapping x : [0, n-1] → {0, 1}

Equivalently, it's a family (xᵢ)_{i∈[0,n-1]} where each xᵢ ∈ {0, 1}.

## The BitVec Type

```kleis
define byte : BitVec(8) = bvzero(8)
define word : BitVec(32) = bvzero(32)
define qword : BitVec(64) = bvzero(64)
```

## Mother Structures

Bit-vectors inherit three fundamental algebraic structures:

### 1. Vector Space over 𝔽₂

The set BitVec(n) forms a vector space over the two-element field 𝔽₂ = {0, 1}:

```kleis
structure VectorSpaceF2 {
    // XOR is the addition operation
    axiom add_commutative : ∀(n : ℕ)(x y : BitVec(n)).
        bvxor(x, y) = bvxor(y, x)
    
    axiom add_associative : ∀(n : ℕ)(x y z : BitVec(n)).
        bvxor(bvxor(x, y), z) = bvxor(x, bvxor(y, z))
    
    axiom add_identity : ∀(n : ℕ)(x : BitVec(n)).
        bvxor(x, bvzero(n)) = x
    
    // Every element is its own additive inverse!
    axiom add_inverse : ∀(n : ℕ)(x : BitVec(n)).
        bvxor(x, x) = bvzero(n)
}
```

### 2. Boolean Algebra

With AND, OR, and NOT, bit-vectors form a Boolean algebra:

```kleis
structure BooleanAlgebra {
    // De Morgan's laws
    axiom demorgan_and : ∀(n : ℕ)(x y : BitVec(n)).
        bvnot(bvand(x, y)) = bvor(bvnot(x), bvnot(y))
    
    axiom demorgan_or : ∀(n : ℕ)(x y : BitVec(n)).
        bvnot(bvor(x, y)) = bvand(bvnot(x), bvnot(y))
    
    // Complement laws
    axiom and_complement : ∀(n : ℕ)(x : BitVec(n)).
        bvand(x, bvnot(x)) = bvzero(n)
    
    axiom or_complement : ∀(n : ℕ)(x : BitVec(n)).
        bvor(x, bvnot(x)) = bvones(n)
    
    // Distributive law
    axiom distribute : ∀(n : ℕ)(x y z : BitVec(n)).
        bvand(x, bvor(y, z)) = bvor(bvand(x, y), bvand(x, z))
}
```

### 3. Ordered Set

Bit-vectors are totally ordered (both unsigned and signed):

```kleis
structure TotalOrder {
    // Trichotomy: exactly one of <, =, > holds
    axiom trichotomy : ∀(n : ℕ)(x y : BitVec(n)).
        bvult(x, y) ∨ x = y ∨ bvult(y, x)
    
    // Transitivity
    axiom transitive : ∀(n : ℕ)(x y z : BitVec(n)).
        bvult(x, y) ∧ bvult(y, z) → bvult(x, z)
}
```

## Bitwise Operations

### Logical Operations

```kleis
structure BitwiseLogic {
    // AND: set intersection on bit positions
    axiom and_idempotent : ∀(n : ℕ)(x : BitVec(n)). bvand(x, x) = x
    
    // OR: set union on bit positions  
    axiom or_idempotent : ∀(n : ℕ)(x : BitVec(n)). bvor(x, x) = x
    
    // XOR: symmetric difference
    axiom xor_self : ∀(n : ℕ)(x : BitVec(n)). bvxor(x, x) = bvzero(n)
    
    // NOT: complement
    axiom not_involution : ∀(n : ℕ)(x : BitVec(n)). bvnot(bvnot(x)) = x
}
```

### Available Operations

| Operation | Syntax | Description |
|-----------|--------|-------------|
| AND | `bvand(x, y)` | Bitwise AND |
| OR | `bvor(x, y)` | Bitwise OR |
| XOR | `bvxor(x, y)` | Bitwise XOR |
| NOT | `bvnot(x)` | Bitwise complement |

## Arithmetic Operations

Bit-vector arithmetic is **modular** (mod 2ⁿ):

```kleis
structure ModularArithmetic {
    // Addition wraps around
    axiom add_commutative : ∀(n : ℕ)(x y : BitVec(n)).
        bvadd(x, y) = bvadd(y, x)
    
    axiom add_zero : ∀(n : ℕ)(x : BitVec(n)).
        bvadd(x, bvzero(n)) = x
    
    // Two's complement negation
    axiom neg_inverse : ∀(n : ℕ)(x : BitVec(n)).
        bvadd(x, bvneg(x)) = bvzero(n)
    
    // Multiplication distributes
    axiom mul_distribute : ∀(n : ℕ)(x y z : BitVec(n)).
        bvmul(x, bvadd(y, z)) = bvadd(bvmul(x, y), bvmul(x, z))
}
```

### Available Operations

| Operation | Syntax | Description |
|-----------|--------|-------------|
| Add | `bvadd(x, y)` | Addition mod 2ⁿ |
| Subtract | `bvsub(x, y)` | Subtraction mod 2ⁿ |
| Multiply | `bvmul(x, y)` | Multiplication mod 2ⁿ |
| Negate | `bvneg(x)` | Two's complement negation |
| Unsigned div | `bvudiv(x, y)` | Unsigned division |
| Signed div | `bvsdiv(x, y)` | Signed division |
| Unsigned rem | `bvurem(x, y)` | Unsigned remainder |

## Shift Operations

```kleis
structure ShiftOps {
    // Left shift: multiply by 2ᵏ
    axiom shl_zero : ∀(n : ℕ)(x : BitVec(n)).
        bvshl(x, bvzero(n)) = x
    
    // Logical right shift: divide by 2ᵏ (zero fill)
    axiom lshr_zero : ∀(n : ℕ)(x : BitVec(n)).
        bvlshr(x, bvzero(n)) = x
    
    // Arithmetic right shift: divide by 2ᵏ (sign extend)
    axiom ashr_zero : ∀(n : ℕ)(x : BitVec(n)).
        bvashr(x, bvzero(n)) = x
}
```

| Operation | Syntax | Description |
|-----------|--------|-------------|
| Left shift | `bvshl(x, k)` | Shift left by k bits |
| Logical right | `bvlshr(x, k)` | Shift right, zero fill |
| Arithmetic right | `bvashr(x, k)` | Shift right, sign extend |

## Comparison Operations

### Unsigned Comparisons

```kleis
structure UnsignedCompare {
    axiom zero_minimum : ∀(n : ℕ)(x : BitVec(n)).
        bvule(bvzero(n), x)
    
    axiom ones_maximum : ∀(n : ℕ)(x : BitVec(n)).
        bvule(x, bvones(n))
}
```

### Signed Comparisons

```kleis
structure SignedCompare {
    // In two's complement, high bit indicates negative
    axiom signed_negative : ∀(n : ℕ)(x : BitVec(n)).
        bit(x, n - 1) = 1 → bvslt(x, bvzero(n))
}
```

| Unsigned | Signed | Description |
|----------|--------|-------------|
| `bvult(x, y)` | `bvslt(x, y)` | Less than |
| `bvule(x, y)` | `bvsle(x, y)` | Less or equal |
| `bvugt(x, y)` | `bvsgt(x, y)` | Greater than |
| `bvuge(x, y)` | `bvsge(x, y)` | Greater or equal |

## Construction and Extraction

### Constants

```kleis
structure Constants {
    // Zero vector (all bits 0)
    axiom zero_all : ∀(n : ℕ)(i : ℕ). i < n → bit(bvzero(n), i) = 0
    
    // Ones vector (all bits 1)
    axiom ones_all : ∀(n : ℕ)(i : ℕ). i < n → bit(bvones(n), i) = 1
    
    // Single 1 in lowest position
    axiom one_bit : bit(bvone(8), 0) = 1
}
```

### Bit Extraction

```kleis
structure BitExtraction {
    // Get individual bit
    axiom bit_range : ∀(n : ℕ)(x : BitVec(n))(i : ℕ). 
        i < n → (bit(x, i) = 0 ∨ bit(x, i) = 1)
    
    // Extract slice [high:low]
    axiom extract_width : ∀(n high low : ℕ)(x : BitVec(n)).
        high ≥ low ∧ high < n → width(extract(high, low, x)) = high - low + 1
}
```

### Extension

```kleis
structure Extension {
    // Zero extension (for unsigned)
    axiom zext_preserves : ∀(n m : ℕ)(x : BitVec(n)).
        m ≥ n → bvult(x, bvzero(n)) = bvult(zext(m, x), bvzero(m))
    
    // Sign extension (for signed)
    axiom sext_preserves : ∀(n m : ℕ)(x : BitVec(n)).
        m ≥ n → bvslt(x, bvzero(n)) = bvslt(sext(m, x), bvzero(m))
}
```

## Z3 Verification

Kleis maps bit-vector operations directly to Z3's native BitVec theory:

```kleis
structure Z3BitVecExample {
    // XOR properties verified by Z3
    axiom xor_cancel : ∀(n : ℕ)(x : BitVec(n)). bvxor(x, x) = bvzero(n)
    
    // De Morgan verified
    axiom demorgan : ∀(n : ℕ)(x y : BitVec(n)).
        bvnot(bvand(x, y)) = bvor(bvnot(x), bvnot(y))
    
    // Arithmetic properties
    axiom add_neg : ∀(n : ℕ)(x : BitVec(n)). bvadd(x, bvneg(x)) = bvzero(n)
}
```

## Example: Cryptographic Rotation

```kleis
structure RotateExample {
    // Left rotation by k bits
    define rotl(n : ℕ, x : BitVec(n), k : BitVec(n)) : BitVec(n) =
        bvor(bvshl(x, k), bvlshr(x, bvsub(bvone(n) * n, k)))
    
    // Right rotation by k bits
    define rotr(n : ℕ, x : BitVec(n), k : BitVec(n)) : BitVec(n) =
        bvor(bvlshr(x, k), bvshl(x, bvsub(bvone(n) * n, k)))
    
    // Rotation is its own inverse
    axiom rotate_inverse : ∀(n k : ℕ)(x : BitVec(n)).
        rotr(n, rotl(n, x, k), k) = x
}
```

## Example: Bit Manipulation

```kleis
structure BitManipulation {
    // Set bit i to 1
    define set_bit(n : ℕ, x : BitVec(n), i : BitVec(n)) : BitVec(n) =
        bvor(x, bvshl(bvone(n), i))
    
    // Clear bit i to 0
    define clear_bit(n : ℕ, x : BitVec(n), i : BitVec(n)) : BitVec(n) =
        bvand(x, bvnot(bvshl(bvone(n), i)))
    
    // Toggle bit i
    define toggle_bit(n : ℕ, x : BitVec(n), i : BitVec(n)) : BitVec(n) =
        bvxor(x, bvshl(bvone(n), i))
    
    // Test if bit i is set
    define test_bit(n : ℕ, x : BitVec(n), i : BitVec(n)) : Bool =
        bvand(x, bvshl(bvone(n), i)) ≠ bvzero(n)
}
```

## Summary

| Category | Operations |
|----------|------------|
| **Bitwise** | `bvand`, `bvor`, `bvxor`, `bvnot` |
| **Arithmetic** | `bvadd`, `bvsub`, `bvmul`, `bvneg`, `bvudiv`, `bvsdiv`, `bvurem` |
| **Shift** | `bvshl`, `bvlshr`, `bvashr` |
| **Unsigned compare** | `bvult`, `bvule`, `bvugt`, `bvuge` |
| **Signed compare** | `bvslt`, `bvsle`, `bvsgt`, `bvsge` |
| **Construction** | `bvzero`, `bvones`, `bvone`, `extract`, `zext`, `sext` |

See `stdlib/bitvector.kleis` for the complete axiom set.

## What's Next?

Learn about string operations and Z3's string theory:

→ [Strings](17-strings.md)

