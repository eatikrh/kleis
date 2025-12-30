"""
Kleis Jupyter Kernels

Jupyter kernels for the Kleis mathematical specification language.

Two kernels are provided:
- KleisKernel: Symbolic kernel (one-shot evaluation)
- KleisNumericKernel: Numeric kernel (persistent REPL for LAPACK ops)
"""

__version__ = "0.1.0"

from .kernel import KleisKernel
from .numeric_kernel import KleisNumericKernel

__all__ = ["KleisKernel", "KleisNumericKernel", "__version__"]
