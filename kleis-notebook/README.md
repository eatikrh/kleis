# Kleis Jupyter Kernel

A Jupyter kernel for the Kleis mathematical specification language with Z3 verification.

## Features

- **Execute Kleis code** in Jupyter notebooks
- **Session context** - definitions persist across cells
- **Z3 verification** - assertions verified by Z3
- **Formatted output** - ✅ green for pass, ❌ red for fail
- **Magic commands** - `%reset`, `%context`, `%version`

## Installation

### Prerequisites

1. **Kleis binary** must be installed and in your PATH:
   ```bash
   cargo install --path /path/to/kleis
   # or ensure ~/.cargo/bin/kleis exists
   ```

2. **Python 3.8+** with pip

### Install the kernel

```bash
cd kleis-notebook

# Create virtual environment (recommended)
python3 -m venv venv
source venv/bin/activate

# Install the package
pip install -e .

# Register the kernel with Jupyter
python -m kleis_kernel.install
```

## Usage

### Start Jupyter

```bash
jupyter notebook
# or
jupyter lab
```

### Create a new notebook

Select **Kleis** from the kernel dropdown when creating a new notebook.

### Example cells

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

### Magic Commands

| Command | Description |
|---------|-------------|
| `%reset` | Clear session context (forget all definitions) |
| `%context` | Show accumulated definitions |
| `%version` | Show Kleis and kernel versions |

## How It Works

1. Each cell's code is appended to the session context
2. The kernel writes context + cell to a temp `.kleis` file
3. Runs `kleis test temp.kleis`
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

