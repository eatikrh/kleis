# Archived Sessions 4-5 (Feb 22, 2026)

*Archived from NEXT_SESSION.md on Feb 26, 2026*

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
