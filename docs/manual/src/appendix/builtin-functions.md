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

| Function | Description | Example |
|----------|-------------|---------|
| `negate(x)` | Unary negation | `negate(5)` → `-5` |
| `abs(x)` | Absolute value | `abs(-3)` → `3` |
| `sqrt(x)` | Square root | `sqrt(16)` → `4` |
| `floor(x)` | Round down | `floor(3.7)` → `3` |
| `ceil(x)` | Round up | `ceil(3.2)` → `4` |
| `round(x)` | Round to nearest | `round(3.5)` → `4` |
| `min(x, y)` | Minimum | `min(3, 7)` → `3` |
| `max(x, y)` | Maximum | `max(3, 7)` → `7` |

## Trigonometric Functions

| Function | Description |
|----------|-------------|
| `sin(x)` | Sine |
| `cos(x)` | Cosine |
| `tan(x)` | Tangent |
| `asin(x)` | Arcsine |
| `acos(x)` | Arccosine |
| `atan(x)` | Arctangent |
| `sinh(x)` | Hyperbolic sine |
| `cosh(x)` | Hyperbolic cosine |
| `tanh(x)` | Hyperbolic tangent |

## Exponential and Logarithmic

| Function | Description | Example |
|----------|-------------|---------|
| `exp(x)` | e^x | `exp(1)` → `2.718...` |
| `ln(x)` | Natural logarithm | `ln(e)` → `1` |
| `log(x)` | Base-10 logarithm | `log(100)` → `2` |
| `log(b, x)` | Logarithm base b | `log(2, 8)` → `3` |

## List Operations

Kleis uses cons-lists with `Cons` and `Nil`:

| Function | Aliases | Description | Example |
|----------|---------|-------------|---------|
| `Cons(x, xs)` | `cons` | Prepend element | `Cons(1, Nil)` |
| `Nil` | `nil` | Empty list | `Nil` |
| `head(xs)` | `car` | First element | `head([1,2,3])` → `1` |
| `tail(xs)` | `cdr` | Rest of list | `tail([1,2,3])` → `[2,3]` |
| `length(xs)` | | List length | `length([1,2,3])` → `3` |
| `nth(xs, n)` | | Get nth element | `nth([1,2,3], 1)` → `2` |

### List Literal Syntax

```kleis
[1, 2, 3]           // Shorthand for Cons(1, Cons(2, Cons(3, Nil)))
[]                  // Empty list (Nil)
```

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

## Constants

| Constant | Unicode | Value |
|----------|---------|-------|
| `pi` | `π` | 3.14159... |
| `e` | | 2.71828... |
| `i` | | √(-1) (imaginary unit) |
| `True` / `true` | | Boolean true |
| `False` / `false` | | Boolean false |

## See Also

- [Operators](./operators.md) - Operator reference
- [LAPACK Functions](./lapack.md) - Numerical linear algebra
- [Matrices](../chapters/19-matrices.md) - Matrix chapter

