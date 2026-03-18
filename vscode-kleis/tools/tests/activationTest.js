#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

try {
  const pkg = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../../package.json'), 'utf8'));
  const activations = pkg.activationEvents || [];
  const required = [
    'onCommand:kleis.openRepl',
    'onCommand:kleis.runSelection',
    'onCommand:kleis.loadFileInRepl',
    'onCommand:kleis.showStatus'
  ];

  let ok = true;
  for (const r of required) {
    if (!activations.includes(r)) {
      console.error(`FAIL: missing activation event ${r}`);
      ok = false;
    }
  }

  const ext = fs.readFileSync(path.resolve(__dirname, '../../src/extension.ts'), 'utf8');
  const checks = [
    'Kleis REPL not found',
    'Kleis LSP not found',
    'Kleis status:'
  ];
  for (const s of checks) {
    if (!ext.includes(s)) {
      console.error(`FAIL: expected string not found in src/extension.ts: ${s}`);
      ok = false;
    }
  }

  if (!ok) process.exit(1);
  console.log('Activation tests passed.');
  process.exit(0);
} catch (err) {
  console.error(err);
  process.exit(1);
}
