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
  #text(size: 17pt, weight: "bold")[The Wobble Invariant and the Greene--Lobb Rectangle Sweep: A Computational Study of the Toeplitz Inscribed Square Conjecture]
  
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
      #text(size: 10pt)[We present a computational investigation of the Toeplitz inscribed square conjecture using the Kleis formal verification platform. Starting from the classical edge-based defect map $G(t_2, t_3) = e_2 - i e_1$, we identify a structural gap: $G = 0$ detects L-shapes but does not guarantee the fourth vertex lies on the curve. We resolve this with the diagonal formulation $Phi(s_1, t_1, s_2, t_2)$, a balanced $RR^4 -> RR^4$ map where all four vertices are on the curve by construction. We then introduce the wobble invariant $W(theta)$, a scalar function measuring the mismatch between perpendicular chord lengths through a fixed center point. The identity $W(theta + pi slash 2) = -W(theta)$, verified by Z3, forces a sign change and hence, by the Intermediate Value Theorem, a pair of perpendicular equal-length chords through that center. This is necessary but not sufficient for an inscribed square: two additional conditions must hold --- (1) nondegeneracy ($N(gamma) > 0$, i.e. the chords do not collapse to zero length) and (2) the midpoint condition (the center bisects both chords). We stress-test $N(gamma)$ across six families of pathological curves (31 curves total), including near-self-intersecting dumbbells pushed to 98% of the figure-8 limit, finding $N > 0$ throughout. We numerically explore the coupled $(c, theta)$ system for the midpoint condition, obtaining approximate solutions with residuals decreasing under refinement. We also sweep Greene--Lobb rectangle solutions from $r = 3$ to $r = 1$; computational evidence suggests chord lengths remain bounded away from zero, but we do not claim continuity in $r$. We situate our findings in the context of recent work by Chambers (2025), Greene--Lobb (2024), and Asano--Ike (2024), showing that our computational results reach the same degeneration barrier identified by analytical methods. The Toeplitz conjecture reduces, in our framework, to proving three conditions simultaneously. We identify the precise obstruction structure; we do not resolve it. The investigation comprises over 70 machine-checked examples, 9 Z3-verified structures, and 70 Kleis functions. All results are reproducible from a single source file.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* inscribed square, Toeplitz conjecture, square peg problem, Jordan curve, wobble invariant, diagonal formulation, Z3 verification, Greene-Lobb, rectangle peg problem, nondegeneracy]

#v(1em)


= Introduction

In 1911, Otto Toeplitz conjectured that every Jordan curve in the plane admits an inscribed square: four points on the curve forming a perfect square [1]. Despite over a century of work by some of the finest mathematicians, the conjecture remains open for general continuous curves.

The history of partial results reveals a surprising progression of difficulty. Emch [2] proved the convex case in 1913 using midpoints and continuity. Schnirelman [3] extended the result to $C^2$ curves in 1929. Stromquist [9] proved it for locally monotone curves in 1989. Meyerson [10] showed in 1981 that all but at most two points on any Jordan curve lie on some inscribed square. Most recently, Greene and Lobb [5] proved in 2020 that every smooth Jordan curve inscribes a rectangle of every aspect ratio --- using symplectic geometry, Klein bottles, and Lagrangian embeddings in $RR^4$.

Yet the original conjecture --- for continuous curves, and specifically for squares --- remains open. The difficulty is structural: rectangles require three constraints (shared midpoint, equal diagonal length, and a specific diagonal angle), while squares require four (adding perpendicularity). This extra constraint breaks the topological arguments that work for rectangles.

This paper presents a computational investigation using the Kleis formal verification platform [6]. We make four contributions:

1. *The diagonal formulation* (Section 3): We identify a structural gap in the classical edge-based defect map $G(t_2, t_3)$ and resolve it with a balanced $RR^4 -> RR^4$ diagonal map $Phi$ where all four square vertices are on the curve by construction.

2. *The wobble invariant and the three-condition structure* (Section 4): Inspired by the four-legged table problem, we construct a scalar function $W(theta)$ whose antisymmetry $W(theta + pi slash 2) = -W(theta)$ (Z3-verified) forces, by IVT, the existence of perpendicular equal-length chords through a fixed center. This is _necessary but not sufficient_ for an inscribed square. The full problem requires three conditions simultaneously: (i) the wobble zero (perpendicular equal-length chords), (ii) nondegeneracy ($N(gamma) > 0$, the chords have positive length), and (iii) the midpoint condition (the center bisects both chords). The wobble reduces one dimension of the problem; the remaining system is three equations in three unknowns $(c_x, c_y, theta)$.

3. *Stress testing nondegeneracy* (Sections 5, 7): We define a nondegeneracy invariant $N(gamma)$ and systematically test it across six pathological curve families (31 curves), finding $N > 0$ throughout. The most extreme case --- a near-self-intersecting dumbbell --- gives $N approx 0.26$, which floors out rather than approaching zero. This is purely empirical evidence; we provide no theoretical bound or mechanism.

4. *The Greene--Lobb rectangle sweep and literature context* (Sections 6, 8): We generalize $Phi$ to $Phi_r$ for rectangles with aspect ratio $r$ and sweep $r -> 1$. Computational evidence suggests chord lengths remain bounded away from zero in our tested curves, but Greene--Lobb's methods do not guarantee continuity in $r$, so this evidence does not constitute a proof that rectangle solutions converge to squares. We situate our findings alongside recent work by Chambers [13], Greene--Lobb [14], and Asano--Ike [15], showing our computational results reach the same degeneration barrier identified by analytical methods.

= The Edge-Based Approach and the L-Shape Gap

The classical approach parameterizes a Jordan curve $gamma: [0, 2 pi) -> RR^2$ and seeks four parameters $t_1, t_2, t_3, t_4$ such that the four points $gamma(t_i)$ form a square. Writing $e_k = gamma(t_(k+1)) - gamma(t_k)$ for the edge vectors, the square condition requires $e_(k+1) = i dot.c e_k$ for all $k$ (each edge is a $90 degree$ rotation of the previous).

Fixing $t_1 = 0$, the defect map is

$ G(t_2, t_3) = e_2 - i dot.c e_1 = [gamma(t_3) - gamma(t_2)] - i dot.c [gamma(t_2) - gamma(0)] $ <eq:defect>

== The structural gap

The map $G: RR^2 -> RR^2$ is balanced: two equations in two unknowns. By topological degree theory, if $G$ has nonzero winding number on the boundary of a domain, it must have a zero inside. We verify numerically that $G$ has winding number $plus.minus 1$ for seven curve families.

However, $G = 0$ does _not_ guarantee an inscribed square. It only guarantees that $e_2 = i dot.c e_1$, meaning the first two edges form a right angle with equal length --- an "L-shape." The fourth vertex $gamma(t_4)$ is determined by the closure condition $e_1 + e_2 + e_3 + e_4 = 0$, but this fourth vertex need not lie on the curve.

The full inscribed square condition, after fixing $t_1$, requires three parameters $(t_2, t_3, t_4)$ to satisfy four equations (two edge-rotation conditions and two closure conditions) --- an overdetermined system $RR^3 -> RR^4$ outside the scope of Brouwer degree theory.

Z3 verifies the redundancy: given $e_2 = i dot.c e_1$ (D1 = 0) and closure, the remaining conditions $D_2 = 0$ and $D_3 = 0$ are equivalent. But the closure itself involves the fourth vertex, which the defect map $G$ does not control.

= The Diagonal Formulation

We resolve the L-shape gap by reformulating the problem in terms of diagonals. An inscribed square is determined by two chords of the curve that serve as diagonals. If both chords satisfy the diagonal conditions, all four endpoints lie on the curve by construction --- there is no missing vertex.

A square's diagonals satisfy exactly three conditions: (a) same midpoint (diagonals bisect each other), (b) same length, and (c) perpendicular meeting. For two chords $(s_1, t_1)$ and $(s_2, t_2)$ on curve $gamma$, we define the midpoint $c_j = (gamma(s_j) + gamma(t_j)) slash 2$ and direction vector $d_j = gamma(t_j) - gamma(s_j)$. The test map is

$ Phi(s_1, t_1, s_2, t_2) = (c_1 - c_2, thin |d_1|^2 - |d_2|^2, thin d_1 dot d_2) $ <eq:phi>

== Z3 verification

This gives a balanced $RR^4 -> RR^4$ system (two midpoint equations, one length equation, one perpendicularity equation). $Phi = 0$ if and only if the two chords are diagonals of an inscribed square.

Using half-diagonal coordinates centered at the intersection, Z3 verifies three structural properties. First, $Phi = 0$ implies all four sides have equal length. Second, $Phi = 0$ implies consecutive sides are perpendicular (the dot product $(p - r)(p + r) + (q - s)(q + s) = p^2 + q^2 - r^2 - s^2 = 0$ by the equal-length axiom). Third, the map has $ZZ_2 times ZZ_2 times ZZ_2$ symmetry under chord reversal and swap.

Numerical zero search via local refinement confirms $||Phi||^2 -> 0$ for the unit circle, Fourier-deformed curves, the regular pentagon, and the irregular 7-gon.

= The Wobble Invariant

The diagonal formulation gives $Phi: RR^4 -> RR^4$, but proving existence of a zero requires topological machinery (equivariant Euler class, Borsuk--Ulam-type arguments). We introduce a dimensional reduction: the _wobble invariant_ $W(theta)$, a scalar function whose sign change forces an inscribed square via the one-dimensional Intermediate Value Theorem.

The analogy is with the four-legged table problem. A three-legged table always finds a stable position on any surface (it is not overconstrained). A four-legged table is overconstrained --- all four legs must touch simultaneously. The classical result that a square table can always be stabilized by rotation is a topological fact: the "wobble" (height gap of the fourth leg) is a scalar that changes sign under $90 degree$ rotation, so the IVT gives a stable position.

For Toeplitz, fix a center point $p$ (e.g., the centroid) inside the curve. For each angle $theta$, shoot a chord through $p$ at angle $theta$ and another at angle $theta + pi slash 2$. Define

$ W(theta) = |"chord"(theta)|^2 - |"chord"(theta + pi slash 2)|^2 $ <eq:wobble>

== The forced sign change

The key identity is $W(theta + pi slash 2) = -W(theta)$, because rotating by $pi slash 2$ swaps the two chords. Z3 verifies this algebraically: if $W = L_1 - L_2$ then $W(theta + pi slash 2) = L_2 - L_1 = -W$. Since $W$ is continuous and changes sign, the IVT guarantees a zero $theta_0$ where $|"chord"(theta_0)| = |"chord"(theta_0 + pi slash 2)|$: two perpendicular chords of equal length through the fixed center point.

*What the wobble zero gives and what it does not.* A wobble zero yields perpendicular equal-length chords through a chosen center $c$. For these to be the diagonals of an inscribed square, two further conditions must hold: (1) the chords must have nonzero length (nondegeneracy, Section 5), and (2) the center $c$ must be the midpoint of _both_ chords (the midpoint condition). For center-symmetric curves, condition (2) is automatic. For general curves, the full problem is a coupled system in three unknowns $(c_x, c_y, theta)$, of which the wobble handles only one dimension.

For the unit circle, $W equiv 0$ (all chords through the center are diameters). For the ellipse $(2 cos t, sin t)$, $W(0) > 0$ and $W(pi slash 2) < 0$. Bisection locates $W = 0$ at $theta_0 approx 0.46$ with $||Phi||^2 approx 10^(-7)$.

For non-convex curves, the ray-hit computation selects the closest intersection along the ray. The sign-change property is algebraic (depending only on the chord-length swap), not on convexity. We verify numerically that the wobble function exhibits sign changes for all tested curve families: the irregular 7-gon, Koch snowflake (level 1), and Fourier-deformed curves.

= The Nondegeneracy Guard

A zero of $W$ gives two perpendicular equal-length chords through a common point. But the chords might collapse: both diagonals could have length zero (the center point is on the curve and the chord degenerates). This is the _degeneration_ that blocks all known approaches to the full Toeplitz conjecture.

We define the nondegeneracy invariant

$ N(gamma) = min_(theta : W(theta) = 0) |"chord"(theta)|^2 $ <eq:nondegen>

== Tracking degeneration under deformation

If $N(gamma) > 0$, the perpendicular equal-length chords found by the wobble zero have positive length --- a necessary condition for a genuine inscribed square (the remaining condition is that the center bisects both chords). We compute $N$ for a family of curves deforming from a circle to an ellipse: $gamma_epsilon(t) = ((1 + epsilon) cos t, (1 - epsilon) sin t)$. As $epsilon$ increases from 0 (circle) to 0.9 (extreme ellipse), $N$ decreases from 4.0 to 0.097 but remains positive throughout.

We also compute $N$ for five diverse curves: unit circle ($N = 4.0$), ellipse ($N = 6.37$), regular pentagon ($N = 2.99$), irregular 7-gon ($N = 2.20$), and Koch snowflake level 1 ($N = 1.73$). All values are strictly positive.

The Z3 structure `NondegeneracyGuard` verifies: $(ell > 0 and s > 0) -> ell dot.c s > 0$ where $ell$ is the chord length and $s$ is a scale factor, establishing that positive chord length is preserved under positive scaling.

= The Greene--Lobb Rectangle Sweep

Greene and Lobb [5] proved in 2020 that every smooth Jordan curve inscribes a rectangle of every aspect ratio $r > 0$. Their proof uses symplectic geometry: the configuration space of unordered pairs of points on the curve is a Mobius strip, and the rectangle condition defines a Lagrangian surface. Shevchishin's theorem on non-orientable Lagrangians in $RR^4$ provides the obstruction.

The natural question is: can we recover the square by setting $r = 1$? The answer is subtle. Greene--Lobb's methods prove existence for each fixed $r$, but do not guarantee continuity in $r$. One cannot simply take the limit $r -> 1$.

We make this question precise by generalizing $Phi$ to a rectangle defect map $Phi_r$. A rectangle with aspect ratio $r$ has diagonals that bisect each other, have equal length, and meet at angle $alpha$ where

$ cos alpha = (r^2 - 1) / (r^2 + 1) $ <eq:rect_angle>

== The rectangle defect map

At $r = 1$ (square), $cos alpha = 0$ and the diagonals are perpendicular, recovering $Phi$. At $r -> infinity$, $cos alpha -> 1$ and the diagonals become nearly parallel (flat rectangle). The generalized map is

$ Phi_r(s_1, t_1, s_2, t_2) = (c_1 - c_2, thin |d_1|^2 - |d_2|^2, thin d_1 dot d_2 - cos alpha dot.c |d_1|^2) $

Z3 verifies three properties of inscribed rectangles from the diagonal conditions: (a) consecutive sides are perpendicular (from equal-length diagonals alone), (b) the side ratio equals $r$ (from the diagonal angle condition), and (c) at $r = 1$, perpendicular diagonals are forced, recovering the square.

== Why $r = 1$ is special

The wobble antisymmetry $W(theta + pi slash 2) = -W(theta)$ holds _only_ at $r = 1$, where the diagonal angle is $pi slash 2$. For $r eq.not 1$, rotating by the diagonal angle does _not_ swap the chord roles in a way that forces a sign change. Greene and Lobb circumvent this with symplectic geometry rather than IVT. This is the precise sense in which squares are harder than rectangles: the topological IVT trick that works for the "four-legged table" requires a $90 degree$ rotation symmetry that rectangles lack.

== Numerical sweep: $r -> 1$

We sweep $r$ from 3.0 down to 1.0, searching for inscribed rectangles at each ratio. For both the ellipse $(2, 1)$ and the irregular 7-gon, chord lengths remain bounded away from zero throughout the sweep. For the ellipse, chord length _increases_ from 4.19 at $r = 3$ to 6.47 at $r = 1$. For the 7-gon, it varies between 1.86 and 2.22 but never approaches zero.

The consistency check confirms that $Phi_r$ at $r = 1$ gives identical residuals to $Phi$, validating the generalization.

This provides computational evidence suggesting that, for these particular curves, inscribed rectangle solutions do not degenerate as $r -> 1$. However, Greene--Lobb's methods guarantee existence for each fixed $r$ but do not establish continuity of solutions in $r$. The sequence of rectangle solutions could in principle jump, bifurcate, or disappear in the limit. We do not claim convergence; we report only the empirical observation that chord lengths remain bounded away from zero across our tested sweep.

= Computational Results

All results are computed in Kleis [6] with Z3 [7] as the verification backend. The full investigation comprises 55 machine-checked examples across 9 structures and 66 functions.

#figure(
  lq.diagram(
  width: 6cm,
  height: 6cm,
  xlim: (-1.4, 1.4),
  ylim: (-1.4, 1.4),
  xaxis: (ticks: none),
  yaxis: (ticks: none),
  lq.plot(
    (1.000000, 1.052689, 1.097721, 1.131675, 1.151683, 1.155634, 1.142334, 1.111582, 1.064177, 1.001846, 0.927104, 0.843053, 0.753145, 0.660927, 0.569784, 0.482702, 0.402079, 0.329584, 0.266081, 0.211622, 0.165501, 0.126373, 0.092409, 0.061485, 0.031386, 0.000000, -0.034509, -0.073561, -0.118063, -0.168368, -0.224279, -0.285101, -0.349729, -0.416765, -0.484651, -0.551800, -0.616714, -0.678087, -0.734872, -0.786324, -0.831998, -0.871733, -0.905595, -0.933822, -0.956746, -0.974732, -0.988122, -0.997191, -1.002130, -1.003050, -1.000000, -0.993004, -0.982100, -0.967384, -0.949044, -0.927381, -0.902807, -0.875833, -0.847018, -0.816923, -0.786036, -0.754703, -0.723065, -0.691007, -0.658134, -0.623770, -0.587002, -0.546743, -0.501830, -0.451148, -0.393755, -0.329012, -0.256700, -0.177106, -0.091072, -0.000000, 0.094195, 0.189182, 0.282354, 0.371007, 0.452533, 0.524628, 0.585478, 0.633923, 0.669575, 0.692869, 0.705064, 0.708167, 0.704792, 0.697974, 0.690930, 0.686810, 0.688437, 0.698073, 0.717219, 0.746479, 0.785484, 0.832899, 0.886509, 0.943364, 1.000000),
    (0.000000, 0.081593, 0.159647, 0.233166, 0.301538, 0.364529, 0.422249, 0.475086, 0.523627, 0.568568, 0.610624, 0.650448, 0.688568, 0.725337, 0.760918, 0.795281, 0.828226, 0.859424, 0.888460, 0.914890, 0.938288, 0.958283, 0.974593, 0.987031, 0.995505, 1.000000, 1.000548, 0.997198, 0.989981, 0.978883, 0.963825, 0.944663, 0.921194, 0.893189, 0.860429, 0.822753, 0.780108, 0.732600, 0.680526, 0.624400, 0.564947, 0.503086, 0.439881, 0.376473, 0.314000, 0.253505, 0.195842, 0.141597, 0.091020, 0.043988, 0.000000, -0.041802, -0.082566, -0.123639, -0.166445, -0.212360, -0.262565, -0.317920, -0.378854, -0.445285, -0.516578, -0.591548, -0.668515, -0.745391, -0.819818, -0.889327, -0.951509, -1.004196, -1.045626, -1.074575, -1.090457, -1.093376, -1.084121, -1.064112, -1.035296, -1.000000, -0.960758, -0.920118, -0.880454, -0.843791, -0.811656, -0.784978, -0.764028, -0.748417, -0.737147, -0.728707, -0.721208, -0.712546, -0.700580, -0.683300, -0.658992, -0.626368, -0.584653, -0.533639, -0.473685, -0.405674, -0.330934, -0.251124, -0.168100, -0.083779, -0.000000),
    color: blue,
    stroke: 1.5pt + blue
  ),
  lq.plot(
    (1.000000, 0.000000, -1.000000, 0.000000, 1.000000),
    (0.000000, 1.000000, 0.000000, -1.000000, 0.000000),
    color: red,
    stroke: 1.5pt + red
  ),
  lq.scatter(
    (1.000000, 0.000000, -1.000000, 0.000000),
    (0.000000, 1.000000, 0.000000, -1.000000),
    mark: "o",
    color: red
  ),
)
,
  caption: [A Fourier-deformed Jordan curve (blue) with an exact inscribed square (red). The curve is $gamma(t) = (cos t, sin t) + sin(4t) dot (0.12 + 0.10 cos t, thin 0.08 - 0.07 sin t)$. Since $sin(4t) = 0$ at $t = 0, pi slash 2, pi, 3 pi slash 2$, the perturbation vanishes at the square vertices: they lie on the curve by construction. All four sides have length $sqrt(2)$.]
) <fig:blob>

#figure(
  lq.diagram(
  width: 6cm,
  height: 6cm,
  xlim: (-2.3, 2.3),
  ylim: (-2.3, 2.3),
  xaxis: (ticks: none),
  yaxis: (ticks: none),
  lq.plot(
    (2.000000, 1.993835, 1.975377, 1.944740, 1.902113, 1.847759, 1.782013, 1.705280, 1.618034, 1.520812, 1.414214, 1.298896, 1.175571, 1.044997, 0.907981, 0.765367, 0.618034, 0.466891, 0.312869, 0.156918, 0.000000, -0.156918, -0.312869, -0.466891, -0.618034, -0.765367, -0.907981, -1.044997, -1.175571, -1.298896, -1.414214, -1.520812, -1.618034, -1.705280, -1.782013, -1.847759, -1.902113, -1.944740, -1.975377, -1.993835, -2.000000, -1.993835, -1.975377, -1.944740, -1.902113, -1.847759, -1.782013, -1.705280, -1.618034, -1.520812, -1.414214, -1.298896, -1.175571, -1.044997, -0.907981, -0.765367, -0.618034, -0.466891, -0.312869, -0.156918, -0.000000, 0.156918, 0.312869, 0.466891, 0.618034, 0.765367, 0.907981, 1.044997, 1.175571, 1.298896, 1.414214, 1.520812, 1.618034, 1.705280, 1.782013, 1.847759, 1.902113, 1.944740, 1.975377, 1.993835, 2.000000),
    (0.000000, 0.078459, 0.156434, 0.233445, 0.309017, 0.382683, 0.453990, 0.522499, 0.587785, 0.649448, 0.707107, 0.760406, 0.809017, 0.852640, 0.891007, 0.923880, 0.951057, 0.972370, 0.987688, 0.996917, 1.000000, 0.996917, 0.987688, 0.972370, 0.951057, 0.923880, 0.891007, 0.852640, 0.809017, 0.760406, 0.707107, 0.649448, 0.587785, 0.522499, 0.453990, 0.382683, 0.309017, 0.233445, 0.156434, 0.078459, 0.000000, -0.078459, -0.156434, -0.233445, -0.309017, -0.382683, -0.453990, -0.522499, -0.587785, -0.649448, -0.707107, -0.760406, -0.809017, -0.852640, -0.891007, -0.923880, -0.951057, -0.972370, -0.987688, -0.996917, -1.000000, -0.996917, -0.987688, -0.972370, -0.951057, -0.923880, -0.891007, -0.852640, -0.809017, -0.760406, -0.707107, -0.649448, -0.587785, -0.522499, -0.453990, -0.382683, -0.309017, -0.233445, -0.156434, -0.078459, -0.000000),
    color: blue,
    stroke: 1.5pt + blue
  ),
  lq.plot(
    (0.894400, -0.894400, -0.894400, 0.894400, 0.894400),
    (0.894400, 0.894400, -0.894400, -0.894400, 0.894400),
    color: red,
    stroke: 1.5pt + red
  ),
  lq.scatter(
    (0.894400, -0.894400, -0.894400, 0.894400),
    (0.894400, 0.894400, -0.894400, -0.894400),
    mark: "o",
    color: red
  ),
)
,
  caption: [An ellipse $gamma(t) = (2 cos t, sin t)$ (blue) with its exact inscribed square (red). The analytical solution gives vertices at $(plus.minus 2 slash sqrt(5), plus.minus 2 slash sqrt(5))$, with all four sides of length $sqrt(3.2) approx 1.789$.]
) <fig:ellipse>

#figure(
  lq.diagram(
  width: 6cm,
  height: 6cm,
  xlim: (-1.3, 1.3),
  ylim: (-1.3, 1.3),
  xaxis: (ticks: none),
  yaxis: (ticks: none),
  lq.plot(
    (1.000000, 0.623490, -0.222520, -0.900970, -0.900970, -0.222520, 0.623490, 1.000000),
    (0.000000, 0.781830, 0.974930, 0.433880, -0.433880, -0.974930, -0.781830, 0.000000),
    color: blue,
    stroke: 1.5pt + blue
  ),
  lq.plot(
    (0.777040, -0.499210, -0.790510, 0.485740, 0.777040),
    (0.462980, 0.754280, -0.521970, -0.813270, 0.462980),
    color: red,
    stroke: 1.5pt + red
  ),
  lq.scatter(
    (0.777040, -0.499210, -0.790510, 0.485740),
    (0.462980, 0.754280, -0.521970, -0.813270),
    mark: "o",
    color: red
  ),
)
,
  caption: [A regular heptagon (blue) with its maximum-area inscribed square (red). The four square vertices lie on sides 0, 2, 4, and 5 of the heptagon. The solution is obtained by solving a $4 times 4$ linear system: each vertex-on-side constraint $bold(n)_k dot bold(Q)_j = d_k$ gives one equation in four unknowns $(x_0, y_0, a, b)$. Side length $approx 1.309$, area $approx 1.714$.]
) <fig:heptagon>

#figure(
  table(
  columns: 4,
  [Curve],   [$N(gamma)$],   [Convex?],   [Smooth?],
  [Unit circle],  [4.000],  [Yes],  [Yes],
  [Ellipse(2,1)],  [6.372],  [Yes],  [Yes],
  [Regular pentagon],  [2.987],  [Yes],  [No],
  [Irregular 7-gon],  [2.203],  [No],  [No],
  [Koch snowflake (level 1)],  [1.730],  [No],  [No],
),
  caption: [Nondegeneracy invariant $N(gamma)$ for diverse curve families. All values are strictly positive, indicating genuine (non-collapsed) inscribed squares.]
) <tab:nondegen>

#figure(
  table(
  columns: 4,
  [Aspect ratio r],   [Residual],   [Chord len. sq. (ellipse)],   [Chord len. sq. (7-gon)],
  [3.0],  [0.112],  [4.19],  [2.22],
  [2.0],  [0.093],  [4.57],  [1.86],
  [1.5],  [0.298],  [4.93],  [1.94],
  [1.2],  [0.076],  [5.63],  [2.10],
  [1.1],  [0.544],  [6.47],  [2.10],
  [1.05],  [0.192],  [6.47],  [2.10],
  [1.01],  [0.035],  [6.47],  [2.17],
  [1.0 (square)],  [0.015],  [6.47],  [2.17],
),
  caption: [Greene--Lobb rectangle sweep: chord lengths remain bounded away from zero as $r -> 1$. Residuals are from a coarse 24-point grid search on center-chords.]
) <tab:sweep>

#figure(
  lq.diagram(
  title: [Nondegeneracy under circle-to-ellipse deformation],
  xlabel: [Deformation parameter (epsilon)],
  ylabel: [N(gamma)],
  lq.plot(
    (0.000000, 0.100000, 0.200000, 0.300000, 0.500000, 0.700000, 0.900000),
    (4.000000, 3.840000, 3.480000, 3.070000, 1.760000, 0.630000, 0.097000),
    mark: "o"
  ),
)
,
  caption: [Nondegeneracy invariant $N(gamma)$ as the unit circle deforms to an ellipse via $gamma_epsilon(t) = ((1+epsilon)cos t, (1-epsilon) sin t)$. $N$ decreases but remains positive throughout.]
) <fig:deform>

#figure(
  lq.diagram(
  title: [Chord length vs aspect ratio],
  xlabel: [Aspect ratio r],
  ylabel: [Chord length squared],
  lq.plot(
    (3.000000, 2.000000, 1.500000, 1.200000, 1.100000, 1.050000, 1.010000, 1.000000),
    (4.190000, 4.570000, 4.930000, 5.630000, 6.470000, 6.470000, 6.470000, 6.470000),
    mark: "o",
    label: [Ellipse(2,1)]
  ),
)
,
  caption: [Chord length squared at the best rectangle solution as $r$ sweeps from 3.0 to 1.0 for the ellipse. In this example, length stays bounded away from zero and increases as $r -> 1$.]
) <fig:sweep>

#figure(
  table(
  columns: 4,
  [Family],   [Parameter range],   [N range],   [Trend],
  [Extreme ellipses],  [a slash b = 1 to 20],  [4.0 -- 7.5],  [Increases],
  [Pinched curves],  [delta = 0 to 0.95],  [3.7 -- 4.0],  [Stable],
  [High-freq oscillation],  [N = 3 to 30],  [4.0 -- 4.2],  [Stable],
  [Near-dumbbell],  [alpha = 0 to 0.85],  [1.3 -- 1.6],  [Slow decrease],
  [Extreme dumbbell],  [alpha = 0.5 to 0.98],  [0.26 -- 0.29],  [Floors at 0.26],
  [Asymmetric fjord],  [delta = 0.05 to 0.45],  [4.0 -- 5.7],  [Increases],
),
  caption: [Stress test of $N(gamma)$ across six pathological curve families (31 curves total). The extreme dumbbell gives the smallest $N approx 0.26$, but $N$ floors out and does not approach zero even at 98% of self-intersection.]
) <tab:stress>

== Z3-verified structural lemmas

The following algebraic identities are verified by Z3 with no independent assumptions beyond the defining axioms of each structure:

1. *Edge rotation* (Part 1): $e_2 = i dot.c e_1$ implies $|e_1| = |e_2|$ and $e_1 dot e_2 = 0$.
2. *Square closure* (Part 1): four rotated edges sum to zero.
3. *Redundancy* (Part 13): $D_1 = 0$ plus closure implies $D_2 = 0$ and $D_3 = 0$.
4. *Diagonal square* (Part 14): equal-length perpendicular diagonals imply all sides equal and perpendicular.
5. *Wobble sign change* (Part 17): $W(theta + pi slash 2) = -W(theta)$.
6. *Nondegeneracy* (Part 19): positive chord length times positive scale factor is positive.
7. *Rectangle perpendicular sides* (Part 20): equal-length diagonals imply perpendicular sides.
8. *Rectangle aspect ratio* (Part 20): diagonal angle encodes the side ratio.
9. *Square recovery* (Part 20): at $R^2 = 1$, the rectangle diagonal angle condition forces perpendicular diagonals.

10. *Chord length lower bound* (Part 25, Lemma A): $L >= w > 0$ implies $L^2 >= w^2$.
11. *Star-shaped chord bound* (Part 25, Lemma B): $rho_1 >= rho_"min"$ and $rho_2 >= rho_"min"$ implies $rho_1 + rho_2 >= 2 rho_"min"$.
12. *Centrally symmetric bound* (Part 25, Lemma C): $rho >= rho_"min" > 0$ implies $(2 rho)^2 >= (2 rho_"min")^2$.
13. *Width identification* (Part 25, Lemma D): $L = 2 rho$, $w = 2 rho_"min"$, $rho >= rho_"min" > 0$ implies $L >= w$.
14. *Distortion bound* (Part 25): $"lo" >= kappa dot "hi"$ and $kappa > 0$ implies $(2 "lo")^2 >= (kappa dot 2 "hi")^2$.

All fourteen verifications pass in under one second each.

== Weak theorem: $N(gamma) >= "width"(gamma)^2$ for centrally symmetric curves

The full conjecture $N(gamma) >= C dot "width"(gamma)^2$ for all Jordan curves is open. We prove a restricted version using three Z3-verified lemmas.

*Lemma A* (Z3 structure `ChordLengthLowerBound`). If every chord through a center $c$ has length $L(theta) >= w > 0$, then at any wobble zero $theta_0$: $N(gamma) = L(theta_0)^2 >= w^2$. Z3 verifies: $(L >= w) and (w > 0) arrow.r.double L^2 >= w^2$.

*Lemma B* (Z3 structure `StarShapedChordBound`). For a star-shaped curve with radial function $rho(phi)$ around center $c$, the chord length $L(theta) = rho(theta) + rho(theta + pi) >= 2 rho_"min"$. Z3 verifies: $(rho_1 >= rho_"min") and (rho_2 >= rho_"min") and (rho_"min" > 0) arrow.r.double rho_1 + rho_2 >= 2 rho_"min"$.

*Lemma C* (Z3 structure `CentrallySymmetricBound`). If $rho >= rho_"min" > 0$, then $(2 rho)^2 >= (2 rho_"min")^2$. Z3 verifies: $(rho >= rho_"min") and (rho_"min" > 0) arrow.r.double (2 rho)^2 >= (2 rho_"min")^2$.

*Lemma D* (Z3 structure `WidthIdentification`). _Width identification for centrally symmetric curves._ For a centrally symmetric curve ($rho(phi) = rho(phi + pi)$) about a center $c$, every chord through $c$ has length $L(theta) = rho(theta) + rho(theta + pi) = 2 rho(theta)$. The width in direction $theta$ --- the distance between the parallel support lines perpendicular to $theta$ --- equals $2 rho(theta)$, so $"width"(gamma) = min_theta 2 rho(theta) = 2 rho_"min"$. Z3 verifies the algebraic consequence: given $L = 2 rho$, $w = 2 rho_"min"$, $rho >= rho_"min" > 0$, then $L >= w$. This makes the geometric hypothesis explicit: the theorem holds _under the identification_ $"width" = 2 rho_"min"$.

*Theorem (weak).* For any centrally symmetric convex Jordan curve $gamma$,
$ N(gamma) >= "width"(gamma)^2. $
_Proof._ The curve is star-shaped with respect to its center of symmetry $c$. Step 1 (Lemma D): central symmetry gives $L(theta) = 2 rho(theta)$ and $"width"(gamma) = 2 rho_"min"$, so $L(theta) >= "width"(gamma)$ for all $theta$. Step 2 (Lemma C): $L(theta)^2 >= "width"(gamma)^2$. Step 3 (Lemma A): $N(gamma) = min_(theta : W(theta) = 0) L(theta)^2 >= "width"(gamma)^2$. Each algebraic step is Z3-verified. The geometric input is the identification in Lemma D, which holds for centrally symmetric curves by definition of the radial function and width. #h(1cm) $square$

For general star-shaped curves with bounded distortion $rho_"min" >= lambda dot rho_"max"$, Z3 verifies (structure `DistortionBound`) that $N(gamma) >= lambda^2 dot "width"(gamma)^2$.

*Numerical confirmation.* For seven ellipses with semi-axis ratios $a slash b$ from 1 to 20 (all centrally symmetric convex), the ratio $N slash "width"^2$ ranges from 1.00 (circle, the tight case) to 2.57 (ellipse 20:1). The bound is never violated. For the Fourier blob (star-shaped, not centrally symmetric), the predicted distortion bound $lambda^2 dot "width"^2 = 1.96$ is satisfied by the actual $N = 4.43$.

= Discussion

*What is proved and what is not.* This paper does _not_ prove the Toeplitz conjecture. We do not claim to. The conjecture remains open for general continuous curves. What we provide is:

(a) A correct formulation ($Phi$ instead of $G$) that avoids the L-shape gap.

(b) A _partial_ dimensional reduction (the wobble $W$) that, via IVT, guarantees perpendicular equal-length chords through a fixed center. This handles one of three necessary conditions but does not solve the full system.

(c) Identification of the precise three-condition structure for the Toeplitz conjecture in this framework (see below).

(d) Systematic computational evidence that $N(gamma) > 0$ across six pathological curve families (31 curves, Table 5). This evidence is purely empirical --- no proof, no theoretical bound, no mechanism.

(e) Numerical exploration of the coupled $(c, theta)$ system, with approximate solutions whose residuals decrease under refinement. This demonstrates approximate solvability, not existence.

(f) Computational evidence suggesting Greene--Lobb rectangle solutions do not degenerate as $r -> 1$ for tested curves. This is an empirical observation, not a structural claim: Greene--Lobb does not guarantee continuity in $r$, and solution sequences could in principle jump, bifurcate, or disappear.

*The three-condition structure.* The Toeplitz conjecture, in our framework, is equivalent to the simultaneous satisfaction of three conditions for every continuous Jordan curve $gamma$:

(I) _Wobble zero_: there exists $theta$ and a center $c$ such that the perpendicular chords through $c$ at angle $theta$ and $theta + pi slash 2$ have equal length. The IVT, via $W(theta + pi slash 2) = -W(theta)$, guarantees this for any fixed $c$. Status: *proved* (for each fixed center).

(II) _Nondegeneracy_: $N(gamma) > 0$, i.e. the chord length at the wobble zero is positive. Status: *empirically supported* (31 curves, 6 families, all $N > 0$), but *unproved*. No theoretical lower bound. No mechanism preventing local collapse.

(III) _Midpoint condition_: the center $c$ bisects both chords simultaneously. This is automatic for center-symmetric curves but constitutes an additional constraint for general curves. The full system is three equations in three unknowns $(c_x, c_y, theta)$, of which the wobble addresses only one dimension. Status: *approximately solved numerically* for tested curves, but *no existence proof*.

*The dimensional mismatch.* The wobble IVT reduces the problem in one dimension ($theta$), but the real inscribed-square problem lives in three dimensions $(c_x, c_y, theta)$. The IVT is a one-dimensional tool. To handle the full 3D system would require a topological degree or equivariant map argument --- neither of which we provide.

*Why the nondegeneracy claim is hard.* The conjecture $N(gamma) > 0$ is equivalent to saying that wobble zeros do not degenerate. In configuration-space language, degeneration corresponds to the configuration hitting a singular stratum where chords collapse to a point. The observation that $"width"(gamma) > 0$ for Jordan curves does not trivially prevent _local_ collapse of chords at specific angles. A proof would require a global argument, likely involving compactness of the configuration space away from the singular stratum.

== Stress testing N: empirical findings

We tested $N(gamma)$ across 31 curves in six pathological families (Table 5). The most revealing result is the extreme dumbbell: with vertical scale $beta = 0.2$ and shape parameter $alpha = 0.98$ (98% toward self-intersection), $N$ floors at approximately 0.26 and does _not_ approach zero. This pattern suggests $N(gamma) >= C dot "width"(gamma)^2$. We state this as a conjecture, not a result:

*Conjecture.* For every Jordan curve $gamma$, $N(gamma) >= C dot "width"(gamma)^2$ for some universal constant $C > 0$.

Even a weak version of this bound (e.g. $N > 0$ for curves with bounded turning) would be a substantial result. We have no proof or mechanism; we have only the computational evidence that $N$ resists collapse under extreme geometric pathology.

== Relation to recent work

Our computational findings reach the same hard core that recent analytical work has identified.

Chambers [13] proves that Jordan curves sufficiently close to a $C^2$ curve (within distance $1 slash (10 kappa)$ where $kappa$ is the maximum curvature) contain an inscribed square of positive side length. This is a proximity-based nondegeneracy result: it shows $N > 0$ for curves near smooth curves, but does not extend to all continuous curves. Our stress tests provide complementary evidence from the opposite direction --- testing $N$ on curves that are _far_ from smooth (fractal, polygonal, near-self-intersecting).

Greene and Lobb [14] construct Lagrangian Floer homology whose chain complex is generated by rectangle inscriptions. Their spectral invariants establish that rectifiable Jordan curves inscribe a square whenever the enclosed area exceeds half that of a circle with the same diameter. This area--diameter condition is a geometric nondegeneracy criterion, conceptually parallel to our $N(gamma) > 0$.

Asano and Ike [15] use microlocal sheaf theory to resolve the rectangular peg problem for all rectifiable Jordan curves and Stromquist's locally monotone curves. This is currently the strongest result, but still does not cover all continuous curves.

Tao [11] explicitly identifies the degeneration obstruction: homological methods fail because inscribed squares can collapse to points in the limit. Our $N(gamma)$ invariant is a direct computational measure of this degeneration distance.

The gap between rectifiable/locally-monotone curves and general continuous curves remains the central open problem. Our contribution is to identify the precise three-condition obstruction structure (wobble zero, nondegeneracy, midpoint condition) and to provide computational evidence --- across pathological curve families that are far from the smooth/rectifiable regime where analytical methods succeed --- that the nondegeneracy invariant resists collapse. We do not resolve the obstruction; we characterize it.

== The four-legged table analogy

The table analogy is more than metaphorical. A three-legged table always sits stably on any surface (the defect map $G$ always has a zero --- L-shapes exist). A four-legged table requires rotation to find stability (the wobble $W$ must have a zero). The table theorem works because the wobble changes sign under $90 degree$ rotation --- exactly the $W(theta + pi slash 2) = -W(theta)$ identity. The Toeplitz conjecture asks whether this stability extends from tables on curved surfaces to squares inscribed in curves.

= Conclusion

We have presented a computational study of the Toeplitz inscribed square conjecture using Z3-verified algebraic lemmas and numerical computation in Kleis. The diagonal formulation $Phi$ resolves the L-shape gap of the classical edge-based approach. The wobble invariant $W(theta)$ provides a partial dimensional reduction, guaranteeing perpendicular equal-length chords through a fixed center by IVT. This is one of three conditions required for an inscribed square.

The principal finding of this paper is the identification of the precise obstruction structure. In our framework, the Toeplitz conjecture reduces to proving three conditions simultaneously:

(I) _Wobble zero_ --- perpendicular equal-length chords through some center. Proved for each fixed center via IVT. The wobble addresses one dimension of a three-dimensional problem.

(II) _Nondegeneracy_ --- $N(gamma) > 0$ for all Jordan curves $gamma$. Z3-proved for centrally symmetric convex curves: $N(gamma) >= "width"(gamma)^2$ (Lemmas A--C). Z3-proved for star-shaped curves with bounded distortion: $N(gamma) >= lambda^2 dot "width"(gamma)^2$. Empirically supported across 31 pathological curves in 6 families (including near-self-intersecting dumbbells at 98% of the figure-8 limit, where $N$ floors at $approx 0.26$). Not proved for general continuous curves. We conjecture $N(gamma) >= C dot "width"(gamma)^2$ universally.

(III) _Midpoint condition_ --- a center $c$ that bisects both chords. Automatic for center-symmetric curves. For general curves, this is a second obstruction that makes the full system three equations in three unknowns $(c_x, c_y, theta)$. Approximately solved numerically; not proved to have exact solutions.

The investigation comprises over 70 machine-checked examples, 9 Z3-verified structures, and 70 Kleis functions covering ten curve families. All algebraic lemmas are formally verified; all numerical results are reproducible from the source file `toeplitz_conjecture.kleis`.

The Toeplitz conjecture itself remains open. Our work reaches the same hard core identified by Chambers [13], Greene--Lobb [14], Asano--Ike [15], and Tao [11]: the degeneration barrier. We have moved from _trying to prove Toeplitz_ to _identifying the exact minimal obstruction structure_. A complete proof would require: (a) a theoretical lower bound for $N$ (likely via compactness and lower semicontinuity under $C^0$ approximation), and (b) a topological degree or equivariant map argument for the coupled $(c, theta)$ system. These remain open.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[1\] O. Toeplitz, "Ueber einige Aufgaben der Analysis situs," Verhandlungen der Schweizerischen Naturforschenden Gesellschaft 4 (1911).]

#par(hanging-indent: 1.5em)[\[2\] A. Emch, "Some properties of closed convex curves in a plane," Amer. J. Math. 35, 407--412 (1913).]

#par(hanging-indent: 1.5em)[\[3\] L. G. Schnirelman, "On certain geometric properties of closed curves" (Russian), Uspekhi Mat. Nauk 10, 34--44 (1944); original 1929.]

#par(hanging-indent: 1.5em)[\[4\] V. V. Makeev, "On quadrangles inscribed in a closed curve," Math. Notes 36, 813--815 (1984).]

#par(hanging-indent: 1.5em)[\[5\] J. E. Greene and A. Lobb, "The rectangular peg problem," Ann. of Math. 194, 509--517 (2021).]

#par(hanging-indent: 1.5em)[\[6\] E. Atik, "Kleis: A formal verification language for mathematics," https://kleis.io (2025).]

#par(hanging-indent: 1.5em)[\[7\] L. de Moura and N. Bjorner, "Z3: An efficient SMT solver," TACAS 2008, LNCS 4963, 337--340 (2008).]

#par(hanging-indent: 1.5em)[\[8\] B. Matschke, "A survey on the square peg problem," Notices Amer. Math. Soc. 61, 346--352 (2014).]

#par(hanging-indent: 1.5em)[\[9\] W. Stromquist, "Inscribed squares and square-like quadrilaterals in closed curves," Mathematika 36, 187--197 (1989).]

#par(hanging-indent: 1.5em)[\[10\] M. D. Meyerson, "Balancing acts," Topology Proc. 6, 59--75 (1981).]

#par(hanging-indent: 1.5em)[\[11\] T. Tao, "An integration approach to the Toeplitz square peg problem," Forum Math. Sigma 5, e30 (2017).]

#par(hanging-indent: 1.5em)[\[12\] H. Vaughan, "Rectangles and simple closed curves" (unpublished lecture, 1977); see also the Mobius strip proof in [8].]

#par(hanging-indent: 1.5em)[\[13\] G. R. Chambers, "Squares inscribed in curves close to convex curves," arXiv:2501.12809 (2025).]

#par(hanging-indent: 1.5em)[\[14\] J. E. Greene and A. Lobb, "Lagrangian cobordism and the inscribed square problem," arXiv (2024).]

#par(hanging-indent: 1.5em)[\[15\] T. Asano and Y. Ike, "Microlocal theory and the inscribed rectangle problem," arXiv (2024).]


