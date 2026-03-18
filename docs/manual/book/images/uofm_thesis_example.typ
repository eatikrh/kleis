#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  numbering: "1",
)
#set text(
  font: "Times New Roman",
  size: 12pt,
  lang: "en",
)
#set par(
  justify: true,
  leading: 2em,
  first-line-indent: 0.5in,
)

// No indent after headings
#show heading: it => {
  it
  par(text(size: 0pt, ""))
}
#set heading(numbering: "1.1")

// Chapter headings (level 1) - 2 inch top margin
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(2in - 1in)
  text(size: 14pt, weight: "bold")[Chapter #counter(heading).display()]
  v(0.3in)
  text(size: 14pt, weight: "bold")[#it.body]
  v(0.5in)
}

// Section headings (level 2)
#show heading.where(level: 2): it => {
  v(0.3in)
  text(size: 12pt, weight: "bold")[#counter(heading).display() #it.body]
  v(0.2in)
}

// Subsection headings (level 3)
#show heading.where(level: 3): it => {
  v(0.2in)
  text(size: 12pt, weight: "bold", style: "italic")[#counter(heading).display() #it.body]
  v(0.15in)
}
#set figure(placement: auto)
#show figure.caption: it => {
  text(size: 10pt)[#it]
}


#page(margin: (top: 2in, bottom: 1in, left: 1in, right: 1in), numbering: none)[
  #align(center)[
    #text(weight: "bold")[Formal Verification of Neural Networks for Safety-Critical Autonomous Systems]
    
    #v(0.5in)
    by
    #v(0.3in)
    Alex Chen
    
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
      Professor Sarah Martinez, Chair #linebreak()
      Professor James Liu, Associate Professor of Electrical Engineering #linebreak()
      Professor Emily Watson, Associate Professor of Robotics #linebreak()
      Dr. Michael Park, Research Scientist, Google DeepMind
    ]
  ]
]


#page(numbering: none)[
  #align(center)[
    #v(1fr)
    Alex Chen
    #v(0.2in)
    alexchen\@umich.edu
    #v(0.2in)
    0000-0002-1234-5678 iD: 0000-0002-1234-5678
    #v(0.5in)
    © Alex Chen 2026
    #v(1fr)
  ]
]


#set page(numbering: "i")
#counter(page).update(2)


#page[
  #v(2in)
  #align(center)[
    #text(style: "italic")[To my grandmother, who always believed in the power of education.]
  ]
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Acknowledgments]
  ]
  #v(0.3in)
  
  I am deeply grateful to my advisor, Professor Sarah Martinez, for her guidance, patience, and unwavering support throughout my doctoral journey. Her insights have shaped not only this dissertation but also my approach to research.

I thank my committee members for their valuable feedback and challenging questions that strengthened this work. Professor Liu's expertise in formal methods, Professor Watson's perspective on robotics applications, and Dr. Park's industry insights were invaluable.

I am grateful to my labmates in the Verified AI Lab for countless discussions, debugging sessions, and moral support. Special thanks to the Rackham Graduate School for fellowship support.

Finally, I thank my family for their unconditional love and encouragement. To my parents, who instilled in me the value of education, and to my partner Jamie, whose patience and support made this possible.
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Preface]
  ]
  #v(0.3in)
  
  This dissertation represents research conducted at the University of Michigan from 2022 to 2026. Portions of Chapter 3 were published in the Proceedings of the International Conference on Computer Aided Verification (CAV 2024). Chapter 4 contains material from a paper accepted to the IEEE Symposium on Security and Privacy (S&P 2025). Chapter 5 is based on ongoing collaboration with Google DeepMind and will be submitted for publication.
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Table of Contents]
  ]
  #v(0.3in)
  #outline(
    title: none,
    depth: 3,
    indent: 1.5em,
  )
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[List of Tables]
  ]
  #v(0.3in)
  #outline(
    title: none,
    target: figure.where(kind: table),
  )
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[List of Figures]
  ]
  #v(0.3in)
  #outline(
    title: none,
    target: figure.where(kind: image),
  )
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[List of Appendices]
  ]
  #v(0.3in)
  // Will be populated by appendix entries
]


#page[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Abstract]
  ]
  #v(0.3in)
  
  Autonomous systems powered by neural networks are increasingly deployed in safety-critical applications, from self-driving vehicles to medical diagnosis systems. However, the black-box nature of deep neural networks poses significant challenges for formal verification. This dissertation presents novel techniques for verifying safety properties of neural networks used in autonomous systems. We introduce three main contributions: (1) a scalable abstraction refinement framework for neural network verification that handles networks with millions of parameters, (2) a compositional verification approach that decomposes complex neural network pipelines into verifiable components, and (3) runtime monitoring techniques that provide safety guarantees even when complete verification is intractable. Our experimental evaluation on autonomous driving benchmarks demonstrates that these techniques can verify properties of production-scale neural networks within practical time bounds, achieving verification speedups of 100-1000x compared to existing methods while maintaining soundness guarantees.
]


#set page(numbering: "1")
#counter(page).update(1)


= Introduction

The deployment of neural networks in safety-critical autonomous systems represents one of the most significant technological shifts of the 21st century. From autonomous vehicles navigating complex urban environments to medical AI systems assisting in diagnosis, these systems make decisions that directly impact human safety. Yet the complexity and opacity of neural networks pose fundamental challenges for ensuring their safe operation.

Traditional software verification techniques, developed over decades of research, rely on the structured nature of programs written in conventional programming languages. These techniques exploit properties like compositionality, abstraction, and modularity to reason about system behavior. Neural networks, in contrast, encode their behavior in millions of learned parameters, making traditional verification approaches largely inapplicable.

This dissertation addresses the fundamental question: How can we provide formal safety guarantees for autonomous systems that rely on neural network components? We approach this challenge from three complementary angles: scalable verification algorithms, compositional reasoning frameworks, and runtime monitoring techniques.

== Motivation and Problem Statement

The motivation for this work stems from a critical gap between the capabilities and safety guarantees of modern AI systems. Consider an autonomous vehicle's perception system, which uses deep neural networks to detect pedestrians, vehicles, and road signs. A single misclassification—confusing a pedestrian for a shadow, or misreading a stop sign—can have catastrophic consequences.

Current approaches to neural network safety fall into two categories: testing and verification. Testing, while practical, provides probabilistic rather than formal guarantees. Verification, while providing formal guarantees, has historically been limited to small networks due to computational complexity.

The central problem this dissertation addresses is: How can we bridge this gap to provide meaningful safety guarantees for production-scale neural networks?

== Research Contributions

This dissertation makes three main contributions to the field of neural network verification:

1. Scalable Abstraction Refinement (Chapter 3): We present a novel verification framework based on counterexample-guided abstraction refinement (CEGAR) adapted for neural networks. Our key insight is that neural network structure provides natural abstractions that can be systematically refined.

2. Compositional Verification (Chapter 4): We develop techniques for decomposing complex autonomous system pipelines into verifiable components, enabling verification of systems that would be intractable to verify monolithically.

3. Runtime Monitoring with Guarantees (Chapter 5): We introduce monitoring techniques that combine lightweight runtime checks with pre-computed verification results to provide safety guarantees even when complete verification is infeasible.

= Background and Related Work

This chapter provides the technical background necessary to understand our contributions and surveys related work in neural network verification, formal methods for autonomous systems, and runtime monitoring.

== Neural Network Fundamentals

A feedforward neural network maps inputs to outputs through alternating linear transformations and nonlinear activation functions. For a network with L layers, the output is computed by composing layer operations, where each layer applies a weight matrix, adds a bias vector, and applies an activation function (typically ReLU, sigmoid, or tanh).

== Formal Verification Foundations

Formal verification provides mathematical proofs that a system satisfies its specification. For neural networks, we focus on safety properties of the form:

∀ x ∈ P : f(x) ∈ Q

where P is a precondition (valid inputs) and Q is a postcondition (safe outputs). The verification problem is to prove or disprove this universal statement.

= Scalable Abstraction Refinement for Neural Networks

This chapter presents our first contribution: a scalable abstraction refinement framework for neural network verification. The key insight is that neural networks admit natural abstractions based on neuron groupings, and these abstractions can be systematically refined to achieve verification precision while maintaining scalability.

$ alpha(f) = brace.l y : exists x in P . f(x) = y brace.r $ <eq:abstraction>

$ alpha_k (f) supset.eq alpha_(k+1) (f) supset.eq dots.h supset.eq op("range")(f) $ <eq:refinement>

$ alpha(f) inter Q^c = nothing => forall x in P : f(x) in Q $ <eq:soundness>

#figure(
  table(
      columns: (auto, auto, auto, auto),
      inset: 8pt,
      align: center,
      [*Network*], [*Layers*], [*Parameters*], [*Domain*],
      [ACAS Xu], [6], [300], [Collision Avoidance],
      [MNIST-FC], [4], [100K], [Digit Classification],
      [ImageNet-CNN], [50], [25M], [Object Detection],
      [DriveNet], [34], [12M], [Autonomous Driving],
    ),
  caption: [Benchmark neural networks used in evaluation]
) <tab:benchmarks>

#figure(
  [
      #rect(width: 80%, stroke: 1pt)[
        #align(center)[
          #stack(dir: ttb, spacing: 1em,
            rect(fill: blue.lighten(80%), inset: 10pt)[Neural Network + Property],
            [↓],
            rect(fill: green.lighten(80%), inset: 10pt)[Initial Abstraction α₀],
            [↓],
            rect(fill: yellow.lighten(60%), inset: 10pt)[Abstract Verification],
            stack(dir: ltr, spacing: 2em,
              [Safe? Yes → ✓],
              [No → Refine → Loop]
            )
          )
        ]
      ]
    ],
  caption: [Overview of the abstraction refinement verification framework]
) <fig:architecture>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Verification Speedup vs Network Size],
        xlabel: [Network Parameters (millions)],
        ylabel: [Speedup (x)],
        lq.plot((0.1, 0.5, 1, 5, 10, 25), (150, 120, 100, 80, 60, 40), mark: "o"),
      )
    },
  caption: [Verification speedup across different network sizes]
) <fig:speedup>

#figure(
  table(
      columns: (auto, auto, auto, auto),
      inset: 8pt,
      align: center,
      [*Benchmark*], [*Baseline*], [*Our Method*], [*Speedup*],
      [ACAS Property 1], [1,240], [12], [103x],
      [ACAS Property 2], [3,600], [45], [80x],
      [MNIST Robustness], [timeout], [890], [>4x],
      [DriveNet Safety], [timeout], [2,340], [>1.5x],
    ),
  caption: [Verification times (seconds) comparing baseline and our approach]
) <tab:results>

= Compositional Verification of Neural Network Pipelines

Autonomous systems rarely consist of a single neural network. Instead, they combine multiple neural network components with traditional software in complex pipelines. This chapter presents our compositional verification framework that enables verification of such systems by decomposing them into independently verifiable components.

$ f = f_n circle.stroked.tiny f_(n-1) circle.stroked.tiny dots.h circle.stroked.tiny f_1 $ <eq:composition>

$ Q_i = P_(i+1) quad "(interface contracts)" $ <eq:interface>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Component Verification Times],
        xlabel: [Component],
        ylabel: [Time (seconds)],
        lq.bar((1, 2, 3, 4, 5), (120, 340, 89, 456, 234)),
      )
    },
  caption: [Component verification times for an autonomous driving pipeline]
) <fig:pipeline>

= Runtime Monitoring with Safety Guarantees

Complete verification of neural networks is not always feasible within practical time bounds, especially for very large networks or complex properties. This chapter presents runtime monitoring techniques that provide safety guarantees by combining lightweight runtime checks with pre-computed verification certificates.

$ M(x) = cases("safe" &"if" x in V, "check" &"if" x in.not V) $ <eq:monitor>

#figure(
  {
      import "@preview/lilaq:0.5.0" as lq
      lq.diagram(
        width: 10cm,
        title: [Monitoring Overhead],
        xlabel: [Input Dimension],
        ylabel: [Overhead (ms)],
        legend: (position: right + top),
        lq.plot((100, 500, 1000, 5000, 10000), (0.1, 0.3, 0.5, 1.2, 2.1), mark: "o", label: [Our Method]),
        lq.plot((100, 500, 1000, 5000, 10000), (1.5, 4.2, 8.9, 45, 120), mark: "x", label: [Baseline]),
      )
    },
  caption: [Runtime monitoring overhead across different input sizes]
) <fig:monitor>

= Conclusion and Future Work

This dissertation has presented three complementary approaches to providing safety guarantees for neural networks in autonomous systems: scalable abstraction refinement, compositional verification, and runtime monitoring. Together, these techniques enable practical verification of production-scale neural networks while maintaining formal soundness.

Our experimental evaluation demonstrates that these techniques achieve verification speedups of 100-1000x compared to existing methods, making it practical to verify networks with millions of parameters. The compositional approach enables verification of complex autonomous system pipelines that would be intractable to verify monolithically. Runtime monitoring provides a practical fallback when complete verification is infeasible.

Future work includes extending these techniques to recurrent neural networks and transformers, developing verification methods for reinforcement learning policies, and integrating verification into neural network training pipelines.

#pagebreak()
// Reset heading counter for appendices
#counter(heading).update(0)
#set heading(numbering: "A.1")

#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  v(2in - 1in)
  text(size: 14pt, weight: "bold")[Appendix #counter(heading).display()]
  v(0.3in)
  text(size: 14pt, weight: "bold")[#it.body]
  v(0.5in)
}
= Proof of Soundness Theorem

This appendix provides the complete proof of Theorem 3.1 (Soundness of Abstraction Refinement).

Theorem 3.1: If the abstraction refinement procedure terminates with result SAFE, then the original verification property holds.

Proof: By induction on the refinement steps...

= Benchmark Details

This appendix provides detailed specifications of all benchmark neural networks and properties used in the experimental evaluation.

ACAS Xu Networks: The Airborne Collision Avoidance System for unmanned aircraft (ACAS Xu) consists of 45 neural networks, each with 6 hidden layers and 50 neurons per layer...

#pagebreak()
#heading(numbering: none)[Bibliography]
[katz2017] Katz, G., Barrett, C., Dill, D. L., Julian, K., & Kochenderfer, M. J. (2017). Reluplex: An efficient SMT solver for verifying deep neural networks. CAV 2017.

[gehr2018] Gehr, T., Mirman, M., Drachsler-Cohen, D., Tsankov, P., Chaudhuri, S., & Vechev, M. (2018). AI2: Safety and robustness certification of neural networks with abstract interpretation. IEEE S&P 2018.

[wang2021] Wang, S., Pei, K., Whitehouse, J., Yang, J., & Jana, S. (2021). Efficient formal safety analysis of neural networks. NeurIPS 2018.

[huang2020] Huang, X., Kwiatkowska, M., Wang, S., & Wu, M. (2020). Safety verification of deep neural networks. CAV 2017.

[singh2019] Singh, G., Gehr, T., Püschel, M., & Vechev, M. (2019). An abstract domain for certifying neural networks. POPL 2019.