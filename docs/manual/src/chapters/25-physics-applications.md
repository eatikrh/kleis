# Physics Applications

Kleis includes extensive physics libraries. This chapter showcases what's possible.

## Electromagnetism: Maxwell's Equations

The `maxwell.kleis` module expresses electromagnetism in covariant (tensor) form.

### The Field Tensor

The electromagnetic field tensor F_μν is antisymmetric and encodes both E and B fields:

```kleis
structure FieldTensorProperties {
    operation F : Nat → Nat → ℝ
    
    // Antisymmetry: F_μν = -F_νμ
    axiom F_antisymmetric : ∀ mu : Nat . ∀ nu : Nat .
        F(mu, nu) = negate(F(nu, mu))
    
    // Diagonal vanishes
    axiom F_diagonal_zero : ∀ mu : Nat . F(mu, mu) = 0
}
```

### Maxwell's Equations

The four Maxwell equations reduce to two tensor equations:

```kleis
structure MaxwellInhomogeneous {
    operation F : Nat → Nat → ℝ  // Field tensor
    operation J : Nat → ℝ         // 4-current
    operation divF : Nat → ℝ      // Divergence of F
    element mu0 : ℝ               // Permeability
    
    // ∂_μ F^μν = μ₀ J^ν
    // This encodes Gauss's law (ν=0) and Ampère's law (ν=i)
    axiom maxwell_inhomogeneous : ∀ nu : Nat .
        divF(nu) = times(mu0, J(nu))
}

structure MaxwellHomogeneous {
    operation F : Nat → Nat → ℝ
    operation cyclicF : Nat → Nat → Nat → ℝ  // ∂_[λ F_μν]
    
    // ∂_λ F_μν + ∂_μ F_νλ + ∂_ν F_λμ = 0
    // This encodes no magnetic monopoles and Faraday's law
    axiom maxwell_homogeneous : ∀ lam : Nat . ∀ mu : Nat . ∀ nu : Nat .
        cyclicF(lam, mu, nu) = 0
}
```

### Einstein-Maxwell

Electromagnetism in curved spacetime (charged black holes):

```kleis
structure EinsteinMaxwell {
    operation g : Nat → Nat → ℝ      // Metric
    operation G : Nat → Nat → ℝ      // Einstein tensor
    operation F : Nat → Nat → ℝ      // EM field tensor
    operation T_EM : Nat → Nat → ℝ   // EM stress-energy
    
    element Lambda : ℝ   // Cosmological constant
    element kappa : ℝ    // 8πG/c⁴
    
    // Einstein-Maxwell field equations:
    // G_μν + Λg_μν = κ T^EM_μν
    axiom einstein_maxwell_field_eqn : ∀ mu : Nat . ∀ nu : Nat .
        plus(G(mu, nu), times(Lambda, g(mu, nu))) = times(kappa, T_EM(mu, nu))
}
```

## Fluid Dynamics: Navier-Stokes

The `fluid_dynamics.kleis` module covers incompressible and compressible flow.

### Continuity Equation

Mass conservation:

```kleis
structure ContinuityEquation {
    element drho_dt : ℝ      // ∂ρ/∂t
    element div_rho_u : ℝ    // ∇·(ρv)
    
    // ∂ρ/∂t + ∇·(ρv) = 0
    axiom continuity : plus(drho_dt, div_rho_u) = 0
}
```

### Momentum Equation

The full Navier-Stokes momentum equation:

```
structure MomentumEquation {
    element rho : ℝ
    operation du_dt : Nat → ℝ
    operation div_momentum : Nat → ℝ
    operation grad_p : Nat → ℝ
    operation div_tau : Nat → ℝ
    operation f : Nat → ℝ
    
    // Navier-Stokes momentum equation
    axiom momentum : ∀ i : Nat .
        plus(times(rho, du_dt(i)), div_momentum(i)) = 
        plus(plus(negate(grad_p(i)), div_tau(i)), times(rho, f(i)))
}
```

### Special Cases

**Incompressible flow** (∇·v = 0):

```kleis
structure IncompressibleFlow {
    element div_u : ℝ
    axiom incompressible : div_u = 0
}
```

**Stokes flow** (creeping flow, Re << 1):

```kleis
structure StokesFlow {
    operation grad_p : Nat → ℝ
    operation laplacian_u : Nat → ℝ
    element mu : ℝ
    
    // ∇p = μ∇²v (inertia negligible)
    axiom stokes : ∀ i : Nat .
        grad_p(i) = times(mu, laplacian_u(i))
}
```

**Euler equations** (inviscid, μ = 0):

```kleis
structure EulerEquations {
    element rho : ℝ
    operation du_dt : Nat → ℝ
    operation convective : Nat → ℝ
    operation grad_p : Nat → ℝ
    
    // ρ∂v/∂t + ρ(v·∇)v = -∇p
    axiom euler : ∀ i : Nat .
        plus(times(rho, du_dt(i)), times(rho, convective(i))) = negate(grad_p(i))
}
```

### Bernoulli's Equation

For steady, inviscid, incompressible flow along a streamline:

```
structure BernoulliEquation {
    element rho : ℝ
    element g : ℝ
    element p1 : ℝ
    element p2 : ℝ
    element v1 : ℝ
    element v2 : ℝ
    element h1 : ℝ
    element h2 : ℝ
    
    // p + ½ρv² + ρgh = constant
    // Scaled: 2p + ρv² + 2ρgh = constant
    axiom bernoulli_conservation : 
        plus(plus(times(2, p1), times(rho, times(v1, v1))),
             times(times(2, rho), times(g, h1))) =
        plus(plus(times(2, p2), times(rho, times(v2, v2))),
             times(times(2, rho), times(g, h2)))
}
```

## Solid Mechanics

The `solid_mechanics.kleis` module covers stress, strain, and failure criteria.

### Stress and Strain Tensors

```kleis
structure StressTensorSymmetry {
    operation sigma : Nat → Nat → ℝ
    
    // σ_ij = σ_ji (angular momentum balance)
    axiom stress_symmetric : ∀ i : Nat . ∀ j : Nat .
        sigma(i, j) = sigma(j, i)
}

structure ElasticityTensorSymmetries {
    // Fourth-order elasticity tensor: σ_ij = C_ijkl ε_kl
    operation C : Nat → Nat → Nat → Nat → ℝ
    
    // Major symmetry (strain energy)
    axiom C_major_symmetry : ∀ i j k l : Nat .
        C(i, j, k, l) = C(k, l, i, j)
    
    // Minor symmetries (stress/strain symmetry)
    axiom C_minor_symmetry_1 : ∀ i j k l : Nat .
        C(i, j, k, l) = C(j, i, k, l)
}
```

### Yield Criteria

**Von Mises** (ductile metals):

```
structure VonMisesYieldCriterion {
    element sigma1 : ℝ
    element sigma2 : ℝ
    element sigma3 : ℝ
    element vm_sq_2 : ℝ
    
    // 2σ_vm² = (σ₁-σ₂)² + (σ₂-σ₃)² + (σ₃-σ₁)²
    axiom von_mises_def : vm_sq_2 = plus(plus(
        times(minus(sigma1, sigma2), minus(sigma1, sigma2)),
        times(minus(sigma2, sigma3), minus(sigma2, sigma3))),
        times(minus(sigma3, sigma1), minus(sigma3, sigma1)))
}
```

**Mohr-Coulomb** (soils and rocks):

```kleis
structure MohrCoulombCriterion {
    element sigma_n : ℝ  // Normal stress
    element c : ℝ        // Cohesion
    element phi : ℝ      // tan(friction angle)
    element tau_critical : ℝ
    
    // |τ| = c + σ_n tan(φ)
    axiom mohr_coulomb : tau_critical = plus(c, times(sigma_n, phi))
}
```

## Quantum Mechanics

The `quantum.kleis` module provides Hilbert space formalism.

### State Vectors

```kleis
structure Ket(dim: Nat, T) {
    operation normalize : Ket(dim, T) → Ket(dim, T)
    operation scale : T → Ket(dim, T) → Ket(dim, T)
}

structure Bra(dim: Nat, T) {
    operation conjugate : Ket(dim, T) → Bra(dim, T)
}
```

### Inner Product

```kleis
structure InnerProduct(dim: Nat) {
    operation inner : Bra(dim, ℂ) → Ket(dim, ℂ) → ℂ
    
    // ⟨φ|ψ⟩ is the probability amplitude
    // |⟨φ|ψ⟩|² is the probability
}
```

### Operators

```kleis
structure Operator(dim: Nat, T) {
    operation apply : Operator(dim, T) → Ket(dim, T) → Ket(dim, T)
    operation adjoint : Operator(dim, T) → Operator(dim, T)
    operation compose : Operator(dim, T) → Operator(dim, T) → Operator(dim, T)
}

structure Commutator(dim: Nat, T) {
    operation commutator : Operator(dim, T) → Operator(dim, T) → Operator(dim, T)
    
    // [Â, B̂] = ÂB̂ - B̂Â
    // [x̂, p̂] = iℏ (Heisenberg uncertainty!)
}
```

### Pauli Matrices

```kleis
structure PauliMatrices(T) {
    operation sigma_x : Operator(2, ℂ)  // |0⟩⟨1| + |1⟩⟨0|
    operation sigma_y : Operator(2, ℂ)  // -i|0⟩⟨1| + i|1⟩⟨0|
    operation sigma_z : Operator(2, ℂ)  // |0⟩⟨0| - |1⟩⟨1|
    
    // σ_x² = σ_y² = σ_z² = I
    // [σ_x, σ_y] = 2iσ_z
}
```

### Time Evolution

```kleis
structure TimeEvolution(dim: Nat) {
    // |ψ(t)⟩ = e^(-iĤt/ℏ) |ψ(0)⟩
    operation evolve : Operator(dim, ℂ) → ℝ → Ket(dim, ℂ) → Ket(dim, ℂ)
    operation propagator : Operator(dim, ℂ) → ℝ → Operator(dim, ℂ)
}
```

## Cosmology

The `cosmology.kleis` module defines standard spacetimes.

### Minkowski (Flat Space)

```kleis
structure MinkowskiSpacetime {
    operation g : Nat → Nat → ℝ
    
    // All curvature vanishes
    axiom minkowski_ricci_vanish : ∀ mu nu : Nat . Ric(mu, nu) = 0
    axiom minkowski_einstein_vanish : ∀ mu nu : Nat . G(mu, nu) = 0
}
```

### de Sitter (Accelerating Universe)

```kleis
structure DeSitterSpacetime {
    operation g : Nat → Nat → ℝ
    operation G : Nat → Nat → ℝ
    element Lambda : ℝ
    
    axiom positive_lambda : Lambda = 1  // Λ > 0
    
    // Vacuum: G_μν = -Λg_μν
    axiom desitter_einstein : ∀ mu nu : Nat .
        G(mu, nu) = negate(times(Lambda, g(mu, nu)))
}
```

### Schwarzschild (Black Hole)

```kleis
structure SchwarzschildSpacetime {
    operation g : Nat → Nat → ℝ
    operation G : Nat → Nat → ℝ
    element M : ℝ  // Black hole mass
    
    // Vacuum with Λ=0
    axiom schwarzschild_vacuum : ∀ mu nu : Nat . G(mu, nu) = 0
}
```

See the [Cartan Geometry appendix](../appendix/cartan-geometry.md) for computing the actual Schwarzschild curvature tensor.

### FLRW (Cosmology)

```kleis
structure FLRWCosmology {
    operation g : Nat → Nat → ℝ
    operation T : Nat → Nat → ℝ  // Stress-energy (perfect fluid)
    
    element Lambda : ℝ   // Cosmological constant
    element rho : ℝ      // Energy density
    element p : ℝ        // Pressure
    
    // Perfect fluid: T_00 = ρ
    axiom perfect_fluid_energy : T(0, 0) = rho
    
    // Einstein field equations
    axiom field_equations : ∀ mu nu : Nat .
        plus(G(mu, nu), times(Lambda, g(mu, nu))) = times(kappa, T(mu, nu))
}
```

## Differential Forms

The `differential_forms.kleis` module provides Cartan calculus:

```kleis
// Wedge product: α ∧ β
structure WedgeProduct(p: Nat, q: Nat, dim: Nat) {
    operation wedge : DifferentialForm(p, dim) → DifferentialForm(q, dim) 
                    → DifferentialForm(p + q, dim)
    
    axiom graded_antisymmetric : ∀ α β .
        wedge(α, β) = scale(power(-1, p*q), wedge(β, α))
}

// Exterior derivative: dα
structure ExteriorDerivative(p: Nat, dim: Nat) {
    operation d : DifferentialForm(p, dim) → DifferentialForm(p + 1, dim)
    
    axiom d_squared_zero : ∀ α . d(d(α)) = 0  // d² = 0
}

// Hodge star: ⋆α
structure HodgeStar(p: Nat, dim: Nat) {
    operation star : DifferentialForm(p, dim) → DifferentialForm(dim - p, dim)
    
    axiom hodge_involutive : ∀ α .
        star(star(α)) = scale(power(-1, p*(dim-p)), α)
}
```

### Cartan's Magic Formula

The Lie derivative ℒ_X connects all operations:

```kleis
// ℒ_X = d ∘ ι_X + ι_X ∘ d
define cartan_magic_impl(X, alpha) = 
    plus(d(interior(X, alpha)), interior(X, d(alpha)))
```

## What's Next

Apply these physics structures with numerical methods:

→ [Next: Control Systems](./26-control-systems.md)

