# ADR-031: Theory MCP — Interactive Theory Building with Agent Co-authorship

**Status:** Proposed  
**Date:** 2026-02-20  
**Related:** ADR-030 (MCP Agent Reasoning Partner), ADR-022 (Z3 Integration)

---

## Context

ADR-030 established the **policy MCP**: an agent queries a fixed set of Kleis rules, evaluates
propositions against them, and reasons about their properties with Z3. The policy file is
read-only. The agent is a **questioner** — it can ask "is this true?" but cannot add new
definitions, structures, or axioms.

This creates an asymmetry. The agent can reason about existing theories but cannot *build*
theories. It can verify `∀(x : R). x + 0 = x` against a loaded Ring structure but cannot
propose a new structure and explore its consequences.

**Motivation:** A conversation about theoretical physics (quarks, ontological Hilbert spaces,
admissible universes, Verlinde's entropic gravity) showed what happens when an LLM speculates
about structures in natural language: it writes plausible-looking pseudo-Kleis, but nothing is
checked. The "formalism" is the *shape* of formalism without the substance. If the agent could
submit structures and axioms to a live Kleis evaluator, every claim would be checked by Z3.
The agent moves from *writing about* theories to *doing* theory.

**Key architectural fact:** The Kleis `Evaluator` is a Rust struct, not a process. `Evaluator::new()`
allocates empty hash maps. `load_program()` populates them from a parsed `.kleis` file. The MCP
server does not shell out to a `kleis` binary — it *is* Kleis. Creating, dropping, and recreating
evaluators is cheap. This makes the theory MCP feasible without new infrastructure.

---

## Decision

**Create a `kleis-theory` MCP server that lets the agent co-author Kleis theories interactively,
with Z3 checking consistency at every step.**

### Architecture: File as State

The evaluator's registries (functions, structures, axioms, data types) are append-only. There is
no `unregister`, `remove_axiom`, or `clear`. Once `load_program()` is called, definitions are
permanent for that evaluator's lifetime.

Rather than fighting this, the theory MCP makes the **file system** the source of truth and the
**evaluator** a disposable read-only view:

```
.theory-sessions/             # Ephemeral (gitignored)
  session.kleis               # Agent's accumulated theory (grows over time)
  scratch.kleis               # Temporary file for try-before-commit

theories/                     # Persistent (version-controlled)
  my_groups.kleis             # Agent-saved theories, importable by future sessions
  pot_exploration_01.kleis
```

Both paths are configurable in `config/kleis.toml` under `[theory]`:

```toml
[theory]
workspace_dir = ".theory-sessions"   # session/scratch files
save_dir = "theories"                # saved theories
```

If no config is provided, these defaults are used. Relative paths resolve from
the project root, ensuring `import` paths to `stdlib/` and `examples/` work correctly.

`session.kleis` uses `import` to compose its foundation:

```kleis
import "stdlib/prelude.kleis"
import "stdlib/algebra.kleis"

structure Hont {
    carrier : Type
    symmetry : GroupAction(carrier)
}

structure ModalFlow(H : Hont) {
    operation flow : H.carrier -> H.carrier
    axiom preserves_symmetry : preserves(flow, invariants(H.symmetry))
}
```

The evaluator is rebuilt from this file whenever the theory changes. "Restart" is
`drop` + `Evaluator::new()` + `parse` + `load_program()`. The `import` directive handles
recursive loading, circular-import protection, and path resolution — all existing machinery.

### Session Startup

On startup, the theory MCP auto-loads a minimal prelude (`stdlib/prelude.kleis`) so that
basic types and operations are available immediately. The agent can begin submitting
structures without an explicit `load_theory` call.

To start a different session — with domain-specific foundations or the agent's own saved
theories — the agent calls `load_theory` with the desired imports. This replaces the
current session entirely: fresh `session.kleis`, fresh evaluator, new foundation.

### Axioms Live in Structures

Kleis does not allow top-level axioms. Axioms are members of structures, alongside fields and
operations. The agent cannot scatter disconnected claims — it must organize them into coherent
structures. This is a feature: it forces the agent to think structurally about its theory.

The unit of theory-building is the **structure**, not the axiom.

### Try Before Commit

Since the evaluator cannot retract definitions, the MCP supports a "dry run" workflow:

1. Agent proposes a new structure (or `define`, or `data` type)
2. MCP writes `session.kleis` + candidate to a scratch file
3. MCP creates a fresh evaluator, loads the scratch file (which transitively loads
   all `import`ed files — stdlib, foundations, prior theories — via Kleis's recursive
   import resolution)
4. MCP checks consistency (Z3)
5. Reports result to agent: consistent or inconsistent (with Z3 explanation)
6. If agent commits → append to `session.kleis`, keep this evaluator as live
7. If agent discards → delete scratch file, keep previous evaluator

The agent is never surprised by an irreversible bad axiom. Every addition is tested before
it becomes permanent.

### Exploring Alternative Universes

The `import` mechanism enables branching exploration:

- **Universe A:** `import "stdlib/algebra.kleis"` → assert continuum hypothesis
- **Universe B:** restart, same imports → assert negation of continuum hypothesis
- **Universe C:** `import "foundations/constructive.kleis"` → entirely different foundation

Within a single evaluator lifetime, knowledge accumulates monotonically — no retraction.
When the agent wants a *different* line of reasoning, it starts a new session with different
imports. This is sound: within one line of reasoning you don't retract axioms; when you want
a different line, you start fresh.

The agent holds the cross-universe knowledge in its own context. The MCP just answers questions
given the current axioms. Clean separation: files provide persistence, the agent provides
continuity, the MCP provides reasoning.

---

## Proposed MCP Tools

The theory MCP is a **separate binary** with its own tool set. It does not modify or
extend the existing policy MCP (`src/mcp/`). Both MCPs share the same Kleis library
crate (`Evaluator`, `KleisParser`, `AxiomVerifier`, `PrettyPrinter`, `StructureRegistry`)
but are independent servers with independent tool definitions.

| Tool | Purpose | Mutates theory? |
|------|---------|-----------------|
| `evaluate` | Evaluate expression or verify proposition via Z3 | No |
| `describe_schema` | Show everything loaded: imports, stdlib, and agent's additions | No |
| `submit_structure` | Add a structure (with fields, operations, axioms) | Yes (after try) |
| `submit_define` | Add a top-level `define` function | Yes (after try) |
| `submit_data` | Add a `data` type definition | Yes (after try) |
| `try_structure` | Dry-run: check if a structure is consistent without committing | No |
| `list_session` | Show session history: what was added in what order | No |
| `load_theory` | Restart with specified imports (new universe) | Yes (replaces) |
| `save_theory` | Write current session to a named `.kleis` file | No (side effect: file) |

`evaluate` and `describe_schema` reimplement the same functionality as in the policy MCP
(same parser, same Z3 pipeline, same `PrettyPrinter` rendering) but in new code, not by
calling into the policy MCP's modules. Policy-specific tools (`check_action`, `list_rules`,
`explain_rule`) have no equivalent here — they are about enforcing rules, not exploring theories.

`describe_schema` returns **everything** the evaluator has loaded: prelude types, imported
stdlib structures, and the agent's own additions. This serves two purposes: the agent learns
Kleis syntax and available operations from the prelude/stdlib, and it sees the full context
it is building on — what types, operations, and axioms are available for use in new structures
and propositions.

### `submit_structure` — the primary tool

```
submit_structure({
    kleis: "structure Projection(H : Hont) {\n  operation pi : H.carrier -> R4\n  axiom surjective : ∀(x : R4). ∃(h : H.carrier). pi(h) = x\n}"
})
```

The agent sends raw Kleis source for a structure definition. The MCP:

1. Appends it to a scratch copy of `session.kleis`
2. Parses the combined file
3. Loads into a fresh evaluator
4. Verifies axioms via Z3
5. Returns: `{ status: "consistent", axioms_verified: ["surjective: VERIFIED"] }`
   or: `{ status: "inconsistent", error: "..." }`
6. On success, `session.kleis` is updated and the evaluator becomes live

Failure at any stage aborts without side effects — `session.kleis` is never modified,
the live evaluator is untouched.

### Consistency Checking Through Use

There is no dedicated `check_consistency` tool. Consistency is discovered through the
natural use of `evaluate` with propositions derived from the theory's own axioms:

1. **Verify an axiom's consequence.** After submitting a structure `ModalFlow` with
   axiom `preserves_symmetry`, evaluate a proposition that should follow from it.
   Z3 loads all background axioms and checks whether the implication holds.

2. **Test for degeneracy.** Evaluate `∀(x y : ModalFlow). x = y`. This should NOT
   follow from the axioms. If Z3 says VERIFIED, the axioms are too strong — they
   collapse the carrier to a single element, or the theory is contradictory.

3. **Test for contradiction.** If Z3 verifies a known falsehood under the loaded
   axioms, the theory is outright inconsistent (ex falso quodlibet).

This is how mathematicians check theories: not by asking "is false provable?" but by
exploring what the axioms entail and whether those entailments make sense. The agent
naturally discovers inconsistency by testing the structure's own axioms and their
implications as propositions.

### `try_structure` — speculation without commitment

Same as `submit_structure` but never updates `session.kleis`. The agent explores
"what if I added this?" without risk. The scratch evaluator is discarded after reporting.

### `load_theory` — universe switching

```
load_theory({
    imports: ["stdlib/prelude.kleis", "stdlib/algebra.kleis", "my_theories/groups.kleis"]
})
```

Creates a fresh `session.kleis` with the specified imports, drops the old evaluator,
builds a new one. Previous session state is gone (unless `save_theory` was called first).

### `save_theory` — persistence

Writes the current `session.kleis` to a named file under a theories directory. The agent's
discoveries become first-class Kleis artifacts: human-readable, version-controllable,
importable by future sessions or other agents.

---

## Error Handling: Three-Stage Pipeline

Agent-submitted Kleis code passes through three stages before it becomes part of the theory.
Failure at any stage is safe — the session file and live evaluator are never modified.

```
Agent submits Kleis source
    │
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  Stage 1: PARSE                                                  │
│  parse_kleis_program(&source) → Result<Program, KleisParseError> │
│                                                                  │
│  Catches: syntax errors, missing delimiters, malformed types     │
│  Returns: "Parse error at position 83: Expected ':' after        │
│            axiom name"                                           │
│  Cost: microseconds, no evaluator or Z3 involved                 │
└──────────────────────────────────────────────────────────────────┘
    │ OK
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  Stage 2: LOAD                                                   │
│  evaluator.load_program(&program) → Result<(), String>           │
│                                                                  │
│  Catches: duplicate definitions, unresolved imports, type errors  │
│  Returns: "Function 'flow' is already defined"                   │
│  Cost: AST iteration, no Z3 involved                             │
└──────────────────────────────────────────────────────────────────┘
    │ OK
    ▼
┌──────────────────────────────────────────────────────────────────┐
│  Stage 3: VERIFY                                                 │
│  Z3 checks axiom consistency                                     │
│                                                                  │
│  Catches: contradictory axioms, unsatisfiable constraints         │
│  Returns: "Axiom 'commutativity' is inconsistent with existing   │
│            axiom 'anti_commutativity'"                            │
│  Cost: Z3 solver time (milliseconds to seconds)                  │
└──────────────────────────────────────────────────────────────────┘
    │ OK
    ▼
  session.kleis updated, evaluator becomes live
```

All three stages operate on a **scratch copy**. The live `session.kleis` and the live
evaluator are only updated after all three stages succeed. This is the same pattern the
policy MCP already uses for `evaluate` — `KleisParseError` is caught and returned as a
structured error message with position information, giving the agent enough detail to
correct the syntax and resubmit.

The agent will make syntax errors — that is inevitable. But the feedback loop is fast
and specific: Kleis reports exactly where and what went wrong. The agent fixes it and
resubmits. No theory corruption is possible.

---

## Difference from Policy MCP

| Aspect | `kleis-policy` (ADR-030) | `kleis-theory` (this ADR) |
|--------|--------------------------|---------------------------|
| **Agent role** | Questioner | Co-author |
| **Theory** | Fixed (loaded once from policy file) | Evolving (agent adds structures) |
| **Mutation** | None — read-only | Append-only via file + reload |
| **Primary use** | "May I do X?" / "Is P always true?" | "What if we assume X? Is Y consistent?" |
| **Tools** | `check_action`, `evaluate`, `describe_schema` | `submit_structure`, `try_structure`, `evaluate`, `load_theory` |
| **Evaluator lifecycle** | Created once, lives forever | Rebuilt on every commit or universe switch |
| **File** | `agent_policy.kleis` (read-only) | `session.kleis` (append, rebuild) |

---

## Why Not Modify the Evaluator?

The evaluator has no `unregister` or `retract`. We could add one, but:

1. **Monotonicity is correct.** In formal reasoning, you don't retract axioms mid-proof.
   If you want different axioms, you start a different proof.
2. **The file-based approach already works.** `import` handles loading, the parser handles
   parsing, `load_program` handles registration. Rebuilding from files uses only existing
   machinery.
3. **Simplicity.** Adding retraction to the evaluator means tracking dependencies (which
   functions reference which axioms, which structures depend on which types). The
   file-and-rebuild approach avoids all of that.
4. **The evaluator is cheap to create.** `Evaluator::new()` is empty hash map allocation.
   `load_program()` is iteration over AST nodes. No process startup, no Z3 context creation.
   Rebuilding is fast.

---

## Implementation Plan

### Phase 1: Minimal Theory MCP

- New binary: `src/bin/theory_server.rs`
- Reuses: `Evaluator`, `AxiomVerifier`, `KleisParser`, `PrettyPrinter`, `StructureRegistry`
- New: `TheoryEngine` (analogous to `PolicyEngine`) managing `session.kleis` and evaluator lifecycle
- Tools: `submit_structure`, `evaluate`, `list_session`, `load_theory`
- Transport: same NDJSON/Content-Length dual-mode as policy MCP

### Phase 2: Try-Before-Commit

- `try_structure` / `try_define` tools
- Scratch file management
- Z3 consistency checking on the scratch evaluator

### Phase 3: Persistence and Multi-Universe

- `save_theory` / named sessions
- Session history (which structures were added in what order)
- Cross-session import (agent imports its own saved theories)

### Phase 4 (Exploratory): Numerical Experimentation

Kleis already has substantial numerical capabilities — LAPACK linear algebra (eigenvalues,
SVD, Cholesky, Schur, Riccati, LQR), an ODE solver (`ode45`), and plotting — all accessible
through the same `Evaluator` that handles symbolic reasoning. In principle, the agent could
move from formal verification to numerical experiment within a single session:

1. Build a theory (structures, axioms) — verified by Z3
2. Instantiate it with concrete values in an `example` block
3. Run numerical computations (eigenvalues of a Hamiltonian, ODE integration of dynamics)
4. Compare predictions against known data

For instance, an agent exploring projection kernels (see `examples/ontology/revised/`) might
define the kernel's axioms formally, then test whether a concrete kernel predicts hydrogen
energy levels or galactic rotation curves numerically.

**However, this phase carries significant uncertainty.** The MCP context is text-based:
the agent receives JSON responses, not graphical output. Kleis's plotting and Typst rendering
produce visual artifacts (SVG, PDF) that an agent cannot directly inspect. Numerical results
as raw numbers (eigenvalue lists, ODE solution vectors) are feasible to return via MCP, but
interpreting large numerical datasets in a text-only channel may prove impractical for
complex experiments. Graphical comparison of predicted vs. observed curves — the natural
endpoint of a numerical test — would require capabilities beyond what the current MCP
transport supports.

This phase is noted here as a natural extension of the architecture, not as a commitment.
The formal reasoning pipeline (Phases 1–3) is the deliverable. Numerical experimentation
may follow if the text-based interaction model proves sufficient for the intended use cases,
or if the MCP transport evolves to support richer output formats.

---

## Consequences

### Positive

1. **Agent becomes a theory builder** — not limited to querying fixed rules
2. **Every claim is Z3-checked** — no more unverified pseudo-formalism
3. **Zero changes to the evaluator** — file-based rebuild uses existing machinery
4. **Structures enforce discipline** — agent must organize axioms coherently
5. **Import enables composition** — agent builds on stdlib and its own prior work
6. **Saved theories are portable** — human-readable `.kleis` files, not opaque state

### Negative

1. **Rebuild cost on every commit** — evaluator is recreated from files
2. **No incremental addition** — even adding one structure reloads everything
3. **Agent must write valid Kleis** — syntax errors are possible

### Mitigations

- Rebuild cost is low (struct allocation + AST iteration, no process startup)
- Parse errors return clear messages; agent can correct and retry
- `try_structure` lets the agent test before committing

---

## Files (Expected)

- `src/mcp/theory.rs` — `TheoryEngine`: session file management, evaluator lifecycle
- `src/mcp/theory_server.rs` or `src/bin/theory_server.rs` — JSON-RPC handlers
- `src/mcp/theory_protocol.rs` — tool definitions for theory MCP
- `tests/mcp_theory_test.rs` — integration tests

---

## Related ADRs

**ADR-030 (MCP Agent Reasoning Partner):** The policy MCP established the pattern of
wrapping an `Evaluator` in an MCP server with JSON-RPC tools. The theory MCP reuses
the same pattern with a different lifecycle model.

**ADR-022 (Z3 Integration):** Z3 verification is the core capability that makes
"try before commit" possible — the MCP can check whether a proposed axiom is consistent
before making it permanent.

---

## Future Directions: Companion MCPs

The theory MCP is one piece of a potential multi-MCP architecture where specialized servers
collaborate through the agent:

### Jupyter Display MCP

Kleis already has a Jupyter kernel that renders Typst output, displays SVG plots, and
shows rich cell output. A Jupyter MCP would let the agent use a running notebook as its
"whiteboard" — sending computed results from the theory MCP to Jupyter for visual rendering.
The agent orchestrates between the two:

- **Theory MCP** → reasoning, axioms, Z3 verification, numerical computation (structured data)
- **Jupyter MCP** → display: plots, rendered equations, tables (rich output in notebook cells)

The notebook becomes a reproducible record of the agent's exploration — formal proofs
alongside numerical experiments and visualizations.

### Document Generation MCP

Kleis compiles Typst to PDF for arXiv papers and theses. A document MCP could let the
agent assemble formal results, numerical outputs, and rendered equations into publication-ready
documents. The agent's theory exploration session could culminate in a structured paper draft
rather than a conversation transcript.

### Architecture Sketch

```
Agent (Cursor / IDE)
   │
   ├─ kleis-theory MCP ──── formal reasoning, Z3, LAPACK
   │
   ├─ kleis-jupyter MCP ─── visual output, interactive notebooks
   │
   └─ kleis-document MCP ── PDF generation, arXiv/thesis formatting
```

Each MCP does one thing well. The agent is the orchestrator. None of these companion MCPs
are in scope for this ADR — they are noted here as natural extensions that the architecture
supports without modification to the theory MCP itself.

