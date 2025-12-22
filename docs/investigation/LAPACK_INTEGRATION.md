# LAPACK Integration Investigation

## Overview

This document investigates options for integrating LAPACK-level linear algebra 
capabilities into Kleis for high-performance numerical computation.

## Current State

Kleis currently has:
- **Z3 Backend**: Symbolic verification, SAT/SMT solving
- **Rust Evaluator**: Concrete matrix operations (add, sub, mul, det 1-3×3, etc.)

Missing high-performance operations:
- Eigenvalue decomposition
- SVD (Singular Value Decomposition)
- LU/QR/Cholesky factorization
- System solving Ax = b (large matrices)
- Matrix inverse (beyond 3×3)
- Determinant (beyond 3×3)

## Options

### Option 1: Pure Rust - nalgebra

**Pros:**
- No external dependencies
- Clean cargo build
- Cross-platform
- Good for small-medium matrices

**Cons:**
- Not as fast as LAPACK for large matrices
- Not as battle-tested (decades vs years)

```toml
[dependencies]
nalgebra = "0.32"
```

```rust
use nalgebra::{DMatrix, SymmetricEigen};

fn eigenvalues(m: &DMatrix<f64>) -> Vec<f64> {
    SymmetricEigen::new(m.clone()).eigenvalues.as_slice().to_vec()
}
```

### Option 2: Pure Rust - faer

**Pros:**
- Modern Rust implementation
- Claims competitive performance with LAPACK
- SIMD optimized
- No external dependencies

**Cons:**
- Newer library (less battle-tested)
- API still evolving

```toml
[dependencies]
faer = "0.19"
```

### Option 3: nalgebra with LAPACK backend

**Pros:**
- Best of both worlds
- Rust API + LAPACK speed
- Battle-tested algorithms

**Cons:**
- Requires system LAPACK installation
- Environment variables needed
- Platform-specific setup

```toml
[dependencies]
nalgebra = { version = "0.32", features = ["lapack"] }
```

Environment setup:
```bash
# macOS
brew install lapack openblas
export LAPACK_DIR=/opt/homebrew/opt/lapack

# Linux
apt install liblapack-dev libblas-dev
```

### Option 4: ndarray-linalg (direct LAPACK)

**Pros:**
- Direct LAPACK bindings
- Full LAPACK functionality
- Very fast

**Cons:**
- Complex build setup
- Multiple backend options (OpenBLAS, Intel MKL, Netlib)

```toml
[dependencies]
ndarray = "0.15"
ndarray-linalg = { version = "0.16", features = ["openblas-system"] }
```

### Option 5: Intel MKL

**Pros:**
- Fastest on Intel CPUs
- Highly optimized
- Industry standard

**Cons:**
- Intel-specific optimizations
- Large dependency
- License considerations

## Cross-Platform Backend Selection

The Rust BLAS/LAPACK ecosystem supports **swappable providers** via `blas-src` and `lapack-src`:

### Available Backends

| Backend | Platform | Notes |
|---------|----------|-------|
| **Apple Accelerate** | macOS | Native, fast, no install needed |
| **OpenBLAS** | Linux/Windows/macOS | Open source, widely used |
| **Intel MKL** | All | Fastest on Intel CPUs |
| **Netlib** | All | Reference implementation |

### Cargo.toml Configuration

```toml
[dependencies]
ndarray = "0.15"
ndarray-linalg = "0.16"

# Platform-specific backends
[target.'cfg(target_os = "macos")'.dependencies]
blas-src = { version = "0.10", features = ["accelerate"] }
lapack-src = { version = "0.10", features = ["accelerate"] }

[target.'cfg(target_os = "linux")'.dependencies]
blas-src = { version = "0.10", features = ["openblas"] }
lapack-src = { version = "0.10", features = ["openblas"] }

[target.'cfg(target_os = "windows")'.dependencies]
blas-src = { version = "0.10", features = ["openblas"] }
lapack-src = { version = "0.10", features = ["openblas"] }
```

### Platform Summary

| Platform | Default Backend | Env Vars Needed? |
|----------|-----------------|------------------|
| **macOS (ARM/Intel)** | Accelerate | ❌ No (system framework) |
| **Linux** | OpenBLAS | Maybe (`OPENBLAS_DIR`) |
| **Windows** | OpenBLAS | Yes (DLL paths) |

### Advantages of Accelerate on macOS

- Pre-installed on all Macs
- Optimized for Apple Silicon (M1/M2/M3)
- No additional dependencies
- Uses Apple's vecLib and BNNS

This means on your Mac ARM, we can use the **native Accelerate framework** with zero external dependencies - similar convenience to pure Rust, with LAPACK-level performance!

## Recommendation

### For Kleis: BLAS/LAPACK via ndarray-linalg

**Primary choice**: `ndarray-linalg` with platform-specific backends.

**Why BLAS/LAPACK over pure Rust:**
- Battle-tested (decades of production use)
- Trusted numerics (control systems, scientific computing)
- Apple Accelerate = zero friction on macOS
- Industry standard (MATLAB, NumPy, Julia all use LAPACK)

```toml
# Cargo.toml
[dependencies]
ndarray = "0.15"
ndarray-linalg = "0.16"

# macOS: Uses Apple Accelerate (pre-installed, M1/M2/M3 optimized)
[target.'cfg(target_os = "macos")'.dependencies]
blas-src = { version = "0.10", features = ["accelerate"] }
lapack-src = { version = "0.10", features = ["accelerate"] }

# Linux: Uses OpenBLAS
[target.'cfg(target_os = "linux")'.dependencies]
blas-src = { version = "0.10", features = ["openblas"] }
lapack-src = { version = "0.10", features = ["openblas"] }

# Windows: Uses OpenBLAS
[target.'cfg(target_os = "windows")'.dependencies]
blas-src = { version = "0.10", features = ["openblas"] }
lapack-src = { version = "0.10", features = ["openblas"] }
```

**On your Mac ARM**: Zero external dependencies needed - Accelerate is a system framework.

### Proposed Operations

| Operation | Kleis Syntax | Description |
|-----------|--------------|-------------|
| `eigenvalues(A)` | `eigenvalues(Matrix(n, n, [...]))` | Compute eigenvalues |
| `eigenvectors(A)` | Returns `(eigenvalues, eigenvectors)` | Full eigen decomposition |
| `svd(A)` | `svd(Matrix(m, n, [...]))` | Singular value decomposition |
| `solve(A, b)` | `solve(A, b)` | Solve Ax = b |
| `inv(A)` | `inv(Matrix(n, n, [...]))` | Matrix inverse |
| `lu(A)` | LU factorization | Returns (L, U, P) |
| `qr(A)` | QR factorization | Returns (Q, R) |
| `cholesky(A)` | Cholesky factorization | For positive definite A |
| `rank(A)` | Matrix rank | Via SVD |
| `norm(A)` | Matrix norm | Various norms |
| `cond(A)` | Condition number | Via SVD |

### Complex Matrix Support

All operations should support complex matrices (ℂ):

```kleis
:eval eigenvalues(Matrix(2, 2, [complex(1, 2), complex(3, 0), complex(0, 1), complex(4, 5)]))
// → [complex(λ1_re, λ1_im), complex(λ2_re, λ2_im)]
```

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Kleis Expression: eigenvalues(Matrix(3, 3, [...]))              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ evaluator.rs: apply_builtin("eigenvalues", args)                │
│   1. Extract matrix from Expression                             │
│   2. Convert to faer/nalgebra matrix                            │
│   3. Call numerical backend                                     │
│   4. Convert result back to Kleis Expression                    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ NumericalBackend trait                                          │
│   - FaerBackend (default, pure Rust)                            │
│   - LapackBackend (optional, feature-gated)                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Vision: Control Toolkit in Kleis

### The Clean Separation

| Layer | Responsibility | Examples |
|-------|---------------|----------|
| **Kleis** | Semantics, equations, correctness, meaning | `x' = Ax + Bu`, stability proofs |
| **Rust** | Numerics, execution, performance | BLAS gemm, LAPACK getrf, expm |

This mirrors proven architectures:
- **MATLAB**: Language vs LAPACK backend
- **Modelica**: Equations vs solvers  
- **Simulink**: Diagrams vs numerical kernels

### What Kleis Should Do (where it shines)

**1️⃣ Declaring control objects**
```kleis
system ContinuousLTI {
  x' = A x + B u
  y  = C x + D u
}
```

**2️⃣ Declaring transformations**
```kleis
discretize sys using ZOH Ts = 0.01
```

**3️⃣ Declaring guarantees**
```kleis
assume A is stable
ensure spectral_radius(Ad) < 1
```

**4️⃣ Declaring equivalence**
```kleis
expm(block(A,B)) ≡ (Ad, Bd)
```

### What Kleis Should NOT Do

Kleis should not:
- Implement LU, QR, Schur, Padé, or Krylov
- Worry about cache, SIMD, or FFI
- Care whether backend is Accelerate, OpenBLAS, or MKL

Those are **numerical plumbing**, not **semantic concerns**.

### Rust's Role: Execution Engine

Rust becomes:
- The Kleis evaluator
- The numerical runtime
- The FFI boundary

```rust
match instr {
    Instr::C2dZoh { sys, ts } => {
        let (ad, bd) = c2d_zoh(sys.a, sys.b, ts)?;
        Value::DiscreteSystem { ad, bd, ... }
    }
}
```

The runtime:
- Chooses BLAS/LAPACK backend
- Logs conditioning warnings
- Returns numerical results

Kleis:
- Reasons about structure
- Proves equivalences
- Remains backend-agnostic

### Capability Interface Design

Define an explicit numeric capability set exposed to Kleis:

```kleis
capability LinearAlgebra {
  matmul
  solve_linear
  schur
  expm
  eig
  svd
  qr
  lu
}
```

Internally backed by BLAS/LAPACK today — but **replaceable later**.

This makes Kleis future-proof and keeps the semantic/numeric boundary clean.

### Why This Matters

> "This is the same reason theorem provers call out to decision procedures."

- Kleis is about **meaning**, not speed
- BLAS/LAPACK give you **trusted numerics**
- You avoid re-encoding numerical algorithms into Kleis
- You keep the semantic/numeric boundary **clean**

---

---

## Direct LAPACK Access for Schur/QZ

### The Problem

No high-level Rust crate exposes `xGEES` (Schur decomposition) directly:
- `ndarray-linalg` doesn't expose it
- `nalgebra` doesn't expose it
- `faer` has its own implementation but different API

### LAPACK Routines We Need

| Routine | Purpose | Kleis Operation |
|---------|---------|-----------------|
| `dgees` | Real Schur decomposition | `schur(A) → (U, T)` |
| `dgges` | Generalized Schur (QZ) | `qz(A, B) → (Q, Z, S, T)` |
| `dtrsen` | Reorder Schur form | `schur_reorder(...)` |
| `dtgsen` | Reorder QZ form | `qz_reorder(...)` |

### Rust Crates for Direct LAPACK

| Crate | Level | Notes |
|-------|-------|-------|
| `lapack` | Mid-level | Safe wrappers, no ndarray integration |
| `lax` | Low-level | Used by ndarray-linalg internally |
| `lapack-sys` | FFI | Raw C bindings |

### Implementation Path

**Option 1: Use `lax` directly (recommended)**
```rust
use lax::DGEES;

// lax provides the LAPACK routines ndarray-linalg uses
```

**Option 2: Use `lapack` crate**
```toml
[dependencies]
lapack = "0.19"
```

```rust
use lapack::dgees;
// Requires manual memory layout handling
```

**Option 3: Wait for faer integration**
- `faer` has native Schur implementation
- Different API than ndarray

### Current Status

Schur/QZ operations are **stubbed out** in `src/numerical/backend.rs`:
- Return `NotImplemented` error
- Document what LAPACK routines are needed
- Will implement when Control Toolkit is built

---

## Next Steps

1. [x] Add `ndarray-linalg` with Accelerate (done)
2. [x] Implement core operations: eig, svd, solve, inv, qr, cholesky (done)
3. [ ] Add `lax` for direct LAPACK calls (Schur, QZ)
4. [ ] Implement Schur decomposition via dgees
5. [ ] Implement QZ decomposition via dgges
6. [ ] Build Control Toolkit structures in Kleis
2. [ ] Implement `NumericalBackend` trait
3. [ ] Add `eigenvalues` operation to evaluator
4. [ ] Add tests for numerical operations
5. [ ] Document in manual
6. [ ] Optional: Add LAPACK backend feature

## Benchmarks Needed

- Compare faer vs nalgebra vs LAPACK for:
  - Eigenvalues 100×100, 1000×1000
  - SVD 100×100, 1000×1000
  - Solve Ax=b 100×100, 1000×1000

## References

- [LAPACK](https://www.netlib.org/lapack/)
- [faer](https://github.com/sarah-ek/faer-rs)
- [nalgebra](https://nalgebra.org/)
- [ndarray-linalg](https://github.com/rust-ndarray/ndarray-linalg)

