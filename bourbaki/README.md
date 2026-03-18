# Bourbaki's Éléments de Mathématique in Kleis

This directory contains formalizations of the 9 volumes of Nicolas Bourbaki's
*Éléments de mathématique* in the Kleis formal verification language.

## Volumes

| File | Volume | Title | Structures |
|------|--------|-------|------------|
| `01_sets.kleis` | I | Theory of Sets | 10 |
| `02_algebra.kleis` | II | Algebra | 17 |
| `03_topology.kleis` | III | General Topology | 17 |
| `04_analysis.kleis` | IV | Functions of a Real Variable | 10 |
| `05_tvs.kleis` | V | Topological Vector Spaces | 14 |
| `06_integration.kleis` | VI | Integration | 11 |
| `07_commutative.kleis` | VII | Commutative Algebra | 15 |
| `08_lie.kleis` | VIII | Lie Groups and Lie Algebras | 22 |
| `09_spectral.kleis` | IX | Spectral Theory | 13 |

**Total: 129 structures**

## Key Concepts Formalized

### Volume I: Theory of Sets
- Set operations (∪, ∩, complement, power set)
- Extensionality axiom
- Ordered pairs and Cartesian products
- Equivalence relations
- Partial, total, and well-orders
- Cardinals and natural numbers (Peano axioms)

### Volume II: Algebra
- Magma, semigroup, monoid, group, abelian group
- Ring, commutative ring, integral domain, field
- Modules and vector spaces
- Tensor products, exterior and symmetric algebras
- Chain complexes and homology

### Volume III: General Topology
- Topological spaces (open sets axioms)
- Continuous functions and homeomorphisms
- Uniform spaces
- Topological groups
- Separation axioms (T0, T1, T2/Hausdorff, T3/Regular, T4/Normal)
- Compactness and connectedness

### Volume IV: Functions of a Real Variable
- Derivatives (ε-δ definition)
- Product rule, chain rule
- Riemann integration
- Fundamental Theorem of Calculus
- Limits and continuity
- Sequences, series, and convergence
- Taylor series
- Elementary functions (exp, ln, sin, cos)

### Volume V: Topological Vector Spaces
- Topological vector spaces
- Locally convex spaces
- Seminorms and norms
- Banach spaces
- Inner product spaces and Hilbert spaces
- Dual spaces and weak topology
- Hahn-Banach theorem

### Volume VI: Integration
- Measure spaces and σ-algebras
- Lebesgue measure
- Lebesgue integration
- Monotone and dominated convergence theorems
- Lp spaces
- Product measures and Fubini's theorem
- Radon-Nikodym theorem

### Volume VII: Commutative Algebra
- Flat modules
- Localization
- Graded and filtered rings
- Completions
- Prime ideals
- Integral extensions
- Dedekind domains
- Valuations and divisors

### Volume VIII: Lie Groups and Lie Algebras
- Lie algebras (bracket, Jacobi identity)
- Subalgebras and ideals
- Nilpotent, solvable, semisimple Lie algebras
- Representations and adjoint representation
- Killing form
- Lie groups and exponential map
- Compact Lie groups
- Root systems and Weyl groups
- Cartan matrices and Dynkin diagrams
- Classification theorem

### Volume IX: Spectral Theory
- Spectrum of an element
- Resolvent function
- Point, continuous, and residual spectrum
- Self-adjoint operators
- Spectral theorem
- Functional calculus
- Compact operators
- Unbounded operators

## Verification

All files parse successfully with Kleis Grammar v0.9:

```bash
for f in bourbaki/*.kleis; do
  kleis --check "$f"
done
# All 9 files: ✅ Syntax OK
```

## Limitations

These formalizations are **axiomatic** - they define the structures and their
properties, but do not provide computational implementations. Z3 can reason
about these axioms to verify properties, but cannot:

- Perform inductive proofs
- Verify limit computations
- Evaluate specific instances

See `docs/CAPABILITY_ASSESSMENT.md` for a detailed analysis of Kleis's
mathematical expressive power.

## Date

Created: December 19, 2025
Grammar Version: Kleis v0.9

