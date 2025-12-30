#!/bin/bash
# Start Jupyter Lab with Kleis kernel
#
# Usage:
#   ./start-jupyter.sh          # Start JupyterLab
#   ./start-jupyter.sh notebook # Start classic Jupyter Notebook
#   ./start-jupyter.sh --help   # Show help

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

show_help() {
    echo "Kleis Jupyter Launcher"
    echo ""
    echo "Usage: $0 [OPTION]"
    echo ""
    echo "Options:"
    echo "  (none)      Start JupyterLab (default)"
    echo "  notebook    Start classic Jupyter Notebook"
    echo "  install     Install/reinstall the kernel and dependencies"
    echo "  --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Start JupyterLab"
    echo "  $0 notebook     # Start Jupyter Notebook"
    echo "  $0 install      # Reinstall everything"
}

check_kleis() {
    if ! command -v kleis &> /dev/null; then
        echo -e "${RED}Error: 'kleis' binary not found in PATH${NC}"
        echo ""
        echo "Install kleis first:"
        echo "  cd /path/to/kleis"
        echo "  export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h"
        echo "  cargo install --path ."
        exit 1
    fi
    echo -e "${GREEN}✓${NC} kleis binary found: $(which kleis)"
}

setup_venv() {
    if [ ! -d "$VENV_DIR" ]; then
        echo -e "${YELLOW}Creating virtual environment...${NC}"
        python3 -m venv "$VENV_DIR"
    fi
    
    # Activate venv
    source "$VENV_DIR/bin/activate"
    
    # Check if packages are installed
    if ! python -c "import jupyterlab" 2>/dev/null; then
        echo -e "${YELLOW}Installing dependencies...${NC}"
        pip install -q -e "$SCRIPT_DIR"
        pip install -q jupyterlab
    fi
    
    # Check if kernels are registered
    if ! jupyter kernelspec list 2>/dev/null | grep -q "^kleis "; then
        echo -e "${YELLOW}Registering Kleis (symbolic) kernel...${NC}"
        python -m kleis_kernel.install
    fi
    
    if ! jupyter kernelspec list 2>/dev/null | grep -q "kleis-numeric"; then
        echo -e "${YELLOW}Registering Kleis Numeric kernel...${NC}"
        python -m kleis_kernel.install_numeric
    fi
    
    echo -e "${GREEN}✓${NC} Virtual environment ready"
}

install_all() {
    echo -e "${YELLOW}Installing Kleis Jupyter kernel...${NC}"
    
    # Remove old venv if exists
    if [ -d "$VENV_DIR" ]; then
        echo "Removing old virtual environment..."
        rm -rf "$VENV_DIR"
    fi
    
    # Create fresh venv
    echo "Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
    source "$VENV_DIR/bin/activate"
    
    # Install packages
    echo "Installing packages..."
    pip install --upgrade pip
    pip install -e "$SCRIPT_DIR"
    pip install jupyterlab
    
    # Register kernels
    echo "Registering Kleis kernels..."
    python -m kleis_kernel.install
    python -m kleis_kernel.install_numeric
    
    echo ""
    echo -e "${GREEN}✓ Installation complete!${NC}"
    echo ""
    echo "Two kernels available:"
    echo "  - Kleis: Symbolic evaluation"
    echo "  - Kleis Numeric: Concrete computation (eigenvalues, SVD, etc.)"
    echo ""
    echo "Run '$0' to start JupyterLab"
}

start_jupyter() {
    local mode="${1:-lab}"
    
    check_kleis
    setup_venv
    
    echo ""
    echo -e "${GREEN}Starting Jupyter ${mode}...${NC}"
    echo -e "Kleis kernel version: $(kleis --version 2>/dev/null || echo 'unknown')"
    echo ""
    
    if [ "$mode" = "notebook" ]; then
        jupyter notebook --notebook-dir="$SCRIPT_DIR"
    else
        jupyter lab --notebook-dir="$SCRIPT_DIR"
    fi
}

# Main
case "${1:-}" in
    --help|-h)
        show_help
        ;;
    install)
        check_kleis
        install_all
        ;;
    notebook)
        start_jupyter notebook
        ;;
    *)
        start_jupyter lab
        ;;
esac

