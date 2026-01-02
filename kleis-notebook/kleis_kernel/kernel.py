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
    language_version = "0.96"
    language_info = {
        "name": "kleis",
        "mimetype": "text/x-kleis",
        "file_extension": ".kleis",
        "codemirror_mode": "kleis",
        "pygments_lexer": "kleis",
    }
    banner = """Kleis v0.96 - Mathematical Specification Language with Z3 Verification

Plotting (Lilaq-style compositional API - https://lilaq.org):
  diagram(
      plot(xs, ys, color = "blue", yerr = errors),
      bar(xs, heights, offset = -0.2, width = 0.4, label = "Series A"),
      width = 10, height = 7, title = "My Plot"
  )
  
  Elements: plot, scatter, bar, hbar, stem, hstem, fill_between,
            boxplot, hboxplot, heatmap, contour, quiver
  
  Named arguments (v0.96): name = value syntax for options
  List functions: list_map(λ x . f(x), xs), list_filter, list_fold

REPL Commands:
  :type <expr>    - Show inferred type
  :eval <expr>    - Evaluate expression
  :verify <expr>  - Verify with Z3
  :env            - Show session context
  :load <file>    - Load .kleis file

Jupyter Commands:
  %reset          - Clear session
  %context        - Show accumulated definitions
  %version        - Show version info
"""

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._kleis_binary = self._find_kleis_binary()
        self._kleis_root = self._find_kleis_root()
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

    def _find_kleis_root(self) -> Optional[str]:
        """Find the Kleis project root directory (where stdlib/ is located).
        
        This is needed so that imports like 'stdlib/prelude.kleis' work correctly.
        Search order:
        1. KLEIS_ROOT environment variable
        2. Directory containing the kleis binary (if installed in project)
        3. Common project locations
        4. Current working directory if it has stdlib/
        """
        # Check environment variable first
        env_root = os.environ.get("KLEIS_ROOT")
        if env_root and os.path.isdir(os.path.join(env_root, "stdlib")):
            return env_root
        
        # Check if kleis binary is in a project directory
        if self._kleis_binary:
            # kleis binary might be in ~/.cargo/bin, so we can't use its location directly
            # But we can check if KLEIS_ROOT was set during install
            pass
        
        # Check common project locations
        home = os.path.expanduser("~")
        candidates = [
            os.path.join(home, "git", "cee", "kleis"),
            os.path.join(home, "projects", "kleis"),
            os.path.join(home, "kleis"),
            "/opt/kleis",
            "/usr/local/share/kleis",
        ]
        
        for path in candidates:
            if os.path.isdir(os.path.join(path, "stdlib")):
                return path
        
        # Check current working directory
        cwd = os.getcwd()
        if os.path.isdir(os.path.join(cwd, "stdlib")):
            return cwd
        
        # Check parent directories (in case we're in a subdirectory)
        parent = os.path.dirname(cwd)
        for _ in range(3):  # Check up to 3 levels up
            if os.path.isdir(os.path.join(parent, "stdlib")):
                return parent
            parent = os.path.dirname(parent)
        
        return None

    def _run_kleis(self, code: str, mode: str = "auto") -> Tuple[int, str, str]:
        """Run kleis binary with the given code.
        
        Modes:
        - "auto": Detect whether to use eval or test based on content
        - "eval": Use kleis eval for expression evaluation
        - "test": Use kleis test for example blocks
        """
        if not self._kleis_binary:
            return (
                1,
                "",
                "Error: Kleis binary not found. Please install Kleis and ensure it's in your PATH.",
            )

        # Auto-detect mode based on content
        if mode == "auto":
            if re.search(r'^\s*example\s+', code, re.MULTILINE):
                mode = "test"
            elif self._is_definition(code):
                mode = "test"  # Definitions need test mode to validate
            else:
                mode = "eval"

        if mode == "eval":
            return self._run_kleis_eval(code)
        else:
            return self._run_kleis_test(code)

    def _run_kleis_eval(self, code: str) -> Tuple[int, str, str]:
        """Run kleis eval for expression evaluation."""
        # For eval, we need to load context file first, then evaluate expression
        # Create temp file with context
        context_code = "\n".join(self._session_context)
        
        try:
            if context_code.strip():
                # Write context to temp file
                with tempfile.NamedTemporaryFile(
                    mode="w", suffix=".kleis", delete=False
                ) as f:
                    f.write(context_code)
                    context_path = f.name
                
                # Run: kleis eval -f context.kleis "expression"
                cmd = [self._kleis_binary, "eval", "-f", context_path, code.strip()]
            else:
                # No context, just evaluate expression
                cmd = [self._kleis_binary, "eval", code.strip()]

            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60,
                cwd=self._kleis_root,  # Run from kleis root so stdlib imports work
            )
            
            return (result.returncode, result.stdout, result.stderr)

        except subprocess.TimeoutExpired:
            return (1, "", "Error: Kleis evaluation timed out (60s limit)")
        except Exception as e:
            return (1, "", f"Error running Kleis: {str(e)}")
        finally:
            if context_code.strip() and 'context_path' in locals():
                try:
                    os.unlink(context_path)
                except:
                    pass

    def _run_kleis_test(self, code: str) -> Tuple[int, str, str]:
        """Run kleis test for example blocks and definitions."""
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
                cwd=self._kleis_root,  # Run from kleis root so stdlib imports work
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

        # Check for SVG plot output (from diagram(), plot(), scatter() etc.)
        if "PLOT_SVG:" in output:
            # Extract SVG content - find the complete SVG element
            svg_start = output.index("PLOT_SVG:") + 9
            svg_content = output[svg_start:]
            
            # Find the end of the SVG (</svg> tag)
            svg_end = svg_content.find("</svg>")
            if svg_end != -1:
                svg_data = svg_content[:svg_end + 6].strip()  # +6 for "</svg>"
            else:
                svg_data = svg_content.strip()
            
            # Also get any text before the SVG (like out() messages)
            text_before = output[:output.index("PLOT_SVG:")].strip()
            
            result = {
                "data": {
                    "image/svg+xml": svg_data,
                    "text/plain": "[Plot]"
                },
                "metadata": {}
            }
            
            return result

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
            elif line.startswith("[") or line.startswith("Matrix(") or self._is_numeric_result(line):
                # Highlight out() results (lists, matrices, numbers)
                html_lines.append(
                    f'<div style="color: #0066cc; font-family: monospace; font-weight: bold; padding: 4px; background: #e6f2ff; border-radius: 3px; margin: 2px 0;">{self._escape_html(line)}</div>'
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

    def _is_numeric_result(self, line: str) -> bool:
        """Check if line looks like a numeric result from out()."""
        line = line.strip()
        # Check for plain numbers (int or float)
        try:
            float(line)
            return True
        except ValueError:
            pass
        # Check for negative numbers
        if line.startswith("-"):
            try:
                float(line[1:])
                return True
            except ValueError:
                pass
        return False

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

        code_stripped = code.strip()

        # Handle Jupyter magic commands (% prefix)
        if code_stripped.startswith("%"):
            return self._handle_magic(code_stripped)

        # Handle Kleis REPL commands (: prefix)
        if code_stripped.startswith(":"):
            returncode, stdout, stderr = self._handle_repl_command(code_stripped)
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

        # Run the code (auto-detect mode: eval vs test)
        returncode, stdout, stderr = self._run_kleis(code)

        # If successful and contains definitions, add to session context
        if returncode == 0 and self._is_definition(code):
            self._session_context.append(code)

        # Format and send output
        if not silent:
            combined = stdout + stderr
            
            # Check for SVG plot output - handle MULTIPLE plots
            if "PLOT_SVG:" in combined:
                # Find all PLOT_SVG markers and extract each plot
                remaining = combined
                
                while "PLOT_SVG:" in remaining:
                    svg_idx = remaining.index("PLOT_SVG:")
                    text_before = remaining[:svg_idx].strip()
                    svg_raw = remaining[svg_idx + 9:]  # Everything after PLOT_SVG:
                    
                    # Find the end of this SVG (</svg> tag)
                    svg_end_idx = svg_raw.find("</svg>")
                    if svg_end_idx != -1:
                        svg_data = svg_raw[:svg_end_idx + 6].strip()  # +6 for "</svg>"
                        remaining = svg_raw[svg_end_idx + 6:]  # Continue after this SVG
                    else:
                        # No closing tag found - use all remaining content
                        svg_data = svg_raw.strip()
                        remaining = ""
                    
                    # Send any text output before this plot
                    if text_before:
                        # Filter out test summary lines for cleaner output
                        text_lines = [
                            line for line in text_before.split("\n")
                            if not line.strip().startswith("✅") 
                            and not line.strip().startswith("❌")
                            and "examples passed" not in line
                        ]
                        filtered_text = "\n".join(text_lines).strip()
                        if filtered_text:
                            self.send_response(self.iopub_socket, "stream", {
                                "name": "stdout",
                                "text": filtered_text + "\n"
                            })
                    
                    # Send the SVG as display_data
                    self.send_response(self.iopub_socket, "display_data", {
                        "data": {
                            "image/svg+xml": svg_data,
                            "text/plain": "[Plot]"
                        },
                        "metadata": {}
                    })
                
                # Send any remaining text after the last plot
                if remaining.strip():
                    # Filter test summary
                    text_lines = [
                        line for line in remaining.split("\n")
                        if not line.strip().startswith("✅") 
                        and not line.strip().startswith("❌")
                        and "examples passed" not in line
                    ]
                    filtered_text = "\n".join(text_lines).strip()
                    if filtered_text:
                        self.send_response(self.iopub_socket, "stream", {
                            "name": "stdout",
                            "text": filtered_text + "\n"
                        })
            else:
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

    def _handle_repl_command(self, code: str) -> Tuple[int, str, str]:
        """Handle Kleis REPL-style commands (:type, :eval, :verify, etc.)."""
        code = code.strip()
        
        # Parse the command
        if code.startswith(":type "):
            expr = code[6:].strip()
            # Use kleis eval with type checking - we need to wrap in a way that shows type
            # For now, use the check command approach
            return self._run_repl_style("type", expr)
        elif code.startswith(":eval "):
            expr = code[6:].strip()
            return self._run_repl_style("eval", expr)
        elif code.startswith(":verify "):
            expr = code[8:].strip()
            return self._run_repl_style("verify", expr)
        elif code.startswith(":ast "):
            expr = code[5:].strip()
            return self._run_repl_style("ast", expr)
        elif code.startswith(":env"):
            return self._show_environment()
        elif code.startswith(":load "):
            filepath = code[6:].strip()
            return self._load_file(filepath)
        else:
            return (1, "", f"Unknown REPL command: {code.split()[0]}\nAvailable: :type, :eval, :verify, :ast, :env, :load")

    def _run_repl_style(self, cmd: str, expr: str) -> Tuple[int, str, str]:
        """Run a REPL-style command by creating a temp file and using kleis."""
        # Build context + command wrapper
        context = "\n".join(self._session_context)
        
        if cmd == "eval":
            # For eval, just evaluate the expression
            full_code = f"{context}\n\n// Evaluate expression\n" if context else ""
            return self._run_kleis_eval(expr)
        elif cmd == "type":
            # For type, we need to use the kleis binary with a special approach
            # Create a file that will show type info
            full_code = f"{context}\n\nexample \"type_check\" {{\n    let _result = {expr}\n    assert(true)\n}}"
        elif cmd == "verify":
            full_code = f"{context}\n\nexample \"verify\" {{\n    assert({expr})\n}}"
        elif cmd == "ast":
            # AST display - just try to parse and show structure
            full_code = f"{context}\n\nexample \"ast\" {{\n    let _ast = {expr}\n    assert(true)\n}}"
        else:
            return (1, "", f"Unknown command: {cmd}")

        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".kleis", delete=False
        ) as f:
            f.write(full_code)
            temp_path = f.name

        try:
            result = subprocess.run(
                [self._kleis_binary, "test", temp_path],
                capture_output=True,
                text=True,
                timeout=60,
            )
            return (result.returncode, result.stdout, result.stderr)
        except Exception as e:
            return (1, "", f"Error: {str(e)}")
        finally:
            os.unlink(temp_path)

    def _show_environment(self) -> Tuple[int, str, str]:
        """Show current session environment (defined functions, structures, etc.)."""
        if not self._session_context:
            return (0, "Session context is empty. Define some structures or functions first.", "")
        
        output_lines = ["📦 Session Context:\n"]
        for i, block in enumerate(self._session_context, 1):
            # Extract first line as summary
            first_line = block.strip().split('\n')[0][:60]
            output_lines.append(f"  [{i}] {first_line}...")
        
        return (0, "\n".join(output_lines), "")

    def _load_file(self, filepath: str) -> Tuple[int, str, str]:
        """Load a .kleis file into the session context."""
        try:
            # Resolve path
            if not os.path.isabs(filepath):
                # Try relative to current directory
                if not os.path.exists(filepath):
                    return (1, "", f"File not found: {filepath}")
            
            with open(filepath, 'r') as f:
                content = f.read()
            
            # Validate by running kleis check
            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".kleis", delete=False
            ) as tf:
                tf.write(content)
                temp_path = tf.name
            
            try:
                result = subprocess.run(
                    [self._kleis_binary, "check", temp_path],
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                
                if result.returncode != 0:
                    return (result.returncode, "", f"Error loading {filepath}:\n{result.stderr}")
                
                # Add to context
                self._session_context.append(f"// Loaded from: {filepath}\n{content}")
                return (0, f"✅ Loaded {filepath}", "")
                
            finally:
                os.unlink(temp_path)
                
        except Exception as e:
            return (1, "", f"Error loading file: {str(e)}")

    def _handle_magic(self, code: str) -> Dict[str, Any]:
        """Handle Jupyter magic commands (% prefix)."""

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
            # Language keywords
            "structure", "data", "operation", "define", "import", "implements",
            "axiom", "example", "assert", "let", "in", "where", "forall", "exists",
            "if", "then", "else", "match", "with", "true", "false",
            # Types
            "ℕ", "ℤ", "ℝ", "ℂ", "Bool", "Set", "List", "Matrix", "Vector",
            # Built-in functions
            "eval", "sin", "cos", "exp", "log", "sqrt", "det", "trace", "transpose",
            "eigenvalues", "eigenvectors", "inverse", "norm", "negate", "out",
            # List functions (v0.96)
            "list_map", "list_filter", "list_fold",
            # Plotting - Lilaq-style compositional API
            "diagram", "plot", "scatter", "bar", "hbar", "stem", "hstem",
            "fill_between", "boxplot", "hboxplot", "heatmap", "contour", "quiver",
            # Plot named arguments (v0.96 - common options)
            "width", "height", "title", "xlabel", "ylabel", "label",
            "color", "stroke", "mark", "mark_size", "yerr", "xerr",
            "offset", "legend_position",
            # REPL commands
            ":type", ":eval", ":verify", ":ast", ":env", ":load",
            # Jupyter magic commands
            "%reset", "%context", "%version",
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
