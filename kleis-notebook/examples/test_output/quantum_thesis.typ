// Styling from template

#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
)


#set text(
  font: "New Computer Modern",
  size: 12pt,
  lang: "en",
)


#set par(
  justify: true,
  leading: 0.65em,
)


#set heading(numbering: "1.1")
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(1in)
  text(size: 16pt, weight: "bold")[Chapter #counter(heading).display()]
  v(0.3in)
  text(size: 14pt, weight: "bold")[#it.body]
  v(0.5in)
}
#show heading.where(level: 2): it => {
  v(0.3in)
  text(size: 12pt, weight: "bold")[#it.body]
  v(0.15in)
}


#set document(
  title: "Quantum Field Theory: A Computational Approach",
  author: "Dr. Jane Smith",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(size: 18pt, weight: "bold")[Quantum Field Theory: A Computational Approach]
    
    #v(0.5in)
    by
    #v(0.3in)
    #text(size: 14pt)[Dr. Jane Smith]
    
    #v(1in)
    Submitted to the Department of Physics #linebreak()
    in partial fulfillment of the requirements for the degree of
    #v(0.3in)
    #text(style: "italic")[DEGREE_NAME]
    #v(0.3in)
    at the
    #v(0.3in)
    #text(weight: "bold")[MASSACHUSETTS INSTITUTE OF TECHNOLOGY]
    #v(0.3in)
    January 2026
  ]
  
  #v(1fr)
  
  #align(left)[
    Thesis Supervisor: Prof. Richard Feynman
  ]
]


This thesis presents novel computational methods for quantum field theory.

#heading(level: 1)[Introduction] <sec:intro>

The foundation of modern physics rests on Einstein mass-energy equivalence:

$ E = m c^(2) $ <eq:einstein>

#heading(level: 1)[Computational Methods] <sec:methods>

For root-finding, we employ the quadratic formula:

$ x = frac(-b plus.minus sqrt(b^(2) - 4 a c), 2 a) $ <eq:quadratic>


#import "@preview/cetz:0.3.2": canvas, draw
#import "@preview/cetz-plot:0.1.1": plot

#figure(
  canvas({
    plot.plot(
      size: (10, 6),
      x-label: [Iteration],
      y-label: [Error (%)],
      x-tick-step: 10,
      y-tick-step: 5,
      {
        plot.add(
          ((0, 15), (10, 8), (20, 4), (30, 2), (40, 1), (50, 0.5)),
          style: (stroke: blue + 2pt),
          label: [Monte Carlo]
        )
        plot.add(
          ((0, 12), (10, 5), (20, 2), (30, 0.8), (40, 0.3), (50, 0.1)),
          style: (stroke: red + 2pt),
          label: [Variational]
        )
        plot.add(
          ((0, 10), (10, 3), (20, 0.8), (30, 0.2), (40, 0.05), (50, 0.01)),
          style: (stroke: green + 2pt),
          label: [Hybrid (Ours)]
        )
      }
    )
  }),
  caption: [Convergence comparison of computational methods]
) <fig:convergence>


#heading(level: 1)[Conclusion]

We have demonstrated the power of symbolic computation.
