# Installing the Kleis VS Code Extension

## Prerequisites: Build the Language Server

For full IDE features (diagnostics, hover, go-to-definition), build the Kleis language server:

```bash
# From the kleis repo root
cargo build --release --bin kleis-lsp

# The server will be at: target/release/kleis-lsp
```

## Quick Install (Development)

### Option 1: Symlink (Fastest)
```bash
cd vscode-kleis

# Install dependencies and compile TypeScript
npm install
npm run compile

# Symlink to VS Code extensions
ln -s "$(pwd)" "$HOME/.vscode/extensions/kleis-0.3.0"
```

Then restart VS Code. Open any `.kleis` file and you'll see:
- Syntax highlighting ✨
- Real-time error diagnostics 🔴
- Hover for type information 💡
- Go to definition (F12 or Cmd+Click) 🔗
- Document outline (Cmd+Shift+O) 📋

### Option 2: Package and Install
```bash
cd vscode-kleis

# Install dependencies and compile
npm install
npm run compile

# Install vsce if you don't have it
npm install -g @vscode/vsce

# Package the extension
vsce package

# Install the .vsix file
code --install-extension kleis-0.3.0.vsix
```

## Testing

1. Open VS Code
2. Create a test file: `test.kleis`
3. Type some Kleis code:
```kleis
structure Matrix(m: Nat, n: Nat, T) {
    operation transpose : Matrix(m, n, T) → Matrix(n, m, T)
}
```
4. You should see syntax highlighting! 🎨

## Publishing to VS Code Marketplace

### Prerequisites
1. Create a Microsoft account
2. Create a publisher: https://marketplace.visualstudio.com/manage
3. Get a Personal Access Token (PAT) from Azure DevOps

### Publish
```bash
vsce login <publisher-name>
vsce publish
```

## Icon Note

The extension uses `icon.svg` currently. For VS Code Marketplace, you need a PNG icon (128x128px).

**Convert SVG to PNG:**
```bash
# Using ImageMagick
convert -background none -resize 128x128 icon.svg icon.png

# Or using any SVG editor (Inkscape, Figma, etc.)
```

Then update `package.json` to reference `icon.png` instead of `icon.svg`.

## Uninstall

```bash
code --uninstall-extension eatikrh.kleis
```

Or through VS Code: Extensions → Kleis → Uninstall

