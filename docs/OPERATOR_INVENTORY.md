# Kleis Operator & Function Inventory

> Generated from parser (`src/kleis_parser.rs`) and evaluator (`src/evaluator.rs`)

## 1. Arithmetic Operators (Parser)

| Operator | Unicode | Name in AST | Example |
|----------|---------|-------------|---------|
| `+` | | `plus` | `3 + 4` |
| `-` | | `minus` | `10 - 3` |
| `*` | `×` | `times` | `6 × 7` |
| `/` | `÷` | `divide` | `15 / 3` |
| `^` | | `power` | `2 ^ 10` |
| `·` | | `·` (dot product) | `a · b` |

## 2. Comparison Operators (Parser)

| Operator | Unicode | Name in AST | Example |
|----------|---------|-------------|---------|
| `=` | | `equals` | `x = y` |
| `==` | | `equals` | `x == y` |
| `!=` | `≠` | `not_equals` | `x ≠ y` |
| `<` | | `less_than` | `x < y` |
| `>` | | `greater_than` | `x > y` |
| `<=` | `≤` | `leq` | `x ≤ y` |
| `>=` | `≥` | `geq` | `x ≥ y` |

## 3. Logical Operators (Parser)

| Operator | Unicode | Name in AST | Example |
|----------|---------|-------------|---------|
| `and` | `∧` | `logical_and` | `P ∧ Q` |
| `or` | `∨` | `logical_or` | `P ∨ Q` |
| `not` | `¬` | `not` / prefix | `¬P` |
| `implies` | `→` `⇒` `⟹` | `implies` | `P → Q` |
| `iff` | `↔` `⟺` `⇔` | `iff` | `P ↔ Q` |
| `&&` | | `logical_and` | `P && Q` |
| `||` | | `logical_or` | `P \|\| Q` |

## 4. Postfix Operators (Parser)

| Operator | Name | Example |
|----------|------|---------|
| `!` | factorial | `n!` |
| `ᵀ` | transpose | `Aᵀ` |
| `†` | dagger/adjoint | `A†` |
| `′` | prime (derivative) | `f′` |
| `″` | double prime | `f″` |
| `‴` | triple prime | `f‴` |
| `⁺` | superscript plus | `A⁺` |
| `⁻` | superscript minus | `A⁻` |

## 5. Prefix Operators (Parser)

| Operator | Name | Example |
|----------|------|---------|
| `-` | negate | `-x` |
| `∇` | gradient | `∇f` |
| `∫` | integral | `∫f` |
| `¬` | logical not | `¬P` |

## 6. Custom Mathematical Operators (Parser)

| Operator | Name | Example |
|----------|------|---------|
| `•` | bullet product | `a • b` |
| `∘` | composition | `f ∘ g` |
| `⊗` | tensor product | `A ⊗ B` |
| `⊕` | direct sum | `A ⊕ B` |
| `⊙` | scalar action | `c ⊙ v` |
| `⊛` | circled asterisk | custom |
| `⊘` | circled slash | custom |
| `⊚` | circled ring | custom |
| `⊝` | circled dash | custom |
| `⊞` | squared plus | custom |
| `⊟` | squared minus | custom |
| `⊠` | squared times | custom |
| `⊡` | squared dot | custom |
| `⨀` | n-ary circled dot | custom |
| `⨁` | n-ary circled plus | custom |
| `⨂` | n-ary circled times | custom |
| `⊓` | square cap | custom |
| `⊔` | square cup | custom |
| `∪` | union | `A ∪ B` |
| `∩` | intersection | `A ∩ B` |
| `⋃` | n-ary union | custom |
| `⋂` | n-ary intersection | custom |
| `△` | triangle | custom |
| `▽` | nabla/del | custom |

## 7. Big Operators (Parser v0.95)

| Operator | Name | Syntax |
|----------|------|--------|
| `Σ` | summation | `Σ(from, to, body)` → `sum_bounds(body, from, to)` |
| `Π` | product | `Π(from, to, body)` → `prod_bounds(body, from, to)` |
| `∫` | integral | `∫(lower, upper, body, var)` → `int_bounds(body, lower, upper, var)` |
| `lim` | limit | `lim(var, target, body)` → `lim(body, var, target)` |

## 8. Type Operators (Parser)

| Operator | Name | Example |
|----------|------|---------|
| `→` | function type | `ℝ → ℝ` |
| `×` | product type | `ℝ × ℝ` |
| `:` | type annotation | `x : ℝ` |

## 9. Quantifiers (Parser)

| Keyword | Unicode | Example |
|---------|---------|---------|
| `forall` | `∀` | `∀(x : ℝ). x = x` |
| `exists` | `∃` | `∃(x : ℝ). x > 0` |

## 10. Keywords (Parser)

### Control Flow
| Keyword | Purpose |
|---------|---------|
| `if` | Conditional start |
| `then` | Conditional consequence |
| `else` | Conditional alternative |
| `match` | Pattern matching |
| `let` | Local binding |
| `in` | Binding body |

### Functions
| Keyword | Purpose |
|---------|---------|
| `lambda` / `λ` | Anonymous function |
| `define` | Function definition |

### Structures
| Keyword | Purpose |
|---------|---------|
| `structure` | Define algebraic structure |
| `implements` | Implement structure |
| `operation` | Declare operation |
| `element` | Declare element |
| `axiom` | Declare axiom |
| `over` | Type constraint |
| `extends` | Structure extension |
| `where` | Constraint clause |

### Data Types
| Keyword | Purpose |
|---------|---------|
| `data` | Algebraic data type |
| `type` | Type alias |

### Modules
| Keyword | Purpose |
|---------|---------|
| `import` | File import |
| `example` | Test block |
| `assert` | Assertion |

---

## 11. Built-in Functions (Evaluator)

### Output/Display
| Function | Aliases | Description |
|----------|---------|-------------|
| `out` | `show`, `print` | Pretty-print value |

### Arithmetic
| Function | Aliases | Description |
|----------|---------|-------------|
| `negate` | | Unary negation |
| `times` | `*`, `mul` | Multiplication |
| `divide` | `/`, `div` | Division |

### String Operations
| Function | Description |
|----------|-------------|
| `concat` | Concatenate strings |
| `strlen` | String length |
| `contains` | Check substring |
| `substr` / `substring` | Extract substring |
| `replace` | Replace substring |

### List Operations
| Function | Aliases | Description |
|----------|---------|-------------|
| `Cons` | `cons` | List constructor |
| `Nil` | `nil` | Empty list |
| `head` | `car` | First element |
| `tail` | `cdr` | Rest of list |
| `length` | | List length |
| `nth` | | Get nth element |

### Matrix Operations (Basic)
| Function | Aliases | Description |
|----------|---------|-------------|
| `matrix_add` | `builtin_matrix_add` | Add matrices |
| `matrix_sub` | `builtin_matrix_sub` | Subtract matrices |
| `multiply` | `builtin_matrix_mul`, `matmul` | Matrix multiply |
| `transpose` | `builtin_transpose` | Transpose |
| `trace` | `builtin_trace` | Matrix trace |
| `det` | `builtin_determinant` | Determinant |
| `scalar_matrix_mul` | `builtin_matrix_scalar_mul` | Scalar × matrix |
| `size` | `shape`, `dims` | Matrix dimensions |
| `nrows` | `num_rows` | Number of rows |
| `ncols` | `num_cols` | Number of columns |
| `matrix_get` | `element` | Get element |
| `matrix_row` | `row` | Get row |
| `matrix_col` | `col` | Get column |
| `matrix_diag` | `diag` | Get diagonal |
| `set_element` | `set` | Set element |
| `set_row` | | Set row |
| `set_col` | | Set column |
| `set_diag` | | Set diagonal |
| `eye` | `identity` | Identity matrix |
| `zeros` | | Zero matrix |
| `ones` | | Matrix of ones |
| `diag_matrix` | `diagonal` | Diagonal matrix |
| `matrix` | | Create matrix |
| `vstack` | `append_rows` | Vertical stack |
| `hstack` | `append_cols` | Horizontal stack |
| `prepend_row` | | Prepend row |
| `append_row` | | Append row |
| `prepend_col` | | Prepend column |
| `append_col` | | Append column |

### Complex Number Operations
| Function | Aliases | Description |
|----------|---------|-------------|
| `complex_add` | `cadd` | Complex addition |
| `complex_sub` | `csub` | Complex subtraction |
| `complex_mul` | `cmul` | Complex multiplication |
| `complex_conj` | `conj`, `conjugate` | Complex conjugate |
| `complex_abs_squared` | `abs_sq` | |z|² |
| `Re` | `re`, `real_part`, `real` | Real part |
| `Im` | `im`, `imag_part`, `imag` | Imaginary part |

### Complex Matrix Operations
| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_zero` | `builtin_cmat_zero` | Complex zero matrix |
| `cmat_eye` | `builtin_cmat_eye` | Complex identity |
| `cmat_from_real` | `builtin_cmat_from_real`, `as_complex` | Real→complex |
| `cmat_from_imag` | `builtin_cmat_from_imag`, `as_imaginary` | Imag→complex |
| `cmat_real` | `builtin_cmat_real`, `real_part_matrix` | Real part |
| `cmat_imag` | `builtin_cmat_imag`, `imag_part_matrix` | Imag part |
| `cmat_add` | `builtin_cmat_add` | Complex matrix add |
| `cmat_sub` | `builtin_cmat_sub` | Complex matrix sub |
| `cmat_mul` | `builtin_cmat_mul` | Complex matrix mul |
| `cmat_conj` | `builtin_cmat_conj` | Conjugate |
| `cmat_transpose` | `builtin_cmat_transpose` | Transpose |
| `cmat_dagger` | `builtin_cmat_dagger`, `cmat_adjoint` | Adjoint (†) |
| `cmat_trace` | `builtin_cmat_trace` | Trace |
| `cmat_scale_real` | `builtin_cmat_scale_real` | Scale by real |

### Matrix Conversion
| Function | Aliases | Description |
|----------|---------|-------------|
| `realify` | `builtin_realify` | Complex→real (2n×2n) |
| `complexify` | `builtin_complexify` | Real (2n×2n)→complex |

### Numerical Linear Algebra (LAPACK)
| Function | Aliases | Description |
|----------|---------|-------------|
| `eigenvalues` | `eigvals` | Eigenvalues |
| `eig` | | Eigenvalues + eigenvectors |
| `svd` | | Singular value decomposition |
| `singular_values` | `svdvals` | Singular values only |
| `solve` | `linsolve` | Solve Ax = b |
| `inv` | `inverse` | Matrix inverse |
| `qr` | | QR decomposition |
| `cholesky` | `chol` | Cholesky decomposition |
| `rank` | `matrix_rank` | Matrix rank |
| `cond` | `condition_number` | Condition number |
| `norm` | `matrix_norm` | Matrix norm |
| `det_lapack` | | Determinant (LAPACK) |
| `schur` | `schur_decomp` | Schur decomposition |
| `expm` | `matrix_exp` | Matrix exponential |
| `mpow` | `matrix_pow` | Matrix power |

### Complex Numerical Linear Algebra
| Function | Aliases | Description |
|----------|---------|-------------|
| `cmat_eigenvalues` | `cmat_eigvals` | Complex eigenvalues |
| `cmat_schur` | `schur_complex` | Complex Schur |
| `cmat_svd` | | Complex SVD |
| `cmat_singular_values` | `cmat_svdvals` | Complex singular values |
| `cmat_solve` | `cmat_linsolve` | Complex linear solve |
| `cmat_inv` | `cmat_inverse` | Complex inverse |
| `cmat_qr` | | Complex QR |
| `cmat_rank` | `cmat_matrix_rank` | Complex rank |
| `cmat_cond` | `cmat_condition_number` | Complex condition |
| `cmat_norm` | `cmat_matrix_norm` | Complex norm |
| `cmat_det` | `cmat_determinant` | Complex determinant |
| `cmat_eig` | | Complex eigenvectors |
| `cmat_expm` | `cmat_matrix_exp` | Complex matrix exp |
| `cmat_mpow` | `cmat_matrix_pow` | Complex matrix power |

---

## 12. Constants

| Constant | Unicode | Value |
|----------|---------|-------|
| `pi` | `π` | 3.14159... |
| `e` | | 2.71828... |
| `i` | | √(-1) |
| `True` / `true` | | Boolean true |
| `False` / `false` | | Boolean false |

---

## 13. Type Names

| Type | Unicode | Description |
|------|---------|-------------|
| `Real` | `ℝ` | Real numbers |
| `Int` | `ℤ` | Integers |
| `Nat` | `ℕ` | Natural numbers |
| `Bool` | `𝔹` | Booleans |
| `Complex` | `ℂ` | Complex numbers |
| `Rational` | `ℚ` | Rational numbers |

---

## Summary Statistics

| Category | Count |
|----------|-------|
| Arithmetic operators | 6 |
| Comparison operators | 7 |
| Logical operators | 7 |
| Postfix operators | 8 |
| Prefix operators | 4 |
| Custom math operators | 26 |
| Big operators | 4 |
| Type operators | 3 |
| Keywords | 25+ |
| Built-in functions | 100+ |
| Constants | 5 |
| Type names | 6 |

---

*Last updated: December 30, 2024*

