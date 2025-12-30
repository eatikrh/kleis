"""
Kleis Jupyter Kernel

This kernel executes Kleis mathematical specifications by calling the
Kleis binary and returning the results to Jupyter.
"""

import os
import re
import shutil
import subprocess
import tempfile
from typing import Any, Dict, Optional, Tuple

from ipykernel.kernelbase import Kernel


class KleisKernel(Kernel):
    """Jupyter kernel for the Kleis mathematical specification language."""

    implementation = "Kleis"
    implementation_version = "0.1.0"
    language = "kleis"
    language_version = "0.95"
    language_info = {
        "name": "kleis",
        "mimetype": "text/x-kleis",
        "file_extension": ".kleis",
        "codemirror_mode": "kleis",
        "pygments_lexer": "kleis",
    }
    banner = "Kleis - Mathematical Specification Language with Z3 Verification"

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._kleis_binary = self._find_kleis_binary()
        self._session_context: list = []  # Accumulated definitions

    def _find_kleis_binary(self) -> Optional[str]:
        """Find the kleis binary in PATH or common locations."""
        # Check PATH first
        kleis_path = shutil.which("kleis")
        if kleis_path:
            return kleis_path

        # Check common installation locations
        home = os.path.expanduser("~")
        candidates = [
            os.path.join(home, ".cargo", "bin", "kleis"),
            "/usr/local/bin/kleis",
            "/usr/bin/kleis",
        ]

        for path in candidates:
            if os.path.isfile(path) and os.access(path, os.X_OK):
                return path

        return None

    def _run_kleis(self, code: str, mode: str = "eval") -> Tuple[int, str, str]:
        """Run kleis binary with the given code."""
        if not self._kleis_binary:
            return (
                1,
                "",
                "Error: Kleis binary not found. Please install Kleis and ensure it's in your PATH.",
            )

        # Create a temporary file with the accumulated context + new code
        full_code = "\n".join(self._session_context + [code])

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".kleis", delete=False
        ) as f:
            f.write(full_code)
            temp_path = f.name

        try:
            cmd = [self._kleis_binary, "test", temp_path]

            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60,
            )

            return (result.returncode, result.stdout, result.stderr)

        except subprocess.TimeoutExpired:
            return (1, "", "Error: Kleis execution timed out (60s limit)")
        except Exception as e:
            return (1, "", f"Error running Kleis: {str(e)}")
        finally:
            os.unlink(temp_path)

    def _format_output(self, stdout: str, stderr: str) -> Dict[str, Any]:
        """Format Kleis output for Jupyter display."""
        output = stdout + stderr

        html_lines = []
        for line in output.split("\n"):
            if not line.strip():
                continue

            if line.startswith("✅") or "passed" in line.lower():
                html_lines.append(
                    f'<div style="color: #28a745; font-family: monospace;">{self._escape_html(line)}</div>'
                )
            elif line.startswith("❌") or "failed" in line.lower():
                html_lines.append(
                    f'<div style="color: #dc3545; font-family: monospace;">{self._escape_html(line)}</div>'
                )
            elif "error" in line.lower():
                html_lines.append(
                    f'<div style="color: #dc3545; font-family: monospace; font-weight: bold;">{self._escape_html(line)}</div>'
                )
            else:
                html_lines.append(
                    f'<div style="font-family: monospace;">{self._escape_html(line)}</div>'
                )

        if html_lines:
            html_content = "\n".join(html_lines)
            return {
                "data": {
                    "text/html": f'<div style="padding: 10px; background: #f8f9fa; border-radius: 4px;">{html_content}</div>',
                    "text/plain": output,
                },
                "metadata": {},
            }
        else:
            return {
                "data": {"text/plain": output or "(no output)"},
                "metadata": {},
            }

    def _escape_html(self, text: str) -> str:
        """Escape HTML special characters."""
        return (
            text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
        )

    def _is_definition(self, code: str) -> bool:
        """Check if the code contains definitions that should persist."""
        definition_patterns = [
            r"^\s*structure\s+",
            r"^\s*data\s+",
            r"^\s*operation\s+",
            r"^\s*define\s+",
            r"^\s*import\s+",
            r"^\s*implements\s+",
            r"^\s*type\s+\w+\s*=",
        ]

        for pattern in definition_patterns:
            if re.search(pattern, code, re.MULTILINE):
                return True
        return False

    def do_execute(
        self,
        code: str,
        silent: bool,
        store_history: bool = True,
        user_expressions: Optional[Dict[str, Any]] = None,
        allow_stdin: bool = False,
        *,
        cell_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Execute Kleis code."""

        # Handle magic commands
        if code.strip().startswith("%"):
            return self._handle_magic(code.strip())

        # Run the code
        returncode, stdout, stderr = self._run_kleis(code)

        # If successful and contains definitions, add to session context
        if returncode == 0 and self._is_definition(code):
            self._session_context.append(code)

        # Format and send output
        if not silent:
            output = self._format_output(stdout, stderr)

            if "text/html" in output.get("data", {}):
                self.send_response(self.iopub_socket, "display_data", output)
            else:
                stream_content = {
                    "name": "stdout" if returncode == 0 else "stderr",
                    "text": stdout + stderr,
                }
                self.send_response(self.iopub_socket, "stream", stream_content)

        return {
            "status": "ok" if returncode == 0 else "error",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }

    def _handle_magic(self, code: str) -> Dict[str, Any]:
        """Handle Jupyter magic commands."""

        if code.startswith("%reset"):
            self._session_context = []
            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stdout", "text": "Session context cleared.\n"},
            )
            return {
                "status": "ok",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }

        elif code.startswith("%context"):
            context_display = (
                "\n---\n".join(self._session_context)
                if self._session_context
                else "(empty)"
            )
            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stdout", "text": f"Current session context:\n{context_display}\n"},
            )
            return {
                "status": "ok",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }

        elif code.startswith("%version"):
            if self._kleis_binary:
                result = subprocess.run(
                    [self._kleis_binary, "--version"],
                    capture_output=True,
                    text=True,
                )
                version_info = result.stdout or result.stderr
            else:
                version_info = "Kleis binary not found"

            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stdout", "text": f"Kleis Kernel v{self.implementation_version}\n{version_info}"},
            )
            return {
                "status": "ok",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }

        else:
            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stderr", "text": f"Unknown magic command: {code}\nAvailable: %reset, %context, %version\n"},
            )
            return {
                "status": "error",
                "execution_count": self.execution_count,
                "ename": "UnknownMagic",
                "evalue": f"Unknown magic command: {code}",
                "traceback": [],
            }

    def do_complete(self, code: str, cursor_pos: int) -> Dict[str, Any]:
        """Provide code completion."""
        code_to_cursor = code[:cursor_pos]
        match = re.search(r"(\w+)$", code_to_cursor)

        if not match:
            return {
                "matches": [],
                "cursor_start": cursor_pos,
                "cursor_end": cursor_pos,
                "metadata": {},
                "status": "ok",
            }

        word = match.group(1)
        cursor_start = cursor_pos - len(word)

        keywords = [
            "structure", "data", "operation", "define", "import", "implements",
            "axiom", "example", "assert", "let", "in", "where", "forall", "exists",
            "if", "then", "else", "match", "with", "true", "false",
            "ℕ", "ℤ", "ℝ", "ℂ", "Bool", "Set", "List", "Matrix", "Vector",
            "eval", "sin", "cos", "exp", "log", "sqrt",
        ]

        matches = [kw for kw in keywords if kw.startswith(word)]

        return {
            "matches": matches,
            "cursor_start": cursor_start,
            "cursor_end": cursor_pos,
            "metadata": {},
            "status": "ok",
        }

    def do_is_complete(self, code: str) -> Dict[str, str]:
        """Check if code is complete."""
        open_braces = code.count("{") - code.count("}")
        open_parens = code.count("(") - code.count(")")

        if open_braces > 0 or open_parens > 0:
            return {"status": "incomplete", "indent": "    "}
        else:
            return {"status": "complete"}


if __name__ == "__main__":
    from ipykernel.kernelapp import IPKernelApp
    IPKernelApp.launch_instance(kernel_class=KleisKernel)
