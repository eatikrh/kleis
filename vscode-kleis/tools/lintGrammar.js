#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

const grammarPath = path.resolve(__dirname, '../syntaxes/kleis.tmLanguage.json');

try {
  const raw = fs.readFileSync(grammarPath, 'utf8');
  JSON.parse(raw);
  console.log('OK: Grammar JSON is valid:', grammarPath);
  process.exit(0);
} catch (err) {
  console.error('ERROR: Invalid JSON in grammar file:', grammarPath);
  console.error(err.message || err);
  process.exit(1);
}
