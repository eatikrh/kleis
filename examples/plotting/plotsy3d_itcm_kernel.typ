#import "@preview/plotsy-3d:0.2.1": *

#set page(width: 7in, height: 10in, margin: 0.5in)
#set text(font: "New Computer Modern", size: 10pt)

// ₂F₁(a, b; c; z) via truncated series
#let hyper2f1(a, b, c, z) = {
  if z == 0 { return 1.0 }
  let s = 1.0
  let term = 1.0
  for n in range(1, 80) {
    term *= (a + n - 1) * (b + n - 1) / ((c + n - 1) * n) * z
    s += term
    if calc.abs(term) < 1e-12 { break }
  }
  s
}

// Dressed case: μ = 1, ν = 0.5
#let a1 = 1.75
#let b1 = 1.25
#let c1 = 2.0

// Panel 1: Raw ITCM kernel — the full ₂F₁ with its pole
#let raw-kernel(x, y) = {
  let X = x + 1.0
  let Y = y + 1.0
  let xi = (Y * Y) / (X * X)
  if xi >= 0.90 { return 10.0 }
  let val = hyper2f1(a1, b1, c1, xi)
  if val > 10.0 { return 10.0 }
  val
}

// Panel 2: Connection coefficient R(x,y) = ₂F₁(a,b;c;ξ) × (1-ξ)
// Removes the universal pole, reveals the pure dressing structure
#let connection-coeff(x, y) = {
  let X = x + 1.0
  let Y = y + 1.0
  let xi = (Y * Y) / (X * X)
  if xi >= 0.98 { return 2.5 }
  let f = hyper2f1(a1, b1, c1, xi)
  let val = f * (1.0 - xi)
  if val > 2.5 { return 2.5 }
  if val < 0 { return 0 }
  val
}

// Heat color: deep blue → cyan → yellow → red
#let heat-color(x, y, z, x-lo, x-hi, y-lo, y-hi, z-lo, z-hi) = {
  let range = z-hi - z-lo
  if range == 0 { return rgb(20, 40, 120) }
  let t = (z - z-lo) / range
  if t < 0 { t = 0 }
  if t > 1 { t = 1 }
  if t < 0.25 {
    let s = t / 0.25
    rgb(int(20 + s * 0), int(40 + s * 140), int(120 + s * 80))
  } else if t < 0.5 {
    let s = (t - 0.25) / 0.25
    rgb(int(20 + s * 60), int(180 + s * 20), int(200 - s * 120))
  } else if t < 0.75 {
    let s = (t - 0.5) / 0.25
    rgb(int(80 + s * 170), int(200 - s * 10), int(80 - s * 60))
  } else {
    let s = (t - 0.75) / 0.25
    rgb(int(250), int(190 - s * 160), int(20 - s * 20))
  }
}

// Cool-warm for connection coefficient: purple → steel blue → sage → gold
#let shape-color(x, y, z, x-lo, x-hi, y-lo, y-hi, z-lo, z-hi) = {
  let range = z-hi - z-lo
  if range == 0 { return rgb(60, 20, 100) }
  let t = (z - z-lo) / range
  if t < 0 { t = 0 }
  if t > 1 { t = 1 }
  if t < 0.33 {
    let s = t / 0.33
    rgb(int(60 + s * 10), int(20 + s * 80), int(100 + s * 60))
  } else if t < 0.66 {
    let s = (t - 0.33) / 0.33
    rgb(int(70 + s * 60), int(100 + s * 60), int(160 - s * 60))
  } else {
    let s = (t - 0.66) / 0.34
    rgb(int(130 + s * 90), int(160 + s * 40), int(100 - s * 60))
  }
}

// ══════════════════════════════════════════════════════════════

#align(center)[
  #text(weight: "bold", size: 15pt)[Kernel Decomposition: Pole $times$ Shape]
  #v(0.3em)
  #text(size: 10pt)[
    $K(x, y) = underbrace(attach(, bl: 2) F_1 (a_1, b_1; c_1; xi), "full kernel")
    = underbrace(1 slash (1 - xi), "universal pole") times
    underbrace(R(xi), "connection coefficient")$
  ]
  #v(0.2em)
  #text(size: 9pt)[
    $xi = y^2 slash x^2$, #h(0.8em)
    dressed case $mu = 1, nu = 0.5$, #h(0.8em)
    $R(xi) = attach(, bl: 2) F_1 (a_1, b_1; c_1; xi) dot (1 - xi)$
  ]
]

#v(0.6em)

// ══════════════════════════════════════════════════════════════
//  Panel 1: Raw kernel (with pole)
// ══════════════════════════════════════════════════════════════

#align(center)[
  #text(weight: "bold", size: 11pt)[
    Full Kernel: $attach(, bl: 2) F_1 (1.75, 1.25; 2; xi)$
  ]
  #v(0.1em)
  #text(size: 9pt, fill: rgb(120, 120, 120))[
    Diverges at $xi = 1$ — the Green's function diagonal
  ]
]

#v(0.3em)

#let sf1 = 0.16
#plot-3d-surface(
  raw-kernel,
  color-func: heat-color,
  subdivisions: 2,
  subdivision-mode: "increase",
  scale-dim: (0.3 * sf1, 0.3 * sf1, 0.07 * sf1),
  xdomain: (0, 10),
  ydomain: (0, 10),
  axis-step: (2, 2, 2),
  dot-thickness: 0.05em,
  front-axis-thickness: 0.1em,
  front-axis-dot-scale: (0.05, 0.05),
  rear-axis-dot-scale: (0.08, 0.08),
  rear-axis-text-size: 0.5em,
  axis-label-size: 1.5em,
  axis-labels: ($x$, $y$, $K$),
  rotation-matrix: ((-2, 2, 4), (0, -1, 0)),
  xyz-colors: (red, green, blue),
)

#v(1em)

// ══════════════════════════════════════════════════════════════
//  Panel 2: Connection coefficient (pole removed)
// ══════════════════════════════════════════════════════════════

#align(center)[
  #text(weight: "bold", size: 11pt)[
    Connection Coefficient: $R(xi) = attach(, bl: 2) F_1 dot (1 - xi)$
  ]
  #v(0.1em)
  #text(size: 9pt, fill: rgb(120, 120, 120))[
    Pole removed — pure dressing structure remains
  ]
]

#v(0.3em)

#plot-3d-surface(
  connection-coeff,
  color-func: shape-color,
  subdivisions: 2,
  subdivision-mode: "increase",
  scale-dim: (0.3 * sf1, 0.3 * sf1, 0.25 * sf1),
  xdomain: (0, 10),
  ydomain: (0, 10),
  axis-step: (2, 2, 1),
  dot-thickness: 0.05em,
  front-axis-thickness: 0.1em,
  front-axis-dot-scale: (0.05, 0.05),
  rear-axis-dot-scale: (0.08, 0.08),
  rear-axis-text-size: 0.5em,
  axis-label-size: 1.5em,
  axis-labels: ($x$, $y$, $R$),
  rotation-matrix: ((-2, 2, 4), (0, -1, 0)),
  xyz-colors: (red, green, blue),
)

#v(0.8em)

#line(length: 100%, stroke: 0.5pt + rgb(180, 180, 180))
#v(0.4em)

#text(size: 9pt)[
  *Interpretation.* The full kernel $K(x,y)$ (top) diverges along the diagonal $y = x$ — the universal $1 slash (1-xi)$ Cauchy pole from the Euler exponent $c - a - b = -1$. Every Sturm--Liouville Green's function has this pole; it carries no spectral information.

  The connection coefficient $R(xi)$ (bottom) is what remains after dividing out the universal pole. This finite, smooth surface encodes the *dressing*: how Hankel-order asymmetry ($mu eq.not nu$) redistributes spectral weight. Near $xi = 0$ (far from diagonal), $R arrow 1$ — the kernel is indistinguishable from undressed. Near $xi = 1$ (the diagonal), $R$ rises to a finite limit determined by the Gamma ratio $Gamma(c) Gamma(|c - a - b|) slash (Gamma(c-a) Gamma(c-b))$ — the Euler Beta function evaluated at the connection parameters. It is this *shape*, not the pole, that determines whether the spectrum has a gap.
]
