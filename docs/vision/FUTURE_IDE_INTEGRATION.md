# Future Vision: IDE Integration

## Status
**Future Vision** - Not for immediate implementation (3-5 years out)

## Context

Once the core Kleis system is mature (type system, extensibility, evaluation engine), consider IDE integration for professional mathematical development environments.

---

## Target Platforms

### VS Code Extension
**Most realistic target**

```
VS Code Marketplace
├── Kleis Language Support
│   ├── Syntax highlighting (.kleis files)
│   ├── IntelliSense (autocomplete)
│   ├── Type hints on hover
│   ├── Go to definition
│   └── Error squiggles
│
└── Kleis Notebook Viewer
    ├── Render .kleis notebooks
    ├── Inline equation preview
    ├── Cell execution
    └── Export commands
```

### JetBrains Plugin
**For IntelliJ, PyCharm, etc.**

Similar features to VS Code extension.

### Standalone Desktop App
**Electron wrapper around web version**

Advantages:
- Better file system access
- Native feel
- Offline capable
- Packaged distribution

---

## Why Wait?

### Prerequisites:

**Must have first:**
1. ✅ Stable file format (.kleis)
2. ⏳ Type system (12-18 months)
3. ⏳ Evaluation engine (12-20 months)
4. ⏳ Language server protocol (6 months)
5. ⏳ Package system (6 months)

**Total:** 3-4 years of foundational work

**Then** build IDE integrations.

### Reasons to Wait:

1. **API stability** - Don't want to rewrite extension when core changes
2. **Feature completeness** - IDE needs type info, evaluation, etc.
3. **User base** - Need users to justify IDE investment
4. **Web-first is fine** - Modern tools are web-based (Overleaf, Google Colab)

---

## Quick Notes for Future

### VS Code Extension Architecture

```
kleis-vscode/
├── client/
│   ├── extension.ts        // VS Code extension entry
│   ├── notebook.ts         // Notebook renderer
│   └── webview.ts          // Structural editor embed
│
├── server/
│   ├── server.ts           // Language server (LSP)
│   ├── type-checker.ts     // Type info provider
│   ├── hover.ts            // Hover tooltips
│   └── completion.ts       // IntelliSense
│
└── syntaxes/
    └── kleis.tmLanguage.json  // Syntax highlighting
```

### Language Server Protocol (LSP)

Implement LSP for Kleis:
- `textDocument/hover` → Show type info
- `textDocument/completion` → Autocomplete symbols
- `textDocument/definition` → Jump to definition
- `textDocument/diagnostic` → Type errors

This enables Kleis support in **any** LSP-compatible editor!

### Electron Desktop App

Simple wrapper:
```javascript
// main.js
const { app, BrowserWindow } = require('electron');

app.whenReady().then(() => {
    const win = new BrowserWindow({
        width: 1400,
        height: 900,
        webPreferences: {
            nodeIntegration: true
        }
    });
    
    // Load Kleis web app
    win.loadURL('http://localhost:3000');
    // Or: win.loadFile('dist/index.html');
});
```

---

## Timeline Estimate

**Assuming core is ready (2027-2028):**

### VS Code Extension: 6-9 months
- Language server: 3-4 months
- Extension UI: 2-3 months
- Testing/polish: 1-2 months

### Desktop App: 2-3 months
- Electron wrapper: 1 month
- Native features: 1 month
- Packaging/distribution: 1 month

### Total: 8-12 months

**But only AFTER the 3-5 years of core development!**

---

## Decision

**Status:** Documented for future reference

**Priority:** Low (distant future)

**Rationale:**
- Web-first is the right strategy
- IDE integration requires stable core
- 3-5 years of foundational work comes first
- Good to have vision documented

**Next step:** Focus on core system, revisit this in 2027-2028

---

**For now:** The web version with v2.2 inline editing is excellent! Focus on making that perfect before thinking about IDE integration.

---

**Date documented:** December 3, 2025  
**Revisit:** Q4 2027 or when core is feature-complete

