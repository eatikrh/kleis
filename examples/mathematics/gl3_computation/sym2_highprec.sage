from sage.all import *

# Strategy: compute 2000 Dirichlet coefficients with higher GP precision
# Then use lfunzeros with explicit precision parameter

N = 2000
binom24 = [binomial(24, k) * (-1)**k for k in range(25)]

print(f"Computing eta^24 product to {N} terms...")
coeffs = [ZZ(0)] * (N + 1)
coeffs[0] = ZZ(1)

for n in range(1, N + 1):
    new_coeffs = [ZZ(0)] * (N + 1)
    for j in range(N + 1):
        if coeffs[j] == 0: continue
        for k in range(25):
            idx = j + n * k
            if idx > N: break
            new_coeffs[idx] += coeffs[j] * binom24[k]
    coeffs = new_coeffs
    if n % 500 == 0:
        print(f"  {n}/{N}")

tau_vals = {}
for p in primes(N + 1):
    tau_vals[p] = coeffs[p - 1]

print(f"tau(2)={tau_vals[2]}, tau(997)={tau_vals[997]}")

# Sym^2 coefficients
sym2 = [ZZ(0)] * (N + 1)
sym2[1] = ZZ(1)
for p in primes(N + 1):
    tp = tau_vals[p]; p11 = ZZ(p)**11
    c1 = tp**2 - p11; c2 = p11*tp**2 - p11**2; c3 = p11**3
    sym2[p] = c1
    prev = [ZZ(1), c1]; pe = p
    while pe * p <= N:
        pe *= p; e = len(prev)
        if e == 2: val = c1*prev[-1] - c2
        else: val = c1*prev[-1] - c2*prev[-2] + c3*prev[-3]
        prev.append(val); sym2[pe] = val

for n in range(2, N + 1):
    if sym2[n] == 0:
        fac = list(factor(n))
        if len(fac) > 1:
            result = ZZ(1); ok = True
            for (p, e) in fac:
                pe = p**e
                if pe <= N and sym2[pe] != 0: result *= sym2[pe]
                else: ok = False; break
            if ok: sym2[n] = result

print(f"Computed {N} Sym^2 coefficients")

# Use GP with higher precision
g = gp
g.eval("default(realprecision, 57)")  # 57 digits
g.eval(f"default(parisizemax, {512*1024*1024})")

coeffs_str = "[" + ",".join(str(sym2[n]) for n in range(1, N+1)) + "]"
g.eval(f"av = {coeffs_str}")
g.eval("Lsym2 = lfuncreate([av, 0, [-11, 0, 11], 23, 1, 1])")

fe = g.eval("lfuncheckfeq(Lsym2)")
print(f"FE check with {N} coeffs, 57-digit precision: {fe}")

# Try finding zeros up to height 60
import re
print("\nSearching for zeros up to height 60...")
raw = g.eval("lfunzeros(Lsym2, 60)")
print(f"Raw result: {raw}")

# Parse the zeros
clean = raw.split('\n')[-1].strip()
if '[' in clean:
    nums = clean.strip('[]').split(',')
    zeros = [s.strip() for s in nums if s.strip()]
    print(f"\n{len(zeros)} zeros found:")
    for i, z in enumerate(zeros):
        print(f"  gamma_{i+1} = {z}")

