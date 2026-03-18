# The Kleis Manual: From Symbols to Structures

---

## Part I: The Mindset Shift

### Intro: Breaking the Engineering Mindset

> "We engineers are not comfortable with this level of abstraction. We start with IEEE legos, then everything has to be concrete — we cannot break that jail cell."

Kleis is mathematics, not programming. To understand it, you must stop asking "How is this stored?" and start asking "How is this related?"

| Engineer asks:               | Mathematician says:                      |
|------------------------------|------------------------------------------|
| "What is G?"                 | "G is just a name"                       |
| "How is mul implemented?"    | "It is an arrow: $G \times G \to G$"     |
| "Where is the data stored?"  | "What data?"                             |
| "But what type is it?"       | "Types emerge from implements"           |

---

### Chapter 1: Names and Arrows

In Kleis, an object has no internal meaning. Its existence is defined solely by how it relates to other names through arrows.

- **The Name**: A label (Sort) with no bit-width or memory layout.
- **The Arrow**: A directed connection (Operation or Element).
- **The Structure**: A blueprint or pattern of names and arrows.
- **The Witness**: An implementation that satisfies a structure. We don't say "Integers are a group"; we say "Integers witness the Group structure."

---

### Chapter 2: The 17 Rules of the Game

Kleis is a Formal System. It is a game of string rewriting based on exactly 17 stipulations.

**The 3 Inference Rules:**

1. **Equality Substitution ($=$)**: If $a = b$, swap the symbols.
2. **Arrow Application ($\to$)**: Given $f: A \to B$ and $x: A$, you have $f(x): B$.
3. **Universal Elimination ($\forall$)**: If a law applies to all, it applies to your specific witness.

**The 14 Definitions:**

Truth tables for $\wedge, \vee, \neg,$ and $\implies$ are simply find-and-replace rules.

---

### Chapter 3: Witnesses and Counterexamples

Kleis validates claims by exhibiting or refuting instances.

- **Witness**: A model (world) where your constraints are satisfied.
- **Counterexample**: A concrete state that proves a law is violated.

Verification is not about finding "Truth"—it is about eliminating the "Impossible."

---

## Part II: The Language Reference

### Chapter 4: Keywords and Scoping

- **define**: Declares a Name.
- **structure**: Declares a Jurisdiction of laws.
- **let ... in**: Creates a local, scoped substitution.
- **over**: Establishes a foundational dependency for a structure.
- **as**: Annotates a shape without casting the underlying data.

---

### Chapter 5: Data and Computation

Kleis becomes a programming language when you define Data Constructors.

- **Constructors**: Arrows that build a tree of symbols.
- **Match**: The eliminator that takes the tree apart.
- **Reduction**: Running code is the process of simplifying these trees using the 17 rules.

---

## Part III: The Standard Library & The Platonic Gap

### Chapter 6: Lifting and the Platonic Gap

We cannot accurately represent a Real number in a finite machine. This is the gap Plato identified between the Ideal and the Physical.

- **Rationals ($\mathbb{Q}$)**: The World of Forms. Perfect and exact.
- **Reals ($\mathbb{R}$)**: The World of Shadows. Approximated by IEEE Legos.
- **Lifting**: The intentional move from the Ideal Form to the Physical Shadow.

---

### Chapter 7: Complex Numbers and Matrices

Kleis uses a **Hybrid Datatype Approach** for complex numbers, defining $i$ via the axiom $i^2 = -1$.

**Matrix Dimensions as Logic:**

Dimensions ($M, N$) are part of the Sort. The Signature Interpreter uses unification to ensure that $(M \times N) \cdot (N \times P)$ results in $(M \times P)$. If $N \neq N$, the code is ill-formed and will not verify.

**Realification:**

To maintain compatibility with real-arithmetic SMT solvers (Z3), Kleis "Realifies" complex matrices into $2n \times 2n$ real block matrices.

$$realify(A, B) = \begin{pmatrix} A & -B \\ B & A \end{pmatrix}$$

---

## Part IV: The REPL and Verification

### Chapter 8: The Three Modes of Operation

The Kleis REPL has three fundamental commands:

| Command | Question | What Happens |
|---------|----------|--------------|
| `:eval` | "What is the value?" | Computes via builtins/pattern matching |
| `:verify` | "Is this always true?" | Z3 proves universal statements |
| `:sat` | "Does a solution exist?" | Z3 finds witnesses |

**Example:**
```
:eval 2 + 3           → 5         (computation)
:verify ∀a. a + 0 = a → Valid     (proof)
:sat x * x = 49       → x = 7     (witness)
```

---

### Chapter 9: The Verification Gap (Critical!)

**Users must understand this limitation.**

`:verify` checks axioms **symbolically** (in Z3's mathematical model).
`:eval` runs code **concretely** (via Rust builtins).

**These are not connected.**

When you verify `∀(a b : ℕ). a + b = b + a`, Z3 proves this for its idealized integers.
When you compute `2 + 3`, Rust's `+` operator runs.

**We never verify that the implementation matches the specification.**

**The Trusted Computing Base:**
- Rust compiler (assumed correct)
- Builtin implementations (assumed correct)
- LAPACK for matrices (assumed correct)
- IEEE 754 floating point (assumed correct)

**What Kleis provides:**
- ✅ Mathematical properties verified symbolically
- ✅ Efficient concrete computation
- ❌ No proof that computation matches specification

**This is pragmatic, not ideal.** True end-to-end verification requires verified compilers (like CompCert) or proof extraction (like Coq → OCaml).

---

### Chapter 10: The Locality of Truth

There are no top-level axioms in Kleis. Truth is never global; it is always scoped within a structure. You can define Tools (Definitions) globally, but you can only legislate Laws (Axioms) locally.

---

**The jail cell is now open.**

You have the power of a Turing-complete language and the **bounded certainty** of mathematical verification — bounded by your trust in the implementation.

