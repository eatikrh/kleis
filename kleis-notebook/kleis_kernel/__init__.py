"""
Kleis Jupyter Kernels

Jupyter kernels for the Kleis mathematical specification language.

Two kernels are provided:
- KleisKernel: Symbolic kernel (one-shot evaluation)
- KleisNumericKernel: Numeric kernel (persistent REPL for LAPACK ops)

Document creation:
- KleisDoc: Create and manage publishable documents
- templates: Available document templates
"""

__version__ = "0.1.0"

from .kernel import KleisKernel
from .numeric_kernel import KleisNumericKernel
from .kleisdoc import KleisDoc, templates

__all__ = ["KleisKernel", "KleisNumericKernel", "KleisDoc", "templates", "__version__"]
