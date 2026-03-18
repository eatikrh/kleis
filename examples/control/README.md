# Control Theory Examples

Kleis examples demonstrating control system analysis and design.

## Files

| File | Description |
|------|-------------|
| `eigenvalues.kleis` | Closed-form eigenvalue formulas for 2×2 and 3×3 matrices |
| `lqg_controller.kleis` | LQG (Linear Quadratic Gaussian) controller design |

## Eigenvalue Computation

For small matrices, eigenvalues can be computed **exactly** using closed-form solutions:

### 2×2 Matrix (Quadratic Formula)

For matrix $A = \begin{bmatrix} a & b \\ c & d \end{bmatrix}$:

$$\lambda_{1,2} = \frac{\text{tr}(A) \pm \sqrt{\text{tr}(A)^2 - 4\det(A)}}{2}$$

where:
- $\text{tr}(A) = a + d$ (trace)
- $\det(A) = ad - bc$ (determinant)

**Stability (Hurwitz criterion):** System is stable iff $\text{tr}(A) < 0$ and $\det(A) > 0$

### 3×3 Matrix (Cardano's Formula)

Characteristic polynomial: $\lambda^3 - p\lambda^2 + q\lambda - r = 0$

Solved using Cardano's method with depressed cubic transformation.

**Stability (Routh-Hurwitz):** System is stable iff $p > 0$, $r > 0$, and $pq > r$

## LQG Controller Design

The `lqg_controller.kleis` example demonstrates the complete LQG design flow:

1. **LQR (Linear Quadratic Regulator)** - Optimal state feedback
   - Minimizes cost $J = \int_0^\infty (x^T Q x + u^T R u) \, dt$
   - Solves the Control Algebraic Riccati Equation (CARE)
   - Gain: $K = R^{-1} B^T P$

2. **Kalman Filter** - Optimal state estimation  
   - Minimizes estimation error covariance
   - Solves the Filter Algebraic Riccati Equation (FARE)
   - Gain: $L = S C^T V^{-1}$

3. **LQG Combination** - Separation principle
   - Design LQR and Kalman independently
   - Combined controller: $\dot{\hat{x}} = (A - BK - LC)\hat{x} + Ly$, $u = -K\hat{x}$

4. **LTR (Loop Transfer Recovery)** - Robustness recovery
   - Recovers full-state feedback robustness margins

## Control Applications

These eigenvalue formulas are fundamental for:

1. **Stability Analysis** - Check if all eigenvalues have $\text{Re}(\lambda) < 0$
2. **Pole Placement** - Design feedback to place eigenvalues at desired locations  
3. **LQR Design** - Optimal control minimizing quadratic cost
4. **Observer Design** - Kalman filter for state estimation

## Future Examples

- `pole_placement.kleis` - Ackermann's formula for pole placement
- `controllability.kleis` - Controllability and observability analysis
- `robust_control.kleis` - H∞ and μ-synthesis

## Running Examples

```bash
# Parse and type-check
cargo run --bin kleis -- check examples/control/eigenvalues.kleis

# Verify axioms
cargo run --bin kleis -- verify examples/control/eigenvalues.kleis
```

