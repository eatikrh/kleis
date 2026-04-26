#import "@preview/lilaq:0.5.0" as lq

#set page(width: 6in, height: 5in, margin: 0.5in)
#set text(font: "New Computer Modern", size: 10pt)

// Gauss hypergeometric function ₂F₁(a, b; c; z) via series
#let hyper2f1(a, b, c, z) = {
  let s = 1.0
  let term = 1.0
  for n in range(1, 80) {
    term *= (a + n - 1) * (b + n - 1) / ((c + n - 1) * n) * z
    s += term
    if calc.abs(term) < 1e-15 { break }
  }
  s
}

// ITCM kernel parameters: a₁ = (μ+ν)/2 + 1, b₁ = (μ-ν)/2 + 1, c₁ = μ + 1
// Free case: μ = ν = 1 → a₁ = 2, b₁ = 1, c₁ = 2
// Dressed case: μ = 1, ν = 0.5 → a₁ = 1.75, b₁ = 1.25, c₁ = 2
// Strong dressing: μ = 1, ν = 0 → a₁ = 1.5, b₁ = 1.5, c₁ = 2

#let zs = lq.linspace(0, 0.95, num: 200)

#let ys_free = zs.map(z => hyper2f1(2, 1, 2, z))
#let ys_dressed = zs.map(z => hyper2f1(1.75, 1.25, 2, z))
#let ys_strong = zs.map(z => hyper2f1(1.5, 1.5, 2, z))

#align(center)[
  #text(weight: "bold", size: 12pt)[ITCM Kernel: Hypergeometric Factor $attach(, bl: 2) F_1 (a_1, b_1; c_1; xi)$]
]

#v(0.5em)

#lq.diagram(
  width: 4.5in,
  height: 3in,
  xlabel: $xi = y^2 slash x^2$,
  ylabel: $attach(, bl: 2) F_1 (a_1, b_1; c_1; xi)$,
  lq.plot(zs, ys_free, label: [$mu = nu = 1$ (free)]),
  lq.plot(zs, ys_dressed, label: [$mu = 1, nu = 0.5$ (dressed)]),
  lq.plot(zs, ys_strong, label: [$mu = 1, nu = 0$ (strong)]),
)

#v(1em)

#text(size: 9pt)[
  *Parameters:* $a_1 = (mu + nu) slash 2 + 1$, $b_1 = (mu - nu) slash 2 + 1$, $c_1 = mu + 1$.
  The universal Euler exponent $c_1 - a_1 - b_1 = -1$ produces a simple pole at $xi = 1$ in all three cases.
  As $mu eq.not nu$ increases (Hankel-order asymmetry), the kernel grows faster near $xi = 1$, strengthening the Green's function singularity that drives confinement.
]
