# IDE Integration Plan

> **Status: ✅ MOSTLY IMPLEMENTED** (Jan 2026)  
> - ✅ **LSP Server**: Fully featured (~3800 lines) - diagnostics, hover, go-to-def, completions, etc.
> - ✅ **VS Code Extension**: Syntax highlighting, LSP client, file watcher
> - ✅ **DAP Debugger**: Fully working, integrated with unified `kleis server`
> - ✅ **REPL Panel**: Implemented in VS Code
> - ❌ **Preview Panel**: Not implemented (rich math rendering in VS Code)
> - ✅ **Notebook Mode**: Jupyter kernel exists (`kleis-notebook/`)

---

## Current State (Dec 23, 2024)

### What We Have

**VS Code Extension (`vscode-kleis/`):**
- ✅ Syntax highlighting (TextMate grammar)
- ✅ LSP client connecting to `kleis-lsp`
- ✅ File watcher for `.kleis` files
- ✅ Version 0.3.0, ready for marketplace

**LSP Server (`src/bin/lsp.rs`):**
- ✅ ~3800 lines, feature-rich implementation
- ✅ Diagnostics (parse errors, type errors)
- ✅ Hover (type signatures, documentation)
- ✅ Go to Definition
- ✅ Find References
- ✅ Document Symbols (outline)
- ✅ Workspace Symbols
- ✅ Completion (context-aware)
- ✅ Signature Help
- ✅ Inlay Hints (type annotations)
- ✅ Formatting
- ✅ Rename
- ✅ Code Actions
- ✅ Semantic Tokens
- ✅ Folding Ranges
- ✅ Code Lens
- ✅ Implementation Provider

### What's Missing

| Feature | Description | Priority |
|---------|-------------|----------|
| **REPL Panel** | Interactive webview for REPL commands | High |
| **Preview Panel** | Render expressions to formatted math | Medium |
| **File → REPL sync** | Notify REPL when files change | Medium |
| **Notebook Mode** | Jupyter-like cells for Kleis | Low (future) |

---

## Architecture Options for REPL Panel

### Option A: Embedded Terminal
- Use VS Code's integrated terminal
- Run `cargo run --bin repl` directly
- **Pros:** Simple, already works
- **Cons:** No rich formatting, plain text only

### Option B: Webview Panel
- Custom webview with REPL-like interface
- Rich rendering (HTML/SVG for math expressions)
- Communicate with `kleis-repl` process via stdin/stdout
- **Pros:** Beautiful, can show rendered math
- **Cons:** More complex to implement

### Option C: Pseudo-Terminal (Recommended)
- Use `vscode.window.createTerminal` with a Pty
- Control the REPL process programmatically
- Intercept output for rich rendering
- **Pros:** Best of both worlds
- **Cons:** Moderate complexity

---

## Recommended Implementation: Webview REPL Panel

### Phase 1: Basic Webview Shell

```
┌─────────────────────────────────────────┐
│  Kleis REPL                        [×]  │
├─────────────────────────────────────────┤
│  λ> :load stdlib/matrices.kleis        │
│  ✅ Loaded 5 functions, 2 structures    │
│                                         │
│  λ> :let A = Matrix(2, 2, [1,0,0,1])   │
│  A = ⌈ 1  0 ⌉                          │
│      ⌊ 0  1 ⌋                          │
│                                         │
│  λ> multiply(A, A)                      │
│  ⌈ 1  0 ⌉                              │
│  ⌊ 0  1 ⌋                              │
│                                         │
├─────────────────────────────────────────┤
│  λ> █                                   │
└─────────────────────────────────────────┘
```

**Features:**
- Input field at bottom
- Scrollable history above
- Basic text output initially

### Phase 2: Rich Rendering

- Render matrices as HTML tables
- Use MathJax/KaTeX for mathematical notation
- Syntax highlighting for Kleis code
- Clickable links for definitions

### Phase 3: Integration

- Sync with editor: "Run selection in REPL"
- File change notifications: "matrices.kleis modified"
- Export to file: Save REPL session

---

## Implementation Plan

### Step 1: Add Webview Contribution

**`vscode-kleis/package.json`:**
```json
{
  "contributes": {
    "viewsContainers": {
      "panel": [{
        "id": "kleis-repl",
        "title": "Kleis REPL",
        "icon": "$(terminal)"
      }]
    },
    "views": {
      "kleis-repl": [{
        "type": "webview",
        "id": "kleis.replPanel",
        "name": "REPL"
      }]
    },
    "commands": [{
      "command": "kleis.openRepl",
      "title": "Kleis: Open REPL"
    }, {
      "command": "kleis.runSelection",
      "title": "Kleis: Run Selection in REPL"
    }]
  }
}
```

### Step 2: Create ReplPanel Class

**`vscode-kleis/src/replPanel.ts`:**
```typescript
import * as vscode from 'vscode';
import { spawn, ChildProcess } from 'child_process';

export class ReplPanel {
    private panel: vscode.WebviewPanel;
    private replProcess: ChildProcess | null = null;
    private history: string[] = [];

    constructor(context: vscode.ExtensionContext) {
        this.panel = vscode.window.createWebviewPanel(
            'kleisRepl',
            'Kleis REPL',
            vscode.ViewColumn.Two,
            { enableScripts: true }
        );
        this.panel.webview.html = this.getHtml();
        this.startRepl();
        this.setupMessageHandling();
    }

    private startRepl() {
        // Find kleis-repl binary (similar to findServer)
        const replPath = this.findRepl();
        if (replPath) {
            this.replProcess = spawn(replPath, [], {
                stdio: ['pipe', 'pipe', 'pipe']
            });
            this.replProcess.stdout?.on('data', (data) => {
                this.sendToWebview(data.toString());
            });
        }
    }

    private sendCommand(cmd: string) {
        if (this.replProcess?.stdin) {
            this.replProcess.stdin.write(cmd + '\n');
            this.history.push(cmd);
        }
    }
    // ...
}
```

### Step 3: Webview HTML/JS

Create interactive frontend with:
- Monaco editor for input (with Kleis syntax)
- Output area with rich formatting
- History navigation (up/down arrows)

### Step 4: REPL Protocol Enhancement

Modify `src/bin/repl.rs` to support:
- JSON output mode (`--json`) for structured data
- Expression type in output
- Rendered output alongside raw

```rust
// New output format when --json flag
{
    "type": "result",
    "value": "Matrix(2, 2, [1, 0, 0, 1])",
    "rendered": "<table>...</table>",
    "inferred_type": "Matrix(2, 2, ℝ)"
}
```

---

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `vscode-kleis/package.json` | Modify | Add webview contributions |
| `vscode-kleis/src/replPanel.ts` | Create | Webview panel class |
| `vscode-kleis/src/extension.ts` | Modify | Register commands, create panel |
| `vscode-kleis/media/repl.html` | Create | Webview HTML |
| `vscode-kleis/media/repl.js` | Create | Webview JavaScript |
| `vscode-kleis/media/repl.css` | Create | Webview styling |
| `src/bin/repl.rs` | Modify | Add `--json` output mode |

---

## Success Criteria

1. **Basic:** User can open REPL panel and type commands
2. **Interactive:** Input history, autocomplete from LSP
3. **Rich:** Matrices and math rendered beautifully
4. **Integrated:** "Run Selection" from editor works
5. **Synced:** File changes notify the REPL

---

## References

- [VS Code Webview API](https://code.visualstudio.com/api/extension-guides/webview)
- [Creating a Webview Panel](https://code.visualstudio.com/api/extension-guides/webview#creating-a-webview-panel)
- [Message Passing](https://code.visualstudio.com/api/extension-guides/webview#passing-messages-from-an-extension-to-a-webview)

