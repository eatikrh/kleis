#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const vsctm = require('vscode-textmate');
const oniguruma = require('vscode-oniguruma');

(async () => {
  try {
    const wasmPath = require.resolve('vscode-oniguruma/release/onig.wasm');
    const wasmBin = fs.readFileSync(wasmPath).buffer;
    await oniguruma.loadWASM(wasmBin);

    const onigLib = {
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (s) => new oniguruma.OnigString(s)
    };

    const grammarPath = path.resolve(__dirname, '../syntaxes/kleis.tmLanguage.json');

    const registry = new vsctm.Registry({
      onigLib: Promise.resolve(onigLib),
      loadGrammar: async (scopeName) => {
        if (scopeName === 'source.kleis') {
          const content = fs.readFileSync(grammarPath, 'utf8');
          return vsctm.parseRawGrammar(content, grammarPath);
        }
        return null;
      }
    });

    const grammar = await registry.loadGrammar('source.kleis');
    if (!grammar) {
      console.error('Failed to load grammar for source.kleis');
      process.exit(2);
    }

    const filePath = path.resolve(__dirname, '../example.kleis');
    const file = fs.readFileSync(filePath, 'utf8');
    const lines = file.split(/\r\n|\n/);

    let ruleStack = vsctm.INITIAL;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const result = grammar.tokenizeLine(line, ruleStack);
      console.log(`${i + 1}: ${line}`);
      result.tokens.forEach((t) => {
        const text = line.slice(t.startIndex, t.endIndex);
        console.log(`  [${t.startIndex}-${t.endIndex}]: '${text}' => ${t.scopes.join(' | ')}`);
      });
      ruleStack = result.ruleStack;
    }
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
