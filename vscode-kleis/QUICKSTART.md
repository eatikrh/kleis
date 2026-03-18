# Quick Start - Test Kleis Extension

## Instant Test (No Installation Required)

1. **Open this folder in VS Code:**
```bash
cd /Users/eatik_1/Documents/git/cee/kleis/vscode-kleis
code .
```

2. **Press F5** (or Run → Start Debugging)
   - This launches a new VS Code window with the extension loaded

3. **In the new window, open `example.kleis`**
   - You should see beautiful syntax highlighting! 🎨

## What You Should See

✅ **Keywords in purple:** structure, implements, operation, axiom, data, match  
✅ **Types in blue:** Matrix, Vector, Scalar, ℝ, ℂ, ℤ, ℕ  
✅ **Operators highlighted:** ∀, ∃, λ, →, ⇒, ×, ·, ∇, ∂, ∫  
✅ **Greek letters:** α, β, γ, etc.  
✅ **Comments in gray:** // and /* */  
✅ **Constructors:** None, Some, Ok, Err  

## Permanent Installation

### Option 1: Symlink (Easiest for Development)
```bash
cd vscode-kleis
ln -s "$(pwd)" "$HOME/.vscode/extensions/kleis-0.1.0"
```

Restart VS Code. All .kleis files will now have syntax highlighting!

### Option 2: Package as VSIX
```bash
# Install packaging tool
npm install -g @vscode/vsce

# Package the extension
vsce package

# Install
code --install-extension kleis-0.1.0.vsix
```

## Testing with Your Actual Kleis Files

Once installed, open any file from:
- `stdlib/*.kleis`
- `kleis/*.kleis`

They should all have beautiful syntax highlighting!

## Publishing to VS Code Marketplace

Once you're happy with the extension:

1. Create publisher account: https://marketplace.visualstudio.com/manage
2. Get PAT from Azure DevOps
3. Run: `vsce publish`

Your extension will be available to all VS Code users worldwide! 🌍

---

**Ready to test? Press F5 in VS Code!** 🚀

