# Control Systems

Kleis provides a complete control systems toolkit: state-space modeling, LQR/LQG design, ODE integration, and stability analysis.

## State-Space Representation

A linear time-invariant (LTI) system is represented as:

```
ẋ = Ax + Bu    (state equation)
y = Cx + Du    (output equation)
```

In Kleis:

```kleis
// State vector x = [x₁, x₂, ..., xₙ]
// Input vector u = [u₁, u₂, ..., uₘ]
// Output vector y = [y₁, y₂, ..., yₚ]

let A = [[a11, a12], [a21, a22]] in  // n×n state matrix
let B = [[b11], [b21]] in             // n×m input matrix
let C = [[c11, c12]] in               // p×n output matrix
let D = [[0]] in                      // p×m feedthrough matrix
```

## Eigenvalue Analysis

System stability depends on eigenvalues of the A matrix:

```kleis
example "stability check" {
    let A = [[-1, 2], [0, -3]] in
    let eigs = eigenvalues(A) in
    out("Eigenvalues:", eigs)
    // If all real parts < 0, system is stable
}
```

| Eigenvalue Location | Continuous-Time | Discrete-Time |
|---------------------|-----------------|---------------|
| Left half-plane (Re < 0) | Stable | — |
| Inside unit circle (\|λ\| < 1) | — | Stable |
| On imaginary axis | Marginally stable | — |
| On unit circle | — | Marginally stable |

## Linear Quadratic Regulator (LQR)

LQR finds the optimal feedback gain K that minimizes:

```
J = ∫₀^∞ (x'Qx + u'Ru) dt
```

### Continuous-Time LQR

```kleis
example "continuous LQR" {
    // Double integrator: ẍ = u
    let A = [[0, 1], [0, 0]] in
    let B = [[0], [1]] in
    
    // Cost matrices
    let Q = [[10, 0], [0, 1]] in  // State cost
    let R = [[1]] in               // Control cost
    
    // Compute optimal gain K and Riccati solution P
    let result = lqr(A, B, Q, R) in
    let K = nth(result, 0) in
    let P = nth(result, 1) in
    
    out("Feedback gain K:", K)
    out("Riccati solution P:", P)
    
    // Closed-loop: ẋ = (A - BK)x
    let A_cl = matrix_sub(A, matmul(B, K)) in
    out("Closed-loop eigenvalues:", eigenvalues(A_cl))
}
```

### The CARE Solver

LQR requires solving the Continuous Algebraic Riccati Equation (CARE):

```
A'P + PA - PBR⁻¹B'P + Q = 0
```

Kleis uses the Schur method via LAPACK:

```kleis
let P = care(A, B, Q, R) in   // Solve CARE for P
let K = lqr(A, B, Q, R) in    // Returns [K, P]
```

## Discrete-Time Control

### Discretization

Convert continuous-time to discrete-time with sampling period Ts:

```kleis
let ts = 0.05 in  // 20 Hz sample rate

// Exact discretization: A_d = e^(A·Ts)
let A_disc = expm(scalar_matrix_mul(ts, A_cont)) in

// First-order approximation: B_d ≈ Ts·B
let B_disc = scalar_matrix_mul(ts, B_cont) in
```

### Discrete LQR (DLQR)

For discrete-time systems xₖ₊₁ = Aₓₖ + Buₖ:

```kleis
example "discrete LQR" {
    let ts = 0.05 in
    
    // Continuous system
    let A_cont = [[0, 1], [10, 0]] in  // Inverted pendulum
    let B_cont = [[0], [1]] in
    
    // Discretize
    let A_disc = expm(scalar_matrix_mul(ts, A_cont)) in
    let B_disc = scalar_matrix_mul(ts, B_cont) in
    
    // Cost matrices
    let Q = [[10, 0], [0, 1]] in
    let R = [[1]] in
    
    // Compute discrete optimal gain
    let result = dlqr(A_disc, B_disc, Q, R) in
    let K = nth(result, 0) in
    
    out("Discrete feedback gain:", K)
}
```

### The DARE Solver

DLQR requires solving the Discrete Algebraic Riccati Equation (DARE):

```
A'PA - P - (A'PB)(B'PB + R)⁻¹(B'PA) + Q = 0
```

```kleis
let P = dare(A, B, Q, R) in    // Solve DARE for P
let K = dlqr(A, B, Q, R) in    // Returns [K, P]
```

## ODE Integration

The `ode45` function integrates ODEs using the Dormand-Prince 5(4) method:

```kleis
let result = ode45(dynamics, t_span, y0, dt)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `dynamics` | `λ t y . [dy/dt]` | System dynamics function |
| `t_span` | `[t_start, t_end]` | Time interval |
| `y0` | `[y₁₀, y₂₀, ...]` | Initial state |
| `dt` | `ℝ` | Output time step |

### Example: Inverted Pendulum

```kleis
example "inverted pendulum with LQR" {
    // System parameters
    let g = 9.81 in
    let ell = 1.0 in
    
    // Linearized system (about upright equilibrium)
    let A = [[0, 1], [g/ell, 0]] in
    let B = [[0], [1/ell]] in
    
    // LQR design
    let Q = [[10, 0], [0, 1]] in
    let R = [[0.1]] in
    let result = lqr(A, B, Q, R) in
    let K = nth(result, 0) in
    let k1 = nth(nth(K, 0), 0) in
    let k2 = nth(nth(K, 0), 1) in
    
    // Nonlinear dynamics with control
    let dynamics = lambda t y .
        let theta = nth(y, 0) in
        let omega = nth(y, 1) in
        let u = neg(k1*theta + k2*omega) in
        [omega, (g/ell)*sin(theta) + u/ell]
    in
    
    // Simulate from initial angle
    let t_span = [0, 5] in
    let y0 = [0.2, 0] in  // 0.2 rad initial tilt
    let dt = 0.05 in
    
    let result = ode45(dynamics, t_span, y0, dt) in
    let times = nth(result, 0) in
    let states = nth(result, 1) in
    
    // Extract theta trajectory
    let thetas = list_map(lambda s . nth(s, 0), states) in
    
    diagram(
        plot(
            line(times, thetas, color = "blue", label = "θ (rad)"),
            xlabel = "Time (s)",
            ylabel = "Angle",
            title = "Inverted Pendulum Stabilization"
        )
    )
}
```

## Complete Example: Pendulum Stabilization

Here's a full control system design workflow:

```kleis
import "stdlib/matrices.kleis"

example "pendulum control design" {
    // Physical parameters
    let g = 9.81 in
    let ell = 1.0 in
    
    // Step 1: State-space model
    // State: [θ, ω] where θ is angle from vertical, ω is angular velocity
    // Input: u = cart acceleration
    // ẋ = Ax + Bu
    let A = [[0, 1], [g/ell, 0]] in
    let B = [[0], [neg(1/ell)]] in
    
    // Step 2: Check controllability
    // The system is controllable if rank([B, AB]) = n
    out("A matrix:", A)
    out("B matrix:", B)
    
    // Step 3: Check open-loop stability
    let open_loop_eigs = eigenvalues(A) in
    out("Open-loop eigenvalues:", open_loop_eigs)
    // One eigenvalue is positive → unstable!
    
    // Step 4: LQR design
    let Q = [[10, 0], [0, 1]] in   // Penalize angle more than velocity
    let R = [[0.1]] in              // Cheap control
    
    let lqr_result = lqr(A, B, Q, R) in
    let K = nth(lqr_result, 0) in
    let k1 = nth(nth(K, 0), 0) in
    let k2 = nth(nth(K, 0), 1) in
    
    out("LQR gain K:", K)
    
    // Step 5: Check closed-loop stability
    let A_cl = matrix_sub(A, matmul(B, K)) in
    let closed_loop_eigs = eigenvalues(A_cl) in
    out("Closed-loop eigenvalues:", closed_loop_eigs)
    // All eigenvalues have negative real parts → stable!
    
    // Step 6: Simulate
    let dynamics = lambda t y .
        let theta = nth(y, 0) in
        let omega = nth(y, 1) in
        let u = neg(k1*theta + k2*omega) in
        [omega, (g/ell)*sin(theta) + u/ell]
    in
    
    let result = ode45(dynamics, [0, 5], [0.2, 0], 0.05) in
    let times = nth(result, 0) in
    let states = nth(result, 1) in
    
    let thetas = list_map(lambda s . nth(s, 0), states) in
    let omegas = list_map(lambda s . nth(s, 1), states) in
    let controls = list_map(lambda s . neg(k1*nth(s, 0) + k2*nth(s, 1)), states) in
    
    // Step 7: Plot results
    diagram(
        plot(
            line(times, thetas, color = "blue", label = "θ (rad)"),
            line(times, omegas, color = "red", label = "ω (rad/s)"),
            line(times, controls, color = "green", label = "u (m/s²)"),
            xlabel = "Time (s)",
            ylabel = "State / Control",
            title = "Inverted Pendulum LQR Control",
            legend = "right + bottom",
            width = 14,
            height = 8
        )
    )
}
```

## Digital Control

For digital (discrete-time) control with zero-order hold:

```kleis
example "digital pendulum control" {
    let ts = 0.05 in  // 20 Hz sampling
    
    // Continuous system
    let A_cont = [[0, 1], [9.81, 0]] in
    let B_cont = [[0], [neg(1)]] in
    
    // Discretize
    let A_disc = expm(scalar_matrix_mul(ts, A_cont)) in
    let B_disc = scalar_matrix_mul(ts, B_cont) in
    
    // Discrete LQR
    let Q = [[10, 0], [0, 1]] in
    let R = [[1]] in
    let result = dlqr(A_disc, B_disc, Q, R) in
    let K = nth(result, 0) in
    
    out("Discrete gain K:", K)
    
    // Verify discrete stability
    let A_cl = matrix_sub(A_disc, matmul(B_disc, K)) in
    let eigs = eigenvalues(A_cl) in
    out("Closed-loop eigenvalues:", eigs)
    // All eigenvalues should be inside unit circle
}
```

## Stability Verification with Z3

Verify control system properties formally:

```kleis
structure StabilityProperties {
    // Continuous-time: eigenvalues have negative real parts
    axiom hurwitz_stability : ∀ λ : ℂ . 
        is_eigenvalue(A, λ) → re(λ) < 0
    
    // Discrete-time: eigenvalues inside unit circle
    axiom schur_stability : ∀ λ : ℂ .
        is_eigenvalue(A_d, λ) → abs_squared(λ) < 1
    
    // Lyapunov stability: ∃ P > 0 such that A'P + PA < 0
    axiom lyapunov_continuous : ∃ P : Matrix(n, n, ℝ) .
        positive_definite(P) ∧ 
        negative_definite(plus(matmul(transpose(A), P), matmul(P, A)))
}
```

## LAPACK Functions Reference

| Function | Description |
|----------|-------------|
| `eigenvalues(A)` | Compute eigenvalues of matrix A |
| `eigenvectors(A)` | Compute eigenvalues and eigenvectors |
| `svd(A)` | Singular value decomposition |
| `expm(A)` | Matrix exponential e^A |
| `care(A, B, Q, R)` | Solve continuous algebraic Riccati equation |
| `dare(A, B, Q, R)` | Solve discrete algebraic Riccati equation |
| `lqr(A, B, Q, R)` | Continuous LQR design (returns [K, P]) |
| `dlqr(A, B, Q, R)` | Discrete LQR design (returns [K, P]) |

See [LAPACK Functions appendix](../appendix/lapack.md) for complete documentation.

## What's Next

Explore more examples:

→ [ODE Solver appendix](../appendix/ode-solver.md) — detailed ode45 documentation  
→ [LAPACK Functions](../appendix/lapack.md) — numerical linear algebra  
→ [Cartan Geometry](../appendix/cartan-geometry.md) — differential geometry  
→ [Next: Interactive Theory Building](./27-theory-building.md) — AI-assisted formal theory development

