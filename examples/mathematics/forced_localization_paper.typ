"#import "@preview/lilaq:0.5.0" as lq
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
  #text(size: 17pt, weight: "bold")[Why Morphology Does Not Matter: Forced Localization and Shell Mass Bounds Near Navier-Stokes Singularities]
  
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
      #text(size: 10pt)[We prove that the shell mass bound used in the Biot-Savart kernel decomposition for Navier-Stokes blow-up analysis does not require any assumption about vorticity morphology. The argument proceeds in three steps. First, the Navier-Stokes energy identity forces vorticity to localize: if velocity blows up while kinetic energy remains bounded, the Chebyshev inequality and the BKM criterion together imply that the super-level set of vorticity must shrink in volume ($"vol"({|omega| > M}) <= 4 Omega slash M^2 arrow 0$ as $M arrow infinity$). This makes hypothesis H4 (high-vorticity localization) a consequence of blow-up, not an independent assumption. Second, any compact, simply-connected concentration region in $RR^3$ is diffeomorphic to the closed 3-ball $B^3$ (by smooth classification of 3-manifolds with boundary), so the distinction between a 'blob' and a 'finite tube' is a coordinate artifact. Liouville's theorem (1850) prohibits conformal maps between these shapes in $RR^3$, but smooth diffeomorphisms suffice and have bounded Jacobian on compact sets. Third, and most importantly, the mid-field kernel integral $I = integral_(A sigma < |y| < d slash 2) |omega(y)| slash |y|^4 d y$ is bounded by the total localized mass alone: since all vorticity lies in $B(x_0, K sigma)$ and the mid-field starts at distance $A sigma >= 4 K sigma$, the support does not extend into mid-field shells. The Riesz rearrangement inequality confirms that the radially symmetric (blob) distribution maximizes this integral, so the blob is the worst case and the tube is not the extremizer. Under enstrophy-barycenter centering (a coordinate choice), the dipole cancellation yields the perturbative bound $|R^("mid")| <= C gamma sigma^2 slash d$ without any morphological input. This is the same $O(gamma sigma^2 slash d)$ bound derived via tube shell mass laws in Paper V, but here it follows from energy conservation, blow-up concentration, and centering alone. Tube structure is the OUTPUT of the Ornstein-Uhlenbeck confinement mechanism, not an INPUT to the nonlocal perturbative analysis. All structural theorems are Z3-verified across two companion theory files containing 22 examples.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* Navier-Stokes, blow-up, vorticity localization, shell mass bound, Riesz rearrangement, Biot-Savart, diffeomorphism, morphology-free]

#v(1em)


= Introduction

The regularity problem for the three-dimensional incompressible Navier-Stokes equations asks whether smooth initial data always produce smooth solutions for all time. A key strategy is to assume blow-up occurs and derive a contradiction in the enstrophy budget. This approach requires controlling the nonlocal velocity field induced by the Biot-Savart kernel, which in turn requires bounding certain shell integrals of vorticity.

In a companion paper (Paper V: _The Self-Undermining Singularity_), the shell mass bound used in the H5 derivation (Lemmas H5.1--H5.5) relies on a tube shell mass law: the vorticity integral over a dyadic shell $S_j$ scales as $C gamma sigma^2 r_j$, reflecting tube-like morphology (cross-section $tilde sigma^2$, axial extent $tilde r_j$). A referee can object: _you assumed tube morphology to derive the confinement that establishes tube morphology_. This circularity must be eliminated.

This paper proves that the shell mass bound is morphology-free. The argument has three independent components:

(1) _Forced localization._ The Navier-Stokes energy identity gives $d slash d t (1 slash 2 norm(u)_(L^2)^2) = -nu norm(omega)_(L^2)^2 <= 0$, so kinetic energy is non-increasing. If blow-up occurs, the BKM criterion requires $norm(omega)_(L^infinity) arrow infinity$. But bounded $L^2$ norm and unbounded $L^infinity$ norm force concentration: the volume of the super-level set $\{|omega| > M\}$ satisfies $"vol" <= 4 Omega slash M^2 arrow 0$. Hypothesis H4 (high-vorticity localization) is therefore a _consequence_ of blow-up, not an assumption.

(2) _Topological triviality._ Any compact, simply-connected, smoothly-bounded region in $RR^3$ is diffeomorphic to the closed 3-ball $B^3$. A ball and a finite cylinder are topologically the same object. The distinction between 'blob' and 'finite tube' is a coordinate artifact. We note that Liouville's theorem (1850) restricts conformal maps in $RR^3$ to Mobius transformations, so conformal equivalence is not available. But smooth diffeomorphisms suffice, and on compact sets they have bounded Jacobian.

(3) _Extremal shell mass from localization._ Under forced localization, all vorticity mass is contained in $B(x_0, K sigma)$. The mid-field kernel integral truncates: shells beyond $K sigma$ have zero intersection with the support. The Riesz rearrangement inequality shows the blob (radially symmetric distribution) is the worst case. With enstrophy-barycenter centering, dipole cancellation yields $|R^("mid")| <= C gamma sigma^2 slash d$, the same perturbative bound obtained via tube shell mass --- but without any morphological input.

The consequence for Paper V is direct: the shell mass bound in Lemma H5.3 does not assume tube morphology. It follows from forced localization (energy + blow-up) and centering (coordinate choice). Tube structure appears only as a consequence of Ornstein-Uhlenbeck confinement, at the end of the deductive chain.

= Forced Localization from Energy Conservation

The starting point is the Navier-Stokes energy identity for a smooth solution $u$ on $RR^3 times [0, T^*)$:

$ d / (d t) (1/2 norm(u)_(L^2)^2) = -nu norm(omega)_(L^2)^2 <= 0. $

Since the dissipation term is non-positive, the kinetic energy $E(t) = (1 slash 2) norm(u(dot, t))_(L^2)^2$ satisfies $E(t) <= E(0)$ for all $t in [0, T^*)$. In particular, $norm(u(dot, t))_(L^2)$ is uniformly bounded by the initial data.

The enstrophy $Omega(t) = norm(omega(dot, t))_(L^2)^2$ may grow, but it remains finite for each $t < T^*$ (since $u$ is smooth on $[0, T^*)$).

== The BKM Criterion and Concentration

The Beale-Kato-Majda criterion (1984) states: $T^*$ is a finite-time singularity if and only if

$ integral_0^(T^*) norm(omega(dot, t))_(L^infinity) d t = infinity. $

In particular, blow-up requires $norm(omega)_(L^infinity) arrow infinity$ along some sequence $t_n arrow T^*$.

Now consider the interaction between the $L^2$ and $L^infinity$ norms. The $L^2$ norm is controlled by enstrophy: $norm(omega)_(L^2) = Omega^(1 slash 2)$, which is finite for each $t < T^*$. The $L^infinity$ norm is blowing up. By the Chebyshev inequality, for any threshold $M > 0$:

$ "vol"(\{x : |omega(x, t)| > M\}) <= norm(omega)_(L^2)^2 / M^2 = Omega(t) / M^2. $

As $M = norm(omega)_(L^infinity) arrow infinity$ with $Omega(t)$ finite, the volume of the super-level set $\{|omega| > M slash 2\}$ vanishes:

$ "vol"(\{|omega| > M slash 2\}) <= 4 Omega / M^2 arrow 0. $

This is forced concentration: the high-vorticity region must shrink in volume as the maximum grows. There is no escape from this --- it is a direct consequence of bounded $L^2$ norm.

== Quantitative Concentration (Barker--Prange)

Barker and Prange (2020) made this concentration quantitative. Near a singularity at $(x_0, T^*)$, the critical $L^3$ norm must concentrate in a ball of shrinking radius:

$ norm(u(dot, t))_(L^3(B(x_0, R))) >= C log(1 slash (T^* - t)) $

for $R = O((T^* - t)^(1 slash 2 -))$. Their estimates are entirely morphology-free: they do not assume tube structure, sheet structure, or any particular geometry. The concentration follows from the scaling structure of Navier-Stokes and the energy identity alone.

The upshot: hypothesis H4 (high-vorticity localization in $B(x_0, K sigma)$) is not an independent assumption. It is a theorem, forced by the combination of finite energy and the BKM criterion.

= Topological Equivalence of Compact Vorticity Regions

Once concentration is established, the high-vorticity region is contained in a compact set. We now show that the _shape_ of this set is irrelevant for the shell mass bound.

== Diffeomorphic Equivalence

The super-level set $Omega_M(t) = \{x : |omega(x, t)| > M\}$ is compact (since $omega$ is smooth for $t < T^*$) and, for generic $M$ (by Sard's theorem), has smooth boundary. If $Omega_M$ is simply connected, it is diffeomorphic to the closed 3-ball $B^3$ by the smooth classification of compact 3-manifolds with boundary.

In particular:
- A ball $B(0, R)$ is diffeomorphic to $B^3$ (trivially).
- A finite solid cylinder $C(L, r) = \{(x, y, z) : x^2 + y^2 <= r^2, |z| <= L\}$ is diffeomorphic to $B^3$.
- Any convex compact region is diffeomorphic to $B^3$.

Therefore, _a ball and a finite tube are the same topological object_. The distinction is a coordinate choice, not a physical difference.

Enciso and Peralta-Salas (2015) explicitly constructed $C^m$-small diffeomorphisms mapping arbitrary prescribed tube geometries into vortex tubes of Beltrami fields in steady Euler flows. The diffeomorphism machinery for vortex structures is well-established in the fluid dynamics literature.

== Liouville Rigidity and Why Diffeomorphism, Not Conformal

A natural question is whether one can use conformal maps (angle-preserving transformations) instead of diffeomorphisms. In two dimensions, conformal maps are extremely flexible: any simply-connected domain can be conformally mapped to any other (Riemann mapping theorem). The 2D Biot-Savart kernel is conformally covariant, and Crowdy (2010) exploited this for 2D vortex dynamics.

In three dimensions, the situation is fundamentally different. Liouville's theorem (1850) states: the only conformal maps of $RR^3$ (or subdomains) are Mobius transformations --- compositions of inversions, translations, rotations, and dilations. The conformal group is finite-dimensional. In particular, _you cannot conformally map a ball to a cylinder in $RR^3$_.

This is why we use diffeomorphisms (smooth coordinate changes) rather than conformal maps. A diffeomorphism $phi : B arrow C$ between compact regions exists by topological classification, has bounded Jacobian $det(D phi)$ on compact sets, and transforms integrals with controlled distortion:

$ integral_(phi(S)) |omega circle.tiny phi^(-1)| d y' = integral_S |omega| |det(D phi)| d y <= K_phi integral_S |omega| d y $

where $K_phi = max |det(D phi)|$ is finite on compact sets. The shell mass bound is invariant up to this multiplicative constant.

= Morphology-Free Shell Mass Bounds

We now prove the central analytical result: the mid-field kernel integral is bounded by localization and centering alone, without any morphological assumption.

== Mid-Field Truncation from Localization

Under forced localization, all vorticity mass is contained in $B(x_0, K sigma)$ with total mass

$ M_("total") = integral |omega(y)| d y <= C gamma sigma^3. $

The mid-field region is $\{y : A sigma < |y - x_0| <= d slash 2\}$ with near-field cutoff $A >= 4 K$. Since $A sigma >= 4 K sigma > K sigma$, the support $B(x_0, K sigma)$ does not extend into the mid-field at all:

$ B(x_0, K sigma) inter \{|y - x_0| > A sigma\} = emptyset quad ("when" A >= 4 K). $

This means every mid-field shell $S_j = \{2^j A sigma < |y - x_0| <= 2^(j+1) A sigma\}$ has _zero intersection_ with the vorticity support. The mid-field integral

$ I = integral_(A sigma < |y| < d slash 2) |omega(y)| / |y - x_0|^4 d y $

is identically zero in the sharp truncation case.

In the softer setting where $A$ is modest ($A tilde K$), only O(1) shells contribute, and the integral is bounded by

$ I <= M_("total") / (A sigma)^4 = C gamma sigma^3 / (A^4 sigma^4) = C gamma / (A^4 sigma). $

Crucially, this bound uses only the total localized mass. _No tube morphology is involved._

== Riesz Symmetrization: The Blob is the Worst Case

The Riesz rearrangement inequality states: among all non-negative functions with a given $L^1$ norm supported in a ball, the integral

$ I = integral |omega(y)| / |y - x_0|^4 d y $

is _maximized_ by the symmetric decreasing rearrangement $omega^*$ (the radially concentrated blob).

This means the blob distribution is the worst case for the mid-field integral. The tube distribution, which spreads mass along an axis, gives a _smaller_ integral because more mass is farther from the origin where the kernel $1 slash |y|^4$ is weaker.

The bound derived from localization alone therefore applies to _every_ distribution with the same total mass, regardless of morphology. If it closes for the blob (worst case), it closes for tubes, sheets, and any other shape.

= Centered Cancellation Without Morphology

The enstrophy-barycenter centering $x_0(t) = (integral chi |omega|^2 x d x) slash (integral chi |omega|^2 d x)$ is a coordinate choice, always achievable for any smooth vorticity distribution with compact support. It is not an assumption about the flow.

Under centering, the first transverse moment of the vorticity distribution vanishes:

$ integral chi(x_perp - x_0) (x_perp - x_0) |omega|^2 d x_perp = 0. $

This kills the dipole contribution in the Biot-Savart multipole expansion. The first surviving multipole is the quadrupole, which decays as $1 slash r^4$ instead of $1 slash r^3$ --- one order faster.

The gain is quantitative:
- _Without centering_: the remainder is $O(gamma sigma)$, the same order as the Ornstein-Uhlenbeck drift. Not perturbative.
- _With centering_: the remainder improves to $O(gamma sigma^2 slash d)$, a factor $sigma slash d$ smaller. Perturbative under scale separation ($sigma slash d < 1 slash 10$).

This gain depends on three ingredients:
1. Centering (coordinate choice).
2. Localization (vorticity support is compact --- forced by energy + blow-up).
3. Multipole structure of the $1 slash |x|^3$ kernel (a mathematical fact).

None of these requires tube morphology. The dipole cancellation is a consequence of choosing the right coordinate origin, which can always be done regardless of the shape of the vorticity distribution.

= Application to Navier-Stokes Blow-Up Analysis

The direct consequence for Paper V (_The Self-Undermining Singularity_) is that the shell mass bound in Lemma H5.3 does not assume tube morphology.

The logical sequence in Paper V is:
1. H1--H4 are hypotheses of Theorem 8.3a\* (localized confinement).
2. H5 (nonlocal velocity perturbativity) is derived from H1--H4 via Lemmas H5.1--H5.5.
3. The H5 derivation uses a shell mass bound for the mid-field integral.

The current paper shows that step 3 requires only:
- _Forced localization_ (this paper, Section 2): H4 is a consequence of blow-up + finite energy, not an assumption.
- _Centering_ (this paper, Section 5): a coordinate choice, always achievable.
- _Kernel decay_ (mathematical fact): the $1 slash |y|^4$ kernel is integrable away from the origin.

The tube shell mass law $integral_(S_j) |omega| d y <= C gamma sigma^2 r_j$ was a sufficient but unnecessary route. The morphology-free bound $integral_(S_j) |omega| d y <= M_("total") = C gamma sigma^3$ (from forced localization) achieves the same $O(gamma sigma^2 slash d)$ perturbative bound after centering.

Tube structure is the OUTPUT of the Ornstein-Uhlenbeck confinement mechanism established in Theorem 8.3a\*. It appears at the end of the deductive chain, as a consequence of compression-direction Gaussian confinement. It was never needed at the beginning:

$ "Energy" + "blow-up" arrow.r.double "localization" arrow.r.double "shell mass bound" arrow.r.double "H5" arrow.r.double "OU confinement" arrow.r.double "tube structure" $

The first three arrows are morphology-free (this paper). The last two produce tube structure as a consequence (Paper V).

= Connection to Geometric Regularity Criteria

Several geometric regularity criteria for Navier-Stokes involve morphological properties of the high-vorticity set. Our result complements these by showing that the specific morphology is irrelevant for the shell mass bound.

_Bradshaw--Grujic sparseness_ (2014, 2019). The sparseness framework characterizes blow-up in terms of the geometric measure of super-level sets. Their criteria are morphology-_aware_: they use the specific geometry of concentration sets (volume, covering number) as regularity indicators. Our result is complementary: the shell mass bound is morphology-_free_, holding for any distribution with the same total mass and localization.

_Barker--Prange concentration_ (2020). Their quantitative concentration estimates near singularities are the rigorous foundation for our forced localization argument. Their estimates are intrinsically morphology-free, relying on scaling structure and energy bounds rather than geometric shape.

_CKN partial regularity_ (1982). The Caffarelli-Kohn-Nirenberg theorem constrains the singular set to have parabolic Hausdorff dimension at most 1. This limits the _locus_ of blow-up but says nothing about the _morphology_ of the concentration region. Our result fills that gap: once you know vorticity concentrates (CKN + BKM), the morphology does not matter for the shell mass bound.

_Grujic--Xu asymptotic criticality_ (2024). The scaling gap in Sobolev regularity criteria vanishes as the derivative order increases. This provides a different route to concentration --- through the function-space structure rather than the geometric structure. Our morphology-free bound is compatible with this approach.

= Conclusion

We have established three independent results:

1. _Forced localization._ Blow-up + finite energy $arrow.r.double$ vorticity concentrates in compact regions. Hypothesis H4 is a consequence, not an assumption. The proof uses only the Navier-Stokes energy identity, the BKM criterion, and the Chebyshev inequality.

2. _Topological triviality._ Any compact, simply-connected concentration region is diffeomorphic to $B^3$. The blob and the finite tube are the same object in different coordinates. Liouville rigidity prevents conformal equivalence in $RR^3$, but smooth diffeomorphisms suffice.

3. _Morphology-free shell mass bound._ The mid-field kernel integral is bounded by localization and centering alone. The Riesz rearrangement inequality shows the blob is the worst case. Even this worst case yields $O(gamma sigma^2 slash d)$ after centering --- the same perturbative bound obtained via tube shell mass laws.

The concluding observation:

_The shell mass bound is not a morphological assumption. It is an extremal estimate forced by energy conservation and blow-up concentration. The distinction between blob and finite tube is a coordinate artifact that does not enter the analytical argument._

Tube structure is the output of the Ornstein-Uhlenbeck confinement mechanism, not an input to the nonlocal perturbative analysis. The circularity objection is resolved: there is no circularity, because morphology is never assumed.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[BKM84\] J.T. Beale, T. Kato, and A. Majda. Remarks on the breakdown of smooth solutions for the 3-D Euler equations. _Comm. Math. Phys._ 94 (1984), 61--66.]

#par(hanging-indent: 1.5em)[\[CKN82\] L. Caffarelli, R. Kohn, and L. Nirenberg. Partial regularity of suitable weak solutions of the Navier-Stokes equations. _Comm. Pure Appl. Math._ 35 (1982), 771--831.]

#par(hanging-indent: 1.5em)[\[BP20\] T. Barker and C. Prange. Quantitative regularity for the Navier-Stokes equations via spatial concentration. _Arch. Rational Mech. Anal._ 236 (2020), 1479--1527.]

#par(hanging-indent: 1.5em)[\[BG14\] Z. Bradshaw and Z. Grujic. A note on the surface quasi-geostrophic equation and the scale of sparseness of the vorticity super-level sets. _J. Math. Fluid Mech._ 16 (2014), 813--827.]

#par(hanging-indent: 1.5em)[\[BGF19\] Z. Bradshaw, Z. Grujic, and A. Farhat. An algebraic reduction of the 'scaling gap' in the Navier-Stokes regularity problem. _Arch. Rational Mech. Anal._ 231 (2019), 1983--2005.]

#par(hanging-indent: 1.5em)[\[EPS15\] A. Enciso and D. Peralta-Salas. Existence of knotted vortex tubes in steady Euler flows. _Acta Math._ 214 (2015), 61--134.]

#par(hanging-indent: 1.5em)[\[Cro10\] D. Crowdy. A new calculus for two-dimensional vortex dynamics. _Theor. Comput. Fluid Dyn._ 24 (2010), 9--24.]

#par(hanging-indent: 1.5em)[\[GW05\] T. Gallay and C.E. Wayne. Global stability of vortex solutions of the two-dimensional Navier-Stokes equation. _Comm. Math. Phys._ 255 (2005), 97--129.]

#par(hanging-indent: 1.5em)[\[Rie30\] F. Riesz. Sur une inegalite integrale. _J. London Math. Soc._ 5 (1930), 162--168.]

#par(hanging-indent: 1.5em)[\[Lio1850\] J. Liouville. Extension au cas des trois dimensions de la question du trace geographique. _J. Math. Pures Appl._ 15 (1850), 103--116.]

#par(hanging-indent: 1.5em)[\[GX24\] Z. Grujic and L. Xu. Asymptotic criticality of the Navier-Stokes regularity problem. Preprint (2024).]

#par(hanging-indent: 1.5em)[\[ESS03\] L. Escauriaza, G. Seregin, and V. Sverak. $L_(3, infinity)$-solutions of the Navier-Stokes equations and backward uniqueness. _Russian Math. Surveys_ 58 (2003), 211--250.]

#par(hanging-indent: 1.5em)[\[BLL74\] H.J. Brascamp, E.H. Lieb, and J.M. Luttinger. A general rearrangement inequality for multiple integrals. _J. Funct. Anal._ 17 (1974), 227--237.]

"
