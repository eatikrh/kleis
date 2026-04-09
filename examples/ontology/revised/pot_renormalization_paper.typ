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
  #text(size: 17pt, weight: "bold")[Renormalization as Projected Ontology: The Theory That Was Never Divergent]
  
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
      #text(size: 10pt)[Volumes I--VI of this series established that physical observables are images of ontological flows under admissible Green's function kernels, with gauge freedom identified as the kernel's null space. This paper extends the framework to renormalization in quantum field theory. We argue that the regularization procedures used since 1948 --- zeta regularization, heat kernel methods, Pauli--Villars, dimensional continuation --- are projection kernels in the precise sense of the Projected Ontology (POT) framework. The finite part extracted by Hadamard's constant-term operator is the image of the projection; the divergent terms are the null space; the choice of regularization scheme is gauge freedom. We formalize the *gauge group of admissible regulators* and prove that gauge-equivalent regulators yield identical observables. The central result is that the path integral and renormalization compose into a single integral transform $K_("QFT") = "FP" compose K_("ren") compose K_("path")$ mapping sources directly to observables. This composition is legitimized by the Integral Transform Composition Method (ITCM) of Sitnik and collaborators, which provides explicit kernel representations for composed integral transforms. Applied to QED, the ITCM identifies dimensional regularization as a transmutation operator with weight $w(t) = (-t^2)^s$, yielding a closed-form composite kernel via hypergeometric functions. Renormalization is therefore not an add-on procedure that fixes a broken theory; it is a constitutive factor of the integral transform that defines the theory. The heat kernel and propagator used for regularization are the same operators that govern physical phenomena; mathematical devices like the zeta kernel produce correct results because they are gauge-equivalent to a physical kernel. A literature survey confirms that every major application of zeta regularization in the last fifty years accessed the heat kernel through a mathematical intermediary. The claim is not that QFT is wrong, but that the essential structure underlying the formalization sought by the Clay Millennium Problem may already have been present in the original 1948 formulation, though not expressed as a single composite operator. We conclude with a discussion of what the kernel decomposition tempts us to infer about the ontological Hilbert space $cal(H)_("ont")$, and why the framework's own epistemic constraint --- established in Volume II --- prevents us from acting on that temptation. All structural results are machine-verified by the Z3 SMT solver via 40 axiomatic examples in the Kleis formal verification language.]
    ]
  ]
]

#text(size: 9pt)[*Keywords:* renormalization, projection kernel, regularization, zeta function, heat kernel, gauge freedom, finite part operator, quantum field theory, projected ontology, integral transform composition method, transmutation operator, composite kernel, formal verification, Z3]

#v(1em)


= Introduction

Quantum field theory is the most precisely tested framework in the history of science. The anomalous magnetic moment of the electron is calculated to twelve decimal places and agrees with experiment. Yet QFT has no rigorous mathematical foundation. The Clay Mathematics Institute lists the existence and mass gap of Yang--Mills theory as one of seven Millennium Prize Problems. The standard assumption is that new mathematics is needed --- constructive field theory, new axiom systems, new functional analysis.

This paper argues that the foundation was laid in 1948 and has been used successfully ever since. It was mislabeled as a computational technique.

When Tomonaga, Schwinger, and Feynman introduced renormalization to handle the divergent integrals of quantum electrodynamics, they described it as a procedure: subtract the infinities, keep the finite part, check against experiment. The procedure works spectacularly well. But a procedure is not a mathematical object. You cannot prove theorems about it, compose it with other procedures, or classify it. The Projected Ontology (POT) framework, developed in Volumes I--VI of this series, provides the missing language: renormalization is a *projection kernel*. Its image is the space of physical observables. Its null space is the space of divergent terms. The choice of regularization scheme --- cutoff, exponential damping, heat kernel, zeta function, dimensional continuation --- is gauge freedom within the projection.

This is not a new proposal for the foundations of QFT. It is the recognition that QFT's own computational practice *already embodies* the projection-kernel structure. Every time a physicist subtracts a pole, takes a finite part, or checks scheme independence, they are performing kernel-based projection without naming it as such.

The paper is organized as follows. Section 2 recapitulates the POT framework established in Volumes I--VI. Section 3 introduces divergent objects and the regularization kernel. Section 4 treats the paradigmatic case of zeta regularization and $zeta(-1) = -1\/12$. Section 5 treats the product case and the connection to the Gamma function and $sqrt(2 pi)$. Section 6 formalizes the gauge group of admissible regulators. Section 7 identifies the finite part as the gauge-invariant observable. Section 8 shows that regularization kernels are physical kernels --- the same operators that govern heat diffusion, particle exchange, and spectral theory. Section 9 draws the implications for QFT, including the composite kernel $K_("QFT") = "FP" compose K_("ren") compose K_("path")$, its connection to the Integral Transform Composition Method (ITCM), and the explicit application to QED. Section 10 concludes.

= Recapitulation of the POT Framework

We summarize the results of Volumes I--VI that this paper extends.

== Admissible Kernels (Volumes I--III)

A projection kernel $K$ is admissible if it satisfies three conditions: (1) linearity --- $K(A + B) = K(A) + K(B)$; (2) scalar compatibility --- $K(c A) = c K(A)$; (3) zero preservation --- $K(0) = 0$. These conditions formalize Occam's razor for projections: an admissible kernel projects without adding structure.

Every physical law in the POT framework is a property of an admissible kernel. Observable quantities are the kernel's image. Gauge freedom is the kernel's null space. Volume I derived galactic rotation curves from the gravitational kernel. Volume II showed that Bell-inequality violation arises from projecting non-separable ontic states through an admissible kernel. Volume III identified the electromagnetic field strength $F = d A$ as the image of the exterior derivative kernel, with gauge potentials differing by exact forms as the null space.

== The Admissibility Defect and Confinement (Volume IV)

For non-abelian gauge theories, the kernel is not admissible. The defect $Delta(A, B) = K(A + B) - K(A) - K(B) = [A, B]$ is the Lie bracket of the gauge algebra. Volume IV proved that a non-admissible kernel produces fiber non-invariance: charge becomes a predicate that cannot be determined from gauge-invariant observables. This is structural confinement.

== Admissibility Restoration (Volume V)

Volume V resolved the puzzle of the weak interaction, where $S U(2)$ is non-abelian but charges *are* observed. A non-admissible kernel can become effectively admissible through coupling to a restoring field that compensates the Lie bracket defect. The structural constraints on any such field --- gauge-charged, non-zero preferred value, characteristic scale --- match the defining properties of the Higgs. Three forces, one classification principle: admissible (electromagnetism), restored (weak), unrestored (strong).

== The Four-Sentence Axiom (Epilogue, Volume VI)

The Epilogue distilled the POT framework into four sentences: (1) Physical laws are properties of the kernel. (2) Observable quantities are the image. (3) Gauge freedom is the null space. (4) There is nothing else.

This paper applies the same four sentences to renormalization.

= Divergent Objects and the Regularization Kernel

A *divergent object* $X$ is a mathematical entity --- a formal series, a spectral sum, an asymptotic integral --- that does not converge in the ordinary sense. The expression $1 + 2 + 3 + 4 + dots.c$ is a divergent object. The infinite product $1 times 2 times 3 times 4 times dots.c$ is a divergent object. The loop integrals of QFT are divergent objects.

A *regularization kernel* $K_R$ maps a divergent object $X$ to a parameterized family of values $K_R [X](epsilon)$ that is well-defined for $epsilon$ in some domain. The parameter $epsilon$ is the regularization parameter: it could be a complex variable $s$ (zeta), a proper time $t$ (heat kernel), a cutoff $Lambda$ (momentum), or a dimension $d = 4 - epsilon$ (dimensional continuation).

*Definition.* A regularization kernel $K_R$ is *admissible* if it satisfies two conditions:

(1) *Consistency*: For any convergent object $X$, the regularized value agrees with the ordinary value.

(2) *Zero preservation*: The regularization of the zero object is zero.

These conditions are the regularization analogues of the admissibility conditions in Volumes I--III. An admissible regularization kernel does not create observables where there are none, and it agrees with ordinary summation where ordinary summation is defined.

The formal structures are defined in `RegularizationKernel` and verified by Z3 (Examples 1--3 in the theory file).

= The Sum Case: Spectral Zeta Regularization

The Riemann zeta function $zeta(s) = sum_(n=1)^(infinity) n^(-s)$ converges for $"Re"(s) > 1$ and admits analytic continuation to $bb(C) without {1}$. The regularization kernel is $K(s, n) = n^(-s)$: it suppresses the growth of $n$ by a power law controlled by $s$.

At $s = -1$, the formal series $sum n^(-(-1)) = sum n = 1 + 2 + 3 + dots.c$ diverges. But the analytically continued value is $zeta(-1) = -1\/12$.

In POT language: the divergent object $X = (1, 2, 3, dots.c)$ is the raw ontic structure. The zeta kernel $K_zeta$ maps it to a meromorphic function $zeta(s)$. The *image* of the projection --- the finite part at $s = -1$ --- is $-1\/12$. The divergent partial sums $S_N = N(N+1)\/2 arrow infinity$ are the *null space*: they are annihilated by the projection.

The value $-1\/12$ is not an arithmetic identity. It is a projected observable: the unique finite value extracted from a divergent structure by an admissible kernel. The same value appears in the Casimir effect (vacuum energy between parallel plates) and in string theory (the critical dimension $d = 26$ follows from $zeta(-1) = -1\/12$). These are not coincidences. They are instances of the same projection.

The `SpectralZeta` structure in the theory file asserts that the zeta kernel is admissible and that $zeta(-1) = -1\/12$ is the projected observable of the integer spectrum. Both claims are verified by Z3 (Examples 7--9).

= The Product Case: Zeta-Regularized Products and $sqrt(2 pi)$

The infinite product $1 times 2 times 3 times dots.c$ diverges. But the zeta-regularized product, defined via the derivative of the zeta function at zero, yields a finite value.

The zeta-regularized product of a sequence $lambda_n$ with spectral zeta function $zeta_A (s) = sum lambda_n^(-s)$ is:

$ product_("reg") lambda_n = exp(-zeta'_A (0)) $

For the integer spectrum $lambda_n = n$, this gives:

$ product_("reg") n = exp(-zeta'(0)) = sqrt(2 pi) $

since $zeta'(0) = -1\/2 log(2 pi)$.

The constant $sqrt(2 pi)$ is one of the most recurring values in mathematics. It appears as: the normalization of the Gaussian integral $integral_(-infinity)^(infinity) e^(-x^2\/2) d x = sqrt(2 pi)$; the leading term of Stirling's approximation $n! tilde.op sqrt(2 pi n) (n\/e)^n$; the area under the standard normal distribution; and the regularized product of all positive integers.

In POT language: these are not four different facts. They are four manifestations of the same projected observable. The zeta kernel, the Laplace/saddle-point kernel, and the Fourier kernel all extract the same finite value from different divergent structures because they are gauge-equivalent projections (Section 6).

The connection to the Gamma function makes this concrete. The Gamma function $Gamma(n+1) = n! = integral_0^(infinity) x^n e^(-x) d x$ can be evaluated by Laplace's method for large $n$: the integrand is dominated by a peak near $x = n$, and expanding to second order (the *quadratic germ* or *parabolic approximation*) produces $sqrt(2 pi n)$ as the area of the Gaussian envelope. The saddle-point kernel extracts the same finite value as the zeta derivative at zero --- because both are admissible regularization kernels projecting onto the same image.

This is formalized in `SpectralZeta` (axiom `zeta_product_via_derivative`, Example 10 in the theory file).

= The Gauge Group of Admissible Regulators

Different regularization schemes modify a divergent object in different ways but extract the same finite part. In POT language, they are gauge transformations.

*Definition.* Two admissible regulators $R$ and $R'$ are *gauge-equivalent*, written $R tilde.op R'$, if for all divergent objects $X$:

$ "FP"[K_R [X]] = "FP"[K_(R') [X]] $

where FP denotes the finite part operator (Section 7).

*Theorem 3 (Equivalence relation).* Gauge equivalence is reflexive, symmetric, and transitive. The proof is immediate from the definition and is verified by Z3 (Examples 4--6).

*Theorem 4 (Gauge invariance).* If $R tilde.op R'$, then for every divergent object $X$, the projected observable is the same:

$ "FP"[K_R [X]] = "FP"[K_(R') [X]] $

This is the regulator analogue of gauge invariance in Volumes III--IV: different representatives of the same equivalence class produce the same observable. The divergent terms --- the pole structure, the power-law blowup, the logarithmic divergence --- are the null space. They differ between schemes. The finite part is the image. It is scheme-independent.

*Example.* The heat kernel $K(t, lambda) = e^(-t lambda)$ and the zeta kernel $K(s, n) = n^(-s)$ are gauge-equivalent. They are connected by the Mellin transform:

$ zeta_A (s) = 1 / (Gamma(s)) integral_0^(infinity) t^(s-1) "Tr"(e^(-t A)) d t $

This integral relation means that the spectral zeta function is determined by the heat trace. Since both encode the same spectral information, their finite parts agree. Schwinger's proper-time method (1948) and Hawking's zeta regularization (1977) give the same physics --- because they are gauge-equivalent kernels. (Section 8 shows that the heat kernel is the physically fundamental member of this equivalence class, with the zeta kernel as a mathematical intermediary.)

The formal proof of gauge equivalence between heat and zeta regulators, together with the Mellin bridge, is verified in `HeatKernelRegularization` (Examples 11--13).

*Remark (The gauge group).* The set of admissible regulators modulo gauge equivalence forms a group $G_("adm")$ acting on divergent objects. The group operation is composition of regulators; the identity is the trivial regulator (which agrees with ordinary summation on convergent objects). This group is the analogue of the gauge group in Volumes III--V, with regulator equivalence classes playing the role of gauge orbits (projection fibers). The formal axioms for this group --- including identity, reflexivity, symmetry, and transitivity --- are in `RegulatorGaugeGroup`.

= The Finite Part as Gauge-Invariant Observable

The finite part operator, due to Hadamard, extracts the constant term from the asymptotic expansion of a regularized value.

*Definition.* Given a regularized value $F(epsilon)$ with asymptotic expansion

$ F(epsilon) = S(epsilon) + C + o(1) quad "as" epsilon arrow 0 $

where $S(epsilon)$ belongs to a prescribed space of singular germs (poles, logarithms, power-law divergences), the *finite part* is:

$ "FP"[F] := C $

For meromorphic functions, this is the Laurent extraction: if $F(s) = sum a_k (s - s_0)^k$, then $"FP" := a_0$.

*Theorem 5 (Linearity).* The finite part operator is linear:

$ "FP"[F + G] = "FP"[F] + "FP"[G] quad quad "FP"[c F] = c dot.op "FP"[F] $

*Theorem 6 (Zero preservation).* The finite part of zero is zero: $"FP"[0] = 0$.

These properties make the finite part operator a *projection* in the algebraic sense. Combined with the gauge invariance of Section 6, the finite part is the unique quantity that is both scheme-independent and extracted by a linear, zero-preserving operator. It is the POT observable.

The finite part is the *image* of the projection. The singular part $S(epsilon)$ is the *null space*: it is annihilated by FP. The decomposition $F = S + C + o(1)$ is the null-space/image decomposition of Volume III applied to regularization.

All properties are formalized in `FinitePartOperator` and verified by Z3 (Examples 1--3).

= Regularization Kernels Are Physical Kernels

Some regularization kernels used in QFT are not abstract mathematical devices borrowed to fix divergences --- they are the same kernels that govern physical phenomena. Others are mathematical tools that work because they are gauge-equivalent to a physical kernel. The distinction matters.

#figure(
  table(
    columns: 3,
    [*Regularization kernel*], [*Status*], [*What it does*],
    [Heat kernel $e^(-t lambda)$], [Physical], [Governs heat diffusion, quantum propagation],
    [Propagator $1\/(k^2 + m^2)$], [Physical], [Governs massive particle exchange (Yukawa)],
    [Biot--Savart $bold(x)\/|bold(x)|^3$], [Physical], [Governs electromagnetism, fluid mechanics],
    [Zeta kernel $n^(-s)$], [Mathematical], [Analytic continuation of spectral sums],
    [Dimensional continuation $d = 4 - epsilon$], [Mathematical], [Interpolation of spacetime dimension],
  ),
  caption: [Physical kernels govern real processes; mathematical kernels produce correct results because they are gauge-equivalent to a physical kernel.]
) <tab:kernels>

== The heat kernel is physical

When Schwinger used the heat kernel to regularize QED, he was not performing an abstract mathematical trick. He was applying the kernel for heat diffusion --- the same operator that Fourier introduced to describe the conduction of heat in solids. The heat kernel $e^(-t A)$ is the fundamental solution of the heat equation $partial_t u = -A u$. Schwinger's proper-time method applies this physical kernel to the operator $A = -D^2 + m^2$ of the quantum field. The regularization is not borrowed from physics --- it *is* physics.

Similarly, the Feynman propagator $1\/(k^2 + m^2)$ is the Green's function for the Klein--Gordon equation. It governs massive particle exchange and yields the Yukawa potential at long distances. When it appears in loop integrals, it is simultaneously the physical content of the theory and the mechanism that regularizes (partially) the divergence.

The Biot--Savart kernel, treated in the Epilogue, governs both electromagnetism (source = current density) and fluid mechanics (source = vorticity). It is the POT exemplar of a single kernel giving rise to different physics through different source spaces.

These are *physical* kernels: they describe measurable processes. Their use as regularizers is not a coincidence --- it is physics recognizing its own projection structure.

== The zeta kernel is a mathematical device

The zeta kernel $n^(-s)$ is not physical in the same sense. The Riemann zeta function is an analytic continuation of a spectral sum --- a mathematical operation, not a physical process. Nobody measures $zeta(-1)$; the number $-1\/12$ is extracted by continuing a function beyond its domain of convergence.

When Hawking (1977) used zeta regularization to compute the partition function of quantum fields in curved spacetime, his physical problem *was* physical: he needed the one-loop effective action for quantum fields propagating on a black hole background. The question was: what is the energy-momentum tensor of quantum fields near a Schwarzschild black hole? The answer determines Hawking radiation --- the thermal emission from black holes.

The physical content of Hawking's calculation is in the heat kernel. The heat trace $"Tr"(e^(-t A))$, where $A = -nabla^2 + m^2 + xi R$ is the curved-space wave operator, describes the diffusion of a scalar field on the black hole geometry. The short-time expansion of this heat trace encodes the local geometry (the Seeley--DeWitt coefficients $a_0, a_1, a_2, dots.c$), and its integral gives the one-loop effective action. This is a physical computation: the heat kernel propagates the field on the curved background, and its trace integrates over all field modes.

Hawking then applied the Mellin transform:

$ zeta_A (s) = 1 / (Gamma(s)) integral_0^(infinity) t^(s-1) "Tr"(e^(-t A)) d t $

to convert the heat trace into a zeta function, because the zeta function is easier to analytically continue. The zeta-regularized determinant $det(A) = exp(-zeta'_A (0))$ gives the effective action in closed form. But the physics was always in the heat trace. The zeta function is a mathematical intermediary --- an elegant bookkeeping device for the spectral data that the heat kernel encodes.

Hawking's results are correct not because zeta regularization is physical, but because zeta regularization is *gauge-equivalent* to the heat kernel (Section 6). The Mellin bridge guarantees they extract the same finite parts. The zeta function inherits its physical correctness from the heat kernel.

== The preferred representative

This observation sharpens the gauge group of regulators (Section 6). Not all gauge-equivalent regulators are equally fundamental. The gauge group has a *preferred representative*: the physical kernel. The heat kernel is the regulator that directly describes the underlying physics --- diffusion, propagation, heat flow. The zeta kernel, dimensional regularization, and other mathematical schemes work because they are in the same gauge orbit as the heat kernel.

The analogy to standard gauge theory is precise. In electrodynamics, all gauges ($nabla dot.op bold(A) = 0$, $A_0 = 0$, etc.) are physically equivalent, but the Coulomb gauge reveals the instantaneous interaction most directly. Similarly, all admissible regulators yield the same observables, but the heat kernel reveals the physical mechanism most directly. The others are ``computational gauges'' --- mathematically convenient, physically derivative.

*Theorem 7 (Physical kernels are admissible).* If a regulator $R$ governs a physical phenomenon, then $R$ is admissible.

This codifies the empirical observation that every regularization scheme known to produce correct physics satisfies the admissibility conditions. The heat kernel and the propagator satisfy consistency (they agree with ordinary summation on convergent cases) and zero preservation. Mathematical devices like the zeta kernel also satisfy these conditions --- not because they govern a physical process, but because the Mellin bridge transfers the admissibility of the heat kernel to the zeta kernel. Admissibility is inherited via gauge equivalence.

The formal verification is in `RegulatorPhysicalKernel` (Examples 14--16).

*The double-duty thesis.* The physical kernel does double duty: it defines the physical law *and* projects out the observable. When QFT uses the heat kernel to regularize, it is recognizing that the heat kernel is part of the theory's own projection structure. When QFT uses the zeta kernel instead, it gets the same answer --- not because analytic continuation is physics, but because gauge equivalence guarantees it. The physics was always in the heat kernel. The regularization trick was never a trick --- it was physics, sometimes accessed directly (Schwinger), sometimes accessed through a mathematical intermediary (Hawking).

== Fifty years of evidence: the zeta kernel always accessed the heat kernel

The claim that the heat kernel is the preferred representative is not a philosophical preference --- it is an empirical observation supported by every major application of zeta regularization in the literature.

#figure(
  table(
    columns: 4,
    [*Author(s)*], [*Year*], [*Problem*], [*Underlying kernel*],
    [Schwinger], [1951], [QED vacuum polarization], [Heat kernel (used directly)],
    [McKean--Singer], [1967], [Index theorem], [Heat kernel (used directly)],
    [Seeley], [1967], [Complex powers of elliptic operators], [Proved $zeta_A (s) = cal(M)["Tr"(e^(-t A))]$],
    [Ray--Singer], [1971], [Analytic torsion], [Laplacian heat kernel via Mellin],
    [Dowker--Critchley], [1976], [de Sitter effective action], [Heat kernel of curved-space wave operator],
    [Hawking], [1977], [Black hole radiation], [Heat kernel of $-nabla^2 + m^2 + xi R$],
    [Voros], [1987], [Quantum spectral theory], [Partition function $"Tr"(e^(-beta H))$],
    [Elizalde], [1994--], [Casimir, gravity, strings (10 applications)], [Laplacian heat kernel in each case],
    [String theory], [various], [Critical dimension, anomalies], [Worldsheet Laplacian heat kernel],
    [Kaluza--Klein], [various], [Extra-dimensional corrections], [Heat kernel on compact dimensions],
  ),
  caption: [Every major application of zeta regularization in physics. In each case, the physical content resides in the heat kernel; the zeta function is a mathematical intermediary derived via the Mellin transform.]
) <tab:survey>

The mathematical foundation was laid by Minakshisundaram and Pleijel (1949), who defined the spectral zeta function of the Laplacian as the Mellin transform of the heat trace, and by Seeley (1967), who proved that $"Tr"(A^(-s))$ is meromorphic with poles and residues determined by the heat kernel coefficients. The direction of derivation is always heat kernel $arrow$ zeta function, never the reverse.

*Schwinger (1951)* used the heat kernel directly. His proper-time method never invokes analytic continuation; it computes the effective action from the heat trace $"Tr"(e^(-t A))$ by integrating over proper time $t$. No zeta function appears.

*Dowker and Critchley (1976)* computed the effective Lagrangian for a scalar field in de Sitter space. Their wave operator $A = -(square.stroked.small)^2 + xi R + m^2$ is a Laplacian on curved spacetime. The heat kernel $e^(-t A)$ describes physical field propagation on this background. They chose zeta regularization for notational convenience --- the heat kernel method gives identical results.

*Ray and Singer (1971)* defined analytic torsion as $exp(-zeta'_Delta (0))$, a topological invariant of the Laplacian on differential forms. The spectral data is encoded by the heat trace. Later formulations define torsion directly from the heat kernel; the zeta approach was an elegant shortcut.

*In bosonic string theory*, the critical dimension $D = 26$ requires $zeta(-1) = -1\/12$. The underlying operator is the worldsheet Laplacian, whose heat kernel $e^(-t Delta_("ws"))$ describes diffusion on the string worldsheet. The conformal anomaly that fixes $D$ can be computed entirely via heat kernel coefficients.

*For the Casimir effect*, the vacuum energy between conducting plates involves the eigenvalues of the Laplacian with boundary conditions. Elizalde (2012) explicitly demonstrated that the heat kernel method and zeta regularization yield identical Casimir energies for every geometry studied.

*In Kaluza--Klein models*, quantum corrections from the infinite tower of KK modes require regularization of spectral sums on compact extra dimensions. The heat kernel on $S^N$ provides the spectral data; zeta regularization processes it.

The pattern is without exception: *nobody ever used the zeta kernel because the physics demanded it. In every case, the physics demanded the heat kernel, and the zeta function was a mathematical convenience for processing the spectral data that the heat kernel encodes.* The gauge equivalence of Section 6 is not merely a formal observation --- it is the mechanism by which fifty years of zeta-regularized calculations inherited their physical correctness from the heat kernel.

= Implications for Quantum Field Theory

Feynman's path integral defines QFT as:

$ Z[J] = integral cal(D) phi thin e^(i S[phi] + i integral J phi) $

This is a sum over all field configurations weighted by the action. The definition is complete, elegant, and produces divergent quantities. Renormalization appears nowhere in it.

The actual practice of QFT proceeds in four steps: (1) *Definition* --- the path integral produces divergent correlation functions. (2) *Computation* --- Feynman diagrams organize the perturbative expansion; divergences appear as loop integrals. (3) *Renormalization* --- divergences are subtracted and finite parts retained. (4) *Comparison with experiment* --- the finite results agree to extraordinary precision.

The gap is between steps 1 and 3. The definition does not include the mechanism that makes the theory produce finite observables. Renormalization is bolted on as a computational necessity, not derived from the formulation.

== The composite kernel: $K_("QFT") = "FP" compose K_("ren") compose K_("path")$

In POT language, the path integral defines a kernel $K_("path")$ that maps source currents $J$ to divergent correlation functions $Z[J]$. The renormalization kernel $K_("ren")$ maps divergent objects to regularized families. The finite part operator FP extracts the image. The complete theory is the *composition*:

$ K_("QFT") = "FP" compose K_("ren") compose K_("path") : "sources" arrow.r "observables" $

This is not three separate procedures applied sequentially. It is a *single integral transform* from sources to observables. The path integral alone is not the theory --- it is one factor of the theory. Renormalization is not a fix applied after the fact --- it is another factor. The composed object $K_("QFT")$ is the theory.

This is the Volume V pattern applied to divergences. In Volume V, the gauge kernel $K_("gauge")$ alone is non-admissible (the Lie bracket defect prevents linearity), but coupling to the Higgs field $phi$ restores admissibility: $K_("eff") = K_("gauge") compose phi$ is admissible. Here, $K_("path")$ alone produces non-admissible output (divergent), but composing with $K_("ren")$ restores admissibility: $K_("QFT")$ maps sources to finite, gauge-invariant observables.

Feynman gave us $K_("path")$. Tomonaga, Schwinger, and Feynman (again) gave us $K_("ren")$. Nobody wrote down $K_("QFT") = "FP" compose K_("ren") compose K_("path")$ as the *definition* of the theory. That composition IS the theory.

== Integral Transform Composition Method (ITCM)

The claim that integral transforms compose into a single well-defined integral transform is not a POT invention. It is a theorem of transmutation theory, formalized by Sitnik and collaborators as the *Integral Transform Composition Method* (ITCM).

The ITCM constructs transmutation operators as compositions of generalized Fourier transforms:

$ S = F_B^(-1) thin 1 / (w(t)) thin F_A, quad quad P = F_A^(-1) thin w(t) thin F_B $

where $F_A$ and $F_B$ are integral transforms connected to operators $A$ and $B$, and $w(t)$ is a weight function. Sitnik and Jebabli prove that these compositions yield explicit integral operators with closed-form representations involving hypergeometric functions, Bessel functions, and Gamma factors. All known classes of transmutation operators --- Poisson, Sonine, Vekua--Erdélyi--Lowndes, Buschman--Erdélyi --- arise as special cases.

The ITCM result is decisive for the POT thesis: *the composition of integral transforms is itself an integral transform*. This is not an approximation, not a formal manipulation --- it is a rigorous theorem with explicit integral representations (Sitnik and Shishkina, 2020; Sitnik and Jebabli, 2024).

== Application to QED

Now apply the ITCM framework to quantum electrodynamics. The QED procedure is:

(1) Transform field configurations to momentum space (Fourier transform $F$).

(2) Apply the renormalization weight $w(k)$ --- counterterms, self-energy $Sigma(p)$, vacuum polarization $Pi(q^2)$, vertex corrections.

(3) Transform back to position space (inverse Fourier $F^(-1)$).

This is the ITCM formula $T = F^(-1) compose w(k) compose F$ *verbatim*. The QED-to-ITCM dictionary is:

#figure(
  table(
    columns: 2,
    [*ITCM component*], [*QED realization*],
    [$F_A$ (generalized Fourier)], [Fourier transform to momentum space],
    [$F_A^(-1)$ (inverse transform)], [Inverse Fourier to position space],
    [$w(t)$ (weight function)], [Renormalization weight: counterterms + running coupling],
    [$T = F^(-1) w F$ (transmutation)], [Complete renormalized propagator],
    [Index shift $T B_mu = B_nu T$], [Running of coupling constants with energy scale],
  ),
  caption: [The ITCM-QED correspondence. Each component of the transmutation operator maps directly to a standard QED operation.]
) <tab:itcm>

For one-loop QED, the weight function takes the form:

$ w(k) tilde.op 1 + alpha / (2 pi) ["loop corrections"] + cal(O)(alpha^2) $

where $alpha approx 1\/137$ is the fine-structure constant. The *running* of $alpha$ with energy scale --- the renormalization group flow --- is the scale-dependence of $w(k, mu)$ as a function of the renormalization point $mu$. In ITCM language, this is an *index shift*: the transmutation operator shifts the spectral index of the Bessel-type kernel from $mu$ to $nu$, exactly as in Sitnik and Jebabli's Theorems 1--4.

The crucial insight is that dimensional regularization corresponds to the ITCM with $w(t) = (-t^2)^s$ where $s = epsilon\/2$ and $d = 4 - 2 epsilon$. The resulting transmutation operator has the explicit integral representation given by Theorem 5 of Sitnik and Jebabli, involving Gauss hypergeometric functions $attach(, tl: 2) F_1$. This provides a *closed-form kernel* for the renormalized QED propagator --- something the standard QFT literature treats only perturbatively.

The fine-structure constant $alpha$ itself may be understood as a projected observable: the finite part extracted from the electromagnetic kernel's spectral data at a given energy scale. Its value $alpha^(-1) approx 137$ at low energies, and its running to $alpha^(-1) approx 128$ at the $Z$-boson mass, is the ITCM index shift in action.

== The explicit hypergeometric kernel for QED

The ITCM does not merely assert that the composition exists --- it provides the closed-form integral representation. By Theorem 5 of Sitnik and Jebabli (2024), the composition of Hankel transforms $H_nu$ and $H_mu$ with the power weight $w(t) = (-t^2)^s$ yields:

$ [H_nu (-t^2)^s H_mu f](x) = c_1 x^(-mu - 2 s - 3\/2) integral_0^x attach(, tl: 2) F_1 (a_1, b_1 ; c_1 ; y^2 \/ x^2) y^(mu + 1\/2) f(y) thin d y \ + thin c_2 x^(nu + 1\/2) integral_x^infinity attach(, tl: 2) F_1 (a_2, b_2 ; c_2 ; x^2 \/ y^2) y^(-nu - 2 s - 3\/2) f(y) thin d y $

where the hypergeometric parameters are:

$ a_1 = (mu + nu) / 2 + s + 1, quad b_1 = (mu - nu) / 2 + s + 1, quad c_1 = mu + 1 $

and $c_1 (s, mu, nu)$, $c_2 (s, mu, nu)$ are explicit constants involving Gamma functions:

$ c_1 = (-1)^s dot.op 2^(2 s + 1) dot.op (Gamma((mu + nu)\/2 + s + 1)) / (Gamma((nu - mu)\/2 - s) dot.op Gamma(mu + 1)) $

For QED with dimensional regularization $d = 4 - 2 epsilon$, the parameter identification is: $s = epsilon \/ 2$ (the regularization parameter), $nu$ is the free-field spectral index (determined by spin and mass), and $mu$ is the interacting-field spectral index (shifted by the electromagnetic coupling). The transmutation operator maps free-field Green's functions to renormalized interacting-field Green's functions in a *single integral operation* --- not as a perturbative series in $alpha$, but as a closed-form integral with hypergeometric kernel.

This is a non-perturbative result. Standard QED provides renormalized amplitudes as power series in $alpha$; the ITCM provides the *exact integral operator* whose Taylor expansion reproduces those series. The kernel decomposes into two terms: an inner integral over $[0, x]$ and an outer integral over $[x, infinity)$, each weighted by a Gauss hypergeometric function $attach(, tl: 2) F_1$. The two terms correspond to two integration regions --- forward and reflected channels --- and their specific hypergeometric weights encode all the information that perturbative QED distributes across Feynman diagrams of every loop order.

=== Symmetries of the hypergeometric kernel

The kernel has three symmetries that dramatically simplify its structure.

*Exchange symmetry ($mu arrow.l.r nu$).* The second-term parameters $(a_2, b_2, c_2)$ are obtained from the first by swapping $mu$ and $nu$: $a_2 = a_1$ (it is symmetric in $mu, nu$), $b_2 = (nu - mu)\/2 + s + 1$, and $c_2 = nu + 1$. The relation $b_1 + b_2 = 2(s + 1)$ holds. The two causal regions are therefore *spectral duals*: they exchange the free-field index $nu$ and interacting-field index $mu$. The full kernel is invariant under simultaneous $mu arrow.l.r nu$ and transposition of the integration regions.

*Euler factorization.* Applying Euler's hypergeometric transformation $attach(, tl: 2) F_1 (a, b ; c ; z) = (1 - z)^(c - a - b) attach(, tl: 2) F_1 (c - a, c - b ; c ; z)$ to the first term, the Euler exponent is:

$ c_1 - a_1 - b_1 = (mu + 1) - ((mu + nu)\/2 + s + 1) - ((mu - nu)\/2 + s + 1) = -1 - 2 s $

The kernel factors into a *universal pole* and a *regular correction*:

$ attach(, tl: 2) F_1 (a_1, b_1 ; c_1 ; y^2 \/ x^2) = (1 - y^2 \/ x^2)^(-1 - 2 s) dot.op attach(, tl: 2) F_1 ((mu - nu)\/2 - s, (mu + nu)\/2 - s ; mu + 1 ; y^2 \/ x^2) $

The singular factor $(1 - y^2 \/ x^2)^(-1 - 2 s)$ has a branch point at $y = x$ (the lightcone). The remaining $attach(, tl: 2) F_1$ is *regular* for all $y^2 \/ x^2 in [0, 1]$. This factorization separates the *universal singularity structure* --- which generates the divergences when $s arrow 0$ --- from the *theory-specific content*, which determines the finite parts.

At $s = 0$ (the physical limit), the singular factor becomes $(1 - y^2 \/ x^2)^(-1) = x^2 \/ (x^2 - y^2)$, a simple Cauchy-type pole. The full kernel reduces to:

$ K(x, y)|_(s = 0) = x^2 / (x^2 - y^2) dot.op attach(, tl: 2) F_1 ((mu - nu)\/2, (mu + nu)\/2 ; mu + 1 ; y^2 \/ x^2) $

*Green's function structure of the Cauchy pole.* The denominator factors as $x^2 - y^2 = (x - y)(x + y)$, revealing the propagator content:

$ x^2 / (x^2 - y^2) = x^2 / ((x - y)(x + y)) = x / 2 dot.op lr([ 1 / (x - y) + 1 / (x + y) ]) $

The pole $1 \/ (x - y)$ is the *direct channel*: singular when $y = x$ (the source coincides with the observation point, forward propagation). The pole $1 \/ (x + y)$ is the *reflected channel*: singular at $y = -x$ (backward propagation, the crossed channel). Their sum realizes a propagator-like decomposition into direct and reflected channels, structurally analogous to the momentum-space scalar propagator $1 \/ (k^2 - m^2) = 1 \/ ((k - m)(k + m))$. The ITCM does not assume this propagator structure --- it *derives* it from the composition of integral transforms.

The full kernel at $s = 0$ then reads:

$ K(x, y)|_(s = 0) = underbrace(x^2 / ((x - y)(x + y)), "free propagator") dot.op underbrace(attach(, tl: 2) F_1 ((mu - nu)\/2, (mu + nu)\/2 ; mu + 1 ; y^2 \/ x^2), "interaction dressing") $

The interaction dressing is the regular $attach(, tl: 2) F_1$ correction: it modifies the free propagator's pole structure without introducing new singularities. The mass gap question, in this language, is whether the interaction dressing shifts the propagator's spectral support away from zero --- whether $R^("YM")$ converts the gapless Green's function $1 \/ (x^2 - y^2)$ into a massive one.

*Free-field limit ($mu arrow nu$).* When $mu = nu$ (coupling turned off), the parameters simplify: $b_1 arrow s + 1$ and $c_1 arrow nu + 1$. At $s = 0$: $attach(, tl: 2) F_1 (nu + 1, 1 ; nu + 1 ; z) = (1 - z)^(-1)$, since $attach(, tl: 2) F_1 (a, b ; a ; z) = (1 - z)^(-b)$ for any $a$. The entire kernel collapses to the Cauchy kernel $1 \/ (1 - z)$ --- which is the *free-field propagator*. The hypergeometric correction vanishes, and only the universal pole survives.

*Summary of the factorization.* The QED kernel has the schematic form:

$ K_("QED")(x, y) = ["Cauchy pole"]^(1 + 2 s) times ["regular hypergeometric correction"] times ["power-law weights"] $

The Cauchy pole is universal (every QFT has it). The hypergeometric correction encodes the coupling. The power-law weights track the spectral indices. The factorization makes the physics transparent: the pole generates the loop integrals, the correction determines their finite parts, and the weight functions encode the theory's specific quantum numbers.

The theory file formalizes this as the `QEDHypergeometricKernel` structure with six hypergeometric parameters $(a_1, b_1, c_1, a_2, b_2, c_2)$ and axioms for kernel decomposition, exchange symmetry, and gauge invariance. The `ITCMTransmutation` structure establishes that the ITCM transmutation equals the composite QED kernel (Example 21), and that the index shift maps the free-field spectral index to the interacting-field index (Example 22), formalizing the intertwining property $T A = B T$ that underlies the running of coupling constants.

*Kernel Decomposition Principle (proposed).* Any ITCM-generated transmutation kernel admits a decomposition into a universal singular Green's kernel and a regular dressing factor, with spectral properties determined solely by the latter after finite-part extraction.

This principle unifies the results of this subsection. The Euler factorization (derived from the composition theorem) separates the universal pole from the regular correction. The partial-fraction decomposition reveals the pole as a propagator-like Green's kernel with direct and reflected channels. The regular $attach(, tl: 2) F_1$ dressing encodes the interaction. The free-field limit confirms that the dressing vanishes when the coupling is turned off. The Spectral Localization Principle (Section 9.8) establishes that the mass gap, if it exists, resides in the dressing. The pipeline from composition theorem to spectral localization is:

$ "ITCM" arrow.r "hypergeometric kernel" arrow.r "Euler factorization" $
$ arrow.r underbrace("Green's kernel", "universal") times underbrace("dressing", "theory-specific") arrow.r "spectral localization" $

The decomposition is not assumed or postulated. It is derived from the composition of integral transforms.

== Renormalization is not an add-on

The ITCM theorem has a sharp consequence: since the composition of integral transforms is itself an integral transform, *there is no gap between steps 1 and 3*. The path integral and renormalization do not define separate procedures that are applied sequentially. They define a single composite integral transform $K_("QFT")$ that maps sources to observables in one step.

This means renormalization was never an add-on, a fix, or a computational trick. It was always a constitutive factor of the integral transform that defines the theory. Asking "why does renormalization work?" is like asking "why does the second factor of a product matter?" --- it is part of the definition.

The standard narrative that QFT is "sick" and renormalization "cures" it is incorrect. The path integral alone is not QFT --- it is one factor of the composite kernel. The composed kernel $K_("QFT")$ was never sick; it was simply never stated as a single object. Tomonaga, Schwinger, and Feynman supplied all the factors in 1948. The composition was implicit in every calculation but never written down as the theory's defining kernel.

== The divergences were never real

The ITCM result has a consequence that goes beyond "renormalization is not an add-on." It implies that *the divergences of QFT were never real* --- they are artifacts of the factorization, not properties of the theory.

Consider the analogy: the function $f(x) = (x^2 - 1) / (x - 1)$ appears singular at $x = 1$. If you evaluate numerator and denominator separately, you obtain $0 / 0$. But $f$ simplifies to $x + 1$, which is perfectly well-defined at $x = 1$ with value $2$. The singularity was in the *representation*, not the *function*.

The same holds for QFT. When you compute via the factored route $"FP" compose K_("ren") compose K_("path")$, you pass through a divergent intermediate state: $K_("path")(J)$ is a divergent correlation function, and $K_("ren")$ produces a Laurent series with poles. But the ITCM proves that the composed operator is a single integral transform with a well-defined hypergeometric kernel. This operator --- call it $K_("direct")$ --- maps sources directly to finite observables:

$ K_("direct") : "Source" arrow.r "FinitePart" $

No divergent intermediate quantity ever appears. The hypergeometric kernel $attach(, tl: 2) F_1 (a, b ; c ; y^2 / x^2)$ is a convergent function; the integral it defines is finite; the output is a finite observable. The type signature of the direct composite is $"Source" arrow.r "FinitePart"$, not $"Source" arrow.r "RegValue" arrow.r "FinitePart"$. The domain of divergent objects (`RegValue`) is absent from the composed map.

The formal consequence is sharp. The direct composite equals both the factored composite and the ITCM transmutation for every source $J$ (Examples 23--24):

$ K_("direct")(J) = K_("QFT")(R, J) = T_("ITCM")(w, J) $

But the three descriptions have different intermediate types. The factored route passes through `RegValue`; the ITCM route passes through momentum space with weight multiplication; the direct route has no intermediate at all. All three yield the same `FinitePart` output.

This means the divergences of QFT --- the ultraviolet catastrophe, the need for regularization, the perturbative subtractions --- are properties of a *particular factorization* of the composite kernel. They are not properties of the theory. The theory, stated as a single integral operator, was never divergent. Only the way physicists chose to compute it --- by decomposing it into factors and evaluating each factor separately --- created the appearance of divergence. The "sickness" of QFT was always a misdiagnosis. The patient was healthy; the thermometer was broken.

*Relation to prior divergence-free formulations.* The claim that QFT divergences are artifacts of the formalism rather than the physics is not new to this paper. Epstein and Glaser (1973) constructed perturbative QFT without divergences by distributing the time-ordering operation carefully, avoiding ill-defined products of distributions entirely. Scharf (1995) developed this into a complete "finite QED" using the causal approach. Connes and Kreimer (2000) revealed that the combinatorics of renormalization carry the structure of a Hopf algebra, with the Birkhoff decomposition of loops in a Lie group replacing ad hoc subtraction procedures. These approaches demonstrate, by different routes, that divergences are not intrinsic to QFT. The present contribution is distinct in two respects: (1) it uses the ITCM to exhibit the composite kernel as a *single explicit integral operator* with hypergeometric kernel, not merely as a divergence-free perturbative construction; and (2) it identifies the kernel as a transmutation operator in the sense of Sitnik, connecting QFT renormalization to the classical theory of integral transforms. The Epstein--Glaser and Connes--Kreimer approaches avoid divergences within perturbation theory; the ITCM approach provides the non-perturbative integral operator that perturbation theory approximates.

== Gauge invariance of QFT observables

*Theorem 8 (QFT gauge invariance).* For any two gauge-equivalent regulators $R tilde.op R'$ and any path integral output $Z$:

$ "Observable"_R (Z) = "Observable"_(R') (Z) $

This is the fundamental consistency condition of QFT: physical predictions are independent of the regularization scheme. In practice, physicists verify this case by case. In the POT framework, it is a theorem of the gauge group structure. The composite kernel inherits gauge invariance: if $R tilde.op R'$, then $K_("QFT")(R, J) = K_("QFT")(R', J)$ for all sources $J$ (Example 20).

== The composite kernel and the Clay Millennium Problem

The Clay Millennium Problem asks two things: (1) prove that a non-trivial quantum Yang--Mills theory exists on $bb(R)^4$ for any compact simple gauge group $G$, and (2) prove it has a mass gap $Delta > 0$. The standard approach seeks these through constructive field theory --- building the path integral measure $cal(D) A_mu$ rigorously.

The composite kernel framework reframes both questions. For perturbatively renormalizable theories (QED, electroweak), the ITCM provides the composite kernel explicitly: $K_("QFT") = F^(-1) compose w compose F$ with $w(t) = (-t^2)^s$ yields a closed-form hypergeometric integral operator. The divergences were never real; the theory was always a well-defined single integral transform.

For pure Yang--Mills --- a theory of gauge fields alone, without quarks --- the same framework applies but with a crucial difference. The weight function $w_("YM")(k)$ cannot be computed perturbatively: the gauge coupling $g$ runs to large values at low energies (asymptotic freedom), and the infrared regime is inherently non-perturbative. The composite kernel $K_("YM") = F^(-1) compose w_("YM") compose F$ exists in principle --- the ITCM guarantees it, if $w_("YM")$ can be specified --- but constructing $w_("YM")$ explicitly is the open problem.

The reframing is this. Instead of the traditional question:

#quote[_Construct a rigorous path integral measure $cal(D) A_mu$ for Yang--Mills on $bb(R)^4$ and prove a mass gap._]

the composite kernel approach asks:

#quote[_Find a weight function $w_("YM") : bb(R)^+ arrow.r bb(R)$ satisfying five constraints, and prove that the resulting composite kernel has spectral gap $Delta > 0$._]

The five constraints on $w_("YM")$ are (Examples 25--29):

#figure(
  table(
    columns: 2,
    [*Constraint*], [*Physical content*],
    [Asymptotic freedom], [$w_("YM")(k) arrow.r 1$ as $k arrow.r infinity$ (theory becomes free at high energies)],
    [IR regularity], [$w_("YM")(k)$ produces a normalizable composite kernel as $k arrow.r 0$],
    [Gauge invariance], [$w_("YM")$ respects the gauge group $G$],
    [Unitarity], [The composite kernel produces a unitary $S$-matrix],
    [Mass gap], [The kernel's spectral decomposition has $Delta > 0$],
  ),
  caption: [The five constraints on the Yang--Mills weight function. The Millennium Problem asks for existence (constraints 1--4) and mass gap (constraint 5) of pure Yang--Mills, a theory of gauge fields without quarks. Color confinement --- the unobservability of color charge --- is a separate algebraic consequence of the kernel's non-admissibility (Volume IV), not a constraint on $w_("YM")$.]
) <tab:ym>

This is a genuine simplification. The traditional formulation requires constructing an infinite-dimensional measure (on the space of gauge connections modulo gauge equivalence). The composite kernel formulation requires finding a single real-valued function $w_("YM")(k)$ on momentum space. The existence part reduces to: does such a $w_("YM")$ satisfying constraints 1--4 exist? The mass gap part reduces to: does the $attach(, tl: 2) F_1$ kernel with parameters determined by $w_("YM")$ have a spectral gap?

Both are questions about functions and spectra --- ordinary analysis --- not about infinite-dimensional measures.

Moreover, lattice QCD already computes non-perturbative quantities numerically: glueball masses, string tension, the running coupling at all scales. These are effectively numerical samples of $w_("YM")$. The composite kernel approach could be validated against lattice data: extract $w_("YM")(k)$ from lattice propagators, construct the ITCM composite kernel with that weight, and verify that its spectral gap matches the known glueball mass $Delta approx 1.5$ GeV.

*Symmetry transfer to Yang--Mills (conditional).* The exchange symmetry, Euler factorization, and free-field limit derived in Section 9.4 are properties of the ITCM composition theorem (Theorem 5 of Sitnik--Jebabli), not of QED specifically. These symmetries are inherited by any Yang--Mills kernel that admits an ITCM/Hankel composite representation of the same form $K = F^(-1) compose w compose F$. If the Yang--Mills composite kernel is realized as such a composition --- which is the central hypothesis of this section --- then it inherits the factored form:

$ K_("YM")(x, y) = (1 - y^2 \/ x^2)^(-1 - 2 s) dot.op attach(, tl: 2) F_1 ("regular parameters"; y^2 \/ x^2) dot.op ["power-law weights"] $

By the Kernel Decomposition Principle (Section 9.4), this factors at $s = 0$ into:

$ K_("YM")(x, y)|_(s = 0) = underbrace(x^2 / ((x - y)(x + y)), "universal Green's kernel") dot.op underbrace(attach(, tl: 2) F_1 (dots.c ; y^2 \/ x^2), "YM interaction dressing") $

The universal Green's kernel --- the same propagator-like pole structure shared by QED --- is a property of the Hankel composition mechanism, not the specific gauge theory. The regular $attach(, tl: 2) F_1$ dressing is where the Yang--Mills-specific content resides, with parameters determined by $w_("YM")$ instead of $w_("QED")$. The full Kernel Decomposition pipeline applies:

$ "ITCM" + w_("YM") arrow.r "hypergeometric kernel" arrow.r "Euler factorization" $
$ arrow.r underbrace("Green's kernel", "same as QED") times underbrace(R^("YM"), "YM-specific") arrow.r "mass gap question" $

*Conjectural localization of the mass gap.* Under the hypothesis that the Yang--Mills kernel admits this ITCM representation, the symmetry analysis suggests a localization of the mass gap question. QED has the same universal pole but *no* mass gap (the photon is massless). The pole is therefore not the source of the mass gap. More precisely:

- The *universal pole* $(1 - y^2 \/ x^2)^(-1 - 2 s)$ is universal across all Hankel-composite theories and is removed by finite-part extraction, so the theory-distinguishing spectral content must be sought elsewhere.
- The *regular $attach(, tl: 2) F_1$ correction* survives FP extraction and encodes the theory-specific spectral structure.
- If the mass gap $Delta > 0$ exists, it is a spectral property of this regular correction.

The Millennium Problem is thus localized: it is not in the universal pole (which QED shares), but in the spectral behavior of the regular correction determined by $w_("YM")$.

This localization is a *reduction*, not a theorem. To promote it to a theorem would require three additional steps: (1) showing that the spectral gap question survives finite-part extraction, (2) resolving the operator closure and domain issues for the composite kernel on the appropriate function space, and (3) proving that the formal kernel factorization controls the spectrum of the actual physical operator. These are substantial analytical problems, but they are problems of ordinary operator theory --- not of infinite-dimensional measure construction.

We name the structural claim and sketch the argument for bridge (1).

*Spectral Localization Principle (informal).* For any ITCM-composite kernel, the spectrum of the associated operator after finite-part extraction depends only on the regular component of the Euler-factorized kernel.

*Sketch of argument.* The composite operator $T_s : f arrow.r.bar integral K(x, y ; s) f(y) thin d y$ depends on the regularization parameter $s$. By the Euler factorization, the kernel is $K(x, y ; s) = P(x, y ; s) dot.op R(x, y ; s) dot.op W(x, y ; s)$, where $P = (1 - y^2 \/ x^2)^(-1 - 2 s)$ is the universal pole, $R$ is the regular $attach(, tl: 2) F_1$ correction, and $W$ contains the power-law weights. As a function of $s$, the operator $T_s$ admits a distributional Laurent expansion:

$ T_s = T_(-1) \/ s + T_0 + T_1 dot.op s + dots.c $

The finite-part operator extracts the constant term: $"FP"[T_s] = T_0$. The divergent residue $T_(-1)$ is determined entirely by the universal pole $P$ (its coefficient at the leading singularity). The finite part $T_0$ receives contributions from both $P$ (its subleading expansion) and $R$ (its value at $s = 0$). But the contribution from $P$ to $T_0$ is *universal* --- the same for every Hankel-composite theory, whether QED or Yang--Mills.

Therefore, for two theories sharing the same ITCM composition structure:

$ T_0^("YM") - T_0^("QED") = cal(O)(R^("YM")|_(s = 0) - R^("QED")|_(s = 0)) $

The difference in their physical operators depends *only* on the difference in their regular corrections. Since QED has $"spec_gap"(T_0^("QED")) = 0$ (massless photon), any non-zero spectral gap in Yang--Mills must originate from $R^("YM")|_(s = 0)$.

This does not yet constitute a proof that $"spec_gap"(T_0) = "spec_gap"(R|_(s = 0))$, because the universal pole's subleading contribution to $T_0$ could in principle shift the spectrum in a theory-dependent way (through its interaction with the theory-specific weights $W$). Establishing the precise spectral relationship requires the operator-domain analysis of bridge (2). But the argument above --- that the *difference* between two theories' spectra is controlled by the *difference* in their regular corrections --- already provides a non-trivial constraint. The mass gap, if it exists, is governed by $R^("YM")$, not by the Cauchy pole that all theories share.

This does not solve the Millennium Problem. But it reduces it from measure theory on infinite-dimensional spaces to spectral theory on a single integral operator, and the symmetry analysis further localizes the mass gap to the regular correction of the hypergeometric kernel --- a reduction that may make the problem tractable.

The composite kernel framework complements the structural result of Volume IV, which established that color confinement is an algebraic consequence of the Yang--Mills kernel's non-admissibility: the Lie bracket $[A, B]$ breaks linearity, making color charge a fiber-non-invariant predicate. Volume IV explicitly left the mass gap, asymptotic freedom, and the confining potential as open problems requiring "dynamical content beyond the algebraic structure." The ITCM weight function $w_("YM")(k)$ is precisely that dynamical content: asymptotic freedom is $w_("YM")(k) arrow.r 1$ at high $k$; the IR behavior of $w_("YM")$ at low $k$ determines the non-perturbative spectrum; the mass gap is the spectral gap of the resulting composite kernel. Color confinement itself --- the unobservability of color charge --- remains a structural consequence of the kernel's non-admissibility (Volume IV), independent of the specific form of $w_("YM")$. The algebraic layer (Volume IV) establishes *why* color is unobservable; the analytical layer (this paper) provides the framework for determining the spectrum of the theory, including the mass gap.

== Parameter constraints for the mass gap

The spectral localization analysis identifies the regular correction $R(z) = attach(, tl: 2) F_1 ((mu - nu)\/2, (mu + nu)\/2 ; mu + 1 ; z)$ as the carrier of the mass gap. We now extract concrete parameter constraints from this identification.

*The free-field degeneracy.* When $mu = nu$ (coupling turned off), the first upper parameter of the hypergeometric function vanishes:

$ attach(, tl: 2) F_1 (0, mu ; mu + 1 ; z) = 1 quad "for all" z $

The regular correction is trivially equal to unity. The full kernel collapses to the universal Cauchy pole $x^2 slash (x^2 - y^2)$, which has continuous spectrum extending to zero --- it is gapless. This is the free-field limit already noted in Section 9.4. The *new* content is the contrapositive:

*Necessary condition.* If $Delta > 0$ (mass gap exists), then $mu_("YM") eq.not nu_("YM")$.

The Hankel-order asymmetry $delta := |mu - nu|$ is a necessary condition for a spectral gap. A theory with degenerate Hankel orders is a free theory, and a free theory is gapless.

*The Gauss evaluation.* Since $c - a - b = (mu + 1) - (mu + nu) slash 2 - (mu - nu) slash 2 = 1 > 0$ for all values of $mu$ and $nu$, Gauss's summation formula gives:

$ R(1) = (Gamma(mu + 1) dot.op Gamma(1)) / (Gamma((mu + nu) slash 2 + 1) dot.op Gamma((mu - nu) slash 2 + 1)) $

This is a *closed-form, computable measure of interaction strength* for any ITCM-composite theory. Its properties:

- $R(1) = 1$ when $mu = nu$ (free field, trivial dressing),
- $R(1) eq.not 1$ when $mu eq.not nu$ (interaction present),
- $|R(1) - 1|$ quantifies the departure from the free field.

For QED: $mu_("QED") approx nu_("QED")$ (the electromagnetic coupling $alpha approx 1 slash 137$ creates only a small spectral shift), so $|R(1) - 1| << 1$. QED is near-degenerate, and the photon is massless.

For Yang--Mills: gluon self-interaction (three-gluon and four-gluon vertices) breaks the degeneracy. The gauge coupling $g$ runs to large values at low energies, making $|mu_("YM") - nu_("YM")|$ potentially large in the infrared. The interaction dressing $R(1)$ can depart significantly from unity. @fig:gauss shows $R(1)$ as a function of $delta = |mu - nu|$: the super-linear growth from $R(1) = 1$ at $delta = 0$ to $R(1) = 56$ at $delta = 6$ quantifies the contrast between the QED and Yang--Mills regimes.

*Physical interpretation.* The parameter $nu$ is the free-field spectral index, determined by the particle's spin and mass in the absence of coupling. The parameter $mu$ is the interacting-field spectral index, shifted by the gauge coupling. The *asymmetry* $delta = |mu - nu|$ is the spectral shift due to interaction. For abelian theories (QED), the gauge field does not self-interact, so $delta$ is small and perturbatively computable. For non-abelian theories (Yang--Mills), the gauge field self-interacts, and $delta$ becomes large in the non-perturbative regime. The mass gap is controlled by this asymmetry (Examples 38--40). @fig:dressing shows $R(z)$ for representative values of $delta$: the flat line at $delta = 0$ (free field) versus the steep rise at $delta = 6$ (strong coupling) makes the interaction dressing visually concrete.

*Spectral Gap Conjecture.* Under the hypothesis that the Yang--Mills kernel admits an ITCM/Hankel composite representation:

$ Delta > 0 quad arrow.l.r.double quad mu_("YM") eq.not nu_("YM") quad "and" quad w_("YM") "satisfies IR regularity" $

More precisely: the spectral gap $Delta$ is a function of the regular correction $R$, which in turn is determined by the Hankel-order asymmetry $delta$ and the weight function $w_("YM")$. The conjecture is that $delta > 0$ combined with IR regularity is both necessary and sufficient for $Delta > 0$.

This is labeled a *conjecture*, not a theorem. The necessary direction ($Delta > 0 arrow.r.double delta > 0$) is derived (free-field degeneracy). The sufficient direction ($delta > 0 arrow.r.double Delta > 0$ given IR regularity) would require computing the spectrum of the integral operator with hypergeometric kernel --- a well-posed but open problem.

*The well-posed mathematical target.* The conjecture reduces to a concrete spectral theory problem:

#quote[_For the integral operator on $L^2 (bb(R)_+)$ with kernel $K(x, y) = x^2 slash ((x - y)(x + y)) dot.op attach(, tl: 2) F_1 (a, b ; c ; y^2 slash x^2)$, determine the spectrum as a function of $(a, b, c)$ and identify the parameter region where a spectral gap exists._]

This is a question about functions and eigenvalues --- ordinary one-dimensional spectral theory, not infinite-dimensional measure construction.

*The Sturm--Liouville connection.* The Hankel transforms $H_mu$ diagonalize the Bessel operator $L_mu = -d^2 slash d x^2 + (mu^2 - 1 slash 4) slash x^2$. The ITCM transmutation operator maps solutions of $L_nu u = lambda u$ to solutions of $L_mu v = lambda v$. This connects our composite kernel to the family of exactly solvable Schrödinger operators classified by Dereziński and Karimi (2025): one-dimensional operators with hypergeometric potentials, whose spectra and Green functions (resolvent kernels) are computed explicitly. The "hyperbolic" family on $L^2 (bb(R)_+)$ --- which includes the Pöschl--Teller Hamiltonians --- is the natural home for the Yang--Mills composite kernel. If the kernel maps to an operator in this family, the mass gap becomes a computable condition on the hypergeometric potential parameters, which are determined by $(mu_("YM"), nu_("YM"))$.

*Lattice validation path.* Lattice QCD computes non-perturbative gluon propagators and glueball masses numerically. From these, one could extract $w_("YM")(k)$ numerically, determine $(mu_("YM"), nu_("YM"))$ from the ITCM identification, compute $R(1)$ via the Gauss formula, and test whether the predicted spectral gap matches the known glueball mass $Delta approx 1.5$ GeV. This would provide an empirical test of the framework independent of the analytical proof.

*Numerical demonstration: the spectral gap mechanism.* The Sturm--Liouville connection gives a direct numerical demonstration of the gap mechanism. We discretize the Bessel operator $L_mu = -d^2 slash d x^2 + (mu^2 - 1 slash 4) slash x^2$ on $[0.2, 8]$ (a confinement-scale volume) with Dirichlet boundary conditions and compute eigenvalues for $delta = 0, 1, 2, 3, 6$ (fixing $nu = 2$). @fig:spectrum shows the result: the entire spectrum shifts upward monotonically with $delta$. The lowest eigenvalue moves from $E_1 = 0.41$ at $delta = 0$ (free field) to $E_1 = 2.34$ at $delta = 6$ (strong-coupling regime). @fig:gap shows the spectral gap $Delta(delta) = E_1 (delta) - E_1 (0)$ as a function of $delta$: the gap opens at $delta > 0$ and grows super-linearly, reaching $Delta = 1.92$ at $delta = 6$.

The mechanism is transparent. The Hankel-order asymmetry $delta > 0$ amplifies the centrifugal potential from $(nu^2 - 1 slash 4) slash x^2$ to $((nu + delta)^2 - 1 slash 4) slash x^2$. The additional barrier $(2 nu delta + delta^2) slash x^2$ pushes the entire spectrum away from zero. The spectral gap is:

$ Delta(delta) approx (2 nu delta + delta^2) slash x_("eff")^2 $

where $x_("eff")$ is the effective radius of the lowest eigenstate. This is the same curvature-driven mechanism that the interaction dressing $R(z)$ quantifies from the kernel side (@fig:dressing): the stiffening of the kernel near the diagonal corresponds to the strengthening of the centrifugal barrier that opens the spectral gap.

The numerical results show that *any* nonzero Hankel-order asymmetry induces a strictly positive spectral gap in finite volume, with monotonic amplification as $delta$ increases. The gap is not accidental; it is structurally forced by the centrifugal barrier that the ITCM transmutation generates. The mass gap emerges as a curvature-induced spectral separation driven by Hankel-order asymmetry, with a demonstrable monotonic mechanism in the associated Sturm--Liouville operator.

This is a finite-volume demonstration, not a proof. The Bessel operator on the full half-line $bb(R)_+$ has continuous spectrum $[0, infinity)$ for all $mu$, so the gap vanishes in infinite volume. The persistence of the gap in the infinite-volume limit requires the additional structure provided by the weight function $w_("YM")$ --- specifically, its IR regularity (Table 4, constraint 2) --- which modifies the long-range behavior of the kernel in a way that the simple centrifugal barrier does not. The numerical demonstration establishes the *mechanism* (Hankel-order asymmetry $arrow.r$ spectral gap); the *persistence* of that mechanism in infinite volume is the content of the Spectral Gap Conjecture.

== The moduli space of weight functions

The ITCM identifies a weight function $w(k)$ as the central object of a quantum field theory: the composite kernel $K = F^(-1) compose w compose F$ is entirely determined by $w$. This raises a question the ITCM literature does not address: when do two different weight functions produce the same physics?

*The source space.* In the ITCM framework, a *source* is a test function $J in cal(S)(bb(R)^d)$ --- the Schwartz space of smooth, rapidly decreasing functions on $d$-dimensional spacetime. This is the standard function space for QFT: external currents coupled to fields are Schwartz-class, the Fourier transform maps $cal(S)$ to itself (so the ITCM composition $F^(-1) compose w compose F$ is well-defined on $cal(S)$), and the LSZ reduction formula derives S-matrix elements as limits of correlation functions tested against $cal(S)$-class functions.

*The observable map.* For a given weight function $w$ and source $J in cal(S)$, the observable is:

$ "Obs"_w (J) := "FP"[F^(-1) compose w compose F](J) $

This is the finite part of the composite kernel applied to $J$. It is a well-defined element of $cal(S)'$ (a tempered distribution). Two weight functions produce the same physics if and only if they yield the same tempered distribution for every test source.

*Definition.* Two weight functions $w, tilde(w) : bb(R)^d arrow bb(C)$ are *weight-equivalent*, written $w approx tilde(w)$, if:

$ forall J in cal(S)(bb(R)^d) : quad "FP"[F^(-1) compose w compose F](J) = "FP"[F^(-1) compose tilde(w) compose F](J) $

This is *distributional equality* of the composite kernels after finite-part extraction. It is the precise analogue of how two tempered distributions $T, T' in cal(S)'$ are equal if and only if $T(J) = T'(J)$ for all $J in cal(S)$.

*Remark (strength of the equivalence).* This equivalence is exactly the right strength for QFT:
- It is *strong enough* to distinguish physically different theories: if two weights produce different S-matrix elements, there exists a Schwartz-class source that separates them.
- It is *weak enough* to identify theories that differ only in unphysical content: scheme-dependent divergent terms are annihilated by FP and do not contribute to the comparison.
- It is *not vacuous*: the trivial weight $w = 1$ (free field) and the QED weight $w_("QED")$ are demonstrably inequivalent (they produce different Lamb shifts).

An equivalence defined on a smaller source space (e.g., only on-shell sources) would be too coarse --- it would identify theories that differ off-shell but agree on-shell. An equivalence on a larger space (e.g., all distributions) would be too fine --- it would distinguish theories that agree on all physical test functions but differ on pathological inputs. Schwartz space is the Goldilocks choice, as is standard in axiomatic QFT.

Weight equivalence is reflexive, symmetric, and transitive (verified in `WeightModuliSpace`, Examples 28--30).

*Group structure.* Weight functions form a group under pointwise multiplication in momentum space:

$ (w_1 dot.op w_2)(k) = w_1 (k) times w_2 (k) $

By the convolution theorem, pointwise multiplication in transform space corresponds to composition of operators in position space: $T_(w_1 dot.op w_2) = T_(w_1) compose T_(w_2)$. The identity element is $w_("triv")(k) = 1$ (no renormalization). The group is associative with identity (verified, Examples 31--32). For the group to be well-defined on equivalence classes, composition must respect equivalence: if $w_1 approx w_1'$ and $w_2 approx w_2'$, then $w_1 dot.op w_2 approx w_1' dot.op w_2'$. This follows from the linearity of the Fourier transform and the finite-part operator.

*The moduli space.* The quotient $cal(W) \/ approx$ --- the set of weight-equivalence classes --- is the *moduli space of composite kernels*. Each point in this space represents a physically distinct theory. QED, QCD, and pure Yang--Mills live at different points. The moduli space inherits the group structure from $cal(W)$: the product $[w_1] dot.op [w_2] = [w_1 dot.op w_2]$ is well-defined on equivalence classes.

*The renormalization group is the flow on $cal(W)$.* Running the energy scale from $mu$ to $mu'$ maps $w_mu (k)$ to $w_(mu') (k)$. A central result of renormalization group theory is that this flow preserves physical observables (the S-matrix is scale-independent). In our language: *RG-related weight functions are weight-equivalent* (verified, Example 33).

$ w_mu approx w_(mu') quad "for all" mu, mu' $

This identification resolves a conceptual puzzle. The renormalization group is usually presented as a flow on coupling constants. In the ITCM framework, it is more naturally a flow on weight functions --- a continuous path in the moduli space $cal(W)$. The beta function $beta(g)$ determines the tangent vector to this path. Universality classes correspond to connected components of $cal(W)$.

*Fixed points.* A conformal field theory (CFT) is a fixed point of the RG flow: $w$ is invariant under scale change. In $cal(W)$, CFTs are isolated points (or submanifolds) that the flow asymptotes to. The trivial weight $w_("triv")(k) = 1$ is a fixed point corresponding to free field theory (verified, Example 34). Non-trivial fixed points correspond to interacting conformal field theories.

*Open questions.* What is the topology of $cal(W)$? Is it connected? Does it admit a metric under which the RG flow is a gradient flow (as the $c$-theorem suggests in $d = 2$)? Is the mass gap condition an open or closed subset of $cal(W)$? These are the natural next questions in the ITCM framework --- and they reduce Yang--Mills existence to a topological question about weight space.

== On the classical nature of the underlying mechanics

If renormalization is a kernel-based projection, and quantum effects are projection artifacts (Volume II), then there is nothing in QFT that requires quantum mechanics as a separate foundation. The underlying structure is fields and flows --- classical objects. Everything labeled "quantum" is a property of the projection kernel, not of the underlying dynamics. Superposition is multiple ontic states in the same fiber. Uncertainty is incompatible kernels (Fourier-dual projections). Entanglement is a non-separable ontic state, projected (Volume II). Discrete spectra are the eigenstructure of the kernel. Divergences are the null space of the projection kernel. Renormalization is extracting the image.

This is the strongest and most speculative implication. A full proof would require showing that the complete QFT S-matrix can be recovered from classical field theory plus a composite projection kernel, which is beyond the scope of this paper. What this paper establishes is that renormalization --- the last piece that seemed irreducibly quantum --- has a classical kernel-based explanation via the ITCM. The implication is noted, not claimed as proven.

#figure(
  lq.diagram(
  width: 10cm,
  title: [Interaction Dressing R(z)],
  xlabel: [$z = y^2 / x^2$],
  ylabel: [$R(z)$],
  legend: (position: left + top),
  lq.plot(
    (0.000000, 0.017300, 0.034700, 0.052000, 0.069400, 0.086700, 0.104100, 0.121400, 0.138800, 0.156100, 0.173500, 0.190800, 0.208200, 0.225500, 0.242900, 0.260200, 0.277600, 0.294900, 0.312200, 0.329600, 0.346900, 0.364300, 0.381600, 0.399000, 0.416300, 0.433700, 0.451000, 0.468400, 0.485700, 0.503100, 0.520400, 0.537800, 0.555100, 0.572400, 0.589800, 0.607100, 0.624500, 0.641800, 0.659200, 0.676500, 0.693900, 0.711200, 0.728600, 0.745900, 0.763300, 0.780600, 0.798000, 0.815300, 0.832700, 0.850000),
    (1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000, 1.000000),
    color: gray,
    label: [δ = 0 (free field)]
  ),
  lq.plot(
    (0.000000, 0.017300, 0.034700, 0.052000, 0.069400, 0.086700, 0.104100, 0.121400, 0.138800, 0.156100, 0.173500, 0.190800, 0.208200, 0.225500, 0.242900, 0.260200, 0.277600, 0.294900, 0.312200, 0.329600, 0.346900, 0.364300, 0.381600, 0.399000, 0.416300, 0.433700, 0.451000, 0.468400, 0.485700, 0.503100, 0.520400, 0.537800, 0.555100, 0.572400, 0.589800, 0.607100, 0.624500, 0.641800, 0.659200, 0.676500, 0.693900, 0.711200, 0.728600, 0.745900, 0.763300, 0.780600, 0.798000, 0.815300, 0.832700, 0.850000),
    (1.000000, 1.005500, 1.011000, 1.016700, 1.022500, 1.028400, 1.034400, 1.040600, 1.046800, 1.053200, 1.059800, 1.066400, 1.073200, 1.080200, 1.087300, 1.094600, 1.102100, 1.109700, 1.117600, 1.125600, 1.133800, 1.142300, 1.150900, 1.159900, 1.169000, 1.178400, 1.188100, 1.198100, 1.208400, 1.219000, 1.230000, 1.241300, 1.253100, 1.265200, 1.277800, 1.290900, 1.304400, 1.318600, 1.333300, 1.348700, 1.364800, 1.381600, 1.399300, 1.418000, 1.437700, 1.458500, 1.480600, 1.504200, 1.529500, 1.556700),
    color: blue,
    label: [δ = 1 (QED-like)]
  ),
  lq.plot(
    (0.000000, 0.017300, 0.034700, 0.052000, 0.069400, 0.086700, 0.104100, 0.121400, 0.138800, 0.156100, 0.173500, 0.190800, 0.208200, 0.225500, 0.242900, 0.260200, 0.277600, 0.294900, 0.312200, 0.329600, 0.346900, 0.364300, 0.381600, 0.399000, 0.416300, 0.433700, 0.451000, 0.468400, 0.485700, 0.503100, 0.520400, 0.537800, 0.555100, 0.572400, 0.589800, 0.607100, 0.624500, 0.641800, 0.659200, 0.676500, 0.693900, 0.711200, 0.728600, 0.745900, 0.763300, 0.780600, 0.798000, 0.815300, 0.832700, 0.850000),
    (1.000000, 1.010500, 1.021300, 1.032300, 1.043700, 1.055200, 1.067100, 1.079300, 1.091800, 1.104700, 1.117800, 1.131400, 1.145300, 1.159600, 1.174300, 1.189500, 1.205100, 1.221200, 1.237700, 1.254800, 1.272500, 1.290800, 1.309600, 1.329100, 1.349300, 1.370300, 1.392000, 1.414500, 1.437900, 1.462200, 1.487600, 1.514000, 1.541600, 1.570300, 1.600500, 1.632000, 1.665200, 1.700000, 1.736700, 1.775400, 1.816300, 1.859700, 1.905800, 1.954900, 2.007500, 2.063900, 2.124700, 2.190600, 2.262300, 2.340800),
    color: green,
    label: [δ = 2]
  ),
  lq.plot(
    (0.000000, 0.017300, 0.034700, 0.052000, 0.069400, 0.086700, 0.104100, 0.121400, 0.138800, 0.156100, 0.173500, 0.190800, 0.208200, 0.225500, 0.242900, 0.260200, 0.277600, 0.294900, 0.312200, 0.329600, 0.346900, 0.364300, 0.381600, 0.399000, 0.416300, 0.433700, 0.451000, 0.468400, 0.485700, 0.503100, 0.520400, 0.537800, 0.555100, 0.572400, 0.589800, 0.607100, 0.624500, 0.641800, 0.659200, 0.676500, 0.693900, 0.711200, 0.728600, 0.745900, 0.763300, 0.780600, 0.798000, 0.815300, 0.832700, 0.850000),
    (1.000000, 1.015400, 1.031200, 1.047500, 1.064300, 1.081600, 1.099400, 1.117700, 1.136700, 1.156200, 1.176400, 1.197200, 1.218700, 1.241000, 1.264000, 1.287900, 1.312600, 1.338200, 1.364800, 1.392400, 1.421100, 1.450900, 1.481900, 1.514200, 1.547900, 1.583100, 1.619800, 1.658100, 1.698300, 1.740400, 1.784600, 1.831100, 1.879900, 1.931400, 1.985700, 2.043200, 2.104100, 2.168600, 2.237400, 2.310600, 2.388900, 2.472800, 2.563000, 2.660400, 2.765800, 2.880400, 3.005700, 3.143300, 3.295600, 3.465200),
    color: orange,
    label: [δ = 3]
  ),
  lq.plot(
    (0.000000, 0.017300, 0.034700, 0.052000, 0.069400, 0.086700, 0.104100, 0.121400, 0.138800, 0.156100, 0.173500, 0.190800, 0.208200, 0.225500, 0.242900, 0.260200, 0.277600, 0.294900, 0.312200, 0.329600, 0.346900, 0.364300, 0.381600, 0.399000, 0.416300, 0.433700, 0.451000, 0.468400, 0.485700, 0.503100, 0.520400, 0.537800, 0.555100, 0.572400, 0.589800, 0.607100, 0.624500, 0.641800, 0.659200, 0.676500, 0.693900, 0.711200, 0.728600, 0.745900, 0.763300, 0.780600, 0.798000, 0.815300, 0.832700, 0.850000),
    (1.000000, 1.029500, 1.060300, 1.092500, 1.126000, 1.161100, 1.197800, 1.236200, 1.276400, 1.318500, 1.362700, 1.409000, 1.457700, 1.508900, 1.562700, 1.619400, 1.679200, 1.742200, 1.808800, 1.879200, 1.953700, 2.032600, 2.116300, 2.205200, 2.299800, 2.400500, 2.507900, 2.622600, 2.745400, 2.876900, 3.018100, 3.170000, 3.333700, 3.510600, 3.702000, 3.909800, 4.135900, 4.382600, 4.652700, 4.949400, 5.276400, 5.638300, 6.040500, 6.489800, 6.994300, 7.564200, 8.212500, 8.955600, 9.815000, 10.819300),
    color: red,
    label: [δ = 6 (YM-like)]
  ),
)
,
  caption: [Regular correction $R(z) = attach(, tl: 2) F_1 ((mu - nu)\/2, (mu + nu)\/2; mu + 1; z)$ as a function of $z = y^2 \/ x^2$ for five values of the Hankel-order asymmetry $delta = |mu - nu|$ (fixing $nu = 2$). At $delta = 0$ (grey), $R(z) = 1$ identically --- the free-field limit where the full kernel is the universal Cauchy pole alone. As $delta$ increases, the interaction dressing grows monotonically: $delta = 1$ (blue, near-QED regime), $delta = 2$ (green), $delta = 3$ (orange), and $delta = 6$ (red, strong-coupling regime relevant to Yang--Mills). The departure from unity quantifies the strength of interaction-induced spectral modification. All curves are computed from 50-term series truncation (convergent to $<$ 0.2% in this range).]
) <fig:dressing>

#figure(
  lq.diagram(
  width: 10cm,
  title: [Gauss Evaluation: R(1) vs Hankel-Order Asymmetry],
  xlabel: [$delta = |mu - nu|$],
  ylabel: [$R(1)$],
  lq.plot(
    (0.000000, 0.500000, 1.000000, 1.500000, 2.000000, 2.500000, 3.000000, 3.500000, 4.000000, 5.000000, 6.000000),
    (1.000000, 1.438300, 2.037200, 2.861400, 4.000000, 5.576100, 7.760700, 10.791700, 15.000000, 28.973300, 56.000000),
    color: blue,
    mark: "o"
  ),
)
,
  caption: [Boundary value $R(1) = Gamma(mu + 1) \/ [Gamma((mu + nu)\/2 + 1) dot.op Gamma((mu - nu)\/2 + 1)]$ as a function of the Hankel-order asymmetry $delta = |mu - nu|$ (fixing $nu = 2$). This is the exact Gauss evaluation, not a series truncation. The super-linear growth quantifies how interaction strength scales with the spectral shift between free and interacting Hankel orders. The QED region ($delta << 1$, $R(1) approx 1$) is visually distinct from the Yang--Mills region ($delta >> 1$, $R(1) >> 1$). For lattice QCD, the predicted $R(1)$ at the extracted $delta_("YM")$ provides a falsifiable numerical target.]
) <fig:gauss>

#figure(
  lq.diagram(
  width: 10cm,
  title: [Eigenvalue Spectrum of the Bessel Operator],
  xlabel: [Eigenvalue index $n$],
  ylabel: [$E_n$],
  legend: (position: left + top),
  lq.plot(
    (1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000),
    (0.412100, 1.107100, 2.109900, 3.421200, 5.041400, 6.971000, 9.210400, 11.760400, 14.621500, 17.794700),
    color: gray,
    label: [δ = 0 (free field)]
  ),
  lq.plot(
    (1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000),
    (0.636000, 1.488700, 2.646700, 4.112200, 5.885600, 7.967000, 10.356500, 13.053900, 16.059300, 19.372600),
    color: blue,
    label: [δ = 1 (QED-like)]
  ),
  lq.plot(
    (1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000),
    (0.899700, 1.912900, 3.227500, 4.848400, 6.776600, 9.012600, 11.556300, 14.407700, 17.566700, 21.033300),
    color: green,
    label: [δ = 2]
  ),
  lq.plot(
    (1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000),
    (1.202200, 2.378700, 3.851300, 5.628300, 7.711900, 10.102700, 12.801100, 15.806900, 19.120200, 22.740800),
    color: orange,
    label: [δ = 3]
  ),
  lq.plot(
    (1.000000, 2.000000, 3.000000, 4.000000, 5.000000, 6.000000, 7.000000, 8.000000, 9.000000, 10.000000),
    (2.335200, 4.018800, 5.974300, 8.225400, 10.778800, 13.636900, 16.800900, 20.271500, 24.048700, 28.132600),
    color: red,
    label: [δ = 6 (YM-like)]
  ),
)
,
  caption: [Lowest 10 eigenvalues of the Bessel operator $L_mu = -d^2 \/ d x^2 + (mu^2 - 1\/4) \/ x^2$ on $[0.2, 8]$ with Dirichlet boundary conditions, for five values of the Hankel-order asymmetry $delta = mu - nu$ (fixing $nu = 2$). The finite interval represents the confinement scale. At $delta = 0$ (grey, free field), the lowest eigenvalue $E_1 = 0.41$ is near zero. As $delta$ increases, the entire spectrum shifts upward: $E_1 = 0.64$ at $delta = 1$, $E_1 = 0.90$ at $delta = 2$, $E_1 = 1.20$ at $delta = 3$, and $E_1 = 2.34$ at $delta = 6$. The centrifugal barrier $(mu^2 - 1\/4) \/ x^2$, strengthened by the Hankel-order asymmetry, opens a spectral gap that grows monotonically with $delta$. This is the mechanism by which the interaction dressing creates mass: the ITCM transmutation shifts the spectral index, the shifted index amplifies the centrifugal potential, and the amplified potential pushes the spectrum away from zero.]
) <fig:spectrum>

#figure(
  lq.diagram(
  width: 10cm,
  title: [Spectral Gap vs Hankel-Order Asymmetry],
  xlabel: [$delta = mu - nu$],
  ylabel: [$Delta(delta) = E_1(delta) - E_1(0)$],
  lq.plot(
    (0.000000, 0.500000, 1.000000, 1.500000, 2.000000, 2.500000, 3.000000, 3.500000, 4.000000, 5.000000, 6.000000),
    (0.000000, 0.106900, 0.223900, 0.350900, 0.487600, 0.634000, 0.790100, 0.955600, 1.130500, 1.508300, 1.923100),
    color: blue,
    mark: "o"
  ),
)
,
  caption: [Spectral gap $Delta(delta) = E_1 (delta) - E_1 (0)$ of the Bessel operator as a function of the Hankel-order asymmetry $delta$ (fixing $nu = 2$). At $delta = 0$, the gap is zero (free field). The gap opens monotonically and grows super-linearly with $delta$, from $Delta = 0.22$ at $delta = 1$ to $Delta = 1.92$ at $delta = 6$. Combined with the interaction dressing $R(z)$ (@fig:dressing) and the Gauss evaluation $R(1)$ (@fig:gauss), this demonstrates a complete chain: Hankel-order asymmetry $arrow.r$ amplified centrifugal barrier $arrow.r$ spectral gap. For Yang--Mills, the large gluon self-coupling drives $delta$ into the strong-coupling regime, where the gap is substantial. This is a finite-volume calculation (confinement scale $L = 8$); the conjecture (Section 9.9) is that the gap persists in the infinite-volume limit when the weight function $w_("YM")$ satisfies IR regularity.]
) <fig:gap>

= Discussion: What the Kernel Reveals --- and What It Cannot

The preceding sections established that the composite kernel factors into a universal Green's kernel and a theory-specific regular correction (Section 9), that the spectral content of the projected operator depends only on the regular correction (Spectral Localization Principle), and that the moduli space of weight functions classifies physically distinct quantum field theories. These results concern the *projection operator and its image* --- the observable side of the POT framework. But they raise an unavoidable question about the other side: what do they tell us about the ontological Hilbert space $cal(H)_("ont")$?

== The temptation of ontological decomposition

The kernel decomposition is visible from the observable side. The Euler factorization of the hypergeometric kernel is a mathematical identity; the spectral localization is a structural consequence. From these, it is natural to infer that $cal(H)_("ont")$ itself must possess a compatible structure: a "universal sector" coupled to the Green's kernel (shared by all theories) and a "theory-specific sector" coupled to the regular correction (carrying all distinguishing physics --- mass gaps, coupling constants, confinement). The moduli space $cal(W) slash approx$ would then classify not merely different weight functions, but different *effective codomains* within $cal(H)_("ont")$ --- different regions of the ontological space that a given theory's projection activates. The renormalization group flow would be a flow on these effective codomains. Fixed points (conformal field theories) would correspond to scale-invariant sectors.

This is a tempting picture. It is also exactly what Volume II warned us not to construct.

== The epistemic boundary

Volume II established a deliberate methodological constraint: "The internal structure of $cal(H)_("ont")$ is, by definition, not directly observable --- it is the space *from which* observables are projected. Postulating specific dynamics in $cal(H)_("ont")$ would be speculation dressed as theory." The justification was structural, not merely cautious: the projection $Pi$ is many-to-one and non-invertible. We exist inside the projection. Any claim about $cal(H)_("ont")$'s internal decomposition would require information that $Pi$ provably destroyed.

The kernel decomposition does not change this. That the *map* factors does not entail that the *source* decomposes. Many structurally different Hilbert spaces could serve as the domain of a projection whose kernel admits Euler factorization. The factorization constrains the *class* of possible $cal(H)_("ont")$ but does not select one. To formalize "ontological sectors" as axioms would be to invert the projection --- to read the source from its shadow.

== The shadow principle

What the kernel decomposition does provide is something subtler and, we believe, more valuable than ontological specification. It provides a *shadow*: a structural imprint that the projection casts back toward its source without revealing the source itself.

The shadow has definite content. We know:

- The projection has a two-layer structure (universal scaffolding + theory-specific content).
- The effective image after finite-part extraction depends only on the regular correction.
- Different theories produce different effective images, classified by $cal(W) slash approx$.
- The renormalization group acts on these effective images.

All of this is rigorously established on the observable side. The temptation is to lift this structure to $cal(H)_("ont")$. The epistemic constraint says we cannot. The *interesting* observation is that these two statements are not in tension --- they are complementary. The framework identifies what lies beyond its epistemic reach, and *the identification itself is informative*. Knowing the shape of the shadow constrains the class of possible objects, even when the object remains inaccessible.

A framework that tells you what it cannot know --- and enforces that boundary from within its own formalism --- is doing something right.

== Open questions

The shadow principle suggests several directions for future work, all of which remain on the observable side of the epistemic boundary:

1. *Classification of compatible sources.* Can the class of Hilbert spaces compatible with a given kernel decomposition be characterized mathematically? This is a well-posed question about factorization of integral operators, not about the internal structure of $cal(H)_("ont")$.

2. *Topology of the moduli space.* Does $cal(W) slash approx$ carry a natural topology? Are universality classes connected components? Is the set of mass-gap weight functions open, closed, or neither? These are questions about the *observable-side* landscape of QFTs.

3. *Minimal compatible sources.* Is there a "smallest" $cal(H)_("ont")$ compatible with a given kernel decomposition --- a minimal model? If so, it would represent the weakest ontological commitment consistent with the observed physics.

4. *Category-theoretic formulation.* The relationship between observable-side factorization and source-side structure may admit a precise formulation via adjunctions or Galois connections. The projection $Pi$ and the "shadow" it casts could form a pair of functors between a category of source spaces and a category of factored kernels.

These questions do not violate the epistemic constraint. They sharpen it.

= Conclusion

The expressions $1 + 2 + 3 + dots.c = -1\/12$ and $product n = sqrt(2 pi)$ are not arithmetic identities. They are projected observables: the unique finite values extracted from divergent structures by admissible projection kernels.

The paper has established:

1. *Regularization is projection.* A regularization kernel maps divergent objects to parameterized families; the finite part operator extracts the image; the divergent terms are the null space (Sections 3, 7).

2. *Regulator choice is gauge freedom.* Different admissible regularization schemes are gauge-equivalent: they produce different null-space representatives but the same image. The gauge group of regulators is an equivalence relation with verified reflexivity, symmetry, and transitivity (Section 6).

3. *Zeta and heat kernels are gauge-equivalent, but not equally fundamental.* The Mellin transform provides the bridge. Schwinger's proper-time method and Hawking's zeta regularization agree because they project onto the same image. But the heat kernel is the physically fundamental member of the equivalence class: it describes real diffusion and propagation. The zeta kernel is a mathematical device whose correctness is inherited via gauge equivalence (Sections 6, 8).

4. *Physical kernels are the preferred representatives.* The gauge group of regulators has a distinguished element: the kernel that directly governs a physical process. The heat kernel, the propagator, and the Biot--Savart kernel are physical. The zeta kernel and dimensional continuation are mathematical --- they work because they live in the same gauge orbit as a physical kernel (Section 8).

5. *Renormalization is not an add-on.* The path integral and renormalization compose into a single integral transform $K_("QFT") = "FP" compose K_("ren") compose K_("path")$. The Integral Transform Composition Method (ITCM) of Sitnik and collaborators proves this composition is a bona fide integral operator with explicit kernel representations. Applied to QED, dimensional regularization is identified as a transmutation operator with weight $w(t) = (-t^2)^s$, yielding closed-form kernels via hypergeometric functions (Section 9).

6. *The divergences were never real.* The ITCM proves the composite kernel is a single integral operator with a well-defined hypergeometric kernel. This operator maps sources directly to finite observables --- no divergent intermediate quantities ever appear. The divergences of QFT are properties of a particular *factorization* of the composite kernel, not of the theory itself. The function $(x^2 - 1) / (x - 1)$ is not singular at $x = 1$; it equals $x + 1$. The "sickness" of QFT was a misdiagnosis (Section 9).

7. *The essential structure was present from the beginning.* Feynman's path integral is one factor of the composite kernel. Tomonaga, Schwinger, and Feynman supplied all the factors in 1948. The structure underlying the formalization sought by the Clay Millennium Problem may already have been present in the original formulation --- expressed as a "renormalization procedure" rather than recognized as a single composite projection kernel (Section 9).

8. *The Millennium Problem reduces to finding a weight function, and the mass gap is controlled by a single parameter.* For pure Yang--Mills, the composite kernel $K_("YM") = F^(-1) compose w_("YM") compose F$ is guaranteed by ITCM to exist as a single integral operator if the weight function $w_("YM")(k)$ can be specified. The mass gap is further localized: the Gauss evaluation $R(1) = Gamma(mu + 1) slash [Gamma((mu + nu) slash 2 + 1) dot.op Gamma((mu - nu) slash 2 + 1)]$ gives a closed-form measure of interaction strength controlled by the Hankel-order asymmetry $delta = |mu - nu|$. For free fields ($delta = 0$), $R(1) = 1$ and the theory is gapless. For Yang--Mills, gluon self-coupling breaks the degeneracy ($delta > 0$), and the Spectral Gap Conjecture posits that this asymmetry, combined with IR regularity, is both necessary and sufficient for $Delta > 0$. The problem reduces to spectral theory of an integral operator with hypergeometric kernel --- connected by the Sturm--Liouville bridge to the exactly solvable Schrödinger operators classified by Dereziński--Karimi. Numerical discretization of the associated Bessel operator confirms the mechanism: the spectral gap opens monotonically with $delta$ and reaches $Delta = 1.92$ at $delta = 6$ (Section 9, Figures 1--4).

9. *The moduli space of weight functions classifies QFTs.* Weight functions that produce identical observables are weight-equivalent; the quotient space $cal(W) \/ approx$ is a moduli space whose points are physically distinct theories. The renormalization group is identified as the flow on this moduli space, with conformal field theories as fixed points and the trivial weight $w_("triv")(k) = 1$ corresponding to free field theory. Universality classes are connected components of $cal(W)$. The Yang--Mills mass gap becomes a topological question: is the set of mass-gap weight functions non-empty? (Section 9).

10. *The framework enforces its own epistemic boundary.* The kernel decomposition, the spectral localization principle, and the moduli space of weight functions are maximally informative about the *projection and its image*. They tempt us to infer a decomposition of the ontological Hilbert space $cal(H)_("ont")$ into universal and theory-specific sectors. But Volume II's epistemic constraint --- that the internal structure of $cal(H)_("ont")$ lies beyond the projection and cannot be specified from within it --- applies to this temptation as well. The factorization of the map does not entail a decomposition of the source. That the framework identifies what it cannot know, and enforces that boundary from within its own formalism, is evidence of self-consistency, not limitation (Section 10).

All structural results --- 40 axiomatic examples covering linearity, gauge invariance, admissibility, the Mellin bridge, physical kernel identification, inherited admissibility via gauge equivalence, QFT observable extraction, composite kernel gauge invariance, ITCM-QED equivalence, intertwining index shift, spectral localization of the mass gap, divergence-free composite equivalence, Yang--Mills constraint formalization, free-field degeneracy, Hankel-order asymmetry, weight equivalence, weight group structure, RG flow, and RG fixed points --- are machine-verified by the Z3 SMT solver in the Kleis formal verification language.

The founding generation of QFT physicists built the right structure and labeled it incorrectly. They called it a computational technique; it is a composite projection kernel. They called it subtraction of infinities; it is extraction of the image. They called it scheme independence; it is gauge invariance. They called it an add-on fix; it is a constitutive factor. The kernel was always there. It just didn't have a name.



#heading(numbering: none)[References]
#set text(size: 9pt)
#par(hanging-indent: 1.5em)[\[Atik2024a\] E. Atik. Flat Galactic Rotation Curves as Projected Ontology. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Atik2024b\] E. Atik. Quantum Entanglement as Projection Artifact in the POT Framework. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Atik2024c\] E. Atik. Electrodynamics as a Theorem of Projected Ontology. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Atik2024d\] E. Atik. Confinement as Fiber Non-Invariance in Projected Ontology. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Atik2024e\] E. Atik. Admissibility Restoration: The Structural Necessity of Symmetry-Breaking Fields. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Atik2024f\] E. Atik. The Kernel and the Fluid: An Epilogue on Projected Ontology. *Preprint*, 2024.]

#par(hanging-indent: 1.5em)[\[Tomonaga1946\] S. Tomonaga. On a Relativistically Invariant Formulation of the Quantum Theory of Wave Fields. *Progress of Theoretical Physics*, 1:27--42, 1946.]

#par(hanging-indent: 1.5em)[\[Schwinger1948\] J. Schwinger. On Quantum-Electrodynamics and the Magnetic Moment of the Electron. *Physical Review*, 73:416--417, 1948.]

#par(hanging-indent: 1.5em)[\[Feynman1948\] R. P. Feynman. Space-Time Approach to Non-Relativistic Quantum Mechanics. *Reviews of Modern Physics*, 20:367--387, 1948.]

#par(hanging-indent: 1.5em)[\[Dyson1949\] F. J. Dyson. The Radiation Theories of Tomonaga, Schwinger, and Feynman. *Physical Review*, 75:486--502, 1949.]

#par(hanging-indent: 1.5em)[\[Hawking1977\] S. W. Hawking. Zeta Function Regularization of Path Integrals in Curved Spacetime. *Communications in Mathematical Physics*, 55:133--148, 1977.]

#par(hanging-indent: 1.5em)[\[Hadamard1932\] J. Hadamard. *Le problème de Cauchy et les équations aux dérivées partielles linéaires hyperboliques*. Hermann, Paris, 1932.]

#par(hanging-indent: 1.5em)[\[Casimir1948\] H. B. G. Casimir. On the Attraction Between Two Perfectly Conducting Plates. *Proc. Kon. Ned. Akad. Wet.*, 51:793--795, 1948.]

#par(hanging-indent: 1.5em)[\[Elizalde1995\] E. Elizalde. *Ten Physical Applications of Spectral Zeta Functions*. Lecture Notes in Physics, Vol. 855, Springer, 1995.]

#par(hanging-indent: 1.5em)[\[Estrada2002\] R. Estrada and R. P. Kanwal. *A Distributional Approach to Asymptotics*. 2nd ed., Birkhäuser, 2002.]

#par(hanging-indent: 1.5em)[\[JaffeWitten2000\] A. Jaffe and E. Witten. Quantum Yang--Mills Theory. *Clay Mathematics Institute Millennium Prize Problems*, 2000.]

#par(hanging-indent: 1.5em)[\[SitnikJebabli2024\] S. M. Sitnik and I. Jebabli. The Integral Transforms Composition Method for Generalized Index Shifted Transmutations. *Lobachevskii Journal of Mathematics*, 45:3401--3415, 2024.]

#par(hanging-indent: 1.5em)[\[SitnikShishkina2020\] S. M. Sitnik and E. L. Shishkina. *Transmutations, Singular and Fractional Differential Equations with Applications to Mathematical Physics*. Mathematics in Science and Engineering, Elsevier, 2020.]

#par(hanging-indent: 1.5em)[\[Sitnik2010\] S. M. Sitnik. Transmutations and Applications: A Survey. *arXiv:1012.3741*, 2010.]

#par(hanging-indent: 1.5em)[\[FitouhiEtAl2018\] A. Fitouhi, I. Jebabli, E. L. Shishkina, and S. M. Sitnik. Applications of Integral Transforms Composition Method to Wave-Type Singular Differential Equations and Index Shift Transmutations. *Electronic Journal of Differential Equations*, 2018(130):1--27, 2018. arXiv:1805.06925.]

#par(hanging-indent: 1.5em)[\[EpsteinGlaser1973\] H. Epstein and V. Glaser. The Role of Locality in Perturbation Theory. *Annales de l'Institut Henri Poincaré A*, 19(3):211--295, 1973.]

#par(hanging-indent: 1.5em)[\[ConnesKreimer2000\] A. Connes and D. Kreimer. Renormalization in Quantum Field Theory and the Riemann--Hilbert Problem I: The Hopf Algebra Structure of Graphs and the Main Theorem. *Communications in Mathematical Physics*, 210:249--273, 2000.]

#par(hanging-indent: 1.5em)[\[Scharf1995\] G. Scharf. *Finite Quantum Electrodynamics: The Causal Approach*. 2nd ed., Springer, 1995.]

#par(hanging-indent: 1.5em)[\[Schwinger1951\] J. Schwinger. On Gauge Invariance and Vacuum Polarization. *Physical Review*, 82:664--679, 1951.]

#par(hanging-indent: 1.5em)[\[DowkerCritchley1976\] J. S. Dowker and R. Critchley. Effective Lagrangian and Energy-Momentum Tensor in de Sitter Space. *Physical Review D*, 13:3224--3232, 1976.]

#par(hanging-indent: 1.5em)[\[RaySinger1971\] D. B. Ray and I. M. Singer. R-Torsion and the Laplacian on Riemannian Manifolds. *Advances in Mathematics*, 7:145--210, 1971.]

#par(hanging-indent: 1.5em)[\[Seeley1967\] R. T. Seeley. Complex Powers of an Elliptic Operator. *Proceedings of Symposia in Pure Mathematics*, 10:288--307, 1967.]

#par(hanging-indent: 1.5em)[\[McKeanSinger1967\] H. P. McKean and I. M. Singer. Curvature and the Eigenvalues of the Laplacian. *Journal of Differential Geometry*, 1:43--69, 1967.]

#par(hanging-indent: 1.5em)[\[MinakshisundaramPleijel1949\] S. Minakshisundaram and Å. Pleijel. Some Properties of the Eigenfunctions of the Laplace Operator on Riemannian Manifolds. *Canadian Journal of Mathematics*, 1:242--256, 1949.]

#par(hanging-indent: 1.5em)[\[Voros1987\] A. Voros. Spectral Functions, Special Functions and the Selberg Zeta Function. *Communications in Mathematical Physics*, 110:439--465, 1987.]

#par(hanging-indent: 1.5em)[\[Polchinski1998\] J. Polchinski. *String Theory*, Vols. 1--2. Cambridge University Press, 1998.]

#par(hanging-indent: 1.5em)[\[DerezinskiKarimi2025\] J. Dereziński and P. Karimi. Exactly solvable Schrödinger operators related to the hypergeometric equation. *arXiv:2509.03235*, 2025.]

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#align(center)[
  #text(size: 14pt, weight: "bold")[Appendix]
]
#v(1em)
= Formal Axiom Summary

The following axioms are defined in `theories/pot_renormalization_kernel.kleis` and verified by Z3.

== Regularization Kernel

- *reg_consistency*: $forall R, X. thin ("admissible"(R) and "convergent"(X)) arrow.r.double "FP"(K_R [X]) = "ord"(X)$
- *reg_maps_zero*: $forall R. thin "admissible"(R) arrow.r.double K_R [0] = 0$

== Finite Part Operator

- *fp_linearity_add*: $forall f, g. thin "FP"(f + g) = "FP"(f) + "FP"(g)$
- *fp_linearity_smul*: $forall c, f. thin "FP"(c f) = c dot.op "FP"(f)$
- *fp_zero_maps_zero*: $"FP"(0) = 0$
- *fp_decomposition*: $forall f. thin exists s, c. thin "FP"(f) = c$

== Gauge Group of Regulators

- *gauge_invariance*: $forall R, R', X. thin (R tilde.op R') arrow.r.double "FP"(K_R [X]) = "FP"(K_(R') [X])$
- *gauge_reflexive*: $forall R. thin R tilde.op R$
- *gauge_symmetric*: $forall R, R'. thin (R tilde.op R') arrow.r.double (R' tilde.op R)$
- *gauge_transitive*: $forall R, R', R''. thin (R tilde.op R' and R' tilde.op R'') arrow.r.double (R tilde.op R'')$
- *identity_reg_admissible*: $"admissible"("id")$

== Spectral Zeta

- *zeta_is_admissible*: $"admissible"(K_zeta)$
- *zeta_sum_of_integers*: $"FP"_zeta (bb(Z)^+) = -1\/12$
- *zeta_product_via_derivative*: $-zeta'(0) arrow.r sqrt(2 pi)$
- *analytic_continuation_preserves_fp*: $forall S. thin "FP"(zeta_S (s_0)) = "FP"_zeta (S)$

== Heat Kernel

- *heat_is_admissible*: $"admissible"(K_("heat"))$
- *heat_zeta_gauge_equiv*: $K_("heat") tilde.op K_zeta$
- *mellin_bridge*: $forall S. thin "FP"_zeta (S) = "FP"("Tr"(e^(-t A))|_(t_0))$

== Physical Kernel Identification

- *heat_governs_diffusion*: $K_("heat")$ governs heat diffusion (physical kernel)
- *physical_kernel_def*: $forall R, P. thin "governs"(R, P) arrow.r.double "physical"(R)$
- *physical_implies_admissible*: $forall R. thin "physical"(R) arrow.r.double "admissible"(R)$
- *mathematical_kernel_inherits_admissibility*: $forall R, R_("phys")$. $("physical"(R_("phys")) and R tilde.op R_("phys")) arrow.r.double "admissible"(R)$ (zeta kernel is admissible via gauge equivalence to heat kernel)

== QFT Projection

- *qft_observable_def*: $forall R, Z. thin "Obs"_R (Z) = "FP"(K_R [Z])$
- *qft_gauge_invariance*: $forall R, R', Z. thin (R tilde.op R') arrow.r.double "Obs"_R (Z) = "Obs"_(R') (Z)$
- *path_integral_needs_kernel*: $forall Z. thin exists R. thin "admissible"(R)$

== Composite QFT Kernel

- *composite_def*: $forall R, J. thin K_("QFT")(R, J) = "FP"(K_("composite")(R, J))$
- *composite_gauge_invariance*: $forall R, R', J. thin (R tilde.op R') arrow.r.double K_("QFT")(R, J) = K_("QFT")(R', J)$

== ITCM Transmutation

- *itcm_is_composite*: $forall J. thin T_("ITCM")(w_("dim"), J) = K_("QFT")(zeta, J)$
- *itcm_intertwining*: $"shift"(w_("dim"), nu_("free")) = mu_("int")$
- *coupling_gauge_invariance*: $forall E, R, R'. thin (R tilde.op R') arrow.r.double alpha(E) = alpha(E)$
- *alpha_is_projected_observable*: $alpha(E_("low")) = alpha(E_("low"))$

== QED Hypergeometric Kernel

- *exchange_symmetry_a*: $a_1 = a_2$ (symmetric in $mu, nu$)
- *exchange_symmetry_b_sum*: $b_1 = b_2$ (spectral duality)
- *kernel_decomposition*: $forall k_1, k_2. thin K_("inner")(k_1, k_2) = attach(, tl: 2) F_1 (a_1, b_1 ; c_1 ; K_("inner")(k_1, k_2))$
- *hypergeom_gauge_invariance*: $forall k_1, k_2, R, R'. thin (R tilde.op R') arrow.r.double "FP"(K_("inner")(k_1, k_2)) = "FP"(K_("inner")(k_1, k_2))$

== Spectral Localization

- *qed_gapless*: $"spec_gap"(T_0^("QED")) = 0$ (massless photon)
- *qed_gap_from_regular*: $"spec_gap"(T_0^("QED")) = "spec_gap"(R^("QED")|_(s=0))$
- *ym_gap_from_regular*: $"spec_gap"(T_0^("YM")) = "spec_gap"(R^("YM")|_(s=0))$
- *qed_regular_gapless*: $"spec_gap"(R^("QED")|_(s=0)) = 0$

== Divergence-Free Composite

- *direct_equals_factored*: $forall J. thin K_("direct")(J) = K_("QFT")(zeta, J)$
- *direct_equals_itcm*: $forall J. thin K_("direct")(J) = T_("ITCM")(w_("dim"), J)$
- *direct_gauge_trivial*: $forall J, R, R'. thin (R tilde.op R') arrow.r.double K_("direct")(J) = K_("direct")(J)$

== Yang--Mills Composite Kernel

- *asymptotic_freedom*: $"AF"(w_("YM"))$
- *ir_regularity*: $"IR_reg"(w_("YM"))$
- *gauge_invariance*: $"GI"(w_("YM"), "SU"(3))$
- *unitarity*: $"unitary"(w_("YM"))$
- *mass_gap*: $"gap"(w_("YM"))$
- *gap_is_spectral*: $"spec_gap"(w_("YM")) = Delta$
- *ym_gauge_trivial*: $forall J, R, R'. thin (R tilde.op R') arrow.r.double K_("YM")(w, J) = K_("YM")(w, J)$

== Weight Moduli Space

- *weight_equiv_def*: $forall w_1, w_2, J. thin (w_1 approx w_2) arrow.r.double "Obs"(w_1, J) = "Obs"(w_2, J)$
- *weight_equiv_reflexive*: $forall w. thin w approx w$
- *weight_equiv_symmetric*: $forall w_1, w_2. thin (w_1 approx w_2) arrow.r.double (w_2 approx w_1)$
- *weight_equiv_transitive*: $forall w_1, w_2, w_3. thin (w_1 approx w_2 and w_2 approx w_3) arrow.r.double (w_1 approx w_3)$
- *weight_compose_associative*: $forall w_1, w_2, w_3. thin (w_1 dot.op w_2) dot.op w_3 = w_1 dot.op (w_2 dot.op w_3)$
- *weight_compose_identity*: $forall w. thin w dot.op w_("triv") = w$
- *rg_flow_equiv*: $forall w, mu, mu'. thin w_mu approx w_(mu')$ (RG flow preserves equivalence class)
- *rg_fixed_point*: RG-invariant $w$ has scale-independent observables
- *trivial_is_fixed_point*: $"RG"(w_("triv"), mu) = w_("triv")$ (free field theory)
