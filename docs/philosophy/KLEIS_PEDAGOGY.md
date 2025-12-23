# Kleis Pedagogy: Breaking the Engineering Mindset

> **Session insight from Dec 22, 2024**
> 
> "We engineers are not comfortable with this level of abstraction. We start with IEEE legos, then everything has to be concrete — we cannot break that jail cell."

## The Core Problem

Kleis is mathematics, not programming. But readers will approach it with programmer instincts:

| Engineer asks: | Mathematician says: |
|----------------|---------------------|
| "What IS G?" | "G is just a name" |
| "How is mul implemented?" | "mul is just an arrow: G × G → G" |
| "Where is the data stored?" | "What data?" |
| "But what type is it?" | "Types emerge from implements" |

These questions are **the bars of the jail cell.** They prevent understanding Kleis.

## The Kleis Philosophy

### 1. Names First, Not Types

```kleis
// We don't start with "ℤ is the integers"
// We start with:

G           // a name (nothing more)
mul         // an arrow: G × G → G
identity    // an arrow: Unit → G
```

**There is no a priori knowledge.** No IEEE floats. No memory layout. Just names and arrows.

### 2. Arrows Define Everything

This is Category Theory's core insight:
- Objects are opaque (just labels)
- Morphisms (arrows) are all that matter
- Properties come from how arrows compose

```kleis
structure Group(G) {
    operation mul : G × G → G
    element identity : G
    axiom assoc: ∀(a b c : G). mul(mul(a,b),c) = mul(a,mul(b,c))
}
```

**What is a Group?** Just this pattern of arrows and laws. Nothing more.

### 3. Types Emerge, Not Define

In programming: Define type → Add behavior → Hope it forms a pattern
In Kleis: Define structure → Types are witnesses that fit the pattern

```kleis
// ℤ doesn't "exist" as a Platonic entity
// ℤ is just something that witnesses the Group structure:
implements Group(ℤ) {
    operation mul = builtin_add
    element identity = 0
}
```

### 4. Structure-First, Not Type-First

```
Programming (bottom-up):          Kleis (top-down):
─────────────────────────         ──────────────────
    Patterns                          Structures
        ↑ discover                        ↓ define
    Interfaces                        Arrows (operations)
        ↑ abstract                        ↓ require  
    Methods                           Laws (axioms)
        ↑ add                             ↓ constrain
    Types                             Witnesses (implements)
        ↑ create                          ↓ instantiate
    Data                              Concrete types emerge
```

## Manual Structure (Proposed)

### Part I: The Mindset Shift

1. **Introduction: Why This Isn't a Programming Book**
   - The engineer's jail cell
   - What we must unlearn
   - "No a priori knowledge"

2. **Names and Arrows**
   - Names are just labels (no inherent meaning)
   - Arrows relate names
   - That's all there is

3. **Structures as Patterns**
   - A structure is a pattern of arrows
   - Not "a set with operations"
   - Just: arrows + composition laws

### Part II: Building Up

4. **Axioms: Laws About Arrows**
   - How arrows compose
   - What we can prove from structure alone

5. **Implements: Witnessing a Structure**
   - "ℤ fits the Group pattern"
   - Not "ℤ IS a group" — ℤ witnesses Group

6. **Consequences: What Falls Out**
   - Theorems proved once, valid everywhere
   - The power of abstraction-first

### Part III: Connecting to Reality

7. **The Bridge to Computation**
   - Now we can talk about builtins
   - IEEE floats as witnesses
   - LAPACK as implementations

8. **Why This Matters**
   - Prove once, use everywhere
   - Abstraction prevents bugs
   - Mathematics as the ultimate type system

## Presupposed Axioms (Must Be Made Explicit)

Kleis and mathematical notation presuppose familiarity with basic logical axioms.
**The manual must lay out ALL presupposed axioms in the introduction.**

No hidden assumptions. Everything explicit.

### Complete List of Presupposed Rewrite Rules

**1. Equality (substitution)**
```
a = b means: wherever you see a, you may write b
```

**2. Boolean Operations (truth table axioms)**
```
∧ (and):                    ∨ (or):
  true  ∧ true  = true        true  ∨ true  = true
  true  ∧ false = false       true  ∨ false = true
  false ∧ true  = false       false ∨ true  = true
  false ∧ false = false       false ∨ false = false

¬ (not):                    ⟹ (implies):
  ¬true  = false              true  ⟹ true  = true
  ¬false = true               true  ⟹ false = false
                              false ⟹ true  = true
                              false ⟹ false = true
```

**3. Quantifiers (scope rules)**
```
∀(x : A). P(x)    "for any x of shape A, P(x) holds"
                   given a : A, you may write P(a)

∃(x : A). P(x)    "there is some x of shape A where P(x) holds"
                   you must exhibit a witness a : A with P(a)
```

**4. Arrow (function application)**
```
f : A → B         "f takes shape A and produces shape B"
                   given x : A, you may write f(x) : B
```

**5. Product (pairing)**
```
A × B             "a pair: first from A, second from B"
                   given a : A and b : B, you may write (a, b) : A × B
```

**6. Membership**
```
x ∈ S             "x is one of the things in S"
x ∉ S             "x is not one of the things in S" (equivalent to ¬(x ∈ S))
```

**7. Type annotation**
```
x : T             "x has shape T"
                   this is a declaration, not a claim
```

**8. Kleis Keywords (specific semantics)**

These keywords have precise meanings that are NOT obvious:

```
structure Name(T) { ... }
    "define a pattern of arrows called Name, parameterized by T"

implements Structure(Type) { ... }
    "Type witnesses the Structure pattern — here's how"

extends Parent
    "this structure includes all arrows from Parent"

over Base
    "this structure is parameterized over Base"
    (the structure operates in the context of Base)

where Constraint
    "this only applies when Constraint holds"
    (a guard, a condition for the definition to be valid)

let x = ... in ...
    "x is a name for ... within the scope of ..."
    (local binding, not assignment)

as Type
    "treat this expression as having shape Type"
    (annotation, not coercion)

define name = ...
    "name is another way to write ..."
    (pure substitution, not assignment)

axiom name: ...
    "we stipulate this rewrite rule"
    (not a claim about truth, a rule we adopt)

operation name : A → B
    "name is an arrow from A to B"
    (declaration of shape, not implementation)

element name : T
    "name is a distinguished thing of shape T"
    (a constant in the structure)

match x { pattern => result, ... }
    "x has one of these forms; for each form, produce this"
    
    This is NOT a "switch statement." It is an elimination rule:
    
    Given:  data Option(T) { None | Some(T) }
    And:    x : Option(T)
    
    match x {
        None    => ...,     // if x was built with None
        Some(v) => ...,     // if x was built with Some, call its contents v
    }
    
    The match MUST cover all constructors (exhaustive).
    Each pattern binds names to the parts inside.
    
    What match does:
    - "x was built somehow — which way?"
    - "for each way, here's a substitution rule"
    
    It's the INVERSE of constructors:
    - Constructor: builds a value of the data type
    - Match: takes apart a value of the data type
    
    WHAT ARE WE MATCHING? (It's string matching on trees)
    
    A value IS a string of symbols:
    
        Some(5)                →  S o m e ( 5 )
        Cons(1, Cons(2, Nil))  →  a tree of symbols
    
    A pattern is a template with holes:
    
        Some(v)    →  S o m e ( ? )    where ? is hole named v
        Cons(x,xs) →  C o n s ( ? , ? )
    
    Matching asks: "Does the value fit the template?"
    
        Value:   Some(5)      S o m e ( 5 )
        Pattern: Some(v)      S o m e ( ? )
        Match?   Yes!         ? = 5, so v = 5
    
    Pattern matching = tree-shaped regex
    - Values are trees (nested constructor applications)
    - Patterns are templates with wildcards
    - Matching = does the tree fit the template?
    - Binding = what fills the holes?

unification
    "two patterns with unknowns — find assignments that make them equal"
    
    This is like SUDOKU:
    
    - Each cell might be unknown (a variable)
    - Constraints say "these cells must match"
    - Unification finds: what values make all constraints true?
    
    Example:
    
        Pattern 1:  f(X, 3)       (X is unknown)
        Pattern 2:  f(2, Y)       (Y is unknown)
        
        Unify: find X and Y such that f(X,3) = f(2,Y)
        
        Solution: X = 2, Y = 3
        Both become: f(2, 3)
    
    Unlike pattern matching (one-way: value fits template?),
    unification is TWO-WAY: both sides have unknowns.
    
    Think of it as:
    - "We haven't committed that X is definitely some value"
    - "X could be anything — until a constraint forces it"
    - "Once X = 2 somewhere, X = 2 everywhere"
    
    Unification = constraint solving on symbol trees
    
    If no consistent assignment exists → unification fails
    If ambiguity remains → result contains remaining variables
```

---

**The introduction must present ALL of these before any Kleis code appears.**
Not as a "logic lesson" but as: "here are the rewrite rules we'll use."

---

## Foundational Definitions (No Assumed Knowledge)

### Equality: The Most Loaded Symbol

`a = b` means: **"wherever you see a, you may write b"**

That's it. Pure substitution. Not:
- "a and b are the same thing" (what is "same"?)
- "a evaluates to b" (what is evaluation?)
- "a is identical to b" (identity is heavy philosophy)

Just: **you may substitute one for the other.**

This is the only definition that doesn't smuggle in assumptions.

### Logic as String Rewrite Rules

Not "Logic 101" (philosophy, truth, validity...)
But "Rewrite Rules" (patterns, substitution, strings)

**The rules are just:**

```
Rule: ∀-elimination
──────────────────
If you have:     ∀(x : G). P(x)
And you have:    a : G
You may write:   P(a)

(substitute a for x in the string P)
```

```
Rule: =-substitution
────────────────────
If you have:     a = b
And you have:    ...a...
You may write:   ...b...

(replace the substring a with b)
```

```
Rule: →-application
───────────────────
If you have:     f : A → B
And you have:    x : A
You may write:   f(x) : B

(stick x after f, result has shape B)
```

**That's all of logic.** Three string rewrite rules.

Not truth. Not meaning. Not philosophy.
Just: **"if you see this pattern, you may produce that pattern."**

### How Many Rules in Logic 101?

**Surprisingly few.**

**Natural Deduction (intro + elim for each connective):**

| Connective | Introduction | Elimination |
|------------|--------------|-------------|
| ∧ (and) | A, B ⊢ A∧B | A∧B ⊢ A |
| ∨ (or) | A ⊢ A∨B | A∨B, A→C, B→C ⊢ C |
| → (implies) | [A]...B ⊢ A→B | A, A→B ⊢ B |
| ¬ (not) | [A]...⊥ ⊢ ¬A | A, ¬A ⊢ ⊥ |
| ∀ (forall) | P(a) arbitrary ⊢ ∀x.P(x) | ∀x.P(x) ⊢ P(a) |
| ∃ (exists) | P(a) ⊢ ∃x.P(x) | ∃x.P(x), [P(a)]...C ⊢ C |
| = (equals) | ⊢ a=a | a=b, P(a) ⊢ P(b) |

**Count: ~14 rules**

**Reduced to pure rewriting:**

```
3 inference rules:
  1. =-substitution     a=b means: replace a with b
  2. →-application      f:A→B, x:A gives f(x):B  
  3. ∀-elimination      ∀x.P(x), a:A gives P(a)

14 truth table axioms (definitions):
  4 for ∧, 4 for ∨, 2 for ¬, 4 for →
```

**Total: 3 rules + 14 axioms = 17 stipulations**

That's it. All of logic. All of mathematics. Built from ~17 stipulations.

Everything else is just applying these 17 things over and over.

### Potential Tool: Kleis → Logic 101 Compiler

Since Kleis is built on these ~17 stipulations, we could write a **cross-compiler**
that lowers Kleis to raw logic:

```
Kleis                              Logic 101 (raw)
─────                              ───────────────

structure Group(G) {               ∀G. ∃mul. ∃id. ∃inv.
    operation mul: G×G→G             (∀x:G.∀y:G. mul(x,y):G) ∧
    element identity: G       →→→    (id:G) ∧
    operation inv: G→G               (∀x:G. inv(x):G) ∧
    axiom assoc: ...                 (∀a:G.∀b:G.∀c:G. ...) ∧ ...
}
```

**Translation table:**

| Kleis Construct | Logic 101 Output |
|-----------------|------------------|
| `structure Name(T)` | `∀T. ∃operations...` |
| `operation f : A → B` | `∀x:A. f(x):B` |
| `element e : T` | `e:T` |
| `axiom name: P` | `P` (raw formula) |
| `implements S(X)` | Witness instantiation |
| `define f = e` | `f = e` (equality assertion) |

**Why this matters:**

1. **Proof of soundness** — Kleis is just logic, nothing hidden
2. **Transparency** — See what's "really happening"
3. **Export to proof assistants** — Lean, Coq, Isabelle could verify Kleis
4. **Pedagogical** — "Here's your structure in raw logic"

**The output would be unreadable.** Pages of squiggles.

But that's the point: **Kleis exists so you don't have to write that.**

The abstractions are for humans. The logic is the truth.

**Meta-circularity: The compiler can be written in Kleis itself.**

```kleis
// Kleis describing its own lowering to Logic 101

data KleisAST {
    constructor Structure(name: String, params: List(String), members: List(Member))
    constructor Implements(structure: String, witness: String, defs: List(Def))
    constructor Axiom(name: String, formula: Formula)
    ...
}

data Logic101 {
    constructor Forall(var: String, body: Logic101)
    constructor Exists(var: String, body: Logic101)
    constructor Implies(left: Logic101, right: Logic101)
    constructor Equals(left: Term, right: Term)
    constructor And(left: Logic101, right: Logic101)
    ...
}

define lower : KleisAST → Logic101 = match ast {
    Structure(name, params, members) => 
        Forall(params, Exists(op_names(members), And(...)))
    ...
}
```

**This is ultimate self-description:**
- Kleis is built on Logic 101
- Kleis can describe the translation to Logic 101
- Therefore Kleis can formalize its own foundations

**The tool describes itself.** No external metalanguage needed.

**The Kleis Manual in Logic 101 form:**

```
Page 1:     The 17 rules (3 inference + 14 axioms)
Pages 2-∞: Squiggles

Table of Contents:
  1. Introduction .......................... 17 lines
  2. Group structure ...................... 47 pages of ∀∃→∧
  3. Ring structure ....................... 89 pages of ∀∃→∧
  4. Vector spaces ........................ 234 pages of ∀∃→∧
  5. Matrices ............................. 1,247 pages of ∀∃→∧
  ...
```

**This is why abstraction exists.**

Not because logic is insufficient.
But because humans cannot read a thousand pages of squiggles.

The 17 rules ARE the content.
The rest is just applying them.
Kleis lets you apply them without seeing them.

---

## The Mystery: Why Does It Match Reality?

**Start with:** 17 meaningless rules
**Apply them:** Get topology, geometry, algebra
**Result:** Structures that MATCH the world we perceive

Eugene Wigner called it: **"The Unreasonable Effectiveness of Mathematics"**

We shuffle symbols. We follow rewrite rules.
And somehow... spacetime falls out. Quantum mechanics falls out.
Structures that describe what we SEE.

**The univalences:**

| Abstract (squiggles) | Perceived (world) |
|----------------------|-------------------|
| Topological spaces | Shapes, continuity, nearness |
| Groups | Symmetries of objects |
| Manifolds | Curved surfaces, spacetime |
| Hilbert spaces | Quantum states |
| Differential forms | Fields, flows |

**These are not analogies.** The math IS the structure.

Topology "feels like" geometry because it IS geometry — 
the abstract version that doesn't depend on coordinates.

**Possible explanations:**

1. **Platonism** — Mathematical structures exist; physics discovers them
2. **Kantianism** — Our minds impose structure; math reflects cognition
3. **Structural realism** — Reality IS relational structure; math captures relations
4. **Comparian view** — Relation is primary; what we call "reality" is pattern of relations

**Kleis takes no position.** It just lets you work with structures.

But the author's view: **relations are primary, objects are secondary.**
The correspondence between abstract and concrete is not a mystery — 
it's because both ARE relational structure. Same thing, different perspectives.

### Truth Tables Are Just Axioms

Truth tables aren't "facts about truth." They're **definitions** — rewrite rules for logical operators:

```
∧ (and) — defined by axioms:        ∨ (or) — defined by axioms:
─────────────────────────────       ────────────────────────────
true  ∧ true  = true                true  ∨ true  = true
true  ∧ false = false               true  ∨ false = true
false ∧ true  = false               false ∨ true  = true
false ∧ false = false               false ∨ false = false

¬ (not) — defined by axioms:        ⟹ (implies) — defined by axioms:
────────────────────────────        ─────────────────────────────────
¬true  = false                      true  ⟹ true  = true
¬false = true                       true  ⟹ false = false
                                    false ⟹ true  = true
                                    false ⟹ false = true
```

These are not discoveries about some Platonic "truth."
They are **stipulations**: "when you see `true ∧ false`, you may write `false`."

In Kleis, this is just a structure:

```kleis
structure Boolean(B) {
    element true : B
    element false : B
    operation and : B × B → B
    operation or  : B × B → B
    operation not : B → B
    
    axiom and_tt: and(true, true)   = true
    axiom and_tf: and(true, false)  = false
    axiom and_ft: and(false, true)  = false
    axiom and_ff: and(false, false) = false
    // ... same for or, not, implies
}
```

**All logical operators are just axiom-defined rewrite rules.**

An engineer can understand this. It's just regex with extra steps.
The manual could literally say: *"You already know find-and-replace. That's logic."*

### Notation to Introduce (Without Being Didactic)

Each symbol should appear in context, then be briefly named:

```
∀(x : G). P(x)
  ↑  ↑ ↑    ↑
  │  │ │    └── a claim about x
  │  │ └─────── x has shape G
  │  └───────── x is a name
  └──────────── "for any choice of x"
```

**Show → Name → Move on.** Never a dedicated "logic lesson."

## Key Phrases to Use

- "Names, not types"
- "Arrows, not methods"
- "Witnesses, not instances"
- "Structures, not classes"
- "No a priori knowledge"
- "All there is are arrows and how they compose"
- "Equality means: you may substitute one for the other"

## Key Phrases to Avoid

- "G is a type" → "G is a name"
- "Create a Group" → "Define the Group structure"
- "ℤ is a Group" → "ℤ witnesses Group"
- "Implement the interface" → "Provide a witness"

### The Java Analogy Problem

Explaining Kleis by analogy to Java is **starting from the wrong end**:

```
❌ WRONG (engineering-first):
"A structure is like a Java interface"
"implements is like a class implementing an interface"

This is not incorrect, but it:
- Starts from Java, maps to math
- Makes structures seem like "fancier interfaces"
- Smuggles in OOP mental models
- Implies structure is secondary to concrete types
```

```
✅ RIGHT (math-first):
"A structure is a pattern of arrows and laws"
"implements says: this thing witnesses that pattern"

Later, as a footnote:
"Programmers may recognize similarity to interfaces.
But structures came first — interfaces are the shadow."
```

**The analogy should be a footnote, not the introduction.**

Kleis is not "Java but mathematical."
Java interfaces are "structures but impoverished."

## The Goal

> "We really want to knock over the pre-established walls."

The manual should not be a happy-go-lucky tutorial. It should be a **paradigm shift document** that acknowledges:

1. This will feel uncomfortable
2. Your instincts will fight you
3. That discomfort is the jail cell breaking
4. Once through, mathematics opens up

---

## Gaps To Fill (Probably Missing)

Things that likely need explanation in the introduction but aren't yet covered:

**Variables and Binding:**
- Free vs bound variables — when is a name "captured" by a quantifier?
- Scope — what does "in scope" mean precisely?
- Capture-avoiding substitution — why can't we always just replace?
- α-equivalence — λx.x and λy.y are "the same"

**Functions and Application:**
- What IS a function? (arrow? set of pairs? rule?)
- Application f(x) — what happens when we "apply"?
- Currying — f(x, y) vs f(x)(y), are they the same?
- λ abstraction — λx. body means "a function that..."

**Composition and Reduction:**
- Composition g ∘ f — "first f, then g" (order matters!)
- β-reduction — (λx. body)(arg) → body[x := arg]
- Normal form — when is an expression "fully simplified"?
- Evaluation order — does it matter which reduction first?

**Recursion and Induction:**
- Recursive definitions — defining f in terms of f
- Well-foundedness — when is recursion "safe"?
- Structural induction — proving things about recursive data
- Termination — does evaluation always finish?

**Types and Generics:**
- Parametricity — what does "for all T" really mean?
- Type variables vs value variables
- Polymorphism — one definition, many types
- Type inference — how does Kleis figure out types?

**Logic Details:**
- Negation ¬ — is it just "not true"? 
- Contradiction / ⊥ — the absurd, proves anything (ex falso quodlibet)
- Double negation — is ¬¬P the same as P?
- Constructive vs classical logic — does Kleis take a side?

**Meta-questions:**
- Decidability — can a computer always check this?
- Soundness — if Kleis says it's true, is it really?
- Completeness — if it's true, can Kleis prove it?
- Consistency — can Kleis prove contradictions?

**Practical:**
- Whitespace and formatting — does it matter?
- Comments — how to write them?
- Error messages — how to read them?
- REPL commands — what are they, why separate from the language?

---

## The Power of Pictures

In category theory, complexity has a SHAPE that can be drawn.

**Pictures break biases from everyday language:**

```
Words carry baggage:
  - "is a" → inheritance? identity? subset?
  - "has a" → containment? ownership? reference?
  - "transforms" → mutation? mapping? conversion?

Arrows carry no baggage:
  
      A ───f──→ B
      
  Just: "there's an arrow called f from A to B"
  No hidden meaning. No metaphor. Just structure.
```

**Commutative diagrams show what words obscure:**

```
       f
  A ──────→ B
  │         │
g │         │ h
  ↓         ↓
  C ──────→ D
       k

"The diagram commutes" means: h ∘ f = k ∘ g

Going A→B→D equals going A→C→D.
You SEE it. No words needed.
```

**String diagrams show composition:**

```
    ┌───┐
 ───┤ f ├───
    └───┘
    ┌───┐
 ───┤ g ├───
    └───┘

Stack them: g after f. Composition is VERTICAL stacking.
Parallel is HORIZONTAL placing. You see the shape of computation.
```

**Why this matters for Kleis:**

- Words like "implements" carry OOP baggage
- Arrows don't: `ℤ ──witness──→ Group`
- The manual should use diagrams HEAVILY
- When words fail, draw the picture
- The picture IS the concept, words are commentary

**Proposed: Every major concept gets a diagram FIRST, words second.**

---

## Writing Philosophy: Simplicity Without Context

**We are leaving "Learn You Some Kleis for Great Good" behind.**

Not catchy. Not clever. Just simple.

```
The problem with Kleis:
─────────────────────────
It IS simple.
People expect complexity.
When they don't find it, they read INTO it.
They assume hidden depth that isn't there.
```

**The manual should:**
- State definitions plainly
- Let relations between definitions create context
- Not motivate, not sell, not promise
- Just: "here is a definition, here is another, here is how they connect"

```
Like a math textbook:

    Definition 1.1: A structure is...
    Definition 1.2: An arrow is...
    
    Proposition 1.3: Given 1.1 and 1.2, we have...

No preamble. No "why this matters."
The meaning emerges from the relations.
```

**Context emerges. It is not injected.**

If the reader asks "but why?" too early — they're looking for something that isn't there yet. The answer is: "keep reading, the relations will show you."

**The simplicity is the point. Don't apologize for it. Don't decorate it.**

---

## The Psychology of Simplicity

**People are afraid of the blank page.**

```
A plain field. Nowhere to hide.

Complexity provides cover:
  - "I don't understand because it's too advanced"
  - "I need more prerequisites"
  - "The notation is confusing"
  - "It's a hard topic"

Simplicity strips that away:
  - If you don't understand, you can't blame the material
  - There's no jargon to shelter behind
  - The definitions are plain
  - You are exposed
```

**This is uncomfortable. That's correct.**

The fear of the white sheet is the fear of:
- Having no excuse
- Being unable to hide behind complexity
- Confronting that the obstacle might be internal

**Kleis is simple. That's what makes it hard.**

Not hard because it's complex.
Hard because there's nothing to hide behind.

**The manual should acknowledge this honestly:**
- "This will feel exposing."
- "The simplicity is intentional."
- "If you're confused, stay with it — the confusion is not from complexity."

---

## Why Abstraction: Humans, Not Machines

**The raw truth:**

We can do ALL of mathematics with what we learned in logic 101:
- Symbols
- Substitution rules
- That's it

**But it would look like this:**

```
∀x.∀y.∀z.((R(x,y) ∧ R(y,z)) → R(x,z)) ∧ ∀x.R(x,x) ∧ 
∀x.∀y.(R(x,y) → R(y,x)) ∧ ∀x.∃y.(R(x,y) ∧ ∀z.(R(x,z) 
→ (z = y ∨ z = x))) ∧ ∀x.∀y.∀z.((R(x,y) ∧ R(x,z)) → 
(R(y,z) ∨ y = z)) ∧ ...
```

Pages and pages of similar symbols. Endless squiggles.
A machine can process it. A human cannot.

**Abstraction is for US:**

```kleis
structure Group(G) {
    operation mul : G × G → G
    element identity : G
    axiom associativity: ...
}
```

This says the same thing. But now we can see it.

**The chunks are for human cognition:**

| Raw | Named |
|-----|-------|
| `∀x.∀y.(R(x,y) → R(y,x))` | `axiom symmetry` |
| `∀x.∃y.(...)` | `operation inverse` |
| Endless squiggles | `structure Group(G)` |

**We name things so we can hold them in mind.**

The formalism is the truth.
The abstractions are for us.
Not because the machine needs them — because WE need them.

---

*This document captures insights from a session on Dec 22, 2024, discussing the philosophical foundations of Kleis and how to teach them.*

