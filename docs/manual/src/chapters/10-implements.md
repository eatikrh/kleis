# Implements

## From Structure to Implementation

A `structure` declares what operations exist. An `implements` block provides the actual definitions:

```kleis
structure Addable(T) {
    operation add : T × T → T
}

implements Addable(ℝ) {
    operation add(x, y) = x + y
}

implements Addable(ℤ) {
    operation add(x, y) = x + y
}
```

## Full Example: Complex Numbers

```kleis
-- Declare the structure
structure Complex {
    field re : ℝ
    field im : ℝ
    
    operation add : Complex → Complex
    operation mul : Complex → Complex
    operation conj : Complex
    operation mag : ℝ
}

-- Implement the operations
implements Complex {
    operation add(z, w) = Complex {
        re = z.re + w.re,
        im = z.im + w.im
    }
    
    operation mul(z, w) = Complex {
        re = z.re * w.re - z.im * w.im,
        im = z.re * w.im + z.im * w.re
    }
    
    operation conj(z) = Complex {
        re = z.re,
        im = -z.im
    }
    
    operation mag(z) = sqrt(z.re^2 + z.im^2)
}
```

## Parametric Implementations

Implement structures with type parameters:

```kleis
structure Stack(T) {
    operation push : T → Stack(T)
    operation pop : Stack(T)
    operation top : T
    operation empty : Bool
}

implements Stack(ℤ) {
    -- Implementation for integer stacks
    operation push(x, s) = Cons(x, s)
    operation pop(s) = match s { Cons(_, rest) => rest }
    operation top(s) = match s { Cons(x, _) => x }
    operation empty(s) = match s { Nil => True, _ => False }
}
```

## Multiple Implementations

The same structure can have multiple implementations:

```kleis
structure Orderable(T) {
    operation compare : T × T → Ordering
}

-- Natural ordering
implements Orderable(ℤ) {
    operation compare(x, y) =
        if x < y then LT
        else if x > y then GT
        else EQ
}

-- Reverse ordering (for max-heaps, etc.)
implements Orderable(ℤ) as ReverseOrder {
    operation compare(x, y) =
        if x > y then LT
        else if x < y then GT
        else EQ
}
```

## Implementing Extended Structures

When a structure extends another, implement all operations:

```kleis
structure Monoid(M) {
    operation e : M
    operation mul : M × M → M
}

structure Group(G) extends Monoid(G) {
    operation inv : G → G
}

-- Must implement both Monoid and Group operations
implements Group(ℤ) {
    operation e = 0
    operation mul(x, y) = x + y
    operation inv(x) = -x
}
```

## Builtin Operations

Some operations can't be defined in pure Kleis — they need native code. The `builtin_` prefix connects Kleis to underlying implementations:

```kleis
implements Matrix(m, n, ℝ) {
    operation transpose = builtin_transpose
    operation add = builtin_matrix_add
    operation mul = builtin_matrix_mul
}
```

### How Builtins Work

When Kleis sees `builtin_foo`, it:
1. Looks up `foo` in the native runtime
2. Calls the Rust/C/hardware implementation
3. Returns the result to Kleis

This enables:
- **Performance**: Native BLAS for matrix operations
- **Hardware access**: GPUs, network cards, sensors
- **System calls**: File I/O, networking, threading
- **FFI**: Calling existing libraries

### The Vision: Hardware as Structures

Imagine:

```kleis
structure NetworkInterface(N) {
    operation send : Packet → Result((), Error)
    operation receive : () → Result(Packet, Error)
    
    axiom delivery : ∀ p : Packet . 
        connected → eventually(delivered(p))
}

implements NetworkInterface(EthernetCard) {
    operation send = builtin_eth_send
    operation receive = builtin_eth_receive
}
```

The **axioms** define the contract. The **builtins** provide the implementation. Z3 can verify that higher-level protocols satisfy their specifications *given* the hardware axioms.

This is how Kleis becomes a **universal verification platform** — not just for math, but for any system with verifiable properties.

## Verification of Implementations

Kleis + Z3 can verify that implementations satisfy axioms:

```kleis
structure Monoid(M) {
    operation e : M
    operation mul : M × M → M
    
    axiom identity : ∀ x : M . mul(e, x) = x ∧ mul(x, e) = x
    axiom associative : ∀ x : M . ∀ y : M . ∀ z : M .
        mul(mul(x, y), z) = mul(x, mul(y, z))
}

implements Monoid(String) {
    operation e = ""
    operation mul(s1, s2) = concat(s1, s2)
}

-- Kleis can verify:
-- 1. concat("", s) = s for all s ✓
-- 2. concat(s, "") = s for all s ✓
-- 3. concat(concat(a, b), c) = concat(a, concat(b, c)) ✓
```

## What's Next?

Learn about Z3 verification in depth!

→ [Next: Z3 Verification](./11-z3-verification.md)
