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
  title: "Test",
  author: "Test Author",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(weight: "bold")[#upper(Test)]
    
    #v(0.5in)
    by
    #v(0.3in)
    #text[Test Author]
    
    #v(1in)
    A dissertation submitted in partial fulfillment \\
    of the requirements for the degree of \\
    #text[] \\
    (#text[]) \\
    in The University of Michigan \\
    #text[]
  ]
  
  #v(1fr)
  
  #align(left)[
    Doctoral Committee:
    #v(0.1in)
    #pad(left: 0.5in)[
      , Chair \\
      
    ]
  ]
]


#heading(level: 1)[Introduction]

Test
