// Styling from template

#set page(
  paper: "us-letter",
  margin: 0.75in,
)


#set text(
  font: "New Computer Modern",
  size: 10pt,
  lang: "en",
)


#set par(
  justify: true,
  first-line-indent: 1em,
  leading: 0.55em,
)


#set heading(numbering: "1.")
#show heading.where(level: 1): it => {
  v(0.4in)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.15in)
}
#show heading.where(level: 2): it => {
  v(0.2in)
  text(size: 11pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.1in)
}


#set document(
  title: "Template-Driven Document Generation: A New Paradigm",
  author: "Alice Chen and Bob Davis",
)

#align(center)[
  #text(size: 20pt, weight: "bold")[Template-Driven Document Generation: A New Paradigm]
]

#align(center)[Alice Chen and Bob Davis]


#block(fill: luma(245), inset: 10pt, radius: 4pt, width: 100%)[
  #text(weight: "bold")[Abstract.]
  ABSTRACT_TEXT
]


#heading(level: 1)[Introduction]

This is a sample paragraph demonstrating the document generation capabilities.
The system automatically applies the correct styling based on the template.

#heading(level: 1)[Methods]

Our template system is described.

#heading(level: 1)[Results]

Experimental results show promise.

#heading(level: 1)[Conclusion]

This approach simplifies document creation.
