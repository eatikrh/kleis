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
  title: "Machine Learning for Autonomous Systems",
  author: "James Park",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(size: 18pt, weight: "bold")[Machine Learning for Autonomous Systems]
    
    #v(0.5in)
    by
    #v(0.3in)
    #text(size: 14pt)[James Park]
    
    #v(1in)
    Submitted to the EECS #linebreak()
    in partial fulfillment of the requirements for the degree of
    #v(0.3in)
    #text(style: "italic")[DEGREE_NAME]
    #v(0.3in)
    at the
    #v(0.3in)
    #text(weight: "bold")[MASSACHUSETTS INSTITUTE OF TECHNOLOGY]
    #v(0.3in)
    May 2025
  ]
  
  #v(1fr)
  
  #align(left)[
    Thesis Supervisor: Professor Maria Santos
  ]
]


Novel ML algorithms for autonomous vehicle navigation.

#heading(level: 1)[Introduction]

This is the introduction.

#heading(level: 1)[Background]

This is the background.
