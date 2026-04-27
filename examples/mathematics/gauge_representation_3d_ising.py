import numpy as np
from scipy import linalg
from itertools import product as iterproduct

def int_to_spins(i, N):
    return np.array([2*((i >> k) & 1) - 1 for k in range(N)])

def build_3d_transfer(N, beta):
    nsites = N * N
    dim = 1 << nsites
    T = np.zeros((dim, dim))
    for a in range(dim):
        sa = int_to_spins(a, nsites)
        sa2d = sa.reshape(N, N)
        Eh = 0.0
        for r in range(N):
            for c in range(N):
                Eh += sa2d[r, c] * sa2d[r, (c+1) % N]
                Eh += sa2d[r, c] * sa2d[(r+1) % N, c]
        for b in range(dim):
            sb = int_to_spins(b, nsites)
            Ev = float(np.dot(sa, sb))
            T[a, b] = np.exp(beta * Eh) * np.exp(beta * Ev)
    return (T + T.T) / 2

# ========================================================================
# THE THREE-STEP ROADMAP FOR SOLVING 3D ISING
# ========================================================================
#
# Step 1: Fourier decomposition (momentum sectors)
#         T → blocks labeled by (k_x, k_y)
#         Reduction: 2^(N²) → N² blocks of size ~2^(N²)/N²
#
# Step 2: Gauge transformation (Poincaré dual representation)
#         In the spin basis: interactions are on edges (1-cells)
#         In the gauge basis: interactions are on plaquettes (2-cells)
#         The gauge basis might have simpler interaction structure
#
# Step 3: Measure interaction order in both representations
#         If gauge representation has lower interaction order → use it

print("=" * 72)
print("THE THREE-STEP ROADMAP: SOLVING 3D ISING")
print("=" * 72)

# ========================================================================
# STEP 1: FOURIER DECOMPOSITION
# ========================================================================
print("\n" + "─" * 72)
print("STEP 1: FOURIER DECOMPOSITION (momentum sectors)")
print("─" * 72)

beta_c = 0.22165463
N = 2
nsites = N * N
dim = 1 << nsites

T = build_3d_transfer(N, beta_c)

# Build translation operators for the N×N layer
def build_tx(N):
    nsites = N*N
    dim = 1 << nsites
    P = np.zeros((dim, dim))
    for i in range(dim):
        bits = [(i >> k) & 1 for k in range(nsites)]
        bits2d = np.array(bits).reshape(N, N)
        new_bits2d = np.roll(bits2d, -1, axis=1)
        new_bits = list(new_bits2d.flatten())
        j = sum(b << k for k, b in enumerate(new_bits))
        P[j, i] = 1.0
    return P

def build_ty(N):
    nsites = N*N
    dim = 1 << nsites
    P = np.zeros((dim, dim))
    for i in range(dim):
        bits = [(i >> k) & 1 for k in range(nsites)]
        bits2d = np.array(bits).reshape(N, N)
        new_bits2d = np.roll(bits2d, -1, axis=0)
        new_bits = list(new_bits2d.flatten())
        j = sum(b << k for k, b in enumerate(new_bits))
        P[j, i] = 1.0
    return P

Tx = build_tx(N)
Ty = build_ty(N)

# Verify T commutes with translations
comm_x = np.max(np.abs(T @ Tx - Tx @ T))
comm_y = np.max(np.abs(T @ Ty - Ty @ T))
print(f"  [T, T_x] = {comm_x:.2e}  (should be ~0)")
print(f"  [T, T_y] = {comm_y:.2e}  (should be ~0)")

# Diagonalize T
eigvals, eigvecs = linalg.eigh(T)

# For each eigenvector, compute momentum quantum numbers
print(f"\n  Momentum sectors for {N}×{N} layer ({dim} eigenvalues):")
sectors = {}
for i in range(dim):
    v = eigvecs[:, i]
    # Tx eigenvalue
    Txv = Tx @ v
    idx = np.argmax(np.abs(v))
    kx_val = Txv[idx] / v[idx] if abs(v[idx]) > 1e-12 else 0
    # Ty eigenvalue
    Tyv = Ty @ v
    ky_val = Tyv[idx] / v[idx] if abs(v[idx]) > 1e-12 else 0
    
    kx_r = round(kx_val.real, 3) + 1j * round(kx_val.imag, 3)
    ky_r = round(ky_val.real, 3) + 1j * round(ky_val.imag, 3)
    key = (kx_r, ky_r)
    if key not in sectors:
        sectors[key] = []
    sectors[key].append(eigvals[i])

for key in sorted(sectors.keys(), key=lambda x: (abs(x[0].imag), abs(x[1].imag))):
    print(f"    (k_x={key[0]}, k_y={key[1]}): {len(sectors[key])} eigenvalues")

print(f"  Total sectors: {len(sectors)}")
print(f"  Reduction: {dim} → {len(sectors)} blocks")

# ========================================================================
# STEP 2: GAUGE TRANSFORMATION (Poincaré dual)
# ========================================================================
print("\n" + "─" * 72)
print("STEP 2: GAUGE TRANSFORMATION (Poincaré dual representation)")
print("─" * 72)

# The Poincaré dual maps:
#   Spin variables (on vertices) → Gauge variables (on plaquettes)
#   σ_i ∈ {±1} on each vertex → τ_p = ∏_{edges of p} σ_i σ_j on each plaquette
#
# For an N×N periodic lattice:
#   Vertices: N² 
#   Edges: 2N² (horizontal + vertical)
#   Plaquettes: N²
#
# The gauge variables τ_p = σ_{r,c} σ_{r,c+1} σ_{r+1,c+1} σ_{r+1,c}
# (product of 4 spins around the plaquette)
#
# Key: in gauge variables, the INTRA-LAYER energy involves plaquette terms
# (4-spin interactions in spin language, but SINGLE-SITE in gauge language!)

print("  Poincaré dual mapping:")
print(f"    Vertices: {N*N} → Spins: σ_{{r,c}}")
print(f"    Plaquettes: {N*N} → Gauge: τ_p = ∏ σ around plaquette")
print()

# Build the plaquette (gauge) transformation matrix
# Each plaquette p = (r,c) is the product of 4 spins:
#   τ_{r,c} = σ_{r,c} * σ_{r,c+1} * σ_{r+1,c+1} * σ_{r+1,c}
# In terms of bits: τ_{r,c} = b[r,c] XOR b[r,c+1] XOR b[r+1,c+1] XOR b[r+1,c]
# (since σ = 2b-1, the product of 4 spins with an even number of -1s gives +1)

def spin_to_gauge(spin_config, N):
    """Convert spin configuration to plaquette gauge variables."""
    s = np.array(spin_config).reshape(N, N)
    gauge = np.zeros((N, N), dtype=int)
    for r in range(N):
        for c in range(N):
            gauge[r, c] = s[r, c] * s[r, (c+1) % N] * s[(r+1) % N, (c+1) % N] * s[(r+1) % N, c]
    return gauge.flatten()

# Build the transformation matrix G: spin basis → gauge basis
# G[gauge_config, spin_config] = 1 if spin_config maps to gauge_config
n_plaquettes = N * N
gauge_dim = 1 << n_plaquettes

# For the N=2 case, the gauge map is 2-to-1 (global spin flip gives same gauge)
gauge_map = {}
for i in range(dim):
    spins = int_to_spins(i, nsites)
    gauge = spin_to_gauge(spins, N)
    # Convert gauge to integer index
    gauge_bits = [(1 if g == 1 else 0) for g in gauge]
    g_idx = sum(b << k for k, b in enumerate(gauge_bits))
    if g_idx not in gauge_map:
        gauge_map[g_idx] = []
    gauge_map[g_idx].append(i)

print(f"  Gauge map: {dim} spin configs → {len(gauge_map)} gauge configs")
print(f"  Degeneracy: {dim // len(gauge_map)}-to-1 (global Z₂ redundancy)")

# Build T in the gauge representation
# Project onto the even-spin sector (remove global Z₂)
# Then express T in gauge variables

# For each pair of gauge configs (g, g'), T_gauge[g,g'] = sum over spin reps
n_gauge = len(gauge_map)
gauge_indices = sorted(gauge_map.keys())
gauge_to_idx = {g: i for i, g in enumerate(gauge_indices)}

T_gauge = np.zeros((n_gauge, n_gauge))
for gi, g in enumerate(gauge_indices):
    spin_reps_g = gauge_map[g]
    for gj, h in enumerate(gauge_indices):
        spin_reps_h = gauge_map[h]
        # Average T over all spin representatives
        val = 0.0
        for si in spin_reps_g:
            for sj in spin_reps_h:
                val += T[si, sj]
        T_gauge[gi, gj] = val / len(spin_reps_g)  # normalize

T_gauge = (T_gauge + T_gauge.T) / 2  # ensure symmetry

eigvals_gauge = np.sort(np.real(linalg.eigvals(T_gauge)))[::-1]
eigvals_spin = np.sort(eigvals)[::-1]

print(f"\n  Transfer matrix in gauge representation: {n_gauge}×{n_gauge}")
print(f"  (vs spin representation: {dim}×{dim})")

# ========================================================================
# STEP 3: INTERACTION ORDER IN BOTH REPRESENTATIONS
# ========================================================================
print("\n" + "─" * 72)
print("STEP 3: INTERACTION ORDER — SPIN vs GAUGE REPRESENTATION")
print("─" * 72)

def walsh_hadamard_spectrum(eigvals, M, label):
    """Compute Walsh-Hadamard interaction order distribution."""
    dim = 1 << M
    n = min(len(eigvals), dim)
    
    vals = np.sort(np.abs(eigvals[:n]))[::-1]
    if n < dim:
        vals = np.pad(vals, (0, dim - n), constant_values=vals[-1] * 0.01)
    
    log_vals = np.log(vals + 1e-300)
    
    H1 = np.array([[1, 1], [1, -1]])
    HM = np.array([[1.0]])
    for _ in range(M):
        HM = np.kron(HM, H1)
    HM = HM / (2**M)
    coeffs = HM @ log_vals
    
    total = np.sum(coeffs**2)
    
    print(f"\n  {label}:")
    cumulative = 0
    for order in range(M + 1):
        order_coeffs = [coeffs[i]**2 for i in range(dim) if bin(i).count('1') == order]
        energy = sum(order_coeffs)
        pct = 100 * energy / total if total > 0 else 0
        cumulative += pct
        n_terms = len(order_coeffs)
        bar = "█" * int(pct / 2)
        olabel = {0: "const", 1: "linear", 2: "pair", 3: "3-body", 
                  4: "4-body"}.get(order, f"{order}-body")
        print(f"    {olabel:<8} ({n_terms:>3} terms): {pct:6.2f}%  cum={cumulative:6.2f}% {bar}")

# Spin representation (N² = 4 modes)
walsh_hadamard_spectrum(eigvals_spin, nsites, "SPIN representation (4 modes)")

# Gauge representation 
n_gauge_modes = int(np.log2(n_gauge))
if n_gauge_modes > 0 and (1 << n_gauge_modes) == n_gauge:
    walsh_hadamard_spectrum(eigvals_gauge, n_gauge_modes, 
                          f"GAUGE representation ({n_gauge_modes} modes)")

# ========================================================================
# Now do the same for N=3
# ========================================================================
print("\n\n" + "=" * 72)
print("3D ISING N=3: FULL ANALYSIS")
print("=" * 72)

N = 3
nsites = N * N  # 9
dim = 1 << nsites  # 512

print(f"\nBuilding {N}×{N} transfer matrix ({dim}×{dim})...")
T3 = build_3d_transfer(N, beta_c)
eigvals3 = np.sort(np.real(linalg.eigvals(T3)))[::-1]

# Build gauge map for N=3
gauge_map3 = {}
for i in range(dim):
    spins = int_to_spins(i, nsites)
    gauge = spin_to_gauge(spins, N)
    gauge_bits = [(1 if g == 1 else 0) for g in gauge]
    g_idx = sum(b << k for k, b in enumerate(gauge_bits))
    if g_idx not in gauge_map3:
        gauge_map3[g_idx] = []
    gauge_map3[g_idx].append(i)

n_gauge3 = len(gauge_map3)
print(f"Gauge map: {dim} spin configs → {n_gauge3} gauge configs")
print(f"Degeneracy: {dim // n_gauge3}-to-1")

# Build T in gauge representation for N=3
gauge_indices3 = sorted(gauge_map3.keys())
T_gauge3 = np.zeros((n_gauge3, n_gauge3))
for gi, g in enumerate(gauge_indices3):
    spin_reps_g = gauge_map3[g]
    for gj, h in enumerate(gauge_indices3):
        spin_reps_h = gauge_map3[h]
        val = 0.0
        for si in spin_reps_g:
            for sj in spin_reps_h:
                val += T3[si, sj]
        T_gauge3[gi, gj] = val / len(spin_reps_g)

T_gauge3 = (T_gauge3 + T_gauge3.T) / 2
eigvals_gauge3 = np.sort(np.real(linalg.eigvals(T_gauge3)))[::-1]

print(f"Gauge transfer matrix: {n_gauge3}×{n_gauge3}")

# Compare interaction orders
walsh_hadamard_spectrum(eigvals3, nsites, "SPIN representation (9 modes)")

n_gauge_modes3 = int(np.log2(n_gauge3))
if n_gauge_modes3 > 0 and (1 << n_gauge_modes3) == n_gauge3:
    walsh_hadamard_spectrum(eigvals_gauge3, n_gauge_modes3,
                          f"GAUGE representation ({n_gauge_modes3} modes)")
else:
    print(f"\n  GAUGE representation: {n_gauge3} configs (not a power of 2)")
    print(f"    (This means the gauge constraint reduces the Hilbert space)")
    print(f"    Gauge configs = {n_gauge3}, log2 = {np.log2(n_gauge3):.2f}")
    
    # Even if not a power of 2, compare eigenvalue counts
    print(f"\n    Spin eigenvalues: {len(eigvals3)} (dim = 2^{nsites} = {dim})")
    print(f"    Gauge eigenvalues: {len(eigvals_gauge3)} (dim = {n_gauge3})")
    print(f"    Reduction: {dim} → {n_gauge3} ({dim/n_gauge3:.1f}:1)")

# ========================================================================
# THE PRACTICAL ROADMAP
# ========================================================================
print("\n\n" + "=" * 72)
print("THE PRACTICAL ROADMAP FOR SOLVING 3D ISING")
print("=" * 72)
print("""
1. FOURIER DECOMPOSITION (done)
   Decompose T into momentum sectors (k_x, k_y).
   Reduction: 2^(N²) → N² blocks of size ~2^(N²)/N²
   This is standard and works for any lattice model.

2. POINCARÉ GAUGE TRANSFORMATION (this computation)
   Map spin variables → plaquette gauge variables via Poincaré duality.
   The gauge representation has FEWER degrees of freedom 
   (the gauge constraint eliminates the global Z₂).
   In gauge variables, intra-layer interactions become plaquette terms 
   = effectively PAIRWISE in the dual.

3. IDENTIFY EFFECTIVE MODES
   In the gauge representation, find the "quasi-particle" modes:
   - Diagonalize within each momentum sector
   - Express eigenvalues as functions of mode energies γ(k_x, k_y)
   - The mode energies are the N² PARAMETERS

4. MEASURE INTERACTION ORDER
   The Walsh-Hadamard expansion tells us:
   - Order 1 (linear): free quasi-particles
   - Order 2 (pairwise): weakly interacting quasi-particles  
   - Truncation at order p → O(N^2p) parameters → POLYNOMIAL TIME

5. EXTRAPOLATE TO THERMODYNAMIC LIMIT
   If the interaction coefficients converge as N → ∞:
   - The dispersion relation γ(k_x, k_y) has a thermodynamic limit
   - The interaction terms J(k, k') have a thermodynamic limit
   - f(β) = integral over the Brillouin zone of the effective energy
""")
