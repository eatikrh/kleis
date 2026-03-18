"""
Install the Kleis kernel specification for Jupyter.

Usage:
    python -m kleis_kernel.install
    python -m kleis_kernel.install --user
    python -m kleis_kernel.install --sys-prefix
"""

import argparse
import json
import os
import shutil
import sys
import tempfile

from jupyter_client.kernelspec import KernelSpecManager


def install_kernel_spec(user: bool = True, prefix: str = None):
    """Install the Kleis kernel specification."""
    
    kernel_json = {
        "argv": [
            sys.executable,
            "-m",
            "kleis_kernel",
            "-f",
            "{connection_file}",
        ],
        "display_name": "Kleis",
        "language": "kleis",
        "metadata": {
            "debugger": False,
        },
    }

    with tempfile.TemporaryDirectory() as td:
        kernel_dir = os.path.join(td, "kleis")
        os.makedirs(kernel_dir)

        with open(os.path.join(kernel_dir, "kernel.json"), "w") as f:
            json.dump(kernel_json, f, indent=2)

        # Copy logo if it exists
        logo_src = os.path.join(os.path.dirname(__file__), "logo.svg")
        if os.path.exists(logo_src):
            shutil.copy(logo_src, os.path.join(kernel_dir, "logo-svg.svg"))
            shutil.copy(logo_src, os.path.join(kernel_dir, "logo-64x64.svg"))

        ksm = KernelSpecManager()
        
        if prefix:
            ksm.install_kernel_spec(kernel_dir, kernel_name="kleis", prefix=prefix)
            print(f"Installed Kleis kernel to {prefix}")
        else:
            ksm.install_kernel_spec(kernel_dir, kernel_name="kleis", user=user)
            location = "user" if user else "system"
            print(f"Installed Kleis kernel ({location})")


def main():
    parser = argparse.ArgumentParser(description="Install the Kleis Jupyter kernel")
    parser.add_argument("--user", action="store_true", default=True, help="Install for current user (default)")
    parser.add_argument("--sys-prefix", action="store_true", help="Install to sys.prefix (for virtualenv/conda)")
    parser.add_argument("--prefix", type=str, help="Install to a custom prefix")

    args = parser.parse_args()

    if args.sys_prefix:
        install_kernel_spec(user=False, prefix=sys.prefix)
    elif args.prefix:
        install_kernel_spec(user=False, prefix=args.prefix)
    else:
        install_kernel_spec(user=args.user)

    print("\n✅ Kleis kernel installed successfully!")
    print("\nTo use:")
    print("  1. Start Jupyter: jupyter lab")
    print("  2. Create a new notebook")
    print("  3. Select 'Kleis' from the kernel dropdown")
    print("\nTo verify installation:")
    print("  jupyter kernelspec list")


if __name__ == "__main__":
    main()
