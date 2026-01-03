"""
Kleis Jupyter Kernels

Jupyter kernels for the Kleis mathematical specification language.

Two kernels are provided:
- KleisKernel: Symbolic kernel (one-shot evaluation)
- KleisNumericKernel: Numeric kernel (persistent REPL for LAPACK ops)

Document creation:
- KleisDoc: Create and manage publishable documents
- templates: Available document templates

Visual editing:
- equation_editor: Embed the Kleis Equation Editor (static/index.html) via iframe
- EquationEditorWidget: Class-based wrapper for the editor
"""

__version__ = "0.1.0"

from .kernel import KleisKernel
from .numeric_kernel import KleisNumericKernel
from .kleisdoc import KleisDoc, list_templates
from .equation_editor import equation_editor, EquationEditorWidget

__all__ = [
    "KleisKernel", 
    "KleisNumericKernel", 
    "KleisDoc",
    "list_templates",
    "equation_editor",
    "EquationEditorWidget",
    "__version__"
]
