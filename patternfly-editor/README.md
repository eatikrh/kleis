# Kleis Equation Editor - PatternFly Edition

A React/PatternFly implementation of the Kleis Equation Editor.

## Overview

This is a modern reimplementation of the Equation Editor (`static/index.html`) using:

- **React** - Component-based architecture
- **TypeScript** - Type safety
- **PatternFly 5** - Professional UI design system
- **Vite** - Fast development builds

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

Then open http://localhost:5173/

## Verification

This implementation is verified against `static/index.html` (the reference implementation):

| Test | Reference Output | PatternFly Output | Status |
|------|------------------|-------------------|--------|
| Fraction button | `{Operation:{name:'scalar_divide',...}}` | Same | ✅ |
| Power button | `{Operation:{name:'power',...}}` | Same | ✅ |
| Integral button | `{Operation:{name:'integral',...}}` | Same | ✅ |

## Architecture

```
src/
├── types/
│   └── ast.ts              # EditorNode type definitions
├── components/
│   ├── Palette/
│   │   ├── astTemplates.ts # AST templates (source of truth)
│   │   ├── buttonConfigs.ts # Tab/button definitions
│   │   ├── PaletteButton.tsx
│   │   └── PaletteTabs.tsx
│   ├── Editor/             # (future) Visual editor
│   └── Preview/
│       └── ASTPreview.tsx  # AST debugging view
├── hooks/                  # (future) useAST, useUndoRedo
├── api/                    # (future) Backend API calls
└── App.tsx                 # Main application
```

## Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| M1 | Scaffold + PatternFly | ✅ |
| M2 | One button (fraction) | ✅ |
| M3 | Palette tabs | ✅ |
| M4 | All buttons (100+ templates) | ✅ |
| M5 | SVG rendering (Typst backend) | ✅ |
| M6 | Click overlays | ✅ |
| M7 | Inline editor | ✅ |
| M8 | Type checking | ✅ |
| M9 | Undo/redo | ✅ |
| M10 | Feature parity | 🔄 (testing) |

### M10 Remaining Items

- [ ] Comparison test suite (automated verification against `static/index.html`)
- [ ] Export to LaTeX/Typst/Kleis (Export buttons)
- [ ] Text mode LaTeX input (partial - UI exists, parsing not wired)
- [ ] Edge case testing (complex nested expressions)

## Benefits Over Reference Implementation

1. **Component Testing** - Safety net for visual bugs
2. **Flexible Tabs** - Move buttons = move one line
3. **Clean State** - React hooks, no global variables
4. **Type Safety** - TypeScript catches errors early
5. **Design System** - PatternFly provides consistent UX

## Reference

- Reference implementation: `../static/index.html`
- Architecture docs: `../docs/NEXT_SESSION.md`
