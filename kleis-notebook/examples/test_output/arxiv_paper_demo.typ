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
  title: "Efficient Transformers",
)

#align(center)[
  #text(size: 20pt, weight: "bold")[Efficient Transformers]
]


#block(fill: luma(245), inset: 10pt, radius: 4pt, width: 100%)[
  #text(weight: "bold")[Abstract.]
  A novel attention mechanism.
]


#heading(level: 1)[Introduction]

Transformers have revolutionized NLP.
