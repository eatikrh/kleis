"""
Kleis Numeric Jupyter Kernel

This kernel runs an interactive Kleis REPL subprocess for concrete
numerical computation (eigenvalues, SVD, matrix operations, etc.)

Unlike the symbolic kernel that uses one-shot `kleis eval` calls,
this kernel maintains a persistent REPL session that can:
- Compute eigenvalues, SVD, matrix inverses
- Maintain state between cells  
- Use LAPACK for numerical linear algebra
"""

import os
import re
import subprocess
import threading
import queue
from typing import Any, Dict, Optional, Tuple

from ipykernel.kernelbase import Kernel

# Import kleis_binary module - handle both package and direct import
try:
    from .kleis_binary import find_kleis_binary
except ImportError:
    from kleis_binary import find_kleis_binary


class KleisNumericKernel(Kernel):
    """Jupyter kernel for Kleis numerical computation via REPL."""

    implementation = "Kleis Numeric"
    implementation_version = "0.1.0"
    language = "kleis"
    language_version = "0.95"
    language_info = {
        "name": "kleis-numeric",
        "mimetype": "text/x-kleis",
        "file_extension": ".kleis",
        "codemirror_mode": "kleis",
        "pygments_lexer": "kleis",
    }
    banner = """Kleis Numeric - Concrete Computation via REPL

This kernel runs a persistent Kleis REPL for numerical operations:
- eigenvalues([[1,2],[3,4]]) → computed values
- svd(matrix) → U, S, V decomposition  
- inv(matrix) → matrix inverse
- det(matrix) → determinant

REPL Commands:
  :eval <expr>    - Evaluate expression concretely
  :type <expr>    - Show inferred type
  :verify <expr>  - Verify with Z3
  :env            - Show session context
  :load <file>    - Load .kleis file
  :help           - Show all commands

Jupyter Commands:
  %reset          - Restart REPL session
  %version        - Show version info
"""

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Use shared discovery module for consistent behavior across all Python code
        self._kleis_binary = find_kleis_binary()
        self._repl_process = None
        self._output_queue = queue.Queue()
        self._reader_thread = None
        self._start_repl()

    def _start_repl(self):
        """Start the Kleis REPL subprocess."""
        if not self._kleis_binary:
            return

        try:
            self._repl_process = subprocess.Popen(
                [self._kleis_binary, "repl"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,  # Line buffered
            )

            # Start reader thread
            self._reader_thread = threading.Thread(
                target=self._read_output, daemon=True
            )
            self._reader_thread.start()

            # Wait for initial banner
            self._wait_for_prompt(timeout=5.0)

        except Exception as e:
            self._repl_process = None
            print(f"Failed to start REPL: {e}")

    def _read_output(self):
        """Background thread to read REPL output."""
        try:
            while self._repl_process and self._repl_process.poll() is None:
                line = self._repl_process.stdout.readline()
                if line:
                    self._output_queue.put(line)
        except Exception:
            pass

    def _wait_for_prompt(self, timeout: float = 10.0) -> str:
        """Wait for the REPL prompt and collect output."""
        output_lines = []
        import time
        start = time.time()

        while time.time() - start < timeout:
            try:
                line = self._output_queue.get(timeout=0.1)
                output_lines.append(line)
                # Check if we got the prompt
                if line.strip().endswith("λ>") or line.strip() == "λ>":
                    break
                # Also check for continuation prompt
                if line.strip() == "":
                    # Empty line might indicate end of output
                    # Give a bit more time for prompt
                    try:
                        line = self._output_queue.get(timeout=0.2)
                        output_lines.append(line)
                        if "λ>" in line:
                            break
                    except queue.Empty:
                        break
            except queue.Empty:
                # Check if there's more coming
                if output_lines:
                    # We have some output, wait a bit more
                    continue
                else:
                    continue

        return "".join(output_lines)

    def _send_command(self, command: str) -> str:
        """Send a command to the REPL and get the response."""
        if not self._repl_process or self._repl_process.poll() is not None:
            # REPL died, try to restart
            self._start_repl()
            if not self._repl_process:
                return "Error: Kleis REPL not available"

        # Clear any pending output
        while not self._output_queue.empty():
            try:
                self._output_queue.get_nowait()
            except queue.Empty:
                break

        # Send command
        try:
            self._repl_process.stdin.write(command + "\n")
            self._repl_process.stdin.flush()
        except Exception as e:
            return f"Error sending command: {e}"

        # Wait for response
        output = self._wait_for_prompt(timeout=30.0)

        # Clean up output - remove the command echo and prompt
        lines = output.split("\n")
        result_lines = []
        for line in lines:
            # Skip the command echo
            if line.strip() == command.strip():
                continue
            # Skip prompts
            if line.strip() in ("λ>", "   "):
                continue
            if line.strip().startswith("λ>"):
                # Extract any content after prompt
                after = line.split("λ>", 1)[1].strip()
                if after:
                    result_lines.append(after)
                continue
            result_lines.append(line)

        return "\n".join(result_lines).strip()

    def _format_output(self, output: str) -> Dict[str, Any]:
        """Format REPL output for Jupyter display."""
        if not output:
            return {
                "data": {"text/plain": "(no output)"},
                "metadata": {},
            }

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
            elif line.startswith("→") or line.startswith("⇒"):
                # Result lines
                html_lines.append(
                    f'<div style="color: #0066cc; font-family: monospace; font-weight: bold;">{self._escape_html(line)}</div>'
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
                "data": {"text/plain": output},
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
        """Execute Kleis code via REPL."""

        code_stripped = code.strip()

        # Handle Jupyter magic commands
        if code_stripped.startswith("%"):
            return self._handle_magic(code_stripped)

        if not self._repl_process:
            error_msg = "Error: Kleis REPL not available. Please install Kleis with 'cargo install --path . --features numerical'"
            if not silent:
                self.send_response(
                    self.iopub_socket,
                    "stream",
                    {"name": "stderr", "text": error_msg + "\n"},
                )
            return {
                "status": "error",
                "execution_count": self.execution_count,
                "ename": "ReplNotAvailable",
                "evalue": error_msg,
                "traceback": [],
            }

        # For expressions, wrap in :eval for concrete evaluation
        # Unless it's already a REPL command
        if code_stripped.startswith(":"):
            command = code_stripped
        else:
            # Wrap in :eval for concrete numerical evaluation
            command = f":eval {code_stripped}"

        # Send to REPL
        output = self._send_command(command)

        # Format and send output
        if not silent:
            formatted = self._format_output(output)
            if "text/html" in formatted.get("data", {}):
                self.send_response(self.iopub_socket, "display_data", formatted)
            else:
                self.send_response(
                    self.iopub_socket,
                    "stream",
                    {"name": "stdout", "text": output + "\n"},
                )

        # Check for errors
        is_error = "error" in output.lower() and not "no error" in output.lower()

        return {
            "status": "error" if is_error else "ok",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }

    def _handle_magic(self, code: str) -> Dict[str, Any]:
        """Handle Jupyter magic commands."""

        if code.startswith("%reset"):
            # Restart REPL
            if self._repl_process:
                self._repl_process.terminate()
                self._repl_process.wait()
            self._start_repl()
            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stdout", "text": "REPL session restarted.\n"},
            )
            return {
                "status": "ok",
                "execution_count": self.execution_count,
                "payload": [],
                "user_expressions": {},
            }

        elif code.startswith("%version"):
            version_info = f"Kleis Numeric Kernel v{self.implementation_version}\n"
            if self._kleis_binary:
                result = subprocess.run(
                    [self._kleis_binary, "--version"],
                    capture_output=True,
                    text=True,
                )
                version_info += result.stdout or result.stderr
            else:
                version_info += "Kleis binary not found"

            self.send_response(
                self.iopub_socket,
                "stream",
                {"name": "stdout", "text": version_info},
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
                {"name": "stderr", "text": f"Unknown magic command: {code}\nAvailable: %reset, %version\n"},
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

        # Numerical operations first
        keywords = [
            # Numerical operations (LAPACK)
            "eigenvalues", "eigvals", "eig", "svd", "inv", "det", "trace",
            "qr", "cholesky", "lu", "solve", "rank", "cond", "norm", "expm",
            "schur",
            # Matrix operations
            "Matrix", "transpose", "matmul", "eye", "zeros", "ones",
            # REPL commands
            ":eval", ":type", ":verify", ":ast", ":env", ":load", ":help", ":quit",
            # Types
            "ℕ", "ℤ", "ℝ", "ℂ", "Bool", "Set", "List", "Vector",
            # Language keywords
            "structure", "data", "operation", "define", "import", "implements",
            "axiom", "example", "assert", "let", "in", "forall", "exists",
            "if", "then", "else", "true", "false",
            # Math functions
            "sin", "cos", "exp", "log", "sqrt", "abs",
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
        open_brackets = code.count("[") - code.count("]")

        if open_braces > 0 or open_parens > 0 or open_brackets > 0:
            return {"status": "incomplete", "indent": "    "}
        else:
            return {"status": "complete"}

    def do_shutdown(self, restart: bool):
        """Clean up on shutdown."""
        if self._repl_process:
            try:
                self._repl_process.stdin.write(":quit\n")
                self._repl_process.stdin.flush()
                self._repl_process.wait(timeout=2)
            except Exception:
                self._repl_process.terminate()
        return {"status": "ok", "restart": restart}


if __name__ == "__main__":
    from ipykernel.kernelapp import IPKernelApp
    IPKernelApp.launch_instance(kernel_class=KleisNumericKernel)

