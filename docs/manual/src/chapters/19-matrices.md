# Matrices

Kleis provides comprehensive matrix support with both symbolic verification (via Z3) and concrete evaluation (via `:eval`).

## Matrix Type

Matrices are parametric types with dimensions:

```kleis
Matrix(m, n, T)   // m rows × n columns of type T
```

Examples:
- `Matrix(2, 2, ℝ)` - 2×2 matrix of reals
- `Matrix(3, 4, ℂ)` - 3×4 matrix of complex numbers

## Creating Matrices

Use the `Matrix` constructor with dimensions and a list of elements (row-major order):

```kleis
// 2×2 matrix: [[1, 2], [3, 4]]
Matrix(2, 2, [1, 2, 3, 4])

// 2×3 matrix: [[1, 2, 3], [4, 5, 6]]
Matrix(2, 3, [1, 2, 3, 4, 5, 6])
```

## Arithmetic Operations

### Addition and Subtraction

Element-wise operations for matrices of the same dimensions:

```kleis
:eval matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [5, 6, 7, 8]))
// → Matrix(2, 2, [6, 8, 10, 12])

:eval matrix_sub(Matrix(2, 2, [10, 20, 30, 40]), Matrix(2, 2, [1, 2, 3, 4]))
// → Matrix(2, 2, [9, 18, 27, 36])
```

### Matrix Multiplication

True matrix multiplication `(m×n) · (n×p) → (m×p)`:

```kleis
:eval multiply(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [5, 6, 7, 8]))
// → Matrix(2, 2, [19, 22, 43, 50])

// Non-square: (2×3) · (3×2) → (2×2)
:eval multiply(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), Matrix(3, 2, [1, 2, 3, 4, 5, 6]))
// → Matrix(2, 2, [22, 28, 49, 64])
```

### Scalar Multiplication

Multiply all elements by a scalar:

```kleis
:eval scalar_matrix_mul(3, Matrix(2, 2, [1, 2, 3, 4]))
// → Matrix(2, 2, [3, 6, 9, 12])
```

## Matrix Properties

### Transpose

Swap rows and columns `(m×n) → (n×m)`:

```kleis
:eval transpose(Matrix(2, 3, [1, 2, 3, 4, 5, 6]))
// → Matrix(3, 2, [1, 4, 2, 5, 3, 6])
```

### Trace

Sum of diagonal elements (square matrices only):

```kleis
:eval trace(Matrix(3, 3, [1, 0, 0, 0, 2, 0, 0, 0, 3]))
// → 6
```

### Determinant

Determinant for 1×1, 2×2, and 3×3 matrices:

```kleis
:eval det(Matrix(2, 2, [4, 3, 6, 8]))
// → 14  (4*8 - 3*6)

:eval det(Matrix(3, 3, [1, 2, 3, 0, 1, 4, 5, 6, 0]))
// → 1
```

## Element Extraction

### Get Single Element

Access element at row `i`, column `j` (0-indexed):

```kleis
:eval matrix_get(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0, 2)
// → 3  (row 0, column 2)

:eval matrix_get(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 1, 1)
// → 5  (row 1, column 1)
```

### Get Row or Column

Extract entire row or column as a list:

```kleis
:eval matrix_row(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0)
// → [1, 2, 3]

:eval matrix_row(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 1)
// → [4, 5, 6]

:eval matrix_col(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0)
// → [1, 4]

:eval matrix_col(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 2)
// → [3, 6]
```

### Get Diagonal

Extract diagonal elements as a list:

```kleis
:eval matrix_diag(Matrix(3, 3, [1, 2, 3, 4, 5, 6, 7, 8, 9]))
// → [1, 5, 9]
```

## Symbolic Matrix Operations

Matrix operations support partial symbolic evaluation. When mixing concrete and symbolic values, Kleis evaluates what it can and leaves the rest symbolic:

```kleis
:eval matrix_add(Matrix(2, 2, [1, 0, 0, 1]), Matrix(2, 2, [a, 0, 0, b]))
// → Matrix(2, 2, [1+a, 0, 0, 1+b])

:eval matrix_add(Matrix(2, 2, [0, 0, 0, 0]), Matrix(2, 2, [x, y, z, w]))
// → Matrix(2, 2, [x, y, z, w])  (0+x = x optimization)
```

### Smart Optimizations

The evaluator applies algebraic simplifications:
- `0 + x = x`
- `x + 0 = x`
- `x - 0 = x`
- `0 * x = 0`
- `1 * x = x`

## Dimension Checking

Kleis enforces dimension constraints at the type level:

```kleis
// This type-checks: same dimensions
matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [5, 6, 7, 8]))

// This fails: dimension mismatch
matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(3, 3, [1, 2, 3, 4, 5, 6, 7, 8, 9]))
// Error: dimension mismatch: 2x2 vs 3x3
```

For matrix multiplication, inner dimensions must match:

```kleis
// OK: (2×3) · (3×2) → (2×2)
multiply(Matrix(2, 3, [...]), Matrix(3, 2, [...]))

// Error: (2×2) · (3×2) - inner dimensions 2 ≠ 3
multiply(Matrix(2, 2, [...]), Matrix(3, 2, [...]))
// Error: inner dimensions don't match
```

## Complete Operations Reference

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `matrix_add(A, B)` | `(m×n) + (m×n) → (m×n)` | Element-wise addition |
| `matrix_sub(A, B)` | `(m×n) - (m×n) → (m×n)` | Element-wise subtraction |
| `multiply(A, B)` | `(m×n) · (n×p) → (m×p)` | Matrix multiplication |
| `scalar_matrix_mul(s, A)` | `ℝ × (m×n) → (m×n)` | Scalar multiplication |
| `transpose(A)` | `(m×n) → (n×m)` | Transpose |
| `trace(A)` | `(n×n) → ℝ` | Sum of diagonal |
| `det(A)` | `(n×n) → ℝ` | Determinant (n ≤ 3) |
| `matrix_get(A, i, j)` | `(m×n) × ℕ × ℕ → T` | Element at (i, j) |
| `matrix_row(A, i)` | `(m×n) × ℕ → List(T)` | Row i |
| `matrix_col(A, j)` | `(m×n) × ℕ → List(T)` | Column j |
| `matrix_diag(A)` | `(n×n) → List(T)` | Diagonal elements |

## See Also

- [Types and Values](./02-types.md) - Matrix as a parametric type
- [Complex Numbers](./14-complex-numbers.md) - Matrices over ℂ
- [The REPL](./12-repl.md) - Using `:eval` for concrete computation

