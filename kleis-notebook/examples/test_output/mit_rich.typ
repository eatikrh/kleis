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
  title: "Symbolic Computation for Modern Physics",
  author: "Jane Doe",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(size: 18pt, weight: "bold")[Symbolic Computation for Modern Physics]
    
    #v(0.5in)
    by
    #v(0.3in)
    #text(size: 14pt)[Jane Doe]
    
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


This thesis presents novel approaches to symbolic computation in physics.

#heading(level: 1)[Introduction] <sec:intro>

This thesis explores symbolic computation in physics apply to Einstein equation:

$ E = m c^(2) $ <eq:einstein>

#heading(level: 1)[Methods] <sec:methods>

We employ the quadratic formula for root finding:

$ x = frac(-b plus.minus sqrt(b^(2) - 4 a c), 2 a) $ <eq:quadratic>

Our results are summarized below:

#heading(level: 1)[Conclusion]

This work demonstrates the power of symbolic methods.
