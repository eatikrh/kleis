# Next Session Notes

**Last Updated:** March 6, 2026 (session 17 — eval_concrete + Z3 Matrix Solving, stdlib Alignment)

---

## Session 17 (Mar 6, 2026): eval_concrete + Z3 Matrix Solving, stdlib Alignment

### What Was Done

**eval_concrete integration with Z3** — reduce concrete sub-expressions (e.g. `multiply(ones, ones)` → `Matrix(2,2,...)`) before sending to Z3, preserving the top-level equality for symbolic solving. This enables the equation editor to verify matrix multiplication results.

**stdlib alignment** — aligned `stdlib/lists.kleis` and `stdlib/matrices.kleis` with eqnlib notation (infix `=` instead of `equals()` wrapper, `→` guards, constructor injectivity axioms, associativity/distributivity).

**Server switched from eqnlib to stdlib** — equation editor server now loads `stdlib/minimal_prelude.kleis`, `stdlib/lists.kleis`, `stdlib/matrices.kleis` (36 structures instead of 11 from eqnlib).

### Key Changes
- `server.rs`: apply `eval_concrete` only to LHS/RHS of top-level `equals`, not the equality itself; switched from eqnlib to stdlib
- `backend.rs`: gate axiom loading to prevent double-load, transactional rollback for `declared_ops` on axiom translation failure, skip injectivity axioms
- `ast.rs`: `decompose_constructor_equalities` with uppercase-first guard, `collect_ops` methods
- `comparison.rs`: return `Err` on sort mismatch instead of panicking
- `axiom_verifier.rs`: transactional `declared_ops` restore on failure
- `stdlib/lists.kleis`: `equals()` → infix `=`, added `→` guard on `nth_succ`, added `ListConstructor` injectivity
- `stdlib/matrices.kleis`: `MatrixConstructor` now has injectivity axiom, added `MatrixMulAssoc` + `MatrixDistributive`
- New `eqnlib/` directory with base, lists, matrix, and test_matrix libs

### Branch / PR
`feature/eqnlib-z3-matrix` — PR #153

### `kleis test` Failure Inventory (as of session 17)

All 2186 Rust tests (`cargo test`) pass. Below are the `kleis test` failures.

#### eqnlib/
- All 6/6 pass (including distributivity)

#### stdlib/
| File | Result | Failures |
|------|--------|----------|
| `combinatorics.kleis` | 10/12 | `all perms 2`, `n perms equals n factorial` — `concat()` doesn't support nested lists |
| `tensors_functional.kleis` | 13/29 | `tensor_get`, `tensor_add`, `tensor_scale`, wedge ops, EM tensor, d-squared — evaluator limitations with tensor indexing and wedge product computation |

#### examples/ (non-ontology)
| File | Result | Issue |
|------|--------|-------|
| `chess/chess.kleis` | 1/10 | Evaluator limitations |
| `contractbridge/contractbridge.kleis` | 3/8 | Evaluator limitations |
| `debug_main.kleis` | 0/1 | Symbolic assertion can't verify |
| `inverted_pendulum_discrete.kleis` | 0/1 | `dlqr` not fully evaluated |
| `ontology/pot_core.kleis` | 8/9 | Z3 quantifier inconclusive |
| `ontology/spacetime_type.kleis` | 0/1 | `component(make(...))` not reduced |
| `petri-nets/mutex_bounded.kleis` | 9/11 | Z3 counterexample |
| `petri-nets/mutex_example.kleis` | 0/4 | Z3 counterexample |
| `petri-nets/mutex_verified.kleis` | 2/8 | Z3 counterexample |
| `sudoku/sudoku.kleis` | 6/10 | Solver limitations |

#### examples/ (errors — parse or panic)
| File | Error |
|------|-------|
| `hardware/alu_verification.kleis` | Panic: `bvadd` sort mismatch in Z3 |
| `hardware/simple_alu.kleis` | Panic: `bvadd` sort mismatch in Z3 |
| `export/render_to_typst_demo.kleis` | Missing import file |
| `debug_test.kleis` | Parse error |
| `lps/test.kleis` | Parse error |
| `mass_from_residue.kleis` | Parse error |
| `ontology/pot_foundations.kleis` | Parse error |

#### examples/ontology/revised/
| File | Result | Issue |
|------|--------|-------|
| `bell_violation_test.kleis` | 9/9 | |
| `cosine_uniqueness_test.kleis` | 4/5 | `z3_inconsistency_detector` (expected failure) |
| `minimal_admissable_kernel_class.kleis` | 2/2 | |
| `pot_arxiv_paper.kleis` | 8/8 | |
| `pot_channel_units.kleis` | 1/1 | |
| `pot_core_kernel_projection.kleis` | — | Z3 hangs (killed after 12+ min) |
| `pot_entanglement_paper.kleis` | 21/24 | `spinor_2d_basis`, `R_irreducibility`, `theorem1_singlet_correlation` |
| `rotation_curve_numerical.kleis` | 2/2 | |

### Open Items
1. **`bvadd` sort mismatch panic** — `hardware/alu_verification.kleis` and `hardware/simple_alu.kleis` panic in Z3 bitvector operations. Needs investigation.
2. **`pot_core_kernel_projection.kleis` hangs Z3** — too many axioms loaded; Z3 solver doesn't terminate.
3. **Petri net counterexamples** — Z3 finds counterexamples for mutex properties. May need axiom refinement or different encoding.
4. **Tensor indexing in evaluator** — `tensor_get`, `tensor_add`, `tensor_scale` not reduced by evaluator.
5. **Parse errors** — `debug_test.kleis`, `lps/test.kleis`, `mass_from_residue.kleis`, `ontology/pot_foundations.kleis` have parse errors.

---

## Session 16 (Mar 5, 2026): Configurable Per-Language LLM Guidelines Prompt

### What Was Done

**Configurable LLM guidelines** — load per-language coding standards into the LLM advisory system prompt so the reviewer checks against specific guidelines (Microsoft Rust, PEP 8, etc.) instead of generic heuristics.

**Grounded findings** — require every LLM finding to cite a specific line number and code snippet. Findings without a line reference are silently dropped, eliminating hallucinated/parroted guideline violations.

- **Config** (`src/config.rs`): Added `guidelines_file: Option<String>` to `LlmConfig` + `PartialLlm` + `KLEIS_LLM_GUIDELINES_FILE` env var override.
- **Advisory** (`src/review_mcp/advisory.rs`):
  - `resolve_guidelines_path()` — 4-step resolution: env var > config > auto-discovery (`examples/guidelines/{lang}_guidelines.txt`) > none
  - `load_guidelines_text()` — reads file, skips comment-only placeholder files
  - `build_system_prompt()` — structured prompt with guidelines + formal rule names when available, generic fallback otherwise
  - `add_line_numbers()` — prepends line numbers to source so LLM can cite them
  - `Advisory` struct now has `line: Option<u32>` and `evidence: Option<String>`
  - `parse_advisories()` filters out findings without a line number
  - 15 unit tests (8 new: prompt generation, resolution order, grounding, line numbers)
- **CLI** (`src/bin/kleis.rs`): Loads guidelines for detected language, extracts formal rule names from engine, passes both to LLM. Renders `(line N)` and evidence snippet.
- **Guidelines files**: `examples/guidelines/rust_guidelines.txt` (condensed Microsoft Pragmatic Rust Guidelines, 157 lines / 8.7KB — distilled from 90KB original), `examples/guidelines/python_guidelines.txt` (placeholder).

### Key Design Decisions

- **Condensed guidelines (8.7KB not 90KB)**: Full guidelines wasted ~22K tokens on prose/examples an LLM already knows. Condensed to guideline ID + one-line rule + "Check for" triggers. ~2100 tokens.
- **Grounded findings**: Without line numbers + evidence requirement, gpt-4o-mini parroted guidelines back as fabricated findings (5/5 were hallucinated in first test). With grounding, findings cite real code and ungrounded ones are filtered out.
- **Per-language**: Resolution auto-discovers `{lang}_guidelines.txt` so adding Python/Go guidelines is just dropping a file.

### Branch / PR

`feature/llm-guidelines-prompt` — merged via PR #151 into `feature/microsoft-rust-guidelines`

### Files Changed
- `src/config.rs` — guidelines_file in LlmConfig + PartialLlm + env override
- `src/review_mcp/advisory.rs` — guidelines resolution, grounded prompts, line numbers, evidence
- `src/bin/kleis.rs` — guidelines loading, rule name extraction, evidence rendering
- `examples/guidelines/rust_guidelines.txt` — condensed Microsoft Rust Guidelines
- `examples/guidelines/python_guidelines.txt` — placeholder

---

## Session 15 (Mar 5, 2026): Advisory Severity Levels for Review Rules

### What Was Done

**Advisory severity levels** — two-tier rule system (`check_*` = blocking error, `advise_*` = non-blocking advisory) so style/documentation rules don't break CI.

- **Engine** (`src/review_mcp/engine.rs`): Added `RuleSeverity` enum (Error, Advisory), `severity` field on `RuleVerdict`, `AdviseFunction` variant on `ReviewRuleKind`. `check_code` and `check_diff` discover both prefixes; only `check_*` failures set `passed = false`. Summary shows three-way counts (errors/advisories/passed).
- **CLI** (`src/bin/kleis.rs`): Advisory failures render as `⚠️` instead of `❌`. Only `check_*` failures contribute to exit code 1 — advisories never break CI.
- **MCP Server** (`src/review_mcp/server.rs`): JSON verdicts include `"severity": "error"|"advisory"`. `list_rules` and `describe_standards` show separate sections. `explain_rule` reports severity-aware kind.
- **Policy** (`examples/policies/rust_review_policy.kleis`): 19 rules renamed from `check_*` to `advise_*` (style, docs, team patterns, AI artifacts). 29 rules remain as blocking `check_*` (safety, security, clippy -D, structural).
- **Tests** (`tests/review_mcp_test.rs`): 2 new tests (`test_advisory_failures_do_not_block`, `test_advisory_summary_counts`). Updated emoji test references and stat assertions. All 36 tests pass.

### Note: LLM advisories (`--advise`) are a separate system

The LLM advisory path (`src/review_mcp/advisory.rs`, `Advisory` struct with `severity: String`) is independent of `RuleSeverity`. Both are non-blocking, but they flow through different code paths. No unification was done — they're conceptually aligned but structurally separate.

### Branch
`feature/microsoft-rust-guidelines`

### Files Changed
- `src/review_mcp/engine.rs` — RuleSeverity enum, severity on verdicts, advise_* discovery
- `src/review_mcp/server.rs` — severity in JSON, list_rules/explain_rule sections
- `src/bin/kleis.rs` — advisory emoji rendering, exit code logic
- `examples/policies/rust_review_policy.kleis` — 19 rules renamed to advise_*
- `tests/review_mcp_test.rs` — 2 new tests, updated assertions

### Microsoft Rust Guidelines Coverage Analysis

The current policy covers **~15 of ~88** combined guidelines from the Microsoft Pragmatic Rust Guidelines and Rust API Guidelines. The covered rules are the ones mechanically detectable via string matching or structural AST analysis.

**What the current scanner CAN'T address** (architectural/runtime, ~50 rules):
M-SMALLER-CRATES, M-HOTPATH, M-THROUGHPUT, M-YIELD-POINTS, M-DESIGN-FOR-AI, M-MOCKABLE-SYSCALLS, M-IMPL-IO, M-INIT-CASCADED, M-INIT-BUILDER, M-DI-HIERARCHY, M-SIMPLE-ABSTRACTIONS, C-BUILDER, C-NEWTYPE, C-OBJECT, C-GENERIC, etc. These require human/LLM judgment or runtime profiling — the `--advise` LLM path is the right tool.

**What an improved Rust parser/scanner COULD address** (~20-25 more rules):
- **Type resolution** → M-PUBLIC-DISPLAY, M-TYPES-SEND, M-ERRORS-CANONICAL-STRUCTS, precise C-GOOD-ERR
- **Trait impl tracking** → C-COMMON-TRAITS (Debug, Clone, PartialEq on pub types), C-CONV-TRAITS, C-DEREF
- **Expression-level parsing** → M-PANIC-ON-BUG, M-REGULAR-FN, precise clippy-style checks
- **Doc comment structure** → M-FIRST-DOC-SENTENCE, M-CANONICAL-DOCS, M-MODULE-DOCS, M-DOC-INLINE, per-fn C-FAILURE

**Low-hanging fruit (no parser upgrade needed):**
- M-DOC-INLINE: `pub use` without `#[doc(inline)]` — string match
- M-PUBLIC-DISPLAY: pub structs missing `Display` derive — structural check (already have pub struct detection)
- M-FIRST-DOC-SENTENCE: doc comment length — structural check on doc comments

A parser with type resolution and trait impl tracking could bring coverage from ~15/88 to ~35-40/88.

---

## Session 14 (Mar 5, 2026): Native Rust Scanner (`scan_rust` builtin)

### What Was Done

**Native Rust structural scanner** — hand-written tokenizer + recursive descent parser (~2400 lines, zero dependencies) that emits Kleis AST identical to the Kleis-based `scan()` in `rust_parser.kleis`.

- **Tokenizer**: Handles string literals (including raw strings `r#"..."#`), all 6 comment types (line, outer/inner line doc, block, outer/inner block doc with nesting), attributes, keywords, punctuation, lifetimes, spans with line numbers.
- **Recursive descent parser**: Parses top-level items (fn, struct, enum, trait, impl, use, mod, const, static, type, macro_rules!), visibility variants (pub, pub(crate), pub(super), pub(self)), function qualifiers (async, const, unsafe, extern), generic parameters, `where` clauses, and computes `body_line_count` + `max_nesting` for function bodies.
- **Kleis AST emission**: Internal Rust AST types (`FnDecl`, `StructDecl`, etc.) convert to Kleis `Expression` via `to_expr()` methods, producing `Crate(items, comments, line_count)` — identical structure to the Kleis-based scanner.
- **`\n` auto-detection**: Matches the `foldLines` builtin behavior — detects whether source contains real newlines or escaped two-char `\n` from Kleis string literals.
- **`scan()` delegation**: `rust_parser.kleis` now delegates `scan(source)` to the native `scan_rust(source)` builtin. All 146 helper functions, 17 data types, and review query functions are unchanged.
- **19 Rust unit tests** + **25/25 Kleis example tests** pass.
- **`kleis review` integration verified** — ran against `verify-cli/src/storage/*.rs` (8 files, 86 rules). Structural rules (`check_structural`, `check_safe_structural`, `check_secure_structural`) fire correctly with accurate line numbers.

### Resolved Limitations

These limitations from the Kleis-based scanner are now fixed:

1. ~~**Brace depth is lexical, not semantic.**~~ — **RESOLVED**: The native tokenizer skips braces inside string literals and comments.
2. ~~**Block comments are not nest-aware.**~~ — **RESOLVED**: The native tokenizer correctly handles nested block comments (`/* /* */ */`).
3. ~~**Multi-line item headers may be incomplete.**~~ — **RESOLVED**: The native parser operates on the full token stream, so multi-line function signatures, `where` clauses, and attributes parse correctly.

### Branch
`feature/rust-scanner`

### Files Changed
- `src/rust_scanner/mod.rs` — module root (new)
- `src/rust_scanner/scanner.rs` — tokenizer + parser + Kleis AST emission (new, ~2400 lines)
- `src/lib.rs` — added `pub mod rust_scanner`
- `src/evaluator/builtins.rs` — `scan_rust` builtin registration
- `examples/meta-programming/rust_parser.kleis` — `scan()` delegates to `scan_rust()`

### Architecture: Why Hand-Written

Evaluated Pest (PEG), LALRPOP (LR(1)), Nom (combinators), and rust-peg. All add dependencies and generate full expression/type parsers we don't need. The native scanner only needs structural extraction (items, signatures, metrics) — a two-phase tokenizer + recursive descent is the right tool. Grammar reference: IntelliJ Rust BNF (MIT).

### Performance

The native scanner processes the full token stream in a single pass. Previously, `scan()` used Kleis-interpreted `foldLines` which executed hundreds of Kleis function calls per source line. The native version eliminates this overhead entirely.

---

## Session 13 (Mar 5, 2026): Equation Editor Z3 + Axiom Consistency Investigation

### What Was Done

**Equation Editor witness display** (stashed, not merged)
- Wired `PrettyPrinter` into `check_sat_handler` and `verify_handler` for human-readable Z3 witness output
- Tracked free variables in `quantifier_vars` so `model_to_witness` extracts structured bindings

**Axiom loading investigation** (stashed, not merged)
- Loading ALL stdlib axioms at once via `initialize_from_registry()` causes UNSAT — but **the individual axioms are proven correct** (tensor symmetries, Einstein equations, Bell violations, Cartan algebra all pass their Z3 proofs)
- The issue is **bulk loading strategy**, not axiom correctness. Each `.kleis` proof file loads only the structures it needs; the Equation Editor was the first place we tried loading everything into one Z3 context
- When abstract algebra structures (`Field(F)`, `Ring(R)`) are loaded with type parameters defaulting to `Int`, and `×` maps to Z3's integer multiplication, the combination creates unsatisfiable constraints — but that's a loading problem, not a math problem
- Added `ConsInjectivity` and `MatrixInjectivity` axioms to stdlib (stashed) — mathematically correct, need proper loading context

### Key Finding: Equation Editor Needs Selective Axiom Loading

The Equation Editor should load axioms the same way `.kleis` proof files do — selectively, based on what the user is working with. The `initialize_from_registry()` bulk-load approach was the wrong strategy. Options:
1. **Load on demand** — detect which structures the expression references, load only those
2. **User-driven** — let the user choose which theory context to work in (matrices, tensors, etc.)
3. **Expression analysis** — inspect the AST for operation names, load matching structures

### Branch
`fix/equation-editor-witness-display` — changes stashed (`git stash`), branch clean

### Stashed Changes
- `src/bin/server.rs` — PrettyPrinter witness display + `initialize_from_registry()` call
- `src/solvers/z3/backend.rs` — parametric structure skip filter + free var tracking
- `stdlib/lists.kleis` — `ConsInjectivity` axioms
- `stdlib/matrices.kleis` — `MatrixInjectivity` axioms
- `docs/NEXT_SESSION.md` — session notes

### Open Items
1. **Equation Editor witness display** — the PrettyPrinter fix itself is clean and correct, but was bundled with the axiom loading work. Could be extracted as a standalone change.
2. **Selective axiom loading for Equation Editor** — needs a strategy to load only relevant structures (like `.kleis` files do), not all 68+ at once.
3. **Matrix Z3 semantics** — `ConsInjectivity` and `MatrixInjectivity` axioms are ready (stashed), need proper loading context in the Equation Editor.

---

## Session 12 (Mar 4, 2026): Polyglot Review — Python Parser, MCP, End-to-End

### What Was Done

**Python Scanner (Rust)**
- **`scan_python(source)` builtin** — hand-written line scanner (~600 lines, zero dependencies) emitting nested Kleis AST
- **9 Kleis data types** — `PyModule`, `PyItem`, `PyFunction`, `PyClass`, `PyStmt`, `PyImport`, `PyFromImport`, `PyDecorator`, `PyExceptHandler`
- **12 query helpers** in `python_types.kleis` — `module_functions`, `module_classes`, `has_decorator`, `count_list`, etc.
- **Code organized** under `src/python/` (scanner.rs + mod.rs)

**Python Review Policy (46 rules)**
- **12 string-based checks** — `check_no_eval`, `check_no_sys_exit`, `check_no_mutable_defaults`, `check_no_bare_except`, `check_no_print_statement`, `check_no_environ_bracket`, `check_no_optional_type`, `check_no_hardcoded_password`, `check_no_debug_breakpoint`, `check_double_quote_strings`, `check_no_wildcard_import`, `check_no_eval`
- **1 structural check** (`check_python_structural`) with 6 sub-rules: long functions, long methods, import placement, bare except (AST with line numbers), missing return types (skips `__init__`), excessive try/except
- **7 diff-aware rules** — `diff_check_image_tag_bump`, `diff_check_requirements_pinned`, `diff_check_file_growth`, `diff_check_new_fns_typed`, `diff_check_sys_exit_introduced`, `diff_check_bare_except_introduced`, `diff_check_print_introduced`
- **Rules inferred from 1,038 MR comments** across 403 MRs in `sso-pipelinelib`

**Polyglot MCP Architecture**
- **Separate MCP instances per language** — `kleis-review-rust` and `kleis-review-python` (not a single MCP with naming hacks)
- **Dynamic server name** — derived from policy filename (`python_review_policy.kleis` → `kleis-review-python`)
- **Language-aware LLM advisory** — `build_system_prompt` accepts language parameter, code fences use correct language tag
- **Stdlib import resolution** — `KLEIS_ROOT` env var + directory walk for `stdlib/` imports, works from any working directory
- **Git context from target files** — `git_repo_root_for(dir)` derives repo root from the files being reviewed, not cwd

**End-to-End Validation**
- Tested `kleis review` CLI against `sso-pipelinelib` — catches real issues (mutable defaults, missing return types, print(), Optional[], sys.exit())
- Tested all MCP tools: `list_rules`, `describe_standards`, `explain_rule`, `check_file`, `check_code`
- **AI agent autonomy test** — a fresh Cursor agent in `sso-pipelinelib` discovered `kleis-review-python`, queried its rules, reviewed 3 changed files, and proposed the correct fix — with zero prior knowledge of Kleis

### Branch
`feature/python-parser`

### Files Changed
- `src/python/scanner.rs` — Python line scanner (new)
- `src/python/mod.rs` — module root (new)
- `src/lib.rs` — added `pub mod python`
- `src/evaluator/builtins.rs` — `scan_python` builtin
- `src/evaluator/mod.rs` — removed old `python_bridge` module
- `src/review_mcp/advisory.rs` — language-aware prompts
- `src/review_mcp/engine.rs` — stdlib import resolution via `KLEIS_ROOT`
- `src/review_mcp/server.rs` — dynamic server name from policy filename
- `src/bin/kleis.rs` — `language_from_path`, `git_repo_root_for`, target-file git context
- `examples/meta-programming/python_types.kleis` — Kleis data types + helpers (new)
- `examples/policies/python_review_policy.kleis` — full Python policy (new)
- `.cursor/mcp.json` — parallel `kleis-review-rust` / `kleis-review-python`
- `docs/manual/src/chapters/28-agent-mcps.md` — polyglot MCP documentation
- `.cursorrules` — "no practical workarounds" rule

### Known Limitations (Python Scanner)
- **Multi-line function signatures** — extracts params from first line only
- **Multi-line `from` imports** — parses first line only
- **Triple-quote tracking** — doesn't distinguish docstrings from strings
- **No expression parsing** — assignments capture target but not value

### Migration Path
If structural rules need expression-level detail, add `ruff_python_parser` (MIT, Rust crate) behind a feature flag. Replace scanner internals; Kleis data types and policies stay unchanged.

### Architecture Decision: Separate MCPs per Language
- Each language gets its own MCP instance with its own policy, advisory prompt, and structural parser
- Cleaner than language-prefix naming conventions (`check_py_*` / `check_rs_*`)
- Future: Kleis structures could namespace rules (`structure PythonReview { ... }`) — the engine would discover `check_*` inside structures instead of only top-level functions

### Open Items
1. **No timeouts** — `eval_concrete` and Z3 can block indefinitely. STILL OPEN.
2. **`check_no_hardcoded_urls` false positive** — flags documentation URLs in comments. Needs structural version that skips comments.
3. **Z3 axioms not wired into automatic review** — `SafeCode`, `SqlSafe` etc. require explicit `evaluate_expression` calls.
4. **Vertex AI auth for `--advise`** — wire `gcloud auth print-access-token` into `advisory.rs` so `kleis review --advise` can use corporate Claude without a static API key.
5. **Semver comparison for diff rules** — `diff_check_version_bump` currently checks "different" but not "greater". Add proper `version_gt(a, b)`.
6. **Generic `extract_key_value`** — needs Kleis lambda/closure support in `foldLines`.
7. **Externalize `build_system_prompt` text** — load from file or config so users can customize without recompiling.

---

## Session 7 (Feb 26, 2026): Rebase, Conflict Resolution, and Merged PRs

### Merged PRs
- **#135** — STRIDE threat model rules, concrete Z3 verification, expanded review coverage
- **#136** — Structural Rust parsing, superseded string checks removed, docs updated, check_file tests

### Current State
- **28 active check_* functions**: 23 string-based + 5 structural (AST-based with line-number reporting)
- **6 Z3 concrete tests** + **6 check_file tests** + original tests = 34 total review MCP tests
- **Rust structural parser** (`rust_parser.kleis`) operational: `scan()`, `production_code()`, `fn_body_text()`, `non_test_fns_containing()`
- **Three-tier review model** documented in `28-agent-mcps.md`: string checks / structural checks / Z3 axioms

### Open Items
1. **No timeouts** — `eval_concrete` and Z3 can block indefinitely. STILL OPEN.
2. ~~**`evaluator.rs` is 10,887 lines**~~ — **DONE** via PR #137. Split into `src/evaluator/` with 7 modules.
3. **`check_no_hardcoded_urls` false positive** — flags documentation URLs in comments. Needs structural version that skips comments.
4. **Z3 axioms not wired into automatic review** — `SafeCode`, `SqlSafe` etc. require explicit `evaluate_expression` calls. Future: parser extracts code fragments, feeds to Z3.
5. ~~**NEXT_SESSION.md is 147K chars**~~ — **DONE**. Cleaned up: archives created, trimmed to ~106 lines.

### Known Limitations: `rust_parser.kleis` Structural Scanner

The Rust structural parser now delegates to a native Rust scanner (`scan_rust` builtin, session 14). Most previous limitations are resolved:

1. ~~**Brace depth is lexical, not semantic.**~~ — **RESOLVED** (session 14): Native tokenizer skips braces inside strings/comments.

2. ~~**Block comments are not nest-aware.**~~ — **RESOLVED** (session 14): Native tokenizer handles nested block comments.

3. ~~**Multi-line item headers may be incomplete.**~~ — **RESOLVED** (session 14): Native parser operates on full token stream.

4. **Macros can masquerade as items.** `macro_rules!` is parsed; attribute macros and DSL-like macros may confuse item detection. Acceptable for review tooling.

### Known Limitations: `kleis_review_policy.kleis` Checks

5. **Security checks are intentionally blunt.** Checks like `contains(prod, "password =")` and `format!(..SELECT..)` work as guardrails but will produce false positives in test fixtures, docs, and examples. Future: an allowlist mechanism or context-aware suppression.

6. **`production_code(source)` split is a correctness bottleneck.** The test-vs-production partition drives many checks. If it's too naive (e.g., misclassifying test helpers or integration tests), it either misses real problems or creates noise. Worth monitoring as the codebase evolves.

---

## Session 6 (Feb 23, 2026): Z3 Safety, Trigonometric Axioms, and Epistemic Boundaries

### CRITICAL: What You Need to Know

1. **Z3 global timeout crashes the solver.** Do NOT set `KLEIS_Z3_TIMEOUT_MS` to a nonzero value unless debugging. Z3's internal timeout fires mid-quantifier-processing and causes `ASSERTION VIOLATION` in `smt_context.cpp` (segfault). Default is now 0 (no timeout). The watchdog via `ContextHandle::interrupt()` is the safe wall-clock timeout.

2. **Universal trig axioms cause E-matching divergence.** We tried `stdlib/trigonometry.kleis` with `∀(a b : ℝ). cos(a+b) = cos(a)*cos(b) - sin(a)*sin(b)`. Z3's E-matching explodes: the nonlinear products in the addition formula interact with the Pythagorean identity, creating infinite instantiation chains (observed 13000+ quantifier instances before killing). **Ground instances at specific angles are the correct approach for Z3.**

3. **`neg_cos` replaced with `cos` in the entanglement theory.** `pot_entanglement_v2.kleis` now uses `cos` directly. `spin_half_overlap` reads naturally: `spinor_inner(proj_a, proj_b) = cos(angle_between(a, b))`.

### What Was Accomplished

1. **Z3 timeout default fixed** — `KLEIS_Z3_TIMEOUT_MS` default changed from 5000 to 0. Global Z3 params (timeout, rlimit, memory, soft_timeout) are now only set when explicitly nonzero. This fixed a regression where `pot_arxiv_paper.kleis` was crashing with Z3 ASSERTION VIOLATION at `smt_context.cpp:2485`.

2. **Trigonometric axioms explored** — Created `stdlib/trigonometry.kleis` with full axiomatic cos/sin (Pythagorean, addition formulas, periodicity, bounds). Confirmed all 14 axioms assert in <10ms, but the consistency check diverges. **Deleted the file** — universal nonlinear real arithmetic is beyond Z3's E-matching capability.

3. **Ground cos instances added to entanglement theory:**
   - `cos(0) = 1`, `cos(pi) = -1` (base values)
   - `cos(pi/2) = 0`, `cos(pi/4)^2 = 1/2` (CHSH angles)
   - `BellWitnessAngles` structure with three detector angles at 0, pi/4, pi/2

4. **Bell violation test created** — `examples/ontology/revised/bell_violation_test.kleis` with 9 tests: cos values, correlation at specific angles, Bell LHS/RHS at CHSH witnesses. All 9 pass.

5. **Cosine uniqueness test updated** — `cosine_uniqueness_test.kleis` migrated from `neg_cos` to `cos`. 4/5 pass (1 expected failure = inconsistency detector).

### Files Modified
- `src/solvers/z3/backend.rs` — timeout default 0, gate global params on nonzero
- `src/bin/kleis.rs` — updated `--help` for KLEIS_Z3_TIMEOUT_MS (default: 0, caution note)
- `theories/pot_entanglement_v2.kleis` — replaced neg_cos with cos, added BellWitnessAngles, updated BellCorrelation and AnticorrelationLemma
- `examples/ontology/revised/cosine_uniqueness_test.kleis` — migrated to cos
- `examples/ontology/revised/bell_violation_test.kleis` — **NEW**, 9 tests for Bell violation at CHSH angles

### Files Deleted
- `stdlib/trigonometry.kleis` — universal trig axioms cause E-matching divergence

### Test Results
- `pot_arxiv_paper.kleis`: 8/8 (regression clean)
- `bell_violation_test.kleis`: 9/9
- `cosine_uniqueness_test.kleis`: 4/5 (1 expected failure)

### Key Findings: Epistemic Boundaries in the Entanglement Theory

**The "Unknown" verdicts from Z3 are correct.** They represent the boundary between:
- **What algebra proves** (linearity, group actions, inner product invariance) — Z3 verifies these
- **What representation theory / analysis proves** (Schur's lemma, Wigner D-matrices, cosine from character theory) — Z3 returns Unknown

**Tightening `is_admissible` (e.g., constraining H_ont's codomain to C^3) does NOT help** because the Unknown axioms are all about SU(2) acting on SpinorField (C^2), not about the kernel's codomain (FieldR4). The projection `project_at` has already dropped from FieldR4 to SpinorField by the time any Unknown axiom is evaluated.

**The path to closing the gap:**
- **Short term:** Ground cos instances (done) — Z3 can verify the Bell violation with concrete values
- **Medium term:** Kleis evaluator as CAS bridge — compute representation theory results, feed to Z3 as ground truths
- **Long term:** Isabelle/HOL integration as a solver backend for formal proofs of representation theory (Schur's lemma, Wigner D-matrix classification)

The cos/sin addition formulas encode the Lie algebra structure of SU(2). They're not external computational facts — they're the content of the ontological commitment "SU(2) is a symmetry of H_ont." The ground instances carry the same ontological content as the universal axioms; Z3 just can't handle the universal form.

### Lessons Learned

1. **Z3's internal timeout is dangerous.** It fires mid-processing and corrupts Z3's internal state. Always use the `ContextHandle::interrupt()` watchdog instead.
2. **Universal quantifiers with nonlinear products = E-matching bomb.** `∀(a b : ℝ). f(a+b) = g(a)*g(b) - h(a)*h(b)` is a pattern Z3 cannot handle. Ground instances are the workaround.
3. **Don't put Z3-hostile axioms in stdlib.** Axioms that cause E-matching divergence should not be in shared libraries. Ground instances belong in the theory files that need them.
4. **Epistemic honesty > verification completeness.** "Unknown" is a valid answer when the mathematics genuinely requires tools beyond SMT (representation theory, analysis). Don't weaken the theory to get "Verified."

### NEXT_SESSION.md Cleanup — DONE
- [x] Mark completed items from sessions 1-5
- [x] Archive sessions older than 2 weeks to `docs/archive/sessions/`
- [x] Keep NEXT_SESSION.md focused on active work + last 2-3 sessions
- [x] Extract future/roadmap items to `docs/ROADMAP.md`
- [x] Archive POT physics notes to `docs/archive/pot-physics-notes.md`

### kleis-review — Context-Aware Parsing for Reduced False Positives

~~The current `kleis-review` MCP uses string matching for code review rules, producing false positives where syntactic context matters.~~ **All three items resolved with structural (AST-based) rules:**

- ~~**`check_no_wildcard_import`** flags `use super::*;` in test modules~~ — **DONE**: `rule_wildcard_imports` uses `non_test_wildcard_uses(c)`, skips test modules.
- ~~**`check_no_narrating_comments`** flags doc comments~~ — **DONE**: `rule_narrating_line_comments` uses `has_narrating_line_comment(crate_comments(c))`, distinguishes `//` from `///`.
- ~~**`check_no_inline_use`** flags `use` inside function bodies~~ — **DONE**: `rule_use_in_fn_body` uses `non_test_fns_containing(source, fns, "use ")`, skips test functions.

---
