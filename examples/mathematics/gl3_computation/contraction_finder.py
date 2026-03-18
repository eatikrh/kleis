import numpy as np

def build_comb_matrix(zeros_list):
    nz = len(zeros_list)
    n = 2 * nz
    mean_gamma = np.mean(zeros_list)
    eps = 2.0 * np.pi / mean_gamma
    H = np.zeros((n, n))
    for i in range(n):
        H[i, i] = 0.5
    off = []
    for k in range(nz):
        off.append(zeros_list[k])
        if k < nz - 1:
            off.append(eps)
    for j in range(len(off)):
        H[j][j+1] = off[j]
        H[j+1][j] = -off[j]
    return H

def comb_map(gammas):
    H = build_comb_matrix(gammas)
    evals = np.linalg.eig(H)[0]
    pos_evals = sorted([e for e in evals if e.imag > 0.01], key=lambda x: x.imag)
    if len(pos_evals) < len(gammas):
        return None, None
    return np.array([e.imag for e in pos_evals[:len(gammas)]]), \
           np.array([e.real for e in pos_evals[:len(gammas)]])

# Known zeta zeros (Odlyzko)
true_5 = np.array([14.1347, 21.0220, 25.0109, 30.4249, 32.9351])
true_10 = np.array([14.1347, 21.0220, 25.0109, 30.4249, 32.9351,
                     37.5862, 40.9187, 43.3271, 48.0052, 49.7738])

# Smooth zero estimates from N_0(T) = (T/2pi)*log(T/2pi*e) + 7/8
# Invert: find T_n such that N_0(T_n) = n
def smooth_zero_estimate(n_zeros):
    """Generate smooth estimates for the first n zeros of zeta(s)."""
    estimates = []
    for n in range(1, n_zeros + 1):
        # Rough inversion of N_0(T) ~ (T/2pi)*log(T/2pi)
        # Start with T ~ 2*pi*n/log(n) for n > 1
        if n == 1:
            T = 10.0  # rough
        else:
            T = 2 * np.pi * n / np.log(n)
        # Newton refinement
        for _ in range(20):
            if T < 1: T = 1.0
            N0 = (T / (2*np.pi)) * np.log(T / (2*np.pi * np.e)) + 7/8
            dN0 = np.log(T / (2*np.pi)) / (2*np.pi)
            if abs(dN0) > 1e-10:
                T = T - (N0 - n) / dN0
        estimates.append(max(T, 1.0))
    return np.array(estimates)

print("=" * 70)
print("CONTRACTION MAPPING AS ZERO-FINDER: RIEMANN ZETA")
print("=" * 70)

# ===================================================================
# Experiment 1: N=5, start from smooth estimates
# ===================================================================
print("\n--- N=5: Start from smooth estimates ---")
smooth_5 = smooth_zero_estimate(5)
print(f"Smooth estimates: {smooth_5}")
print(f"True zeros:       {true_5}")
print(f"Initial error:    {np.abs(smooth_5 - true_5)}")
print(f"Initial max err:  {np.max(np.abs(smooth_5 - true_5)):.4f}")

gammas = smooth_5.copy()
for i in range(200):
    new_g, re_parts = comb_map(gammas)
    if new_g is None:
        print(f"Iter {i}: eigenvalue extraction failed")
        break
    diff = np.max(np.abs(new_g - gammas))
    err_vs_true = np.max(np.abs(new_g - true_5))
    if i < 10 or i % 20 == 0 or diff < 1e-14:
        print(f"Iter {i:3d}: {np.array2string(new_g, precision=4)}  "
              f"change={diff:.2e}  err_vs_true={err_vs_true:.4f}")
    gammas = new_g
    if diff < 1e-15:
        print(f"Converged at iteration {i}")
        break

print(f"\nFinal:  {gammas}")
print(f"True:   {true_5}")
print(f"Error:  {np.abs(gammas - true_5)}")
print(f"Max error vs true zeros: {np.max(np.abs(gammas - true_5)):.6f}")

# ===================================================================
# Experiment 2: N=10, start from smooth estimates
# ===================================================================
print("\n--- N=10: Start from smooth estimates ---")
smooth_10 = smooth_zero_estimate(10)
print(f"Smooth estimates: {np.array2string(smooth_10, precision=2)}")
print(f"True zeros:       {np.array2string(true_10, precision=2)}")

gammas = smooth_10.copy()
for i in range(200):
    new_g, re_parts = comb_map(gammas)
    if new_g is None:
        print(f"Iter {i}: failed")
        break
    diff = np.max(np.abs(new_g - gammas))
    err_vs_true = np.max(np.abs(new_g - true_10))
    if i < 10 or i % 20 == 0 or diff < 1e-14:
        print(f"Iter {i:3d}: change={diff:.2e}  max_err_vs_true={err_vs_true:.4f}")
    gammas = new_g
    if diff < 1e-15:
        print(f"Converged at iteration {i}")
        break

print(f"\nFinal:  {np.array2string(gammas, precision=4)}")
print(f"True:   {np.array2string(true_10, precision=4)}")
print(f"Error:  {np.array2string(np.abs(gammas - true_10), precision=4)}")

# ===================================================================
# Experiment 3: N=5, start from UNIFORM spacing (no knowledge at all)
# ===================================================================
print("\n--- N=5: Start from uniform spacing [10, 20, 30, 40, 50] ---")
uniform_5 = np.array([10.0, 20.0, 30.0, 40.0, 50.0])
gammas = uniform_5.copy()
for i in range(200):
    new_g, re_parts = comb_map(gammas)
    if new_g is None:
        break
    diff = np.max(np.abs(new_g - gammas))
    err_vs_true = np.max(np.abs(new_g - true_5))
    if i < 10 or i % 20 == 0 or diff < 1e-14:
        print(f"Iter {i:3d}: {np.array2string(new_g, precision=4)}  "
              f"change={diff:.2e}  err_vs_true={err_vs_true:.4f}")
    gammas = new_g
    if diff < 1e-15:
        print(f"Converged at iteration {i}")
        break

print(f"\nFinal:  {gammas}")
print(f"True:   {true_5}")
print(f"Max error: {np.max(np.abs(gammas - true_5)):.6f}")

# ===================================================================
# Experiment 4: N=5, start from perturbed true zeros
# ===================================================================
print("\n--- N=5: Start from perturbed true zeros (±2.0 random) ---")
np.random.seed(42)
perturbed = true_5 + np.random.uniform(-2.0, 2.0, 5)
print(f"Perturbed: {perturbed}")
gammas = perturbed.copy()
for i in range(200):
    new_g, re_parts = comb_map(gammas)
    if new_g is None:
        break
    diff = np.max(np.abs(new_g - gammas))
    err_vs_true = np.max(np.abs(new_g - true_5))
    if i < 10 or i % 20 == 0 or diff < 1e-14:
        print(f"Iter {i:3d}: {np.array2string(new_g, precision=4)}  "
              f"change={diff:.2e}  err_vs_true={err_vs_true:.4f}")
    gammas = new_g
    if diff < 1e-15:
        print(f"Converged at iteration {i}")
        break

print(f"\nFinal:  {gammas}")
print(f"True:   {true_5}")
print(f"Max error: {np.max(np.abs(gammas - true_5)):.6f}")

# ===================================================================
# Summary
# ===================================================================
print("\n" + "=" * 70)
print("SUMMARY: Does the contraction mapping find the true zeros?")
print("=" * 70)
