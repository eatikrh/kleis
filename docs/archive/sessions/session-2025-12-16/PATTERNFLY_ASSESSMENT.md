# PatternFly Editor Assessment (Dec 16, 2025)

## Summary

The PatternFly editor migration has resulted in **partially modular code** - hooks are well-separated, but the core components (`App.tsx`, `SVGEditor.tsx`) have become monoliths that port imperative DOM manipulation patterns from `static/index.html` into React.

## Code Metrics

| File | Lines | Assessment |
|------|-------|------------|
| `App.tsx` | 783 | ❌ Monolith - should be ~200 max |
| `SVGEditor.tsx` | 480 | ❌ Monolith - DOM manipulation nightmare |
| `hooks/*.ts` | 494 total | ✅ Well-separated |
| `utils/astUtils.ts` | 182 | ✅ Clean utilities |

## The Core Problem

**20 comments saying "like static/index.html"** throughout the codebase indicate the approach was:

> "Port the DOM manipulation logic from static/index.html into React"

Rather than:

> "Redesign using React's declarative patterns"

### Examples of Anti-Patterns

**Imperative DOM manipulation in React:**
```tsx
// SVGEditor.tsx - fighting React's model
useEffect(() => {
  document.querySelectorAll('.arg-overlay').forEach(el => {
    el.classList.remove('active-marker');
    if (el instanceof SVGRectElement) {
      el.setAttribute('stroke-width', '2');
      // ... more direct DOM manipulation
    }
  });
}, [activeMarkerId]);
```

**Window globals for event handlers:**
```tsx
// App.tsx - bypassing React's event system
useEffect(() => {
  (window as any).handleSlotClick = (event, id, path, nodeId) => {
    // ... complex handler exposed globally
  };
}, [currentAST]);
```

**Race conditions from mixing paradigms:**
```tsx
// Refs to work around stale closures
const activeMarkerPathRef = useRef<number[] | null>(null);
const isInsertingRef = useRef<boolean>(false);

// Guard to prevent inline commit during palette insertion
if (isInsertingRef.current) {
  console.log('handleInlineCommit: blocked by isInsertingRef guard');
  return;
}
```

## What's Actually Modular

The **hooks** follow good React patterns:
- `useAST.ts` - AST state management
- `useKleisAPI.ts` - Backend communication
- `useUndoRedo.ts` - History management
- `useVerify.ts` - Z3 verification

The **utilities** are clean:
- `astUtils.ts` - Pure functions for AST manipulation

## What's Not Modular

**`App.tsx` (783 lines)** contains:
- All UI state (mode, zoom, markers, inline editing)
- Complex event handlers exposed to window
- Multiple refs to work around closure issues
- DOM manipulation in effects

**`SVGEditor.tsx` (480 lines)** contains:
- 200+ line `useMemo` for overlay generation
- Direct DOM queries (`document.querySelector`)
- foreignObject injection for inline editing
- Window globals (`appendToInlineEditor`)

## Root Cause

The original `static/index.html` evolved organically over time with tight coupling between:
- SVG rendering
- Placeholder coordinates
- Overlay positioning
- Click handling
- Inline editing
- AST updates

Trying to recreate this behavior in React while maintaining **exact parity** forced the migration to replicate the same coupling, just with React syntax on top.

## Options Going Forward

### Option 1: Refactor to Proper React

- Extract state to Zustand/Redux store
- Make overlays declarative React components
- Remove window globals, use proper event delegation
- **Effort:** High (essentially rewrite)
- **Risk:** May still have parity issues

### Option 2: Accept Current State

- Document known issues
- Fix critical bugs only
- Focus on other priorities
- **Effort:** Low
- **Risk:** Technical debt accumulates

### Option 3: Embed Original Editor

- Keep `static/index.html` as the equation editor
- Use PatternFly for notebook shell (file browser, cell management)
- Communicate via postMessage or shared state
- **Effort:** Medium
- **Risk:** Two codebases to maintain

### Option 4: Different Framework

- Consider lighter alternatives (Solid.js, Svelte)
- Or Web Components for framework-agnostic isolation
- **Effort:** High (another migration)
- **Risk:** Same problems if approach doesn't change

## Recommendation

**Short-term:** Option 2 - Stabilize current PatternFly editor, document limitations.

**Long-term:** Option 3 - For Kleis Notebook, build the notebook shell in PatternFly/React but embed the battle-tested `static/index.html` for the actual equation editing. This avoids reimplementing the complex SVG overlay logic while still getting React's benefits for the notebook structure.

## Lessons Learned

1. **Porting imperative code to React doesn't make it declarative** - the paradigm must change, not just the syntax.

2. **Exact behavioral parity is expensive** - accepting "good enough" parity early would have saved pain.

3. **DOM manipulation in React = fighting the framework** - every `querySelector`, `classList.add`, manual event handler is a point of friction.

4. **The original was a prototype** - `static/index.html` was never designed for extraction/reuse; it's a working demo, not a component library.

---
*Documented: Dec 16, 2025*

