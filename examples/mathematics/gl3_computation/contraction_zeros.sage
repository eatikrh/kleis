import numpy as np

def build_comb_matrix(zeros_list):
    """Build the antisymmetric spectral comb matrix from zeros."""
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
    """The spectral comb map F: zeros -> eigenvalue imaginary parts."""
    H = build_comb_matrix(gammas)
    evals = np.linalg.eig(H)[0]
    
    # Sort by imaginary part (positive)
    pos_evals = sorted([e for e in evals if e.imag > 0.01], key=lambda x: x.imag)
    
    if len(pos_evals) != len(gammas):
        # Fall back: take top half sorted by |imag|
        sorted_all = sorted(evals, key=lambda x: -abs(x.imag))
        pos_evals = sorted([e for e in sorted_all[:len(gammas)*2] if e.imag > 0], 
                          key=lambda x: x.imag)
    
    return np.array([e.imag for e in pos_evals[:len(gammas)]]), \
           np.array([e.real for e in pos_evals[:len(gammas)]])

# Known zeros from Pari/GP
known = [5.70568, 8.18249]

print("=" * 70)
print("SPECTRAL COMB CONTRACTION ITERATION FOR L(s, Sym^2 Delta)")
print("=" * 70)

# ===================================================================
# Experiment 1: Verify the 2 known zeros (N=2)
# ===================================================================
print("\n--- Experiment 1: N=2 (verification of known zeros) ---")
gammas = np.array(known)
im_out, re_out = comb_map(gammas)
print(f"Input:  {gammas}")
print(f"Output: {im_out}")
print(f"Re:     {re_out}")
print(f"Error:  {np.abs(im_out - gammas)}")
print(f"|Re - 0.5|: {np.abs(re_out - 0.5)}")

# ===================================================================
# Experiment 2: Start with known + rough estimates, iterate
# ===================================================================
print("\n--- Experiment 2: N=5 (2 known + 3 estimated), contraction iteration ---")

# Rough estimates based on zero density
# Average gap from first 2: ~2.5. Extrapolate conservatively.
initial_guesses = [5.71, 8.18, 11.0, 14.0, 17.0]
gammas = np.array(initial_guesses)

print(f"Initial: {gammas}")
for iteration in range(30):
    new_gammas, re_parts = comb_map(gammas)
    if len(new_gammas) == len(gammas):
        diff = np.max(np.abs(new_gammas - gammas))
        gammas = new_gammas
        if iteration < 10 or iteration % 5 == 0 or diff < 1e-12:
            print(f"Iter {iteration:2d}: {gammas}  max_change={diff:.2e}  max|Re-0.5|={np.max(np.abs(re_parts-0.5)):.2e}")
        if diff < 1e-14:
            print(f"Converged at iteration {iteration}")
            break
    else:
        print(f"Iter {iteration}: eigenvalue count mismatch ({len(new_gammas)} vs {len(gammas)})")
        break

print(f"\nFinal zeros (N=5): {gammas}")

# ===================================================================
# Experiment 3: Start with slightly different initial estimates
# ===================================================================
print("\n--- Experiment 3: N=5, different starting point ---")
initial_guesses2 = [6.0, 9.0, 12.0, 15.0, 18.0]
gammas2 = np.array(initial_guesses2)

print(f"Initial: {gammas2}")
for iteration in range(30):
    new_gammas, re_parts = comb_map(gammas2)
    if len(new_gammas) == len(gammas2):
        diff = np.max(np.abs(new_gammas - gammas2))
        gammas2 = new_gammas
        if iteration < 10 or iteration % 5 == 0 or diff < 1e-12:
            print(f"Iter {iteration:2d}: {gammas2}  max_change={diff:.2e}")
        if diff < 1e-14:
            print(f"Converged at iteration {iteration}")
            break
    else:
        print(f"Iter {iteration}: eigenvalue count mismatch")
        break

print(f"\nFinal zeros (N=5, start 2): {gammas2}")
print(f"Agreement with Exp 2: {np.max(np.abs(gammas - gammas2)):.2e}")

# ===================================================================
# Experiment 4: N=8, push further
# ===================================================================
print("\n--- Experiment 4: N=8 (push to more zeros) ---")
initial_guesses3 = [5.71, 8.18, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0]
gammas3 = np.array(initial_guesses3)

print(f"Initial: {gammas3}")
for iteration in range(50):
    new_gammas, re_parts = comb_map(gammas3)
    if len(new_gammas) == len(gammas3):
        diff = np.max(np.abs(new_gammas - gammas3))
        gammas3 = new_gammas
        if iteration < 10 or iteration % 5 == 0 or diff < 1e-12:
            print(f"Iter {iteration:2d}: {np.array2string(gammas3, precision=5)}  max_change={diff:.2e}")
        if diff < 1e-14:
            print(f"Converged at iteration {iteration}")
            break
    else:
        print(f"Iter {iteration}: eigenvalue count mismatch ({len(new_gammas)} vs {len(gammas3)})")
        break

print(f"\nFinal zeros (N=8): {gammas3}")

# ===================================================================
# Experiment 5: N=10
# ===================================================================
print("\n--- Experiment 5: N=10 ---")
# Use converged N=8 values + 2 more estimates
if len(gammas3) == 8:
    g10_init = list(gammas3) + [29.0, 32.0]
else:
    g10_init = [5.71, 8.18, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0, 29.0, 32.0]
gammas10 = np.array(g10_init)

print(f"Initial: {np.array2string(gammas10, precision=4)}")
for iteration in range(50):
    new_gammas, re_parts = comb_map(gammas10)
    if len(new_gammas) == len(gammas10):
        diff = np.max(np.abs(new_gammas - gammas10))
        gammas10 = new_gammas
        if iteration < 5 or iteration % 10 == 0 or diff < 1e-12:
            print(f"Iter {iteration:2d}: {np.array2string(gammas10, precision=5)}  max_change={diff:.2e}")
        if diff < 1e-14:
            print(f"Converged at iteration {iteration}")
            break
    else:
        print(f"Iter {iteration}: eigenvalue count mismatch")
        break

print(f"\nFinal zeros (N=10): {np.array2string(gammas10, precision=8)}")

# ===================================================================
# Summary: contraction diagnostics on the converged values
# ===================================================================
print("\n" + "=" * 70)
print("CONTRACTION DIAGNOSTICS ON CONVERGED VALUES")
print("=" * 70)

for label, glist in [("N=2", known), ("N=5", gammas.tolist()), ("N=8", gammas3.tolist()), ("N=10", gammas10.tolist())]:
    garr = np.array(glist)
    H = build_comb_matrix(garr)
    evals = np.linalg.eig(H)[0]
    pos_evals = sorted([e for e in evals if e.imag > 0.01], key=lambda x: x.imag)
    
    max_re_dev = max(abs(e.real - 0.5) for e in pos_evals[:len(garr)])
    
    # Total error (self-consistency)
    im_parts = [e.imag for e in pos_evals[:len(garr)]]
    total_err = sum(abs(im_parts[i] - garr[i]) for i in range(len(garr)))
    
    eps = 2 * np.pi / np.mean(garr)
    
    print(f"\n{label}: eps={eps:.4f}")
    print(f"  Max |Re - 0.5| = {max_re_dev:.2e}")
    print(f"  Total error = {total_err:.2e}")
    print(f"  Mean error = {total_err/len(garr):.2e}")
    
    # Contraction norm via finite differences
    delta = 1e-4
    im_out0 = np.array(im_parts[:len(garr)])
    frob_sq = 0.0
    for j in range(len(garr)):
        gp = garr.copy()
        gp[j] += delta
        Hp = build_comb_matrix(gp)
        ep = np.linalg.eig(Hp)[0]
        pos_ep = sorted([e for e in ep if e.imag > 0.01], key=lambda x: x.imag)
        im_p = np.array([e.imag for e in pos_ep[:len(garr)]])
        for i in range(len(garr)):
            deriv = (im_p[i] - im_out0[i]) / delta
            entry = deriv - 1.0 if i == j else deriv
            frob_sq += entry ** 2
    frob_norm = np.sqrt(frob_sq)
    print(f"  ||J_F - I||_F = {frob_norm:.6f}  (< 1: {frob_norm < 1.0})")

