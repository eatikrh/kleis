"""
Kleis Jupyter Kernels

Jupyter kernels for the Kleis mathematical specification language.

Two kernels are provided:
- KleisKernel: Symbolic kernel (one-shot evaluation)
- KleisNumericKernel: Numeric kernel (persistent REPL for LAPACK ops)

Document creation (thin shell - Kleis does the heavy lifting):
- compile_to_pdf: Compile a .kleis document to PDF
- compile_to_typst: Compile a .kleis document to Typst
- list_templates: List available document templates
- validate: Validate a .kleis document

Visual editing:
- equation_editor: Embed the Kleis Equation Editor (static/index.html) via iframe
- EquationEditorWidget: Class-based wrapper for the editor

Binary discovery:
- find_kleis_binary: Find the Kleis binary (respects KLEIS_ROOT env var)
- find_kleis_root: Find the Kleis project root directory
- get_kleis_status: Get status information about Kleis installation
"""

__version__ = "0.1.0"

from .kernel import KleisKernel
from .numeric_kernel import KleisNumericKernel
from .kleisdoc_shell import compile_to_pdf, compile_to_typst, list_templates, validate
from .equation_editor import equation_editor, EquationEditorWidget
from .kleis_binary import (
    find_kleis_binary, 
    find_kleis_root, 
    get_status as get_kleis_status
)

__all__ = [
    "KleisKernel", 
    "KleisNumericKernel", 
    "compile_to_pdf",
    "compile_to_typst",
    "list_templates",
    "validate",
    "equation_editor",
    "EquationEditorWidget",
    "find_kleis_binary",
    "find_kleis_root",
    "get_kleis_status",
    "__version__"
]
