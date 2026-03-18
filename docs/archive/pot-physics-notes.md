# POT Physics Research Notes

*Archived from NEXT_SESSION.md on Feb 26, 2026*

Covers: Flat rotation curves, philosophical principles, future research directions,
entanglement formalization, neutrino oscillations, special relativity,
Schwarzschild metric, spectral residues, admissible kernel class.

---

### Related Work to Review

- **François, J. & Ravera, L. (2025).** "Raising galaxy rotation curves via dressing." Phys. Rev. D 112(8). DOI: 10.1103/m9xl-9vvk
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
