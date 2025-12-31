# Appendix: LAPACK Functions

Kleis provides comprehensive numerical linear algebra operations through LAPACK integration. These functions are available when Kleis is compiled with the `numerical` feature.

> **Note:** These operations perform concrete numerical computation. For symbolic matrix operations, see [Built-in Functions](./builtin-functions.md).

## Eigenvalues and Eigenvectors

| Function | Aliases | Description | Returns |
|----------|---------|-------------|---------|
| `eigenvalues(A)` | `eigvals` | Compute eigenvalues | List of eigenvalues |
| `eig(A)` | | Eigenvalues + eigenvectors | `[eigenvalues, eigenvectors]` |

### Example

```kleis
// 2×2 matrix
let A = [[4, 2], [1, 3]] in
eigenvalues(A)   // → [-5.0, 2.0] (approximately)
```

## Singular Value Decomposition

| Function | Aliases | Description | Returns |
|----------|---------|-------------|---------|
| `svd(A)` | | Full SVD decomposition | `[U, S, Vt]` |
| `singular_values(A)` | `svdvals` | Singular values only | List of singular values |

### Example

```kleis
let A = [[1, 2], [3, 4], [5, 6]] in
let [U, S, Vt] = svd(A) in
// A ≈ U × diag(S) × Vt
singular_values(A)   // → [9.52..., 0.51...]
```

## Matrix Decompositions

| Function | Aliases | Description | Returns |
|----------|---------|-------------|---------|
| `qr(A)` | | QR decomposition | `[Q, R]` |
| `cholesky(A)` | `chol` | Cholesky decomposition | Lower triangular L |
| `schur(A)` | `schur_decomp` | Schur decomposition | `[T, Z]` |

### QR Example

```kleis
let A = [[1, 2], [3, 4], [5, 6]] in
let [Q, R] = qr(A) in
// A = Q × R, where Q is orthogonal, R is upper triangular
```

### Cholesky Example

```kleis
// Positive definite matrix
let A = [[4, 2], [2, 3]] in
let L = cholesky(A) in
// A = L × Lᵀ
```

## Linear Systems

| Function | Aliases | Description |
|----------|---------|-------------|
| `solve(A, b)` | `linsolve` | Solve Ax = b |
| `inv(A)` | `inverse` | Matrix inverse A⁻¹ |

### Example

```kleis
let A = [[3, 1], [1, 2]] in
let b = [9, 8] in
solve(A, b)   // → [2, 3] (solution x where Ax = b)

inv(A)        // → [[0.4, -0.2], [-0.2, 0.6]]
```

## Matrix Properties

| Function | Aliases | Description |
|----------|---------|-------------|
| `rank(A)` | `matrix_rank` | Matrix rank |
| `cond(A)` | `condition_number` | Condition number |
| `norm(A)` | `matrix_norm` | Matrix norm (Frobenius) |
| `det_lapack(A)` | | Determinant (via LU) |

### Example

```kleis
let A = [[1, 2], [3, 4]] in
rank(A)    // → 2
cond(A)    // → 14.93... (κ(A))
norm(A)    // → 5.47... (Frobenius norm)
```

## Matrix Functions

| Function | Aliases | Description |
|----------|---------|-------------|
| `expm(A)` | `matrix_exp` | Matrix exponential e^A |
| `mpow(A, n)` | `matrix_pow` | Matrix power A^n |

### Example

```kleis
let A = [[0, 1], [-1, 0]] in
expm(A)   // → rotation matrix (since A is skew-symmetric)

let B = [[1, 1], [0, 1]] in
mpow(B, 3)   // → [[1, 3], [0, 1]]
```

---

## Complex Matrix Operations

For complex matrices, use the `cmat_*` variants:

### Eigenvalues and Decompositions

| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_eigenvalues(A)` | `cmat_eigvals` | Complex eigenvalues |
| `cmat_eig(A)` | | Complex eigenvalues + eigenvectors |
| `cmat_svd(A)` | | Complex SVD |
| `cmat_singular_values(A)` | `cmat_svdvals` | Complex singular values |
| `cmat_schur(A)` | `schur_complex` | Complex Schur decomposition |

### Linear Systems

| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_solve(A, b)` | `cmat_linsolve` | Solve complex Ax = b |
| `cmat_inv(A)` | `cmat_inverse` | Complex inverse |
| `cmat_qr(A)` | | Complex QR decomposition |

### Matrix Properties

| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_rank(A)` | `cmat_matrix_rank` | Complex matrix rank |
| `cmat_cond(A)` | `cmat_condition_number` | Complex condition number |
| `cmat_norm(A)` | `cmat_matrix_norm` | Complex matrix norm |
| `cmat_det(A)` | `cmat_determinant` | Complex determinant |

### Matrix Functions

| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_expm(A)` | `cmat_matrix_exp` | Complex matrix exponential |
| `cmat_mpow(A, n)` | `cmat_matrix_pow` | Complex matrix power |

---

## Complex Matrix Utilities

| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_zero(m, n)` | `builtin_cmat_zero` | Complex zero matrix |
| `cmat_eye(n)` | `builtin_cmat_eye` | Complex identity |
| `cmat_from_real(A)` | `as_complex` | Real → complex matrix |
| `cmat_from_imag(A)` | `as_imaginary` | Imag → complex matrix |
| `cmat_real(A)` | `real_part_matrix` | Extract real part |
| `cmat_imag(A)` | `imag_part_matrix` | Extract imaginary part |
| `cmat_add(A, B)` | | Complex addition |
| `cmat_sub(A, B)` | | Complex subtraction |
| `cmat_mul(A, B)` | | Complex multiplication |
| `cmat_conj(A)` | | Element-wise conjugate |
| `cmat_transpose(A)` | | Transpose |
| `cmat_dagger(A)` | `cmat_adjoint` | Conjugate transpose (A†) |
| `cmat_trace(A)` | | Complex trace |
| `cmat_scale_real(c, A)` | | Scale by real scalar |

---

## Matrix Conversion

| Function | Aliases | Description |
|----------|---------|-------------|
| `realify(A)` | `builtin_realify` | Complex n×n → Real 2n×2n |
| `complexify(A)` | `builtin_complexify` | Real 2n×2n → Complex n×n |

These functions convert between complex matrices and their real-valued block representations:

```
Complex matrix:     Real representation:
[a+bi  c+di]   →    [a  -b  c  -d]
[e+fi  g+hi]        [b   a  d   c]
                    [e  -f  g  -h]
                    [f   e  h   g]
```

---

## Jupyter Notebook Usage

When using Kleis Numeric kernel in Jupyter:

```kleis
// Define a matrix
let A = [[1, 2], [3, 4]]

// Compute eigenvalues
eigenvalues(A)

// Pretty-print with out()
out(inv(A))
```

## See Also

- [Built-in Functions](./builtin-functions.md) - Basic matrix operations
- [Operators](./operators.md) - Operator reference
- [Jupyter Notebook](../chapters/21-jupyter-notebook.md) - Using numerical Kleis

