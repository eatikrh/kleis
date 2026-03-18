# GL(3) Zero Computation Pipeline

SageMath/Pari/GP scripts used to compute zeros of L(s, Sym²Δ),
the Gelbart-Jacquet lift of Ramanujan Delta from GL(2) to GL(3).

## Scripts (in order of development)

### `sym2_highprec.sage`
First successful computation. Computes tau(n) via iterative eta product
expansion, builds Sym² Dirichlet coefficients with prime power recurrence,
and calls Pari/GP `lfunzeros`. Found 2 zeros with 2000 coefficients.

### `gl3_zeros_v2.sage`
Refined version using Sage's built-in `CuspForms(1, 12)` for tau(n).
3000 coefficients, proper prime power handling. Confirms the 2 zeros
with better FE agreement.

### `contraction_zeros.sage`
Tests the spectral comb's Banach contraction iteration as a zero-finder.
Starts from rough estimates and iterates the comb map F. Key finding:
Re = 1/2 holds to machine precision at every iteration for all N tested.

### `pinned_iteration.sage`
Pins the 2 known Pari/GP zeros and iterates remaining entries.
Includes smooth-zero failure test (278× degradation) and antisymmetry
cliff test (10⁻¹⁶ → -33).

## Requirements

- SageMath 10.8+
- Pari/GP 2.17+ (included with SageMath)

## Results

| Zero   | Value (57-digit) |
|--------|-------------------|
| γ₁     | 5.70568488235770752285491825667482085898052721243775351582 |
| γ₂     | 8.18249171274299092495417383539430774353178543388505629967 |

Langlands parameters: {-11, 0, 11}, weight 23, conductor 1, ε = +1.
Functional equation verified to 10⁻⁶ agreement.
