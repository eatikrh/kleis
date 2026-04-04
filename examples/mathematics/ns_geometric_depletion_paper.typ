#import "@preview/lilaq:0.5.0" as lq
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
  header: align(right)[_Preprint_],
)
#set text(
  font: "New Computer Modern",
  size: 11pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 0.65em,
  first-line-indent: 1em,
)

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Section headings (level 1)
#show heading.where(level: 1): it => {
  v(1em)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.5em)
}

// Subsection headings (level 2)
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 11pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.4em)
}

// Subsubsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.6em)
  text(size: 10pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.3em)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 9pt)[#it]
}
#show link: it => text(fill: blue.darken(20%))[#underline[#it]]


#align(center)[
  #text(size: 17pt, weight: "bold")[Geometric Depletion of Vortex Stretching: Machine-Verified Conditions for Navier-Stokes Regularity]
  
  #v(1em)
  
  Engin Atik#super[1]
  
  #v(0.5em)
  
  #super[1]Kleis Research, https://kleis.io
]

#v(1em)

#align(center)[
  #rect(width: 85%, stroke: none)[
    #align(left)[
      #text(weight: "bold")[Abstract]
      #v(0.3em)
      #text(size: 10pt)[We decompose the vortex stretching integral into geometric variables — strain eigenvalues $lambda_i$ and alignment weights $alpha_i = (xi dot e_i)^2$ — and use the Z3 SMT solver to determine which geometric conditions close the half-derivative gap identified in [1]. Thirteen satisfiability tests yield four results. First, bounding the alignment weight $alpha_1$ alone is insufficient: the intermediate eigenvalue $lambda_2$ provides an uncontrolled stretching channel that preserves the worst-case exponent scaling. Second, the exponent threshold $a + b = 2$ is sharp: every $a + b > 2$ (including $2.1$) admits blow-up, with no gradual transition. Third, gap closure requires both scale-dependent alignment depletion ($alpha_1 <= C slash Omega$) and biaxial strain ($lambda_2 <= 0$); neither condition alone suffices. Fourth, a Depletion Boundedness Theorem: for the coupled enstrophy-alignment ODE with any positive depletion rate $kappa$, the total stretching integral is bounded by $alpha_0 slash kappa$, giving $Omega(t) <= Omega_0 exp(alpha_0 slash kappa - 2 nu t) -> 0$. These results reduce the Millennium Problem to a single geometric question: does the Navier-Stokes dynamics itself produce scale-dependent depletion and biaxial strain in the high-enstrophy regime?]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Navier-Stokes equations, vortex stretching, geometric depletion, alignment weights, strain eigenvalues, formal verification, SMT solver, biaxial strain, depletion boundedness]

#v(1em)


= Introduction

In a companion paper [1], we showed that the 3D incompressible Navier-Stokes smoothness problem is undecidable within the scalar Sobolev framework: no inequality chain among the norms $E$, $Omega$, $H_(3 slash 2)$, $P$ can exclude blow-up. The critical finding was the exponent sum theorem: for any stretching bound $S^2 <= Omega^a P^b$, blow-up is excluded if and only if $a + b <= 2$. The standard bound has $a + b = 4$; the Kato-Ponce inequality [2] reduces this to $a + b = 3$; the remaining unit of exponent reduction must come from non-scalar structure.

That paper concluded with a sharpened open question: does the geometry of divergence-free flows on $TT^3$ force any stretching bound with effective $a + b <= 2$? The present paper addresses this question by decomposing the stretching integral into geometric variables and using the Z3 SMT solver to determine exactly which geometric conditions close the gap.

The key identity is classical [3, 4]:

$ omega dot S omega = |omega|^2 sum_i lambda_i alpha_i $

where $lambda_1 >= lambda_2 >= lambda_3$ are the eigenvalues of the strain rate tensor $S = 1 slash 2 (nabla v + nabla v^T)$, $e_i$ are the corresponding eigenvectors, $xi = omega slash |omega|$ is the vorticity direction, and $alpha_i = (xi dot e_i)^2$ are alignment weights satisfying $sum alpha_i = 1$. Scalar methods implicitly assume the worst case $alpha_1 = 1$; geometry may force $alpha_1 << 1$.

We formalize this decomposition in the Kleis formal verification language [5] and conduct thirteen Z3 satisfiability tests. The results identify the exact pair of geometric conditions — scale-dependent alignment depletion and biaxial strain — that would close the half-derivative gap, and establish an analytical boundedness theorem for the coupled dynamics.

= Geometric Decomposition of Stretching

The strain rate tensor $S_(i j) = 1 slash 2 (partial_i v_j + partial_j v_i)$ is symmetric and trace-free (by incompressibility: $nabla dot v = 0$). Its eigenvalues satisfy:

$ lambda_1 >= lambda_2 >= lambda_3 , quad lambda_1 + lambda_2 + lambda_3 = 0 $

The corresponding eigenvectors $e_1 , e_2 , e_3$ form an orthonormal basis. The alignment weights $alpha_i = (xi dot e_i)^2$, where $xi = omega slash |omega|$ is the vorticity direction, satisfy $alpha_1 + alpha_2 + alpha_3 = 1$ and $alpha_i >= 0$.

The stretching integral decomposes as:

$ S = integral omega dot S omega thin d x = integral |omega|^2 sum_i lambda_i alpha_i thin d x = Omega dot sigma_("eff") $

where $sigma_("eff") = sum_i lambda_i alpha_i$ is the effective stretching rate. The enstrophy equation becomes:

$ d Omega slash d t = Omega dot sigma_("eff") - 2 nu P $

In the scalar framework, $sigma_("eff")$ is bounded by $lambda_1 <= ||nabla v||_infinity <= sqrt(Omega dot P)$ (interpolation), giving $S^2 <= Omega^3 P$ ($a + b = 4$). The question is whether the geometric variables $alpha_i$ and $lambda_i$ permit a tighter bound.

== Geometric variables

We define the following observables for the decomposed stretching analysis.

#figure(
  table(columns: (auto, auto, auto), align: (left, left, left), table.header([*Variable*], [*Definition*], [*Constraint*]), [$lambda_1 , lambda_2 , lambda_3$], [Strain eigenvalues], [$lambda_1 >= lambda_2 >= lambda_3$, $sum lambda_i = 0$], [$alpha_1 , alpha_2 , alpha_3$], [Alignment weights $(xi dot e_i)^2$], [$alpha_i >= 0$, $sum alpha_i = 1$], [$sigma_("eff")$], [Effective stretching rate $sum lambda_i alpha_i$], [$sigma_("eff") >= 0$ for growth], [$A(t)$], [Bad alignment $integral |omega|^2 alpha_1 d x$], [$A <= Omega$], [$B(t)$], [Neutral alignment $integral |omega|^2 alpha_2 d x$], [$B <= Omega$]),
  caption: [Geometric variables used in the Z3 axiom systems. All are real-valued; alignment weights are on the unit sphere.]
) <tab:geo_variables>

= Alignment Weight Tests

We first test whether controlling the alignment weight $alpha_1$ — the fraction of enstrophy aligned with the most stretching eigenvector — is sufficient to close the gap. Each test uses a single Kleis structure with the decomposed stretching axiom, eigenvalue ordering, trace-free condition, the $lambda_1$ bound ($lambda_1^2 <= Omega dot P$), Poincaré ($P >= Omega$), the enstrophy equation ($d Omega slash d t = S - 2 nu P$ with $nu = 1$), and a growth condition ($d Omega slash d t > 0$). SAT means the axiom system is consistent with blow-up; UNSAT means blow-up is excluded.

*Test AW1* (worst-case, $alpha_1 = 1$): $S = Omega dot lambda_1$. This is the scalar worst case. Result: *SAT*. Expected — the standard bound $S^2 <= Omega^3 P$ admits blow-up.

*Test AW2* (constant depletion, $alpha_1 <= 1 slash 2$): Even with half the vorticity deflected from $e_1$, the remaining alignment plus the $lambda_2 alpha_2$ contribution is enough to sustain growth. Result: *SAT*.

*Test AW3* (scale-dependent depletion, $alpha_1 dot Omega <= 1$): The "bad alignment" decays inversely with enstrophy. This is the strongest pointwise control on $alpha_1$ alone. Result: *SAT*. Z3 finds a blow-up scenario where $lambda_2$ drives the stretching.

*Test AW4* (intermediate alignment, $alpha_2 = 1$): Vorticity fully aligned with $e_2$. Since $lambda_2$ can be positive (up to $lambda_2 = lambda_1$ in the isotropic case) and satisfies the same bound $lambda_2^2 <= Omega dot P$, the effective stretching $S = Omega dot lambda_2$ has the same worst-case scaling as $S = Omega dot lambda_1$. Result: *SAT*.

== Interpretation: the intermediate eigenvalue channel

All four tests return SAT, including AW3 where $alpha_1 <= 1 slash Omega$. The explanation is structural: bounding $alpha_1$ redirects vorticity toward $e_2$ and $e_3$, but the intermediate eigenvalue $lambda_2$ provides an _uncontrolled stretching channel_. Since $lambda_2$ can be positive (trace-free requires only $lambda_1 + lambda_2 + lambda_3 = 0$, not $lambda_2 <= 0$), and since $lambda_2$ satisfies the same magnitude bound as $lambda_1$, the effective stretching rate $sigma_("eff") = lambda_1 alpha_1 + lambda_2 alpha_2 + lambda_3 alpha_3$ retains the worst-case scaling even when $alpha_1$ is small.

This is a genuine finding: *controlling alignment alone does not change the exponent scaling, only the constants*. The exponent sum $a + b$ is invariant under redistribution of vorticity among eigenvectors as long as the eigenvalue bounds are unchanged.

#figure(
  table(columns: (auto, auto, auto), align: (left, left, center), table.header([*Test*], [*Alignment condition*], [*Result*]), [AW1], [Worst case: $alpha_1 = 1$], [SAT], [AW2], [Constant depletion: $alpha_1 <= 1 slash 2$], [SAT], [AW3], [Scale-dependent: $alpha_1 dot Omega <= 1$], [SAT], [AW4], [Intermediate only: $alpha_2 = 1$], [SAT]),
  caption: [Alignment weight tests. Controlling $alpha_1$ alone does not close the gap. The intermediate eigenvalue $lambda_2$ provides an uncontrolled stretching channel.]
) <tab:alignment>

= Exponent Threshold Scan

We verify the sharpness of the $a + b = 2$ threshold from [1] by scanning the exponent sum with direct stretching bounds $S^2 <= Omega^a P^b$ at four values, using the same framework (Poincaré, enstrophy equation, growth condition, $Omega > 10$, $nu = 1$).

*Test D6a* ($a + b = 2.5$): $S^2 <= Omega^(1.5) dot P$ → *SAT*.

*Test D6b* ($a + b = 2.25$): $S^2 <= Omega^(1.25) dot P$ → *SAT*.

*Test D6c* ($a + b = 2.1$): $S^2 <= Omega^(1.1) dot P$ → *SAT*.

*Test D6d* ($a + b = 2.0$): $S^2 <= Omega dot P$ → *UNSAT*.

The transition is instantaneous: there is no gradual weakening of blow-up scenarios as $a + b$ decreases toward $2$. At $a + b = 2.1$, Z3 finds a consistent blow-up model; at $a + b = 2.0$, the axioms become contradictory. This confirms the critical exponent sum theorem of [1] and establishes that the required geometric gain is _exactly_ one unit in exponent sum — no less will do.

#figure(
  table(columns: (auto, auto, auto, auto), align: (left, center, center, center), table.header([*Test*], [*Bound*], [*$a + b$*], [*Result*]), [D6a], [$S^2 <= Omega^(1.5) P$], [$2.5$], [SAT], [D6b], [$S^2 <= Omega^(1.25) P$], [$2.25$], [SAT], [D6c], [$S^2 <= Omega^(1.1) P$], [$2.1$], [SAT], [D6d], [$S^2 <= Omega P$], [$2.0$], [*UNSAT*]),
  caption: [Exponent threshold scan. The phase transition at $a + b = 2$ is sharp: every value above $2$ admits blow-up.]
) <tab:threshold>

= Necessary and Sufficient Geometric Conditions

We now identify the exact geometric conditions that close the gap. The decomposed stretching is $S = Omega dot (lambda_1 alpha_1 + lambda_2 alpha_2 + lambda_3 alpha_3)$. We test three combinations of geometric constraints, each with the full eigenvalue structure (ordering, trace-free, $lambda_1^2 <= Omega dot P$), Poincaré, enstrophy equation, $Omega > 10$, $nu = 1$, and a growth condition.

*Test D7* (scale-dependent depletion + biaxial strain): $alpha_1 dot Omega <= 1$ *and* $lambda_2 <= 0$. Result: *UNSAT*. Growth is impossible. Both conditions together close the gap.

*Test D8* (scale-dependent depletion alone): $alpha_1 dot Omega <= 1$, $lambda_2$ unconstrained. Result: *SAT*. Depletion alone is insufficient — the $lambda_2 alpha_2$ channel sustains stretching.

*Test D9* (biaxial strain alone): $lambda_2 <= 0$, $alpha_1$ unconstrained. Result: *SAT*. Biaxial strain alone is insufficient — vorticity can still align with $e_1$ and get stretched.

== Algebraic verification

The UNSAT result of Test D7 follows from a clean algebraic chain. With $lambda_2 <= 0$ (biaxial strain), the only positive contribution to $sigma_("eff")$ comes from $lambda_1 alpha_1$, since $lambda_2 alpha_2 <= 0$ and $lambda_3 alpha_3 <= 0$ (both $lambda_2 , lambda_3 <= 0$ from trace-free and ordering). Therefore:

$ sigma_("eff") = lambda_1 alpha_1 + lambda_2 alpha_2 + lambda_3 alpha_3 <= lambda_1 alpha_1 $

With scale-dependent depletion $alpha_1 <= 1 slash Omega$:

$ sigma_("eff") <= lambda_1 slash Omega <= sqrt(Omega dot P) slash Omega = sqrt(P slash Omega) $

Therefore:

$ S = Omega dot sigma_("eff") <= Omega dot sqrt(P slash Omega) = sqrt(Omega dot P) $

Hence $S^2 <= Omega dot P$, which is exactly $a + b = 2$ — the critical threshold for global basin collapse.

The two conditions are *individually necessary*: removing either one reintroduces a stretching channel with $a + b > 2$. Scale-dependent depletion controls the $lambda_1 alpha_1$ term but leaves $lambda_2 alpha_2$ uncontrolled. Biaxial strain eliminates $lambda_2 alpha_2$ but leaves $lambda_1 alpha_1$ at worst-case strength.

#figure(
  table(columns: (auto, auto, auto, auto), align: (left, left, left, center), table.header([*Test*], [*Depletion*], [*Strain*], [*Result*]), [D7], [$alpha_1 Omega <= 1$], [$lambda_2 <= 0$ (biaxial)], [*UNSAT*], [D8], [$alpha_1 Omega <= 1$], [unconstrained], [SAT], [D9], [unconstrained], [$lambda_2 <= 0$ (biaxial)], [SAT]),
  caption: [Necessary and sufficient geometric conditions. Neither condition alone suffices; both together imply $S^2 <= Omega P$ ($a + b = 2$).]
) <tab:necessary>

= Depletion Boundedness Theorem

We now consider the dynamics of depletion. Suppose the alignment weight $alpha_1$ is not fixed but evolves according to a depletion law $d alpha_1 slash d t = -kappa alpha_1 Omega$, where $kappa > 0$ is the depletion rate. This models the tendency of vorticity to be rotated away from the stretching direction at a rate proportional to the local strain intensity. Under the biaxial assumption ($lambda_2 <= 0$), the effective stretching is $S approx Omega^2 alpha_1$ (using $lambda_1 approx Omega$ in the Poincaré worst case). The coupled system is:

$ d Omega slash d t = Omega^2 alpha_1 - 2 nu Omega , quad d alpha_1 slash d t = -kappa alpha_1 Omega $

*Theorem (Depletion Boundedness).* _Let $(Omega(t) , alpha_1(t))$ be a solution of the coupled system with $Omega(0) = Omega_0 > 0$, $alpha_1(0) = alpha_0 > 0$, $nu > 0$, $kappa > 0$. Then:_

_(i)_ $Omega(t) <= Omega_0 dot exp(alpha_0 slash kappa - 2 nu t)$ _for all_ $t >= 0$.

_(ii)_ $Omega(t) -> 0$ _as_ $t -> infinity$.

_(iii)_ _There is no finite-time blow-up._

*Proof.* The enstrophy equation gives $d ln Omega slash d t = Omega alpha_1 - 2 nu$, so:

$ ln(Omega(t) slash Omega_0) = integral_0^t (Omega(s) alpha_1(s) - 2 nu) thin d s $

The depletion law gives $alpha_1(t) = alpha_0 exp(-kappa integral_0^t Omega(s) thin d s)$. Define $F(t) = integral_0^t Omega(s) thin d s$, so $F'(t) = Omega(t)$. Then:

$ integral_0^t Omega(s) alpha_1(s) thin d s = alpha_0 integral_0^t F'(s) exp(-kappa F(s)) thin d s = (alpha_0 slash kappa)(1 - exp(-kappa F(t))) <= alpha_0 slash kappa $

This bound is _independent of the trajectory_. It holds whether $Omega$ grows, oscillates, or decays — the substitution $u = F(s)$ converts the integral to $integral exp(-kappa u) thin d u = 1 slash kappa$ regardless of the time-parametrization. Therefore:

$ ln(Omega(t) slash Omega_0) <= alpha_0 slash kappa - 2 nu t $

so $Omega(t) <= Omega_0 exp(alpha_0 slash kappa) exp(-2 nu t) -> 0$ as $t -> infinity$. #h(1fr) $square$

The theorem has three consequences.

First, *any positive depletion rate prevents blow-up*. The depletion need not be strong — it only needs to be present. The total stretching "budget" $integral_0^infinity Omega alpha_1 thin d s <= alpha_0 slash kappa$ is finite, and the ever-present dissipation $2 nu Omega$ eventually dominates.

Second, *the transient peak is bounded*: $Omega_("max") <= Omega_0 exp(alpha_0 slash kappa)$. For small $kappa$, the peak is enormous (exponential in $1 slash kappa$), explaining why numerical ODE solvers fail for weak depletion — the trajectory comes arbitrarily close to blow-up before reversing.

Third, *the bound is trajectory-independent*. The key substitution $F = integral Omega thin d t$ converts the integral to a form that does not depend on the growth rate of $Omega$. This is why the result holds for _any_ $kappa > 0$: the depletion mechanism "spends" the alignment budget at a rate proportional to $Omega$ itself, ensuring the total is always finite.

== Numerical verification

We verify the theorem by integrating the coupled ODE with $Omega_0 = 50$, $alpha_0 = 0.8$, $nu = 1$, and compare four cases: no depletion ($alpha_1 = 1$ fixed), constant depletion ($alpha_1 = 0.1$ fixed), strong dynamic depletion ($kappa = 0.1$), and very strong dynamic depletion ($kappa = 0.5$).

Without depletion, enstrophy blows up at $t approx 0.021$. With constant $alpha_1 = 0.1$, blow-up is delayed to $t approx 0.33$ but still occurs — constant depletion does not change the exponent sum.

With dynamic depletion at $kappa = 0.1$, the trajectory reverses: enstrophy peaks and then decays to $Omega = 6.78$ at $t = 5$, with $alpha_1 = 2.4 times 10^(-8)$. At $kappa = 0.5$, the peak is smaller and the decay faster: $Omega = 0.011$ at $t = 5$.

For $kappa = 0.5$, the predicted bound is $Omega_("max") <= 50 dot exp(0.8 slash 0.5) = 50 dot exp(1.6) approx 248$. The observed peak is $Omega approx 216$, confirming the bound (Figure 1). By $t = 10$, enstrophy has decayed to $Omega approx 5.7 times 10^(-7)$, confirming asymptotic decay (Figure 2).

#figure(
  lq.diagram(
  width: 10cm,
  title: [Enstrophy with geometric depletion],
  xlabel: [$t$],
  ylabel: [$Omega(t)$],
  legend: (position: right + top),
  lq.plot(
    (0.000000, 0.010000, 0.020000, 0.030000, 0.040000, 0.050000, 0.060000, 0.070000, 0.080000, 0.090000, 0.100000, 0.110000, 0.120000, 0.130000, 0.140000, 0.150000, 0.160000, 0.170000, 0.180000, 0.190000, 0.200000, 0.210000, 0.220000, 0.230000, 0.240000, 0.250000, 0.260000, 0.270000, 0.280000, 0.290000, 0.300000, 0.310000, 0.320000, 0.330000, 0.340000, 0.350000, 0.360000, 0.370000, 0.380000, 0.390000, 0.400000, 0.410000, 0.420000, 0.430000, 0.440000, 0.450000, 0.460000, 0.470000, 0.480000, 0.490000, 0.500000, 0.510000, 0.520000, 0.530000, 0.540000, 0.550000, 0.560000, 0.570000, 0.580000, 0.590000, 0.600000, 0.610000, 0.620000, 0.630000, 0.640000, 0.650000, 0.660000, 0.670000, 0.680000, 0.690000, 0.700000, 0.710000, 0.720000, 0.730000, 0.740000, 0.750000, 0.760000, 0.770000, 0.780000, 0.790000, 0.800000, 0.810000, 0.820000, 0.830000, 0.840000, 0.850000, 0.860000, 0.870000, 0.880000, 0.890000, 0.900000, 0.910000, 0.920000, 0.930000, 0.940000, 0.950000, 0.960000, 0.970000, 0.980000, 0.990000, 1.000000, 1.010000, 1.020000, 1.030000, 1.040000, 1.050000, 1.060000, 1.070000, 1.080000, 1.090000, 1.100000, 1.110000, 1.120000, 1.130000, 1.140000, 1.150000, 1.160000, 1.170000, 1.180000, 1.190000, 1.200000, 1.210000, 1.220000, 1.230000, 1.240000, 1.250000, 1.260000, 1.270000, 1.280000, 1.290000, 1.300000, 1.310000, 1.320000, 1.330000, 1.340000, 1.350000, 1.360000, 1.370000, 1.380000, 1.390000, 1.400000, 1.410000, 1.420000, 1.430000, 1.440000, 1.450000, 1.460000, 1.470000, 1.480000, 1.490000, 1.500000, 1.510000, 1.520000, 1.530000, 1.540000, 1.550000, 1.560000, 1.570000, 1.580000, 1.590000, 1.600000, 1.610000, 1.620000, 1.630000, 1.640000, 1.650000, 1.660000, 1.670000, 1.680000, 1.690000, 1.700000, 1.710000, 1.720000, 1.730000, 1.740000, 1.750000, 1.760000, 1.770000, 1.780000, 1.790000, 1.800000, 1.810000, 1.820000, 1.830000, 1.840000, 1.850000, 1.860000, 1.870000, 1.880000, 1.890000, 1.900000, 1.910000, 1.920000, 1.930000, 1.940000, 1.950000, 1.960000, 1.970000, 1.980000, 1.990000, 2.000000, 2.010000, 2.020000, 2.030000, 2.040000, 2.050000, 2.060000, 2.070000, 2.080000, 2.090000, 2.100000, 2.110000, 2.120000, 2.130000, 2.140000, 2.150000, 2.160000, 2.170000, 2.180000, 2.190000, 2.200000, 2.210000, 2.220000, 2.230000, 2.240000, 2.250000, 2.260000, 2.270000, 2.280000, 2.290000, 2.300000, 2.310000, 2.320000, 2.330000, 2.340000, 2.350000, 2.360000, 2.370000, 2.380000, 2.390000, 2.400000, 2.410000, 2.420000, 2.430000, 2.440000, 2.450000, 2.460000, 2.470000, 2.480000, 2.490000, 2.500000, 2.510000, 2.520000, 2.530000, 2.540000, 2.550000, 2.560000, 2.570000, 2.580000, 2.590000, 2.600000, 2.610000, 2.620000, 2.630000, 2.640000, 2.650000, 2.660000, 2.670000, 2.680000, 2.690000, 2.700000, 2.710000, 2.720000, 2.730000, 2.740000, 2.750000, 2.760000, 2.770000, 2.780000, 2.790000, 2.800000, 2.810000, 2.820000, 2.830000, 2.840000, 2.850000, 2.860000, 2.870000, 2.880000, 2.890000, 2.900000, 2.910000, 2.920000, 2.930000, 2.940000, 2.950000, 2.960000, 2.970000, 2.980000, 2.990000, 3.000000, 3.010000, 3.020000, 3.030000, 3.040000, 3.050000, 3.060000, 3.070000, 3.080000, 3.090000, 3.100000, 3.110000, 3.120000, 3.130000, 3.140000, 3.150000, 3.160000, 3.170000, 3.180000, 3.190000, 3.200000, 3.210000, 3.220000, 3.230000, 3.240000, 3.250000, 3.260000, 3.270000, 3.280000, 3.290000, 3.300000, 3.310000, 3.320000, 3.330000, 3.340000, 3.350000, 3.360000, 3.370000, 3.380000, 3.390000, 3.400000, 3.410000, 3.420000, 3.430000, 3.440000, 3.450000, 3.460000, 3.470000, 3.480000, 3.490000, 3.500000, 3.510000, 3.520000, 3.530000, 3.540000, 3.550000, 3.560000, 3.570000, 3.580000, 3.590000, 3.600000, 3.610000, 3.620000, 3.630000, 3.640000, 3.650000, 3.660000, 3.670000, 3.680000, 3.690000, 3.700000, 3.710000, 3.720000, 3.730000, 3.740000, 3.750000, 3.760000, 3.770000, 3.780000, 3.790000, 3.800000, 3.810000, 3.820000, 3.830000, 3.840000, 3.850000, 3.860000, 3.870000, 3.880000, 3.890000, 3.900000, 3.910000, 3.920000, 3.930000, 3.940000, 3.950000, 3.960000, 3.970000, 3.980000, 3.990000, 4.000000, 4.010000, 4.020000, 4.030000, 4.040000, 4.050000, 4.060000, 4.070000, 4.080000, 4.090000, 4.100000, 4.110000, 4.120000, 4.130000, 4.140000, 4.150000, 4.160000, 4.170000, 4.180000, 4.190000, 4.200000, 4.210000, 4.220000, 4.230000, 4.240000, 4.250000, 4.260000, 4.270000, 4.280000, 4.290000, 4.300000, 4.310000, 4.320000, 4.330000, 4.340000, 4.350000, 4.360000, 4.370000, 4.380000, 4.390000, 4.400000, 4.410000, 4.420000, 4.430000, 4.440000, 4.450000, 4.460000, 4.470000, 4.480000, 4.490000, 4.500000, 4.510000, 4.520000, 4.530000, 4.540000, 4.550000, 4.560000, 4.570000, 4.580000, 4.590000, 4.600000, 4.610000, 4.620000, 4.630000, 4.640000, 4.650000, 4.660000, 4.670000, 4.680000, 4.690000, 4.700000, 4.710000, 4.720000, 4.730000, 4.740000, 4.750000, 4.760000, 4.770000, 4.780000, 4.790000, 4.800000, 4.810000, 4.820000, 4.830000, 4.840000, 4.850000, 4.860000, 4.870000, 4.880000, 4.890000, 4.900000, 4.910000, 4.920000, 4.930000, 4.940000, 4.950000, 4.960000, 4.970000, 4.980000, 4.990000, 5.000000),
    (50.000000, 79.670892, 179.257055, 13432.621700, 137673.577389, 134948.079063, 132273.780191, 129656.279032, 127088.983410, 124572.725046, 122105.775613, 119687.189391, 117316.830016, 114993.173156, 112718.402751, 110485.164073, 108296.473479, 106153.805917, 104050.398350, 101991.887902, 99970.870768, 97991.612933, 96050.229179, 94149.244937, 92284.652328, 90458.740667, 88667.427945, 86910.795142, 85189.924493, 83503.526212, 81849.988615, 80227.894130, 78639.752014, 77083.651679, 75557.661038, 74060.110699, 72594.746819, 71156.532606, 69747.548185, 68366.487868, 67013.037105, 65686.296892, 64385.139687, 63110.542657, 61861.299446, 60635.397010, 59435.014714, 58257.931768, 57104.835127, 55973.862411, 54864.672606, 53779.383909, 52714.726496, 51670.806420, 50647.180423, 49643.949871, 48661.218171, 47698.035098, 46753.850506, 45827.945929, 44920.317189, 44030.744492, 43158.575037, 42304.030122, 41466.618034, 40645.233367, 39840.420842, 39051.468047, 38278.515132, 37520.303289, 36777.621624, 36049.237068, 35335.181293, 34635.994597, 33949.504873, 33277.478438, 32618.588634, 31972.695463, 31339.831844, 30719.078976, 30110.670330, 29514.418586, 28930.131629, 28357.416275, 27795.605034, 27245.314697, 26706.006712, 26177.185017, 25658.463301, 25150.608434, 24652.757546, 24164.520588, 23685.865058, 23216.752484, 22757.179806, 22306.603778, 21865.057444, 21431.938578, 21007.859225, 20591.659822, 20183.832656, 19784.139959, 19392.451989, 19008.546818, 18632.124867, 18263.078599, 17901.583942, 17547.212505, 17199.453266, 16858.911676, 16525.041227, 16197.755920, 15877.148600, 15562.871256, 15254.656429, 14952.590059, 14656.516054, 14366.345807, 14081.670271, 13803.001267, 13529.755852, 13261.850503, 12999.244802, 12741.736654, 12489.301452, 12242.136783, 11999.789390, 11761.974351, 11529.289974, 11300.907216, 11077.201032, 10857.928878, 10642.780363, 10431.943120, 10225.558893, 10023.078250, 9824.576061, 9629.997910, 9439.316175, 9252.358603, 9069.224243, 8889.704270, 8713.584418, 8541.155181, 8371.940797, 8206.205048, 8043.687406, 7884.443469, 7728.236046, 7575.194808, 7425.280610, 7278.258026, 7134.062955, 6992.805040, 6854.266490, 6718.645805, 6585.593194, 6455.127001, 6327.376060, 6202.025201, 6079.262740, 5958.939267, 5840.931137, 5725.251858, 5611.858550, 5500.782458, 5391.850457, 5285.068210, 5180.430930, 5077.818885, 4977.340891, 4878.694560, 4782.144261, 4687.423129, 4594.628691, 4503.681973, 4414.435914, 4327.069453, 4241.408856, 4157.384584, 4075.029317, 3994.385083, 3915.307982, 3837.787327, 3761.762830, 3687.249909, 3614.250569, 3542.684170, 3472.509677, 3403.741457, 3336.371176, 3270.322003, 3205.569713, 3142.070938, 3079.819910, 3018.849457, 2959.089376, 2900.503161, 2843.080046, 2786.810077, 2731.602883, 2677.470587, 2624.471368, 2572.518955, 2521.590960, 2471.665313, 2422.688209, 2374.754815, 2327.720286, 2281.598592, 2236.437726, 2192.164929, 2148.753543, 2106.198540, 2064.517603, 2023.593323, 1983.567931, 1944.249808, 1905.769468, 1868.037046, 1831.038544, 1794.794308, 1759.243436, 1724.425745, 1690.252450, 1656.817402, 1623.969660, 1591.842640, 1560.299451, 1529.420214, 1499.126296, 1469.451919, 1440.347545, 1411.835061, 1383.876161, 1356.466243, 1329.624607, 1303.278232, 1277.471213, 1252.191555, 1227.373394, 1203.081539, 1179.260375, 1155.903522, 1133.022760, 1110.583113, 1088.593742, 1067.042097, 1045.908125, 1025.197637, 1004.905126, 985.003663, 965.489441, 946.377911, 927.646859, 909.271087, 891.255245, 873.615991, 856.322395, 839.359994, 822.736317, 806.450558, 790.482864, 774.826999, 759.484705, 744.448667, 729.707965, 715.256628, 701.092434, 687.212298, 673.606506, 660.267696, 647.190894, 634.373514, 621.814034, 609.504453, 597.436764, 585.605157, 574.005862, 562.637152, 551.498065, 540.580704, 529.878135, 519.385247, 509.098325, 499.015046, 489.134258, 479.450583, 469.958156, 460.652700, 451.530508, 442.588442, 433.823933, 425.234292, 416.814862, 408.561913, 400.472005, 392.541898, 384.768554, 377.149137, 369.681011, 362.361203, 355.186363, 348.153517, 341.259786, 334.502382, 327.878613, 321.385876, 315.021664, 308.783563, 302.669249, 296.676216, 290.801867, 285.043818, 279.399738, 273.867352, 268.444433, 263.128813, 257.918375, 252.811053, 247.804839, 242.897774, 238.087955, 233.373532, 228.752539, 224.223014, 219.783158, 215.431201, 211.165408, 206.984075, 202.885529, 198.868131, 194.930273, 191.070380, 187.286909, 183.578347, 179.943216, 176.380068, 172.887490, 169.464097, 166.108514, 162.819289, 159.595121, 156.434755, 153.336954, 150.300499, 147.324191, 144.406853, 141.547323, 138.744462, 135.997147, 133.304279, 130.664773, 128.077568, 125.541619, 123.055902, 120.619413, 118.231166, 115.890194, 113.595551, 111.346309, 109.141561, 106.980355, 104.861614, 102.784545, 100.748391, 98.752403, 96.795842, 94.877978, 92.998088, 91.155460, 89.349391, 87.579186, 85.844159, 84.143632, 82.476937, 80.843416, 79.242418, 77.673302, 76.135435, 74.628193, 73.150963, 71.703137, 70.284120, 68.893323, 67.530167, 66.194082, 64.884507, 63.600889, 62.342686, 61.109362, 59.900392, 58.715259, 57.553456, 56.414483, 55.297850, 54.203077, 53.129600, 52.076838, 51.044454, 50.032120, 49.039508, 48.066294, 47.112156, 46.176775, 45.259833, 44.361017, 43.480014, 42.616513, 41.770209, 40.940795, 40.127969, 39.331431, 38.550884, 37.786031, 37.036580, 36.302241, 35.582725, 34.877746, 34.187021, 33.510270, 32.847213, 32.197575, 31.561083, 30.937463, 30.326449, 29.727773, 29.141172, 28.566383, 28.003147, 27.451209, 26.910312, 26.380206, 25.860640, 25.351368, 24.852145, 24.362728, 23.882877, 23.412355, 22.950926, 22.498359, 22.054422, 21.618888, 21.191530, 20.772129, 20.360687, 19.957204, 19.561551, 19.173599, 18.793222, 18.420294, 18.054689, 17.696283, 17.344953, 17.000577, 16.663034, 16.332203, 16.007965, 15.690201, 15.378796, 15.073631, 14.774593, 14.481566, 14.194438, 13.913096, 13.637428, 13.367324, 13.102676, 12.843373, 12.589310, 12.340379, 12.096475, 11.857494, 11.623332, 11.393887, 11.169056, 10.948740, 10.732840, 10.521255, 10.313889, 10.110646, 9.911429, 9.716144, 9.524697, 9.336996, 9.152949, 8.972465, 8.795455, 8.621830, 8.451501, 8.284383, 8.120396, 7.959601, 7.801990, 7.647500, 7.496070, 7.347638, 7.202145, 7.059533, 6.919745, 6.782725),
    color: blue,
    label: [Dynamic ($kappa = 0.1$)]
  ),
  lq.plot(
    (0.000000, 0.010000, 0.020000, 0.030000, 0.040000, 0.050000, 0.060000, 0.070000, 0.080000, 0.090000, 0.100000, 0.110000, 0.120000, 0.130000, 0.140000, 0.150000, 0.160000, 0.170000, 0.180000, 0.190000, 0.200000, 0.210000, 0.220000, 0.230000, 0.240000, 0.250000, 0.260000, 0.270000, 0.280000, 0.290000, 0.300000, 0.310000, 0.320000, 0.330000, 0.340000, 0.350000, 0.360000, 0.370000, 0.380000, 0.390000, 0.400000, 0.410000, 0.420000, 0.430000, 0.440000, 0.450000, 0.460000, 0.470000, 0.480000, 0.490000, 0.500000, 0.510000, 0.520000, 0.530000, 0.540000, 0.550000, 0.560000, 0.570000, 0.580000, 0.590000, 0.600000, 0.610000, 0.620000, 0.630000, 0.640000, 0.650000, 0.660000, 0.670000, 0.680000, 0.690000, 0.700000, 0.710000, 0.720000, 0.730000, 0.740000, 0.750000, 0.760000, 0.770000, 0.780000, 0.790000, 0.800000, 0.810000, 0.820000, 0.830000, 0.840000, 0.850000, 0.860000, 0.870000, 0.880000, 0.890000, 0.900000, 0.910000, 0.920000, 0.930000, 0.940000, 0.950000, 0.960000, 0.970000, 0.980000, 0.990000, 1.000000, 1.010000, 1.020000, 1.030000, 1.040000, 1.050000, 1.060000, 1.070000, 1.080000, 1.090000, 1.100000, 1.110000, 1.120000, 1.130000, 1.140000, 1.150000, 1.160000, 1.170000, 1.180000, 1.190000, 1.200000, 1.210000, 1.220000, 1.230000, 1.240000, 1.250000, 1.260000, 1.270000, 1.280000, 1.290000, 1.300000, 1.310000, 1.320000, 1.330000, 1.340000, 1.350000, 1.360000, 1.370000, 1.380000, 1.390000, 1.400000, 1.410000, 1.420000, 1.430000, 1.440000, 1.450000, 1.460000, 1.470000, 1.480000, 1.490000, 1.500000, 1.510000, 1.520000, 1.530000, 1.540000, 1.550000, 1.560000, 1.570000, 1.580000, 1.590000, 1.600000, 1.610000, 1.620000, 1.630000, 1.640000, 1.650000, 1.660000, 1.670000, 1.680000, 1.690000, 1.700000, 1.710000, 1.720000, 1.730000, 1.740000, 1.750000, 1.760000, 1.770000, 1.780000, 1.790000, 1.800000, 1.810000, 1.820000, 1.830000, 1.840000, 1.850000, 1.860000, 1.870000, 1.880000, 1.890000, 1.900000, 1.910000, 1.920000, 1.930000, 1.940000, 1.950000, 1.960000, 1.970000, 1.980000, 1.990000, 2.000000, 2.010000, 2.020000, 2.030000, 2.040000, 2.050000, 2.060000, 2.070000, 2.080000, 2.090000, 2.100000, 2.110000, 2.120000, 2.130000, 2.140000, 2.150000, 2.160000, 2.170000, 2.180000, 2.190000, 2.200000, 2.210000, 2.220000, 2.230000, 2.240000, 2.250000, 2.260000, 2.270000, 2.280000, 2.290000, 2.300000, 2.310000, 2.320000, 2.330000, 2.340000, 2.350000, 2.360000, 2.370000, 2.380000, 2.390000, 2.400000, 2.410000, 2.420000, 2.430000, 2.440000, 2.450000, 2.460000, 2.470000, 2.480000, 2.490000, 2.500000, 2.510000, 2.520000, 2.530000, 2.540000, 2.550000, 2.560000, 2.570000, 2.580000, 2.590000, 2.600000, 2.610000, 2.620000, 2.630000, 2.640000, 2.650000, 2.660000, 2.670000, 2.680000, 2.690000, 2.700000, 2.710000, 2.720000, 2.730000, 2.740000, 2.750000, 2.760000, 2.770000, 2.780000, 2.790000, 2.800000, 2.810000, 2.820000, 2.830000, 2.840000, 2.850000, 2.860000, 2.870000, 2.880000, 2.890000, 2.900000, 2.910000, 2.920000, 2.930000, 2.940000, 2.950000, 2.960000, 2.970000, 2.980000, 2.990000, 3.000000, 3.010000, 3.020000, 3.030000, 3.040000, 3.050000, 3.060000, 3.070000, 3.080000, 3.090000, 3.100000, 3.110000, 3.120000, 3.130000, 3.140000, 3.150000, 3.160000, 3.170000, 3.180000, 3.190000, 3.200000, 3.210000, 3.220000, 3.230000, 3.240000, 3.250000, 3.260000, 3.270000, 3.280000, 3.290000, 3.300000, 3.310000, 3.320000, 3.330000, 3.340000, 3.350000, 3.360000, 3.370000, 3.380000, 3.390000, 3.400000, 3.410000, 3.420000, 3.430000, 3.440000, 3.450000, 3.460000, 3.470000, 3.480000, 3.490000, 3.500000, 3.510000, 3.520000, 3.530000, 3.540000, 3.550000, 3.560000, 3.570000, 3.580000, 3.590000, 3.600000, 3.610000, 3.620000, 3.630000, 3.640000, 3.650000, 3.660000, 3.670000, 3.680000, 3.690000, 3.700000, 3.710000, 3.720000, 3.730000, 3.740000, 3.750000, 3.760000, 3.770000, 3.780000, 3.790000, 3.800000, 3.810000, 3.820000, 3.830000, 3.840000, 3.850000, 3.860000, 3.870000, 3.880000, 3.890000, 3.900000, 3.910000, 3.920000, 3.930000, 3.940000, 3.950000, 3.960000, 3.970000, 3.980000, 3.990000, 4.000000, 4.010000, 4.020000, 4.030000, 4.040000, 4.050000, 4.060000, 4.070000, 4.080000, 4.090000, 4.100000, 4.110000, 4.120000, 4.130000, 4.140000, 4.150000, 4.160000, 4.170000, 4.180000, 4.190000, 4.200000, 4.210000, 4.220000, 4.230000, 4.240000, 4.250000, 4.260000, 4.270000, 4.280000, 4.290000, 4.300000, 4.310000, 4.320000, 4.330000, 4.340000, 4.350000, 4.360000, 4.370000, 4.380000, 4.390000, 4.400000, 4.410000, 4.420000, 4.430000, 4.440000, 4.450000, 4.460000, 4.470000, 4.480000, 4.490000, 4.500000, 4.510000, 4.520000, 4.530000, 4.540000, 4.550000, 4.560000, 4.570000, 4.580000, 4.590000, 4.600000, 4.610000, 4.620000, 4.630000, 4.640000, 4.650000, 4.660000, 4.670000, 4.680000, 4.690000, 4.700000, 4.710000, 4.720000, 4.730000, 4.740000, 4.750000, 4.760000, 4.770000, 4.780000, 4.790000, 4.800000, 4.810000, 4.820000, 4.830000, 4.840000, 4.850000, 4.860000, 4.870000, 4.880000, 4.890000, 4.900000, 4.910000, 4.920000, 4.930000, 4.940000, 4.950000, 4.960000, 4.970000, 4.980000, 4.990000, 5.000000),
    (50.000000, 74.794582, 113.631255, 160.894795, 196.735564, 212.487292, 215.791550, 214.094161, 210.716509, 206.833987, 202.837530, 198.855594, 194.930185, 191.074776, 187.292920, 183.584845, 179.949888, 176.386751, 172.894037, 169.470577, 166.114866, 162.825520, 159.601326, 156.441116, 153.343466, 150.307021, 147.330581, 144.413108, 141.553655, 138.750891, 136.003468, 133.310258, 130.670344, 128.082967, 125.546911, 123.060972, 120.624125, 118.235483, 115.894277, 113.599506, 111.350134, 109.145240, 106.983962, 104.865499, 102.789062, 100.753747, 98.758713, 96.803151, 94.886286, 93.007374, 91.165704, 89.360534, 87.591109, 85.856708, 84.156631, 82.490198, 80.856749, 79.255647, 77.686274, 76.148007, 74.640202, 73.162250, 71.713552, 70.293528, 68.901608, 67.537241, 66.199888, 64.889023, 63.604137, 62.344707, 61.110214, 59.900162, 58.714068, 57.551455, 56.411858, 55.294822, 54.199902, 53.126661, 52.074674, 51.043525, 50.032801, 49.042086, 48.070986, 47.119115, 46.186093, 45.271547, 44.375112, 43.496428, 42.635144, 41.790915, 40.963402, 40.152275, 39.357208, 38.577885, 37.813985, 37.065201, 36.331237, 35.611805, 34.906621, 34.215405, 33.537882, 32.873782, 32.222839, 31.584792, 30.959384, 30.346366, 29.745489, 29.156511, 28.579196, 28.013309, 27.458624, 26.914917, 26.381968, 25.859532, 25.347398, 24.845374, 24.353270, 23.870898, 23.398075, 22.934618, 22.480349, 22.035090, 21.598666, 21.170906, 20.751640, 20.340702, 19.937926, 19.543152, 19.156219, 18.776970, 18.405252, 18.040913, 17.683802, 17.333773, 16.990682, 16.654387, 16.324748, 16.001629, 15.684895, 15.374414, 15.070057, 14.771698, 14.479209, 14.192397, 13.911126, 13.635307, 13.364851, 13.099671, 12.839681, 12.584792, 12.334920, 12.089979, 11.849884, 11.614553, 11.383901, 11.157846, 10.936307, 10.719201, 10.506449, 10.297970, 10.093685, 9.893516, 9.697384, 9.505212, 9.316924, 9.132443, 8.951693, 8.774602, 8.601093, 8.431094, 8.264532, 8.101335, 7.941431, 7.784749, 7.631220, 7.480774, 7.333342, 7.188856, 7.047248, 6.908451, 6.772399, 6.639026, 6.508268, 6.380061, 6.254339, 6.131042, 6.010105, 5.891468, 5.775070, 5.660849, 5.548762, 5.438831, 5.331029, 5.225323, 5.121677, 5.020057, 4.920430, 4.822761, 4.727018, 4.633168, 4.541177, 4.451014, 4.362645, 4.276039, 4.191164, 4.107988, 4.026481, 3.946611, 3.868347, 3.791660, 3.716519, 3.642894, 3.570756, 3.500074, 3.430821, 3.362967, 3.296484, 3.231342, 3.167515, 3.104975, 3.043693, 2.983644, 2.924799, 2.867132, 2.810616, 2.755226, 2.700935, 2.647719, 2.595550, 2.544405, 2.494258, 2.445085, 2.396862, 2.349564, 2.303168, 2.257649, 2.212986, 2.169163, 2.126184, 2.084039, 2.042713, 2.002193, 1.962466, 1.923517, 1.885335, 1.847906, 1.811216, 1.775254, 1.740007, 1.705461, 1.671605, 1.638426, 1.605911, 1.574048, 1.542826, 1.512232, 1.482254, 1.452880, 1.424100, 1.395900, 1.368271, 1.341199, 1.314675, 1.288686, 1.263222, 1.238272, 1.213825, 1.189869, 1.166396, 1.143393, 1.120850, 1.098758, 1.077105, 1.055882, 1.035079, 1.014685, 0.994691, 0.975087, 0.955863, 0.937011, 0.918520, 0.900381, 0.882585, 0.865123, 0.847990, 0.831188, 0.814712, 0.798556, 0.782715, 0.767184, 0.751957, 0.737031, 0.722398, 0.708055, 0.693997, 0.680218, 0.666713, 0.653478, 0.640507, 0.627796, 0.615341, 0.603135, 0.591175, 0.579457, 0.567974, 0.556723, 0.545700, 0.534899, 0.524316, 0.513947, 0.503788, 0.493834, 0.484080, 0.474523, 0.465159, 0.455982, 0.446990, 0.438178, 0.429541, 0.421077, 0.412780, 0.404647, 0.396674, 0.388858, 0.381194, 0.373679, 0.366308, 0.359079, 0.351988, 0.345030, 0.338203, 0.331505, 0.324936, 0.318494, 0.312178, 0.305985, 0.299914, 0.293961, 0.288126, 0.282406, 0.276799, 0.271303, 0.265916, 0.260637, 0.255463, 0.250392, 0.245423, 0.240554, 0.235783, 0.231108, 0.226526, 0.222038, 0.217640, 0.213330, 0.209108, 0.204971, 0.200918, 0.196946, 0.193055, 0.189242, 0.185506, 0.181846, 0.178258, 0.174743, 0.171298, 0.167922, 0.164613, 0.161370, 0.158190, 0.155073, 0.152018, 0.149021, 0.146083, 0.143202, 0.140376, 0.137603, 0.134883, 0.132214, 0.129595, 0.127027, 0.124509, 0.122040, 0.119619, 0.117245, 0.114918, 0.112637, 0.110400, 0.108208, 0.106060, 0.103954, 0.101890, 0.099868, 0.097885, 0.095943, 0.094039, 0.092174, 0.090347, 0.088556, 0.086801, 0.085082, 0.083397, 0.081747, 0.080129, 0.078545, 0.076992, 0.075471, 0.073981, 0.072520, 0.071089, 0.069687, 0.068313, 0.066966, 0.065646, 0.064353, 0.063085, 0.061842, 0.060623, 0.059429, 0.058257, 0.057109, 0.055982, 0.054877, 0.053793, 0.052730, 0.051686, 0.050663, 0.049659, 0.048674, 0.047709, 0.046762, 0.045835, 0.044925, 0.044033, 0.043159, 0.042302, 0.041462, 0.040639, 0.039832, 0.039041, 0.038266, 0.037507, 0.036763, 0.036034, 0.035319, 0.034619, 0.033933, 0.033261, 0.032602, 0.031957, 0.031325, 0.030706, 0.030099, 0.029504, 0.028921, 0.028350, 0.027791, 0.027243, 0.026706, 0.026179, 0.025663, 0.025158, 0.024662, 0.024176, 0.023700, 0.023233, 0.022775, 0.022326, 0.021885, 0.021453, 0.021030, 0.020614, 0.020206, 0.019806, 0.019414, 0.019029, 0.018652, 0.018283, 0.017921, 0.017566, 0.017218, 0.016877, 0.016543, 0.016215, 0.015894, 0.015579, 0.015271, 0.014968, 0.014672, 0.014381, 0.014097, 0.013818, 0.013544, 0.013276, 0.013013, 0.012756, 0.012503, 0.012256, 0.012013, 0.011775, 0.011542, 0.011314),
    color: green,
    label: [Dynamic ($kappa = 0.5$)]
  ),
)
,
  caption: [Enstrophy trajectories under dynamic depletion from $Omega_0 = 50$ ($nu = 1$, $alpha_0 = 0.8$). Blue ($kappa = 0.1$): enstrophy peaks near $Omega approx 1.4 times 10^5$ then decays to $6.78$ at $t = 5$. Green ($kappa = 0.5$): peak is $Omega approx 216$ (below the predicted bound $Omega_0 exp(alpha_0 slash kappa) approx 248$), decaying to $0.011$ at $t = 5$. Without depletion ($alpha_1 = 1$, not shown), blow-up occurs at $t approx 0.021$; with constant $alpha_1 = 0.1$, at $t approx 0.33$.]
) <fig:enstrophy>

#figure(
  lq.diagram(
  width: 10cm,
  title: [Alignment weight $alpha_1 (t)$],
  xlabel: [$t$],
  ylabel: [$alpha_1$],
  legend: (position: right + top),
  lq.plot(
    (0.000000, 0.010000, 0.020000, 0.030000, 0.040000, 0.050000, 0.060000, 0.070000, 0.080000, 0.090000, 0.100000, 0.110000, 0.120000, 0.130000, 0.140000, 0.150000, 0.160000, 0.170000, 0.180000, 0.190000, 0.200000, 0.210000, 0.220000, 0.230000, 0.240000, 0.250000, 0.260000, 0.270000, 0.280000, 0.290000, 0.300000, 0.310000, 0.320000, 0.330000, 0.340000, 0.350000, 0.360000, 0.370000, 0.380000, 0.390000, 0.400000, 0.410000, 0.420000, 0.430000, 0.440000, 0.450000, 0.460000, 0.470000, 0.480000, 0.490000, 0.500000, 0.510000, 0.520000, 0.530000, 0.540000, 0.550000, 0.560000, 0.570000, 0.580000, 0.590000, 0.600000, 0.610000, 0.620000, 0.630000, 0.640000, 0.650000, 0.660000, 0.670000, 0.680000, 0.690000, 0.700000, 0.710000, 0.720000, 0.730000, 0.740000, 0.750000, 0.760000, 0.770000, 0.780000, 0.790000, 0.800000, 0.810000, 0.820000, 0.830000, 0.840000, 0.850000, 0.860000, 0.870000, 0.880000, 0.890000, 0.900000, 0.910000, 0.920000, 0.930000, 0.940000, 0.950000, 0.960000, 0.970000, 0.980000, 0.990000, 1.000000, 1.010000, 1.020000, 1.030000, 1.040000, 1.050000, 1.060000, 1.070000, 1.080000, 1.090000, 1.100000, 1.110000, 1.120000, 1.130000, 1.140000, 1.150000, 1.160000, 1.170000, 1.180000, 1.190000, 1.200000, 1.210000, 1.220000, 1.230000, 1.240000, 1.250000, 1.260000, 1.270000, 1.280000, 1.290000, 1.300000, 1.310000, 1.320000, 1.330000, 1.340000, 1.350000, 1.360000, 1.370000, 1.380000, 1.390000, 1.400000, 1.410000, 1.420000, 1.430000, 1.440000, 1.450000, 1.460000, 1.470000, 1.480000, 1.490000, 1.500000, 1.510000, 1.520000, 1.530000, 1.540000, 1.550000, 1.560000, 1.570000, 1.580000, 1.590000, 1.600000, 1.610000, 1.620000, 1.630000, 1.640000, 1.650000, 1.660000, 1.670000, 1.680000, 1.690000, 1.700000, 1.710000, 1.720000, 1.730000, 1.740000, 1.750000, 1.760000, 1.770000, 1.780000, 1.790000, 1.800000, 1.810000, 1.820000, 1.830000, 1.840000, 1.850000, 1.860000, 1.870000, 1.880000, 1.890000, 1.900000, 1.910000, 1.920000, 1.930000, 1.940000, 1.950000, 1.960000, 1.970000, 1.980000, 1.990000, 2.000000, 2.010000, 2.020000, 2.030000, 2.040000, 2.050000, 2.060000, 2.070000, 2.080000, 2.090000, 2.100000, 2.110000, 2.120000, 2.130000, 2.140000, 2.150000, 2.160000, 2.170000, 2.180000, 2.190000, 2.200000, 2.210000, 2.220000, 2.230000, 2.240000, 2.250000, 2.260000, 2.270000, 2.280000, 2.290000, 2.300000, 2.310000, 2.320000, 2.330000, 2.340000, 2.350000, 2.360000, 2.370000, 2.380000, 2.390000, 2.400000, 2.410000, 2.420000, 2.430000, 2.440000, 2.450000, 2.460000, 2.470000, 2.480000, 2.490000, 2.500000, 2.510000, 2.520000, 2.530000, 2.540000, 2.550000, 2.560000, 2.570000, 2.580000, 2.590000, 2.600000, 2.610000, 2.620000, 2.630000, 2.640000, 2.650000, 2.660000, 2.670000, 2.680000, 2.690000, 2.700000, 2.710000, 2.720000, 2.730000, 2.740000, 2.750000, 2.760000, 2.770000, 2.780000, 2.790000, 2.800000, 2.810000, 2.820000, 2.830000, 2.840000, 2.850000, 2.860000, 2.870000, 2.880000, 2.890000, 2.900000, 2.910000, 2.920000, 2.930000, 2.940000, 2.950000, 2.960000, 2.970000, 2.980000, 2.990000, 3.000000, 3.010000, 3.020000, 3.030000, 3.040000, 3.050000, 3.060000, 3.070000, 3.080000, 3.090000, 3.100000, 3.110000, 3.120000, 3.130000, 3.140000, 3.150000, 3.160000, 3.170000, 3.180000, 3.190000, 3.200000, 3.210000, 3.220000, 3.230000, 3.240000, 3.250000, 3.260000, 3.270000, 3.280000, 3.290000, 3.300000, 3.310000, 3.320000, 3.330000, 3.340000, 3.350000, 3.360000, 3.370000, 3.380000, 3.390000, 3.400000, 3.410000, 3.420000, 3.430000, 3.440000, 3.450000, 3.460000, 3.470000, 3.480000, 3.490000, 3.500000, 3.510000, 3.520000, 3.530000, 3.540000, 3.550000, 3.560000, 3.570000, 3.580000, 3.590000, 3.600000, 3.610000, 3.620000, 3.630000, 3.640000, 3.650000, 3.660000, 3.670000, 3.680000, 3.690000, 3.700000, 3.710000, 3.720000, 3.730000, 3.740000, 3.750000, 3.760000, 3.770000, 3.780000, 3.790000, 3.800000, 3.810000, 3.820000, 3.830000, 3.840000, 3.850000, 3.860000, 3.870000, 3.880000, 3.890000, 3.900000, 3.910000, 3.920000, 3.930000, 3.940000, 3.950000, 3.960000, 3.970000, 3.980000, 3.990000, 4.000000, 4.010000, 4.020000, 4.030000, 4.040000, 4.050000, 4.060000, 4.070000, 4.080000, 4.090000, 4.100000, 4.110000, 4.120000, 4.130000, 4.140000, 4.150000, 4.160000, 4.170000, 4.180000, 4.190000, 4.200000, 4.210000, 4.220000, 4.230000, 4.240000, 4.250000, 4.260000, 4.270000, 4.280000, 4.290000, 4.300000, 4.310000, 4.320000, 4.330000, 4.340000, 4.350000, 4.360000, 4.370000, 4.380000, 4.390000, 4.400000, 4.410000, 4.420000, 4.430000, 4.440000, 4.450000, 4.460000, 4.470000, 4.480000, 4.490000, 4.500000, 4.510000, 4.520000, 4.530000, 4.540000, 4.550000, 4.560000, 4.570000, 4.580000, 4.590000, 4.600000, 4.610000, 4.620000, 4.630000, 4.640000, 4.650000, 4.660000, 4.670000, 4.680000, 4.690000, 4.700000, 4.710000, 4.720000, 4.730000, 4.740000, 4.750000, 4.760000, 4.770000, 4.780000, 4.790000, 4.800000, 4.810000, 4.820000, 4.830000, 4.840000, 4.850000, 4.860000, 4.870000, 4.880000, 4.890000, 4.900000, 4.910000, 4.920000, 4.930000, 4.940000, 4.950000, 4.960000, 4.970000, 4.980000, 4.990000, 5.000000),
    (0.800000, 0.751409, 0.668419, 0.234654, -0.000000, -0.000001, 0.000001, -0.000000, -0.000000, -0.000001, -0.000000, 0.000000, 0.000001, 0.000001, -0.000001, 0.000000, 0.000001, -0.000000, 0.000001, -0.000001, 0.000001, 0.000000, 0.000001, 0.000000, 0.000001, -0.000001, -0.000001, 0.000000, 0.000000, -0.000000, -0.000000, 0.000001, 0.000001, -0.000001, -0.000001, 0.000001, -0.000001, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, 0.000000, -0.000000, -0.000001, 0.000001, 0.000000, 0.000001, -0.000000, 0.000000, 0.000002, -0.000000, -0.000001, -0.000001, 0.000000, 0.000001, 0.000000, -0.000000, -0.000001, -0.000001, -0.000000, -0.000000, 0.000001, 0.000000, -0.000000, 0.000000, 0.000000, 0.000001, -0.000000, 0.000000, -0.000000, 0.000000, 0.000001, -0.000001, 0.000001, 0.000001, 0.000000, 0.000000, -0.000000, 0.000000, 0.000001, 0.000001, 0.000000, -0.000000, 0.000001, 0.000001, -0.000000, -0.000000, 0.000001, 0.000000, -0.000000, 0.000000, 0.000001, 0.000001, 0.000001, 0.000000, -0.000000, 0.000000, -0.000001, 0.000000, 0.000001, 0.000001, 0.000000, -0.000000, -0.000000, 0.000001, -0.000000, -0.000001, 0.000001, 0.000001, 0.000001, 0.000001, 0.000001, -0.000000, 0.000000, 0.000000, 0.000000, -0.000000, 0.000001, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000001, 0.000000, -0.000000, 0.000002, -0.000000, 0.000000, -0.000000, -0.000001, 0.000001, 0.000001, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000001, 0.000000, -0.000001, 0.000000, -0.000001, 0.000000, -0.000000, -0.000000, -0.000000, 0.000001, 0.000001, -0.000000, -0.000000, 0.000001, 0.000001, 0.000002, 0.000000, 0.000000, 0.000001, 0.000000, 0.000001, 0.000000, -0.000000, -0.000000, 0.000000, 0.000001, -0.000000, -0.000000, 0.000000, -0.000000, 0.000001, -0.000001, 0.000001, -0.000000, 0.000000, -0.000000, -0.000001, 0.000001, -0.000000, -0.000001, 0.000000, 0.000001, -0.000000, -0.000001, -0.000001, -0.000000, 0.000001, 0.000000, 0.000000, 0.000001, 0.000001, 0.000000, -0.000000, -0.000000, 0.000001, 0.000002, 0.000001, 0.000001, 0.000000, -0.000000, -0.000001, -0.000000, 0.000002, 0.000001, 0.000000, -0.000000, -0.000000, 0.000001, -0.000001, -0.000000, 0.000001, 0.000000, -0.000000, -0.000000, 0.000000, -0.000001, 0.000001, -0.000001, 0.000001, 0.000000, -0.000000, 0.000000, -0.000000, 0.000000, -0.000001, 0.000001, -0.000001, 0.000001, -0.000000, 0.000001, -0.000000, 0.000001, -0.000000, 0.000000, -0.000000, -0.000000, 0.000000, -0.000001, 0.000000, 0.000001, -0.000001, 0.000001, 0.000000, 0.000000, 0.000001, -0.000000, 0.000000, 0.000000, -0.000000, 0.000000, 0.000000, -0.000001, -0.000000, 0.000001, 0.000000, -0.000001, -0.000000, 0.000001, 0.000000, -0.000000, 0.000000, 0.000001, 0.000000, -0.000000, 0.000000, 0.000000, -0.000000, -0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000001, -0.000000, 0.000000, 0.000001, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000001, 0.000001, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000),
    color: blue,
    label: [Strong ($kappa = 0.1$)]
  ),
  lq.plot(
    (0.000000, 0.010000, 0.020000, 0.030000, 0.040000, 0.050000, 0.060000, 0.070000, 0.080000, 0.090000, 0.100000, 0.110000, 0.120000, 0.130000, 0.140000, 0.150000, 0.160000, 0.170000, 0.180000, 0.190000, 0.200000, 0.210000, 0.220000, 0.230000, 0.240000, 0.250000, 0.260000, 0.270000, 0.280000, 0.290000, 0.300000, 0.310000, 0.320000, 0.330000, 0.340000, 0.350000, 0.360000, 0.370000, 0.380000, 0.390000, 0.400000, 0.410000, 0.420000, 0.430000, 0.440000, 0.450000, 0.460000, 0.470000, 0.480000, 0.490000, 0.500000, 0.510000, 0.520000, 0.530000, 0.540000, 0.550000, 0.560000, 0.570000, 0.580000, 0.590000, 0.600000, 0.610000, 0.620000, 0.630000, 0.640000, 0.650000, 0.660000, 0.670000, 0.680000, 0.690000, 0.700000, 0.710000, 0.720000, 0.730000, 0.740000, 0.750000, 0.760000, 0.770000, 0.780000, 0.790000, 0.800000, 0.810000, 0.820000, 0.830000, 0.840000, 0.850000, 0.860000, 0.870000, 0.880000, 0.890000, 0.900000, 0.910000, 0.920000, 0.930000, 0.940000, 0.950000, 0.960000, 0.970000, 0.980000, 0.990000, 1.000000, 1.010000, 1.020000, 1.030000, 1.040000, 1.050000, 1.060000, 1.070000, 1.080000, 1.090000, 1.100000, 1.110000, 1.120000, 1.130000, 1.140000, 1.150000, 1.160000, 1.170000, 1.180000, 1.190000, 1.200000, 1.210000, 1.220000, 1.230000, 1.240000, 1.250000, 1.260000, 1.270000, 1.280000, 1.290000, 1.300000, 1.310000, 1.320000, 1.330000, 1.340000, 1.350000, 1.360000, 1.370000, 1.380000, 1.390000, 1.400000, 1.410000, 1.420000, 1.430000, 1.440000, 1.450000, 1.460000, 1.470000, 1.480000, 1.490000, 1.500000, 1.510000, 1.520000, 1.530000, 1.540000, 1.550000, 1.560000, 1.570000, 1.580000, 1.590000, 1.600000, 1.610000, 1.620000, 1.630000, 1.640000, 1.650000, 1.660000, 1.670000, 1.680000, 1.690000, 1.700000, 1.710000, 1.720000, 1.730000, 1.740000, 1.750000, 1.760000, 1.770000, 1.780000, 1.790000, 1.800000, 1.810000, 1.820000, 1.830000, 1.840000, 1.850000, 1.860000, 1.870000, 1.880000, 1.890000, 1.900000, 1.910000, 1.920000, 1.930000, 1.940000, 1.950000, 1.960000, 1.970000, 1.980000, 1.990000, 2.000000, 2.010000, 2.020000, 2.030000, 2.040000, 2.050000, 2.060000, 2.070000, 2.080000, 2.090000, 2.100000, 2.110000, 2.120000, 2.130000, 2.140000, 2.150000, 2.160000, 2.170000, 2.180000, 2.190000, 2.200000, 2.210000, 2.220000, 2.230000, 2.240000, 2.250000, 2.260000, 2.270000, 2.280000, 2.290000, 2.300000, 2.310000, 2.320000, 2.330000, 2.340000, 2.350000, 2.360000, 2.370000, 2.380000, 2.390000, 2.400000, 2.410000, 2.420000, 2.430000, 2.440000, 2.450000, 2.460000, 2.470000, 2.480000, 2.490000, 2.500000, 2.510000, 2.520000, 2.530000, 2.540000, 2.550000, 2.560000, 2.570000, 2.580000, 2.590000, 2.600000, 2.610000, 2.620000, 2.630000, 2.640000, 2.650000, 2.660000, 2.670000, 2.680000, 2.690000, 2.700000, 2.710000, 2.720000, 2.730000, 2.740000, 2.750000, 2.760000, 2.770000, 2.780000, 2.790000, 2.800000, 2.810000, 2.820000, 2.830000, 2.840000, 2.850000, 2.860000, 2.870000, 2.880000, 2.890000, 2.900000, 2.910000, 2.920000, 2.930000, 2.940000, 2.950000, 2.960000, 2.970000, 2.980000, 2.990000, 3.000000, 3.010000, 3.020000, 3.030000, 3.040000, 3.050000, 3.060000, 3.070000, 3.080000, 3.090000, 3.100000, 3.110000, 3.120000, 3.130000, 3.140000, 3.150000, 3.160000, 3.170000, 3.180000, 3.190000, 3.200000, 3.210000, 3.220000, 3.230000, 3.240000, 3.250000, 3.260000, 3.270000, 3.280000, 3.290000, 3.300000, 3.310000, 3.320000, 3.330000, 3.340000, 3.350000, 3.360000, 3.370000, 3.380000, 3.390000, 3.400000, 3.410000, 3.420000, 3.430000, 3.440000, 3.450000, 3.460000, 3.470000, 3.480000, 3.490000, 3.500000, 3.510000, 3.520000, 3.530000, 3.540000, 3.550000, 3.560000, 3.570000, 3.580000, 3.590000, 3.600000, 3.610000, 3.620000, 3.630000, 3.640000, 3.650000, 3.660000, 3.670000, 3.680000, 3.690000, 3.700000, 3.710000, 3.720000, 3.730000, 3.740000, 3.750000, 3.760000, 3.770000, 3.780000, 3.790000, 3.800000, 3.810000, 3.820000, 3.830000, 3.840000, 3.850000, 3.860000, 3.870000, 3.880000, 3.890000, 3.900000, 3.910000, 3.920000, 3.930000, 3.940000, 3.950000, 3.960000, 3.970000, 3.980000, 3.990000, 4.000000, 4.010000, 4.020000, 4.030000, 4.040000, 4.050000, 4.060000, 4.070000, 4.080000, 4.090000, 4.100000, 4.110000, 4.120000, 4.130000, 4.140000, 4.150000, 4.160000, 4.170000, 4.180000, 4.190000, 4.200000, 4.210000, 4.220000, 4.230000, 4.240000, 4.250000, 4.260000, 4.270000, 4.280000, 4.290000, 4.300000, 4.310000, 4.320000, 4.330000, 4.340000, 4.350000, 4.360000, 4.370000, 4.380000, 4.390000, 4.400000, 4.410000, 4.420000, 4.430000, 4.440000, 4.450000, 4.460000, 4.470000, 4.480000, 4.490000, 4.500000, 4.510000, 4.520000, 4.530000, 4.540000, 4.550000, 4.560000, 4.570000, 4.580000, 4.590000, 4.600000, 4.610000, 4.620000, 4.630000, 4.640000, 4.650000, 4.660000, 4.670000, 4.680000, 4.690000, 4.700000, 4.710000, 4.720000, 4.730000, 4.740000, 4.750000, 4.760000, 4.770000, 4.780000, 4.790000, 4.800000, 4.810000, 4.820000, 4.830000, 4.840000, 4.850000, 4.860000, 4.870000, 4.880000, 4.890000, 4.900000, 4.910000, 4.920000, 4.930000, 4.940000, 4.950000, 4.960000, 4.970000, 4.980000, 4.990000, 5.000000),
    (0.800000, 0.588634, 0.369702, 0.185826, 0.075348, 0.026876, 0.009180, 0.003131, 0.001083, 0.000381, 0.000137, 0.000050, 0.000019, 0.000007, 0.000003, 0.000001, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, 0.000000, 0.000000, -0.000000, -0.000001, -0.000000, 0.000000, 0.000001, 0.000000, -0.000000, -0.000001, 0.000000, 0.000001, 0.000001, -0.000000, -0.000000, 0.000000, 0.000001, 0.000000, 0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, -0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000),
    color: green,
    label: [Very strong ($kappa = 0.5$)]
  ),
)
,
  caption: [Alignment weight $alpha_1(t)$ under dynamic depletion from $alpha_0 = 0.8$. Both depletion rates drive $alpha_1$ to zero within $t approx 1$. The collapse of alignment is the mechanism that reverses enstrophy growth: once $alpha_1 dot Omega < 2 nu$, dissipation dominates.]
) <fig:alignment>

= Connection to Paper 1 and Physical Interpretation

The results of Sections 3-6 form a bridge between the scalar diagnosis of [1] and the geometric structure of the Navier-Stokes equations.

Paper 1 established that the scalar Sobolev framework cannot close the half-derivative gap: no inequality among $E$, $Omega$, $H_(3 slash 2)$, $P$ excludes blow-up. The critical threshold is $a + b = 2$ in the stretching bound $S^2 <= Omega^a P^b$. Kato-Ponce already gives $a + b = 3$; the remaining unit must come from geometry.

This paper identifies what that geometry is. The stretching integral $S = Omega dot sigma_("eff")$ has an effective rate $sigma_("eff") = sum lambda_i alpha_i$ that depends on two geometric quantities: *how vorticity is oriented* (the $alpha_i$) and *what the strain field looks like* (the $lambda_i$). The Z3 tests show that controlling either quantity alone is insufficient (Tests D8, D9), but controlling both produces $a + b = 2$ (Test D7).

The two required conditions have a clear physical interpretation:

*Scale-dependent alignment depletion* ($alpha_1 <= C slash Omega$) means that as gradients intensify, vorticity increasingly avoids the most stretching eigenvector. This is consistent with the empirically observed tendency of vorticity to align with the _intermediate_ eigenvector $e_2$ of the strain rate tensor [3, 4]. The mechanism is plausible: the most stretching direction ($e_1$) rotates vortex tubes into thin sheets, which are then re-oriented by pressure gradients and viscous diffusion. At high enstrophy, this re-orientation may outpace the alignment, producing a net depletion.

*Biaxial strain* ($lambda_2 <= 0$) means the intermediate eigenvalue is non-positive. This occurs when the strain is dominated by one stretching direction and two compressive directions — a sheet-like deformation. In fully biaxial strain, vorticity aligned with $e_2$ or $e_3$ experiences no stretching (since $lambda_2 , lambda_3 <= 0$), and only the $lambda_1 alpha_1$ channel remains. This makes the alignment depletion effective: once $alpha_1$ is small, _all_ stretching is small.

The Depletion Boundedness Theorem (Section 6) shows that these conditions are not merely sufficient for closing the gap — they produce a _stronger_ conclusion than regularity: any positive depletion rate $kappa > 0$ forces $Omega(t) -> 0$ as $t -> infinity$, with a trajectory-independent bound on the total stretching. The finite "stretching budget" $integral Omega alpha_1 thin d t <= alpha_0 slash kappa$ is the key structural invariant: depletion spends the alignment budget at a rate proportional to $Omega$, ensuring the total is always finite regardless of how large $Omega$ becomes transiently.

= Conclusion

We have presented a machine-verified analysis of geometric depletion in the 3D Navier-Stokes vortex stretching term. The investigation spans thirteen Z3 satisfiability tests, four alignment weight configurations, an exponent threshold scan, three geometric condition combinations, and an analytical boundedness theorem with numerical verification. The principal findings are:

(1) *Alignment control alone is insufficient*: Bounding $alpha_1$ — the fraction of enstrophy aligned with the most stretching eigenvector — does not change the exponent scaling. The intermediate eigenvalue $lambda_2$ provides an uncontrolled stretching channel. Even scale-dependent depletion ($alpha_1 <= 1 slash Omega$) returns SAT when $lambda_2$ is unconstrained.

(2) *The threshold is sharp*: The exponent sum $a + b = 2$ is a strict phase boundary. Every $a + b > 2$ — including $2.1$, $2.25$, $2.5$ — admits blow-up scenarios. There is no gradual transition.

(3) *Gap closure requires two conditions*: Scale-dependent alignment depletion ($alpha_1 Omega <= C$) and biaxial strain ($lambda_2 <= 0$) are individually insufficient but jointly sufficient to close the half-derivative gap. Together they imply $S^2 <= Omega P$ ($a + b = 2$), triggering the global basin collapse identified in [1].

(4) *Depletion Boundedness Theorem*: For the coupled enstrophy-alignment ODE with any depletion rate $kappa > 0$, the total stretching integral is bounded by $alpha_0 slash kappa$, giving $Omega(t) <= Omega_0 exp(alpha_0 slash kappa - 2 nu t) -> 0$. The proof is a three-step substitution argument; the bound is trajectory-independent.

Together with [1], these results reduce the Navier-Stokes Millennium Problem to a single geometric question: *does the 3D incompressible Navier-Stokes dynamics produce scale-dependent alignment depletion and biaxial strain in the high-enstrophy regime?* The first condition asks whether vorticity avoids sustained alignment with the most stretching direction as gradients intensify. The second asks whether the strain field is predominantly sheet-like (one stretching, two compressive directions) in regions of high vorticity.

Both conditions are consistent with empirical observations from direct numerical simulations [3, 4], which report preferential alignment of vorticity with the intermediate eigenvector $e_2$ in turbulent flows. Whether this observed tendency is a consequence of the Navier-Stokes dynamics — rather than an artifact of the particular flows simulated — remains the critical open question.

The full test suite is available in the Kleis repository at `theories/ns_alignment_weights.kleis` and `theories/ns_depletion_d*.kleis`.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[atik2026halfderiv\] Atik, E. (2026). The Half-Derivative Gap: Machine-Verified Structural Diagnosis of Navier-Stokes Smoothness. Preprint.]

#par(hanging-indent: 1.5em)[\[kato1988\] Kato, T. & Ponce, G. (1988). Commutator estimates and the Euler and Navier-Stokes equations. Communications on Pure and Applied Mathematics, 41(7), 891-907.]

#par(hanging-indent: 1.5em)[\[ashurst1987\] Ashurst, W. T. et al. (1987). Alignment of vorticity and scalar gradient with strain rate in simulated Navier-Stokes turbulence. Physics of Fluids, 30(8), 2343-2353.]

#par(hanging-indent: 1.5em)[\[constantin1994\] Constantin, P. (1994). Geometric statistics in turbulence. SIAM Review, 36(1), 73-98.]

#par(hanging-indent: 1.5em)[\[atik2026kleis\] Atik, E. (2026). Kleis: A Symbolic Mathematics Language with Integrated Formal Verification. https://kleis.io.]

#par(hanging-indent: 1.5em)[\[fefferman2000\] Fefferman, C. L. (2000). Existence and smoothness of the Navier-Stokes equation. Clay Mathematics Institute Millennium Prize Problems.]


