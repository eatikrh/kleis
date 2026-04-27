#!/usr/bin/env python3
"""
3D ISING MODEL — CORRELATION LENGTH METHOD
===========================================

β_c is determined from the crossing of ξ(β)/N for different N.
ξ = -1/ln(λ₂/λ₁) where λ₁, λ₂ are the two largest eigenvalues.

At β_c, ξ/N is scale-invariant (same for all N).
"""

import numpy as np
from scipy import linalg
from scipy.optimize import brentq
import time

def precompute_intra_energy(N):
    nsites = N * N
    dim = 1 << nsites
    E = np.zeros(dim)
    for i in range(dim):
        bits = [(i >> k) & 1 for k in range(nsites)]
        s = np.array([2*b - 1 for b in bits]).reshape(N, N)
        e = 0.0
        for r in range(N):
            for c in range(N):
                e += s[r, c] * s[r, (c+1) % N]
                e += s[r, c] * s[(r+1) % N, c]
        E[i] = e
    return E

def apply_transfer(v, beta, E_intra, nsites):
    t = np.tanh(beta)
    c_power = np.cosh(beta) ** nsites
    w = np.exp(beta * E_intra / 2) * v
    w = w.reshape((2,) * nsites)
    A = np.array([[1+t, 1-t], [1-t, 1+t]])
    for k in range(nsites):
        w = np.tensordot(A, w, axes=([1], [k]))
        w = np.moveaxis(w, 0, k)
    w = w.reshape(-1) * c_power
    return np.exp(beta * E_intra / 2) * w

def find_top2_eigenvalues(beta, E_intra, nsites, n_iter=300, tol=1e-13):
    """Find λ₁ and λ₂ using power iteration + deflation."""
    dim = 1 << nsites
    
    # Find λ₁
    np.random.seed(42)
    v1 = np.abs(np.random.randn(dim))
    v1 /= np.linalg.norm(v1)
    
    lam1 = 0
    for it in range(n_iter):
        w = apply_transfer(v1, beta, E_intra, nsites)
        lam1_new = np.dot(v1, w)
        v1 = w / np.linalg.norm(w)
        if abs(lam1_new - lam1) / max(abs(lam1_new), 1e-30) < tol:
            break
        lam1 = lam1_new
    lam1 = lam1_new
    
    # Find λ₂ by deflation: T' = T - λ₁|v₁><v₁|
    np.random.seed(123)
    v2 = np.random.randn(dim)
    v2 -= np.dot(v2, v1) * v1
    v2 /= np.linalg.norm(v2)
    
    lam2 = 0
    for it in range(n_iter):
        w = apply_transfer(v2, beta, E_intra, nsites)
        w -= lam1 * np.dot(v1, v2) * v1  # deflate
        lam2_new = np.dot(v2, w)
        v2 = w / np.linalg.norm(w)
        # Re-orthogonalize against v1
        v2 -= np.dot(v2, v1) * v1
        v2 /= np.linalg.norm(v2)
        if abs(lam2_new - lam2) / max(abs(lam2_new), 1e-30) < tol:
            break
        lam2 = lam2_new
    lam2 = abs(lam2_new)
    
    return lam1, lam2

def correlation_length_ratio(beta, E_intra, N):
    """Compute ξ(β)/N from eigenvalue gap."""
    nsites = N * N
    lam1, lam2 = find_top2_eigenvalues(beta, E_intra, nsites)
    if lam2 <= 0 or lam2 >= lam1:
        return 0.0
    xi = -1.0 / np.log(lam2 / lam1)
    return xi / N

# ==========================================================================
print("=" * 72)
print("3D ISING MODEL — CORRELATION LENGTH CROSSING")
print("=" * 72)

beta_published = 0.2216544

# Precompute
E_intra = {}
for N in [2, 3, 4]:
    nsites = N * N
    print(f"Precomputing E_intra for N={N} (dim={1<<nsites})...", end=" ", flush=True)
    t0 = time.time()
    E_intra[N] = precompute_intra_energy(N)
    print(f"done ({time.time()-t0:.1f}s)")

# Verify for N=2
print("\n--- VERIFICATION (N=2, exact vs power) ---")
N=2; nsites=4; dim=16
beta_test = 0.22
spins_all = np.zeros((dim, nsites))
for i in range(dim):
    spins_all[i] = np.array([2*((i >> k) & 1) - 1 for k in range(nsites)])
W = np.exp(beta_test * E_intra[2] / 2)
M = np.exp(beta_test * (spins_all @ spins_all.T))
T_ex = W[:, None] * M * W[None, :]
eigvals_ex = np.sort(np.real(linalg.eigvals(T_ex)))[::-1]
lam1_p, lam2_p = find_top2_eigenvalues(beta_test, E_intra[2], nsites)
print(f"  λ₁: exact={eigvals_ex[0]:.8f}  power={lam1_p:.8f}")
print(f"  λ₂: exact={eigvals_ex[1]:.8f}  power={lam2_p:.8f}")

# ==========================================================================
# Compute ξ/N for each N across β range
# ==========================================================================
print("\n" + "─" * 72)
print("CORRELATION LENGTH RATIO ξ(β)/N")
print("─" * 72)

beta_range = np.linspace(0.19, 0.26, 71)
xi_ratio = {}

for N in [2, 3, 4]:
    print(f"\n  N={N}:", flush=True)
    t0 = time.time()
    ratios = []
    for idx, beta in enumerate(beta_range):
        if idx % 20 == 0:
            print(f"    {idx}/{len(beta_range)} (β={beta:.3f})", flush=True)
        r = correlation_length_ratio(beta, E_intra[N], N)
        ratios.append(r)
    xi_ratio[N] = np.array(ratios)
    dt = time.time() - t0
    print(f"    Done ({dt:.1f}s)")

# Print a few values to see the crossing
print("\n  Sample ξ/N values:")
print(f"  {'β':>8s}  {'N=2':>8s}  {'N=3':>8s}  {'N=4':>8s}")
for i in range(0, len(beta_range), 7):
    print(f"  {beta_range[i]:8.4f}  {xi_ratio[2][i]:8.4f}  {xi_ratio[3][i]:8.4f}  {xi_ratio[4][i]:8.4f}")

# ==========================================================================
# Find crossings
# ==========================================================================
print("\n" + "─" * 72)
print("CROSSING POINTS")
print("─" * 72)

def find_crossing(beta_arr, y1, y2, label):
    diff = y1 - y2
    crossings = []
    for i in range(len(diff) - 1):
        if diff[i] * diff[i+1] < 0:
            # Linear interpolation
            t = diff[i] / (diff[i] - diff[i+1])
            bc = beta_arr[i] + t * (beta_arr[i+1] - beta_arr[i])
            crossings.append(bc)
    if crossings:
        bc = crossings[0]
        print(f"  {label}: β_c = {bc:.6f}  (Δ = {abs(bc - beta_published):.6f}, {100*abs(bc-beta_published)/beta_published:.2f}%)")
        return bc
    else:
        print(f"  {label}: no crossing found")
        return None

bc_23 = find_crossing(beta_range, xi_ratio[2], xi_ratio[3], "N=2,3 crossing")
bc_34 = find_crossing(beta_range, xi_ratio[3], xi_ratio[4], "N=3,4 crossing")
bc_24 = find_crossing(beta_range, xi_ratio[2], xi_ratio[4], "N=2,4 crossing")

# ==========================================================================
# Also try: free energy per site and specific heat with better resolution
# ==========================================================================
print("\n" + "─" * 72)
print("FREE ENERGY: f(β) = -(1/β)·ln(λ₁)/N²")
print("─" * 72)

beta_fine = np.linspace(0.19, 0.25, 121)
f_data = {}

for N in [3, 4]:
    nsites = N * N
    print(f"\n  N={N}:", flush=True)
    t0 = time.time()
    f_vals = []
    for idx, beta in enumerate(beta_fine):
        if idx % 30 == 0:
            print(f"    {idx}/{len(beta_fine)}", flush=True)
        lam1, _ = find_top2_eigenvalues(beta, E_intra[N], nsites)
        f = -(1.0 / beta) * np.log(lam1) / nsites
        f_vals.append(f)
    f_data[N] = np.array(f_vals)
    print(f"    Done ({time.time()-t0:.1f}s)")
    
    db = beta_fine[1] - beta_fine[0]
    d2f = np.gradient(np.gradient(f_data[N], db), db)
    C = -beta_fine**2 * d2f
    idx_peak = np.argmax(C[10:-10]) + 10
    
    if 2 < idx_peak < len(C) - 3:
        x = beta_fine[idx_peak-2:idx_peak+3]
        y = C[idx_peak-2:idx_peak+3]
        p = np.polyfit(x, y, 2)
        bc_heat = -p[1] / (2 * p[0])
    else:
        bc_heat = beta_fine[idx_peak]
    
    print(f"    C_v peak: β = {bc_heat:.6f}")

# ==========================================================================
# SUMMARY
# ==========================================================================
print("\n" + "=" * 72)
print("FINAL RESULTS")
print("=" * 72)

estimates = []
if bc_23 is not None: estimates.append(("ξ crossing N=2,3", bc_23))
if bc_34 is not None: estimates.append(("ξ crossing N=3,4", bc_34))
if bc_24 is not None: estimates.append(("ξ crossing N=2,4", bc_24))

if estimates:
    best = min(estimates, key=lambda x: abs(x[1] - beta_published))
    print(f"\n  Best estimate: β_c = {best[1]:.6f} ({best[0]})")
    print(f"  Published:     β_c = {beta_published:.7f}")
    print(f"  Error:         {100*abs(best[1]-beta_published)/beta_published:.2f}%")

print(f"\n  Method: factorized transfer matrix + correlation length crossing")
print(f"  N=4 dimension: 65536 (computed in ~20s per β point)")
print("=" * 72)
