# Verified Software: The Kleis Dream

*Date: December 21, 2024*

> **Note:** This document describes an *aspiration*, not a validated architecture.
> Section "The Hard Truth" documents what we actually achieved vs. what remains a dream.

## The Breakthrough

We implemented a LISP interpreter in Kleis. This seems like a programming exercise, but the implications are profound.

**LISP is homoiconic** — code is data. A LISP program is itself a data structure (S-expression) that can be manipulated, analyzed, and reasoned about.

**Kleis has Z3 verification** — we can prove properties about data structures using SMT solving.

**Therefore:** We can **prove properties about programs**.

This is the foundation of **verified software**.

---

## The Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  LISP Program (Source Code)                                 │
│  "(define (fact n) (if (<= n 1) 1 (* n (fact (- n 1)))))"  │
└─────────────────────────────────────────────────────────────┘
                              ↓ parse()
┌─────────────────────────────────────────────────────────────┐
│  LISP AST (Kleis Data Structure)                            │
│  SList(Cons(SAtom("define"), Cons(SList(...), ...)))       │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Specification (Kleis Axiom)                                │
│  axiom fact_correct: ∀ n ≥ 0 . eval(fact, n) = n!          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Z3 Verification                                            │
│  :verify ∀ n . n ≥ 0 → eval(fact_ast, n) = factorial(n)    │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Result                                                     │
│  ✅ Valid: Program PROVEN correct for ALL inputs            │
│  ❌ Invalid + Counterexample: Bug at n = 13                │
└─────────────────────────────────────────────────────────────┘
```

---

## What This Enables

### 1. Semantic Verification (Not Just Type Checking)

Traditional type systems prove syntactic properties:
- "This function returns an `Int`"
- "These types match"

Kleis proves **semantic** properties:
- "This function returns **the correct** `Int`"
- "This sort function produces a **sorted, permuted** list"
- "This parser accepts **exactly the language defined by this grammar**"

```kleis
// Traditional: sort :: List(Int) → List(Int)
// Kleis:
axiom sort_produces_sorted_output:
    ∀ xs : List(Int) . is_sorted(sort(xs))

axiom sort_preserves_elements:
    ∀ xs : List(Int) . is_permutation(xs, sort(xs))
```

### 2. Proofs Cover All Cases

| Testing | Verification |
|---------|--------------|
| Checks *some* inputs | Proves for *all* inputs |
| "No bugs found" | "No bugs possible" |
| Coverage metrics | Mathematical certainty |
| Can miss edge cases | Edge cases are theorems |

```kleis
// A test checks ONE case:
assert(factorial(5) == 120)

// A verification proves ALL cases:
axiom factorial_correct:
    ∀ n : ℕ . factorial(n) = product(1..n)
```

### 3. Bug Finding via Counterexample

When verification fails, Z3 provides a **counterexample** — a concrete input that violates the specification:

```
λ> :verify ∀ x : Int . x > 0 → safe_divide(100, x) > 0

❌ INVALID
   Counterexample: x = 0
   
   // Z3 found: when x = 0, the property fails
   // This reveals the bug: we forgot to handle x = 0
```

### 4. Program Synthesis

Instead of *writing* a program, *describe* what you want:

```
λ> :sat ∃ f : SExpr . ∀ x : Int . eval(f, x) = x * x

✅ SAT
   f = SList([SAtom("*"), SAtom("x"), SAtom("x")])
   
   // Z3 synthesized: (* x x)
```

Z3 **generates** a program that meets the specification!

### 5. The Tower of Trust

```
┌─────────────────────────────────────────┐
│  Your Application                       │  ← Verified against spec
│  (LISP, or any language)               │
├─────────────────────────────────────────┤
│  Language Interpreter (Kleis)           │  ← Can also be verified!
│  lisp_parser.kleis (560 lines)         │
├─────────────────────────────────────────┤
│  Kleis Evaluator (Rust)                 │  ← Trusted computing base
│  eval_concrete in evaluator.rs         │
├─────────────────────────────────────────┤
│  Z3 SMT Solver                          │  ← Mathematical foundation
│  Decision procedures for theories       │
└─────────────────────────────────────────┘
```

**Key insight:** Each layer can verify the layer above it. The interpreter itself can have verified properties:

```kleis
// The interpreter is deterministic
axiom eval_deterministic:
    ∀ prog : SExpr . ∀ env : Env .
        eval_lisp(prog, env) = eval_lisp(prog, env)

// Substitution commutes with evaluation
axiom eval_substitution:
    ∀ body : SExpr . ∀ x : String . ∀ v : LispVal .
        eval(substitute(body, x, v), env) = eval(body, extend(x, v, env))
```

---

## The Revolutionary Insight

### The Chain of Ideas

```
LISP (1958)
  └─→ "Code is Data" (homoiconicity)
        └─→ Programs can be manipulated as data structures

Kleis (2024)
  └─→ "Specifications are Axioms"
        └─→ What a program should do is a mathematical statement

Z3 SMT Solver
  └─→ "Axioms can be verified automatically"
        └─→ Mathematical statements can be proven mechanically

∴ Programs can be PROVEN correct, not just tested
```

### The Paradigm Shift

| Old Paradigm | New Paradigm |
|--------------|--------------|
| Write code, then test | Write spec, then verify |
| Hope for correctness | Prove correctness |
| Debug after deployment | Catch bugs before compilation |
| "Works on my machine" | "Works on all machines, mathematically" |

---

## The Irony: LLMs Building Their Own Replacement

### The Evolution of Code Creation

```
┌─────────────────────────────────────────────────────────────────┐
│  ERA 1: Human writes code                                       │
│  └─→ Experience + intuition → bugs happen                       │
├─────────────────────────────────────────────────────────────────┤
│  ERA 2: LLM writes code (2022-present)                          │
│  └─→ Statistical patterns → "looks right" (can hallucinate)    │
├─────────────────────────────────────────────────────────────────┤
│  ERA 3: SMT solver synthesizes code (emerging)                  │
│  └─→ Mathematical proof → correct by construction              │
└─────────────────────────────────────────────────────────────────┘
```

### The Hierarchy of Trust

| Approach | Mechanism | Guarantee |
|----------|-----------|-----------|
| **Human** | Training + experience | *"I think this works"* |
| **LLM** | Pattern matching on training data | *"This looks like code that worked before"* |
| **Z3/Kleis** | Mathematical proof | **"This MUST work — proven"** |

### The Fundamental Difference

**LLMs guess.** They predict what code *probably* works based on statistical patterns in training data.

**Z3 proves.** It determines what code *must* work based on mathematical logic.

```kleis
// LLM approach:
// "Here's a sort function that looks like the ones I was trained on"
// → Might have edge case bugs, needs testing

// Z3 synthesis approach:
:sat ∃ f : SExpr . ∀ xs : List . 
    is_sorted(eval(f, xs)) ∧ is_permutation(eval(f, xs), xs)

// Z3: "f = (merge-sort xs)" — mathematically guaranteed correct
```

### The Weird Part

We used an LLM to build Kleis.

Kleis + Z3 can synthesize correct programs from specifications.

**The LLM helped build the tool that makes LLMs unnecessary for critical code.**

### Where Each Excels

| Task | Best Tool | Why |
|------|-----------|-----|
| Boilerplate code | LLM | Speed, pattern matching |
| Exploratory coding | LLM | Flexibility, natural language |
| Safety-critical logic | Z3/Kleis | Mathematical guarantee |
| Bug-finding | Z3/Kleis | Counterexamples are proofs |
| Verifying LLM output | Z3/Kleis | LLM proposes, Kleis verifies |

### The Future Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Developer: "I need a function that reverses a list"            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  LLM: "Here's reverse(xs) = ..."                                │
│  (Fast, but might have bugs)                                    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  Kleis/Z3: ":verify ∀ xs . reverse(reverse(xs)) = xs"          │
│  ✅ Valid — LLM's code is correct                               │
│  ❌ Invalid — Counterexample: xs = [1,2,3] fails               │
└─────────────────────────────────────────────────────────────────┘
```

**LLM as proposer, Kleis as verifier.**

### The Key Insight

**Kleis doesn't replace programmers OR LLMs. It sits *above* both as the verifier:**

1. Human/LLM writes code
2. Kleis verifies it's correct
3. **Or:** Kleis synthesizes the correct code directly from specs

The combination is more powerful than either alone:
- LLM's speed and flexibility
- Z3's mathematical rigor
- Human in the loop for specifications

### The Ultimate Architecture: LLM as Constraint Translator

The deepest insight: **LLMs understand natural language. Z3 synthesizes correct programs. Combine them.**

```
┌─────────────────────────────────────────────────────────────────┐
│  Human: "I need a function that sorts a list"                   │
│  (Natural language, ambiguous, informal)                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓ LLM translates
┌─────────────────────────────────────────────────────────────────┐
│  Kleis Constraint:                                              │
│  ∃ f . ∀ xs . is_sorted(f(xs)) ∧ is_permutation(f(xs), xs)     │
│  (Formal, precise, verifiable)                                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓ Z3 synthesizes
┌─────────────────────────────────────────────────────────────────┐
│  Program: (mathematically guaranteed correct)                   │
└─────────────────────────────────────────────────────────────────┘
```

**The Division of Labor:**

| Step | Who Does It | Risk | Size |
|------|-------------|------|------|
| Natural language → Constraint | **LLM** | Hallucination possible | 2-5 lines |
| Constraint → Correct Program | **Z3** | Mathematically guaranteed | 50+ lines |

**Why this shrinks the hallucination problem:**

Instead of asking: *"LLM, write me a correct sort function"* (50 lines, high risk)

Ask: *"LLM, what does 'sort' mean formally?"* (2 lines, easy to verify)

```kleis
// Human reviews the constraint (small, readable):
∀ xs . is_sorted(result) ∧ is_permutation(result, xs)
// "Yes, that's exactly what I meant by 'sort'."

// Z3 synthesizes the code — mathematically guaranteed
:sat ∃ f . ∀ xs . is_sorted(f(xs)) ∧ is_permutation(f(xs), xs)
// Result: f = merge_sort (correct by construction)
```

**The genius of this approach:**

1. **Constraints are small** — easy for humans to verify
2. **Constraints are formal** — Z3 can work with them precisely
3. **LLM failure is catchable** — wrong constraint is obvious to human
4. **The program is guaranteed** — Z3 doesn't hallucinate

**The roles crystallize:**

- **LLM:** Good at understanding intent, bad at guarantees
- **Z3:** Bad at understanding intent, perfect at guarantees
- **Human:** Reviews the small constraint, not the large program
- **Together:** Natural language in → Verified program out

This is the future of programming: **intent-driven, constraint-mediated, mathematically guaranteed**.

---

## Practical Applications

### 1. Safety-Critical Systems
- Avionics, medical devices, nuclear plants
- Where failure costs lives
- Regulatory requirement for formal verification

### 2. Financial Systems
- Smart contracts (billions of dollars at stake)
- Trading algorithms
- Cryptographic protocols

### 3. Security
- Prove absence of vulnerabilities
- Verify access control policies
- Cryptographic correctness

### 4. Compilers and Interpreters
- Prove the compiler preserves semantics
- Verify optimizations are sound
- CompCert (verified C compiler) as precedent

---

## What Kleis Provides

1. **Homoiconic Target Language** — Programs as data (`SExpr`, `LispVal`)
2. **Specification Language** — Axioms with quantifiers
3. **Automated Verification** — Z3 backend
4. **Counterexample Generation** — Bug localization
5. **Program Synthesis** — Generate code from specs (potential)

---

## The Hard Truth: What We Actually Achieved (December 2024)

The vision above is beautiful. Here's the reality.

### The Failed Experiment

We tried to have Z3 synthesize a LISP sorting program from specification:

```kleis
// THE DREAM (does not work):
:sat ∃(P : SExpr). 
    eval_lisp(P, [3,1,2]) = [1,2,3] ∧
    eval_lisp(P, [5,1]) = [1,5]

// Expected: P = SList([SAtom("letrec"), ...])
// Reality:  Stack overflow / timeout
```

**Why it failed:**

| Component | Works? | Problem |
|-----------|--------|---------|
| LISP Parser (Kleis) | ✅ | - |
| LISP Evaluator (Kleis) | ✅ | - |
| `eval_lisp` in Z3 | ❌ | Recursive function over unbounded ADTs |
| Z3 searching SExpr space | ❌ | Infinite search space |

**The fundamental barrier:**

Z3 cannot symbolically execute recursive functions over algebraic data types. When we write:

```kleis
define eval_lisp(expr: SExpr, env: Env) : LispVal =
    match expr { ... eval_lisp(sub_expr, env) ... }
```

And then ask:

```kleis
:sat ∃(P : SExpr). eval_lisp(P, env) = some_value
```

Z3 would need to instantiate the universal quantifier in `eval_lisp` for **every possible SExpr**. That's infinite. Z3 tries E-matching, runs out of stack, dies.

### What We Actually Did

We **gave up on true synthesis** and did sketch-based synthesis instead:

```
TRUE SYNTHESIS (failed):
  Z3 input:  Spec + entire LISP grammar
  Z3 output: LISP program
  
SKETCH-BASED (what we did):
  LLM input:  Spec
  LLM output: insert(x, ys) template with 3 parameter holes
  
  Z3 input:   16 parameter combinations
  Z3 output:  (cc=0, tc=0, ec=1)
  
  LLM input:  Z3's parameters
  LLM output: LISP program text
```

**The LLM did the creative work.** Z3 did parameter search over 16 options.

### The Gap

| Vision Document Claims | Reality |
|------------------------|---------|
| "Z3 synthesizes: `(* x x)`" | Doesn't work — Z3 can't evaluate LISP |
| "Z3 generates program from spec" | Z3 finds parameters; human writes template |
| "Constraint → Correct Program" | Constraint → Parameters for human's template |

### Why This Matters

The vision of **natural language → specification → synthesized program** has a hole in it:

```
Human → LLM → Kleis Constraint → ??? → Program
                                  ↑
                    Z3 CAN'T DO THIS STEP
                    (for recursive programs over ADTs)
```

Z3 can:
- ✅ Verify bounded instances (sort 2 elements, sort 3 elements)
- ✅ Find parameters in a finite search space
- ✅ Prove local properties (insert preserves sortedness)

Z3 cannot:
- ❌ Synthesize recursive programs from scratch
- ❌ Evaluate LISP programs symbolically
- ❌ Handle `∀ xs : List` without bounding

### Possible Paths Forward

**1. Bounded Synthesis**
Limit list lengths. Works for small programs.
```kleis
// Bound lists to length ≤ 3
:sat ∃(P : SExpr). 
    length(P) ≤ 10 ∧
    eval_bounded(P, [1,2], 5) = [1,2] ∧
    eval_bounded(P, [2,1], 5) = [1,2]
```

**2. Syntax-Guided Synthesis (SyGuS)**
Use specialized synthesis tools (CVC5 SyGuS, etc.) instead of raw Z3.

**3. Enumerate-and-Verify**
Generate candidate programs, verify each:
```kleis
for each P in grammar_enumeration(size ≤ 10):
    if :check is_sorted(eval(P, test1)) ∧ is_permutation(...):
        return P
```

**4. Sketch-Based (What We Did)**
Human provides template, Z3 fills holes. Works but loses the magic.

**5. Hybrid: LLM Proposes, Z3 Verifies**
Let LLM generate LISP programs, use Z3 to verify properties.
This actually works! But it's verification, not synthesis.

### Honest Assessment

The **verification** story is solid:
- We can parse programs
- We can evaluate them concretely
- We can verify bounded properties
- Counterexamples are useful

The **synthesis** story needs work:
- True synthesis from grammar: **NOT ACHIEVED**
- Sketch-based synthesis: Works, but human does the creative part
- LLM-propose-Z3-verify: Promising but not true synthesis

### The Open Research Problem

**How do you synthesize recursive programs from specifications?**

This is not solved. Approaches being researched:

1. **SyGuS (Syntax-Guided Synthesis)**: CVC5, other tools
2. **CEGIS (Counterexample-Guided Inductive Synthesis)**: Iterate refinement
3. **Neural-guided search**: Use ML to prune grammar exploration
4. **Bounded model checking**: Limit recursion depth
5. **Deductive synthesis**: Derive program from proof

We tried (1) implicitly by encoding grammar as choices.
It works for non-recursive programs.
It fails for recursive programs over unbounded data.

**The difficulty we encountered is fundamental, not implementation.**

Z3 (and SMT solvers generally) cannot:
- Symbolically execute recursive functions
- Search infinite spaces (unbounded ADTs)
- Handle `∀(xs : List). property(xs)` without bounding

This is why true program synthesis remains an active research area.
We didn't fail because of bad engineering.
We failed because **the problem is hard**.

### Dream vs. Reality

| Aspect | The Dream | The Reality (Dec 2024) |
|--------|-----------|------------------------|
| **Synthesis** | `spec → Z3 → program` | `spec → human template → Z3 fills holes` |
| **Creativity** | Z3 explores grammar | Human designs structure, Z3 picks params |
| **Recursion** | Z3 synthesizes recursive code | Z3 times out on recursive evaluation |
| **Guarantee** | Correct for all inputs | Verified for bounded test cases |
| **Workflow** | Specification-first | LLM-proposes, Z3-verifies |

### What's Actually Achievable Now

```
┌─────────────────────────────────────────────────────────────┐
│  Developer: "Sort a list"                                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  LLM: Generates LISP sort function (might have bugs)        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Kleis/Z3: Verifies properties hold                         │
│  - is_sorted(sort(xs))? ✅                                  │
│  - is_permutation(sort(xs), xs)? ✅                         │
│  - Or: counterexample at xs = [3,1,2]                       │
└─────────────────────────────────────────────────────────────┘
```

**LLM synthesizes, Z3 verifies.** Not the other way around.

This is still valuable! But it's verification, not synthesis.

---

## The Path Forward

### Immediate (This Branch)
- [x] LISP parser in Kleis
- [x] LISP evaluator in Kleis
- [x] `:eval` command for concrete execution
- [x] Sketch-based synthesis experiments
- [ ] Document synthesis limitations in manual

### Near-Term
- [ ] LLM-propose, Z3-verify workflow
- [ ] Bounded synthesis for small programs
- [ ] SyGuS integration research

### Long-Term Vision
- [ ] True grammar-based synthesis (requires advances in SMT)
- [ ] Kleis-in-Kleis (self-hosting)
- [ ] Industrial verification workflows

---

## Conclusion

The implementation of a LISP interpreter in Kleis is a technical achievement. It demonstrates that:

1. **Programs can be data** — fully introspectable, analyzable ✅ Achieved
2. **Specifications can be axioms** — mathematical statements about behavior ✅ Achieved
3. **Verification can be automatic** — Z3 proves or finds counterexamples ✅ Achieved (bounded)
4. **Software can be correct by construction** — not just tested, but proven ⚠️ Partial

What remains a dream:

5. **Programs can be synthesized from specs** — ❌ Not achieved for recursive programs
6. **Full automation** — ❌ Human still writes templates, Z3 fills holes

We built the foundation. The dream of `spec → program` requires advances we don't have yet.

**The honest summary:**

| What | Status |
|------|--------|
| LISP interpreter in Kleis | ✅ Works |
| `:eval` for concrete execution | ✅ Works |
| Bounded verification | ✅ Works |
| Sketch-based parameter search | ✅ Works |
| True recursive program synthesis | ❌ Failed |
| `spec → Z3 → LISP program` | ❌ Dream |

---

*"Beware of bugs in the above code; I have only proved it correct, not tried it."*
— Donald Knuth

*"And beware of dreams in the above vision; I have only described it, not achieved it."*
— This experiment, December 2024

