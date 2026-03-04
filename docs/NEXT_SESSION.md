# Next Session Notes

**Last Updated:** February 26, 2026 (session 8 — cleaned up, archives created)

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

The Kleis-based Rust structural parser (`rust_parser.kleis`) is intentionally **not** a compiler-grade parser. It's a lightweight scanner for review tooling. Rule authors should be aware of these sharp edges:

1. **Brace depth is lexical, not semantic.** `brace_delta(line)` counts `{`/`}` even inside string literals, raw strings, and comments. This can skew nesting depth and any body-size metrics. Fix: a lightweight string/comment-aware brace counter (still not a full tokenizer).

2. **Block comments are not nest-aware.** Continuation detection uses `contains("*/")`, but Rust block comments can nest (`/* /* */ */`). Robust "ignore content in comments" needs a nesting counter rather than a boolean `in_block`.

3. **Multi-line item headers may be incomplete.** Function signatures, `where` clauses, and attributes can span lines. The scanner works line-by-line, so some item facts may be partial unless a "header accumulation" mode is added.

4. **Macros can masquerade as items.** `macro_rules!`, attribute macros, and DSL-like macros can confuse `is_*_line` heuristics. This is acceptable for review tooling but should be documented so users don't assume compiler-grade accuracy.

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
