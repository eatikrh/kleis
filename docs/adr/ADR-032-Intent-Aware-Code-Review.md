# ADR-032: Intent-Aware Code Review

**Status:** Proposed  
**Date:** 2026-03-14  
**Related:** ADR-030 (MCP Agent Reasoning Partner), ADR-014 (Hindley-Milner Type System), ADR-016 (Operations in Structures)

---

## Context

The Kleis review MCPs (`kleis-review-rust`, `kleis-review-python`) and the CLI
(`kleis review`) enforce code quality through three tiers: string checks,
structural/AST analysis, and Z3-verified properties. The `agent_policy.kleis`
gates actions before they happen. Together they answer: "Is this code good?"

They do not answer: "Is this code *the right code for this change*?"

A developer fixing an input validation bug who accidentally refactors the
parser will pass every quality check. The review engine sees clean code; it
does not see scope drift. A feature branch that adds a public API without
documentation passes if the project doesn't mandate docs globally — but a
*feature* branch should.

The Agentic Continuous Delivery (ACD) framework
(https://migration.minimumcd.org/docs/agentic-cd/) addresses this by requiring
per-change specification artifacts: intent descriptions, BDD scenarios, feature
constraints, and acceptance criteria. This approach has two problems:

1. **Ceremony.** Writing BDD specs before every change turns agents into
   test-script executors. The spec layer becomes a chore that developers game
   (padding specs to satisfy the gate) rather than a tool that catches mistakes.

2. **Redundancy.** Intent already exists in multiple places: commit messages,
   MR/PR descriptions, branch names, issue trackers, ADRs, and session documents
   like `NEXT_SESSION.md`. Requiring a new artifact duplicates what the team
   already produces.

The insight is that intent is not missing — it is **disconnected** from the
review engine. The review engine runs rules against code in isolation. It does
not know what the change is *for*, which module's architectural constraints
apply, or whether the scope matches the stated goal.

---

## Decision

**Introduce a three-layer review architecture that connects intent to code
review without adding ceremony.**

### Layer 1: Project Standards (Always-On)

The existing `check_*`, `advise_*`, and Z3-backed rules in
`rust_review_policy.kleis` and `python_review_policy.kleis`. These apply to
every file, every change, unconditionally. They encode team conventions and
hard-won lessons.

**No change required.** This layer is implemented and working.

### Layer 2: Module Standards (Topology-Driven)

A stable mapping from file paths to the architectural standards that govern
them. This is the project's structural DNA — it changes when ADRs change, not
when commits change.

```kleis
define module_standards(path) =
    if hasPrefix(path, "src/type_inference") then
        "adr-014: constraint-based inference, principal types, no hardcoded results"
    else if hasPrefix(path, "src/kleis_parser") then
        "grammar-spec: changes must match docs/grammar/kleis_grammar_v05.ebnf"
    else if hasPrefix(path, "src/evaluator") then
        "adr-016: operations defined in structures, not hardcoded in Rust"
    else if hasPrefix(path, "src/review_mcp") then
        "review-architecture: three tiers, check_*/advise_*/rule_* conventions"
    else if hasPrefix(path, "stdlib/") then
        "type-system: structures are source of truth for types"
    else "general"
```

Review rules reference this mapping to activate context-specific checks:

```kleis
define check_module_compliance(source) =
    let path = review_path() in
    let standards = module_standards(path) in
    if contains(standards, "adr-014") then
        if contains(source, "hardcoded") ∨ contains(source, "// HACK") then
            "fail: ADR-014 — no hardcoded type results, use constraint-based inference"
        else "pass"
    else "pass"
```

This layer bridges the existing `before_file_edit` preconditions in
`agent_policy.kleis` (which gate *actions*) into the review engine (which
evaluates *code quality* in context).

**Recursive self-consistency:** The grammar-spec module standard deserves
special attention. The structural review tier (Tier 2) is itself built on
grammar-based parsing — `scan_rust` and `scan_python` parse source code into
ASTs that review rules then query. The grammar is not just an architectural
constraint on the parser module; it is the **foundation of the review system's
correctness**. If someone changes the Kleis parser without updating the
grammar, the structural review tier could produce wrong ASTs and therefore
wrong review verdicts. The module standard `"grammar-spec: changes must match
docs/grammar/kleis_grammar_v05.ebnf"` is a meta-consistency requirement: the
review system that checks code depends on the same grammar discipline it
enforces.

This property extends to the target-language parsers (`scan_rust`,
`scan_python`). These are grammar-based parsers that produce ASTs from Rust
and Python source code respectively. If a source file cannot be parsed — due
to syntax errors, unsupported constructs, or parser limitations — the
structural review tier cannot produce an AST, and the review degrades
gracefully to an automatic "cannot review" verdict for structural checks.
The file is not silently skipped or assumed clean; the inability to parse is
itself a signal. String-based checks (Tier 1) and Z3 axioms (Tier 3) still
run, but the structural analysis that catches context-sensitive violations
(`.unwrap()` in production vs test code, naming conventions, parameter
counts) is explicitly reported as unavailable.

### Layer 3: Change Intent (Per-Change)

Intent flows into the review engine from different sources depending on
context. The key insight is that **intent exists before commits do** — in an
MCP agent session, the agent already knows what it's doing from the
conversation, the ticket, or `NEXT_SESSION.md`. Commit messages are only one
source, and they arrive late.

**Intent sources by context:**

| Context | Intent source | When available | Richness |
|---------|-------------- |----------------|----------|
| MCP session (pre-commit) | Direct conversation with user | Before any code is written | Highest — full context of why |
| MCP session (pre-commit) | `NEXT_SESSION.md` | Session start | High — accumulated plans, lessons, constraints |
| MCP session (pre-commit) | ADRs | Always | High — architectural rationale |
| MCP session (pre-commit) | Issue tracker ticket | When ticket ID known | Medium — story/task description |
| Pre-push hook | Commit messages on the branch | After commit, before push | Medium — what and why |
| CI/CD pipeline | Commit messages + MR description | After push | Medium — what and why |
| CI/CD pipeline | Issue tracker API (from branch name) | With API integration | Medium — story/task description |

In an **MCP session**, the agent populates intent from conversation context.
When the user says "fix the input validation for special characters," the
agent passes that as intent to `check_code` / `check_file`. No commit exists
yet — the intent is proactive, not retrospective.

In **CI/CD**, commit messages are the primary source because the conversation
is over. The CLI accepts intent via `--intent`:

```bash
INTENT=$(git log --format='%s' origin/main..HEAD | tr '\n' ' ')
kleis review src/**/*.rs -p policy.kleis --intent "$INTENT" --base-branch main
```

In both cases, the policy accesses intent the same way via `review_intent()`:

```kleis
define check_fix_has_test(source) =
    let intent = review_intent() in
    if contains(intent, "fix") ∨ contains(intent, "bug") then
        let c = scan(source) in
        if length(crate_test_functions(c)) = 0 then
            "fail: bug fix — add at least one test"
        else "pass"
    else "pass"
```

If intent is not provided, `review_intent()` returns an empty string and
all intent-aware rules silently pass. Fully backwards-compatible.

**The MCP advantage:** In an agent session, intent-aware review is
**preventive** — the agent checks code against stated intent as it writes,
before committing. In CI/CD, intent-aware review is **detective** — it catches
misalignment after the fact. The same `review_intent()` mechanism serves both,
but the MCP path catches problems earlier because intent is available sooner.

### Semantic Intent Extraction (Optional, LLM-Assisted)

Simple `contains()` checks on commit messages work for conventional commit
prefixes (`fix:`, `feat:`, `refactor:`). For richer semantic understanding, the
existing LLM advisory engine can extract structured intent:

```
Commit messages (natural language)
    ↓ LLM extracts
Structured intent: { domain, change_type, components, constraints, risk_areas }
    ↓ consumed by
Kleis review rules (formal checks against structured intent)
```

This is optional and uses the same `--advise` infrastructure. The LLM is a
**translator** between natural language and formal review rules — it does not
review the code itself.

### Intent Coherence (Advisory, Not Blocking)

The review engine checks whether the commit messages are coherent with the
files changed. These are always `advise_*`, never `check_*` — they inform
without blocking:

```kleis
define advise_intent_coverage(source) =
    let intent = review_intent() in
    let path = review_path() in
    let standards = module_standards(path) in
    if eq(intent, "") then
        "advise: no commit message context — consider describing why, not just what"
    else if contains(standards, "adr-014") ∧ not(contains(intent, "type"))
         ∧ not(contains(intent, "infer")) then
        "advise: change touches type inference but commit doesn't mention types — intentional?"
    else "pass"
```

Key principle: the review asks questions ("intentional?") rather than issuing
verdicts. It treats the developer as an adult. A commit that says "cleanup" and
touches three unrelated modules gets a gentle flag, not a pipeline failure.

---

## How This Relates to Existing Kleis Practices

The three layers formalize what the project already does informally:

| Practice | Layer | Currently |
|----------|-------|-----------|
| ADRs (`docs/adr/`) | Layer 2 | Permanent architectural intent; read by humans and agents |
| `NEXT_SESSION.md` | Layer 3 | Session-to-session working memory; read by agents at start |
| `.cursorrules` | Layer 2 | Project-wide constraints; read by agents always |
| `agent_policy.kleis` preconditions | Layer 2 | Action gates before editing specific modules |
| Review policy `check_*` rules | Layer 1 | Always-on code quality enforcement |
| Commit messages | Layer 3 | Per-change intent; currently not consumed by review |

The only *new* elements are:
1. `--intent` CLI flag and `review_intent()` built-in function
2. `module_standards()` convention in review policies
3. `review_path()` built-in function (current file being reviewed)
4. Optional LLM-based structured intent extraction

---

## Kleis Capabilities Against the ACD Framework

The Agentic Continuous Delivery framework
(https://migration.minimumcd.org/docs/agentic-cd/) defines eight constraints
for agent-generated changes. This section maps each constraint to Kleis's
current and proposed capabilities.

### ACD Prerequisites

ACD requires these CD foundations before agents can contribute. Kleis's
position on each:

| ACD Prerequisite | Kleis Implementation | Status |
|------------------|---------------------|--------|
| Continuous Integration | Pre-push hook: fmt, clippy, test, manual validation, sitemap | **Implemented** |
| Testing Fundamentals | `cargo test --all` (unit + integration + vendored crate tests) | **Implemented** |
| Build Automation | `./scripts/build-kleis.sh` — single command, handles Z3 headers, installs binary | **Implemented** |
| Work Decomposition | ADRs define scope; `NEXT_SESSION.md` tracks increments | **Practiced** |
| Code Review | Review MCPs with three-tier analysis (string, structural, Z3) | **Implemented and exceeds** |
| Everything as Code | Policies in `.kleis`, infrastructure in version control, rules declarative | **Implemented** |
| Single Path to Production | Pre-push hook is the single gate; `kleis review` CLI for CI/CD | **Implemented** |
| Deterministic Pipeline | Same inputs → same outputs; Z3 proofs are deterministic | **Implemented** |

### ACD Constraint 1: Explicit, Human-Owned Intent for Every Change

**ACD says:** Every change needs an intent description — a problem statement
and hypothesis owned by a human.

**Kleis approach:** Intent exists at three time horizons, none of which require
a new per-change artifact:

| Time Horizon | Artifact | Owner |
|-------------|----------|-------|
| Permanent | ADRs (`docs/adr/`) — 31 architectural decisions | Human |
| Session | `NEXT_SESSION.md` — current work, plans, lessons | Human |
| Per-change | Commit messages — what and why | Human |

ADRs are stronger than ACD's intent descriptions because they are
**accumulative**: ADR-014 (Hindley-Milner) still constrains every type system
change today. ACD's intent descriptions are disposable per-change documents.

**With this ADR (Layer 3):** Commit messages are connected to the review engine
via `--intent` / `review_intent()`, closing the gap without new ceremony.

### ACD Constraint 2: Intent and Architecture as Delivery Artifacts

**ACD says:** Intent and architecture must be represented as versioned
artifacts delivered with the change.

**Kleis approach:**

- ADRs are versioned in `docs/adr/` — delivered with every commit
- `.cursorrules` defines project-wide architectural constraints
- `agent_policy.kleis` codifies preconditions per module
- Review policies (`rust_review_policy.kleis`) encode team standards

**With this ADR (Layer 2):** `module_standards()` explicitly maps files to the
ADRs and specs that govern them, making architecture a first-class input to
code review.

### ACD Constraint 3: Delivery Artifacts Versioned with the Change

**ACD says:** All delivery artifacts must be versioned and delivered together.

**Kleis approach:** ADRs, policies, `.cursorrules`, and review rules are all in
the same repository, versioned by git. When a policy changes, it ships with the
code change that motivated it.

**Status:** **Satisfied.** No gap.

### ACD Constraint 4: Behavior Represented Independently of Implementation

**ACD says:** Use BDD scenarios or similar to represent intended behavior
separately from code.

**Kleis approach:** This is the one area where Kleis diverges from ACD by
design. Kleis does not require BDD scenarios. Instead:

- Z3-verified axioms in structures define **formal behavioral properties**
  (e.g., `SafeCode`, `SqlSafe`) that hold over all possible inputs — stronger
  than individual BDD scenarios
- Integration tests in `tests/` verify concrete behavior
- `example` blocks in `.kleis` files serve as executable specifications checked
  by Z3

Z3 axioms are arguably more rigorous than BDD scenarios: a BDD scenario checks
one path; a Z3 axiom proves a property for all paths.

**Status:** **Different approach, equivalent or stronger guarantees.**

### ACD Constraint 5: Consistency Between Intent, Tests, Implementation, Architecture

**ACD says:** Enforce that all artifacts are mutually consistent.

**Kleis current state:** Review checks code quality in isolation — no
cross-artifact consistency checking.

**With this ADR:** The three-layer architecture connects them:

- Layer 1 enforces implementation quality (existing)
- Layer 2 enforces implementation-architecture consistency (new:
  `module_standards()` ties code to ADRs)
- Layer 3 enforces implementation-intent consistency (new: `review_intent()`
  ties code to commit messages)
- Advisory checks flag drift between layers without blocking

**Status:** **Gap being closed by this ADR.**

### ACD Constraint 6: Agent Changes Comply with Documented Constraints

**ACD says:** Agent-generated changes must follow all documented rules.

**Kleis approach:** This is Kleis's strongest area:

| Mechanism | What it enforces |
|-----------|-----------------|
| `kleis-policy` MCP | Action gating: file edits, deletions, commands, git operations |
| `kleis-review-rust` MCP | Rust code quality: ~30 string checks, ~11 structural checks, 4 Z3 structures |
| `kleis-review-python` MCP | Python code quality: ~11 string checks, ~7 structural checks |
| LLM advisory (`--advise`) | Architectural judgment: API design, patterns, module org, type safety — guided by per-language guidelines and role-based prompts |
| Pre-push hook | fmt, clippy, all tests, manual validation, sitemap |
| `kleis review` CLI | Same rules in CI/CD pipelines (including `--advise` for LLM tier) |

ACD describes "expert validation agents" conceptually. Kleis has them running
at four tiers:

- **Tier 1-3 (formal):** MCP servers with Z3-backed verification — the
  `SqlSafe` structure proves no tainted input reaches a query without
  sanitization. That is not a heuristic; it is a theorem.
- **Tier 4 (advisory):** An LLM given a specific role ("you are a Rust code
  reviewer"), 158 guidelines from Microsoft's Pragmatic Rust Guidelines, and
  explicit instructions not to duplicate what the formal engine already caught.
  The LLM handles the judgment calls (API design, abstraction boundaries,
  performance patterns) that formal rules cannot.

**Status:** **Exceeds ACD requirements.**

### ACD Constraint 7: Agents Cannot Promote Their Own Changes to Production

**ACD says:** The agent that writes the code must not be able to push it to
production.

**Kleis approach:**

```kleis
define check_git_push(branch, force) =
    if force = 1 then "deny"
    else if branch = "production" then "deny"
    else "allow"
```

Z3-verifiable: `∀(b : String, f : ℤ). implies(f = 1, check_git_push(b, f) = "deny")`
proves force-push is always denied regardless of branch.

**Status:** **Implemented and formally verified.**

### ACD Constraint 8: Red Pipeline → Agents Fix Only

**ACD says:** While the pipeline is red, agents may only make changes that
restore pipeline health.

**Kleis current state:** The pre-push hook blocks pushes that fail quality
gates (fmt, clippy, tests). An agent cannot push broken code. However, there is
no explicit "fix-only mode" that restricts what the agent can *work on* during
a red pipeline — it simply cannot push until the pipeline is green.

**Status:** **Partially satisfied.** The pipeline is self-healing in the sense
that broken code cannot advance. A dedicated "fix-only mode" in
`agent_policy.kleis` could strengthen this by restricting file edits to
pipeline-related files when the last push failed.

### Summary: Kleis vs ACD

| ACD Constraint | Kleis Status | Mechanism |
|---------------|-------------|-----------|
| 1. Human-owned intent | **Satisfied** (ADRs + NEXT_SESSION.md + commits) | Three time horizons, no new artifacts |
| 2. Intent as delivery artifacts | **Satisfied** (ADRs + policies in repo) | With this ADR: `module_standards()` |
| 3. Artifacts versioned together | **Satisfied** | All in same git repo |
| 4. Behavior independent of implementation | **Different, stronger** | Z3 axioms > BDD scenarios |
| 5. Cross-artifact consistency | **Gap → closing** | This ADR: three-layer review |
| 6. Agent compliance with constraints | **Exceeds** | Policy MCP + Review MCPs + Z3 |
| 7. Agents cannot promote to production | **Verified** | `check_git_push` + Z3 proof |
| 8. Red pipeline → fix only | **Partial** | Pre-push blocks; no explicit fix-only mode |

### Where Kleis Is Architecturally Ahead of ACD

ACD's "expert validation agents" are described as a future aspiration for most
teams. Kleis has them shipping:

1. **Four-tier review** — string checks, structural/AST analysis, Z3 axioms,
   and LLM advisory (not just linting)
2. **Formal verification of review rules** — Z3 proves properties over all
   inputs, not just test cases
3. **Formal verification of the policy itself** — agents can prove
   `∀(d : String). implies(check_git_commit(d) = "allow", isAscii(d))`
4. **Review as dialogue** — MCP tools (`explain_rule`, `evaluate`,
   `describe_standards`) let agents reason about rules, not just obey them
5. **Declarative, hot-reloadable rules** — add a `check_*` function to the
   `.kleis` file, restart the MCP, and the rule is live. No recompilation.
6. **Dual-mode** — same rules run interactively (MCP) and in CI/CD (`kleis review`)
7. **LLM advisory with role prompting and deduplication** — see below

### LLM Advisory: Role-Based Expert Reviewer

The fourth tier of Kleis's review system is the LLM advisory
(`src/review_mcp/advisory.rs`). When `--advise` is passed, the review engine
calls an OpenAI-compatible LLM endpoint with a carefully structured prompt:

**System prompt structure:**

```
"You are a {language} code reviewer enforcing the coding standards below.

IMPORTANT: A formal static analysis tool (Kleis) has already checked for:
- check_no_unwrap
- check_safe_structural
- advise_no_emoji
- ... (all loaded check_*/advise_* rule names)
Do NOT repeat those. Only report NEW findings from the guidelines that the
formal tool cannot check.

Focus especially on architectural and design guidelines that require judgment
(e.g., module organization, API design, type safety, design patterns,
abstraction boundaries, error design, performance).

## Coding Standards
{contents of examples/guidelines/rust_guidelines.txt}
..."
```

**Key design decisions:**

1. **Role assignment** — The LLM is told "you are a {language} code reviewer
   enforcing the coding standards below." It is given a specific role with
   specific standards. When per-language guidelines are available (the primary
   path), the prompt includes the full standards text and focuses the LLM on
   architectural and design judgment. When no guidelines file exists, a generic
   fallback prompt is used that still assigns the reviewer role but covers
   broader concerns (bugs, idioms, naming, performance).

2. **Per-language guidelines files** — Standards are loaded from
   `examples/guidelines/{language}_guidelines.txt`. The Rust guidelines include
   158 rules from Microsoft's Pragmatic Rust Guidelines, organized by category
   (safety, documentation, API design, performance, resilience, UX). These are
   external, version-controlled, and human-readable.

3. **Formal check deduplication** — The system prompt lists every `check_*` and
   `advise_*` rule name from the loaded policy. The LLM is explicitly told not
   to repeat findings that the formal engine already caught. This prevents the
   two tiers from producing duplicate noise.

4. **Anti-hallucination grounding** — The LLM must provide a specific line
   number and code snippet for every finding. Findings without a line number
   are filtered out (`filter(|item| item.line.is_some())`). The LLM cannot
   report violations it cannot point to in the source.

5. **Structured JSON output** — The LLM returns a JSON array of findings, each
   with `check` (guideline ID), `line`, `severity`, `evidence`, and `reason`.
   This integrates cleanly with the same output format as formal checks.

6. **Never blocks the pipeline** — Advisory findings use `⚠️` and `ℹ️` markers
   and never affect the exit code. Formal checks use `❌`/`✅` and control
   the pipeline.

**The two-tier model (formal + advisory) in action:**

```
❌ src/server.rs
  ❌ check_safe_structural — .unwrap() in production fn: handle_request (line 42)
  ❌ check_structural — pub fn missing doc comment: process (line 15)
  ℹ️  [advisory] M-INIT-BUILDER — 4+ optional params in Config::new (line 28)
  ⚠️  [advisory] M-STRONG-TYPES — String used for file path, consider PathBuf (line 55)
```

Formal verdicts block. Advisory findings inform. The LLM handles the
architectural and design judgment that string matching and AST analysis cannot
— precisely the "expert agent" role that ACD describes but leaves abstract.

**Relevance to intent-aware review:** The system prompt in
`build_guidelines_prompt()` (`src/review_mcp/advisory.rs`, lines 195-219) is
the concrete integration point for Layer 3. When `--intent` is provided, an
additional section is appended to the prompt:

```
## Change Intent

The developer described this change as:
{intent from commit messages}

Verify that the code is consistent with this stated intent.
Flag code that appears unrelated to the intent or contradicts it.
Do NOT block on scope — use severity "info" for drift observations.
```

This gives the LLM three simultaneous review lenses:

1. **Standards compliance** — does this code follow the guidelines? (existing)
2. **Formal deduplication** — don't repeat what Kleis already caught (existing)
3. **Intent alignment** — does this code match what the developer said they're
   doing? (new, from `--intent`)

When intent is absent, the section is omitted and the prompt is unchanged —
fully backwards-compatible. The LLM advisory is the natural home for intent
alignment because it requires the kind of semantic judgment that formal rules
(`check_*`, `advise_*`) cannot provide. A `contains(intent, "parser")` check
can detect keyword mismatch; the LLM can detect that "fix input validation
for special characters" and a change to the Typst rendering pipeline are
semantically unrelated — even if neither mentions "parser."

---

ACD acknowledges that "human review at Test Validation and Code Review is an
interim state" to be replaced by expert agents. Kleis already made that
replacement for mechanical review (formal checks with Z3 proofs) and is
progressively replacing it for architectural review (LLM advisory with
role-based prompting and per-language standards).

---

## Alternatives Considered

### Alternative 1: ACD Specification Layer

**Approach:** Require BDD scenarios, intent descriptions, and acceptance
criteria as versioned artifacts for every change.

**Pros:**
- Explicit, unambiguous intent
- Machine-verifiable specification-to-implementation consistency

**Cons:**
- Heavy ceremony; developers game it
- Duplicates intent already in commit messages, MR descriptions, and issue trackers
- Per-change artifacts become stale and are rarely updated after initial write
- Turns agents into test-script executors rather than informed contributors

**Rejected:** Intent should flow from where it naturally lives (commit
messages, ADRs, topology), not from a parallel artifact system.

### Alternative 2: Conventional Commits Only

**Approach:** Rely on `fix:`, `feat:`, `refactor:` prefixes to drive
intent-aware rules.

**Pros:**
- Simple, no LLM needed
- Already a common convention

**Cons:**
- Labels, not intent — `fix:` says nothing about *what* is being fixed
- Cannot express domain, scope, constraints, or risk areas
- Collapses all fixes into one category regardless of criticality

**Rejected as sole source.** Useful as a baseline signal, but insufficient
for semantically rich intent. The `--intent` flag accepts the full commit
message, not just the prefix.

### Alternative 3: Gitignored `feature.kleis` Per Branch

**Approach:** A local `.kleis` file declares the work item (ticket, intent,
scope, constraints). Gitignored so it never enters history.

**Pros:**
- Structured intent in Kleis syntax
- Available to MCP sessions and local review

**Cons:**
- Not available in CI/CD (gitignored)
- Requires manual creation per branch
- Another file to maintain

**Not rejected but deferred.** This could complement `--intent` for local MCP
sessions where richer structure is useful. It is not the primary mechanism.

---

## Implementation Plan

### Phase 1: Core Plumbing

1. Add `--intent <string>` flag to `kleis review` CLI
2. Implement `review_intent()` built-in that returns the intent string
3. Implement `review_path()` built-in that returns the current file path
4. Both return empty strings when not set (backwards-compatible)

### Phase 2: Module Standards Convention

1. Document `module_standards(path)` convention in review policy files
2. Add initial topology mapping for the Kleis project itself
3. Add `check_module_compliance` and `advise_intent_coverage` examples to
   both `rust_review_policy.kleis` and `python_review_policy.kleis`

### Phase 3: MCP Integration

1. Accept `intent` parameter in `check_code` and `check_file` MCP tools
2. Make `review_intent()` available in MCP review sessions
3. Agent populates intent from whichever source is available:
   - Direct user instruction ("fix the input validation for special characters")
   - `NEXT_SESSION.md` (read at session start, carries accumulated context)
   - Relevant ADR (agent reads ADR-014 before touching type inference)
   - Issue tracker ticket (agent extracts ticket ID from branch name or user mention)
4. Intent in MCP is **proactive** — available before code is written, enabling
   preventive review during development rather than detective review after commit

### Phase 4: Semantic Extraction (Optional)

1. Add `--extract-intent` flag that pipes commit messages through the LLM
   advisory engine to produce structured intent
2. Structured intent available via `review_intent_structured()` built-in
3. Requires `--advise` infrastructure (LLM endpoint configured)

---

## Consequences

### Positive

1. **No new ceremony** — intent flows from commit messages, not new artifacts
2. **Backwards-compatible** — all new features are opt-in; existing policies
   work unchanged
3. **Three layers separate concerns** — project quality, architectural
   compliance, and change-level intent are independently maintainable
4. **Advisory by default** — intent coherence checks inform, never block
5. **Bridges MCP and CI/CD** — same `review_intent()` function works in both
   agent sessions and pipeline runs
6. **Leverages existing practices** — ADRs, `NEXT_SESSION.md`, commit messages,
   and topology are already part of the workflow

### Negative

1. **Intent quality depends on commit messages** — garbage in, garbage out;
   mitigated by advisory checks that flag vague messages
2. **Module topology is manual** — `module_standards()` must be maintained as
   the codebase evolves; mitigated by keeping it in the review policy file
   alongside the rules it activates
3. **LLM extraction adds latency and cost** — mitigated by making it optional
   (`--extract-intent` flag) and using the existing advisory infrastructure

---

## Files to Change

- `src/review_mcp/engine.rs` — accept and expose intent string, path tracking
- `src/review_mcp/protocol.rs` — add `intent` parameter to `check_code` / `check_file`
- `src/bin/kleis.rs` — add `--intent` CLI flag to `review` subcommand
- `src/evaluator/builtins.rs` — `review_intent()` and `review_path()` built-ins
- `examples/policies/rust_review_policy.kleis` — add `module_standards()` and intent-aware rules
- `examples/policies/python_review_policy.kleis` — same
- `tests/review_mcp_test.rs` — integration tests for intent-aware review
- `docs/manual/src/chapters/28-agent-mcps.md` — document the three-layer architecture

