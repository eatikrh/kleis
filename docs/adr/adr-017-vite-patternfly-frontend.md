# ADR-017: Vite + PatternFly for Frontend Architecture

## Status

**Proposed** - Future Implementation

## Context

### Current State

The Kleis equation editor and structural mode are currently implemented as:
- Single `static/index.html` file (~3300 lines)
- Vanilla JavaScript with no build system
- Manual DOM manipulation
- Ad-hoc CSS styling
- MathJax for mathematical rendering

**Strengths:**
- Simple deployment (single file)
- No build step required
- Works immediately in browser
- Fast prototyping

**Pain Points:**
- Growing file size (hard to maintain)
- No component modularity
- No TypeScript type safety
- Difficult to test
- CSS organization challenging
- Limited reusability across notebook environment

### Future Vision

The Kleis project aims to become:
1. **Equation Editor** - Rich mathematical notation editor
2. **Notebook Environment** - Computational notebook like Jupyter/Mathematica
3. **Document Authoring** - Paper/book creation with live mathematics
4. **Visual Reasoning** - Structural editor for symbolic computation

**These all need:**
- Shared UI components (buttons, modals, toolbars)
- Consistent design language
- Professional, accessible interface
- Maintainable codebase

---

## Decision

Adopt **Vite + PatternFly** as the frontend architecture for future development:

### Build Tool: Vite

**Why Vite:**
- ⚡ Instant server start (no bundling in dev)
- 🔥 Lightning-fast HMR (sub-100ms updates)
- 📦 Simple configuration (vs webpack complexity)
- 🎯 First-class TypeScript support
- 🌲 Automatic code splitting and tree-shaking
- 🚀 Optimized production builds
- 📚 Excellent documentation and ecosystem

**Alternatives Considered:**
- **Webpack** - Too complex, slower build times
- **Parcel** - Less ecosystem support
- **No build tool** - Current approach doesn't scale

### UI Framework: PatternFly

**Why PatternFly:**
- 🎨 **Enterprise-grade components** - Professional, polished
- ♿ **Accessibility first** - WCAG compliant, keyboard nav, screen readers
- 📐 **Design system** - Consistent patterns, spacing, colors
- 🧩 **Component library** - Buttons, modals, tables, forms, toolbars
- 📱 **Responsive** - Works on desktop, tablet, mobile
- 💼 **Red Hat backed** - Stable, well-maintained, long-term support
- 🎯 **Framework agnostic** - React, Vue, or vanilla JS
- 📚 **Excellent docs** - Comprehensive examples and guidelines

**Alternatives Considered:**
- **Material-UI** - Good, but more opinionated (Google style)
- **Ant Design** - Strong, but less accessible
- **Bootstrap** - Dated, less suitable for complex applications
- **Custom CSS** - Too much work, accessibility challenges

---

## Architecture

### Project Structure (Proposed)

```
kleis/
├── frontend/                    # New Vite project
│   ├── src/
│   │   ├── main.ts             # Entry point
│   │   ├── App.tsx             # Root component
│   │   ├── components/
│   │   │   ├── EquationEditor/
│   │   │   │   ├── EquationEditor.tsx
│   │   │   │   ├── Palette.tsx
│   │   │   │   ├── MatrixBuilder.tsx     # Dedicated component
│   │   │   │   ├── StructuralMode.tsx
│   │   │   │   └── TextMode.tsx
│   │   │   ├── Notebook/
│   │   │   │   ├── NotebookCell.tsx
│   │   │   │   ├── CodeCell.tsx
│   │   │   │   └── MarkdownCell.tsx
│   │   │   └── Common/
│   │   │       ├── EditMarker.tsx
│   │   │       └── MathRenderer.tsx
│   │   ├── services/
│   │   │   ├── api.ts              # Backend API calls
│   │   │   └── parser.ts           # LaTeX parsing
│   │   └── styles/
│   │       └── custom.css          # Custom overrides
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── index.html                  # Vite entry point
├── static/                         # Legacy (keep for now)
│   └── index.html                  # Current equation editor
└── src/                            # Rust backend (unchanged)
```

### Technology Stack

**Development:**
- **TypeScript** - Type safety, better IDE support
- **Vite** - Build tool and dev server
- **PatternFly React** - UI component library
- **React** - Component framework (if using PF React variant)

**OR (Alternative):**
- **PatternFly Core** - Vanilla JS/CSS (no React)
- **Lit** or **Vanilla TS** - For custom components

**Rendering:**
- **MathJax** - Continue using for LaTeX rendering
- **SVG** - For structural mode (Typst backend)

---

## Migration Strategy

### Phase 1: Parallel Development (Low Risk)

**Keep existing `static/index.html` working:**
- Don't touch current implementation
- Users can continue using it
- No breaking changes

**Build new version in `frontend/`:**
- Start with equation editor
- Port features incrementally
- Test thoroughly before switching

**Timeframe:** 2-3 weeks

### Phase 2: Feature Parity

**Port existing features:**
- ✅ Text mode (LaTeX input)
- ✅ Structural mode (interactive editing)
- ✅ Palette with all templates
- ✅ Matrix builder
- ✅ Edit markers
- ✅ Undo/redo

**Add improvements:**
- Better state management
- Component testing
- Keyboard shortcuts
- Accessibility

**Timeframe:** 1-2 weeks

### Phase 3: Enhancement

**Beyond parity:**
- Multiple equations (notebook cells)
- Code execution cells
- Markdown cells
- Document export
- Collaboration features

**Timeframe:** Ongoing

### Phase 4: Deprecation

**When new version is stable:**
- Redirect `/` to new Vite app
- Keep `static/index.html` as `/legacy`
- Eventual removal after migration period

---

## Consequences

### Positive

1. **Maintainability**
   - Modular components vs monolithic file
   - TypeScript catches errors at compile time
   - Easier to onboard new contributors

2. **Developer Experience**
   - Hot module replacement (instant feedback)
   - Better debugging tools
   - IDE autocomplete and type checking

3. **Professional UI**
   - Consistent design with PatternFly
   - Accessibility built-in
   - Responsive across devices

4. **Scalability**
   - Easy to add notebook features
   - Component reuse across editor/notebook
   - Better code organization

5. **Testing**
   - Unit test components
   - Integration testing easier
   - Visual regression testing possible

### Negative

1. **Build Complexity**
   - Requires Node.js and npm/yarn
   - Build step adds deployment complexity
   - More dependencies to manage

2. **Migration Effort**
   - 2-3 weeks to port existing features
   - Need to maintain both versions temporarily
   - Learning curve for PatternFly

3. **Bundle Size**
   - React + PatternFly = larger initial load
   - ~200KB vs current ~100KB
   - Mitigated by code splitting

4. **Deployment**
   - Need to build before deploying
   - Can't just copy HTML file
   - CI/CD becomes necessary

### Neutral

1. **PatternFly Commitment**
   - Locked into their design system
   - Updates require following their releases
   - But: Stable, enterprise-backed

2. **React vs Vanilla**
   - Could use PatternFly Core (vanilla)
   - Or PatternFly React (better DX)
   - Decision needed

---

## Implementation Plan

### Prerequisites

1. **Keep current editor working**
   - Tag current version: `v0.2.0-matrix-builder`
   - Continue bug fixes as needed
   - No breaking changes

2. **Set up Vite project**
   - Create `frontend/` directory
   - Initialize with `npm create vite@latest`
   - Add PatternFly dependencies

3. **Proof of Concept**
   - Port matrix builder as first component
   - Verify integration with Rust backend
   - Validate build/deploy process

### Week 1: Setup & POC

- [ ] Initialize Vite project
- [ ] Add PatternFly (React or Core)
- [ ] Set up TypeScript
- [ ] Create basic layout
- [ ] Port matrix builder component
- [ ] Test backend API integration

### Week 2-3: Feature Parity

- [ ] Port palette system
- [ ] Port text mode editor
- [ ] Port structural mode
- [ ] Port edit marker system
- [ ] Port undo/redo
- [ ] Achieve feature parity

### Week 4: Testing & Polish

- [ ] Comprehensive testing
- [ ] Accessibility audit
- [ ] Performance optimization
- [ ] Documentation
- [ ] Migration guide

---

## Alternatives Considered

### Alternative 1: Stay with Vanilla JS

**Pros:** No build step, simple deployment  
**Cons:** Doesn't scale to notebook environment

**Verdict:** Doesn't meet long-term vision

### Alternative 2: Vue or Svelte

**Pros:** Lighter than React, good DX  
**Cons:** PatternFly best supported with React

**Verdict:** React + PatternFly is more mature

### Alternative 3: Custom Build System

**Pros:** Full control  
**Cons:** Reinventing wheel, maintenance burden

**Verdict:** Vite is proven, maintained, excellent

---

## Decision Criteria

✅ **Supports notebook environment** - PatternFly has layout components  
✅ **Professional appearance** - Enterprise-grade UI  
✅ **Accessibility** - WCAG compliant out of box  
✅ **Maintainability** - Component-based architecture  
✅ **Developer experience** - Vite is fast and modern  
✅ **Long-term support** - Red Hat backing  
✅ **Migration path** - Can run in parallel  

---

## Open Questions

1. **PatternFly React vs PatternFly Core?**
   - React: Better DX, more components
   - Core: No framework lock-in, lighter

2. **Monorepo or separate repos?**
   - Keep frontend in same repo
   - Or split into `kleis` (backend) + `kleis-ui` (frontend)

3. **Server-side rendering?**
   - Probably not needed (equation editor is interactive)
   - Could add later for SEO/sharing

4. **State management?**
   - React Context API sufficient?
   - Or add Redux/Zustand for complex state?

---

## References

- **Vite:** https://vitejs.dev/
- **PatternFly:** https://www.patternfly.org/
- **PatternFly React:** https://www.patternfly.org/get-started/develop#react
- **PatternFly Core:** https://www.patternfly.org/get-started/develop#html-css

Related ADRs:
- **ADR-009:** WYSIWYG Structural Editor (established need for rich UI)
- **ADR-011:** Notebook Environment (defines scope beyond equation editor)
- **ADR-012:** Document Authoring (full application vision)

---

## Recommendation

**Accept this ADR** with the following approach:

1. **Tag current milestone:** `v0.2.0-matrix-builder`
2. **Start Vite+PatternFly POC** (1 week)
3. **Parallel development** (keep current editor working)
4. **Gradual migration** (port features incrementally)
5. **Switch when ready** (after thorough testing)

**Timeline:** 4-6 weeks for complete migration

**This positions Kleis for:**
- 📚 Notebook environment
- 📝 Document authoring
- 🤝 Collaboration features
- 🌐 Web-based distribution

---

**Status:** Proposed  
**Next Step:** Review and accept ADR, then create POC  
**Impact:** High (changes entire frontend development approach)  
**Risk:** Medium (mitigated by parallel development)


