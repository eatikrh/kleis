#set document(
  title: "Test Document",
  author: "Test Author",
)

#set page(paper: "us-letter", margin: 1in)
#set text(size: 11pt)

#align(center)[
  #text(size: 20pt, weight: "bold")[Test Document]
]

#align(center)[Test Author]

#heading(level: 1)[Introduction]

This document tests the rendering pipeline.

$ E = m c^(2) $ <eq:einstein>

#heading(level: 1)[Conclusion]

The test was successful.
