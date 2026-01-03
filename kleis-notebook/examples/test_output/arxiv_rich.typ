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
  title: "Template-Driven Document Generation",
  author: "Alice Chen and Bob Davis",
)

#align(center)[
  #text(size: 20pt, weight: "bold")[Template-Driven Document Generation]
]

#align(center)[Alice Chen and Bob Davis]


#block(fill: luma(245), inset: 10pt, radius: 4pt, width: 100%)[
  #text(weight: "bold")[Abstract.]
  ABSTRACT_TEXT
]


#heading(level: 1)[Introduction]

Consider Einstein's famous equation:

$ E = m c^(2) $ <eq:einstein>

#heading(level: 1)[Methods]

Root finding uses the quadratic formula:

$ x = frac(-b plus.minus sqrt(b^(2) - 4 a c), 2 a) $ <eq:quadratic>

#heading(level: 1)[Conclusion]

Template-driven generation works.
