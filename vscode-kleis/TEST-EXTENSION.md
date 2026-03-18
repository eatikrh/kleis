# Test Your Kleis VS Code Extension

## ✅ Extension is Ready!

All files created. Your VS Code extension includes:
- ✅ `package.json` - Extension manifest
- ✅ `syntaxes/kleis.tmLanguage.json` - Your comprehensive grammar
- ✅ `language-configuration.json` - Bracket matching, comments
- ✅ `README.md` - Extension documentation
- ✅ `CHANGELOG.md` - Version history
- ✅ `example.kleis` - Test file with sample code
- ✅ `icon.svg` - Key logo for branding

## 🚀 Test It Now!

### Method 1: Press F5 (Immediate Test)
1. Open the `vscode-kleis` folder in VS Code
2. Press **F5** (Start Debugging)
3. A new VS Code window opens with the extension loaded
4. In that window, open `example.kleis`
5. **See syntax highlighting!** 🎨

### Method 2: Symlink Install (Permanent)
```bash
cd /Users/eatik_1/Documents/git/cee/kleis/vscode-kleis
ln -s "$(pwd)" "$HOME/.vscode/extensions/kleis-0.1.0"
```

Restart VS Code. All `.kleis` files now have highlighting!

## 📦 Package for Distribution

```bash
# Install packaging tool (one-time)
npm install -g @vscode/vsce

# Package the extension
cd /Users/eatik_1/Documents/git/cee/kleis/vscode-kleis
vsce package

# Creates: kleis-0.1.0.vsix
```

Share the `.vsix` file with users, or publish to VS Code Marketplace!

## 🌐 Publish to Marketplace (Make it Public!)

1. Go to: https://marketplace.visualstudio.com/manage
2. Create publisher account (use "eatikrh" as publisher name)
3. Get Personal Access Token from Azure DevOps
4. Run:
```bash
vsce login eatikrh
vsce publish
```

**Your extension will be installable by anyone in VS Code!**

---

**Try it now: Open vscode-kleis folder in VS Code and press F5!** 🚀

