---
name: ""
overview: ""
todos: []
isProject: false
---

# Paper 8: One Field, Two Projections — The Classical-Quantum Kernel Reach Structure

## Thesis

For any physical phenomenon admitting both classical and quantum descriptions,
there exist kernels K_cl and K_qu acting on the **same** modal flow field such
that ker(K_qu) ⊆ ker(K_cl). The classical/quantum divide is a property of the
projection, not the ontology.

## Motivation

The seven K-Q papers catalogued kernels across domains: Feynman integrals,
exterior derivative, Biot-Savart, logarithmic Green's function, spinor
projection. The abstract framework (Paper 7) unified them. But a key
observation emerged: classical and quantum descriptions of the same phenomenon
(e.g., Maxwell/QED) must not both appear as separate fields in the modal flow —
that double-counts. One field, two kernels.

This forces:

- ker(K_qu) ⊆ ker(K_cl) — quantum reaches more of the source
- ker(K_cl) \ ker(K_qu) = the "classically invisible, quantum-activated" sector
- Gap inheritance: gap(K_cl) ⊆ gap(K_qu)
- Quantization is not a procedure — it is a finer kernel applied to the same source

## Sections

### 1. Introduction: The No-Double-Counting Constraint

- The modal flow in H_ont cannot contain both classical and quantum fields for
the same phenomenon
- This forces a single source field with multiple (K, Q) projections
- The classical/quantum distinction moves from ontology to observation

### 2. The Kernel Inclusion Theorem

- **Definition**: K_cl and K_qu are a classical-quantum pair if they act on the
same source field and Q_cl ∘ K_cl, Q_qu ∘ K_qu produce respectively the
classical and quantum observables of the same phenomenon
- **Theorem**: ker(K_qu) ⊆ ker(K_cl)
(everything the quantum kernel cannot see, the classical kernel also cannot see)
- **Definition**: The quantum-activated sector Δ = ker(K_cl) \ ker(K_qu)
- **Corollary**: im(K_cl) ⊆ im(K_qu) — the quantum image is strictly richer

### 3. Gap Inheritance

- **Theorem**: gap(K_cl, Q_cl) ⊆ gap(K_qu, Q_qu)
(classical gaps are subsets of quantum gaps)
- Proof from kernel inclusion + the three-space decomposition
- Classical theories have smaller gaps because K_cl reaches less of the source
- The six atlas types live in gap(K_qu) but not (all) in gap(K_cl)

### 4. Instantiation: Electromagnetism / QED

- Source field: (A_μ, ψ) — U(1) connection + charged spinor
- K_cl = exterior derivative d: acts on A_μ, produces F_μν. ψ ∈ ker(K_cl).
- K_qu = Feynman kernel: acts on both A_μ and ψ, produces QED amplitudes
- Quantum-activated sector Δ = {ψ} — the electron field
- Classical EM: empty gap (from Paper 7). QED: rich gap (six types, from Paper 6)
- The electron was always in the modal flow; the classical kernel couldn't see it

### 5. Instantiation: Gravity

- Source field: (g_μν, matter fields)
- K_cl = linearized Green's function: produces gravitational waves from metric perturbations
- K_qu = graviton propagator (if it exists): would activate matter-gravity loops
- Quantum-activated sector: matter fields as dynamical contributors to
spacetime geometry at loop level
- Non-admissibility of full GR: the curvature self-coupling K_YM = d + [·,·]
analogy persists

### 6. Instantiation: Fluids

- Source field: velocity field + vorticity
- K_cl = Biot-Savart: classical fluid dynamics
- K_qu = quantum fluid kernel: Bose-Einstein condensate, superfluid order parameter
- Quantum-activated sector: phase coherence (classically invisible)
- Test: does ker(K_cl) \ ker(K_qu) contain the superfluid order parameter?

### 7. The Minimum Field Content Question

- For each domain, Δ = ker(K_cl) \ ker(K_qu) catalogues exactly what
quantization "activates"
- Across all domains, the union of Δ's constrains the minimum field content
of the modal flow
- This is a concrete, answerable question — not ontological speculation
- The codomain dimension of the modal flow = dim(im(Q_qu)) + dim(gap(K_qu))
where gap is the largest (quantum) gap

### 8. The Philosophical Payoff

- Quantization is not a procedure applied to a classical theory
- It is a finer kernel applied to the same source
- The classical/quantum divide lives in (K, Q), not in H_ont
- This is consistent with POT's non-identifiability principle: the ontology
(the source field) is one thing; the variety of physics comes from the
variety of projections
- Does NOT violate the "no Lagrangian for the modal flow" boundary — we
characterize what K must reach, not what the source dynamics are

### 9. Open Questions

- Is ker(K_qu) ⊆ ker(K_cl) always strict, or can they be equal?
(If equal: the phenomenon has no quantum-activated sector — purely classical)
- Does the quantum-activated sector Δ determine the gap structure uniquely?
- Is there a "maximally fine" kernel K_max such that ker(K_max) is minimal?
What would that be?
- Connection to decoherence: is the classical limit K_cl obtained from K_qu
by restricting kernel reach (a coarsening)?

## Implementation Plan

### Step 1: Theory file — `theories/pot_classical_quantum_kernel_reach.kleis`

Target: ~24 Z3-verified axioms across 5-6 structures:

1. **KernelInclusionAxioms** (~4 axioms)
  - K_cl, K_qu as kernels on same source
  - ker(K_qu) ⊆ ker(K_cl)
  - Quantum-activated sector Δ nonempty
  - im(K_cl) ⊆ im(K_qu)
2. **GapInheritance** (~4 axioms)
  - gap(K_cl) ⊆ gap(K_qu)
  - Classical gap ≤ quantum gap in dimension
  - Six atlas types present in quantum gap
  - Classical gap contains at most gauge orbits
3. **EMInstantiation** (~4 axioms)
  - K_cl = d, K_qu = Feynman kernel on (A, ψ)
  - ψ ∈ ker(K_cl), ψ ∉ ker(K_qu)
  - Δ_EM = {ψ}
  - gap_cl(EM) empty, gap_qu(QED) rich
4. **GravityInstantiation** (~4 axioms)
  - K_cl = linearized Green's fn
  - Quantum-activated: matter at loop level
  - Non-admissibility persists from Paper 7
5. **FluidInstantiation** (~4 axioms)
  - K_cl = Biot-Savart
  - Quantum-activated: phase coherence / order parameter
6. **MinimumFieldContent** (~4 axioms)
  - Union of Δ across domains bounds modal flow
  - Codomain dimension = observable + quantum gap
  - The modal flow field content is constrained by Δ-union
  - Quantization = kernel refinement, not ontological upgrade

### Step 2: Paper file — `examples/ontology/revised/pot_classical_quantum_kernel_reach_paper.kleis`

9 sections following the outline above.
All theorems and definitions properly typeset.
References to Papers 1-7 plus standard physics references.

### Step 3: Compile and verify

- `kleis test theories/pot_classical_quantum_kernel_reach.kleis`
- `kleis test examples/ontology/revised/pot_classical_quantum_kernel_reach_paper.kleis`
- Extract Typst, compile PDF

### Step 4: Deploy

- Copy to `docs/papers/`
- Update NEXT_SESSION.md

