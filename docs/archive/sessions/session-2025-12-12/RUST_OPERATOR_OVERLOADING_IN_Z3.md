# Rust Operator Overloading in Z3 Bindings

**Date:** December 12, 2025  
**Question:** Does Rust have operator overloading? (Answer: YES!)

---

## ✅ Rust Has Operator Overloading

**Rust supports operator overloading through traits:**
- `std::ops::Add` for `+`
- `std::ops::Mul` for `*`
- `std::ops::Sub` for `-`
- `std::ops::Div` for `/`
- And many more...

---

## 🔍 Z3 Rust Bindings Use Operator Overloading

The Z3 Rust bindings implement these traits for their AST types.

### Example: Two Equivalent Ways

**Style 1: Operator syntax (uses overloading)**
```rust
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");

let result = &x * &y;           // Uses Mul trait
let sum = &x + &y;              // Uses Add trait
let diff = &x - &y;             // Uses Sub trait
let neg = -&x;                  // Uses Neg trait
```

**Style 2: Explicit function calls (no overloading)**
```rust
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");

let result = Int::mul(&ctx, &[&x, &y]);  // Explicit
let sum = Int::add(&ctx, &[&x, &y]);     // Explicit
let diff = Int::sub(&ctx, &[&x, &y]);    // Explicit
let neg = Int::unary_minus(&x);          // Explicit
```

**Both are valid!** The Z3 bindings provide both APIs.

---

## 📚 How Operator Overloading Works in Rust

### Definition: Implement the trait

```rust
use std::ops::Mul;

struct MyInt(i64);

impl Mul for MyInt {
    type Output = MyInt;
    
    fn mul(self, rhs: MyInt) -> MyInt {
        MyInt(self.0 * rhs.0)
    }
}

// Now you can use:
let a = MyInt(5);
let b = MyInt(3);
let c = a * b;  // Calls the mul() method!
```

### Z3 Does This for Int, Real, Bool, etc.

```rust
// Simplified version of what Z3 bindings do:
impl<'ctx> Mul for &Int<'ctx> {
    type Output = Int<'ctx>;
    
    fn mul(self, rhs: &Int<'ctx>) -> Int<'ctx> {
        // Calls Z3 C API internally
        // Creates multiplication AST node
    }
}
```

---

## 🎯 Which Style to Use?

### In Our Tests: Operator Syntax ✅ (Preferred)

**Why?**
- ✅ More readable: `&x * &y` vs `Int::mul(&ctx, &[&x, &y])`
- ✅ Matches mathematical notation
- ✅ Less verbose
- ✅ Standard Rust idiom

**Example:**
```rust
// Readable
let quadratic = &(&x * &x) + &(&two * &x) + &one;

// vs Verbose
let x_squared = Int::mul(&ctx, &[&x, &x]);
let two_x = Int::mul(&ctx, &[&two, &x]);
let quadratic = Int::add(&ctx, &[&Int::add(&ctx, &[&x_squared, &two_x]), &one]);
```

### In axiom_verifier.rs: Explicit Calls (Current)

**Why?**
- Our code pre-dates operator syntax usage
- Uses older Z3 API style
- Works with context explicitly

**Could be refactored:**
```rust
// Current:
let result = Int::add(&ctx, &[&left, &right]);

// Could be:
let result = &left + &right;
```

---

## 🔧 Why We Need References (&x, &y)

**Ownership in Rust:**
```rust
let x = Int::fresh_const("x");
let y = Int::fresh_const("y");

// This moves x and y (consumes them):
let result = x * y;  // ❌ Can't use x or y afterward!

// This borrows x and y (doesn't consume):
let result = &x * &y;  // ✅ x and y still usable!
```

**The `&` is for borrowing, not pointer arithmetic!**

---

## 📊 Comparison

| Feature | Operator Syntax | Explicit Functions |
|---------|----------------|-------------------|
| **Readability** | ✅ High | ⚠️ Verbose |
| **Rust idiom** | ✅ Standard | ⚠️ Unusual |
| **Type safety** | ✅ Same | ✅ Same |
| **Performance** | ✅ Same (inlined) | ✅ Same |
| **Clarity** | ✅ Math-like | ⚠️ Function calls |

---

## 💡 Examples from Our Tests

### What We Wrote:
```rust
let x_squared = &x * &x;
let x_squared_plus_1 = &x_squared + &one;
```

### What's Actually Happening:
```rust
// Rust compiler translates:
let x_squared = (&x).mul(&x);  // Calls Mul::mul()
let x_squared_plus_1 = (&x_squared).add(&one);  // Calls Add::add()
```

### Which Calls Z3:
```rust
// Z3 Rust bindings implement:
impl Mul for &Int {
    fn mul(self, rhs: &Int) -> Int {
        // Call Z3 C API: Z3_mk_mul(...)
        // Returns new Int AST node
    }
}
```

---

## ✅ Conclusion

**YES, Rust has operator overloading!**

- ✅ Implemented through traits (`std::ops::*`)
- ✅ Z3 Rust bindings use it heavily
- ✅ `&x * &y` is idiomatic Rust
- ✅ More readable than explicit function calls
- ✅ Type-safe and performant

**The `&x * &x` syntax is:**
- NOT C-style pointer arithmetic
- NOT magic or unsafe
- EXACTLY how Rust operator overloading works
- Standard practice in Rust libraries

**Our tests use idiomatic Rust!** ✅

---

## 📖 Further Reading

**Rust traits used:**
- `std::ops::Add` - Addition (`+`)
- `std::ops::Sub` - Subtraction (`-`)
- `std::ops::Mul` - Multiplication (`*`)
- `std::ops::Div` - Division (`/`)
- `std::ops::Neg` - Negation (`-x`)
- `std::ops::Not` - Logical NOT (`!`)

**Z3 implements all of these for Int, Real, Bool, etc.**

