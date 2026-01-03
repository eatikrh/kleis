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

This is a sample paragraph demonstrating the document generation capabilities.
The system automatically applies the correct styling based on the template.

#heading(level: 1)[Methodology]

We employ rigorous mathematical methods.

#heading(level: 1)[Results]

Our findings demonstrate significant improvements.

#heading(level: 1)[Conclusion]

This work opens new avenues for research.
