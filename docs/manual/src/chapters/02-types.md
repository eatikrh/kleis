# Types and Values

## Why Types Matter

Types are the foundation of Kleis. Every expression has a type, and the type system catches errors before they become problems.

```kleis
define answer = 42                // 42 is an integer
define pi_val = 3.14              // 3.14 is a real number
define flag = True                // True is a boolean
```

## Built-in Types

### Numeric Types

| Type | Unicode | Full Name | ASCII | Examples |
|------|---------|-----------|-------|----------|
| Natural | `ℕ` | `Nat` | `N` | `0`, `42`, `100` |
| Integer | `ℤ` | `Int` | `Z` | `-5`, `0`, `17` |
| Rational | `ℚ` | `Rational` | `Q` | `rational(1, 2)`, `rational(3, 4)` |
| Real | `ℝ` | `Real` or `Scalar` | `R` | `3.14`, `-2.5`, `√2` |
| Complex | `ℂ` | `Complex` | `C` | `3 + 4i`, `i` |

### Other Basic Types

| Type | Unicode | Full Name | Values |
|------|---------|-----------|--------|
| Boolean | `𝔹` | `Bool` | `True`, `False` |
| String | — | `String` | `"hello"`, `"world"` |
| Unit | — | `Unit` | `()` |

### Parameterized Primitive Types

| Type | Syntax | Description |
|------|--------|-------------|
| Bit-Vector | `BitVec(n)` | n-bit binary vector (e.g., `BitVec(8)`, `BitVec(32)`) |
| Set | `Set(T)` | Set of elements of type T (e.g., `Set(ℤ)`, `Set(ℝ)`) |

```kleis
// Boolean values
define flag = True
define not_flag = False

// Boolean in quantified expressions (inside structures)
structure BoolExamples {
    axiom reflexive_unicode : ∀(p : 𝔹). p = p
    axiom reflexive_full    : ∀(q : Bool). q = q
}
```

### The Unit Type

The `Unit` type represents "no meaningful value" — like `void` in C or `()` in Rust/Haskell. It has exactly one value: `()`.

**When to use Unit:**

1. **Result types that can fail but return nothing on success:**

```kleis
// A validation that succeeds with () or fails with an error message
data ValidationResult = Ok(Unit) | Err(String)

define validate_positive(x : ℝ) : ValidationResult =
    if x > 0 then Ok(()) else Err("must be positive")
```

2. **Optional values where presence matters, not content:**

```kleis
// Option type - Some(value) or None
data Option(T) = Some(T) | None

// A flag that's either set or not (no associated value)
define flag_set : Option(Unit) = Some(())
define flag_unset : Option(Unit) = None
```

3. **Proof terms with no computational content:**

```kleis
// A theorem that x = x (the proof itself carries no data)
structure Reflexivity {
    axiom refl : ∀(x : ℝ). x = x
}
// The "witness" of this axiom would have type Unit
```

## Type Annotations

You can explicitly annotate types with `:`:

```kleis
// Variable annotation
define typed_let = let x : ℝ = 3.14 in x * 2

// Function parameter and return types
define f(x : ℝ) : ℝ = x * x

// Expression-level annotation (ascription)
define sum_typed(a, b) = (a + b) : ℝ
```

## Function Types

Functions have types too! The notation `A → B` means "a function from A to B":

```kleis
// square takes a Real and returns a Real
define square(x : ℝ) : ℝ = x * x
// Type: ℝ → ℝ

// add takes two Reals and returns a Real
define add(x : ℝ, y : ℝ) : ℝ = x + y
// Type: ℝ × ℝ → ℝ (or equivalently: ℝ → ℝ → ℝ)
```

### Higher-Order Function Types

Functions can take other functions as arguments or return functions. These are called **higher-order functions**:

```kleis
// A function that takes a function as an argument
define apply_twice(f : ℝ → ℝ, x : ℝ) : ℝ = f(f(x))
// Type: (ℝ → ℝ) × ℝ → ℝ

// A function that returns a function
define make_adder(n : ℝ) : ℝ → ℝ = ???
// Type: ℝ → (ℝ → ℝ)
```

The parentheses matter! Compare:
- `(ℝ → ℝ) → ℝ` — takes a function, returns a number
- `ℝ → (ℝ → ℝ)` — takes a number, returns a function
- `ℝ → ℝ → ℝ` — curried function (associates right: `ℝ → (ℝ → ℝ)`)

### Function Type Examples

| Type | Meaning |
|------|---------|
| `ℝ → ℝ` | Function from real to real |
| `ℝ → ℝ → ℝ` | Curried binary function |
| `(ℝ → ℝ) → ℝ` | Takes a function, returns a value (e.g., definite integral) |
| `ℝ → (ℝ → ℝ)` | Returns a function (function factory) |
| `(ℝ → ℝ) → (ℝ → ℝ)` | Function transformer (e.g., derivative operator) |

## Set Types

Kleis provides a built-in `Set(T)` type backed by Z3's native set theory. Sets are unordered collections of unique elements:

```kleis
// Declare a set type
define S : Set(ℤ)

// Set operations (see stdlib/sets.kleis for full structure)
in_set(x, S)              // Membership: x ∈ S → Bool
union(A, B)               // Union: A ∪ B → Set(T)
intersect(A, B)           // Intersection: A ∩ B → Set(T)
difference(A, B)          // Difference: A \ B → Set(T)
complement(A)             // Complement: ᶜA → Set(T)
subset(A, B)              // Subset test: A ⊆ B → Bool
insert(x, S)              // Add element: S ∪ {x} → Set(T)
remove(x, S)              // Remove element: S \ {x} → Set(T)
singleton(x)              // Singleton set: {x} → Set(T)
empty_set                 // Empty set: ∅
```

### Set Theory Axioms

Sets come with a complete axiomatization (see `stdlib/sets.kleis`):

```kleis
structure SetTheory(T) {
    // Core operations
    operation in_set : T → Set(T) → Bool
    operation union : Set(T) → Set(T) → Set(T)
    operation intersect : Set(T) → Set(T) → Set(T)
    element empty_set : Set(T)
    
    // Extensionality: sets are equal iff they have the same elements
    axiom extensionality: ∀(A B : Set(T)). 
        (∀(x : T). in_set(x, A) ↔ in_set(x, B)) → A = B
    
    // Union definition
    axiom union_def: ∀(A B : Set(T), x : T). 
        in_set(x, union(A, B)) ↔ (in_set(x, A) ∨ in_set(x, B))
    
    // De Morgan's laws
    axiom de_morgan_union: ∀(A B : Set(T)). 
        complement(union(A, B)) = intersect(complement(A), complement(B))
}
```

### Using Sets in Verification

Sets are particularly useful for specifying properties involving collections:

```kleis
structure MetricSpace(X) {
    operation d : X → X → ℝ
    operation ball : X → ℝ → Set(X)
    
    // Open ball definition
    axiom ball_def: ∀(center : X, radius : ℝ, x : X).
        in_set(x, ball(center, radius)) ↔ d(x, center) < radius
}
```

## Parametric Types

Types can have parameters:

```kleis
// Parametric type examples:
List(ℤ)           // List of integers
Matrix(3, 3, ℝ)   // 3×3 matrix of reals
Vector(4)         // 4-dimensional vector
Set(ℝ)            // Set of real numbers
```

## Dimension Expressions

When working with parameterized types like `Matrix(m, n, ℝ)`, the dimension parameters are not just simple numbers — they can be **dimension expressions**. This enables type-safe operations where dimensions depend on each other.

### Supported Dimension Expressions

| Category | Operators | Examples |
|----------|-----------|----------|
| Arithmetic | `+`, `-`, `*`, `/` | `n+1`, `2*n`, `n/2` |
| Power | `^` | `n^2`, `2^k` |
| Grouping | `( )` | `(n+1)*2` |
| Functions | `min`, `max` | `min(m, n)` |

### Why Dimension Expressions Matter

Consider the **realification functor** from control theory, which embeds a complex n×n matrix into a real 2n×2n matrix:

```kleis
// Complex matrix represented as (real_part, imag_part)
type ComplexMatrix(m: Nat, n: Nat) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))

// Realification: embed C^(n×n) into R^(2n×2n)
structure Realification(n: Nat) {
    operation realify : ComplexMatrix(n, n) → Matrix(2*n, 2*n, ℝ)
    operation complexify : Matrix(2*n, 2*n, ℝ) → ComplexMatrix(n, n)
}
```

The `2*n` dimension expression captures the **invariant** that the output dimension is always twice the input dimension.

### How Dimension Unification Works

When Kleis type-checks your code, it must verify that dimension expressions match. This uses a built-in **dimension solver** that can handle common arithmetic constraints.

#### What the Solver Can Unify

| Expression 1 | Expression 2 | Result |
|--------------|--------------|--------|
| `2*n` | `2*n` | ✅ Structurally equal |
| `2*n` | `6` | ✅ Solved: `n = 3` |
| `n + 1` | `5` | ✅ Solved: `n = 4` |
| `n^2` | `9` | ✅ Solved: `n = 3` |
| `2^k` | `8` | ✅ Solved: `k = 3` |

#### What the Solver Rejects

| Expression 1 | Expression 2 | Result |
|--------------|--------------|--------|
| `2*n` | `n` | ❌ Different structure (unless `n = 0`) |
| `n + 1` | `n` | ❌ Different structure |
| `n*m` | `6` | ⚠️ Underdetermined |

### Examples in Practice

**Matrix multiplication** requires matching inner dimensions:

```kleis
// Matrix(m, n) × Matrix(n, p) → Matrix(m, p)
structure MatrixMultiply(m: Nat, n: Nat, p: Nat) {
    operation matmul : Matrix(m, n, ℝ) → Matrix(n, p, ℝ) → Matrix(m, p, ℝ)
}
```

The `n` dimension must match on both sides — the solver verifies this.

**SVD decomposition** produces matrices with `min(m, n)` dimensions:

```
// Illustrative — tuple return types in structures are aspirational
structure SVD(m: Nat, n: Nat) {
    // A = U * Σ * Vᵀ where Σ is min(m,n) × min(m,n)
    operation svd : Matrix(m, n, ℝ) → 
        (Matrix(m, min(m,n), ℝ), Matrix(min(m,n), min(m,n), ℝ), Matrix(min(m,n), n, ℝ))
}
```

### Simplification

The dimension solver simplifies expressions before comparing them:

| Expression | Simplified |
|------------|------------|
| `0 + n` | `n` |
| `1 * n` | `n` |
| `n^1` | `n` |
| `n^0` | `1` |
| `2 * 3` | `6` |

This means `Matrix(1*n, n+0, ℝ)` correctly unifies with `Matrix(n, n, ℝ)`.

### Design Philosophy

The dimension solver is **deliberately bounded**:

- It handles practical cases (linear arithmetic, powers, min/max)
- It fails clearly on complex constraints it cannot solve
- It doesn't require external dependencies (no SMT solver needed for type checking)

If you need more advanced constraint solving, use the `:verify` command with Z3 at the value level.

## Type Inference

Kleis often infers types automatically:

```kleis
define double(x) = x + x
// Kleis infers: double : ℝ → ℝ (or more general)

define square_five = let y = 5 in y * y
// Kleis infers: y : ℤ
```

But explicit types make code clearer and catch errors earlier!

## The Type Hierarchy

```
                    Any
         /     |      \       \
     Scalar  String  Collection  Set(T)
     /    \              |
    ℂ    Bool          List
    |                 /    \
    ℝ            Vector   Matrix
    |
    ℚ
    |
    ℤ
    |
    ℕ
```

Note: `ℕ ⊂ ℤ ⊂ ℚ ⊂ ℝ ⊂ ℂ` (naturals ⊂ integers ⊂ rationals ⊂ reals ⊂ complex)

**Set(T)** is parameterized by its element type. `Set(ℤ)` is a set of integers, `Set(ℝ)` is a set of reals, etc.

## Type Promotion (Embedding)

When you mix numeric types in an expression, Kleis automatically **promotes** the smaller type to the larger one. This is called **type embedding**, not subtyping.

### Embedding vs Subtyping

| Concept | Meaning | Kleis Approach |
|---------|---------|----------------|
| **Subtyping** | `S` can be used anywhere `T` is expected, with identical behavior | Not used |
| **Embedding** | `S` can be converted to `T` via an explicit `lift` function | ✓ Used |

The difference is subtle but important:

```kleis
// Embedding: Integer 3 is lifted to Rational before the operation
rational(1, 2) + 3
// Becomes: rational_add(rational(1, 2), lift(3))
// Result: rational(7, 2) — exact!
```

### How Promotion Works

1. **Type inference** determines the result type (the "common supertype")
2. **Lifting** converts arguments to the target type
3. **Operation** executes at the target type

```
Int + Rational
    ↓ find common supertype
  Rational
    ↓ lift Int to Rational
  lift(Int) + Rational
    ↓ execute operation
  rational_add(Rational, Rational)
    ↓
  Rational result
```

### The `Promotes` Structure

Type promotion is defined by the `Promotes(From, To)` structure:

```kleis
structure Promotes(From, To) {
  operation lift : From → To
}

// Built-in promotions
implements Promotes(ℕ, ℤ) { operation lift = builtin_nat_to_int }
implements Promotes(ℤ, ℚ) { operation lift = builtin_int_to_rational }
implements Promotes(ℚ, ℝ) { operation lift = builtin_rational_to_real }
implements Promotes(ℝ, ℂ) { operation lift = builtin_real_to_complex }
```

### Defining Your Own Promotions

You can define promotions for your own types. Unlike built-in types (which use `builtin_*` functions), **you must write the conversion function yourself**:

```kleis
data Percentage = Pct(value: ℝ)

// Step 1: Define the conversion function
define pct_to_real(p: Percentage) : ℝ =
  match p { Pct(v) => divide(v, 100) }

// Step 2: Register the promotion, referencing YOUR function
implements Promotes(Percentage, ℝ) {
  operation lift = pct_to_real   // References the function above
}
```

Now this works in the REPL:

```
λ> :eval 0.5 + pct_to_real(Pct(25))
✅ 0.75
```

**Key difference from built-in types:**

| Type | Lift Implementation |
|------|---------------------|
| Built-in (`ℤ → ℚ`) | `operation lift = builtin_int_to_rational` (provided by Kleis) |
| User-defined | `operation lift = your_function` (you must define it) |

> **Important**: For concrete execution (`:eval`), you must provide an actual `define` for the lift function. Without it:
> - `:verify` (symbolic) — Works (Z3 treats `lift` as uninterpreted)
> - `:eval` (concrete) — **Fails** ("function not found")

### Precision Considerations

**Warning**: Promotion can lose precision!

| Promotion | Precision |
|-----------|-----------|
| `ℕ → ℤ` | ✓ Exact (integers contain all naturals) |
| `ℤ → ℚ` | ✓ Exact (rationals contain all integers) |
| `ℚ → ℝ` | ⚠️ **May lose precision** (floating-point approximation) |
| `ℝ → ℂ` | ✓ Exact (complex with zero imaginary part) |

```kleis
// Exact in Rational
define third : ℚ = rational(1, 3)  // Exactly 1/3

// Approximate in Real (floating-point)
define approx : ℝ = 1.0 / 3.0      // 0.333333...

// If you promote:
define promoted = third + 0.5      // third lifted to ℝ, loses exactness!
```

**Recommendation**: When precision matters, be explicit about types:

```kleis
// Keep it in Rational for exact arithmetic
define exact_sum : ℚ = rational(1, 3) + rational(1, 6)  // Exactly 1/2

// Or use type annotations to prevent accidental promotion
define result(x : ℚ, y : ℚ) : ℚ = x + y
```

### No LSP Violations

Because Kleis uses embedding (not subtyping), operations are always resolved at the **target type** after lifting. This means:

- `Int + Int` uses integer addition
- `Int + Rational` lifts the Int first, then uses rational addition
- You never accidentally get integer truncation when you expected rational division

```kleis
5 / 3           // Integer division → 1 (if both are Int)
rational(5, 1) / rational(3, 1)   // Rational division → 5/3 (exact)
```

## What's Next?

Types are the foundation. Now let's see how to define functions!

→ [Next: Functions](./03-functions.md)
