# Next Session Notes

**Last Updated:** February 26, 2026 (session 7)

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
2. **`evaluator.rs` is 10,887 lines** — too large for the structural parser to review. Needs splitting into modules (`eval_core`, `eval_builtins`, `eval_z3`, `eval_types`, `eval_io`).
3. **`check_no_hardcoded_urls` false positive** — flags documentation URLs in comments. Needs structural version that skips comments.
4. **Z3 axioms not wired into automatic review** — `SafeCode`, `SqlSafe` etc. require explicit `evaluate_expression` calls. Future: parser extracts code fragments, feeds to Z3.
5. **NEXT_SESSION.md is 147K chars** — needs cleanup per session archival rules.

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

### TODO: NEXT_SESSION.md Cleanup
- [ ] Mark completed items from sessions 1-5
- [ ] Move permanent knowledge (Z3 lessons, architecture decisions) to `docs/` subdirectories
- [ ] Archive sessions older than 2 weeks to `docs/archive/sessions/`
- [ ] Keep NEXT_SESSION.md focused on active work + last 2-3 sessions

### Future: kleis-review — Context-Aware Parsing for Reduced False Positives

The current `kleis-review` MCP uses string matching (`contains`, `hasPrefix`, `hasSuffix`) for code review rules. This works well for most patterns but produces false positives in cases where syntactic context matters:

- **`check_no_wildcard_import`** flags `use super::*;` in test modules (idiomatic Rust).
- **`check_no_narrating_comments`** flags doc comments (`/// Create a new attribute`) that legitimately start with a verb.
- **`check_no_inline_use`** flags `use` inside function bodies, which is valid Rust (e.g., inside tests or to limit import scope).

Fixing these requires parsing structure, not just string matching. A future version of the review engine should support AST-aware or at least scope-aware checks — for example, suppressing certain rules inside `#[cfg(test)]` modules or distinguishing `///` doc comments from `//` line comments.

---

## Session 5 (Feb 22, 2026): Axiom Consistency Detection + non_separable Bug Found

### CRITICAL: What You Need to Know

**The entanglement paper (`pot_entanglement_paper.kleis`) does NOT compile.**
Kleis now detects that its axiom set is **inconsistent** (contradictory).
All 24 Z3-verified assertions were vacuously true — derived from `False`.

### What Was Accomplished

1. **Axiom Consistency Detection implemented** — Kleis now checks whether loaded axioms are mutually satisfiable before verifying any assertions.
   - `SolverBackend::check_consistency()` — two-phase approach:
     - Phase 1: bare `solver.check()` with 5s timeout
     - Phase 2: MBQI (Model-Based Quantifier Instantiation) with 15s timeout
   - `AxiomVerifier` runs the check once per lifetime; `Evaluator` caches result across assertions
   - New variant `VerificationResult::InconsistentAxioms` propagated through evaluator, REPL, MCP, theory engine
   - Clear error: "AXIOM INCONSISTENCY DETECTED: The loaded axioms are mutually unsatisfiable..."

2. **Root cause identified: `non_separable` axiom in `pot_entanglement_v2.kleis`**
   - The axiom says: for ALL `psi_A, psi_B`, the entangled state `psi_AB` cannot be decomposed as `flow_add(psi_A, psi_B)`
   - But `flow_add_id` says: `flow_add(x, flow_zero) = x`
   - Instantiating with `psi_A = psi_AB, psi_B = flow_zero` gives: `project_at(G, psi_AB, a) ≠ project_at(G, psi_AB, a)` — contradiction

3. **Rust unit test** (`test_consistency_check_detects_quantifier_inconsistency`) reproduces the exact Z3 behavior

4. **Files modified:**
   - `src/solvers/backend.rs` — added `check_consistency` to `SolverBackend` trait
   - `src/solvers/z3/backend.rs` — two-phase consistency check implementation + unit test
   - `src/axiom_verifier.rs` — consistency check integration + cache API
   - `src/evaluator.rs` — `axiom_consistency_cache` field, cache propagation in `verify_with_z3`
   - `src/repl.rs` — `InconsistentAxioms` display
   - `src/mcp/policy.rs`, `src/theory_mcp/engine.rs` — `InconsistentAxioms` handling
   - `tests/axiom_verification_integration_test.rs`, `tests/z3_tensor_test.rs`, `tests/multi_level_structure_test.rs`, `tests/logical_operators_test.rs` — new match arms

### What Must Happen Next

1. **Fix the `non_separable` axiom** in `theories/pot_entanglement_v2.kleis`
   - Option A: Remove it entirely — rely on `Separability.entangled_exists` for non-separability
   - Option B: Refine it to exclude trivial decompositions (e.g., require `psi_A ≠ flow_zero` and `psi_B ≠ flow_zero`)
   - Either way, `pot_entanglement_paper.kleis` must compile clean afterward

2. **Re-derive the cosine law** — once axioms are consistent, check which claims still hold
   - The `correlation_def` sign may also need fixing (gives +1 at θ=0, expected -1)

3. **Re-verify ALL entanglement paper assertions** against the corrected axiom set

### Test Results (This Session)

| Test | Result |
|------|--------|
| Flat rotation curves paper (`pot_arxiv_paper.kleis`) | 8/8 pass (consistent) |
| Entanglement paper (`pot_entanglement_paper.kleis`) | 0/24 — **INCONSISTENT** |
| Cosine uniqueness test (`cosine_uniqueness_test.kleis`) | 0/6 — **INCONSISTENT** |
| 857 lib unit tests | All pass |
| 62 integration tests | All pass |

### Lessons Learned

- Z3's E-matching is **order-dependent** for universally quantified axioms. A bare `solver.check()` may return Unknown even for provably inconsistent axioms, depending on assertion order. MBQI is more reliable.
- **Caching** the consistency result is essential — without it, the 20-second check (5s + 15s) runs per assertion, making consistent files unbearably slow.
- The `•` (group operation) type mismatch (`Real` vs `Int`) in Group/AbelianGroup is a separate pre-existing issue that should also be addressed.

---

## Session 4 (Feb 22, 2026): GHZ Contextuality Added to Entanglement Paper + Z3 Int→Real Fix

### What Was Accomplished

1. **GHZ contextuality section added to entanglement paper** (`pot_entanglement_paper.kleis`)
   - New Section 6: "Beyond Bell: The GHZ Contextuality Test"
   - Three subsections: GHZ state/parities, noncontextual contradiction (Theorem 5), POT contextual consistency (Theorem 6)
   - Abstract, intro, conclusion, appendix, keywords all updated
   - GHZ reference (Greenberger, Horne, & Zeilinger 1989) added
   - Two new Z3 verification examples: `theorem5_ghz_no_go` (UNSAT), `theorem6_pot_ghz_contextual` (SAT)
   - Paper is now 14 pages covering both 2-particle (Bell) and 3-particle (GHZ)

2. **Refactored GHZ theory file** (`theories/pot_ghz_contextuality_v1.kleis`)
   - Now imports `pot_entanglement_v1.kleis` instead of duplicating all structures

3. **Fixed Z3 backend Int→Real type promotion** (`src/solvers/z3/backend.rs`)
   - **`Expression::Const`**: Now parses float literals (`f64`) and converts to Z3 `Real` via rational representation
   - **Uninterpreted function calls**: Auto-promotes `Int→Real` when function signature expects `Real`
   - This fixed `theorem1_singlet_correlation` which was failing due to `neg_cos(0)` type mismatch
   - **Result: 10/10 tests pass** (was 9/10 before)
   - Remaining `•` warnings are stdlib Group/AbelianGroup Real→Int (reverse direction, correctly not auto-demoted)

### Known Issue: Equation Alignment in ArxivSubsection

**Problem:** Multi-line display equations inside `ArxivSubsection` strings don't align at the `=` sign.

**What was tried (none worked):**
- Typst `&=` alignment markers with `\\` line breaks in single math block
- Typst `grid()` with column alignment embedded as raw Typst in the string
- Typst `mat()` with `delim: #none`
- Separate `$ ... $` display equations (current approach — readable but unaligned)

**Root cause hypothesis:** The ArxivSubsection template wraps content in a Typst context that may interfere with math alignment. The `&` markers and `\` line breaks in the generated Typst output appear syntactically correct, but alignment doesn't take effect visually. Needs investigation of how `ArxivSubsection` renders its content string — likely `#text(...)` or similar wrapping that prevents math alignment from working across lines.

**Files to investigate:**
- `stdlib/templates/arxiv_paper.kleis` — how ArxivSubsection renders content
- `/tmp/align_test4.typ` — standalone Typst test file showing the issue (grid works in standalone but not through template)

**Impact:** Cosmetic only. The equations are correct and readable, just not perfectly column-aligned.

### CRITICAL: Technical Review of Entanglement Paper (Feb 22, 2026)

A rigorous technical review identified the following issues that must be addressed
before journal submission. These define the research program going forward.

**Issue 1: E10 (cosine law) is axiomatized, not derived (MOST IMPORTANT)**

The paper's language implies `E(a,b) = -cos(θ)` follows from kernel admissibility,
but E10 is an axiom — it is *imposed* as a structural axiom on the entangled-state
kernel. The admissible kernel axioms (A1–A4) constrain the class of kernels but do
not force the specific angular dependence.

*To resolve:* Derive `-cos(θ)` from kernel admissibility + SU(2) representation theory.
This requires showing that rotational covariance of admissible kernels constrains
the angular structure, and that the 2-component spinor representation of SO(3) via
SU(2) pins the kernel to cosine form. Essentially: re-derive Wigner's classification
inside the projection formalism. **This would be a separate paper.**

*Alternative (honest minimal fix):* Reword the paper to clearly state that E10 is a
structural axiom encoding the quantum prediction, and that the paper verifies
*consistency* of this axiom with the rest of the framework — not derivation from
first principles. Change "follows from" to "is consistent with."

**Issue 2: Non-separability may be QM entanglement in new notation**

POT denies Bell's separability assumption. This is logically valid. But a referee
will ask: is POT's non-separability mathematically distinguishable from standard QM
entanglement? If not, POT is a reinterpretation (respectable) not new physics.

*To resolve:* Either (a) show a structural constraint that kernel admissibility
imposes beyond what standard QM entanglement provides, or (b) acknowledge this
explicitly and position the paper as a reinterpretation with novel verification
methodology.

**Issue 3: No-signaling is assumed but not proved**

The paper implicitly assumes marginal independence `P(o_A | a, b) = P(o_A | a)` but
never proves it within the POT framework. A referee will demand this.

*To resolve:* Add a theorem proving no-signaling from the kernel factorization
structure. Specifically: show that summing/integrating over B's outcomes with the
kernel factorization yields a marginal that is independent of b.

**Issue 4: Kernel unification claim is philosophically suggestive but mathematically weak**

The gravity and measurement kernels share linearity (admissibility), but "many
linear operators exist." Shared linearity is necessary but insufficient for
unification.

*To resolve:* Prove a non-trivial structural theorem linking elliptic (gravity) and
spinor (measurement) kernel sectors. For example: show that a single admissibility
condition constrains BOTH the logarithmic Green's function structure and the SU(2)
inner-product structure in a way that is not individually obvious.

**Honest classification of the paper (current state):**
- Technically coherent: YES
- Mathematically consistent: YES
- Physically complete: NOT YET
- Philosophically mature: YES
- Machine-verification: VERY STRONG (genuinely novel)
- Classification: **Consistent ontological reinterpretation of QM** (not new physics yet)

**The paper becomes new physics if and only if:**
1. The cosine law is *derived* (not axiomatized) from kernel structure, OR
2. POT makes a different prediction from QM somewhere, OR
3. Kernel unification produces a non-trivial forced theorem linking gravity + measurement

**Priority for next session:**
1. (Minimal) Reword paper for honesty — E10 is a structural axiom, not a derivation
2. (Medium) Prove no-signaling theorem within POT framework
3. (Ambitious) Begin representation-theoretic derivation of cosine law from kernel admissibility + SU(2)
4. (Ambitious) Strengthen unification argument beyond shared linearity

---

## CURRENT WORK: POT Flat Rotation Curves — PROVED + Paper Written

### What Was Accomplished (Feb 21, 2026)

**Bug fix: Theory MCP crash was UTF-8 string slicing, NOT Z3.**
- `server.rs:472`: `&kleis_source[..80]` sliced inside multi-byte `ℝ` char
- Fixed all 3 instances to use `.chars().take(N).collect()`
- Also wrapped parser in `catch_unwind` for robustness
- **Important**: Must use `./scripts/build-kleis.sh` to build, not bare `cargo build` (sandbox redirects `CARGO_TARGET_DIR`)

**Flat rotation curves: Z3-verified theorems**
- Loaded `theories/pot_admissible_kernels_v2.kleis`
- Submitted 4 structures: ModalDensity, ModalPhase, ProjectedPotential, LinearMassGrowth
- Saved as `theories/pot_flat_rotation_v1.kleis`
- Added GalacticTransition (inner Newtonian + outer flat)
- Saved as `theories/pot_flat_rotation_v2.kleis`
- Added TullyFisher + CoreScaling (M ∝ v⁴)
- Saved as `theories/pot_flat_rotation_v3.kleis`

**Z3-verified theorems (all PROVED):**
1. v²(r) = λ for r ≥ R_c — flat rotation curve
2. v²(r) > λ for 0 < r < R_c — rising inner region
3. v²(r₁) = v²(r₂) for both in outer region — flatness
4. v²(r_in) > v²(r_out) — transition from inner to outer
5. v² > 0 — real orbits
6. M(r) > 0 — non-degenerate matter
7. M_baryonic = a · v⁴ — Tully-Fisher relation

**Numerical computation:**
- `examples/ontology/revised/rotation_curve_numerical.kleis`
- 60-point curves: POT flat at 223.6 vs Newtonian declining to 91.3

**arXiv paper:**
- `examples/ontology/revised/pot_arxiv_paper.kleis`
- Compiles to PDF via Typst with live-computed rotation curve plots
- PDF at `examples/ontology/revised/pot_flat_rotation_paper.pdf`
- Live at https://kleis.io/docs/papers/pot_flat_rotation_curves.pdf
- Proper math typesetting (subscripts, display equations)
- Title: "Flat Galactic Rotation Curves as a Theorem of Projected Ontology"
- 10 references including recent weak-lensing and modified gravity results

**Session 3 additions (Feb 21, 2026):**
- Added two new references to the paper:
  - O'Brien, Chiarelli & Kerin (2024) — empirical MOND/conformal gravity fits (APS April Meeting)
  - Mistele, McGaugh, Lelli, Schombert & Li (2024) — weak lensing shows flat curves to 1 Mpc, BTFR at large radii (ApJL 969, L3)
- The Mistele paper is particularly relevant: NFW dark matter halos predict declining velocities beyond virial radius, but observations show no decline out to 1 Mpc — consistent with POT's logarithmic kernel which has no built-in truncation
- Compared POT with O'Brien et al. (2024) approach: they fit existing modified gravity models (MOND, conformal gravity) empirically; POT derives flat curves axiomatically from projection
- Compared POT with "Galactic Pizza" (Novais & Ribeiro, 2025 CTEC): literal mass growth over cosmic time vs POT's static geometric projection
- POT parameter count: 1 free parameter per galaxy (λ or equivalently R_c) vs dark matter's 2-3, MOND's 1 universal + 1 per galaxy
- Clarified Hont → R⁴ → R³ projection language in paper: "one natural interpretation" (not a POT assertion), removed factorization-independence claim (inconsistent with FieldR4 in Kleis axioms)

### Related Work to Review

- **François, J. & Ravera, L. (2025).** "Raising galaxy rotation curves via dressing." Phys. Rev. D 112(8). DOI: 10.1103/m9xl-9vvk
  - Uses the Dressing Field Method (DFM) to extract gauge-invariant corrections from GR + scalar fields
  - Produces flat rotation curves without dark matter; tested against SPARC database
  - Published in Phys. Rev. D — shows mainstream journals accepting "no dark matter" rotation curve papers
  - Key difference from POT: DFM works within GR (dressing the metric), POT works from a deeper projection framework
  - **TODO:** Get full PDF and compare in detail — does it derive Tully-Fisher? How many free parameters?

### Key Philosophical Principle: Why No Lagrangian (applies to ALL POT papers)

**The projection is lossy and we exist inside it.**

POT's refusal to postulate a Lagrangian for Hont is not a gap — it is a
consequence of the theory's own axioms. The projection Π is many-to-one
and non-invertible (Axiom: Irreversible Projection). Information is destroyed.

Both the theorist and any reasoning tool (human or AI) exist as outputs
of the projection — we ARE the projection. Therefore we cannot reconstruct
the complete dynamics (Lagrangian) of the source space Hont. Claiming to
know a specific Lagrangian would require information that the projection
provably destroyed. Any specific Lagrangian we write down would be one of
infinitely many consistent with the same projected observables.

What we CAN do: find the *constraints* (axioms) that the projection
preserves. These are the limiting conditions on the class of possible
dynamics in Hont. Everything beyond these constraints is in the nullspace
— epistemically inaccessible from within the projection.

This is POT's epistemological boundary — analogous to Godel's incompleteness
(can't fully describe the system you're in) or the halting problem (can't
predict behavior from inside). The axioms are the strongest statements
possible from this side of the projection.

**This applies to both the rotation curves paper and the entanglement paper.**
Both should reference this principle in their "Limitations" sections.
The rotation curves paper's "What This Paper Does Not Do" should be
updated to match the entanglement paper's treatment.

### Future Research Directions (from entanglement paper review)

**Reviewed by ChatGPT — refinements captured below.**

1. **Kernel factorization vs AQFT**: POT's K = K_univ · K_dyn · K_rep maps
   onto Haag-Kastler local algebras, DHR superselection sectors, and the
   time evolution automorphism. POT may be a "pre-algebraic QFT" — the step
   before local algebras, explaining where they come from.

   **Refinement (ChatGPT):** The mapping K_univ ↔ A is imprecise because
   AQFT's core object is the NET O ↦ A(O), not A alone. K_univ is better
   understood as "a recipe that yields a net after projection."

   **Bridge theorem needed:** From (Hont, Π, K), construct a net O ↦ A(O)
   and a state ω such that Haag-Kastler axioms hold (at least isotony +
   locality + some covariance) and correlations match standard QFT.

   **DHR connection:** K_rep looks DHR-ish (superselection sectors), but
   to earn the parallel we need a POT notion of "localized charge" that
   survives projection as a sector label.

2. **Projection as C*-algebra state restriction**: If Π is a state restriction
   ω|_{A(O)}, then GNS gives Hont for free, Tomita-Takesaki gives emergent
   time, and the split property relates to kernel composition.

   **Refinement (ChatGPT):** "Π is state restriction" should be sharpened to:
   "Π is a completely positive unital map (quantum channel) from the global
   algebra to an observable algebra, and restriction to regions O corresponds
   to composing with the inclusion/projection onto A(O)."

   Reason: "lossy projection" is more naturally a CP map / conditional
   expectation than mere restriction, unless we already have net structure.

   **Tomita-Takesaki caveat:** Modular time is canonical given (M, ω) but
   not automatically physical time. Must show that in the POT regime,
   modular flow corresponds to expected physical dynamics. That's a
   theorem-shaped goal, not a vibe.

3. **GHZ test (next Z3 verification target)**: The violin string analogy may
   break for 3-party entanglement (GHZ). GHZ has basis-dependent parity —
   a single measurement rules out hidden variables deterministically.

   **Concrete approach (ChatGPT):** Don't simulate amplitudes. Encode the
   four GHZ operator identities as Z3 theorems:
   ```
   (X⊗X⊗X)|GHZ⟩ = +|GHZ⟩
   (X⊗Y⊗Y)|GHZ⟩ = −|GHZ⟩
   (Y⊗X⊗Y)|GHZ⟩ = −|GHZ⟩
   (Y⊗Y⊗X)|GHZ⟩ = −|GHZ⟩
   ```
   In POT terms: define ψ_ABC as a single non-separable flow, define
   project_at with basis choice (X or Y), verify the four eigen-relations.
   If they hold → POT supports GHZ contextuality.
   If not → pinpoints exactly which axiom is missing (operator algebra
   structure, composition rules, or how basis enters K_rep).

   This is feasible with the current kleis-theory MCP.

   **Concrete GHZ session plan (from ChatGPT):**
   File: `theories/pot_ghz_contextuality_v1.kleis` (small, surgical)

   Step A — Show GHZ is UNSAT for pre-assigned outcomes:
   ```
   x_A, y_A, x_B, y_B, x_C, y_C ∈ {+1, -1}  (so x² = 1)
   x_A · x_B · x_C = +1
   x_A · y_B · y_C = -1
   y_A · x_B · y_C = -1
   y_A · y_B · x_C = -1
   → Multiply all four: (x_A·y_A)²·(x_B·y_B)²·(x_C·y_C)² = +1·(-1)³ = -1
   → But LHS = +1 (all squares). CONTRADICTION. Z3 confirms UNSAT.
   ```
   This proves no noncontextual hidden variable model works for GHZ.

   Step B — Show POT CAN satisfy the constraints (non-pre-assigned):
   Connect project_at(G, ψ_ABC, basis) to outcome variables.
   Basis choice (X or Y) parameterizes K_rep.
   Single ontological mode → constraints satisfied because outcomes
   are not pre-assigned; they depend on the projection basis.

   Step C — The diagnostic value:
   If Z3 validates → POT handles 3-party contextuality.
   If not → pinpoints missing axiom (operator algebra, composition,
   or basis-dependent K_rep).

   **DONE (this session):** Both Step A and Step B verified by Z3.
   - Step A: DISPROVED — no ±1 assignment satisfies all four GHZ parities
   - Step B: VERIFIED — POT's context-dependent outcomes satisfy all four
   Saved as theories/pot_ghz_contextuality_v1.kleis

   **Precision claim (from ChatGPT review, must include in paper):**
   "Step B does not 'solve' GHZ; it demonstrates that POT is not a
   noncontextual hidden-variable theory. GHZ specifically refutes the
   existence of a single context-independent value assignment for X and Y
   at each site. POT avoids this by making outcomes functions of the
   measurement context (projection basis), not pre-assigned values."

   **CP map refinement (from ChatGPT):**
   POT's projection reads as Heisenberg picture (maps observables/fields
   forward) while the "lossy state" reading is Schrödinger (maps states).
   Must pick one explicitly when formalizing. For modular theory
   (Tomita-Takesaki), need von Neumann algebras + faithful states,
   which is naturally Heisenberg/algebraic.

### Entanglement Paper: Formalization Plan (from prior notes)

**Core axioms to formalize in Kleis (ready for kleis-theory MCP):**

1. **Non-Separability Axiom**: ψ_AB is a single vector in Hont, NOT ψ_A ⊗ ψ_B.
   Denies the standard separability assumption. This is the axiom that sidesteps Bell.

2. **Enriched Modal Flow**: φ: X × R_τ → Cⁿ (vector-valued codomain for spin/flavor).
   Internal degrees of freedom encoded directly in the value type of the flow.

3. **Unified Projection**: π_A and π_B are the SAME operator Π evaluated at different
   spatial arguments. "Non-local" correlation = consistency condition of singular projection.

4. **Kernel Factorization**: K(x,ξ) = K_univ · K_dyn · K_rep
   - K_univ: universal structural sector
   - K_dyn: dynamical sector (elliptic for gravity, hyperbolic for propagation)
   - K_rep: representation-dependent sector (spin, flavor)
   This unifies the rotation curve kernel G with measurement kernel K(θ).

5. **Measurement as Kernel Parameterization**: Detector angle = alignment of projection
   operator. "Collapse" = context-dependent change in which modal components survive projection.

6. **Bell's Step Rejected**: POT rejects the factorization P(a,b) = ∫ A(a,λ)B(b,λ)ρ(λ)dλ.
   Since A and B are not separable systems, there is no independent "outcome A" to factorize.

7. **Interference Formula** (from neutrino work):
   P_{α→β}(t) = |Σᵢ U_{βi} e^{-iωᵢt} U*_{αi}|²
   Entangled measurements = different spatial samplings of same rotating modal vector.

**Proof strategy for Kleis:**
1. Define `EntangledState` as single vector in Hont (not tensor product)
2. Define projection operators π_A, π_B as same Π at different coordinates
3. Prove: correlation E(a,b) = -cos(θ) for spin-½ singlet follows from kernel structure
4. Show: Bell inequality |E(a,b) - E(a,c)| ≤ 1 + E(b,c) is violated
5. But without non-locality — because π_A and π_B are local projections of shared state

**Key distinction from rotation curves paper:**
- Rotation curves used scalar kernel (coherence function h(G,r))
- Entanglement uses matrix-valued kernel sector K_rep acting on spinor representation
- Both are sectors of the SAME unified kernel K — this is the Kernel Unification Theorem

### Next: Continue POT Physics Formalization

| Result | Status | What to Formalize |
|--------|--------|-------------------|
| **Flat rotation curves** | ✅ PROVED + Paper | Done — 7 theorems, numerical curves, arXiv paper |
| **Tully-Fisher relation** | ✅ PROVED | M = a·v⁴ verified by Z3 |
| **SR from projection slicing** | Conceptual work done | Formalize observer-dependent slicing kernels (see notes below) |
| **Neutrino oscillations** | Conceptual work done | Matrix-valued Green's function projection of enriched modal flow (see notes below) |
| **Schwarzschild weak-field** | Conceptual work done | Axiomatize vortex modal structure (see notes below) |
| **Einstein Cross / gravitational lensing** | Conceptual work done | Project lensing from Hont; needs analyticity + band-limitation constraints (see notes below) |
| **Quantitative galaxy fits** | Not started | Fit SPARC survey data, test R_c ∝ v² prediction |
| **Bullet Cluster** | Not started | Multi-kernel / extended coherence model |
| **Entanglement / non-locality** | Conceptual work done | Shared flow ⟹ correlated projections; sidesteps Bell (see notes below) |
| **Mass as spectral residue** | Conceptual work done | m = R/G_c from pole of modal Green's function (see notes below) |
| **Charge as phase winding** | Conceptual work done | Quantized winding number → Coulomb 1/r² (see notes below) |

### Neutrino Oscillations as Modal Beating (from prior work)

**Core insight:** Neutrino flavor oscillations are interference patterns from
a matrix-valued Green's function projection of an enriched modal flow.

**Already derived conceptually:**
- The flow in Hont is not scalar but vector-valued (C³ codomain for 3 flavors)
- The projection kernel is a 3x3 matrix-valued Green's function (the PMNS
  mixing matrix is a property of this kernel, not a separate postulate)
- Flavor oscillations = beating patterns between eigenmodes of the matrix kernel
- Mass differences between neutrino species arise from different spectral
  residues of the matrix-valued kernel's poles

**Kleis formalization plan:**
1. Define `MatrixKernel` structure extending `GreenKernel` with matrix-valued output
2. Define `FlavorState` as C³-valued flow
3. Prove: oscillation probability P(ν_e → ν_μ) follows from kernel eigenvalues
4. Derive: mass splittings from spectral residue differences
5. Show PMNS matrix emerges from kernel diagonalization

### Special Relativity from Projection Slicing (from prior work)

**Core insight:** SR is not a postulate — it is a *theorem* of observer-dependent
projection. Different observers correspond to different slicing kernels that
aggregate modal information from Hont into their respective spacetimes.

**Already derived conceptually:**
- Time dilation: arises from different observers slicing the same modal flow
  at different angles — a tilted slice aggregates more modal cycles per
  projected second
- Invariant spacetime interval: the interval ds² is invariant because it
  measures a property of the *flow itself*, not of any particular slice —
  all slicing kernels preserve it
- Lorentz transformations: the group of transformations that relate different
  slicing kernels while preserving the interval — emerges from the geometry
  of projection, not postulated

**The key reframing:**
- Standard SR: "The speed of light is constant" (postulate) → derive everything
- POT SR: "Observers are projection slices of modal flow" (structural) →
  constant c, time dilation, length contraction, and Lorentz invariance
  are all *theorems*

**Kleis formalization plan:**
1. Define `Observer` structure with slicing kernel and projection axis
2. Define `Interval(f, x1, x2)` as a flow-intrinsic quantity
3. Prove: interval is invariant under change of observer (change of slicing kernel)
4. Derive time dilation factor γ = 1/√(1 - v²/c²) as a projection geometry result
5. Prove Lorentz group emerges as the symmetry group of admissible slicing kernels

### Schwarzschild Metric & Gravitational Lensing Notes (from prior work)

**Schwarzschild weak-field (already derived conceptually):**
- Defined a "vortex" (phase singularity) in the modal phase of Hont
- Successfully derived the weak-field limit of the Schwarzschild metric
- Curvature is NOT an independent geometric property — it is a *projected
  artifact of modal shear*: when modal flow lines in Hont diverge or
  converge, their projection into R^4 produces curved geodesics
- The effective line element induced by modal flow causes deviated light paths
- Observer perceives deviations as spatial curvature and time dilation —
  standard GR characteristics, recovered as emergent phenomena from POT

**Einstein Cross ("four-piece-bagel" experiment):**
- Performed numerical calculations to project a gravitational lensing
  structure from Hont into R^4
- Initial result: "Einstein's four-piece-bagel" — recognizable but imperfect
- This was a *diagnostic signal*: the modal structure needs additional
  constraints (analyticity and band-limitation) to project into physically
  accurate forms
- These constraints are not ad hoc — they are the modal equivalent of
  regularity conditions on Green's functions

**Mass as source:**
- Localized distortions in the projected field require mass-like residues
  as ontological sources — connecting to Postulate 4 (Spectral Residues)
- This links gravitational lensing to the mass/charge emergence program

**Kleis formalization plan:**
1. Define `VortexMode` structure with phase singularity axioms
2. Derive weak-field Schwarzschild metric as a theorem of modal shear
3. Prove light deflection angle matches GR prediction (1.75 arcsec at solar limb)
4. Add analyticity/band-limitation axioms to `AdmissibleKernel`
5. Reproduce Einstein Cross geometry from constrained projection

### Entanglement Formalization Notes (from prior work)

**Core insight (Standing Wave Realism):** Entangled particles A and B are not
correlated separate systems — they are spatial manifestations of a *single
ontological standing wave* expressed at multiple spacetime coordinates in the
projected universe.

**Rejection of separability:** The standard framing assumes A and B are
independent entities that must be reconnected by some causal or probabilistic
bridge. POT denies this premise — A and B share a single flow `f`, and
`G[f]` evaluated at two locations is one field, not two correlated fields.

**Analogies (from author's notes):**
- *Violin string:* Two nodes on one string vibrate in harmony because they
  are part of the same physical mode, not because a signal travels between them.
- *Ocean swell:* Two valleys of the same swell at distant shores are connected
  in the body of the wave, not through point-to-point signaling.

**Sidesteps Bell's theorem:** Bell assumes distinct systems with local hidden
variables. POT has one system projected to two locations. The theorem's
separability premise does not hold, so its inequalities are irrelevant.

**Kleis formalization plan:**
1. Define `Entangled(G, f, x1, x2)` — two observations from the same flow
2. Prove: correlated outcomes are a consequence of kernel linearity (A1-A3)
3. Key theorem: "separability is NOT derivable from POT axioms" — Z3 should
   show that no axiom implies O_1 and O_2 are independent
4. Show Bell-type inequalities do not apply when separability is absent

**What this eliminates (no need for):**
- Magical observers collapsing reality (Copenhagen)
- Universe-branching at every measurement (Many-Worlds)
- Fine-tuned hidden variables (Superdeterminism)

### Spectral Residues: Mass and Charge as Projection Invariants (from prior work)

The `residue` operation is already in `pot_foundations_kernel_projection.kleis`:
```
axiom survival_principle: ∀(G : GreenKernel, a b : Flow, e : Event, c : Channel).
    apply_kernel(G, a) = apply_kernel(G, b) →
        residue(apply_kernel(G, a), e, c) = residue(apply_kernel(G, b), e, c)
```

**Point Mass as Spectral Residue (already derived conceptually):**
- Started from Poisson equation: ∇²Φ = 4πG_c ρ
- Used Divergence Theorem: flux of gravitational field through closed surface
  yields enclosed mass
- Defined Residue: R = -(1/4π) ∮_S g⃗ · dS⃗
- Derived strict equality: **m = R / G_c**
- Was one of the first concepts encoded in Kleis (v0.2) using symbolic operations
- Ontological axiom: "no sharp objects in Hont" — a point mass in R^4 is a
  collapsed residue of a smooth modal flow in Hilbert space

**Electric Charge as Phase Winding (already derived conceptually):**
- Every mode in Hont carries a complex phase (e^{iθ})
- Total charge = topological winding number of this phase evolution
- **Quantization explained:** winding number must be an integer → charge is quantized
- **Polarity explained:** direction of phase circulation → positive or negative charge
- **Coulomb's Law recovered:** if the projection kernel matches a Laplace Green's
  function in 3D for phase singularities, the projected field obeys 1/r² force law
- Explored modal coupling between opposite charges (stationary charge induces
  rotation in opposite charge) — shelved because it yielded a "free rotor"
  spectrum found unsatisfactory

**Summary — both are projection-surviving invariants:**
| Property | Mechanism in Hont | Mathematical form |
|----------|------------------|-------------------|
| **Mass** | Pole in modal Green's function | Spectral residue R/G_c |
| **Charge** | Phase singularity winding | Topological winding number |

Both survive the "structure-losing" projection from Hont into R^4 because they
are topological/spectral invariants — robust enough to persist through projection.

**Kleis formalization plan:**
1. Define `SpectralResidue` structure with pole/residue axioms
2. Define `PhaseWinding` structure with integer-valued winding number
3. Prove m = R/G_c as a theorem of the residue axioms
4. Prove charge quantization as a theorem of winding number integrality
5. Derive Coulomb 1/r² from Laplace Green's function projection
6. Connect to existing `Event`, `Channel`, `Residue` primitives

### BUG STATUS: Z3 crash — RESOLVED

**Root cause was NOT Z3.** It was UTF-8 string slicing in `server.rs`.
The previous `vendor/z3/src/func_decl.rs` changes (`try_apply`) are still
good defensive code but were not the crash source.

**Workaround no longer needed.** Axioms with nonlinear real arithmetic
(multiplication of operations) work fine. Z3 may return "Unknown" for
complex nonlinear queries but does not crash.

**Previous note (obsolete):** Avoid `sqrt`, `log`, `sin`, `cos`, `exp` in axioms. Use
algebraic equivalents (e.g., `v_squared = M/r` instead of `v = sqrt(M/r)`).
Keep transcendentals as abstract uninterpreted operations with axiomatized
properties — don't ask Z3 to evaluate them directly.

**Root cause:** The crash is in Z3's C library (libz3), not Rust code.
`catch_unwind` cannot catch it. It occurs when multiple structures with
nonlinear axioms interact — e.g., `v_squared * r = projected_mass` combined
with `projected_mass = mass_rate * r` creates a system Z3's C backend aborts on.
A single structure with `mass_rate * r` works fine on its own.

**Fix needed:** Either pre-validate axiom complexity before sending to Z3,
or run Z3 verification in a subprocess that can be killed without crashing
the MCP server. Also improve `func_decl.rs` to not unwrap None (done: added
`try_apply`), though the C-level crash bypasses this.

### Session Startup

1. Read this file
2. `scripts/theory.sh on` (enable kleis-theory MCP)
3. `load_theory(imports: ["theories/pot_admissible_kernels_v2.kleis"])`
4. Read `examples/ontology/revised/` for existing POT formalizations
5. Pick a result from the table above and start formalizing

---

## 💡 IDEA: Paper Review Rules as Kleis Policy

**Origin:** The POT arXiv paper went through ~6 rounds of peer review (by the author), each catching substantive issues. The review process surfaced implicit quality rules that could be formalized as Kleis policies — enabling Z3-backed verification of scientific papers.

**Rules that emerged from the review:**

1. **Dimensional Consistency** — Every axiom must be scale-free. No magic constants (e.g., `r > 1`) that depend on unit choice.
2. **Physical Honesty** — If a plot contradicts intuition, the paper must explain why. Don't hide assumptions (e.g., uniform-density core).
3. **Ontological Precision** — Distinguish measured quantities (baryonic mass) from computed quantities (projected mass). Never conflate the two.
4. **Concrete Grounding** — Abstract axioms are necessary but not sufficient. Numerical results must trace to an explicit kernel/function.
5. **Counter-theory Acknowledgment** — If replacing Theory X, acknowledge whether the dominant theory (e.g., GR) also fails in the same regime.
6. **Presentation as Rigor** — Formatting errors are conceptual errors in disguise. No tooling artifacts in output.
7. **Intellectual Sovereignty** — Don't bind to labels ("open source") that constrain future decisions.

**Potential formalization:** These are axiomatizable as structural checks on a document AST. Z3 could verify, e.g., that every axiom used in a numerical section has a concrete instantiation, or that every claim about Theory X references a counter-theory. A `paper_review_policy.kleis` could enforce these during paper generation.

---

## PREVIOUS: Equation Editor Enhancements (Jan 3-4, 2026)

### Branch: `feature/copy-typst-button`

**Status:** ✅ Implemented, ready to merge

### What Was Built (Jan 4)

| Feature | Status |
|---------|--------|
| **"📋 Copy Typst" button** | ✅ |
| `/api/export_typst` endpoint | ✅ |
| PhD candidate workflow: visual → copy → paste into thesis | ✅ |

**The workflow:**
1. Build equation visually in Equation Editor (WYSIWYG, Typst-rendered)
2. Click "📋 Copy Typst"
3. Paste into `thesis.kleis` document
4. Compile to PDF

No Typst syntax knowledge needed — the editor generates it!

### Previous Work (Jan 3): Jupyter Integration

| Feature | Status |
|---------|--------|
| Jupyter mode detection (`?mode=jupyter`) | ✅ |
| "📤 Send to Jupyter" button | ✅ |
| Auto-switch to structural mode | ✅ |
| `kleisInitialData` listener (for re-editing) | ✅ |
| `add_equation_from_ast()` helper | ✅ |

### ⏳ BLOCKERS: Before Documenting in Manual

The Equation Editor works but has two blockers before adding a manual chapter:

| Blocker | Issue | Solution Ideas |
|---------|-------|----------------|
| **Server dependency** | Requires `kleis server` running separately | Auto-start? WASM Typst? |
| **Jupyter display** | Need to embed editor inline in notebook cells | IFrame widget? Full integration? |

**Why these matter:**
- Users shouldn't need to manually start a server
- Jupyter is becoming the default interface — editor should live there
- Documentation should show a seamless workflow, not "first start server in terminal..."

**See also:** Section 9 (Jupyter + Equation Editor Integration) for architecture options

---

## 📦 Previous Work: University Templates Complete (Jan 3, 2026)

### Branch: `main`

### What Was Built

| Template | Status | Example Document |
|----------|--------|------------------|
| **MIT Thesis** (`stdlib/templates/mit_thesis.kleis`) | ✅ Complete | `examples/documents/jane_smith_thesis.kleis` |
| **UofM Rackham** (`stdlib/templates/uofm_thesis.kleis`) | ✅ Complete | `examples/documents/alex_chen_dissertation.kleis` |
| **arXiv Preprint** (`stdlib/templates/arxiv_paper.kleis`) | ✅ Complete | `examples/documents/sample_arxiv_paper.kleis` |

### MIT Thesis Features
- Title page, signature page, abstract
- Acknowledgments, dedication
- Table of Contents, List of Figures, List of Tables
- Chapters, sections, subsections
- Equations (numbered), figures, tables, diagrams (Lilaq)
- Appendices, bibliography
- Bachelor/Master/Doctor degree types

### UofM Rackham Features  
- Rackham-compliant formatting (1" margins, Times 12pt, double-spaced)
- Title page with full committee listing
- Identifier/Copyright page with ORCID
- Dedication, acknowledgments, preface
- TOC, List of Tables, List of Figures, List of Appendices
- Abstract (required)
- Chapters with 2" top margin
- Appendices, bibliography
- PhD/DMA/DNP/DrPH/EdD/DArch degree types
- Roman numerals for front matter, Arabic for body

### arXiv Preprint Features
- Based on [arxiv-style](https://github.com/kourgeorge/arxiv-style) (NeurIPS aesthetic)
- Single-column, 11pt New Computer Modern font
- Multiple authors with numbered affiliations
- Abstract with optional keywords
- Sections, subsections, subsubsections
- Equations, figures, tables, diagrams (Lilaq)
- Algorithms with pseudocode
- Optional acknowledgments section
- References section
- Optional appendix
- Preprint header on each page

### ✅ Inline Math Works!

Inline math in text works using Typst's `$...$` syntax:

```kleis
MITSection("Methods", "A function $f: RR^n -> RR^m$ maps inputs to outputs.")
```

For literal dollar signs, use `\$`:
```kleis
"The cost is \$100 per unit."
```

No parser changes needed — Typst handles it natively.

---

## 📦 Previous Work: Document System Cleanup (Jan 3, 2026)

### What Changed

**DELETED the old Python-heavy `KleisDoc` class** (3000+ lines) in favor of:
- Pure Kleis documents (`.kleis` files)
- Templates in `stdlib/templates/`
- Thin Python shell (`kleisdoc_shell.py`, 227 lines)

### Old Design Docs (DELETED)

The `docs/jupyter-equation-editor-poc/` directory was deleted (Jan 4, 2026).
It contained early POC documents that were superseded by the current implementation.

**Current documentation:** See [Chapter 23: Document Generation](https://kleis.io/docs/manual/book/chapters/23-document-generation.html)

### Current Examples

| File | Purpose |
|------|---------|
| [`jane_smith_thesis.kleis`](../examples/documents/jane_smith_thesis.kleis) | MIT PhD thesis example |
| [`alex_chen_dissertation.kleis`](../examples/documents/alex_chen_dissertation.kleis) | UofM Rackham dissertation |
| [`sample_arxiv_paper.kleis`](../examples/documents/sample_arxiv_paper.kleis) | arXiv paper example |
| [`lisp_parser.kleis`](../examples/meta-programming/lisp_parser.kleis) | LISP interpreter (proof of Document = Program) |

### Key Architecture (Final)

1. **Documents are Kleis Programs** - Not Python objects
2. **Templates are Kleis Files** - `stdlib/templates/*.kleis`
3. **Python is a Thin Shell** - Just calls `kleis test` and `typst compile`
4. **No EditorNode in Documents** - Equations are Typst strings (simpler)
5. **Compile via Example Blocks** - `example "compile" { out(compile_mit_thesis(thesis)) }`

---

## Previous: Jupyter + Equation Editor POC (Jan 2, 2026)

*(POC documentation deleted Jan 4, 2026 - key findings preserved below)*

### Summary

1. **Iframe Embedding POC** ✅
   - Tested embedding HTML widgets in Jupyter via iframe
   - Three methods work: Direct IFrame, Toggle Button, Message Passing

2. **Export Typst Functions** ✅
   - `export_typst(...)` - Returns complete Typst code with preamble
   - `export_typst_fragment(...)` - Returns just `lq.diagram()` for embedding
   - Files: `examples/export/export_typst_demo.kleis`

3. **PDF Workflow**
   ```
   Kleis → export_typst() → paper.typ → typst compile → paper.pdf
   ```

4. **LaTeX/arXiv Finding**
   - Typst does NOT generate LaTeX (it's a replacement, not a frontend)
   - arXiv accepts PDF directly
   - Future: Could add `export_latex()` using PGFPlots

### Future Enhancements (Not Yet Implemented)
- `save_typst(filename, ...)` - Write to file directly
- `compile_pdf(filename, ...)` - Compile to PDF in one step
- Real Equation Editor integration with `?mode=jupyter`
- `export_latex()` for PGFPlots output

---

## ✅ DONE: `and`/`or`/`not` as General Logical Operators (Grammar v0.97)

**Added:** January 5, 2026  
**Implemented:** January 5, 2026

### What Was Implemented

`and`, `or`, and `not` now work as general operators in all contexts:

```kleis
// Both work identically:
assert(P and Q)   // ASCII
assert(P ∧ Q)     // Unicode

structure DeMorgan {
    axiom law1: forall P : Bool . forall Q : Bool .
        not (P and Q) = (not P) or (not Q)
}

example "test" {
    assert(True and False = False)
    assert(True or False = True)
    assert(not False = True)
}
```

### Files Changed

| File | Change |
|------|--------|
| `vscode-kleis/docs/grammar/kleis_grammar_v097.ebnf` | New grammar version |
| `vscode-kleis/docs/grammar/kleis_grammar_v097.md` | Documentation |
| `src/kleis_parser.rs` | Updated `parse_conjunction`, `parse_disjunction`, `parse_primary` |
| `tests/grammar_v097_and_or_test.rs` | 20 Rust tests |
| `tests/test_and_or_operators.kleis` | Kleis test file |
| `tests/test_operators.kleis` | Updated with v0.97 operators |

### Branch

`grammar-v097-and-or`

---

## 🔧 FUTURE: Set Operators as Infix Syntax (Grammar v0.97)

**Added:** January 5, 2026

### Current State

Set operators require function-call syntax:
```kleis
in_set(x, S)        // instead of x ∈ S
subset(A, B)        // instead of A ⊆ B
proper_subset(A, B) // instead of A ⊂ B
```

### Proposed Enhancement

Add infix operators to the grammar:

```ebnf
binaryOp ::= ... existing operators ...
           | "∈" | "∉" | "⊆" | "⊂"  // NEW: Set operators
```

### Implementation

1. **Grammar v0.97**: Add set operators to `binaryOp` production
2. **Parser**: Add to `try_parse_infix_operator()`:
   ```rust
   '∈' => Some("in_set".to_string()),
   '∉' => Some("not_in_set".to_string()),
   '⊆' => Some("subset".to_string()),
   '⊂' => Some("proper_subset".to_string()),
   ```
3. **Precedence**: Same as comparison operators (level 6)
4. **Tests**: Add to `tests/test_operators.kleis`
5. **Documentation**: Update `grammar.md`

### Why This Was Removed

Set operators existed in grammar v03-v08 but were removed. Possibly:
- Z3 set theory was added later than the grammar
- Function-call syntax was simpler for initial implementation
- No pressing need at the time

### Effort Estimate

~1 hour: Simple parser addition, well-defined semantics, existing function implementations.

---

## 🔧 FUTURE: User-Implementable Unicode Operators

**Added:** January 7, 2026

### Current Limitation

Unicode operators like `•`, `⊗`, `⊕`, `∘` are **syntactic only**:
- They parse as infix: `a • b` → `•(a, b)`
- But they **cannot be computed** — they stay symbolic forever
- Users cannot define implementations for them

### Why Users Can't Implement Them

| Approach | Result |
|----------|--------|
| `define •(a, b) = a * b` | ❌ Parse error — `•` not a valid identifier |
| `operation • : T × T → T` in structure | ❌ Parse error — same reason |
| Define `dot` and hope `•` uses it | ❌ No connection — `•` stays symbolic |

### Proposed Solutions

**Option 1: Add Built-in Aliases**

Add common operators to `evaluator.rs`:
```rust
"•" | "dot" | "inner" => self.builtin_dot_product(args),
"∘" | "compose" => self.builtin_compose(args),
"⊗" | "tensor" => self.builtin_tensor_product(args),
```

**Option 2: Operator Mapping in Structures**

Allow structures to map operators to implementations:
```kleis
structure VectorSpace(V) {
    operation dot : V × V → ℝ
    infix • = dot   // NEW: operator alias
}
```

**Option 3: Parser-Level Rewriting**

Make parser rewrite `a • b` → `dot(a, b)` based on registered mappings.

### Current Documentation

The operators appendix now correctly states these limitations. See:
`docs/manual/src/appendix/operators.md` — "Custom Mathematical Operators" section.

### Effort Estimate

- Option 1: ~2 hours (add builtins, implement semantics)
- Option 2: ~4 hours (parser + evaluator changes)
- Option 3: ~6 hours (complex parser rewriting)

---

## Previous: Unified Plotting API Complete! (Jan 1, 2026)

### What's Done

- ✅ **Unified `graph()` API** - ONE function for all 14 plot types
- ✅ **Legacy functions removed** (`plot`, `scatter`, `bar`, etc. - all gone)
- ✅ **Clean namespace** - Only `graph("type", ...)` remains
- ✅ **Grouped bars with errors** - `graph("grouped_bars", xs, series, labels, errors)`
- ✅ **Jupyter kernel updated** - Banner and completions reflect new API
- ✅ **Examples updated** - `basic_plots.kleis` uses only `graph()`
- ✅ **20 examples passing**

### The Unified API

```kleis
// ONE function for everything:
graph("line", xs, ys)
graph("scatter", xs, ys)
graph("bar", xs, heights)
graph("heatmap", matrix)
graph("grouped_bars", xs, [series1, series2], ["Label1", "Label2"])

// Types: line, scatter, bar, hbar, stem, hstem, fill_between,
//        boxplot, hboxplot, heatmap, contour, quiver, grouped_bars
```

### Known Limitations

#### 1. Calculated Expressions in Plots

**What WORKS:**
```kleis
// Arithmetic in lists
graph("line", [0, 1, 2, 3], [0, 1*1, 2*2, 3*3])  // ✅

// Let bindings
let xs = [0, 1, 2, 3] in
let ys = [0, 1, 4, 9] in
graph("line", xs, ys)  // ✅

// negate()
graph("line", x, [0, negate(1), negate(2)])  // ✅
```

**What DOESN'T work (yet):**
```kleis
// No list comprehensions
graph("line", x, [x*x for x in xs])  // ❌ No syntax

// No map over lists  
graph("line", x, map(square, xs))  // ❌ Not implemented

// No linspace/arange
graph("line", linspace(0, 10, 100), ...)  // ❌ Future phase
```

**Root cause:** Lists must be written explicitly. We lack programmatic list generation.

#### 2. Jupyter Kernel: KLEIS_ROOT Environment Variable ✅ RESOLVED

**Problem:** When Jupyter runs from arbitrary directories, `import "stdlib/prelude.kleis"` fails.

**Solution:** The `KLEIS_ROOT` environment variable is now supported in all import resolution:
```bash
export KLEIS_ROOT=/path/to/kleis
```

**Implementation (Jan 2, 2026):**
1. ✅ All `resolve_import_path` functions check `KLEIS_ROOT` first
2. ✅ Documented in manual Chapter 21 (Jupyter Notebook) Troubleshooting section
3. ✅ Kernel already checks KLEIS_ROOT (was implemented previously)

**Files updated:** `src/bin/kleis.rs`, `src/main.rs`, `src/repl.rs`, `src/context.rs`, 
`src/lsp/server.rs`, `src/bin/lsp.rs`, `src/bin/server.rs`

#### 3. Plotting API Design ✅ RESOLVED

**Previous approach:** Many separate functions (plot, scatter, bar, etc.)

**New approach:** Single unified function
```kleis
graph("line", xs, ys)
graph("scatter", xs, ys)
graph("bar", xs, heights)
graph("heatmap", matrix)
```

**Why unified is better:**
- Simpler API surface (one function to learn)
- Easier to switch between plot types  
- Clean namespace (no clutter)
- Options dict supported: `graph("line", xs, ys, { title: "Plot" })`

**Implemented January 1, 2026.** Legacy functions removed, all 20 examples converted.

#### 4. Manual: Symbolic Differentiation Example is Weak

**Location:** `docs/manual/src/chapters/13-applications.md`

**Problem:** The current example just shows code without:
- Naming the calculus rules (Power Rule, Product Rule, Chain Rule, etc.)
- Referencing Calculus 1 and Calculus 2 curriculum
- Using axioms to STATE the rules
- Verifying the implementation against the axioms
- Showing the Quotient Rule, Inverse Trig, etc.

**Should be rewritten as:**
```kleis
structure DifferentiationRules {
    // Calculus 1
    axiom constant_rule: ∀(c : ℝ). diff(Const(c), x) = Const(0)
    axiom power_rule: ∀(n : ℝ). diff(Pow(x, n), x) = Mul(n, Pow(x, n - 1))
    axiom sum_rule: diff(Add(f, g), x) = Add(diff(f, x), diff(g, x))
    axiom product_rule: diff(Mul(f, g), x) = Add(Mul(diff(f, x), g), Mul(f, diff(g, x)))
    axiom quotient_rule: diff(Div(f, g), x) = ...
    axiom chain_rule: diff(Compose(f, g), x) = Mul(diff(f, g(x)), diff(g, x))
    
    // Calculus 2
    axiom exp_rule: diff(Exp(f), x) = Mul(Exp(f), diff(f, x))
    axiom ln_rule: diff(Ln(f), x) = Div(diff(f, x), f)
    axiom sin_rule: diff(Sin(x), x) = Cos(x)
    axiom cos_rule: diff(Cos(x), x) = Neg(Sin(x))
}
```

Then implement `diff` and VERIFY it satisfies the axioms. That's the Kleis way.

#### 5. Manual: Incorrect Command Path

**Problem:** The manual says:
```
Checker    ./scripts/kleis check    Validate .kleis files
```

**Should say:**
```
Checker    kleis check    Validate .kleis files
```

`./scripts/kleis` is a developer wrapper script (sets up Z3 env). End users would just run `kleis check`.

#### 6. Jupyter: Render Expressions as SVG (Beautiful Math Display)

**Insight:** We already have all the pieces to render Kleis expressions as beautifully typeset math in Jupyter:

```
Expression (Kleis AST)
     ↓ translate_to_editor()
EditorNode
     ↓ render_editor_node(Typst)
Typst code
     ↓ typst compile
SVG
     ↓ Jupyter display_data
Beautiful rendered equation! 📐
```

**What this enables:**
- `∀(x : ℝ). x + 0 = x` → rendered as proper math notation (not text)
- Verification results: theorem + "✓ Verified by Z3" as formatted equations
- Step-by-step derivations rendered beautifully
- Jupyter notebooks look like LaTeX papers

**Implementation:**
1. Add `render(expr)` function that outputs `EXPR_SVG:<svg>` to stdout
2. Kernel detects `EXPR_SVG:` and displays as SVG (like plots)
3. Could also support `render(expr, "latex")` for LaTeX output

**This is "executable mathematics" — compute AND display beautifully.**

#### 7. Long-Term Vision: Executable Papers

**The big picture:** Kleis documents that export to PDF and arXiv.

```
┌─────────────────────────────────────────────────────────────┐
│                    Kleis Document                            │
├─────────────────────────────────────────────────────────────┤
│  Text         │ Abstract, introduction, prose               │
│  Formulas     │ Theorems, definitions (verified by Z3)      │
│  Plots        │ Visualizations (Lilaq → SVG)                │
│  Tables       │ Data, results                               │
│  Code         │ Examples, computations                       │
│  Proofs       │ Step-by-step, machine-checked               │
└─────────────────────────────────────────────────────────────┘
                          ↓
              ┌───────────┴───────────┐
              ↓                       ↓
         Typst → PDF              LaTeX → arXiv
```

**Aspirational syntax:**
```kleis
document "My Paper" {
    section "Introduction" {
        text "We prove the following result..."
    }
    
    section "Main Theorem" {
        theorem "Commutativity" {
            statement: ∀(a b : ℝ). a + b = b + a
            proof: verified  // Z3 checked!
        }
        
        example "Numerical demonstration" {
            graph("line", [0,1,2], [2,1,0], "a + b = b + a")
        }
    }
}

export("paper.pdf")
export("arxiv/", format: "latex")
```

**What this enables:**
- Write once → PDF, arXiv, Jupyter, HTML
- Every theorem is verified by Z3
- Every plot is computed from data
- No copy-paste errors between code and paper
- Reproducible science by design

**Components needed:**
1. ✅ Expression → EditorNode → Typst (exists)
2. ✅ Plotting → SVG (exists)
3. ⏳ Text + math mixed rendering
4. ⏳ Document structure (sections, theorems)
5. ⏳ Tables
6. ⏳ Export to PDF (Typst does this)
7. ⏳ Export to LaTeX (for arXiv)

**This is what scientists actually need.**

#### 8. Kleis AST → EditorNode Translator: Limited for 2D Math Rendering

**Problem:** The `translate_to_editor()` function in `src/editor_ast.rs` is too basic for proper 2D mathematical rendering. It only handles tensors with index structure but doesn't recognize common mathematical operations that need special visual layout.

**Current limitations:**

| Kleis AST | Current Translation | Should Be |
|-----------|--------------------| -----------|
| `a / b` | `Operation {name: "div", args: [a, b]}` | `kind: "frac"` for 2D layout |
| `sqrt(x + y)` | `Operation {name: "sqrt", args: [...]}` | `kind: "sqrt"` with radical bar |
| `sum(i, 1, n, f)` | Basic operation | `kind: "sum"` with stacked bounds |
| `c / (a + b)` | Keeps parentheses in args | Frac bar makes parens implicit |

**What's needed:**

1. **Operation recognition** - Map `div` → fraction, `sqrt` → radical, etc.
2. **Parenthesis elimination** - Know when 2D layout makes parens unnecessary
3. **Big operator templates** - `sum`, `prod`, `int` need stacked bound notation

**Enhancement approach:**

```rust
// In translate_with_context():
"div" | "/" => EditorNode::operation_with_kind("frac", args, "frac"),
"sqrt" => EditorNode::operation_with_kind("sqrt", args, "sqrt"),
"sum" | "Σ" => /* create stacked bounds */ ,
```

**Files:**
- `src/editor_ast.rs` - `translate_to_editor()` and `translate_with_context()` (lines 194-310)

**Impact:** Enables beautiful math rendering in Jupyter via the existing pipeline:
```
Expression → translate_to_editor() → EditorNode → render(Typst) → SVG
```

**Priority:** Medium (depends on whether Jupyter math rendering is pursued)

#### 9. Jupyter + Equation Editor Integration: Publication-Quality Equations

**Objective:** Scientists using Jupyter notebooks need beautiful, publication-quality rendered equations. The Equation Editor already provides this capability — the challenge is bringing it INTO Jupyter.

**What the Equation Editor provides:**
- Visual, template-based equation building (no LaTeX syntax to memorize)
- Beautiful SVG output via Typst rendering
- Export to LaTeX (for journal submission), Kleis (for Z3 verification)

**The integration challenge:**

```
┌─────────────────┐          ┌─────────────────┐
│  Jupyter        │    ???   │  Equation       │
│  Notebook       │ ←──────→ │  Editor         │
│  (Python kernel)│          │  (Rust server)  │
└─────────────────┘          └─────────────────┘
```

The Equation Editor is a web app (`static/index.html` or `patternfly-editor/`) that requires `kleis server` running. Jupyter has its own Python kernel. These are separate processes.

**Key insight:** The deliverable is the **rendered SVG**, not Kleis code. Scientists want:
```
      ∂ψ          ℏ²  ∂²ψ
  iℏ ─── = -  ─── ───── + V(x)ψ
      ∂t          2m  ∂x²
```
Not:
```
scalar_divide(diff(psi, t), scalar_multiply(i, hbar))
```

**Possible integration approaches:**

| Approach | Pros | Cons |
|----------|------|------|
| **User runs `kleis server` separately** | Simple, works now | Manual step, two processes |
| **Kernel spawns server as subprocess** | Automatic | Port conflicts, lifecycle management |
| **Bundle kleis binary with Python package** | Clean install | Large binary, platform-specific builds |
| **ipywidgets custom widget** | Native Jupyter feel | Requires server running, complex widget dev |
| **WebAssembly (Typst in browser)** | No server needed! | Significant engineering effort |

**The WASM option is particularly interesting:**
- If Typst rendering runs in the browser (via WASM), the entire Equation Editor becomes client-side
- No `kleis server` dependency
- Works in any Jupyter environment (JupyterHub, Colab, etc.)
- Typst already has WASM support: https://github.com/typst/typst

**HOWEVER: User-Extensible Templates Require Filesystem Access**

The Equation Editor is designed to be **user-extensible** via `.kleist` template files:

```kleist
// std_template_lib/chemistry.kleist (user-defined!)
@template reaction_arrow {
    pattern: "reaction_arrow(reactants, products)"
    unicode: "{reactants} → {products}"
    latex: "{reactants} \\rightarrow {products}"
    typst: "{reactants} arrow {products}"
    category: "chemistry"
}
```

Standard template library: `std_template_lib/*.kleist` (12 files: basic, calculus, tensors, quantum, etc.)

**This means:**
- Templates must be loaded from filesystem
- Pure browser WASM alone won't work for custom templates
- Need some backend or filesystem API to read `.kleist` files
- Users expect to add domain-specific notation (chemistry, biology, music, etc.)

**Possible solutions:**
1. **Hybrid:** WASM for rendering + HTTP endpoint for template loading
2. **Bundle standard templates** in WASM, custom templates via server
3. **Jupyter kernel** reads templates, sends to frontend
4. **File upload** mechanism for custom templates in browser

**Components to study:**
- `patternfly-editor/` — React app, could become ipywidget frontend
- `src/bin/server.rs` — HTTP API endpoints for rendering
- `src/typst_renderer.rs` — The Typst code generation
- `src/render_editor.rs` — Template-based rendering

**Files:**
- `static/index.html` — Original Equation Editor
- `patternfly-editor/` — PatternFly/React version
- `kleis-notebook/` — Current Jupyter kernel
- `std_template_lib/*.kleist` — Standard template library (12 files)

**Priority:** High for publication use case, but significant engineering effort

#### 10. Fun Project: 4-Voice Counterpoint Verification

**Idea:** Implement Bach-style counterpoint rules as Kleis axioms. This would demonstrate Kleis's generality beyond pure mathematics.

**The classic rules (Fux's species counterpoint):**

```kleis
structure FourVoiceCounterpoint {
    data Voice = Soprano | Alto | Tenor | Bass
    data Interval = Unison | Third | Fifth | Sixth | Octave | ...
    data Motion = Parallel | Contrary | Oblique | Similar
    
    // No parallel fifths or octaves
    axiom no_parallel_fifths: ∀(v1 v2 : Voice, t : Time).
        interval(v1, v2, t) = Fifth ∧ interval(v1, v2, t+1) = Fifth ∧
        motion(v1, v2, t) = Parallel → violation
    
    axiom no_parallel_octaves: ∀(v1 v2 : Voice, t : Time).
        interval(v1, v2, t) = Octave ∧ interval(v1, v2, t+1) = Octave ∧
        motion(v1, v2, t) = Parallel → violation
    
    // Voice ranges
    axiom soprano_range: ∀(t : Time). C4 ≤ pitch(Soprano, t) ≤ G5
    axiom bass_range: ∀(t : Time). E2 ≤ pitch(Bass, t) ≤ C4
    
    // No voice crossing
    axiom no_crossing: ∀(t : Time). 
        pitch(Bass, t) < pitch(Tenor, t) ∧
        pitch(Tenor, t) < pitch(Alto, t) ∧
        pitch(Alto, t) < pitch(Soprano, t)
    
    // Dissonances must resolve
    axiom dissonance_resolution: ∀(v : Voice, t : Time).
        is_dissonance(v, t) → resolves_by_step(v, t+1)
}
```

**Use case:** Music theory students submit exercises, Z3 verifies all rules instantly.

**Why this matters:** Shows Kleis is a **general verification platform**, not just a "math tool". Any domain with formalizable rules can use the same machinery.

**Files to create:**
- `examples/music/counterpoint_rules.kleis` — Axiom definitions
- `examples/music/bach_example.kleis` — Verify a Bach chorale
- `std_template_lib/music.kleist` — Musical notation templates (♩, ♪, ♯, ♭)

**Priority:** Fun project, educational value, great demo

#### 11. Future Documentation: Equation Editor & .kleist Templates

**Status:** NOT YET DOCUMENTED in the manual. Users don't know the Equation Editor exists.

**The plan:**
When we integrate the Equation Editor with Jupyter notebooks, we will document:

1. **The Equation Editor itself** - What it is, how it works, visual editing
2. **`.kleist` template system** - How templates define notation
3. **User extensibility** - How users can add custom templates (chemistry, music, etc.)
4. **Template anatomy** - Pattern, rendering for multiple targets, categories

**Documentation approach (lesson learned Jan 2, 2026):**
Just like we documented the Solver Abstraction Layer by **reading the code first**, we will:
- Read `static/index.html`, `patternfly-editor/`, `std_template_lib/*.kleist`
- Understand the actual implementation
- Then document what exists

**Why wait for Jupyter integration?**
- Equation Editor as standalone web app is less compelling to document
- Integration with Jupyter gives it a clear use case (publication-quality equations in notebooks)
- The documentation will show the complete workflow

**Files to document when ready:**
- `static/index.html` — Original Equation Editor
- `patternfly-editor/` — PatternFly/React version
- `std_template_lib/*.kleist` — Standard template library (12 files)
- `src/typst_renderer.rs` — Template-based rendering
- `src/render_editor.rs` — EditorNode rendering

**See also:** Section 9 (Jupyter + Equation Editor Integration)

---

## ✅ DONE: Inline Math in Document Text (Jan 4, 2026)

**It already works!** Typst's `$...$` syntax passes through Kleis strings correctly.

### The Solution

Just use Typst inline math syntax in your strings:

```kleis
// ✅ WORKS - inline math rendered properly
define sec = UMichSection("Background",
    "A neural network is a function $f: RR^n -> RR^m$ that maps inputs to outputs.")

// ✅ Literal dollar sign - use backslash
define cost = MITSection("Cost", "The price is \\$100 per unit.")
```

### Why It Works

Kleis strings pass through to Typst unchanged. Typst interprets `$...$` as math mode.

No parser changes, no evaluator changes, no special handling needed.

### Syntax Reference

| What You Want | Syntax | Result |
|---------------|--------|--------|
| Inline math | `$f: RR^n -> RR^m$` | *f*: ℝⁿ → ℝᵐ |
| Block math | `$ E = m c^2 $` (spaces) | Centered equation |
| Literal $ | `\\$100` | $100 |

**Tested and verified: Jan 4, 2026**

---

## 🔧 FUTURE: Externalize Configuration (Ports, Timeouts)

**Added:** January 5, 2026

### Current State

Several configuration values are hardcoded in Rust:

| Setting | Current Value | Location |
|---------|---------------|----------|
| Z3 solver timeout | 30 seconds | `src/solvers/z3/backend.rs` |
| LSP server port | stdio | `src/bin/kleis.rs` |
| DAP server port | dynamic | `src/bin/kleis.rs` |
| Equation Editor server port | 3000 | `src/bin/server.rs` |

### Proposed Solution

1. **Configuration file** (e.g., `kleis.toml` or `.kleisrc`):
   ```toml
   [solver]
   backend = "z3"           # future: "cvc5", "lean", etc.
   timeout_seconds = 30
   
   [server]
   port = 3000
   
   [lsp]
   trace = "off"            # "off", "messages", "verbose"
   ```

2. **Environment variable overrides**:
   ```bash
   KLEIS_Z3_TIMEOUT=60 kleis test file.kleis
   KLEIS_SERVER_PORT=8080 kleis server
   ```

3. **Command-line flags**:
   ```bash
   kleis test --timeout 60 file.kleis
   kleis server --port 8080
   ```

### Why This Matters

- **Z3 timeout**: Some proofs need more time; users can't adjust
- **Ports**: Docker/Kubernetes deployments may require specific ports
- **Future solvers**: When adding CVC5, Lean, etc., need backend selection
- **Development vs Production**: Different settings for different environments

### Implementation Plan

1. Add `kleis.toml` parser (use `toml` crate)
2. Check env vars with `std::env::var()`
3. CLI flags via `clap` (already used)
4. Priority: CLI > env > config file > defaults

### Effort Estimate

~2-3 hours for basic implementation.

---

## 🔧 FUTURE: Code Organization & Technical Debt

### Overview

The codebase has grown significantly and needs modularization. Key issues:

1. **`evaluator.rs` is 9,325 lines** — needs splitting into modules
2. **Hardcoded types in Rust** — violates ADR-016 (types should come from stdlib)
3. **57 TODOs/FIXMEs across src/** — need systematic resolution
4. **Deprecated/duplicate code** — `src/dap.rs` marked deprecated

### Priority 1: Modularize `evaluator.rs`

**Current:** Single 9,325-line file with 178 methods

**Proposed structure:**
```
src/evaluator/
├── mod.rs           # Re-exports, Evaluator struct (~200 lines)
├── core.rs          # Loading, bindings, basic operations (~400 lines)
├── eval.rs          # Main evaluation logic (~400 lines)
├── substitution.rs  # substitute(), pattern matching (~150 lines)
├── lambda.rs        # β-reduction, α-conversion, free vars (~800 lines)
├── examples.rs      # Example blocks, assert, Z3 verification (~400 lines)
├── concrete.rs      # Concrete evaluation (~200 lines)
└── builtins/
    ├── mod.rs       # apply_builtin dispatch (~200 lines)
    ├── arithmetic.rs # +, -, *, /, pow, etc. (~600 lines)
    ├── string.rs    # String operations (~300 lines)
    ├── list.rs      # List operations (~400 lines)
    ├── plotting.rs  # diagram, plot, bar, etc. (~1500 lines!)
    ├── typst.rs     # export_typst, render_to_typst (~600 lines)
    └── matrix.rs    # Matrix operations (~400 lines)
```

**Benefit:** No file over 1500 lines. Clear separation of concerns.

### Priority 2: Remove Hardcoded Types (ADR-016)

| Hardcoded in Rust | Should Be in stdlib |
|-------------------|---------------------|
| `Type::matrix()`, `Type::pmatrix()`, etc. | `stdlib/types.kleis` |
| `"Scalar"`, `"Vector"`, `"Complex"` literals | Data registry lookups |
| Matrix dimension checking in Rust | Structure axioms in Kleis |

**Files affected:**
- `src/type_inference.rs` (35 occurrences of Scalar/Matrix/Vector)
- `src/type_context.rs` (8 occurrences)

**Target:** Type inference queries registry, doesn't hardcode type names.

### Priority 3: Clean Up TODOs

| File | TODOs | Notable Issues |
|------|-------|----------------|
| `src/math_layout/mod.rs` | 11 | Layout system incomplete |
| `src/render.rs` | 8 | Rendering edge cases |
| `src/type_inference.rs` | 7 | ADR-016 migration notes |
| `src/math_layout/typst_adapter.rs` | 7 | Typst integration |
| `src/bin/server.rs` | 4 | Server cleanup |

**Total:** 57 TODOs across 19 files

### Priority 4: Remove Deprecated Code

| File | Status | Action |
|------|--------|--------|
| `src/dap.rs` | Marked `#[deprecated]` | Delete after confirming `kleis server` works |
| `src/bin/debug.rs` vs `src/bin/commands/debug.rs` | Duplicate? | Consolidate |

### Estimated Effort

| Task | Sessions |
|------|----------|
| Modularize evaluator.rs | 2-3 |
| Remove hardcoded types | 1-2 |
| Clean up TODOs | 1-2 |
| Remove deprecated code | 0.5 |
| **Total** | **5-8 sessions** |

### Related ADRs

- **ADR-016:** Operations in Structures (types from stdlib, not Rust)
- **ADR-014:** Hindley-Milner Type System
- **ADR-021:** Data types (future)

---

## 🐛 Known Issue: Exponentiation Operator (^) for Complex Numbers

**Discovered:** December 19, 2024  
**Status:** Open - workaround available

### The Problem

The `^` operator for exponentiation crashes or misbehaves with complex numbers in Z3:

```
λ> :sat ∃(z : ℂ). z^2 = -1
thread 'main' panicked at vendor/z3/src/func_decl.rs:224:18
```

**Also:** Superscript notation `z²` is parsed as a variable name, not `power(z, 2)`.

### Root Cause

- `translate_power` in `src/solvers/z3/translators/arithmetic.rs` only handles `Int^Int`
- For all other types, it falls back to uninterpreted function
- **No `complex_power` implementation exists** (unlike `complex_add`, `complex_mul`, etc.)
- `power` is NOT in `DISPATCHABLE_OPS` in `type_mapping.rs`

### Workaround

Use explicit multiplication:
```kleis
:sat ∃(z : ℂ). z * z = complex(-1, 0)
✅ Satisfiable: z = -i
```

### Fix Options

**Option 1: Add `complex_power` to Z3 backend**
- For integer exponents, expand to repeated multiplication: `z^3 = z * z * z`
- Add to `translate_operation` dispatch in `backend.rs`

**Option 2: Axiomatic definition** (preferred, aligns with Kleis philosophy)
```kleis
structure Power(T) over Monoid(T) {
    operation power : T × ℕ → T
    axiom power_zero : ∀(x : T). power(x, 0) = e
    axiom power_succ : ∀(x : T)(n : ℕ). power(x, n + 1) = x * power(x, n)
}

implements Power(ℂ) {
    operation power = complex_power  // Rust builtin
}
```

**Option 3: Parser enhancement**
- Lex `z²` (superscript) as `power(z, 2)`
- Desugar `x^n` to `power(x, n)` before type inference

### Priority

**Medium** - workaround exists (`z * z`), but syntax should work eventually.

### Files to Modify

- `src/solvers/z3/backend.rs` - Add `complex_power` case
- `src/solvers/z3/translators/arithmetic.rs` - Update `translate_power`
- `src/solvers/z3/type_mapping.rs` - Add `power` to `DISPATCHABLE_OPS` if using type dispatch

---

## 🎯 NEXT: Transcendental Functions (sin, cos, log, exp, etc.)

### The Gap

Kleis currently handles:
- ✅ Verification (Z3)
- ✅ Numerical calculations (arithmetic)
- ✅ Plotting (Lilaq/Typst)

But lacks **transcendental functions** for scientific computing:

```kleis
// These don't work yet:
let y = sin(x)      // ❌
let z = exp(-t)     // ❌
plot(xs, map(cos, xs))  // ❌
```

### Implementation Plan

**Use Rust's `std::f64`** — no external dependencies needed!

| Function | Rust Implementation | Notes |
|----------|---------------------|-------|
| `sin(x)` | `x.sin()` | Radians |
| `cos(x)` | `x.cos()` | Radians |
| `tan(x)` | `x.tan()` | Radians |
| `asin(x)` | `x.asin()` | Returns radians |
| `acos(x)` | `x.acos()` | Returns radians |
| `atan(x)` | `x.atan()` | Returns radians |
| `atan2(y, x)` | `y.atan2(x)` | 2-argument arctangent |
| `sinh(x)` | `x.sinh()` | Hyperbolic |
| `cosh(x)` | `x.cosh()` | Hyperbolic |
| `tanh(x)` | `x.tanh()` | Hyperbolic |
| `exp(x)` | `x.exp()` | e^x |
| `log(x)` | `x.ln()` | Natural log |
| `log10(x)` | `x.log10()` | Base-10 log |
| `log2(x)` | `x.log2()` | Base-2 log |
| `sqrt(x)` | `x.sqrt()` | Square root |
| `pow(x, y)` | `x.powf(y)` | x^y |
| `abs(x)` | `x.abs()` | Absolute value |
| `floor(x)` | `x.floor()` | Round down |
| `ceil(x)` | `x.ceil()` | Round up |
| `round(x)` | `x.round()` | Round to nearest |

**Accuracy:** All functions are IEEE 754 compliant, < 1-2 ULP accuracy (same as NumPy, MATLAB, Julia).

### Files to Modify

1. **`src/evaluator.rs`** — Add `builtin_sin`, `builtin_cos`, etc.
2. **`stdlib/prelude.kleis`** — Declare operations with types:
   ```kleis
   operation sin : ℝ → ℝ
   operation cos : ℝ → ℝ
   operation exp : ℝ → ℝ
   operation log : ℝ → ℝ
   // etc.
   ```
3. **`examples/math/transcendental.kleis`** — Test examples
4. **`docs/manual/`** — Document in reference

### Example Usage (After Implementation)

```kleis
example "damped oscillation" {
    let t = [0, 0.1, 0.2, 0.3, 0.4, 0.5]
    let y = [exp(negate(0)) * cos(0),
             exp(negate(0.1)) * cos(0.1),
             exp(negate(0.2)) * cos(0.2),
             exp(negate(0.3)) * cos(0.3),
             exp(negate(0.4)) * cos(0.4),
             exp(negate(0.5)) * cos(0.5)]
    plot(t, y, "Damped Oscillation")
}
```

### Priority

**High** — Needed for scientific plotting and numerical examples.

---

## 🚀 PREVIOUS: Self-Hosted Differential Forms (Dec 30, 2024)

### The Breakthrough

**Kleis can implement exterior algebras in PURE KLEIS** — no new Rust builtins needed!

We discovered that:
1. Nested lists work for tensors (block matrices already prove this)
2. Recursion + pattern matching + fold/map = Turing complete
3. Permutations can be generated in pure Kleis
4. All differential geometry operations reduce to these primitives

### Architecture (ADR-026)

```
Layer 4: Differential Geometry (pure Kleis)
  wedge, d, ⋆, ι_X, ℒ_X
           ↓
Layer 3: Tensor Algebra (pure Kleis)
  antisymmetrize, permute_indices, tensor_product
           ↓
Layer 2: Combinatorics (pure Kleis)
  all_permutations, perm_sign, factorial
           ↓
Layer 1: Functional Core (pure Kleis)
  fold, map, range, filter, length, append
           ↓
Layer 0: Primitives (Rust - invisible)
  +, -, *, /, Cons, Nil, match, if, λ
```

### Implementation Plan

- [x] **`stdlib/func_core.kleis`** - fold, map, filter, range, length, append (5 tests ✅)
- [x] **`stdlib/combinatorics.kleis`** - factorial, permutations, signs (12 tests ✅)
- [x] **`stdlib/tensors_functional.kleis`** - tensor ops as Kleis functions (16 tests ✅)
  - wedge antisymmetric: `α ∧ β = -(β ∧ α)` PROVEN! ✅
  - wedge nilpotent: `α ∧ α = 0` PROVEN! ✅
  - Parallelogram area via wedge product = determinant ✅
  - Electromagnetic field tensor (from Jackson's Electrodynamics) ✅
- [x] **Update `stdlib/differential_forms.kleis`** - replace builtin_* with pure Kleis imports

### ✅ FIXED: Example Block Assertion Bug (Jan 2, 2026)

**Discovery:** `assert(sin(0) = 0)` was failing because `eval()` returned
`Operation{sin, [0]}` instead of the value `0`.

**Fix:** `eval_equality_assert()` now uses `eval_concrete()` which fully
evaluates expressions including all builtin functions.

Also added floating-point epsilon comparison (1e-10 relative tolerance)
for numeric assertions to handle floating point rounding.

**Note:** Bare equality `expr = value` (without `assert()`) is still side-effect-only.
Always use `assert(a = b)` for actual assertions.

### Gap Analysis (All Resolved!)

| Feature | Status | Notes |
|---------|--------|-------|
| Tensor type | ✅ | Use nested lists |
| fold/map/range | ✅ | Define in Kleis, tested working |
| Permutations | ✅ | Implement recursively in Kleis |
| Permutation sign | ✅ | Count inversions |
| Tensor product | ✅ | Outer product via nested map |
| Antisymmetrization | ✅ | Sum over permutations with signs |
| Wedge product | ✅ | antisymmetrize(tensor_product) |
| Cartan formula | ✅ | **The axiom IS the implementation!** |

### Key Insight

**Cartan's Magic Formula:**
```kleis
// This isn't just an axiom — it's the IMPLEMENTATION
define lie(X, α) = plus(d(interior(X, α)), interior(X, d(α)))
```

The axioms we wrote for verification ARE the executable definitions!

### Branch

Working in branch: `exterior-algebras`

---

## ✅ IMPLEMENTED: Unified Type System for Debugger (Dec 29, 2024)

The debugger now uses the same type infrastructure as Z3, ensuring consistency across the platform.

### Changes Made

1. **`TypedBinding` struct** in `src/debug.rs`
   - Stores variable value, inferred type, and verification status
   - Includes `display()` method for formatted output: `M : Matrix(2,3,ℝ) = [[...]]`
   - Added `Serialize` derive for DAP protocol compatibility

2. **`on_bind_typed` hook** in `DebugHook` trait
   - Type-aware version of `on_bind`
   - Implementations in `InteractiveDebugHook` and `DapDebugHook`

3. **`on_assert_verified` hook** in `DebugHook` trait
   - Called when assertions are verified by Z3
   - Reports verification status (passed/failed/disproved/unknown)

4. **DAP variable responses** now include `type` field
   - Updated in `src/bin/kleis.rs` (`handle_dap_request`)
   - VS Code Variables panel can display types

5. **`format_type` function** in `src/debug.rs`
   - Converts `type_inference::Type` to human-readable strings
   - Uses mathematical notation: `ℕ`, `Matrix(2,3,ℝ)`, `α → β`

### Benefits

| Feature | Before | After |
|---------|--------|-------|
| Variable display | `x = 42` | `x : ℕ = 42` |
| Matrix display | `M = <expr>` | `M : Matrix(2,3,ℝ) = [[...]]` |
| Assertion status | Not shown | `✓` / `✗` badges |
| Complex numbers | `c = (1,2)` | `c : ℂ = 1+2i` |

### Architecture

```
Equation Editor ─┐
                 │
Debugger (DAP) ──┼──→ type_inference::Type ──→ z3/type_mapping.rs ──→ Z3
                 │
kleis test ──────┘
```

All components now use the same canonical type system.

---

## ✅ IMPLEMENTED: Concrete Evaluation via Z3 Simplify (Dec 29, 2024)

### The Problem

Kleis is a **symbolic language** — expressions are kept as ASTs for formal reasoning, not reduced to concrete values. This means:

```kleis
// User expectation:
define compute_box_row(r) = if r ≤ 3 then 1 else if r ≤ 6 then 2 else 3
compute_box_row(5)  // Expected: 2

// Kleis reality:
compute_box_row(5)  // Returns: if 5 ≤ 3 then 1 else if 5 ≤ 6 then 2 else 3
                    // (expression is NOT reduced)
```

### Proposed Solution: `eval()` or `reduce()` via Z3

Use Z3's `simplify` function for ground term (no free variables) reduction:

```kleis
// New syntax option A: eval() function
example "concrete calculations" {
    assert(eval(1 + 2 * 3) = 7)
    assert(eval(compute_box_row(5)) = 2)
    assert(eval(∀(x : ℕ). x = x) = true)
}

// New syntax option B: reduce() function  
example "concrete calculations" {
    assert(reduce(1 + 2 * 3) = 7)
}
```

### Implementation Approach

| Approach | Pros | Cons |
|----------|------|------|
| **Z3 simplify** | Semantically consistent with verification | Slower, requires Z3 |
| **Rust evaluator** | Fast, simple | Could diverge from Z3 semantics |
| **Hybrid** | Fast for arithmetic, Z3 for complex | More complex implementation |

**Recommended: Z3 simplify** — keeps semantics consistent across evaluation and verification.

### Implementation Plan

1. **Add `eval()` operation** to parser (returns result of Z3 simplify)
2. **Ground term check** — only evaluate if no free variables
3. **Timeout protection** — 1 second max per evaluation
4. **Return type** — same as input expression type
5. **Error handling** — return expression unchanged if can't evaluate

### Z3 Backend Changes

```rust
// In src/solvers/z3/backend.rs
impl Z3Backend {
    /// Evaluate a ground term to a concrete value using Z3 simplify
    pub fn evaluate_ground(&self, expr: &Expression) -> Result<Expression, String> {
        if self.has_free_variables(expr) {
            return Err("Cannot evaluate expression with free variables".to_string());
        }
        
        let z3_ast = self.kleis_to_z3(expr)?;
        let simplified = z3_ast.simplify();
        let result = self.z3_to_kleis(&simplified)?;
        Ok(result)
    }
}
```

### Use Cases

| Use Case | Example |
|----------|---------|
| **Unit testing definitions** | `assert(eval(factorial(5)) = 120)` |
| **Sanity checks** | `assert(eval(box_row(5)) = 2)` |
| **Interactive exploration** | REPL: `:eval 1 + 2 * 3` |
| **Debugging** | See concrete value of complex expression |

### Side Effects / Risks

| Risk | Mitigation |
|------|------------|
| **Non-termination** | Timeout protection (1 second) |
| **Free variable error** | Clear error message, return expression unchanged |
| **Semantic mismatch** | Use Z3 (same engine as verification) |
| **User confusion** | Clear documentation: `eval` for ground terms only |

### Implementation (Dec 29, 2024)

**Files Modified:**
- `src/evaluator.rs` — Added `eval()` operation handling and `eval_ground_term()` method
- `src/axiom_verifier.rs` — Added `simplify()` method that delegates to Z3Backend
- `examples/sudoku/sudoku.kleis` — Added concrete evaluation examples

**How It Works:**
1. Parser sees `eval(expr)` → treated as built-in operation
2. Evaluator evaluates the argument first
3. Checks `is_symbolic()` — if expression has free variables, returns error
4. Calls `AxiomVerifier::simplify()` which uses Z3's simplify
5. Z3 reduces the ground term to a concrete value

**Test Results:**
- 795 unit tests pass
- 39 eval_concrete integration tests pass
- 10 Sudoku examples pass (including 4 new `eval()` tests)

**Limitations:**
- Boolean comparison: `eval(5 ≤ 3) = false` doesn't work due to `Const("false")` vs `Object("false")` mismatch
- Workaround: Use conditional `eval(if 5 ≤ 3 then 1 else 0) = 0`

### Related

- ADR-016: Operations in Structures (self-hosting)
- Sudoku example (`examples/sudoku/sudoku.kleis`) demonstrates the feature

---

## 🎯 FUTURE: Big Operators as Unified Binders (Dec 28, 2024)

### Unifying Slogan

**Σ/Π/∫/lim are big operators. Big operators are binders.**

### Binder Structure

Every binder has:
1. **Bound variable** — the index/parameter being abstracted
2. **Domain specification** — how it ranges (set, interval, filter, approach)
3. **Body** — the expression being computed
4. **Semantics** — algebra/topology that gives meaning

### Current Binders in Kleis

| Binder | Syntax | Bound Var | Domain | Body |
|--------|--------|-----------|--------|------|
| `∀` | `∀(x : T). P(x)` | x | type T | P(x) |
| `∃` | `∃(x : T). P(x)` | x | type T | P(x) |
| `λ` | `λ x . e` | x | implicit | e |
| `let` | `let x = v in e` | x | singleton | e |
| `match` | `match e { P => b }` | pattern vars | scrutinee | b |

### Proposed Big Operator Syntax (Future)

Harmonize with existing binders:

```kleis
// Sum: Σ(i : ℤ, 1 ≤ i ≤ n). f(i)
// Prod: Π(i : ℤ, i ∈ S). g(i)
// Integral: ∫(x : ℝ, a ≤ x ≤ b). h(x) dx
// Limit: lim(x → a). f(x)
```

Or simpler prefix form:
```kleis
Σ(i = 1..n) f(i)
Π(i ∈ S) g(i)
∫(x ∈ [a,b]) h(x)
lim(x → a) f(x)
```

### ✅ IMPLEMENTED: Sugar Syntax (Dec 28, 2024)

**Parser now supports Unicode big operator syntax:**

```kleis
// Summation: Σ(from, to, body) → sum_bounds(body, from, to)
Σ(1, n, λ i . f(i))

// Product: Π(from, to, body) → prod_bounds(body, from, to)
Π(1, n, λ i . f(i))

// Integral: ∫(lower, upper, body, var) → int_bounds(body, lower, upper, var)
∫(0, 1, λ x . x * x, x)

// Limit: lim(var, target, body) → lim(body, var, target)
lim(x, 0, sin(x) / x)
```

**Also supports simple prefix forms:**
```kleis
Σx    // → Sum(x)
∫f    // → Integrate(f)
```

### Kleis Renderer (Round-Trip)

The Kleis renderer outputs parseable syntax:
- `sum_bounds(body, from, to)` → `Σ(from, to, body)`
- `prod_bounds(body, from, to)` → `Π(from, to, body)`
- `int_bounds(body, lower, upper, var)` → `∫(lower, upper, body, var)`
- `lim(body, var, target)` → `lim(var, target, body)`

### 🏗️ ARCHITECTURE: BigOp as First-Class Binders (v2.0 Target)

**ChatGPT's Design Proposal:**

```rust
// Dedicated AST node (like Quantifier)
Expression::BigOp {
    op: BigOpKind,              // Sum | Prod | Integral | Limit | Sup | Inf
    binder: (String, Option<TypeExpr>),  // (var, type)
    domain: DomainExpr,         // Range(a,b) | Set(S) | Filter(P) | Approach(x→a)
    body: Box<Expression>,
    annotations: HashMap<String, Expression>,  // measure, differential, etc.
}

// DomainExpr variants
enum DomainExpr {
    Range { from: Expr, to: Expr },           // 1..n, a..b
    Set(Expr),                                // S, {1,2,3}
    Filter { domain: Expr, predicate: Expr }, // i ∈ ℤ where P(i)
    Approach { var: String, target: Expr },   // x → a, x → ∞
}
```

**Why This Is More Correct:**

1. **Binder visibility** — Bound variable explicit in AST, not hidden inside lambda
2. **Type checking** — Clear bound variable type annotation
3. **Pattern matching** — Match on `BigOp` variant, not function name
4. **Rendering** — Direct access to binder for pretty-printing (subscript/superscript)
5. **Alpha-equivalence** — Proper variable renaming without lambda inspection
6. **Domain clarity** — Range vs Set vs Filter vs Approach are distinct

**Comparison:**

| Aspect | Current (v0.95) | ChatGPT (v2.0 target) |
|--------|-----------------|----------------------|
| Implementation | ✅ Done, works now | Requires AST + parser + evaluator changes |
| Binder visibility | Hidden inside lambda | Explicit in AST |
| Type checking | Lambda body inference | Clear bound variable type |
| Rendering | Reconstruct from lambda | Direct access to binder |
| Pattern matching | Match on function name | Match on BigOp variant |
| Semantic clarity | "Function with lambda" | "Binder-like operator" |

**Current Approach (v0.95) — Pragmatic Stepping Stone:**

- ✅ Works now
- ✅ Integrates with existing parser/evaluator
- ✅ Can be refactored later without breaking user code
- ✅ Surface syntax (`Σ(1, n, body)`) stays the same

**Recommendation:**

Document ChatGPT's design as the "proper" architecture for v2.0. The current
implementation is a pragmatic stepping stone that:
1. Validates the surface syntax design
2. Provides working semantics for users
3. Can be upgraded to first-class binders when resources allow

**Migration Path:**

1. v0.95 (current): Functions + lambdas, `Σ(from, to, body)` syntax
2. v2.0 (future): `Expression::BigOp` AST node, same surface syntax
3. Users: No code changes required — surface syntax unchanged

### Z3 Limitation

Z3 is first-order — cannot quantify over functions. Higher-order axioms are **specifications**, not Z3-verifiable. See `stdlib/bigops.kleis` for documented semantics.

### Files Created/Updated

- `stdlib/bigops.kleis` — Big operator declarations with equation-editor-compatible names
- `examples/calculus/sum_examples.kleis` — 4 tests
- `examples/calculus/integral_examples.kleis` — 3 tests
- `src/kleis_parser.rs` — Parser for Σ, Π, ∫, lim
- `src/render.rs` — Updated Kleis templates for round-trip

**7/7 examples pass.**

### Parser Tests Added

- `test_parse_sum_sugar` — Σ(1, 10, x) → sum_bounds(x, 1, 10)
- `test_parse_product_sugar` — Π(1, n, f(i)) → prod_bounds(...)
- `test_parse_integral_sugar` — ∫(0, 1, x, x) → int_bounds(x, 0, 1, x)
- `test_parse_limit_sugar` — lim(x, 0, f(x)) → lim(f(x), x, 0)
- `test_parse_sum_prefix` — Σx → Sum(x)
- `test_parse_integral_prefix` — ∫x → Integrate(x)

---

## ✅ DONE: Bitvector Theory Examples (Dec 27, 2024)

### Summary

Created `examples/bitvectors/` directory with comprehensive bitvector theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `bv_types.kleis` | Type/operation declarations for 8-bit bitvectors |
| `bv_axioms.kleis` | Formal axioms (bitwise logic, arithmetic, zero/ones, comparisons, shifts) |
| `bv_examples.kleis` | Axiom verification tests (39 examples) |

### Operations Supported

**Bitwise Logic:**
- `bvand`, `bvor`, `bvxor`, `bvnot`

**Arithmetic (modular):**
- `bvadd`, `bvsub`, `bvmul`, `bvneg`
- `bvudiv`, `bvurem` (unsigned division/remainder)
- `bvsdiv` (signed division)

**Shifts:**
- `bvshl` (left shift)
- `bvlshr` (logical right shift)
- `bvashr` (arithmetic right shift)

**Comparisons:**
- `bvult`, `bvule` (unsigned)
- `bvslt`, `bvsle` (signed)

**Constants:**
- `bv_zero` (0x00)
- `bv_ones` (0xFF)

### Results

**39/39 examples pass** using Z3's built-in decidable bitvector theory (QF_BV).

---

## ✅ DONE: String Theory Examples (Dec 27, 2024)

### Summary

Created `examples/strings/` directory with comprehensive string theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `string_types.kleis` | Type/operation declarations for strings |
| `string_axioms.kleis` | Formal axioms (basics, concatenation, containment, substring, indexOf, conversion) |
| `string_examples.kleis` | Full axiom verification tests (20 examples) |
| `string_simple_examples.kleis` | Direct Z3 string operation tests (20 examples) |

### Operations Supported

- `concat(s, t)` - String concatenation
- `strlen(s)` - String length
- `empty_string` - Empty string constant ""
- `contains(s, sub)` - Substring containment
- `hasPrefix(s, p)` - Prefix check
- `hasSuffix(s, suf)` - Suffix check
- `substr(s, start, len)` - Substring extraction
- `indexOf(s, sub, start)` - Find index of substring
- `replace(s, old, new)` - Replace first occurrence
- `charAt(s, i)` - Get character at index
- `strToInt(s)` / `intToStr(n)` - String-integer conversion

### Results

**40/40 examples pass** using Z3's built-in decidable string theory (QF_SLIA).

---

## ✅ DONE: Set Theory Examples (Dec 27, 2024)

### Summary

Created `examples/sets/` directory with comprehensive set theory axioms and examples verified by Z3.

### Files Created

| File | Purpose |
|------|---------|
| `set_types.kleis` | Type/operation declarations for sets |
| `set_axioms.kleis` | Formal axioms (membership, operations, algebra laws, subsets) |
| `set_empty_examples.kleis` | Empty set behavior tests (10 examples) |
| `set_examples.kleis` | Full axiom verification tests (22 examples) |
| `set_simple_examples.kleis` | Direct Z3 set operation tests (23 examples) |

### Key Fixes

1. **Added `iff` operator handling** - `↔` (biconditional) now maps to Z3 boolean equality
2. **Added `dynamic_to_set()` helper** - Properly converts Dynamic set variables to Z3 Set type
3. **Added `empty_set` special case** - Recognizes `empty_set` as a nullary operation returning the empty set

### Results

**55/55 examples pass** using Z3's built-in decidable set theory.

---

## ✅ DONE: Z3 Built-in Arithmetic Mapping (Dec 27, 2024)

### Summary

Fixed critical performance issue where Z3 was hanging on quantified axioms. The root cause was using **uninterpreted functions** for arithmetic operations instead of Z3's **built-in decidable arithmetic**.

### Problem

Kleis files defined operations like `rat_add`, `rat_mul`, `flow_add`, etc. These were being translated to Z3 as **uninterpreted functions**:

```smt2
; SLOW - Uninterpreted function
(forall ((a Real) (b Real)) (= (rat_mul a b) (rat_mul b a)))
```

With quantifiers, Z3 uses E-matching to instantiate universally quantified axioms. This can be:
- Slow (exponential search)
- Incomplete (may return "unknown")
- Prone to matching loops

### Solution

Map common arithmetic operation names to Z3's **built-in arithmetic**:

```smt2
; FAST - Built-in multiplication
(forall ((a Real) (b Real)) (= (* a b) (* b a)))
```

Z3's built-in Real/Integer arithmetic is **decidable**, so quantified formulas complete instantly.

### Operations Mapped

| User Operation | Z3 Built-in |
|---------------|-------------|
| `plus`, `add`, `rat_add` | `+` |
| `minus`, `subtract`, `rat_sub` | `-` |
| `times`, `multiply`, `rat_mul` | `*` |
| `negate`, `rat_neg` | unary `-` |
| `rat_inv`, `inv`, `reciprocal` | `1/x` |
| `rat_div`, `divide` | `/` |
| `rat_lt`, `rat_gt`, `rat_le`, `rat_ge` | `<`, `>`, `<=`, `>=` |

### Results

- `examples/rationals/rational_examples.kleis`: 18/18 pass (was hanging)
- `examples/ontology/revised/*.kleis`: All complete quickly
- Z3 verification is now practical for field axioms

### Key Insight

**Use built-in arithmetic when possible, uninterpreted functions only when necessary.**

- Field theory (rationals, reals): Use built-in
- Custom abstract algebra (groups over user-defined types): Use uninterpreted
- Mixed: Map operations to built-in where types match

---

## ✅ DONE: Import Registry Fix (Dec 27, 2024)

### Summary

Fixed critical issue where `kleis test` command wasn't loading imported structures and their operations into the registry for Z3 verification.

### Problem

When running `kleis test` on a file that imports another file:
1. The imported file's structures were NOT loaded into the evaluator
2. Operations from imported structures weren't in the registry
3. Z3 fell back to declaring all operations as untyped (`flow_add → Int`)
4. Implements blocks weren't loaded

### Fixes

1. **Added `load_imports_recursive` in `kleis.rs`**
   - `run_test` now recursively loads all imports before the main file
   - Imported structures, operations, and data types are now in the evaluator

2. **Added `implements_blocks` to Evaluator**
   - New field: `implements_blocks: Vec<ImplementsDef>`
   - Loaded in `load_program_with_file`
   - Added to registry in `build_registry`

3. **Registry now complete**

   | TopLevel Variant | Loaded in Evaluator | Added to Registry |
   |-----------------|---------------------|-------------------|
   | `StructureDef` | ✅ | ✅ |
   | `OperationDecl` | ✅ | ✅ |
   | `DataDef` | ✅ | ✅ |
   | `FunctionDef` | ✅ | N/A |
   | `ImplementsDef` | ✅ | ✅ |
   | `TypeAlias` | ✅ | ✅ |

### Tests

Added `tests/import_registry_test.rs` with 5 tests:
- `test_imported_structures_in_registry`
- `test_standalone_structures_no_import_needed`
- `test_structure_registry_has_operations`
- `test_multiple_structures_operations_accessible`
- `test_implements_blocks_in_registry`

---

## ✅ DONE: Z3 Backend Major Fixes (Dec 27, 2024)

### Summary

Fixed multiple critical bugs in Z3 axiom verification:

1. **Quantifier Translation Bug (CRITICAL)**
   - `translate_quantifier` was NOT wrapping axiom bodies in `forall_const`
   - Axioms like `∀(G a b). apply_kernel(G, a) = apply_kernel(G, b) → equiv(G, a, b)` were being asserted as just the implication body WITHOUT the quantifier
   - Z3 treated quantified variables as free constants, making all reflexivity proofs fail
   - **Fix:** `translate_quantifier` now uses `z3::ast::forall_const()` and `exists_const()` properly

2. **Typed Function Declarations**
   - Previously: All uninterpreted functions declared with `Int` domain
   - Now: Looks up operation signatures from registry and declares with proper sorts
   - `flow_smul : ℂ × Flow → Flow` now declares `Complex × Int → Int` in Z3
   - Added `get_operation_signature()` to `StructureRegistry`
   - Added top-level operations storage to `StructureRegistry` and `Evaluator`

3. **Complex Type Bound Variables**
   - `fresh_complex_const` was constructing Complex values instead of bound variables
   - This caused sort mismatches when applying functions in quantified contexts
   - **Fix:** Use `Dynamic::fresh_const(name, &complex_sort)` for proper Z3 bound variables

4. **AssertResult::Unknown Handling (CRITICAL)**
   - Previously: Unknown was treated as Passed (optimistic!)
   - **Fix:** Unknown now correctly fails with "Assertion unknown: ..." message
   - Z3 timeouts and inconclusive results are no longer falsely reported as success

5. **Z3 Timeout**
   - Added 30-second timeout to prevent infinite hangs on complex quantified axioms
   - Set via `solver.set_params()` with `timeout: 30000`

### Future Enhancement: Configurable Timeout

The 30-second timeout is currently hardcoded. It should be configurable per-assertion:

```kleis
// Option A: Assert-level timeout
assert reflexivity: equiv(f, f) timeout 60s
assert quick_check: x = x timeout 1s
assert default: y = y  // uses default 30s

// Option B: Example block timeout
example "complex proofs" timeout 120s {
    assert reflexivity: equiv(f, f)
}
```

**Implementation would require:**
1. Grammar change to support `timeout Ns` clause
2. Parser update to parse timeout value
3. AST field for optional timeout
4. Evaluator to pass timeout to backend
5. Backend method to set timeout per-assertion

**Priority:** Low (current 30s default works for most cases)

### Future Enhancement: Parameterized Structures with Structure Dependencies

Current limitation: Structures can only extend other structures, not take them as parameters.

**Proposed syntax:**
```kleis
structure AdmissibleKernel(
  G        : GreenKernel,
  FlowAlg  : FlowAlgebra,
  FieldAlg : FieldR4Algebra
) {
  // Local shorthands using dot notation
  define flow_add  = FlowAlg.flow_add
  define field_add = FieldAlg.field_add

  // Axioms referencing parameter structures
  axiom linearity:
    ∀(α : ℂ, f g : FieldR4).
      G.apply(field_add(f,g)) = field_add(G.apply(f), G.apply(g))
}
```

**Features needed:**
1. Structure parameters with structure types (`G : GreenKernel`)
2. Dot notation for accessing operations (`FlowAlg.flow_add`)
3. Local `define` inside structure body
4. Parameter structures as first-class values

**Implementation would require:**
1. Grammar extension for structure parameters with types
2. Parser support for dot notation in expressions
3. Name resolution for structure member access
4. Type checker updates for structure-typed parameters

**Priority:** Medium (enables cleaner POT formalization)

### Test Results

- All 755 unit tests pass
- POT examples verify correctly (when Z3 completes in time)
- Examples that timeout are correctly reported as "unknown" (failed)

---

## ✅ DONE: Z3 Enhanced Registry Integration (Dec 27, 2024)

### Summary

Extended Z3 backend to leverage `data_types` and `type_aliases` from the StructureRegistry for enhanced verification capabilities.

### What Was Implemented

1. **Z3 ADT Declaration from Registry**
   - New method: `declare_data_types_from_registry()`
   - Converts Kleis `data` declarations into Z3 algebraic data types
   - Automatic constructor distinctness: `Mass ≠ EM ≠ Spin ≠ Color`
   - New field: `declared_data_types: HashMap<String, DatatypeSort>`

2. **Type Alias Resolution**
   - New method: `resolve_type_alias(&TypeExpr) -> TypeExpr`
   - Resolves type aliases before Z3 sort mapping
   - Supports parameterized alias substitution

3. **Enhanced `type_name_to_sort`**
   - Now checks declared data types first
   - Then checks type aliases from registry
   - Falls back to built-in primitives

4. **Registry Iterator Methods**
   - Added `data_types()` iterator
   - Added `type_aliases()` iterator
   - Added `data_type_count()` and `type_alias_count()`

### Complete Registry Support Table

| TopLevel Variant | Loaded in Evaluator | Added to Registry | Used by Z3 |
|-----------------|---------------------|-------------------|------------|
| `StructureDef` | ✅ | ✅ | ✅ (operations, axioms) |
| `OperationDecl` | ✅ | ✅ | ✅ (typed declarations) |
| `DataDef` | ✅ | ✅ | ✅ (Z3 ADT, distinctness) |
| `FunctionDef` | ✅ | N/A | N/A |
| `ImplementsDef` | ✅ | ✅ | ⏳ (verification planned) |
| `TypeAlias` | ✅ | ✅ | ✅ (sort resolution) |

### Benefits of Z3 Using Registry Data Types

1. **Automatic Constructor Distinctness** - Z3 knows `Mass ≠ EM` without explicit axioms
2. **Exhaustiveness Checking** - Z3 can verify pattern matching covers all cases
3. **Accessor Functions** - Fields accessible in Z3 reasoning
4. **No Hardcoding** - User-defined data types get first-class Z3 support
5. **Inductive Reasoning** - For recursive types like `List(T)`

### Benefits of Z3 Using Type Aliases

1. **Consistent Sort Resolution** - `type Scalar = ℝ` always resolves to Real sort
2. **Semantic Type Names** - Write axioms using domain-meaningful names
3. **Parameterized Resolution** - `type Matrix(m, n) = ...` can be resolved

### New Tests

Added to `tests/z3_backend_fixes_test.rs`:
- `test_data_types_registered_in_registry`
- `test_type_aliases_registered_in_registry`
- `test_z3_declares_data_types_from_registry` (with axiom-verification feature)
- `test_z3_resolves_type_aliases` (with axiom-verification feature)
- `test_z3_data_type_constructor_distinctness`
- `test_registry_iteration_methods`

### Documentation

Updated ADR-022 (Z3 Integration) with new "Enhanced Registry Integration" section documenting:
- Benefits of data types → Z3 ADTs
- Benefits of type alias resolution
- Implementation plan and impact assessment

---

## 🎯 POT Formalization: Admissible Kernel Class (Next Steps)

### Current Status (Dec 27, 2024)

The POT formalization in `examples/ontology/revised/` is now **airtight**:
- ✅ Option A refactor complete: all projection is kernel-parameterized
- ✅ `apply_kernel(G, ψ)` is the canonical operation (no implicit kernel)
- ✅ `equiv(G, a, b)` and `in_nullspace(G, a)` are definitional (bidirectional)
- ✅ Field extensionality via `field_at` + `field_ext`
- ✅ No "hidden context" leakage

### Next Move: Minimal Admissible Kernel Class (v0)

Pin down constraints on valid kernels that are:
1. Expressible in Kleis today (no integrals needed)
2. Not so strong it hard-codes known physics
3. Strong enough to generate falsifiable constraints

#### 1) Algebraic Admissibility

**(K1) Linearity over flows** — superposition must survive projection:
```kleis
axiom kernel_linear_add: ∀(G : GreenKernel, a b : Flow).
    apply_kernel(G, flow_add(a, b)) = field_add(apply_kernel(G, a), apply_kernel(G, b))

axiom kernel_linear_smul: ∀(G : GreenKernel, α : ℂ, a : Flow).
    apply_kernel(G, flow_smul(α, a)) = field_smul(α, apply_kernel(G, a))
```

**(K2) Zero preservation** — zero flow projects to zero field:
```kleis
axiom kernel_zero: ∀(G : GreenKernel).
    apply_kernel(G, flow_zero) = field_zero
```

**Status:** K1 already implemented (`project_lin_add`, `project_lin_smul`). K2 needs adding.

#### 2) Observational Equivalence Compatibility

**(K3) Equivalence respects kernel action** — already have via `equiv_elim`/`equiv_intro`.

#### 3) Regularity / Locality (Weak, Falsifiable)

**(K4) Event-local determinacy via probes**:
```kleis
// Residues depend only on local probe at the event
operation probe : GreenKernel × Flow × Event → ℝ

axiom residue_local: ∀(G : GreenKernel, ψ1 ψ2 : Flow, e : Event, c : Channel).
    probe(G, ψ1, e) = probe(G, ψ2, e) → residue(apply_kernel(G, ψ1), e, c) = residue(apply_kernel(G, ψ2), e, c)
```

This keeps "physics local-ish" without hardcoding PDEs.

#### 4) Dimensional Well-Typedness

**(K5) Units constraint** — residues must output quantities with declared units:
```kleis
// Mass channel returns Quantity(kg), Charge returns Quantity(C), etc.
// Prevents "mass in bananas" from being a legal model
```

This requires deciding if Kleis should have a units system (future work).

### Falsifiable Claim Patterns

Once `AdmissibleKernel(G)` exists:

**Pattern A: Invariants**
```kleis
// For all admissible kernels, conserved channels satisfy constraint C
∀(G : AdmissibleKernel). conservation_law(G) → constraint(G)
```

**Pattern B: Geometry Emergence**
```kleis
// For all admissible kernels with symmetry S, induced metric has property P
∀(G : AdmissibleKernel). has_symmetry(G, S) → metric_property(apply_kernel(G, _), P)
```

These are falsifiable because P can be tested against observation.

### Files

- `examples/ontology/revised/pot_core_kernel_projection.kleis` — core formalization
- `examples/ontology/revised/pot_foundations_kernel_projection.kleis` — postulates
- `examples/ontology/revised/spacetime_type_kernel_projection.kleis` — spacetime types

---

---

## ✅ DONE: DAP Debugger Fully Working! (Dec 26, 2024)

### What Works
- ✅ Cross-file debugging (VS Code opens imported files)
- ✅ Correct line numbers for ALL operation types (arithmetic, logical, comparison)
- ✅ Breakpoints work in both main and imported files
- ✅ Variables panel shows AST expressions (symbolic representation!)
- ✅ Stack frames tracked correctly
- ✅ Step over, step into, step out all work
- ✅ **assert() uses Z3 for symbolic verification!**

### Key Insight: DAP as a Window to Kleis Internals
The debugger shows variables as **AST expressions**, not evaluated values:
```
doubled = Operation { name: "plus", args: [Object("x"), Object("x")], span: ... }
x = Const("10")
```

This is **exactly right** for a symbolic mathematics system! Variables hold
symbolic expressions that can be passed to Z3 for verification.

### Fixes Applied (Dec 26, 2024)
1. **Skip expressions without spans** - No more line 1 spurious stops
2. **Parser span capture at START** - Fixed 8 parsing functions to capture span
   before parsing, not after (parse_arithmetic, parse_term, parse_factor,
   parse_comparison, parse_conjunction, parse_disjunction, parse_implication,
   parse_biconditional, parse_where_term)
3. **Fixed double pop_frame bug** - Removed redundant pop_frame() call
4. **Custom operator spans** - Fixed parse_where_term

### Future Ideas

#### 1. Eval Command in Debug Panel
Add ability to evaluate an AST expression to a concrete value during debugging.
The infrastructure exists (`evaluator.eval()`).

#### 2. Extend `example` Block Grammar
Current grammar only allows: `let`, `assert`, expressions.

**Could add:**
```kleis
example "test" {
    define local_fn(x) = x + 1   // Local function definition
    let y = local_fn(5)
    assert(y = 6)
}
```

**Pros:** Self-contained test cases, useful for testing helpers
**Cons:** `example` is for testing not defining; functions can be top-level

#### 3. ✅ Wire assert() to Z3 - DONE! (Dec 26, 2024)
**IMPLEMENTED!** `assert()` in example blocks now uses Z3 for symbolic verification:

```kleis
structure CommutativeRing(R) {
    operation (+) : R × R → R
    axiom commutativity: ∀(a b : R). a + b = b + a
}

example "test commutativity" {
    assert(x + y = y + x)  // ✅ Z3 verifies this using the commutativity axiom!
}
```

**How it works:**
1. `eval_assert()` checks if expressions are symbolic (`is_symbolic()`)
2. If symbolic → calls `verify_with_z3()` using `AxiomVerifier`
3. Z3 loads structure axioms and verifies/disproves the assertion
4. Results: `Verified`, `Disproved { counterexample }`, or `Unknown`

**Test cases added:**
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom  
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

---

---

## ✅ DONE: Type Promotion (Lift) Implemented (Dec 26, 2024)

### What Was Fixed

The type checker now correctly promotes types through the `Promotes` structure.

**Before:** `:type 1 + sin(x)` → `Int` ❌
**After:** `:type 1 + sin(x)` → `Scalar` ✅

### Bugs Fixed

1. **OperationRegistry.merge() missing fields**
   - Added merge for `structure_extends` and `type_promotions`
   - Without this, promotions registered in stdlib weren't available to type checker

2. **Unicode type names not normalized when registering**
   - `implements Promotes(ℕ, ℤ)` was registering as `("ℕ", "ℤ")`
   - But `has_promotion` and `find_common_supertype` normalize to `("Nat", "Int")`
   - Fix: Normalize in `register_implements` before storing

3. **Top-level operations not registered**
   - Operations like `operation sin : ℝ → ℝ` were ignored (TODO stub)
   - Added `toplevel_operation_types` to `OperationRegistry`
   - Type inference now queries these for function return types

4. **Added type_expr_to_type helper**
   - Converts `TypeExpr` to `Type` for return type extraction
   - Handles Function, Named, Parametric, Product, ForAll, DimExpr

### Test Results

All 8 type promotion tests pass:
- `:type sin(x) = Scalar` ✅ (was `Var(TypeVar(0))`)
- `:type 1 + sin(x) = Scalar` ✅ (was `Int`)
- `:type (1 + sin(x)) / 2 = Scalar` ✅ (was `Int`)
- `:type 1 + 3.14 = Scalar` ✅
- Promotions registered: Nat→Int, Int→Scalar, etc. ✅

### Files Modified
- `src/type_context.rs` - Major fixes to registry and type lookup
- `tests/type_promotion_test.rs` - New test file with 8 tests

---

## ✅ DONE: First-Class Function Types Implemented (Dec 26, 2024)

### What Was Implemented

Added `Type::Function(Box<Type>, Box<Type>)` variant to the type system:

```rust
pub enum Type {
    // ...
    /// Function type: A → B
    Function(Box<Type>, Box<Type>),
    // ...
}
```

### Files Modified
- `src/type_inference.rs` - Added Function variant, updated unify(), occurs(), apply()
- `src/type_context.rs` - Updated type_expr_to_type() and interpret_toplevel_operation_type()
- `tests/function_type_test.rs` - New test file with 9 tests

### What Works Now
- **Display:** `sin : Scalar → Scalar` displays correctly with arrow
- **Unification:** Function types unify properly (same domains/codomains)
- **Occurs check:** Prevents infinite types like `α = α → ℝ`
- **Higher-order functions:** Can represent `(T → U) → List(T) → List(U)`
- **Curried functions:** Can represent `ℝ → ℝ → ℝ`

### ✅ Product Types - DONE

Product types now have proper support with `Type::Product(Vec<Type>)` variant.

---

## 🔴 Tech Debt: Hardcoded Type Annotation Parsing

### Problem

`type_inference.rs` has `parse_type_annotation()` (lines 1017-1080) that parses type 
annotation strings like `"Matrix(3, 3, ℝ)"`. It **hardcodes** type names instead of 
querying the registry.

**Location:** `src/type_inference.rs` lines 1017-1080

```rust
fn parse_type_annotation(&self, annotation: &str) -> Type {
    match annotation.trim() {
        "ℝ" | "Real" => return Type::scalar(),    // Hardcoded
        "ℂ" | "Complex" => /* hardcoded */,
        // ...
    }
    
    match type_name {
        "Matrix" => /* hardcoded parsing */,       // Should query registry
        "Vector" => /* hardcoded parsing */,       // Should query registry
        // ...
    }
}
```

Also: convenience constructors `Type::matrix()`, `Type::pmatrix()`, etc. at lines 2087-2131.

### Impact

- Works fine because Matrix/Vector ARE defined in stdlib
- But violates ADR-016 (operations/types should come from structures, not Rust)
- Adding new parametric types requires Rust code changes

### Solution

Query registry for known parametric types:
1. Get list of parametric structures from registry
2. Parse type args based on structure's parameter list
3. Remove hardcoded type name matching

### Workaround

Works today - just not self-hosting. Low priority.

---

## ✅ DONE: N-ary Product Types (Grammar v0.94)

Parser now supports n-ary product types:

```kleis
operation mass_at : GreenKernel × Flow × Event → ℝ  // ✅ Works!
```

**Implementation:** `src/kleis_parser.rs` lines 1609-1635
- `parse_product_type()` is right-associative
- `A × B × C × D` flattens into `TypeExpr::Product([A, B, C, D])`
- `×` binds tighter than `→`

**✅ DONE:** `Type::Product(Vec<Type>)` variant added - full product type support in type inference

---

## ✅ DONE: assert() Uses Z3 Verification (Dec 26, 2024)

**Implemented!** `assert()` in example blocks now uses Z3 for symbolic verification.

### Changes Made
- Added `is_symbolic()` to detect if expressions contain unbound variables
- Added `verify_with_z3()` to call `AxiomVerifier.verify_axiom()`
- Modified `eval_equality_assert()` to try Z3 when expressions are symbolic
- Added `AssertResult::Verified` and `AssertResult::Disproved` variants

### Tests Added (`tests/crossfile_debug_test.rs`)
- `test_assert_with_z3_verification` - Verifies commutativity axiom
- `test_assert_associativity` - Verifies associativity axiom
- `test_assert_invalid_symbolic` - Z3 correctly disproves `x + y = y + y`
- `test_assert_concrete_values` - Structural equality for bound variables

### How It Works
```kleis
assert(x + y = y + x)   // ✅ Z3 verifies via commutativity axiom
assert(x + y = y + y)   // ❌ Z3 disproves: "Counterexample: y!1 -> 1, x!0 -> 0"
assert(4 = 4)           // ✅ Concrete equality (no Z3 needed)
```

---

## ✅ DONE: Thread-Safe AST Cache (ADR-025)

**See:** `docs/adr/adr-025-debugger-shared-context.md`

Implemented thread-safe AST cache shared between LSP and DAP:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Thread-Safe AST Cache                         │
│     Arc<RwLock<HashMap<PathBuf, CachedDocument>>>               │
│                                                                  │
│  CachedDocument {                                                │
│    source: String,                                               │
│    program: Option<Program>,  // The AST                         │
│    imports: HashSet<PathBuf>, // Dependencies                    │
│    dirty: bool,               // Needs re-parse?                 │
│  }                                                               │
└─────────────────────────────────────────────────────────────────┘
           ↑                              ↑
           │ write                        │ read (or write if miss)
           │                              │
    ┌──────┴───────┐               ┌──────┴───────┐
    │     LSP      │               │     DAP      │
    │  (Thread 1)  │               │  (Thread 2)  │
    │              │               │              │
    │  Evaluator   │               │  Evaluator   │
    │  (own copy)  │               │  (own copy)  │
    └──────────────┘               └──────────────┘
```

**Key features:**
- LSP updates cache when documents change
- DAP reads from cache (or parses and caches if missing/dirty)
- Cascade invalidation: dirty files propagate to dependents
- Each thread has its own `Evaluator` (because `RefCell` is not `Sync`)

---

## ✅ DONE: DAP Line Number Issues FIXED! (Dec 26, 2024)

### What Was Fixed

1. **Parser span capture at START of operations** - Fixed 8 parsing functions
2. **Skip expressions without spans** - No more line 1 spurious stops
3. **Custom operator spans** - Fixed parse_where_term

### Current State (ALL WORKING!)

| Component | Status |
|-----------|--------|
| Parser populates `SourceSpan` with file path | ✅ |
| `ExampleStatement` carries location | ✅ |
| Evaluator calls `on_eval_start()` for every expression | ✅ |
| `DapDebugHook` exists with channel-based communication | ✅ |
| DAP returns stack traces with file paths | ✅ |
| VS Code shows debugger UI | ✅ |
| DAP wires hook to evaluator | ✅ |
| Cross-file debugging (file switching) | ✅ |
| **Line numbers accurate in cross-file stepping** | ✅ FIXED! |

### Architecture (from `REPL_ENHANCEMENTS.md`)

```
┌─────────────────────────────────────────────────────────────┐
│                     kleis server                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐       │
│  │   LSP       │◄─►│  Shared     │◄─►│   DAP       │       │
│  │  Handler    │   │  Context    │   │  Handler    │       │
│  └─────────────┘   │ - Evaluator │   └─────────────┘       │
│                    │ - Types     │                          │
│                    │ - Structs   │                          │
│                    └─────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

**Key Design Points:**
- **RefCell** ensures zero overhead when not debugging (hook is `None`)
- **DapDebugHook** blocks in evaluator thread, communicates via channels
- **DapDebugController** held by DAP server, sends commands, receives events
- **DO NOT change RefCell** - it's there for a purpose!

### Implementation Plan

#### Step 1: Update `DapState` to Hold Controller

```rust
struct DapState {
    // ... existing fields ...
    
    /// Controller for channel-based communication with DebugHook
    controller: Option<DapDebugController>,
    
    /// Handle to evaluation thread
    eval_thread: Option<std::thread::JoinHandle<()>>,
    
    /// Parsed program (for finding example blocks)
    program: Option<Program>,
}
```

#### Step 2: Wire `launch` Handler

1. Parse file with `parse_kleis_program_with_file(source, canonical_path)`
2. Find first `ExampleBlock` to debug
3. Create `DapDebugHook` + `DapDebugController` via `DapDebugHook::new()`
4. Store controller in `DapState`
5. **Don't start evaluation yet** (wait for `configurationDone`)

#### Step 3: Wire `setBreakpoints` Handler

1. Create `Breakpoint { file, line, enabled: true }` for each
2. Store in `DapState.breakpoints`
3. Will be added to hook before evaluation starts

#### Step 4: Wire `configurationDone` Handler

1. Lock evaluator, set hook: `evaluator.set_debug_hook(hook)`
2. Spawn evaluation thread:
   ```rust
   thread::spawn(move || {
       evaluator.eval_example_block(&example);
       // Send terminated when done
   });
   ```
3. Wait for first `StopEvent` from `controller.event_rx`
4. Send `stopped` event to VS Code

#### Step 5: Wire Step Commands

| DAP Command | DebugAction |
|-------------|-------------|
| `next` | `StepOver` |
| `stepIn` | `StepInto` |
| `stepOut` | `StepOut` |
| `continue` | `Continue` |

1. Send via `controller.command_tx.send(action)`
2. Wait for `StopEvent` from `controller.event_rx`
3. Update `current_file` and `current_line` from event
4. Send `stopped` event to VS Code

#### Step 6: Wire `stackTrace` Handler

- Get stack from `StopEvent.stack`
- Store latest stack in `DapState`
- Return frames with `source.path` (absolute paths)

#### Step 7: Wire `variables` Handler

- Get bindings from top stack frame
- Return as DAP variables

#### Step 8: Handle Evaluation Complete

- Add `Terminated` variant to `StopEvent` (or use channel close)
- Send `terminated` event to VS Code

### Why This Works for Cross-File Debugging

The evaluator calls `on_eval_start` with whatever `SourceLocation` the AST has.
When stepping into a function from an imported file, the AST node has that file's path.
The hook receives it, checks breakpoints, sends stop event with the correct file.
**No per-construct hardcoding needed.**

---

## 🧠 CRITICAL ARCHITECTURE: SharedContext AST Cache

### The Insight

**LSP already parses every file the user has open.** It re-parses on every edit.
DAP should NOT parse files separately — it should use the SAME cached AST.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              SharedContext.documents                         │
│                                                              │
│   HashMap<PathBuf, CachedDocument>                          │
│                                                              │
│   "/path/to/main.kleis"    → AST (parsed by LSP on open)    │
│   "/path/to/helper.kleis"  → AST (parsed by LSP on open)    │
│   "/path/to/stdlib/prelude" → AST (parsed by DAP if needed) │
│                                                              │
└─────────────────────────────────────────────────────────────┘
        ↑                              ↑
   LSP updates on edit             DAP reads (parses only if missing)
```

### The Rule

1. **DAP checks cache first** before parsing any file
2. **If found** → use it (FREE, already parsed by LSP)
3. **If not found** → parse, then ADD to cache for future use
4. **Both LSP and DAP use the same cache**

### Cache Invalidation (CRITICAL)

**When a file changes, all files that IMPORT it must be evicted from cache.**

Example:
```
main.kleis imports helper.kleis
helper.kleis imports stdlib/prelude.kleis

If stdlib/prelude.kleis changes:
  → Evict helper.kleis (imports stdlib)
  → Evict main.kleis (imports helper which imports stdlib)
```

This requires **dependency tracking**:
```rust
struct CachedDocument {
    ast: Program,
    imports: Vec<PathBuf>,        // Files this doc imports
    imported_by: Vec<PathBuf>,    // Files that import this doc (reverse)
}
```

When file X changes:
1. Evict X from cache
2. For each file that imports X, recursively evict

### Performance Impact

| Without Cache | With Cache |
|---------------|------------|
| Debug start: parse file (50ms) | 0ms (already parsed) |
| Step into import: parse (50ms) | 0ms if open in editor |
| Edit during debug: parse twice | Parse once (LSP only) |

### Why This Matters

> **The user's editor IS the source of truth.**
> LSP sees what user sees. DAP uses what LSP sees.
> No stale ASTs. No duplicate parsing.

### The Algorithm (Classic Incremental Compilation)

This is the same algorithm used by `make`, `cargo`, Webpack, and TypeScript.

**1. Build Dependency Graph (on parse):**
```rust
fn on_parse(file: &Path, ast: &Program) {
    for import_path in ast.imports() {
        // Forward edge: file imports import_path
        cache[file].imports.push(import_path);
        // Reverse edge: import_path is imported_by file
        cache[import_path].imported_by.push(file);
    }
}
```

**2. Invalidation (on file change) — propagate UP the tree:**
```rust
fn invalidate(file: &Path) {
    if let Some(doc) = cache.remove(file) {
        // Recursively invalidate all dependents
        for dependent in doc.imported_by {
            invalidate(&dependent);
        }
    }
}
```

**3. Lazy Re-parse (on demand) — parse dependencies FIRST:**
```rust
fn get_ast(file: &Path) -> &Program {
    if cache.contains(file) {
        return &cache[file].ast;
    }
    
    // Parse the file
    let ast = parse(file);
    
    // Ensure all imports are in cache first (topological order)
    for import_path in ast.imports() {
        get_ast(&import_path);  // Recursive
    }
    
    // Store and return
    cache.insert(file, CachedDocument { ast, ... });
    &cache[file].ast
}
```

**Visual Example:**
```
stdlib/prelude.kleis CHANGES
         ↓ invalidate
    helper.kleis (imports stdlib) → EVICTED
         ↓ invalidate  
    main.kleis (imports helper) → EVICTED

Later, when DAP needs main.kleis:
    get_ast(main.kleis)
        → get_ast(helper.kleis)  // dependency first
            → get_ast(stdlib/prelude.kleis)  // leaf first
            ← parse stdlib, cache it
        ← parse helper, cache it
    ← parse main, cache it
```

**Key Properties:**
- Parse each file at most once per change
- Dependencies parsed before dependents (topological order)
- Lazy: only re-parse when actually needed
- Minimal work: only affected files re-parsed

### Files to Modify

| File | Changes |
|------|---------|
| `src/bin/kleis.rs` | Update `DapState`, wire handlers |
| `src/debug.rs` | Add `Terminated` event (if needed) |

### Technical Debt to Address

**1. Consolidate DAP Implementations**
- `src/dap.rs` — Library version (marked `#[deprecated]`)
- `src/bin/kleis.rs` — Used by `kleis server` (the active one)
- **Action:** Remove `src/dap.rs` after confirming `kleis server` works end-to-end

**2. Review DebugHook Implementations**
We have 3 implementations in `src/debug.rs`:
- `NoOpDebugHook` — Zero overhead when not debugging (KEEP)
- `InteractiveDebugHook` — For REPL `:debug` command (KEEP for REPL)
- `DapDebugHook` — For VS Code DAP integration (KEEP for DAP)

**Action:** After wiring is complete, review if `InteractiveDebugHook` and `DapDebugHook` can share more code or if the separation is justified.

**3. Squash Commits Before Merging**
The `feature/debugger-dap` branch has 63+ incremental commits. Before merging to `main`, squash into logical commits:
- "Add example blocks and assert to grammar (v0.93)"
- "Implement REPL :debug command"  
- "Add DAP infrastructure for VS Code debugging"
- "Add source location tracking to parser"
- "Wire DAP to evaluator with DapDebugHook"

**Command:** `git rebase -i origin/main` then squash/fixup related commits.

### Test Plan

1. Set breakpoint in `examples/debug_main.kleis` on line 8
2. Set breakpoint in `examples/debug_helper.kleis` on line 6
3. Start debugging `debug_main.kleis`
4. Should stop at line 8
5. Step over to line 11 (`let doubled = double(x)`)
6. Step into → should jump to `debug_helper.kleis` line 6
7. Step out → should return to `debug_main.kleis`

### Key Documents

1. **`docs/plans/REPL_ENHANCEMENTS.md`** — Master plan, Phase 6 (Debugging)
2. **`docs/plans/EXPRESSION_SPANS.md`** — Future: spans on all Expressions
3. **`src/debug.rs`** — DebugHook trait and DapDebugHook implementation

---

## ✅ Current Debugger Status (ALL WORKING!)

| Feature | Status |
|---------|--------|
| Launch/attach | ✅ |
| Breakpoints (set) | ✅ |
| Breakpoints (hit) | ✅ Real, wired to evaluator |
| Breakpoints in imported files | ✅ Works! |
| Step in/over/out | ✅ Real evaluation |
| Continue | ✅ Real evaluation |
| Stack trace | ✅ Correct file paths |
| Variables | ✅ Shows AST expressions |
| Cross-file (file switching) | ✅ Works |
| Cross-file (line numbers) | ✅ **FIXED!** All operations correct |
| assert() with Z3 | ✅ **NEW!** Symbolic verification |

### Files to Review

- `src/bin/kleis.rs` — Unified binary (DAP implementation here)
- `src/debug.rs` — DebugHook trait and DapDebugHook
- `src/evaluator.rs` — Calls debug hooks at key points
- `vscode-kleis/src/extension.ts` — VS Code integration

---

## ✅ DONE: Matrix Arithmetic Type Inference Fix

**Problem:** `minus(Matrix, Matrix)` was incorrectly returning `Scalar` type.

**Root Cause:** The hardcoded type hierarchy in `type_inference.rs` (lines 1401-1489) checked for Complex, Rational, Scalar, Int, Nat but **never checked for Matrix**. If nothing matched, it defaulted to Scalar.

**Fix:** Added Matrix handling before the default fallback (lines 1474-1485):
```rust
// Check for Matrix - if either arg is Matrix, return that Matrix type
if let Type::Data { constructor, .. } = &t1 {
    if constructor == "Matrix" {
        return Ok(t1.clone());
    }
}
// ... similar for t2
```

**Future Work (TODO #10):** Per ADR-016, all ~400 lines of hardcoded type logic should move to `stdlib/prelude.kleis` structures and be queried from the registry. Current approach works but isn't self-hosting.

---

## ✅ DONE: Equation Editor `let x =` Template

Added `let_simple` template for 2-argument let bindings:
- Button in "Logic & Set Theory" palette
- Template in `std_template_lib/logic.kleist`
- Implemented for ℝ, Matrix, and Bool types in `stdlib/prelude.kleis`

---

## 🎯 Equation Editor: Add `let x =` Template

The equation editor needs a template for let bindings:

```
let x = [value] in [body]
```

This allows users to define local variables in the visual editor.

**Files to modify:**
- `static/index.html` - Add button/template
- Template structure: `Let { pattern: "x", value: Placeholder, body: Placeholder }`

---

## 🎯 Equation Editor: Set Type Templates

The Equation Editor should support Set operations with proper type inference.

**Current Status:**
- ✅ REPL can infer Set types: `insert(5, empty_set)` → `Set(Int)`
- ❌ Equation Editor doesn't have Set operation templates

**Needed templates:**
- `in_set(x, S)` - membership test (x ∈ S)
- `union(A, B)` - set union (A ∪ B)
- `intersect(A, B)` - intersection (A ∩ B)
- `difference(A, B)` - difference (A \ B)
- `subset(A, B)` - subset test (A ⊆ B)
- `empty_set` - empty set (∅)
- `singleton(x)` - singleton set ({x})
- `insert(x, S)` - add element

**Files to modify:**
- `static/index.html` - Add buttons to palette
- `std_template_lib/sets.kleist` - Template definitions
- `src/render_editor.rs` - Rendering templates
- `patternfly-editor/` - PatternFly integration

**Leave for future branch:** `feature/equation-editor-sets`

---

## ⚠️ Program Synthesis: Documented Limitation

**The Dream:** `spec → Z3 → program`

**The Reality:** Z3 cannot synthesize recursive programs from grammar. We tried and documented the failure in `feature/program-synthesis` branch.

**What works:**
- Sketch-based synthesis (human provides template, Z3 fills parameters)
- Bounded verification (sort 2-3 elements)
- LLM proposes, Z3 verifies

**Architecture going forward:**
```
LLM → proposes program → Z3 → verifies properties
                              ✓ or counterexample
```

See `docs/vision/VERIFIED_SOFTWARE_DREAM.md` (in abandoned branch) for full analysis.

---

## ✅ DONE: LISP Interpreter in Kleis

- ✅ Parser (recursive descent, S-expressions)
- ✅ Evaluator (arithmetic, lambda, let, letrec)  
- ✅ Recursion: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix
- ✅ `:eval` command for concrete execution
- ❌ `(verify ...)` form — **CANCELLED** (program synthesis doesn't work as envisioned)

---

## ✅ DONE: LISP Interpreter Uses stdlib Ordering Operations

The LISP interpreter (`examples/meta-programming/lisp_parser.kleis`) already:
1. ✅ Imports `stdlib/prelude.kleis`
2. ✅ Uses `le`, `lt`, `gt`, `ge`, `eq` from stdlib `Ordered(T)` structure

No changes needed - this was already working correctly.

---

## ✅ DONE: Type Inference for User-Defined Types

Fixed Dec 21, 2024:
- `:load` now registers data types with TypeChecker
- `:type VNum(42)` → `VNum(Scalar)` ✅
- `:type SAtom("hello")` → `SAtom("hello")` ✅

---

## 📝 Key Learnings (Dec 21, 2024)

1. **Kleis is Turing complete** — proved by implementing LISP interpreter
2. **Data constructors create concrete objects** — not just symbols
3. **Z3 cannot unroll recursion over unbounded ADTs** — fundamental limitation
4. **`:eval` enables execution** — concrete evaluation in Rust
5. **Verification ≠ Synthesis** — Z3 verifies, LLMs synthesize

---

## 🚫 CANCELLED: Implement `(verify ...)` in LISP Interpreter

**Reason:** The program synthesis vision didn't work. Z3 can't evaluate LISP programs symbolically, so `(verify ...)` can't use Z3 the way we hoped.

### What We Have
- ✅ LISP parser (recursive descent, S-expressions)
- ✅ LISP evaluator (arithmetic, comparisons, lambda, let, letrec)
- ✅ Recursion working: `fib(10) = 55`, `fact(5) = 120`
- ✅ Documented in manual appendix

### What We Need to Design
1. **How does `(verify expr)` call Z3?**
   - Option A: Translate LISP → Kleis expression → Z3
   - Option B: Direct LISP → Z3 (bypass Kleis translation)
   - Option C: Add Z3 access to Rust evaluator as a built-in

2. **What syntax for quantifiers?**
   - `(forall (x) (= (+ x 0) x))` - LISP-style
   - How to specify types for quantified variables?

3. **Return value on failure?**
   - `VBool(false)` vs `VSym("Counterexample: x = 42")`

### Why This Matters
See `docs/vision/VERIFIED_SOFTWARE_VISION.md` — this enables:
- Programs with embedded proofs
- Design-by-contract with verification
- The path to "correct by construction" software

### Files to Modify
- `examples/meta-programming/lisp_parser.kleis` - Add verify form
- `src/evaluator.rs` - May need Z3 integration
- `docs/manual/src/appendix/lisp-interpreter.md` - Update with new code

---

## 🎯 PRIORITY: Bourbaki Compliance Roadmap

Based on capability assessment (Dec 19, 2025), here's what's needed to increase Bourbaki coverage from ~15-20% to higher levels.

### Priority 1: Parser Fixes ✅ COMPLETE (Grammar v0.9)

**Status: DONE** (Dec 22, 2025) - All parser issues resolved!

| Issue | Status | Verified By |
|-------|--------|-------------|
| **∀ inside ∧** | ✅ Works | `tests/grammar_v09_test.rs::test_quantifier_in_conjunction` |
| **Function types in quantifiers** | ✅ Works | `tests/grammar_v09_test.rs::test_function_type_with_nested_quantifier` |
| **→ as implication** | ✅ Works | Used throughout axiom definitions |
| **ε-δ limit definition** | ✅ Works | `tests/grammar_v09_test.rs::test_epsilon_delta_limit` |

**Impact:** Full ε-δ analysis definitions, nested quantifiers, and function types in quantifiers all work.

**Next Steps:** Priorities 2-5 are pure Kleis stdlib code (no more Rust changes needed).

### Priority 2: Set Theory in stdlib (Foundation) 📚

Set(T) exists but operations need defining:

```kleis
// Add to stdlib/sets.kleis:
structure SetTheory(X) {
    operation (⊆) : Set(X) × Set(X) → Bool
    operation (∪) : Set(X) × Set(X) → Set(X)
    operation (∩) : Set(X) × Set(X) → Set(X)
    operation 𝒫 : Set(X) → Set(Set(X))
    element ∅ : Set(X)
    
    axiom subset_def: ∀(A B : Set(X)). A ⊆ B ↔ ∀(x : X). in_set(x, A) → in_set(x, B)
    axiom union_def: ∀(A B : Set(X), x : X). in_set(x, A ∪ B) ↔ in_set(x, A) ∨ in_set(x, B)
    axiom power_set_def: ∀(S A : Set(X)). in_set(A, 𝒫(S)) ↔ A ⊆ S
}
```

**Impact:** Enables Bourbaki Vol I (Set Theory foundations).

### Priority 3: Topology in stdlib 🌐

Now verified to be expressible:

```kleis
// Add to stdlib/topology.kleis:
structure TopologicalSpace(X) {
    element tau : Set(Set(X))
    
    axiom empty_open: in_set(∅, tau)
    axiom full_open: in_set(X, tau)
    axiom union_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(union(U, V), tau)
    axiom intersection_closed: ∀(U V : Set(X)). in_set(U, tau) ∧ in_set(V, tau) → in_set(intersect(U, V), tau)
}

structure Continuous(X, Y) over TopologicalSpace(X), TopologicalSpace(Y) {
    operation f : X → Y
    axiom continuity: ∀(V : Set(Y)). in_set(V, tau_Y) → in_set(preimage(f, V), tau_X)
}
```

**Impact:** Enables Bourbaki Vol III (Topology).

### Priority 4: Analysis Structures 📈

```kleis
// Add to stdlib/analysis.kleis:
structure MetricSpace(X) {
    operation d : X × X → ℝ
    
    axiom non_negative: ∀(x y : X). d(x, y) >= 0
    axiom identity: ∀(x y : X). d(x, y) = 0 ↔ x = y
    axiom symmetry: ∀(x y : X). d(x, y) = d(y, x)
    axiom triangle: ∀(x y z : X). d(x, z) <= d(x, y) + d(y, z)
}

structure Limit {
    // Requires parser fix for nested quantifiers
    axiom epsilon_delta: ∀(L a : ℝ, epsilon : ℝ) where epsilon > 0.
        ∃(delta : ℝ). delta > 0
}
```

**Impact:** Enables Bourbaki Vol IV (Analysis), after parser fixes.

### Priority 5: ZFC Axioms (Long-term) 🏛️

```kleis
// Add to stdlib/foundations/zfc.kleis:
structure ZFC {
    // Extensionality
    axiom extensionality: ∀(A B : Set). (∀(x). in_set(x, A) ↔ in_set(x, B)) → A = B
    
    // Pairing
    axiom pairing: ∀(a b). ∃(c : Set). in_set(a, c) ∧ in_set(b, c)
    
    // Union
    axiom union: ∀(F : Set(Set)). ∃(U : Set). ∀(x). in_set(x, U) ↔ ∃(A : Set). in_set(A, F) ∧ in_set(x, A)
    
    // Power Set
    axiom power: ∀(A : Set). ∃(P : Set). ∀(B : Set). in_set(B, P) ↔ B ⊆ A
    
    // Infinity (requires ordinals)
    // axiom infinity: ...
}
```

**Impact:** Full foundational rigor, but Z3 verification may struggle with some axioms.

---

## ⚠️ Z3 Capabilities (Clarified Dec 19, 2025)

**Z3 CAN verify (no Kleis implementation needed):**
- Arithmetic: `∀(n : ℕ). n + 0 = n` ✅
- Algebra: `∀(a b : ℝ). (a-b)*(a+b) = a²-b²` ✅
- Logic: De Morgan, distributivity ✅
- Most Bourbaki-style axioms about ℝ, ℂ, topology ✅

**Z3 struggles with:**

| Limitation | Example | Status |
|------------|---------|--------|
| **Structural induction** | `length(xs ++ ys) = length(xs) + length(ys)` | May timeout |
| **Limits/Convergence** | ε-δ proofs with nested quantifiers | May timeout |
| **Type-level arithmetic** | `Vec(m+n)` from `Vec(m) ++ Vec(n)` | Not expressible |

**Key insight:** Bourbaki is mostly continuous math (ℝ, ℂ, topology) where Z3 works well. Structural induction on lists/trees is rare in Bourbaki.

---

## ✅ Recently Completed

### Operator Overloading (Dec 19, 2025)
- Natural arithmetic: `3 + 4*i = complex(3, 4)` ✅
- Type-directed lowering working
- 17 integration tests

### Capability Assessment (Dec 19, 2025)
- Verified Kleis capabilities against Bourbaki
- Found more works than expected (~15-20% not 5%)
- Documented real limitations

---

## 📊 Current Stats

| Metric | Value |
|--------|-------|
| Tests | 755+ passing |
| Commits | 850+ |
| ADRs | 25 |
| Grammar | v0.93 |
| Unique Cloners | 505+ |
| Bourbaki Coverage | ~15-20% (axiomatic) |
| DAP Debugger | ✅ Fully working! |
| Z3 Assert Verification | ✅ Implemented! |

---

## 🏗️ Architecture Notes

### Operator Overloading Pipeline

```
Parser → Type Inference → Lowering → Z3 Backend
                              ↓
              Rewrites: plus(ℂ, ℂ) → complex_add
                        times(ℝ, ℂ) → complex_mul(lift, _)
```

### Bourbaki Coverage Path

```
Current: Basic Algebra (Groups, Rings, Fields, Vector Spaces)
    ↓ Priority 1-2 (parser + set theory)
Next: Set Theory foundations
    ↓ Priority 3
Next: Topology (open sets, continuity)
    ↓ Priority 4
Next: Analysis (limits, metric spaces)
    ↓ Priority 5
Long-term: ZFC foundations
    ↓ New backend
Ultimate: Induction, transfinite, category theory
```

---

## IDEA: Data-Driven Policy Action Registry

**Problem:** Adding a new policy action type (e.g., `git_pr_create`) currently
requires editing Rust code in `src/mcp/policy.rs` and `src/mcp/protocol.rs` —
the action-to-function dispatch and the JSON schema enum are both hardcoded.
This means every new action type is a code change + recompile.

**Inspiration:** The Z3 backend uses `src/solvers/z3/capabilities.toml` to
declare its capabilities declaratively. The policy system should follow the
same pattern.

**Proposed design:** A `policy_actions.toml` (or similar) that declares:
- Action name (e.g., `git_pr_create`)
- Parameters and their types (e.g., `branch: String`)
- Which Kleis functions to call (`check_git_pr_create`, `before_git_pr_create`)
- The mapping from JSON fields to function arguments

The Rust dispatcher would read this file at startup and dynamically build the
action routing — no code changes needed to add new action types. The policy
`.kleis` file already defines the `check_*` / `before_*` functions; the TOML
just tells the MCP server how to wire JSON requests to those functions.

**Concrete trigger:** We wanted to add a `git_pr_create` rule (agent must
inform user before creating a PR) but realized it required touching Rust code
in three places. That's wrong — policy should be entirely in Kleis + config.

---

*See `docs/CAPABILITY_ASSESSMENT.md` for full analysis.*
