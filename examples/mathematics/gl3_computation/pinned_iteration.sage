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
    return np.array([e.imag for e in pos_evals[:len(gammas)]]), \
           np.array([e.real for e in pos_evals[:len(gammas)]])

# ===================================================================
# Cross-validation: verify our 2 GP zeros match the Pari/GP values
# ===================================================================
print("=" * 70)
print("CROSS-VALIDATION WITH PARI/GP ZEROS")
print("=" * 70)

gp_zeros = [5.70568488235770752, 8.18249171274299092]
print(f"\nPari/GP zeros (57-digit): {gp_zeros}")

# At N=2, run 5000 iterations to find the comb's fixed point
gammas = np.array(gp_zeros)
for i in range(5000):
    new_g, _ = comb_map(gammas)
    if len(new_g) == 2:
        diff = np.max(np.abs(new_g - gammas))
        if i < 5 or i % 500 == 0:
            print(f"Iter {i:4d}: [{new_g[0]:.10f}, {new_g[1]:.10f}]  change={diff:.2e}")
        gammas = new_g
        if diff < 1e-15:
            print(f"Converged at iteration {i}")
            break

print(f"\nComb fixed point (N=2): [{gammas[0]:.10f}, {gammas[1]:.10f}]")
print(f"Difference from GP:     [{abs(gammas[0]-gp_zeros[0]):.6f}, {abs(gammas[1]-gp_zeros[1]):.6f}]")

# ===================================================================
# Pinned iteration: fix first 2, iterate remaining
# ===================================================================
print("\n" + "=" * 70)
print("PINNED ITERATION: Fix gamma_1, gamma_2 from Pari/GP, iterate rest")
print("=" * 70)

for N_total in [5, 8, 10]:
    n_free = N_total - 2
    # Initial rough estimates for unknown zeros
    spacing = 3.0
    init_free = [gp_zeros[-1] + spacing * (i+1) for i in range(n_free)]
    gammas = np.array(gp_zeros + init_free)
    
    print(f"\n--- N={N_total} ({n_free} free zeros, 2 pinned) ---")
    print(f"Initial: {np.array2string(gammas, precision=4)}")
    
    for iteration in range(2000):
        new_g, re_parts = comb_map(gammas)
        if len(new_g) == N_total:
            # Pin first 2 zeros
            new_g[0] = gp_zeros[0]
            new_g[1] = gp_zeros[1]
            diff = np.max(np.abs(new_g[2:] - gammas[2:]))
            if iteration < 5 or iteration % 200 == 0 or diff < 1e-12:
                print(f"Iter {iteration:4d}: {np.array2string(new_g, precision=5)}  free_change={diff:.2e}")
            gammas = new_g
            if diff < 1e-13:
                print(f"Converged at iteration {iteration}")
                break
        else:
            print(f"Iter {iteration}: eigenvalue count mismatch")
            break
    
    # Final diagnostics
    H = build_comb_matrix(gammas)
    evals = np.linalg.eig(H)[0]
    pos_evals = sorted([e for e in evals if e.imag > 0.01], key=lambda x: x.imag)
    
    max_re = max(abs(e.real - 0.5) for e in pos_evals[:N_total])
    im_parts = np.array([e.imag for e in pos_evals[:N_total]])
    total_err = np.sum(np.abs(im_parts - gammas))
    eps = 2 * np.pi / np.mean(gammas)
    
    # Contraction norm
    delta = 1e-4
    frob_sq = 0.0
    for j in range(N_total):
        gp_arr = gammas.copy()
        gp_arr[j] += delta
        Hp = build_comb_matrix(gp_arr)
        ep = np.linalg.eig(Hp)[0]
        pos_ep = sorted([e for e in ep if e.imag > 0.01], key=lambda x: x.imag)
        im_p = np.array([e.imag for e in pos_ep[:N_total]])
        for i in range(N_total):
            deriv = (im_p[i] - im_parts[i]) / delta
            entry = deriv - 1.0 if i == j else deriv
            frob_sq += entry ** 2
    frob_norm = np.sqrt(frob_sq)
    
    # Min gap and predicted bound
    gaps = [gammas[i+1] - gammas[i] for i in range(N_total - 1)]
    min_gap = min(gaps)
    pred_bound = np.sqrt(3 * N_total) * 2 * eps**2 / min_gap**2
    safety = pred_bound / frob_norm if frob_norm > 0 else float('inf')
    
    print(f"\n  RESULTS (N={N_total}):")
    print(f"  epsilon = {eps:.4f}")
    print(f"  Max |Re - 0.5| = {max_re:.2e}")
    print(f"  Total error = {total_err:.2e}")
    print(f"  Mean error = {total_err/N_total:.2e}")
    print(f"  ||J_F - I||_F = {frob_norm:.6f}  (< 1: {frob_norm < 1.0})")
    print(f"  Predicted bound = {pred_bound:.6f}")
    print(f"  Safety factor = {safety:.1f}x")
    print(f"  Min gap = {min_gap:.4f}")
    print(f"  Zeros: {np.array2string(gammas, precision=6)}")

# ===================================================================
# Smooth-zero failure test (N=5)  
# ===================================================================
print("\n" + "=" * 70)
print("SMOOTH-ZERO FAILURE TEST (GL(3), N=5)")
print("=" * 70)

# Use the pinned-iterated N=5 zeros
# Re-compute the N=5 pinned solution
gammas5 = np.array(gp_zeros + [gp_zeros[-1] + 3*(i+1) for i in range(3)])
for i in range(2000):
    new_g, _ = comb_map(gammas5)
    if len(new_g) == 5:
        new_g[0] = gp_zeros[0]; new_g[1] = gp_zeros[1]
        gammas5 = new_g

# Smooth approximation (evenly spaced based on density)
mean_g = np.mean(gammas5)
smooth5 = np.array([mean_g * (i+1)/3.0 for i in range(5)])

H_actual = build_comb_matrix(gammas5)
ev_actual = np.linalg.eig(H_actual)[0]
pos_actual = sorted([e for e in ev_actual if e.imag > 0.01], key=lambda x: x.imag)
im_actual = np.array([e.imag for e in pos_actual[:5]])
err_actual = np.sum(np.abs(im_actual - gammas5))

H_smooth = build_comb_matrix(smooth5)
ev_smooth = np.linalg.eig(H_smooth)[0]
pos_smooth = sorted([e for e in ev_smooth if e.imag > 0.01], key=lambda x: x.imag)
im_smooth = np.array([e.imag for e in pos_smooth[:5]])
err_smooth = np.sum(np.abs(im_smooth - gammas5))

print(f"Arithmetic zeros: {np.array2string(gammas5, precision=4)}")
print(f"Smooth zeros:     {np.array2string(smooth5, precision=4)}")
print(f"Actual error:  {err_actual:.6f}")
print(f"Smooth error:  {err_smooth:.6f}")
print(f"Degradation:   {err_smooth/err_actual:.0f}x")

# ===================================================================
# Antisymmetry cliff test (N=5)
# ===================================================================
print("\n" + "=" * 70)
print("ANTISYMMETRY CLIFF TEST (GL(3), N=5)")
print("=" * 70)

n = 10
eps5 = 2 * np.pi / np.mean(gammas5)
off = []
for k in range(5):
    off.append(gammas5[k])
    if k < 4: off.append(eps5)

# Antisymmetric
H_anti = np.zeros((n, n))
for i in range(n): H_anti[i,i] = 0.5
for j in range(len(off)):
    H_anti[j][j+1] = off[j]
    H_anti[j+1][j] = -off[j]
ev_anti = np.linalg.eig(H_anti)[0]
pos_anti = sorted([e for e in ev_anti if e.imag > 0.01], key=lambda x: x.imag)
max_re_anti = max(abs(e.real - 0.5) for e in pos_anti[:5])

# Symmetric
H_sym = np.zeros((n, n))
for i in range(n): H_sym[i,i] = 0.5
for j in range(len(off)):
    H_sym[j][j+1] = off[j]
    H_sym[j+1][j] = off[j]  # SAME sign = symmetric
ev_sym = np.linalg.eig(H_sym)[0]
re_sym1 = sorted(ev_sym.real)[0]

print(f"Antisymmetric: max |Re - 0.5| = {max_re_anti:.2e}")
print(f"Symmetric:     Re(lambda_1) = {re_sym1:.4f}")
print(f"CLIFF:         {max_re_anti:.2e} -> {abs(re_sym1-0.5):.2f}")

