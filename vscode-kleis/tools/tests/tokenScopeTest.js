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

    const grammarPath = path.resolve(__dirname, '../../syntaxes/kleis.tmLanguage.json');

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

    const filePath = path.resolve(__dirname, '../../example.kleis');
    const file = fs.readFileSync(filePath, 'utf8');
    const lines = file.split(/\r\n|\n/);

    const testCases = [
      { text: 'e', expected: 'constant.language.symbolic.kleis' },
      { text: 'π', expected: 'constant.language.symbolic.kleis' },
      { text: '□', expected: 'constant.language.placeholder.kleis' },
      { text: '⊕', expected: 'keyword.operator.unicode.kleis' },
      { text: 'α', expected: 'variable.type.greek.kleis' },
      { text: 'Matrix', expected: 'entity.name.type.constructor.kleis' },
      { text: 'structure', expected: 'keyword.control.kleis' }
    ];

    const results = new Map();
    testCases.forEach(tc => results.set(tc.text, { found: false, ok: false, scopes: [] }));

    let ruleStack = vsctm.INITIAL;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const res = grammar.tokenizeLine(line, ruleStack);
      res.tokens.forEach(t => {
        const text = line.slice(t.startIndex, t.endIndex);
        if (results.has(text)) {
          const rec = results.get(text);
          rec.found = true;
          rec.scopes = t.scopes;
          const expected = testCases.find(tc => tc.text === text).expected;
          if (t.scopes.includes(expected)) rec.ok = true;
        }
      });
      ruleStack = res.ruleStack;
    }

    let failed = false;
    for (const tc of testCases) {
      const rec = results.get(tc.text);
      if (!rec || !rec.found) {
        console.error(`FAIL: token '${tc.text}' not found in example.kleis`);
        failed = true;
        continue;
      }
      if (!rec.ok) {
        console.error(`FAIL: token '${tc.text}' scopes=${JSON.stringify(rec.scopes)} expected to include '${tc.expected}'`);
        failed = true;
      } else {
        console.log(`OK: '${tc.text}' -> contains '${tc.expected}'`);
      }
    }

    if (failed) process.exit(1);
    console.log('All token scope assertions passed.');
    process.exit(0);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
