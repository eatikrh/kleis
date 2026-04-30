# Next Session Notes

**Last Updated:** April 27, 2026

---

## Active Research

### Kernel Factorization Reconciliation — IMPORTANT FUTURE WORK

There are **two different kernel decomposition architectures** in the POT papers that need to be reconciled:

**Architecture 1: Three-factor factorization** (Entanglement, Electrodynamics, Rotation Curves papers)
```
K = K_univ ∘ K_dyn ∘ K_rep
```
- K_univ: universal geometric structure
- K_dyn: dynamical sector (elliptic for gravity, hyperbolic for propagation)
- K_rep: representation/measurement sector (scalar for gravity, matrix-valued for spin)
- Each factor is admissible; composition is admissible by `compose_admissible`
- The EM paper extends this: K_em-sector = K_univ ∘ K_dyn ∘ K_em (where K_em = d|Ω¹)
- The entanglement paper uses K_rep as the spinor projection parameterized by detector angle

**Architecture 2: K-Q pipeline** (GR paper, Yang-Mills paper, K-Q Atlas, abstract framework)
```
Configuration → K (production) → intermediate → Q (projection) → observables
```
- K maps configurations to intermediate mathematical objects (curvature, field strength)
- Q extracts observables (Ricci tensor, physical cross-sections)
- Admissibility is tested on K and Q separately
- The formulation fiber analysis lives here (where in the K-Q pipeline is non-admissibility?)

**The tension**: The three-factor factorization decomposes the *production kernel* K into composable admissible pieces. The K-Q pipeline separates *production* from *observation*. These are orthogonal decompositions of the same overall process:

```
Full pipeline:  config → [K_univ ∘ K_dyn ∘ K_rep] → intermediate → Q → observables
                         ←── Architecture 1 ──→    ←── Arch 2 ──→
```

**Questions to resolve**:
1. Is K in the K-Q pipeline = K_univ ∘ K_dyn ∘ K_rep? If so, what is Q in terms of the three factors?
2. For non-admissible K (GR, Yang-Mills): which factor breaks admissibility? Is it K_dyn (the ω∧ω term lives in dynamics)?
3. The EM paper says K_em = d is admissible because U(1) is abelian. In the K-Q pipeline, this means K is admissible and Q (extracting observables from F) is also admissible. Does K_em map to K_dyn or K_rep in the three-factor scheme?
4. Can the formulation fiber analysis (Cartan vs TEGR moving non-admissibility between K and Q) be expressed as re-partitioning the three factors between K and Q?
5. The entanglement paper says the gravitational and measurement kernels are "different faces of the same underlying operator." But the GR paper says K_GR is non-admissible while K_measurement (spinor projection) is admissible. How do these compose if one factor is non-admissible?

**This is not a contradiction** — the three-factor scheme was developed for admissible sectors (gravity = logarithmic kernel, EM, measurement), while the K-Q pipeline handles non-admissible sectors (full GR, Yang-Mills). The reconciliation likely involves: the three-factor decomposition applies when K is admissible; when K is non-admissible, the factorization breaks and you need the K-Q pipeline instead. But this needs to be formalized.

### Projection Residues as Elementary Objects — NEW RESEARCH DIRECTION

**Status: Not yet started. Needs a dedicated paper.**

The original POT insight: when a projection kernel degenerates, the residues of the Laurent expansion *are* the elementary physical objects — point masses, point charges, angular momenta. Particles are not fundamental inputs to the theory; they are what remains when an ontological projection collapses.

**Core claim:** For a kernel K that degenerates at a point x₀, the Laurent expansion K(x) ~ R/(x - x₀)ⁿ + ... yields residues R that correspond to physical charges (mass, electric charge, spin).

**Connections across existing papers:**

1. **Renormalization → counterterms**: Q's poles in the regularization parameter ε have residues that are the physical coupling constants. This is where the idea originated — the Q operator annihilates infinity and the residue is the finite answer (-1/12, etc.).

2. **Gravity → mass**: Kernel degeneracy at a spatial point → Schwarzschild-like mass as a residue. The 1/r potential is literally the Green's function with a point-source residue.

3. **Electrodynamics → charge**: Kernel degeneracy → Coulomb charge as a residue. The EM kernel from the electrodynamics paper should have pole structure whose residue is e.

4. **Black hole singularities → no-hair theorem**: Complete kernel collapse at the singularity. The no-hair theorem says only mass, charge, and spin survive — these may be exactly the residues of the degenerate projection kernel. If so, no-hair is a corollary of the residue structure, not a separate theorem.

5. **Non-patchability**: The flat rotation curves and GR papers showed that kernels don't patch spatially. The residue analysis may explain *why* — the pole structure creates topological obstructions to global extension (analogous to how a meromorphic function can't be extended across its poles).

**Approach:**
- Start with the kernels already computed in the GR and electrodynamics papers
- Identify where they degenerate (poles, essential singularities)
- Compute the Laurent expansion and characterize residues
- Show that the residues correspond to known physical quantities
- Investigate whether the residue structure is invariant across K-Q factorization schemes (scheme independence)
- Use Kleis numerical tools (eigenvalues, FFT, tensor operations) for explicit computations

**Key question:** Is the residue structure invariant across all non-unique K-Q factorizations? If yes, the residues are the scheme-independent physical content — the part of the theory that doesn't depend on how you split K from Q.

### Future Research Questions

- **Kernel unification problem**: A reader of the four-kernel table would naturally ask whether a single kernel can reproduce all four properties (rotation curves, frame-dragging, waves, binary pulsar). A naive spatial patch fails because gravitational waves propagate through intergalactic space. A true unified kernel would need to reduce to logarithmic behavior for quasi-static galactic potentials and to linearized GR behavior for dynamical/radiative modes.

- **Teleparallel kernel in Kleis**: Formalize the TEGR pipeline in a full Kleis theory file. Computing the torsion scalar and showing it matches the Einstein-Hilbert Lagrangian (up to a boundary term) would be a strong computational verification.

- **The circularity question**: The Deser self-coupling argument says gravity must couple to its own energy. But non-admissibility makes gravitational energy non-localizable. In TEGR, gravitational energy IS localizable but Q is non-admissible. The circularity moves but doesn't resolve.

### DO NOT

- Do NOT edit the Typst output file directly
- Do NOT change the plan file
- Do NOT use `render_paper()` — the correct function is `compile_arxiv_paper()`
- Do NOT use `ArxivPaper(...)` — the correct constructor is `Paper(...)`
- Do NOT use `$Q circ K$` in paper source — Typst wants `$Q compose K$`

---

### Paper 8: One Field, Two Projections — READY TO IMPLEMENT

**Status: Plan written. Ready to implement.**

Plan file: `.cursor/plans/classical_quantum_kernel_reach_paper_8.plan.md`

#### The kernel inclusion theorem

For any phenomenon with both classical and quantum descriptions:

    ker(K_qu) ⊆ ker(K_cl)

The quantum kernel reaches strictly more of the source. The "classically invisible, quantum-activated" sector is:

    Δ = ker(K_cl) \ ker(K_qu)

For EM/QED: Δ = {ψ} (the electron field). Classical EM sees only A_μ through the exterior derivative. QED sees both A_μ and ψ through the Feynman kernel.

#### What the paper will contain

1. Kernel inclusion axioms
2. EM/QED instantiation — Δ = {ψ}
3. Gravity instantiation — linearized Green's fn vs graviton propagator
4. Fluid instantiation — Biot-Savart vs quantum fluid kernel
5. Minimum field content — union of Δ across domains
6. Philosophical payoff — quantization is kernel refinement, not ontological upgrade

---

### POT Volume XI — BUILT BUT NOT YET PUBLISHED

**Theory file:** `theories/pot_quantization_kernel.kleis` — 19/19 Z3-verified
**Worked file:** `theories/pot_quantization_kernel_worked.kleis` — 7/7 verified
**Paper file:** `examples/ontology/revised/pot_quantization_kernel_paper.kleis`
**Status:** PRs created, landing page link deliberately NOT added — needs further conceptual review

#### THE PROBLEM: Circularity

The paper assumes quantization happens and then finds patterns across quantization formalisms. This is taxonomic, not derivational. For POT, this is a problem.

#### THE KEY INSIGHT: Composite Kernel

K_obs = K_detector ∘ K_propagation ∘ K_source

The discreteness you observe is a property of the composite kernel, not an axiom about Hilbert spaces.

#### What to do next

**Option A:** Publish Volume XI as-is (structural survey). Begin Volume XII (composite kernel thesis).
**Option B:** Fold Volume XI into Volume XII.
**Option C:** Reframe Volume XI to include composite kernel section.

**Decision deferred.** Theory files, Z3 proofs, and worked examples are solid and will be reused.

---

### YANG-MILLS MASS GAP PROGRAM — 7 THEORY FILES, 230+ EXAMPLES

#### The conditional theorem

**Under Assumptions A-D, the IR singularity of the YM weight forces the dressed resolvent into the α = γ = 1/2 Darboux asymptotic class, yielding linear confinement and gap scaling ~ σ^{2/3} · 1.750.**

#### Theory files

| # | File | Examples | What it establishes |
|---|------|----------|---------------------|
| 1 | `pot_spectral_transfer.kleis` | 28 | Spectral mapping theorem, resolvent gap transfer |
| 2 | `pot_green_identification.kleis` | 33 | Anchor theorem, parameter matching, Born series |
| 3 | `pot_weight_families.kleis` | 66 | IR classification (β threshold), Darboux bridge |
| 4 | `pot_ym_darboux_matching.kleis` | 25 | Darboux universality family, gap scaling |
| 5 | `pot_ir_dressing_bridge.kleis` | 34 | Hankel duality, Born dressing, bridge equation |
| 6 | `pot_ym_assumptions.kleis` | 22 | Assumptions isolation, conditional theorem |
| 7 | `pot_assumption_c_proof.kleis` | 22 | Watson's lemma, IR/UV convergence |

#### The five assumptions

| Assumption | Status | What closes it |
|-----------|--------|----------------|
| **A** (γ > 0) | Level C | Derive γ from YM Lagrangian |
| **B** (kernel = resolvent) | Level B/C | Verify resolvent equation |
| **C** (Hankel regularity) | **Level A/B** | Watson's lemma verified; mild condition remains |
| **D** (inverse extraction) | Level B | Apply Gel'fand-Levitan |
| **E** (QFT construction) | Level C | Construct 4D YM (= Clay problem) |

#### Upgrade priority (next targets)

1. **Assumption D** — apply Gel'fand-Levitan to specific spectral measure
2. **Assumption B** — verify resolvent equation for ITCM kernel — **the decisive step**
3. **Assumption A** — non-perturbative QCD input
4. **Assumption E** — the hardest open problem in mathematical physics

#### Key technical lessons

- Z3 `divide`/`rat_div` requires numeric arguments — encode as multiplications
- Z3 `implies` with `element = 1` comparisons can fail — use separate structures
- `let` variable names can conflict with structure elements — use distinct names
- Self-contained theory files (no imports) avoid cross-file Z3 context issues
- **BUG: Nullary `ℝ` operations + equality → Z3 inconsistency.** Declaring `operation foo : ℝ` at file scope and constraining with `axiom : foo = 0.6602` causes Z3 UNSAT. Nullary `ℤ` with equality works fine. Nullary `ℝ` with inequalities works fine. **Workaround:** use `element foo : ℝ` inside the structure.
- **BUG: Evaluator substitution — compound expressions with repeated let-bound variables give wrong results inside `list_map` lambdas.** The expression `a*b - c*d` is evaluated as `((a*b) - c) * d` when operands are `fst`/`snd` applied to let-bound variables inside a `list_map` lambda. **Workaround:** decompose into separate `let` bindings.

---

### TWIN PRIME CONJECTURE — STATUS

#### Strategic evolution

1. **Path A** (comb → eigenvector delocalization → macroscopic S_N(2)) was **refuted** by
   block Jacobi elimination (see `jacobi_comb_operator.kleis`).
2. **Conrey-Keating** route formalized: RH + ratios conjecture → twin primes (two assumptions).
3. **Direct route** (contraction mixing → |P(x)| = o(x)) was **refuted** by the spectral
   comb paper's own numerical analysis (ε → 0, J_F → I, zeros decouple). March 2026.
4. **Reductio ad absurdum** attempted but **removed** — circular argument. March 2026.

#### The Conrey-Keating route (only surviving forward route)

```
RH (proved) + Ratios Conjecture (assumption — CFZ 2008)
  → Conrey-Keating 2016 theorem → Hardy-Littlewood → twin primes
```

This is a **two-assumption** result. The ratios conjecture remains an external assumption.

#### Files

| File | Purpose |
|------|---------|
| `examples/mathematics/twin_prime_correlation.kleis` | All routes (21 Z3 examples) |
| `examples/mathematics/jacobi_comb_operator.kleis` | Path A refutation |

#### Next steps

1. **Path B (ITCM):** The heat kernel / integral transform route bypasses the comb's internal structure entirely. Remaining independent forward route.
2. **Accept the conditional result:** The Conrey-Keating route is clean, well-formalized, and the only live forward chain.

#### Key technical lessons (twin primes)

- Block Jacobi elimination converts a hypothesis-failing scalar comb into an effective operator that DOES satisfy Yafaev's hypotheses
- Eigenvector localization is physical: each eigenvalue "lives" on its peak pair
- S_N(2) → 0 but is structured: spectral pair sum is a separate eigenvalue-only observable
- Skolem element mismatch in Kleis/Z3: axioms for generic elements don't propagate to concrete literals
- Bool sort mismatch: `is_prime(4) = false` fails — use `not(is_prime(4))`

---

### NEXT STEPS: Pressure Hessian Analysis

The next theorem-shaped target is to:

1. **Write down the strain evolution explicitly:**
   DS/Dt = -(S² + W²/4) - H_tf + ν∇²S (where H_tf = trace-free pressure Hessian)

2. **Isolate the off-diagonal component M₁₂ = e₂·(DS/Dt)e₁:**
   - S² contribution: 0 (diagonal in eigenframe)
   - W² contribution: -√α₁√α₂|ω|² (uses Wᵢⱼ = -εᵢⱼₖωₖ)
   - H_tf contribution: e₂·H_tf·e₁ (nonlocal, from Poisson equation)

3. **Determine the sign of M₁₂/(λ₁-λ₂):**
   - W² gives POSITIVE contribution (drives alignment UP)
   - Pressure Hessian H_tf must overcome this
   - Connects to "restricted Euler" vs "full Euler" distinction (Vieillefosse 1982)

4. **Handle eigenvalue-gap degeneracy:**
   - When λ₁ ≈ λ₂: |De₁/Dt| can diverge, but α₁ and α₂ become interchangeable
   - Need to show gap collapse events don't accumulate harmful sign

5. **Candidate formalization in Kleis:**
   - Encode W² contribution as Z3 axiom (exact)
   - Encode H_tf contribution as bounded unknown
   - Test whether known pressure Hessian bounds force Re < 0

---

### Next Paper Candidates (Volume V options)

#### Option B: Aharonov-Bohm as Kernel Non-Surjectivity

**Thesis:** On topologically nontrivial manifolds, the EM kernel d is not surjective onto closed 2-forms. The gap (H²_dR ≠ 0) produces observable effects even where F=0.

**Risk:** Moderate. Well-understood physics, clean kernel interpretation.

#### Option C: Admissibility Restoration via Additional Fields

**Thesis:** Can additional degrees of freedom restore effective admissibility to a non-admissible kernel? Derive the mechanism structurally, then identify whether it corresponds to spontaneous symmetry breaking.

**Risk:** High. Natural sequel to Volume IV.

#### Option D: Mass Gap as Topological Obstruction on the Nullspace Variety

**Seed idea (from Gemini review of Volume IV):** Reframe the mass gap from "lowest eigenvalue of QCD Hamiltonian?" to "spectral gap of the Laplacian on the nullspace variety?" Uses Cheeger inequality, Lichnerowicz theorem.

**Risk:** Very high. Requires differential geometry on the moduli space.

#### Option E: The Standard Model Gauge Sector — SU(3) × SU(2) × U(1)

**Risk:** Very high. Requires careful treatment of spontaneous symmetry breaking.

---

## Active Engineering

### HACKATHON CODE REVIEW — IN PROGRESS

**Last Updated:** April 19, 2026

### Context
Applied the HACKATHON 5-angle AI code review methodology to the full Kleis codebase. Deep Claude review found **39 findings at confidence ≥ 80**.

### Merged (PR #35 on origin, #33 on fork — branch `review/hackathon-code-review`)
13 fixes across Z3 backend, type system, evaluator, MCP servers, DAP, LSP:
- Z3 push/pop leaks in `evaluate()` and `are_equivalent()`
- Watchdog timeout for `are_equivalent()` + explicit `Unknown` handling
- `evaluate()` Sat path: explicit error when model extraction fails
- `translate_list_to_cons` panic → proper Err return
- `dynamic_to_set` type-unsoundness (verify Array range is Bool)
- `dynamic_to_string` type-unsoundness (use `Z3_is_string_sort`)
- `TypeExpr::Var` collapse to `TypeVar(0)` — unique ids per variable name
- `pretty_print_matrix` panic on empty/ragged matrices
- Path traversal defense in `save_theory` (null byte + canonicalize)
- Content-Length cap (64 MiB) in all MCP servers + DAP
- DAP ephemeral port race (pass pre-bound TcpListener)
- Redundant `STDIO_MODE.store` removal

### Current branch: `review/hackathon-code-review-2`
1 fix so far:
- `verify_axiom_impl` swallowed `ensure_structure_loaded` errors → now propagates with `?`

### Remaining findings (26 items, by tier)

**Tier 1 — Critical bugs (2):**
- #6 (Conf 88): `foldLines` arg order swapped vs documentation in `evaluator/builtins.rs`
- A (Conf 90): `check_consistency` error swallowed in `axiom_verifier.rs:649`

**Tier 2 — Security (3):**
- #10 (Conf 90): `readFile` arbitrary file read — no path restriction
- #11 (Conf 88): Unescaped import strings inject into session file
- #12 (Conf 88): `check_file` arbitrary file read — no workspace root restriction

**Tier 3 — Important bugs (8):**
- #13 (Conf 97): Lossy rational conversion via f64
- #14 (Conf 96): `bind_pattern_variables` ignores ADT field types
- #15 (Conf 96): `Type::Data` unification ignores constructor identity
- #16 (Conf 96): `check_action` schema mismatch for `git_push`
- #18 (Conf 95): Pattern guard default is `true`
- #19 (Conf 94): `check_function_def` stores body type, not arrow
- #20 (Conf 93): Unknown `TypeExpr::Named` silently becomes scalar
- #21 (Conf 90): Equality uses sort_kind not sort identity

**Tier 4 — Quality/DoS (13):**
- #23–39: Recursion depth limits, stack overflow in cons lists, parser nesting limits, operation registry silent overwrites, `declare_uninterpreted` always Int→Int, `alpha_convert` doesn't descend into Quantifier/Match, type ascription ignored, fail-open policy, unknown types default to Int sort, and more.

### Workflow
Generic command: "Pick the next unfixed finding from the Claude code review triage (Tier 1 first, then Tier 2, then Tier 3, by descending confidence). Read the code, understand the bug, fix it. Run all ~2400 tests. Then do a proper deep Claude code review — read the changed code and surrounding context yourself, apply the 5-angle HACKATHON methodology, and produce findings with confidence scores. Not just the MCP lint tool. Commit, push to both origin and fork, and update the PR."

### Key lesson
The MCP `check_code` lint tool is NOT a Claude code review. A proper review means reading the code, understanding the semantics, and producing findings with confidence scores using the 5-angle HACKATHON methodology.

---

### Future Kleis Capability: Tensor and FFT Support

The Python programs developed for the Fiber Solvability paper (`solve_3d_ising.py`, `gauge_representation_3d_ising.py`) use capabilities that Kleis should eventually support natively:

1. **Tensor operations**: The factorized transfer matrix uses a tensor product decomposition of 2×2 matrices applied to state vectors. The key trick: reshaping a 2^(N²) vector as an N²-index tensor, applying each 2×2 factor along one axis, then reshaping back. Natural for a Kleis `Tensor` structure with `contract`, `reshape`, and `kronecker` operations.

2. **DFT/FFT**: The Fourier decomposition of the transfer matrix into momentum sectors (k_x, k_y) uses the discrete Fourier transform. Kleis could support `dft(vector)` and `fft(vector)` as built-in operations on numeric arrays.

3. **Walsh-Hadamard transform**: Used for interaction-order analysis. Binary analog of the DFT, applying the Hadamard matrix H⊗H⊗...⊗H. Could be a Kleis built-in: `walsh_hadamard(vector)`.

4. **Power iteration / eigenvalue computation**: Finding the top eigenvalues of a matrix too large to diagonalize. Adding numerical linear algebra (at least `power_iteration(matrix_apply_fn, dim)`) would allow these computations to live inside `.kleis` files.

5. **Sparse/structured matrix-vector products**: The inter-layer coupling is never stored as a dense matrix — it's applied as a sequence of 2×2 operations. Kleis could support lazy/functional matrix representations where `apply(T, v)` is defined by a function, not by storing T explicitly.

**Goal:** Reproduce the 3D Ising β_c computation entirely in Kleis, with the tensor trick, FFT decomposition, and power iteration all expressed as Kleis operations verified alongside the Z3 theory.

---

### Planned: LilyPond Integration (Phase 1.5)

#### Decision (ADR-033 updated March 2026)

LilyPond cannot be compiled as a library (107k LOC monolithic CLI, deep Guile
Scheme dependency, no embedding API). Strategy: subprocess via
`render_score_svg()` built-in, feature-gated under `lilypond`. See ADR-033.

#### Implementation

- `src/evaluator/music.rs` — `render_score_svg(score)` built-in
- `Cargo.toml` — `lilypond` feature flag
- `scripts/build-kleis.sh` — LilyPond detection

---

### TODO: Integrate 3D Plotting in Kleis (plotsy-3d)

**Priority:** Medium (no urgency) — enables 3D visualization in papers, Jupyter notebooks, and REPL.

#### Context and Prototype

Prototyped 3D surface plotting using `plotsy-3d` Typst package (v0.2.1, built on CeTZ). Pure Typst/SVG output, compiles in ~1.4s.

**Prototype file:** `examples/plotting/plotsy3d_itcm_kernel.typ` — fully working, do not start from scratch.

#### Key Architectural Finding: Lilaq and plotsy-3d Cannot Compose

Lilaq uses native Typst primitives. plotsy-3d uses CeTZ. Two separate rendering stacks. However, Kleis documents can contain both as separate figures.

#### Pipeline (shared infrastructure)

```
2D: diagram(plot(...)) → PlotElement structs → generate lilaq Typst    → compile_to_svg()
3D: diagram3d(surface(...)) → [new structs]  → generate plotsy-3d Typst → compile_to_svg()
                                                                              ↓
                                                                    PLOT_SVG → Jupyter/REPL
```

#### Three Implementation Options

**Option A: Full Mirror** — New `src/plotting3d.rs`, `PlotElement3D` enum, `diagram3d()` builtin.
Pro: Cleanest API. Con: Most Rust work; lambda-to-Typst fragile.

**Option B: Thin Data Wrapper** — Pre-evaluate lambdas on grid in Rust, bake z-values into Typst.
Pro: Much less work; faster. Con: Grid resolution fixed at call time.
**This is the recommended starting point.**

**Option C: Raw Typst Escape Hatch** — `typst_svg(code_string)` builtin.
Pro: Zero Rust changes. Con: Not Kleis-native.

**Option D: Wait for Lilaq Native 3D** — Monitor [lilaq issue #31](https://github.com/lilaq-project/lilaq/issues/31).
Pro: Zero effort. Con: No timeline.

#### Gotchas from Prototyping

- `scale-dim` values must be tiny (0.01-0.05 range)
- `plotsy-3d` uses integer `range()` internally
- `subdivision-mode: "decrease"` = coarser grid, `"increase"` = finer
- Color functions receive 9 args: (x, y, z, x-lo, x-hi, y-lo, y-hi, z-lo, z-hi)
- Multiple surfaces in one scene is feasible via composable render functions

#### Decision: Deferred

Options ranked: D (wait) > B (thin wrapper) > A (full mirror) > C (raw Typst).

---

### TODO: Additional Kleis Publication Templates

Create Kleis template wrappers for major publication venues:

**Priority targets (Typst packages already exist):**
- [ ] **AMS** (`unequivocal-ams`) — American Mathematical Society style
- [ ] **Springer Nature** (`stellar-springer-nature`) — Nature, Nature Physics, etc.
- [ ] **IEEE** (`charged-ieee`) — IEEE conferences/journals
- [ ] **APS / RevTeX** (`revtyp`) — Physical Review style

**Secondary targets:**
- [ ] **Elsevier** (`elspub`)
- [ ] **NeurIPS** (`bloated-neurips`)
- [ ] **LNCS** (`fine-lncs`)
- [ ] **IOP** (`ioppub`)

---

## Reference Material

### ResearchGate DOIs

Papers published on ResearchGate with permanent DOIs:

| Paper | DOI | Date |
|-------|-----|------|
| Independence as Non-Invariance: Detecting Undecidability via Projection Fibers in SMT-Backed Shadow Theories | 10.13140/RG.2.2.22374.18243 | 2026-04-24 |
| Observable Bounds on Ontological Dimension: A Constructive Consequence of Projection Fiber Theory | 10.13140/RG.2.2.11468.99206 | 2026-04-24 |

**Next to upload:** Abstract K-Q Framework (uploaded but rate-limited, needs license/details fix), Moonlight Sonata

### Recommended Publication Order (ResearchGate)

1. ~~Independence as Non-Invariance (Projection Fibers)~~ — **DONE**
2. ~~Observable Bounds on Ontological Dimension (Fiber Dimension)~~ — **DONE**
3. The Abstract K-Q Framework — **uploaded, needs details fix**
4. The Beauty is in the Skolems (Moonlight Sonata)
5. Theory Selection and Divergence Kernels
6. Flat Galactic Rotation Curves (POT)
7. Electrodynamics as a Theorem of POT
8. Confinement as Fiber Non-Invariance (Yang-Mills)
9. Admissibility Restoration (Higgs necessity)
10. Renormalization as Projected Ontology (Volume VII)
11. Conditional Reduction of Yang-Mills Mass Gap (Volume VIII)
12. Yang-Mills Vacuum Stability (Volume IX)
13. Projection Singularities: Why Physics Has No Infinities (Volume X)
14. Quantization as Projection Kernel (Volume XI)
15. The Spectral Comb and the Riemann Hypothesis
16. Transfer Function (Hilbert-Pólya)
17. The Hum (Twin Prime Beat Structure)
18. NS Smoothness (Half-Derivative Gap)
19. NS Geometric Depletion
20. NS Bent Tubes
21. NS Dynamical Closure
22. NS Forced Localization
23. NS Unconditional Regularity (Grand Finale)
24. NS Epilogue (Kernel and the Fluid)
25. φ⁴ One-Loop
26. QED Vacuum Polarization
27. YM One-Loop Gluon Self-Energy
28. Ghost Activity Theorem
29. Gauge Dependence and Ghost Activity
30. Structural Atlas of ker(Q)
31. POT vs GR: Gravitational Lensing
32. Schanuel's Conjecture
33. Toeplitz Inscribed Square
34. Selberg Universality
35. Classical Spectral Essay (Mass Gap Epilogue)
36. Technical Brief: Realization Tautology
37. Quantum Entanglement as POT

**Rationale:** Lead with the conceptual root (fibers), then the generalization (K-Q), then the attention-grabber (music). Follow with POT foundations, the renormalization arc, RH, NS, one-loop stress tests, and remaining papers.

---

### POT PHILOSOPHICAL BOUNDARY: NON-IDENTIFIABILITY OF ONTOLOGY

**This is a non-negotiable constraint on all future papers.**

#### The Three-Part Principle

**Principle (Non-identifiability of ontology).** Observable data determine only im(Q). The pre-image is many-to-one; therefore ontology is not uniquely identifiable.

**Consequence.** Do not specify ontological dynamics (e.g., a Lagrangian for the modal flow in ontological Hilbert space). Instead, characterize the admissible structure of (K, Q).

**Structural Claim.** ker(Q) encodes the constraints discarded by projection; its internal organization is observable via its effects on im(Q), even though its elements are not.

#### ker(Q) is constrained residue, not arbitrary

From the five K-Q papers, ker(Q) is constrained by:
- **Symmetries** — gauge/Lorentz invariance → Ward/Slavnov-Taylor identities
- **Consistency of Q** — same observable must arise from equivalent pre-images
- **Compatibility with K** — Q∘K must land in im(Q) with correct invariants
- **Regularity/admissibility** — existence of convergent representatives on [0,1]

#### ker(Q) is already being studied

| Paper | What was found in ker(Q) |
|-------|--------------------------|
| φ⁴ | Contains A₀ and scheme-dependent constants. Passive. |
| QED | Ward identity shrinks ker(Q). Ghost sector present but inert. |
| Yang-Mills | Ghost sector active — shapes β₀ through Q∘K. |
| Ghost theorem | Activity iff f^{abc} ≠ 0. Algebra determines structure. |
| Gauge dependence | ker(Q) realization is representation-local; effects on im(Q) are invariant. |

#### Promote structure, not dynamics

Do not write a Lagrangian for the pre-projection modal flow. But do extract structural statements about ker(Q) that are representation-robust:
- Representation-local vs. representation-invariant
- Active vs. inert sectors
- Kernel-induced constraints

#### Natural next theorem

**Representation-Invariant Decomposition (sketch).** For representations R₁, R₂ of the same theory:
- im(Q) is identical
- the realization of ker(Q) differs
- there exists a transformation that pushes forward the constraint flow through K so that Q∘K agrees on invariants

---

### POT VUFT Series (current inventory)

| Volume | Title | Kernel | Status |
|--------|-------|--------|--------|
| I | Flat Galactic Rotation Curves from Projected Ontology | Gravitational (logarithmic Green's function) | Published |
| II | Quantum Entanglement as a Projection Artifact | Measurement (spinor projections) | Published |
| III | Electrodynamics as a Theorem of Projected Ontology | Gauge (d\|_Ω¹, admissible, nilpotent) | Complete |
| IV | Confinement as Fiber Non-Invariance | Non-admissible Yang-Mills (Lie bracket defect) | Complete |
| V | Admissibility Restoration: Structural Necessity of SSB | Restored (coupling to Higgs restoring field) | Complete |
| VI | The Kernel and the Fluid: An Epilogue | Biot-Savart (epilogue, all four forces) | Complete |
| VII | Renormalization as Projected Ontology: The Theory That Was Never Divergent | Composite (FP ∘ K_ren ∘ K_path), ITCM hypergeometric | Complete |

Each volume is independently verifiable via `kleis test`. The substrate (stdlib) is shared.

---

### Seven-paper inventory (K-Q series) + abstract framework

| # | Paper | Theory file | Paper file | Results |
|---|-------|-------------|------------|---------|
| 1 | φ⁴ one-loop | pot_phi4_oneloop.kleis | pot_phi4_oneloop_paper.kleis | 18 worked |
| 2 | QED vacuum pol. | pot_qed_vacuum_polarization.kleis | pot_qed_vacuum_polarization_paper.kleis | 15 worked |
| 3 | Yang-Mills | pot_ym_vacuum_polarization.kleis | pot_ym_vacuum_polarization_paper.kleis | 14 worked |
| 4 | Ghost theorem | pot_ghost_activity_theorem.kleis | pot_ghost_activity_theorem_paper.kleis | 17 Z3 |
| 5 | Gauge dependence | pot_gauge_dependence_ghost.kleis | pot_gauge_dependence_ghost_paper.kleis | 16 Z3 |
| 6 | Structural atlas | pot_ker_q_atlas.kleis | pot_ker_q_atlas_paper.kleis | 24 Z3 |
| 7 | Abstract K-Q framework | pot_abstract_kq_framework.kleis | pot_abstract_kq_framework_paper.kleis | 24 Z3 |

---

## Growing Brain — Distributed Expert Architecture

**Insight (April 30, 2026):** The self-growing transformer brain is not just a Kleis learner — it's a general-purpose architecture for building a *community of small expert models* that self-organize.

### The Architecture

1. **Train specialized brains independently** — one per subject area (algebra, topology, physics, music, etc.), each with its own domain-tuned tokenizer. Each brain self-grows its architecture (layers, heads, FFN) to match the complexity of its domain.

2. **Self-selecting routing** — no explicit router needed. Feed the prompt to all brains; each reports its perplexity. Low perplexity = "I understand this." The brain with the lowest score handles generation.

3. **Graceful uncertainty** — if all brains score high perplexity, the system knows it doesn't know. Signal to train a new brain. No hallucination.

4. **Independent updates** — when new information arrives in one domain, retrain only that brain. Others remain stable. No catastrophic forgetting by design.

5. **Kleis as arbitration** — when multiple brains claim knowledge (e.g., algebra and topology both recognize "fundamental group"), they generate competing completions. Kleis formalizes both as structures, Z3 checks consistency, detects subsumption or genuine contradiction.

### The Closed Loop

```
Train brains independently (knowledge production)
         ↓
Brains self-select on prompts (routing via perplexity)
         ↓
Multiple brains respond (competing claims)
         ↓
Kleis formalizes and verifies (arbitration via Z3)
         ↓
Consolidated knowledge feeds back (learning)
```

Disagreement between brains isn't a bug — it's a signal that there's a deeper connection to formalize. This is how mathematics advances: fields develop independently, then someone discovers they're isomorphic, and a unifying structure emerges.

### Why We Sleep (tongue in cheek)

Turns out nature already shipped this architecture. Two hemispheres (left: symbolic/sequential, right: spatial/holistic) are independent experts with different internal architectures, connected by a 200-million-fiber arbitration bus (corpus callosum).

And the reconciliation pass? That's sleep:

- **Awake**: Independent experts process input, accumulate competing claims
- **Sleep**: System goes offline for batch verification — replay, check consistency, merge convergent knowledge, flag contradictions
- **Wake**: "I figured it out overnight" — unified knowledge available

Sleep isn't triggered by darkness. It's triggered by the reconciliation queue hitting capacity. The drowsiness signal is: "experts have diverged beyond threshold, batch consolidation required." Motor atonia (muscle paralysis during REM) is the I/O lockout — you don't let half-merged knowledge drive actuators. Children sleep more because they accumulate more competing claims per day. Sleep deprivation causes hallucination because unreconciled claims leak into output when you skip the verification step.

We accidentally reinvented the biological brain architecture: independent self-growing experts, perplexity-based self-selection, mandatory offline arbitration with I/O lockout until consistency is restored. Nature's cron job for formal verification.

### Repositories

- [kleis-brain-v1](https://github.com/engingithub/kleis-brain-v1) — Character-level, Rust
- [kleis-brain-v2](https://github.com/engingithub/kleis-brain-v2) — Kleis-aware BPE tokenizer, Rust
- Python prototype: `examples/mathematics/growing_transformer_brain.py` (branch: `feature/growing-brain-python`)

---

## Archived

Completed session notes and paper documentation have been moved to:

**[`docs/archive/sessions/sessions-2026-feb-apr.md`](../archive/sessions/sessions-2026-feb-apr.md)**

Covers: Sessions 6-32f (Feb 23 - Apr 27, 2026), all completed POT papers (Volumes III-VII, K-Q Papers 1-7), NS regularity papers (Papers 1-5), RH papers, Music Theory, Independence Paper, GR Lensing, and engineering work.

Previous archive: [`docs/archive/sessions/sessions-dec-jan.md`](../archive/sessions/sessions-dec-jan.md)
