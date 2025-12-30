# Kleis Jupyter Kernels

Jupyter kernels for the Kleis mathematical specification language with Z3 verification and numerical computation.

## Kernels

Two kernels are provided:

| Kernel | Display Name | Description |
|--------|--------------|-------------|
| `kleis` | **Kleis** | Symbolic evaluation with Z3 verification |
| `kleis-numeric` | **Kleis Numeric** | Concrete numerical computation via REPL |

## Features

- **Execute Kleis code** in Jupyter notebooks
- **Session context** - definitions persist across cells
- **Z3 verification** - assertions verified by Z3
- **Numerical computation** - eigenvalues, SVD, matrix operations (LAPACK)
- **Formatted output** - ✅ green for pass, ❌ red for fail
- **Magic commands** - `%reset`, `%context`, `%version`

## Installation

### Prerequisites

1. **Kleis binary** must be installed with numerical features:
   ```bash
   cd /path/to/kleis
   export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
   cargo install --path . --features numerical
   ```

2. **Python 3.8+** with pip

### Quick Start

```bash
cd kleis-notebook
./start-jupyter.sh
```

The script will:
- Create a virtual environment if needed
- Install all dependencies
- Register both kernels
- Launch JupyterLab

### Manual Installation

```bash
cd kleis-notebook

# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate

# Install the package
pip install -e .
pip install jupyterlab

# Register both kernels
python -m kleis_kernel.install
python -m kleis_kernel.install_numeric
```

## Usage

### Start Jupyter

```bash
./start-jupyter.sh              # Start JupyterLab (default)
./start-jupyter.sh notebook     # Start classic Jupyter Notebook
./start-jupyter.sh install      # Reinstall everything
```

### Create a New Notebook

Select **Kleis** or **Kleis Numeric** from the kernel dropdown when creating a new notebook.

### Example Cells

**Cell 1: Define a structure**
```kleis
structure Group(G) {
    operation (*) : G × G → G
    element e : G
    
    axiom identity: ∀(a : G). a * e = a
}
```

**Cell 2: Test with example block**
```kleis
example "group properties" {
    assert(e * e = e)
}
```

**Cell 3: Compute eigenvalues** (requires `--features numerical`)
```kleis
eigenvalues([[1.0, 2.0], [3.0, 4.0]])
```
Output: `[-0.3722813232690143, 5.372281323269014]`

**Cell 4: Matrix operations**
```kleis
det([[1.0, 2.0], [3.0, 4.0]])      // → -2
inv([[1.0, 2.0], [3.0, 4.0]])      // → Matrix(2, 2, [-2, 1, 1.5, -0.5])
svd([[1.0, 2.0], [3.0, 4.0]])      // → (U, S, Vt)
```

**Cell 5: Use REPL commands**
```kleis
:type 1 + 2 * 3
:eval sin(0) + cos(0)
:verify ∀(x : ℝ). x + 0 = x
```

### REPL Commands

| Command | Description |
|---------|-------------|
| `:type <expr>` | Show inferred type of expression |
| `:eval <expr>` | Evaluate expression to concrete value |
| `:verify <expr>` | Verify assertion with Z3 |
| `:ast <expr>` | Show parsed AST structure |
| `:env` | Show current session context |
| `:load <file>` | Load a .kleis file into session |

### Numerical Operations (LAPACK)

When Kleis is compiled with `--features numerical`:

| Function | Description |
|----------|-------------|
| `eigenvalues(M)` | Compute eigenvalues of square matrix |
| `eig(M)` | Eigenvalues and eigenvectors |
| `svd(M)` | Singular value decomposition |
| `inv(M)` | Matrix inverse |
| `det(M)` | Determinant |
| `solve(A, b)` | Solve linear system Ax = b |
| `qr(M)` | QR decomposition |
| `cholesky(M)` | Cholesky decomposition |
| `rank(M)` | Matrix rank |
| `cond(M)` | Condition number |
| `norm(M)` | Matrix norm |
| `expm(M)` | Matrix exponential |
| `schur(M)` | Schur decomposition |

Matrix syntax: `[[1, 2], [3, 4]]` (nested lists, row-major order)

### Jupyter Magic Commands

| Command | Description |
|---------|-------------|
| `%reset` | Clear session context (forget all definitions) |
| `%context` | Show accumulated definitions |
| `%version` | Show Kleis and kernel versions |

## How It Works

1. Each cell's code is appended to the session context
2. For example blocks: runs `kleis test temp.kleis`
3. For expressions: runs `kleis eval <expression>` 
4. Parses output and displays with formatting
5. Definitions (`structure`, `define`, etc.) are remembered for subsequent cells

## Development

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black kleis_kernel/

# Type check
mypy kleis_kernel/
```

## License

BSD-3-Clause (same as Kleis)
