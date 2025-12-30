"""
Entry point for running the Kleis kernel.

This allows the kernel to be run with:
    python -m kleis_kernel
"""

from ipykernel.kernelapp import IPKernelApp
from .kernel import KleisKernel


def main():
    """Launch the Kleis kernel."""
    IPKernelApp.launch_instance(kernel_class=KleisKernel)


if __name__ == "__main__":
    main()
