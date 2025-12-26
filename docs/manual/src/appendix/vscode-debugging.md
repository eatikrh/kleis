# VS Code Debugging

This appendix explains how to set up and use the VS Code debugger with Kleis.

## Prerequisites

1. **VS Code** installed
2. **Kleis extension** installed (provides syntax highlighting + debugging)
3. **Kleis binaries** built:
   ```bash
   cargo build --release --bin kleis --bin kleis-lsp
   ```

## Extension Setup

The Kleis VS Code extension is located in `kleis-vscode/`. Install it:

```bash
cd kleis-vscode
npm install
npm run compile
code --install-extension kleis-0.1.0.vsix
```

Or for development, open the extension folder in VS Code and press F5 to launch an Extension Development Host.

## Launch Configuration

Create `.vscode/launch.json` in your project:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "kleis",
            "request": "launch",
            "name": "Debug Kleis File",
            "program": "${file}",
            "stopOnEntry": false
        }
    ]
}
```

### Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `program` | string | Path to .kleis file to debug |
| `stopOnEntry` | boolean | Stop at first line (default: false) |

## Setting Breakpoints

Click in the gutter (left margin) next to any line in an example block:

```kleis
example "my test" {
    let x = 5          // ← Click here to set breakpoint
    let y = double(x)  // ← Or here
    assert(y = 10)
}
```

**Note:** Breakpoints only work on executable lines inside example blocks. Function definitions are declarations, not executable code.

## Starting a Debug Session

1. Open a `.kleis` file with example blocks
2. Set breakpoints on lines you want to inspect
3. Press **F5** or click **Run → Start Debugging**
4. Select "Debug Kleis File" configuration

## Debug Controls

| Key | Action | Description |
|-----|--------|-------------|
| F5 | Continue | Run until next breakpoint |
| F10 | Step Over | Execute current line, don't enter functions |
| F11 | Step Into | Enter function calls |
| Shift+F11 | Step Out | Finish current function, return to caller |
| Shift+F5 | Stop | End debug session |

## Inspecting Variables

The **Variables** panel (left sidebar) shows:

- **Local variables** — Let bindings in current scope
- **Function parameters** — Arguments passed to current function

Example:

```kleis
example "inspection demo" {
    let x = 5
    let y = 10
    // Breakpoint here shows: x = 5, y = 10
    let sum = x + y
}
```

## Call Stack

The **Call Stack** panel shows the execution path:

```
fibonacci (n=5)          ← Currently here
fibonacci (n=6)
example "fib test"       ← Entry point
```

Click any frame to see its local variables and source location.

## Cross-File Debugging

When stepping into imported functions, VS Code opens the source file:

```kleis
// main.kleis
import "helpers.kleis"

example "cross-file" {
    let result = helper_function(5)  // ← Step Into (F11)
    // VS Code opens helpers.kleis at helper_function definition
}
```

### How It Works

Every expression carries its **source location** (line, column, file path). When the evaluator processes an expression from an imported file, the debugger reports that file's location to VS Code, which opens it automatically.

## Debug Console

The Debug Console (bottom panel) shows:

- Evaluation progress
- Assertion results (pass/fail)
- Error messages

You can also evaluate expressions in the console during a paused debug session.

## Troubleshooting

### Breakpoints Not Hitting

**Problem:** Breakpoint shows as gray (unverified) or never hits.

**Solutions:**
1. Ensure breakpoint is on a line inside an example block
2. Ensure the example block is actually executed
3. Rebuild the Kleis binaries: `cargo build --release`

### "File not found" Errors

**Problem:** Debugger can't find imported files.

**Solutions:**
1. Use relative paths in imports: `import "stdlib/complex.kleis"`
2. Run debug session from the project root directory
3. Check that the imported file exists at the specified path

### Slow Stepping

**Problem:** Each step takes several seconds.

**Solutions:**
1. Use release builds: `cargo build --release`
2. Avoid stepping through deeply recursive functions
3. Use "Step Over" (F10) instead of "Step Into" (F11) for library functions

### Debug Session Won't Start

**Problem:** F5 does nothing or shows an error.

**Solutions:**
1. Check `.vscode/launch.json` exists and is valid JSON
2. Ensure Kleis extension is installed and enabled
3. Check the Output panel (View → Output → Kleis) for error messages
4. Verify binaries exist: `ls target/release/kleis*`

## Architecture

The debugging system uses the **Debug Adapter Protocol (DAP)**:

```
┌─────────────┐      DAP         ┌──────────────┐
│   VS Code   │ ←───────────────→ │ kleis server │
│   (client)  │    JSON-RPC      │   (adapter)  │
└─────────────┘                   └──────────────┘
                                        │
                                        ▼
                                  ┌──────────────┐
                                  │  Evaluator   │
                                  │  + DebugHook │
                                  └──────────────┘
```

1. VS Code sends DAP commands (setBreakpoints, next, stepIn, etc.)
2. Kleis server translates to evaluator debug hook calls
3. Evaluator pauses at breakpoints, reports current expression's span
4. Server sends stopped events with location (line, column, file)
5. VS Code highlights the current line

### Source Span Tracking

The key to accurate debugging is **SourceSpan**:

```rust
pub struct SourceSpan {
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub file: Option<Arc<PathBuf>>,  // File path (Arc for cheap cloning)
}
```

Every `Expression` node has an optional span. The parser attaches the span during parsing. When evaluating, the span travels with the expression, so the debugger always knows the source location.

## Tips for Effective Debugging

1. **Start with simple examples** — Debug small example blocks first
2. **Use Step Over for library code** — Don't step into stdlib functions unless needed
3. **Watch the Variables panel** — See how values change as you step
4. **Set multiple breakpoints** — Mark key points in your logic
5. **Use the Call Stack** — Understand how you got to the current line

## See Also

- [Example Blocks](../chapters/20-example-blocks.md) — How to write debuggable code
- [The REPL](../chapters/12-repl.md) — Interactive exploration
- [Grammar Reference](./grammar.md) — Full language syntax

