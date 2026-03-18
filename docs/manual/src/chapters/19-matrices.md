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

Use the `Matrix` constructor with dimensions and a list of elements (row-major order).
The element type is inferred from the list contents:

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

## Row and Column Manipulation

### Stacking Matrices

Combine matrices vertically or horizontally:

```
λ> :let A = matrix([[1, 2], [3, 4]])
λ> :let B = matrix([[5, 6], [7, 8]])

λ> :eval vstack(A, B)
✅ Matrix(4, 2, [1, 2, 3, 4, 5, 6, 7, 8])  // A on top, B below

λ> :eval hstack(A, B)
✅ Matrix(2, 4, [1, 2, 5, 6, 3, 4, 7, 8])  // A left, B right
```

### Appending Rows and Columns

Add a single row or column:

```
λ> :let A = matrix([[1, 2], [3, 4]])

λ> :eval append_row(A, [5, 6])
✅ Matrix(3, 2, [1, 2, 3, 4, 5, 6])  // Row added at bottom

λ> :eval prepend_row([0, 0], A)
✅ Matrix(3, 2, [0, 0, 1, 2, 3, 4])  // Row added at top

λ> :eval append_col(A, [10, 20])
✅ Matrix(2, 3, [1, 2, 10, 3, 4, 20])  // Column added at right

λ> :eval prepend_col([10, 20], A)
✅ Matrix(2, 3, [10, 1, 2, 20, 3, 4])  // Column added at left
```

### Building Augmented Matrices

Useful for linear algebra (solving `Ax = b`):

```
λ> :let A = matrix([[1, 2], [3, 4]])
λ> :let b = matrix([[5], [6]])
λ> :eval hstack(A, b)
✅ Matrix(2, 3, [1, 2, 5, 3, 4, 6])  // [A | b]
```

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `vstack(A, B)` | m×n, k×n → (m+k)×n | Stack rows |
| `hstack(A, B)` | m×n, m×k → m×(n+k) | Stack columns |
| `append_row(M, r)` | m×n, [n] → (m+1)×n | Add row at bottom |
| `prepend_row(r, M)` | [n], m×n → (m+1)×n | Add row at top |
| `append_col(M, c)` | m×n, [m] → m×(n+1) | Add column at right |
| `prepend_col(c, M)` | [m], m×n → m×(n+1) | Add column at left |

## Setting Values

Kleis matrices are immutable, but you can create new matrices with modified values:

### Set Individual Element

```
λ> :let A = zeros(3, 3)
λ> :eval A
✅ matrix([[0, 0, 0], [0, 0, 0], [0, 0, 0]])

λ> :eval set_element(A, 1, 1, 99)
✅ matrix([[0, 0, 0], [0, 99, 0], [0, 0, 0]])
```

### Set Row or Column

```
λ> :eval set_row(zeros(3, 3), 0, [1, 2, 3])
✅ matrix([[1, 2, 3], [0, 0, 0], [0, 0, 0]])

λ> :eval set_col(zeros(3, 3), 2, [7, 8, 9])
✅ matrix([[0, 0, 7], [0, 0, 8], [0, 0, 9]])
```

### Set Diagonal

```
λ> :eval set_diag(zeros(3, 3), [1, 2, 3])
✅ matrix([[1, 0, 0], [0, 2, 0], [0, 0, 3]])
```

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `set_element(M, i, j, v)` | m×n → m×n | Set element at (i,j) |
| `set_row(M, i, [v...])` | m×n → m×n | Set row i |
| `set_col(M, j, [v...])` | m×n → m×n | Set column j |
| `set_diag(M, [v...])` | n×n → n×n | Set diagonal |

## Matrix Size

```
λ> :let A = matrix([[1, 2, 3], [4, 5, 6]])
λ> :eval size(A)
✅ [2, 3]

λ> :eval nrows(A)
✅ 2

λ> :eval ncols(A)
✅ 3
```

| Operation | Result | Description |
|-----------|--------|-------------|
| `size(M)` | `[m, n]` | Get dimensions as list |
| `nrows(M)` | `m` | Number of rows |
| `ncols(M)` | `n` | Number of columns |

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

### Matrix Constructors

| Constructor | Example | Description |
|-------------|---------|-------------|
| `matrix([[...], [...]])` | `matrix([[1,2],[3,4]])` | **Nested list syntax** (recommended) |
| `Matrix(m, n, [...])` | `Matrix(2, 2, [1,2,3,4])` | Explicit dimensions + flat list |
| `eye(n)` | `eye(3)` → 3×3 identity | n×n identity matrix |
| `zeros(n)` | `zeros(3)` → 3×3 zeros | n×n zero matrix |
| `zeros(m, n)` | `zeros(2, 3)` → 2×3 zeros | m×n zero matrix |
| `ones(n)` | `ones(3)` → 3×3 ones | n×n matrix of ones |
| `ones(m, n)` | `ones(2, 4)` → 2×4 ones | m×n matrix of ones |
| `diag_matrix([...])` | `diag_matrix([1,2,3])` | Diagonal matrix from list |

**Nested list syntax** is recommended for readability:

```
λ> :eval matrix([[1, 2, 3], [4, 5, 6], [7, 8, 9]])
✅ Matrix(3, 3, [1, 2, 3, 4, 5, 6, 7, 8, 9])

λ> :eval matrix([[0, -1], [1, 0]])
✅ Matrix(2, 2, [0, -1, 1, 0])

λ> :eval eigenvalues(matrix([[0, -1], [1, 0]]))
✅ [complex(0, 1), complex(0, -1)]
```

Typed identity is a structure element (no top-level operation). Use a type annotation
to pick the dimension:

```
(identity : Matrix(3, 3, ℝ))
```

Other constructors:

```
λ> :eval eye(3)
✅ Matrix(3, 3, [1, 0, 0, 0, 1, 0, 0, 0, 1])

λ> :eval diag_matrix([5, 10, 15])
✅ Matrix(3, 3, [5, 0, 0, 0, 10, 0, 0, 0, 15])
```

### Core Matrix Operations

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

### LAPACK Numerical Operations

When compiled with the `numerical` feature, Kleis provides high-performance
numerical linear algebra via LAPACK:

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `lapack_eigenvalues(A)` | `(n×n) → List(ℂ)` | Eigenvalues |
| `lapack_eig(A)` | `(n×n) → [eigenvalues, eigenvectors]` | Full eigendecomposition |
| `lapack_svd(A)` | `(m×n) → [U, S, Vᵀ]` | Singular value decomposition |
| `lapack_singular_values(A)` | `(m×n) → List(ℝ)` | Singular values only |
| `lapack_solve(A, b)` | `(n×n) × (n) → (n)` | Solve Ax = b |
| `lapack_inv(A)` | `(n×n) → (n×n)` | Matrix inverse |
| `lapack_qr(A)` | `(m×n) → [Q, R]` | QR decomposition |
| `lapack_cholesky(A)` | `(n×n) → (n×n)` | Cholesky factorization L |
| `lapack_rank(A)` | `(m×n) → ℕ` | Matrix rank |
| `lapack_cond(A)` | `(m×n) → ℝ` | Condition number |
| `lapack_norm(A)` | `(m×n) → ℝ` | Frobenius norm |
| `lapack_det(A)` | `(n×n) → ℝ` | Determinant (any size) |
| `schur(A)` | `(n×n) → [U, T, eigenvalues]` | Schur decomposition (LAPACK dgees) |

### Schur Decomposition for Control Theory

The Schur decomposition A = UTUᵀ is critical for control theory applications:

```kleis
// Compute Schur decomposition
let A = Matrix(3, 3, [-2, 0, 1, 0, -1, 0, 1, 0, -3])
:eval schur(A)
// Returns [U, T, eigenvalues] where:
// - U is orthogonal (Schur vectors)
// - T is quasi-upper-triangular (Schur form)
// - eigenvalues are (real, imag) pairs
```

**Use cases:**
- Stability analysis (check if eigenvalues have Re(λ) < 0)
- Lyapunov equation solvers
- CARE/DARE (algebraic Riccati equations)
- Pole placement algorithms

## Example: LQG Controller Design

This complete example demonstrates designing an LQG (Linear-Quadratic-Gaussian) controller for a mass-spring-damper system using Kleis's numerical matrix operations.

### System Definition

```
λ> :let A = matrix([[0, 1], [-2, -3]])
λ> :let B = matrix([[0], [1]])
λ> :let C = matrix([[1, 0]])
```

The system `ẋ = Ax + Bu`, `y = Cx` represents a 2nd-order mechanical system.

### Open-Loop Stability Analysis

```
λ> :eval eigenvalues(A)
✅ [-1, -2]
```

Both eigenvalues have negative real parts → system is stable, but response is slow.

### LQR Controller Design (Pole Placement)

**Goal:** Place closed-loop poles at -5, -5 for faster response.

**Step 1:** Check controllability (det ≠ 0 means we can place poles anywhere):

```
λ> :let Wc = hstack(B, multiply(A, B))
λ> :eval Wc
✅ matrix([[0, 1], [1, -3]])

λ> :eval det(Wc)
✅ -1
```

**Step 2:** Extract open-loop characteristic polynomial coefficients.

The open-loop characteristic polynomial is det(sI - A) = s² + a₁s + a₀.
From A = [[0, 1], [-2, -3]], we get: s² + 3s + 2, so a₁ = 3, a₀ = 2.

```
λ> :let a1 = 3
λ> :let a0 = 2
```

**Step 3:** Set desired closed-loop characteristic polynomial.

For poles at -5, -5: (s+5)² = s² + 10s + 25, so α₁ = 10, α₀ = 25.

```
λ> :let alpha1 = 10
λ> :let alpha0 = 25
```

**Step 4:** Compute feedback gain K using coefficient matching.

For a controllable canonical form with B = [[0], [1]], the gain is K = [α₀-a₀, α₁-a₁]:

```
λ> :let k1 = alpha0 - a0
λ> :let k2 = alpha1 - a1
λ> :eval k1
✅ 23
λ> :eval k2
✅ 7

λ> :let K = matrix([[k1, k2]])
λ> :eval K
✅ matrix([[23, 7]])
```

**Step 5:** Verify closed-loop eigenvalues:

```
λ> :let A_cl = matrix_sub(A, multiply(B, K))
λ> :eval A_cl
✅ matrix([[0, 1], [-25, -10]])

λ> :eval eigenvalues(A_cl)
✅ [-5.000000042200552, -4.999999957799446]
```

Closed-loop poles are at -5 (2.5× faster than open-loop).

### Kalman Observer Design

**Goal:** Design observer with poles at -10, -10 (faster than controller for separation principle).

**Step 1:** Check observability:

```
λ> :let Wo = vstack(C, multiply(C, A))
λ> :eval Wo
✅ matrix([[1, 0], [0, 1]])

λ> :eval det(Wo)
✅ 1
```

**Step 2:** Apply duality - observer design uses Aᵀ and Cᵀ.

For observer error dynamics (A - LC), the characteristic polynomial is:
det(sI - A + LC) = s² + (3 + l₁)s + (2 + l₂)

Desired: (s+10)² = s² + 20s + 100

**Step 3:** Compute observer gain L:

```
λ> :let l1 = 20 - 3
λ> :let l2 = 100 - 2
λ> :eval l1
✅ 17
λ> :eval l2
✅ 98

λ> :let L = matrix([[l1], [l2]])
λ> :eval L
✅ matrix([[17], [98]])
```

**Step 4:** Verify observer eigenvalues:

```
λ> :let A_obs = matrix_sub(A, multiply(L, C))
λ> :eval A_obs
✅ matrix([[-17, 1], [-100, -3]])

λ> :eval eigenvalues(A_obs)
✅ [complex(-10, 7.14), complex(-10, -7.14)]
```

Observer eigenvalues have Re(λ) = -10 → faster error convergence than the controller.

### Controllability and Observability

```
λ> :let Wc = hstack(B, multiply(A, B))
λ> :eval Wc
✅ matrix([[0, 1], [1, -3]])

λ> :eval det(Wc)
✅ -1
```

det(Wc) ≠ 0 → System is **controllable**.

```
λ> :let Wo = vstack(C, multiply(C, A))
λ> :eval Wo
✅ matrix([[1, 0], [0, 1]])

λ> :eval det(Wo)
✅ 1
```

det(Wo) ≠ 0 → System is **observable**.

### Summary

| Property | Value | Status |
|----------|-------|--------|
| Open-loop poles | -1, -2 | Stable but slow |
| Closed-loop poles (LQR) | -5, -5 | ✅ 2.5× faster |
| Observer poles (Kalman) | -10 ± 7.14i | ✅ 2× faster than controller |
| Controllable | det(Wc) ≠ 0 | ✅ Yes |
| Observable | det(Wo) ≠ 0 | ✅ Yes |

The **Separation Principle** is satisfied: LQR and Kalman can be designed independently, and the combined LQG controller is guaranteed stable.

## See Also

- [Types and Values](./02-types.md) - Matrix as a parametric type
- [Complex Numbers](./14-complex-numbers.md) - Matrices over ℂ
- [The REPL](./12-repl.md) - Using `:eval` for concrete computation

