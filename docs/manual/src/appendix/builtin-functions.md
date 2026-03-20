# Appendix: Built-in Functions

This appendix covers built-in functions for basic operations. For numerical linear algebra (eigenvalues, SVD, etc.), see [LAPACK Functions](./lapack.md).

## Output Functions

Functions for displaying values:

| Function | Aliases | Description |
|----------|---------|-------------|
| `out(x)` | `show(x)`, `print(x)` | Pretty-print value and return it |

### Example

```kleis
out([[1, 2], [3, 4]])
// Prints:
// ┌      ┐
// │ 1  2 │
// │ 3  4 │
// └      ┘
```

## Arithmetic Functions

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `negate(x)` | | Unary negation | `negate(5)` → `-5` |
| `abs(x)` | `fabs` | Absolute value | `abs(-3)` → `3` |
| `sqrt(x)` | | Square root | `sqrt(16)` → `4` |
| `pow(x, y)` | `power` | x^y | `pow(2, 3)` → `8` |
| `floor(x)` | | Round down | `floor(3.7)` → `3` |
| `ceil(x)` | `ceiling` | Round up | `ceil(3.2)` → `4` |
| `round(x)` | | Round to nearest | `round(3.5)` → `4` |
| `trunc(x)` | `truncate` | Truncate toward zero | `trunc(-3.7)` → `-3` |
| `frac(x)` | `fract` | Fractional part | `frac(3.7)` → `0.7` |
| `sign(x)` | `signum` | Sign (-1, 0, or 1) | `sign(-5)` → `-1` |
| `min(x, y)` | | Minimum | `min(3, 7)` → `3` |
| `max(x, y)` | | Maximum | `max(3, 7)` → `7` |
| `mod(x, y)` | `fmod`, `remainder` | Modulo/remainder | `mod(7, 3)` → `1` |
| `hypot(x, y)` | | √(x² + y²) stable | `hypot(3, 4)` → `5` |

## Trigonometric Functions (radians)

All trigonometric functions use radians, not degrees. Use `radians(deg)` to convert.

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `sin(x)` | | Sine | `sin(0)` → `0` |
| `cos(x)` | | Cosine | `cos(0)` → `1` |
| `tan(x)` | | Tangent | `tan(0)` → `0` |
| `asin(x)` | `arcsin` | Arcsine | `asin(1)` → `π/2` |
| `acos(x)` | `arccos` | Arccosine | `acos(1)` → `0` |
| `atan(x)` | `arctan` | Arctangent | `atan(1)` → `π/4` |
| `atan2(y, x)` | `arctan2` | 2-arg arctangent | `atan2(1, 1)` → `π/4` |
| `radians(deg)` | `deg_to_rad` | Degrees to radians | `radians(180)` → `π` |

## Hyperbolic Functions

| Function | Aliases | Description |
|----------|---------|-------------|
| `sinh(x)` | | Hyperbolic sine |
| `cosh(x)` | | Hyperbolic cosine |
| `tanh(x)` | | Hyperbolic tangent |
| `asinh(x)` | `arcsinh` | Inverse hyperbolic sine |
| `acosh(x)` | `arccosh` | Inverse hyperbolic cosine |
| `atanh(x)` | `arctanh` | Inverse hyperbolic tangent |

**Identity:** `cosh(x)² - sinh(x)² = 1`

## Exponential and Logarithmic

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `exp(x)` | | e^x | `exp(1)` → `2.718...` |
| `exp2(x)` | | 2^x | `exp2(3)` → `8` |
| `log(x)` | `ln` | Natural logarithm | `log(e())` → `1` |
| `log10(x)` | | Base-10 logarithm | `log10(100)` → `2` |
| `log2(x)` | | Base-2 logarithm | `log2(8)` → `3` |

## List Operations

### Basic List Functions

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `Cons(x, xs)` | `cons` | Prepend element | `Cons(1, Nil)` |
| `Nil` | `nil` | Empty list | `Nil` |
| `head(xs)` | `car` | First element | `head([1,2,3])` → `1` |
| `tail(xs)` | `cdr` | Rest of list | `tail([1,2,3])` → `[2,3]` |
| `length(xs)` | `list_length` | List length | `length([1,2,3])` → `3` |
| `nth(xs, n)` | `list_nth` | Get nth element (0-indexed) | `nth([1,2,3], 1)` → `2` |

### List Literal Syntax

```kleis
[1, 2, 3]           // Bracket list (preferred for numeric work)
[]                  // Empty list
```

### List Generation

| Function | Description | Example |
|----------|-------------|---------|
| `range(n)` | Integers 0 to n-1 | `range(4)` → `[0, 1, 2, 3]` |
| `range(start, end)` | Integers from start to end-1 | `range(2, 5)` → `[2, 3, 4]` |
| `linspace(start, end)` | 50 evenly spaced floats | `linspace(0, 1)` → `[0, 0.0204..., ...]` |
| `linspace(start, end, n)` | n evenly spaced floats | `linspace(0, 1, 5)` → `[0, 0.25, 0.5, 0.75, 1]` |

### Higher-Order List Functions

These functions take a lambda as their first argument.

| Function | Aliases | Description |
|----------|---------|-------------|
| `list_map(f, xs)` | | Apply f to each element |
| `list_filter(pred, xs)` | | Keep elements where pred returns true |
| `list_fold(f, init, xs)` | | Left fold with accumulator |
| `list_flatmap(f, xs)` | `flatmap`, `concat_map` | Map then flatten results |
| `list_zip(xs, ys)` | | Pair corresponding elements |

#### list_map

Apply a function to each element:

```kleis
list_map(lambda x . x * 2, [1, 2, 3])
// → [2, 4, 6]

list_map(lambda x . x * x, range(5))
// → [0, 1, 4, 9, 16]
```

#### list_filter

Keep elements satisfying a predicate:

```kleis
list_filter(lambda x . x > 2, [1, 2, 3, 4, 5])
// → [3, 4, 5]
```

#### list_fold

Reduce a list with an accumulator (left fold):

```kleis
// Sum: f(f(f(0, 1), 2), 3) = ((0+1)+2)+3 = 6
list_fold(lambda acc x . acc + x, 0, [1, 2, 3])
// → 6

// Product
list_fold(lambda acc x . acc * x, 1, [2, 3, 4])
// → 24
```

#### list_flatmap

Map a function that returns lists, then flatten:

```kleis
list_flatmap(lambda x . [x, x*10], [1, 2, 3])
// → [1, 10, 2, 20, 3, 30]
```

#### list_zip

Pair corresponding elements (stops at shorter list):

```kleis
list_zip([1, 2, 3], ["a", "b", "c"])
// → [Pair(1, "a"), Pair(2, "b"), Pair(3, "c")]
```

Use `fst` and `snd` to extract pair components:

```kleis
let p = Pair(1, "a") in fst(p)  // → 1
let p = Pair(1, "a") in snd(p)  // → "a"
```

### List Manipulation

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `list_concat(xs, ys)` | `list_append` | Concatenate two lists | `list_concat([1,2], [3,4])` → `[1,2,3,4]` |
| `list_flatten(xss)` | `list_join` | Flatten nested list | `list_flatten([[1,2], [3,4]])` → `[1,2,3,4]` |
| `list_slice(xs, start, end)` | | Sublist from start to end-1 | `list_slice([a,b,c,d], 1, 3)` → `[b,c]` |
| `list_rotate(xs, n)` | | Rotate left by n positions | `list_rotate([a,b,c], 1)` → `[b,c,a]` |

## String Operations

| Function | Description | Example |
|----------|-------------|---------|
| `concat(a, b)` | Concatenate strings | `concat("hello", "world")` → `"helloworld"` |
| `strlen(s)` | String length | `strlen("hello")` → `5` |
| `contains(s, sub)` | Check substring | `contains("hello", "ell")` → `true` |
| `substr(s, start, len)` | Extract substring | `substr("hello", 1, 3)` → `"ell"` |
| `replace(s, old, new)` | Replace substring | `replace("hello", "l", "L")` → `"heLLo"` |

## Matrix Operations (Basic)

For advanced operations (eigenvalues, SVD), see [LAPACK Functions](./lapack.md).

### Matrix Creation

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `matrix(rows, cols, elements)` | | Create matrix | `matrix(2, 2, [1,2,3,4])` |
| `eye(n)` | `identity(n)` | Identity matrix | `eye(3)` |
| `zeros(m, n)` | | Zero matrix | `zeros(2, 3)` |
| `ones(m, n)` | | Matrix of ones | `ones(2, 3)` |
| `diag_matrix(elements)` | `diagonal` | Diagonal matrix | `diag_matrix([1,2,3])` |

### Matrix Literals

```kleis
[[1, 2, 3],
 [4, 5, 6]]         // 2×3 matrix
```

### Matrix Properties

| Function | Aliases | Description |
|----------|---------|-------------|
| `size(A)` | `shape`, `dims` | Dimensions `[rows, cols]` |
| `nrows(A)` | `num_rows` | Number of rows |
| `ncols(A)` | `num_cols` | Number of columns |

### Element Access

| Function | Aliases | Description |
|----------|---------|-------------|
| `matrix_get(A, i, j)` | `element` | Get element at (i, j) |
| `matrix_row(A, i)` | `row` | Get row i |
| `matrix_col(A, j)` | `col` | Get column j |
| `matrix_diag(A)` | `diag` | Get diagonal |

### Element Modification

| Function | Description |
|----------|-------------|
| `set_element(A, i, j, val)` | Set element at (i, j) |
| `set_row(A, i, row)` | Set row i |
| `set_col(A, j, col)` | Set column j |
| `set_diag(A, diag)` | Set diagonal |

### Basic Arithmetic

| Function | Aliases | Description |
|----------|---------|-------------|
| `matrix_add(A, B)` | `builtin_matrix_add` | A + B |
| `matrix_sub(A, B)` | `builtin_matrix_sub` | A - B |
| `multiply(A, B)` | `matmul`, `builtin_matrix_mul` | A × B |
| `scalar_matrix_mul(c, A)` | `builtin_matrix_scalar_mul` | c × A |
| `transpose(A)` | `builtin_transpose` | Aᵀ |
| `trace(A)` | `builtin_trace` | tr(A) |
| `det(A)` | `builtin_determinant` | det(A) |

### Matrix Stacking

| Function | Aliases | Description |
|----------|---------|-------------|
| `vstack(A, B)` | `append_rows` | Stack vertically |
| `hstack(A, B)` | `append_cols` | Stack horizontally |
| `prepend_row(A, row)` | | Add row at top |
| `append_row(A, row)` | | Add row at bottom |
| `prepend_col(A, col)` | | Add column at left |
| `append_col(A, col)` | | Add column at right |

## Review Context

These zero-argument functions are available inside review policy files.
They return the intent and file path set by the current `check_code`,
`check_file`, or `diff_check_file` invocation.

| Function | Returns | Description |
|----------|---------|-------------|
| `review_intent()` | `String` | The change intent passed by the caller (empty string if none) |
| `review_path()` | `String` | The file path being reviewed (empty string if none) |

See [Agent MCP Servers — Intent-Aware Review](../chapters/28-agent-mcps.md#intent-aware-review).

## Mathematical Constants

| Function | Unicode | Value | Description |
|----------|---------|-------|-------------|
| `pi()` | `π` | 3.14159... | Pi |
| `e()` | | 2.71828... | Euler's number |
| `tau()` | `τ` | 6.28318... | τ = 2π |
| `i` | | √(-1) | Imaginary unit |

**Note:** `pi()`, `e()`, and `tau()` are zero-argument functions.

## Boolean Constants

| Constant | Description |
|----------|-------------|
| `True` / `true` | Boolean true |
| `False` / `false` | Boolean false |

## See Also

- [Operators](./operators.md) - Operator reference
- [LAPACK Functions](./lapack.md) - Numerical linear algebra
- [Matrices](../chapters/19-matrices.md) - Matrix chapter


