// No template - using minimal defaults
#set page(margin: 1in)
#set text(size: 11pt)

#set document(
  title: "Neural Network Approaches to Quantum Error Correction: Theory, Implementation, and Experimental Validation",
)

#align(center)[
  #text(size: 20pt, weight: "bold")[Neural Network Approaches to Quantum Error Correction: Theory, Implementation, and Experimental Validation]
]


    Quantum computing promises revolutionary advances in computational capability, 
    yet remains fundamentally limited by decoherence and gate errors. This dissertation 
    presents a comprehensive framework for neural network-based quantum error correction 
    (NNQEC) that achieves state-of-the-art performance on both simulated and experimental 
    quantum systems.

    We begin by establishing the theoretical foundations of quantum error correction, 
    introducing a novel mathematical framework that unifies stabilizer codes with 
    machine learning optimization objectives. Our key theoretical contribution is the 
    proof that neural decoders can achieve the maximum likelihood decoding threshold 
    for any stabilizer code, resolving a long-standing open question in the field.

    The dissertation then presents three major algorithmic innovations: (1) a 
    transformer-based architecture for syndrome decoding that achieves 99.7% accuracy 
    on the surface code, (2) a reinforcement learning approach to adaptive error 
    correction that outperforms static decoders by 23%, and (3) a variational quantum 
    eigensolver enhanced by neural error mitigation that enables chemistry simulations 
    on near-term devices.

    We validate our methods through extensive experiments on IBM and Google quantum 
    processors, demonstrating practical improvements in logical error rates. Our 
    neural decoder reduces the physical qubit overhead required for fault-tolerant 
    computation by approximately 40%, representing a significant step toward 
    practical quantum advantage.

    The dissertation concludes with a roadmap for scaling these techniques to 
    larger quantum systems and discusses implications for the timeline of 
    fault-tolerant quantum computing.
    

#heading(level: 1)[Dedication]


To my parents, Wei and Lin Chen, who taught me that the pursuit of knowledge 
is the highest calling. To my partner, Michael, whose unwavering support made 
this journey possible. And to my advisor, Professor Sarah Williams, who showed 
me what it means to be a scientist.


#heading(level: 1)[Acknowledgments]


This dissertation would not have been possible without the support and guidance 
of many individuals. First and foremost, I am deeply grateful to my advisor, 
Professor Sarah Williams, whose brilliance, patience, and mentorship have shaped 
me as a researcher. Her vision for the intersection of machine learning and 
quantum computing inspired this entire body of work.

I thank my committee members—Professors David Liu, Jennifer Martinez, and 
Robert Thompson—for their insightful feedback and challenging questions that 
strengthened this research. The quantum computing group at Michigan has been 
an incredible intellectual home, and I am grateful to my lab mates for countless 
discussions and collaborations.

This research was supported by the National Science Foundation (Grant No. 
PHY-2112893), the Department of Energy (DE-SC0019380), and the Google Quantum 
AI Research Award. I am grateful for access to IBM Quantum and Google Quantum 
AI hardware through their academic programs.

Finally, I thank my family for their love and encouragement. My parents' 
sacrifices made my education possible, and my partner Michael's patience 
during late nights and weekends of writing kept me going.


#heading(level: 1)[Introduction] <ch:intro>


The quest for practical quantum computing stands at a critical juncture. While 
remarkable progress has been made in building quantum processors with increasing 
numbers of qubits, the fundamental challenge of quantum error correction remains 
the primary barrier to achieving computational advantage for practical problems. 
This dissertation addresses this challenge through the lens of machine learning, 
developing neural network approaches that push the boundaries of what is possible 
with near-term quantum devices.


#heading(level: 2)[Motivation and Context] <sec:motivation>


Quantum computers exploit the principles of superposition and entanglement to 
perform computations that would be intractable on classical machines. The promise 
of exponential speedups for problems in cryptography, optimization, and quantum 
simulation has driven billions of dollars of investment from governments and 
industry. Yet current quantum processors, often termed Noisy Intermediate-Scale 
Quantum (NISQ) devices, are fundamentally limited by errors.

Every quantum gate introduces errors at rates of approximately 0.1-1%, and 
quantum states decohere on timescales of microseconds to milliseconds. For 
algorithms requiring millions of gates, the accumulated errors render the 
computation meaningless without active error correction. The standard approach 
to quantum error correction (QEC) encodes logical qubits into many physical 
qubits, using redundancy to detect and correct errors.


$ p_("logical") = A dot p_("physical")^(floor.l (d+1)/2 floor.r) $ <error_rate>


Equation 1 shows the logical error rate as a function of the physical error rate 
for a distance-d code, where A is a constant depending on the code. The key 
insight is that below a threshold physical error rate, increasing the code 
distance exponentially suppresses logical errors.


#heading(level: 2)[Challenges in Quantum Error Correction] <sec:challenges>


While the theory of quantum error correction is well-established, practical 
implementation faces several challenges:

1. *Decoding Complexity*: Optimal decoding of quantum error correction codes 
   is computationally hard in general. Maximum likelihood decoding is NP-hard 
   for most codes of interest, necessitating approximate algorithms.

2. *Real-time Requirements*: Error syndromes must be processed and corrections 
   applied within the coherence time of the quantum system, typically 
   microseconds. This places severe constraints on decoder latency.

3. *Correlated Errors*: Real quantum devices exhibit correlated errors due to 
   crosstalk, cosmic rays, and other environmental factors. Standard decoders 
   assume independent errors and perform poorly on correlated noise.

4. *Overhead*: Current estimates suggest that millions of physical qubits 
   may be required for useful fault-tolerant computation, far exceeding the 
   capabilities of near-term devices.

This dissertation addresses these challenges through neural network approaches 
that learn to decode efficiently, operate in real-time, adapt to correlated 
noise, and reduce qubit overhead.


#heading(level: 2)[Contributions] <sec:contributions>


This dissertation makes the following contributions to the field:

*Theoretical Contributions:*
- A unified mathematical framework connecting stabilizer codes with neural 
  network optimization (Chapter 2)
- Proof that neural decoders can achieve maximum likelihood performance for 
  any stabilizer code under mild assumptions (Theorem 2.3)
- Analysis of the sample complexity of learning quantum decoders (Chapter 3)

*Algorithmic Contributions:*
- TransformerQEC: A transformer-based architecture for syndrome decoding that 
  achieves state-of-the-art accuracy on the surface code (Chapter 4)
- AdaptiveQEC: A reinforcement learning approach to adaptive error correction 
  that outperforms static decoders on time-varying noise (Chapter 5)
- NeuralVQE: A variational quantum eigensolver with neural error mitigation 
  for near-term chemistry simulations (Chapter 6)

*Experimental Contributions:*
- Demonstration of neural decoding on IBM and Google quantum processors, 
  achieving practical improvements in logical error rates (Chapter 7)
- Open-source implementation of all algorithms, enabling reproducibility 
  and further research (Appendix A)


#heading(level: 2)[Dissertation Outline] <sec:outline>


The remainder of this dissertation is organized as follows:

*Chapter 2* establishes the theoretical foundations, reviewing quantum error 
correction and introducing our mathematical framework.

*Chapter 3* develops the theory of neural quantum decoding, including 
expressiveness results and sample complexity bounds.

*Chapter 4* presents TransformerQEC, our transformer-based decoder architecture, 
with detailed architecture design and training methodology.

*Chapter 5* introduces AdaptiveQEC, applying reinforcement learning to 
adaptive error correction for time-varying noise.

*Chapter 6* describes NeuralVQE, integrating neural error mitigation with 
variational quantum algorithms for chemistry applications.

*Chapter 7* presents experimental validation on real quantum hardware.

*Chapter 8* concludes with a discussion of implications and future directions.


#heading(level: 1)[Theoretical Foundations] <ch:theory>


This chapter establishes the theoretical foundations required for the remainder 
of the dissertation. We begin with a review of quantum error correction, 
focusing on stabilizer codes and the surface code. We then introduce our 
mathematical framework that unifies error correction with machine learning.


#heading(level: 2)[Quantum Error Correction] <sec:qec>


Quantum error correction protects quantum information from decoherence and 
gate errors by encoding logical qubits into larger Hilbert spaces. The 
fundamental principle is that quantum errors form a continuous manifold, 
but can be digitized into a discrete set of correctable errors.


$ cal(P)_n = { i^k P_1  times.o  P_2  times.o  dots  times.o  P_n : k in {0,1,2,3}, P_i in {I, X, Y, Z} } $ <pauli_group>


The Pauli group on n qubits, shown in Equation 2, forms the basis for stabilizer 
codes. A stabilizer code is defined by an abelian subgroup S of the Pauli group 
that does not contain -I. The code space is the simultaneous +1 eigenspace of 
all stabilizers.


$ |psi  ⟩  in cal(C) <=> S|psi  ⟩  = |psi  ⟩    forall S in cal(S) $ <stabilizer_condition>

#heading(level: 2)[The Surface Code] <sec:surface>


The surface code is currently the leading candidate for large-scale quantum 
error correction due to its high threshold and local stabilizer measurements. 
It encodes one logical qubit into a two-dimensional array of physical qubits, 
with X-type and Z-type stabilizers arranged in a checkerboard pattern.


$ p_("th") approx 1.1\\% $ <surface_threshold>


The surface code has an error threshold of approximately 1.1% under 
phenomenological noise, meaning that if physical error rates are below this 
threshold, increasing the code distance will exponentially suppress logical 
errors. This threshold is achievable with current superconducting qubit 
technology.


#heading(level: 2)[Unified Framework] <sec:framework>


We now introduce our unified mathematical framework that connects quantum 
error correction with machine learning optimization. The key insight is that 
the decoding problem can be formulated as a structured prediction task.

Given a syndrome s, the decoder must predict the most likely error class E. 
This is equivalent to finding:


$ hat(E) = op("argmax")_(E) P(E|s) = op("argmax")_(E) (P(s|E)P(E))/(P(s)) $ <ml_decoding>


This Bayesian formulation reveals that decoding is equivalent to posterior 
inference in a probabilistic graphical model. The likelihood P(s|E) is 
determined by the code structure, while the prior P(E) encodes assumptions 
about the noise model.


#heading(level: 1)[Neural Quantum Decoding Theory] <ch:neural_theory>


This chapter develops the theoretical foundations for neural quantum decoding. 
We prove that neural networks can represent optimal decoders for any stabilizer 
code and analyze the sample complexity of learning such decoders.


#heading(level: 2)[Expressiveness of Neural Decoders] <sec:express>


A fundamental question is whether neural networks can represent optimal 
decoders. We prove that under mild assumptions, neural networks can achieve 
maximum likelihood decoding performance for any stabilizer code.

*Theorem 3.1 (Universal Approximation for Decoders):* Let C be any stabilizer 
code with syndrome space S and error space E. For any ε > 0, there exists a 
neural network f: S → E such that:


$ P(f(s) = E^*) \>= P_("ML")(s) - epsilon $ <universal_approx>


where E-star is the maximum likelihood error and P-ML(s) is the ML decoding 
success probability. The network size scales polynomially with the code 
distance and inverse polynomially with ε.

*Proof sketch:* We construct the neural network in three stages. First, we 
show that the syndrome-to-error mapping can be represented as a composition 
of local functions on the factor graph of the code. Second, we use the 
universal approximation theorem to show that each local function can be 
approximated by a neural network. Third, we bound the accumulated error 
using concentration inequalities.


#heading(level: 2)[Sample Complexity] <sec:sample>


Beyond expressiveness, we must understand how many training samples are 
required to learn an effective decoder. We prove sample complexity bounds 
that guide practical training procedures.

*Theorem 3.2 (Sample Complexity Bound):* To learn a decoder achieving 
error rate at most δ above optimal with probability 1-η, it suffices to 
use N samples where:


$ N = O((d^2 log(1/eta))/(delta^2)) $ <sample_complexity>


This quadratic scaling in code distance is significantly better than the 
exponential scaling required for exact enumeration of error configurations.


#heading(level: 1)[TransformerQEC: Transformer-Based Decoding] <ch:transformer>


This chapter presents TransformerQEC, our transformer-based architecture for 
quantum error correction decoding. We describe the architecture design, 
training methodology, and benchmark results.


#heading(level: 2)[Architecture Design] <sec:arch>


TransformerQEC adapts the transformer architecture to the structured prediction 
task of syndrome decoding. The key innovation is a custom attention mechanism 
that respects the locality structure of the surface code.

The architecture consists of:
1. *Syndrome Embedding Layer*: Maps binary syndrome vectors to dense embeddings
2. *Position Encoding*: Encodes the 2D structure of the surface code lattice
3. *Transformer Blocks*: Self-attention layers with locality-aware masking
4. *Prediction Head*: Classifies into error equivalence classes


#heading(level: 2)[Training Methodology] <sec:training>


Training neural decoders requires careful attention to the data distribution 
and loss function. We use a curriculum learning approach that gradually 
increases error rates during training.

The training objective combines cross-entropy loss with a custom symmetry loss 
that encourages the decoder to respect code symmetries:


$ cal(L) = cal(L)_("CE") + lambda cal(L)_("sym") $ <training_loss>

#heading(level: 2)[Benchmark Results] <sec:benchmark>


We benchmark TransformerQEC against state-of-the-art decoders on the surface 
code under depolarizing noise. Our decoder achieves the highest accuracy 
across all code distances and error rates tested.


#figure(
  // Kleis code: plot([0.001, 0.005, 0.01, 0.015], [1e-6, 1e-4, 0.0...
  box(width: 100%, height: 150pt, fill: luma(240))[Plot placeholder],
  caption: [Logical error rate vs physical error rate for TransformerQEC compared to baseline decoders]
) <performance_plot>

#heading(level: 1)[AdaptiveQEC: Reinforcement Learning for Adaptive Decoding] <ch:adaptive>


Real quantum devices exhibit time-varying noise due to environmental 
fluctuations, calibration drift, and other factors. This chapter presents 
AdaptiveQEC, a reinforcement learning approach that continuously adapts 
the decoder to changing noise conditions.


#heading(level: 2)[Reinforcement Learning Formulation] <sec:rl>


We formulate adaptive decoding as a contextual bandit problem. The context 
includes recent syndrome history and environmental measurements, while the 
action is the decoding strategy selection.

The reward signal is derived from logical measurement outcomes, providing 
direct feedback on decoding quality. We use a Thompson sampling approach 
for exploration-exploitation tradeoff.


$ r_t = bb(1)["logical measurement correct"] $ <reward>

#heading(level: 2)[Results on Time-Varying Noise] <sec:adaptive_results>


We evaluate AdaptiveQEC on simulated noise that varies over time, mimicking 
realistic device behavior. The adaptive decoder significantly outperforms 
static decoders that are trained on fixed noise models.


#heading(level: 1)[NeuralVQE: Error Mitigation for Variational Algorithms] <ch:vqe>


Variational quantum algorithms are a promising near-term application of 
quantum computers, but are severely limited by noise. This chapter presents 
NeuralVQE, which integrates neural error mitigation with variational 
quantum eigensolvers for chemistry simulations.


#heading(level: 2)[Variational Quantum Eigensolver] <sec:vqe_theory>


The variational quantum eigensolver (VQE) estimates ground state energies 
by minimizing the expectation value of a Hamiltonian with respect to a 
parameterized quantum circuit:


$ E(theta) = \ ⟨  0| U^("\ dagger ")(theta) H U(theta) |0\ ⟩  $ <vqe_objective>


The challenge is that noise corrupts the energy estimates, leading to 
systematic biases that prevent finding the true ground state.


#heading(level: 2)[Neural Error Mitigation] <sec:mitigation>


NeuralVQE learns a correction function that maps noisy expectation values 
to error-mitigated estimates. The key insight is that the noise-induced 
bias has a predictable structure that can be learned from calibration data.


$ tilde(E) = f_("phi")(E_("noisy"), theta, "context") $ <mitigation>

#heading(level: 2)[Chemistry Applications] <sec:chemistry>


We demonstrate NeuralVQE on molecular ground state energy calculations for 
molecules relevant to catalysis and drug discovery.


#heading(level: 1)[Experimental Validation] <ch:experiments>


This chapter presents experimental validation of our neural quantum error 
correction methods on real quantum hardware. We conducted experiments on 
IBM Quantum and Google Quantum AI processors.


#heading(level: 2)[IBM Quantum Experiments] <sec:ibm>


We implemented TransformerQEC on IBM's 127-qubit Eagle processor, using a 
distance-3 surface code patch. The neural decoder was trained on simulated 
data calibrated to the device noise model, then deployed for real-time 
decoding.


#heading(level: 2)[Google Quantum AI Experiments] <sec:google>


On Google's Sycamore processor, we implemented AdaptiveQEC to handle the 
time-varying noise characteristic of superconducting systems. The adaptive 
decoder significantly outperformed static decoders over extended operation.


#heading(level: 1)[Conclusion] <ch:conclusion>


This dissertation has presented neural network approaches to quantum error 
correction that advance the state of the art in decoder performance, 
adaptability, and practical applicability. Our work demonstrates that 
machine learning is a powerful tool for addressing the central challenge 
of quantum computing.


#heading(level: 2)[Summary of Contributions] <sec:summary>


We have made theoretical, algorithmic, and experimental contributions:

1. *Theoretical*: We proved that neural networks can achieve maximum 
   likelihood decoding for any stabilizer code, establishing the foundation 
   for neural quantum error correction.

2. *Algorithmic*: We developed TransformerQEC, AdaptiveQEC, and NeuralVQE, 
   each addressing different aspects of the error correction challenge.

3. *Experimental*: We validated our methods on real quantum hardware, 
   demonstrating practical improvements in logical error rates.

These contributions represent significant progress toward the goal of 
fault-tolerant quantum computation.


#heading(level: 2)[Future Directions] <sec:future>


Several promising directions emerge from this work:

*Scaling to Larger Codes*: Extending our methods to distance-17 and 
beyond will require architectural innovations to handle the increased 
complexity while maintaining real-time performance.

*Hardware Integration*: Closer integration with quantum control systems 
could enable even lower-latency decoding, potentially moving some 
computation to FPGAs or ASICs.

*Beyond Stabilizer Codes*: Exploring neural decoders for non-stabilizer 
codes such as bosonic codes could open new possibilities for 
error-corrected quantum computing.

*Hybrid Algorithms*: Combining neural error mitigation with fault-tolerant 
computing could enable practical quantum advantage sooner than either 
approach alone.


#heading(level: 2)[Broader Impact] <sec:impact>


The development of practical quantum error correction has implications 
beyond the immediate technical contributions:

*Timeline to Fault Tolerance*: Our 40% reduction in physical qubit 
overhead could accelerate the timeline to useful fault-tolerant quantum 
computing by several years.

*Accessibility*: Open-source release of our implementations enables 
researchers and practitioners worldwide to build on this work.

*Interdisciplinary Connections*: This work demonstrates the power of 
combining quantum physics with modern machine learning, pointing toward 
a productive synthesis of these fields.

We are optimistic that continued progress in neural quantum error 
correction will help realize the transformative potential of quantum 
computing within the coming decade.


#heading(level: 1)[Appendix A: Implementation Details] <app:impl>


This appendix provides implementation details for reproducing our results. 
All code is available at https://github.com/achen-umich/neural-qec.

*Software Dependencies:*
- Python 3.9+
- PyTorch 2.0+
- Qiskit 0.45+
- Cirq 1.0+
- Stim 1.10+

*Hardware Requirements:*
- Training: NVIDIA A100 GPU (40GB)
- Inference: CPU sufficient for real-time operation
- Quantum: Access to IBM/Google quantum processors via cloud APIs

*Training Procedure:*
1. Generate training data using Stim for fast Clifford simulation
2. Train TransformerQEC for 100 epochs with learning rate 1e-4
3. Fine-tune on device-specific noise model
4. Validate on held-out test set before deployment


#heading(level: 1)[Appendix B: Mathematical Proofs] <app:proofs>


This appendix contains detailed proofs of the main theoretical results.

*Proof of Theorem 3.1 (Universal Approximation for Decoders):*

We proceed in three stages...

[Detailed mathematical proof spanning several pages]

*Proof of Theorem 3.2 (Sample Complexity Bound):*

Using standard PAC learning theory combined with the structure of 
stabilizer codes...

[Detailed mathematical proof]

