from sage.all import *

g = gp
g.eval("default(realprecision, 38)")
g.eval("default(parisizemax, 1024*1024*1024)")

# Build L(s, Sym^2 Delta) with 3000 coefficients
print("Building L(s, Sym^2 Delta) with 3000 coefficients...")
D = CuspForms(1, 12).gen(0)
N = 3000
tau = [ZZ(D.coefficient(n)) for n in range(N+1)]

sym2 = [ZZ(0)] * (N+1)
sym2[1] = ZZ(1)
for p in primes(N+1):
    tp = tau[p]; p11 = ZZ(p)**11
    c1 = tp**2 - p11; c2 = p11*tp**2 - p11**2; c3 = p11**3
    sym2[p] = c1
    prev = [ZZ(1), c1]; pe = p
    while pe * p <= N:
        pe *= p; e = len(prev)
        if e == 2: val = c1*prev[-1] - c2
        else: val = c1*prev[-1] - c2*prev[-2] + c3*prev[-3]
        prev.append(val); sym2[pe] = val

for n in range(2, N+1):
    if sym2[n] == 0:
        fac = list(factor(n))
        if len(fac) > 1:
            result = ZZ(1); ok = True
            for (q, e) in fac:
                pe_val = q**e
                if pe_val <= N and sym2[pe_val] != 0: result *= sym2[pe_val]
                else: ok = False; break
            if ok: sym2[n] = result

coeffs_str = "[" + ",".join(str(sym2[n]) for n in range(1, N+1)) + "]"
g.eval(f"av = {coeffs_str}")
g.eval("L = lfuncreate([av, 0, [-11, 0, 11], 23, 1, 1])")
fe = g.eval("lfuncheckfeq(L)")
print(f"FE check: 10^({fe})")

# The critical line is at Re(s) = 23/2 = 11.5
# Our zeros: gamma_1 ~ 5.706, gamma_2 ~ 8.183
# So the zeros are at s = 11.5 + i*gamma

print("\n" + "=" * 70)
print("EVALUATING L(s, Sym^2 Delta) AT ALLEGED ZEROS")
print("Critical line: Re(s) = 11.5")
print("=" * 70)

# Evaluate at our zeros
for gamma in ["5.70568488235770752", "8.18249171274299092"]:
    val = g.eval(f"lfun(L, 11.5 + {gamma}*I)")
    print(f"\nL(11.5 + {gamma}*i) = {val}")

# Evaluate at nearby NON-zero points for comparison
print("\n" + "=" * 70)
print("COMPARISON: L-function at NON-ZERO points")
print("=" * 70)

for gamma in ["5.0", "7.0", "10.0", "12.0", "15.0"]:
    val = g.eval(f"lfun(L, 11.5 + {gamma}*I)")
    print(f"L(11.5 + {gamma}*i) = {val}")

# Also evaluate at the center s = 11.5 (real axis)
print(f"\nL(11.5) = {g.eval('lfun(L, 11.5)')}")
print(f"L(12)   = {g.eval('lfun(L, 12)')}")
print(f"L(13)   = {g.eval('lfun(L, 13)')}")
