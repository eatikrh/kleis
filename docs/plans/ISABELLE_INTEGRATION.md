# Isabelle/HOL Backend Integration

**Branch:** `feature/isabelle-solver-backend`  
**Status:** Planning  
**Created:** January 7, 2026

## Overview

Integrate Isabelle/HOL as an alternative verification backend for Kleis, enabling deep proofs that Z3 cannot handle (induction, termination, higher-order reasoning).

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Kleis                                 │
│  - User-friendly syntax                                      │
│  - Quick Z3 for "easy" proofs (default)                     │
│  - Structures, axioms, documents                             │
└─────────────────────┬───────────────────────────────────────┘
                      │ When Z3 times out or user requests
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    Isabelle/HOL                              │
│  - Deep induction proofs                                     │
│  - Termination proofs                                        │
│  - AFP library access (Neural Networks, Petri Nets, etc.)   │
└─────────────────────────────────────────────────────────────┘
```

## Isabelle Server API

**Documentation:** [The Isabelle System Manual, Chapter 4](https://isabelle.in.tum.de/dist/Isabelle2025-1/doc/system.pdf)

### Server Commands

| Command | Purpose |
|---------|---------|
| `session_build` | Build HOL session |
| `session_start` | Start verification session |
| `use_theories` | **Main entry point** — load and check theories |
| `purge_theories` | Unload theories |
| `session_stop` | End session |

### Protocol

1. Start server: `isabelle server` → outputs host:port and password
2. Connect via TCP socket
3. Authenticate with password
4. Send JSON commands
5. Parse sync/async responses

### Verified Working (Jan 7, 2026)

```bash
# Server started successfully
server "isabelle" = 127.0.0.1:58865 (password "...")

# Commands tested:
- echo ✅
- help ✅
- session_start {"session":"HOL"} ✅
```

## Translation: Kleis → Isar

### Structures → Locales

```kleis
// Kleis
structure Group(G) {
    operation e : G
    operation inv : G → G
    operation mul : G × G → G
    
    axiom left_identity: ∀(x : G). mul(e, x) = x
}
```

```isabelle
-- Isar
locale group =
  fixes e :: "'g"
  fixes inv :: "'g ⇒ 'g"
  fixes mul :: "'g ⇒ 'g ⇒ 'g"
  assumes left_identity: "mul e x = x"
```

### Data Types → Datatypes

```kleis
// Kleis
data Tree(T) = Leaf(T) | Node(Tree(T), Tree(T))
```

```isabelle
-- Isar
datatype 'a tree = Leaf 'a | Node "'a tree" "'a tree"
```

## Implementation Plan

### Phase 1: Infrastructure (~2 days)

- [ ] Create `src/solvers/isabelle_backend.rs`
- [ ] Implement TCP connection to Isabelle server
- [ ] Handle password authentication
- [ ] Parse JSON responses
- [ ] Server lifecycle management (spawn, detect, cleanup)

### Phase 2: Translation (~1 week)

- [ ] Kleis AST → Isar string generator
- [ ] Handle structures → locales
- [ ] Handle data types → datatypes
- [ ] Handle axioms → assumes/lemmas
- [ ] Handle functions → definitions

### Phase 3: Verification Loop (~2 days)

- [ ] Generate `.thy` file from Kleis
- [ ] Send via `use_theories` API
- [ ] Parse verification result
- [ ] Report errors with source locations
- [ ] Timeout handling

### Phase 4: User Experience (~1 day)

- [ ] Syntax: `verify axiom with isabelle`
- [ ] REPL command: `:isabelle <axiom>`
- [ ] Error messages from Isabelle → Kleis format
- [ ] Progress feedback for long proofs

### Phase 5: AFP Integration (~2 days)

- [ ] Support importing AFP sessions
- [ ] Example: `verify with isabelle using Neural_Networks`
- [ ] Document available AFP entries

## Key Resources

| Resource | URL |
|----------|-----|
| Isabelle Documentation | https://isabelle.in.tum.de/documentation.html |
| System Manual (Ch. 4: Server) | https://isabelle.in.tum.de/dist/Isabelle2025-1/doc/system.pdf |
| AFP Main Site | https://www.isa-afp.org/ |
| AFP Neural Networks | https://www.isa-afp.org/entries/Neural_Networks.html |

## Test Cases

### Petri Net Mutual Exclusion

The `examples/petri-nets/mutex_verified.kleis` has properties that Z3 cannot prove (induction over reachable states). Isabelle should handle these.

### Neural Network Properties

Using the AFP Neural Networks entry, verify:
- Activation function properties
- Lipschitz bounds
- BIBO stability

## Notes

- Isabelle is pre-installed at `/Applications/Isabelle2025-1.app/`
- Server tested and working (Jan 7, 2026)
- AFP Neural Networks downloaded to `~/Downloads/Neural_Networks/`

## Related

- [NEXT_SESSION.md: Isabelle Integration](../NEXT_SESSION.md) - Future enhancement documentation
- [ADR-016: Operations in Structures](../adr/adr-016-operations-in-structures.md) - Related architecture

