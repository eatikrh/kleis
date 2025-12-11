# Inspect Scopes Tool

This small README explains how to run the scope inspector that tokenizes `example.kleis` using the TextMate grammar.

Prerequisites:
- Node.js and npm installed (tested with Node 16+)

Install dependencies and run the inspector:

```bash
cd /Users/eatik_1/Documents/git/cee/kleis/vscode-kleis
npm install
npm run inspect-scopes
```

What it does:
- Loads `./syntaxes/kleis.tmLanguage.json` with `vscode-textmate` and `vscode-oniguruma`.
- Tokenizes `example.kleis` line-by-line and prints token ranges with their scopes.

How to use:
- Edit `example.kleis` to add lines or tokens you want to inspect.
- Re-run `npm run inspect-scopes` to see updated scopes.

Troubleshooting:
- If you see JSON parse errors, ensure `./syntaxes/kleis.tmLanguage.json` is valid JSON.
- If oniguruma fails to load, confirm `node_modules/vscode-oniguruma/release/onig.wasm` exists after `npm install`.
