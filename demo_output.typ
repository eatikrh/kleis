#set document(
  title: "KleisDoc Demo",
  author: "Kleis Project",
)

#set page(paper: "us-letter", margin: 1in)
#set text(size: 11pt)

#align(center)[
  #text(size: 20pt, weight: "bold")[KleisDoc Demo]
]

#align(center)[Kleis Project]

#heading(level: 1)[Abstract]
This document was generated programmatically using the new KleisDoc Python API, demonstrating the integration between Jupyter notebooks and the Kleis document system.

#heading(level: 1)[Introduction]


KleisDoc is a format-agnostic document management system that bridges Jupyter notebook workflows with publishable documents. It supports:

- Flexible metadata management
- Hierarchical section organization  
- Equation storage with re-editable ASTs
- Figure management with regenerable Kleis plots


#heading(level: 1)[Key Features]


*Equations* can be stored with their EditorNode AST, allowing them to be re-opened in the Equation Editor for modification across sessions.

*Figures* can include Kleis plotting code, enabling regeneration when data changes.

*Templates* are loaded from external .kleis files, making the system extensible.


#heading(level: 1)[Conclusion]

This demonstrates the KleisDoc API generating real Typst output.
