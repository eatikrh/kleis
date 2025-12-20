# ADR-024: Kleis Notebook with Monaco Editor

## Status

Proposed

## Context

Kleis needs a user-friendly editing environment that:

1. **Supports Unicode** - Mathematical symbols (∀, ∃, ℝ, →, etc.)
2. **Provides grammar-aware suggestions** - Context-sensitive completions
3. **Works on web and desktop** - Same experience everywhere
4. **Feels like a real editor** - Not block-based like Scratch

### The UX Spectrum

| Approach | Pros | Cons |
|----------|------|------|
| **Block-based (Scratch)** | No syntax errors possible | Feels like toys, slow for experts |
| **Pure text (Vim)** | Maximum freedom | Syntax errors, steep learning curve |
| **Text + smart help** | Best of both worlds | Requires more engineering |

We want the middle ground: text editing with intelligent assistance.

### Alternatives Considered

1. **Purely block-based editor** (like Scratch/Blockly)
   - Rejected: Too constraining for mathematicians
   
2. **Plain text editor** (no assistance)
   - Rejected: Unicode input is painful, no error feedback
   
3. **CodeMirror 6**
   - Viable but less ecosystem than Monaco
   
4. **Monaco Editor** ← Chosen
   - Same editor as VSCode
   - Works in browser AND desktop
   - Rich completion/hover/diagnostics API
   - Already used by Jupyter, GitHub, CodeSandbox

## Decision

Use **Monaco Editor** as the foundation for Kleis Notebooks:

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  BROWSER (notebooks.kleis.io)                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Monaco Editor                        │  Live Preview               │   │
│  │ ─────────────────────────────────────│───────────────────────────  │   │
│  │ define square(x : ℝ) : ℝ =           │   square: ℝ → ℝ             │   │
│  │     x * x                            │   square(x) = x²            │   │
│  │                                      │                             │   │
│  │ axiom comm : ∀(a b : ℝ). a+b = b+a   │   ∀a,b ∈ ℝ: a+b = b+a      │   │
│  │           ↑                          │                             │   │
│  │    ┌──────┴───────┐                  │                             │   │
│  │    │ Completions: │                  │                             │   │
│  │    │  ∀  forall   │                  │                             │   │
│  │    │  ∃  exists   │                  │                             │   │
│  │    │  →  arrow    │                  │                             │   │
│  │    └──────────────┘                  │                             │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │ WebSocket (for Z3)                          │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               ▼
                    ┌─────────────────────┐
                    │ Z3 Verification     │
                    │ Server              │
                    └─────────────────────┘
```

### Key Insight: AST Constructors = Templates

The Kleis AST in `kleis_in_kleis.kleis` defines:

```kleis
data Expression =
    EVariable(name: String)           // → [VAR] template
  | ENumber(value: ℝ)                // → [NUM] template
  | EOperation(name: String, args)    // → [OP] template
  | EIf(cond, then_expr, else_expr)   // → [IF] template
  | EForAll(vars, body)               // → [∀] template
  | ...
```

Each AST constructor is a "template" with typed slots:
- `EIf(cond: Expression, ...)` → cond slot shows Expression suggestions
- `EForAll(vars: List(VarDecl), ...)` → vars slot shows VarDecl suggestions

**The grammar itself dictates what completions to show!**

### Grammar-Aware Completion Provider

```javascript
monaco.languages.registerCompletionItemProvider('kleis', {
  provideCompletionItems: function(model, position) {
    // Kleis WASM parser determines context
    const context = kleisWasm.parsePartial(textUntilPosition);
    
    // Grammar determines suggestions
    if (context.expecting === 'Type') {
      return { suggestions: [
        { label: 'ℝ', insertText: 'ℝ' },
        { label: 'ℤ', insertText: 'ℤ' },
        { label: 'ℕ', insertText: 'ℕ' },
        { label: 'Bool', insertText: 'Bool' },
        { label: '→', insertText: '→' },
      ]};
    }
    if (context.expecting === 'Expression') {
      return { suggestions: [
        { label: '∀', insertText: '∀(${1:x} : ${2:Type}). $0' },
        { label: 'if', insertText: 'if ${1:cond} then ${2:then} else ${3:else}' },
        { label: 'let', insertText: 'let ${1:x} = ${2:value} in $0' },
        { label: 'λ', insertText: 'λ ${1:x} . $0' },
      ]};
    }
    // ... Pattern, Declaration, etc.
  }
});
```

### Monaco Features We Use

| Feature | Monaco API | Kleis Use |
|---------|------------|-----------|
| Completions | `registerCompletionItemProvider` | Grammar-aware suggestions |
| Snippets | `insertTextRules: SnippetString` | Templates with tab stops |
| Hover | `registerHoverProvider` | Show types on hover |
| Diagnostics | `editor.setModelMarkers` | Parse error squiggles |
| Trigger chars | `triggerCharacters` | `\` triggers Unicode palette |
| Themes | `defineTheme` | Math-friendly color scheme |

### Unicode Input Strategy

```javascript
// Trigger on backslash
triggerCharacters: ['\\']

// \forall → ∀, \exists → ∃, \R → ℝ, etc.
const unicodeMap = {
  '\\forall': '∀', '\\exists': '∃',
  '\\R': 'ℝ', '\\Z': 'ℤ', '\\N': 'ℕ', '\\Q': 'ℚ', '\\C': 'ℂ',
  '\\to': '→', '\\lambda': 'λ', '\\in': '∈',
  '\\and': '∧', '\\or': '∨', '\\not': '¬',
  '\\alpha': 'α', '\\beta': 'β', '\\gamma': 'γ', '\\Gamma': 'Γ',
  // ... etc
};
```

### Web vs Desktop

**Same code, two deployments:**

| Platform | Implementation |
|----------|----------------|
| **Web** | Monaco in browser + WebSocket to Z3 server |
| **Desktop (VSCode)** | vscode-kleis extension (already exists) |

Monaco IS the VSCode editor, so:
- Same syntax highlighting
- Same completions
- Same snippets
- Same keybindings

### Real-Time Parsing with WASM

For instant feedback without server round-trips:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  BROWSER                                                                    │
│  ┌─────────────────────────────────┐  ┌────────────────────────────────┐   │
│  │  kleis_parser.wasm              │  │  Monaco Editor                 │   │
│  │  ─────────────────              │  │                                │   │
│  │  • Parse on keystroke           │──│  • Error squiggles            │   │
│  │  • < 1ms response               │  │  • Context for completions    │   │
│  │  • No network latency           │  │  • Grammar-aware suggestions  │   │
│  └─────────────────────────────────┘  └────────────────────────────────┘   │
│                                                                             │
│                              │ Only for verification                        │
└──────────────────────────────┼──────────────────────────────────────────────┘
                               ▼ WebSocket
                    ┌─────────────────────┐
                    │ Z3 Server           │
                    │ (heavy lifting)     │
                    └─────────────────────┘
```

## Important: Two Separate ASTs

**The Kleis AST and Editor AST are intentionally separate:**

| Kleis AST | Editor AST |
|-----------|------------|
| Pure language semantics | Visual/layout metadata |
| What the math MEANS | How it LOOKS on page |
| Z3 verification target | Renderer input |
| Stable, minimal | Can evolve for UX |

```
Monaco (text) ──parse──▶ Kleis AST ──translate──▶ Editor AST ──render──▶ Typst/LaTeX
                              │
                              ▼
                         Z3 Verify

Equation Editor ──build──▶ Editor AST ──translate──▶ Kleis text ──▶ Monaco
```

This separation keeps Kleis pure and lets the Editor AST carry visual hints
(position, display style, font size) that Kleis doesn't need.

## Z3 Deployment Strategy

Z3 is a native binary (~50MB). Running it in a web context requires thought.

### Options Considered

| Approach | Latency | Offline? | Complexity | Power |
|----------|---------|----------|------------|-------|
| **Server Z3** | 50-200ms | ❌ | Low | Full |
| **Z3 WASM** | 0ms | ✅ | High | Limited (~15MB, slow startup) |
| **Hybrid** | Varies | Partial | Medium | Full |

### Decision: Tiered Verification

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ TIER 1: Instant (Client-side WASM)                                          │
│ ───────────────────────────────────                                         │
│ • Syntax validation                                                         │
│ • Type checking                                                             │
│ • Variable binding                                                          │
│ → Green checkmark appears immediately                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ TIER 2: Background (Server Z3)                                              │
│ ──────────────────────────────                                              │
│ • Consistency with other axioms                                             │
│ • Quick satisfiability checks                                               │
│ → "Verifying..." spinner, 100-500ms                                         │
│ → "✓ Verified" or "⚠ Contradiction"                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ TIER 3: On Demand (Server Z3, heavy)                                        │
│ ────────────────────────────────────                                        │
│ • Deep theorem proving                                                      │
│ • User clicks [Prove] button                                                │
│ → May take seconds, progress indicator                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Server Architecture

```
┌─────────────────────┐          ┌─────────────────────────────────────────┐
│  BROWSER            │          │  Z3 SERVER                              │
│  ────────           │          │  ─────────                              │
│  • Monaco Editor    │ WebSocket│  ┌─────────────────────────────────┐   │
│  • Equation Editor  │─────────▶│  │ Job Queue                       │   │
│  • Kleis WASM       │          │  │ • Priority lanes (quick/heavy)  │   │
│  • Tier 1 checks    │◀─────────│  │ • Fair scheduling per user      │   │
│                     │  results │  └──────────────┬──────────────────┘   │
└─────────────────────┘          │                 ▼                       │
                                 │  ┌─────────────────────────────────┐   │
                                 │  │ Z3 Worker Pool                  │   │
                                 │  │ • Timeout enforcement (30s max) │   │
                                 │  │ • Memory limits                 │   │
                                 │  │ • Result caching                │   │
                                 │  └─────────────────────────────────┘   │
                                 └─────────────────────────────────────────┘
```

### SaaS Concerns

| Concern | Solution |
|---------|----------|
| ∀∀∀∀ runs forever | Timeout enforcement (30s default) |
| One user starves others | Fair queuing, per-user limits |
| Repeated same query | Result caching (hash → result) |
| Memory exhaustion | Per-query memory limits |
| Connection overhead | Z3 context pooling |

## Consequences

### Positive

- **Unified experience** - Same editor on web and desktop
- **No syntax errors** - Grammar-aware suggestions guide users
- **Unicode friendly** - Backslash shortcuts + palette
- **Real-time feedback** - WASM parser for instant error detection
- **Extensible** - Monaco's rich plugin API
- **Familiar** - Millions of developers know VSCode

### Negative

- **Monaco bundle size** - ~2MB (acceptable for notebook app)
- **WASM compilation** - Need to compile Kleis parser to WASM
- **Completion provider complexity** - Must maintain grammar ↔ suggestions mapping

### Implementation Phases

1. **Phase 1: Basic Notebook**
   - Monaco with Kleis syntax highlighting
   - Unicode shortcuts (\forall → ∀)
   - Cell-based interface

2. **Phase 2: Smart Completions**
   - Kleis WASM parser
   - Context-aware suggestions
   - Real-time error markers

3. **Phase 3: Z3 Integration**
   - WebSocket to Z3 server
   - Verification results in output cells
   - Proof status indicators

4. **Phase 4: Collaboration**
   - Shareable notebook URLs
   - Real-time collaboration (CRDT)
   - Notebook versioning

## Related

- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
- `kleis_in_kleis.kleis` - AST types that define "templates"
- `vscode-kleis/` - Existing VSCode extension
- ADR-023: Kleis ABI (for potential WASM compilation target)

