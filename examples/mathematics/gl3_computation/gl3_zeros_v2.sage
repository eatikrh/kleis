from sage.all import *

print("Computing tau(n) via Sage modular forms...")
D = CuspForms(1, 12).gen(0)
N = 3000
tau = [ZZ(D.coefficient(n)) for n in range(N+1)]
print(f"tau(2)={tau[2]}, tau(3)={tau[3]}, tau(5)={tau[5]}")

print("Computing Sym^2 Dirichlet coefficients (with prime powers)...")
sym2 = [ZZ(0)] * (N+1)
sym2[1] = ZZ(1)

for p in primes(N+1):
    tp = tau[p]
    p11 = ZZ(p)**11
    c1 = tp**2 - p11
    c2 = p11 * tp**2 - p11**2
    c3 = p11**3

    sym2[p] = c1
    prev = [ZZ(1), c1]
    pe = p
    while pe * p <= N:
        pe *= p
        e = len(prev)
        if e == 2:
            val = c1 * prev[-1] - c2
        else:
            val = c1 * prev[-1] - c2 * prev[-2] + c3 * prev[-3]
        prev.append(val)
        sym2[pe] = val

for n in range(2, N+1):
    if sym2[n] == 0:
        fac = list(factor(n))
        if len(fac) > 1:
            result = ZZ(1)
            ok = True
            for (q, e) in fac:
                pe_val = q**e
                if pe_val <= N and sym2[pe_val] != 0:
                    result *= sym2[pe_val]
                else:
                    ok = False
                    break
            if ok:
                sym2[n] = result

nz = sum(1 for i in range(1, N+1) if sym2[i] != 0)
print(f"Non-zero coeffs: {nz}/{N}")
print(f"sym2[2]={sym2[2]}, sym2[3]={sym2[3]}, sym2[4]={sym2[4]}, sym2[6]={sym2[6]}")

g = gp
g.eval("default(realprecision, 77)")
g.eval("default(parisizemax, 2048*1024*1024)")

coeffs_str = "[" + ",".join(str(sym2[n]) for n in range(1, N+1)) + "]"
g.eval(f"av = {coeffs_str}")
g.eval("L = lfuncreate([av, 0, [-11, 0, 11], 23, 1, 1])")

fe = g.eval("lfuncheckfeq(L)")
print(f"\nFE check with {N} coeffs (prime powers included): {fe}")

for h in [20, 40, 60, 80, 100]:
    print(f"\nSearching zeros to height {h}...")
    raw = g.eval(f"lfunzeros(L, {h})")
    clean = raw.strip()
    if '[' in clean:
        inner = clean.split('[')[-1].split(']')[0]
        nums = [s.strip() for s in inner.split(',') if s.strip()]
        print(f"  Found {len(nums)} zeros:")
        for i, z in enumerate(nums):
            print(f"    gamma_{i+1} = {z}")
    else:
        print(f"  Raw: {clean[:120]}")
