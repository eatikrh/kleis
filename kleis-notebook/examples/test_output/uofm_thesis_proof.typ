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
  title: "Advanced Document Generation Systems",
  author: "John Smith",
)


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(weight: "bold")[Advanced Document Generation Systems]
    
    #v(0.5in)
    by
    #v(0.3in)
    John Smith
    
    #v(1in)
    A dissertation submitted in partial fulfillment #linebreak()
    of the requirements for the degree of #linebreak()
    Doctor of Philosophy #linebreak()
    (Computer Science and Engineering) #linebreak()
    in The University of Michigan #linebreak()
    2026
  ]
  
  #v(1fr)
  
  #align(left)[
    Doctoral Committee:
    #v(0.1in)
    #pad(left: 0.5in)[
      Prof. Alice Johnson, Chair #linebreak()
      Prof. Bob Williams #linebreak()
      Dr. Carol Brown #linebreak()
      Prof. David Lee
    ]
  ]
]


This dissertation presents a novel document generation framework.

#heading(level: 1)[Introduction]

This is a sample paragraph demonstrating the document generation capabilities.
The system automatically applies the correct styling based on the template.

#heading(level: 1)[Background]

Prior work in this area is reviewed.

#heading(level: 1)[Methodology]

Our approach uses template-driven generation.

#heading(level: 1)[Conclusion]

We have demonstrated the effectiveness of our system.
