// Styling from template

#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
)


#set text(
  font: "Times New Roman",
  size: 12pt,
  lang: "en",
)


#set par(
  justify: true,
  leading: 1.5em,
  first-line-indent: 0.5in,
)


#set heading(numbering: "1.1")
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(2in - 1in)  // 2 inch top margin for chapter starts
  text(size: 14pt, weight: "bold")[Chapter #counter(heading).display() #it.body]
  v(0.5in)
}
#show heading.where(level: 2): it => {
  v(0.3in)
  text(size: 12pt, weight: "bold")[#it.body]
  v(0.2in)
}


#set document(
  title: "Advances in Quantum Error Correction",
  author: "Emily Chen",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(weight: "bold")[Advances in Quantum Error Correction]
    
    #v(0.5in)
    by
    #v(0.3in)
    Emily Chen
    
    #v(1in)
    A dissertation submitted in partial fulfillment \\
    of the requirements for the degree of \\
    Doctor of Philosophy \\
    (Physics) \\
    in The University of Michigan \\
    2025
  ]
  
  #v(1fr)
  
  #align(left)[
    Doctoral Committee:
    #v(0.1in)
    #pad(left: 0.5in)[
      Professor Robert Williams, Chair \\
      Professor Sarah Miller \\
      Professor David Brown
    ]
  ]
]


This dissertation presents novel approaches to quantum error correction in noisy intermediate-scale quantum devices.

#heading(level: 1)[Introduction]

Quantum computing promises exponential speedups for certain problems.

#heading(level: 1)[Methods]

We propose a new class of stabilizer codes.

#heading(level: 1)[Conclusion]

We have demonstrated practical quantum error correction.
