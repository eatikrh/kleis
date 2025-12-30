"""
Installer for the Kleis Numeric Jupyter kernel.

This installs the numeric kernel that runs a persistent REPL
for concrete numerical computation.
"""

import json
import os
import shutil
import sys
from pathlib import Path

# Kernel specification
KERNEL_SPEC = {
    "argv": [
        sys.executable,
        "-m",
        "kleis_kernel.numeric_kernel",
        "-f",
        "{connection_file}",
    ],
    "display_name": "Kleis Numeric",
    "language": "kleis",
    "metadata": {
        "debugger": False,
    },
}


def install_kernel(user: bool = True, prefix: str = None):
    """Install the Kleis Numeric kernel specification."""
    from jupyter_client.kernelspec import KernelSpecManager

    ksm = KernelSpecManager()

    # Create a temporary directory with kernel files
    import tempfile

    with tempfile.TemporaryDirectory() as td:
        kernel_dir = Path(td) / "kleis-numeric"
        kernel_dir.mkdir()

        # Write kernel.json
        kernel_json_path = kernel_dir / "kernel.json"
        with open(kernel_json_path, "w") as f:
            json.dump(KERNEL_SPEC, f, indent=2)

        # Copy logo if available
        logo_src = Path(__file__).parent / "logo.svg"
        if logo_src.exists():
            shutil.copy(logo_src, kernel_dir / "logo-64x64.svg")
            shutil.copy(logo_src, kernel_dir / "logo-32x32.svg")

        # Install the kernel spec
        dest = ksm.install_kernel_spec(
            str(kernel_dir),
            kernel_name="kleis-numeric",
            user=user,
            prefix=prefix,
        )
        print(f"✅ Installed Kleis Numeric kernel to: {dest}")

    return dest


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Install the Kleis Numeric Jupyter kernel"
    )
    parser.add_argument(
        "--user",
        action="store_true",
        default=True,
        help="Install for current user (default)",
    )
    parser.add_argument(
        "--sys-prefix",
        action="store_true",
        help="Install to sys.prefix (for virtualenvs)",
    )
    parser.add_argument(
        "--prefix",
        type=str,
        help="Install to a specific prefix",
    )

    args = parser.parse_args()

    if args.sys_prefix:
        install_kernel(user=False, prefix=sys.prefix)
    elif args.prefix:
        install_kernel(user=False, prefix=args.prefix)
    else:
        install_kernel(user=args.user)


if __name__ == "__main__":
    main()

