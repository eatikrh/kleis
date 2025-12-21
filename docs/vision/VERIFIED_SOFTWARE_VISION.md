# Verified Software: The Kleis Vision

*Date: December 21, 2024*

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

## The Path Forward

### Immediate (This Branch)
- [x] LISP parser in Kleis
- [x] LISP evaluator in Kleis
- [ ] Add `(verify ...)` form to LISP
- [ ] Document in the manual

### Near-Term
- [ ] Verify properties of the LISP interpreter itself
- [ ] Add more LISP features (`define`, `set!`, macros)
- [ ] Create examples of verified LISP programs

### Long-Term Vision
- [ ] Kleis-in-Kleis (self-hosting)
- [ ] Verified Kleis compiler
- [ ] Industrial verification workflows

---

## Conclusion

The implementation of a LISP interpreter in Kleis is not just a technical achievement. It demonstrates that:

1. **Programs can be data** — fully introspectable, analyzable
2. **Specifications can be axioms** — mathematical statements about behavior
3. **Verification can be automatic** — Z3 proves or finds counterexamples
4. **Software can be correct by construction** — not just tested, but proven

This is the path to software we can truly trust.

---

*"Beware of bugs in the above code; I have only proved it correct, not tried it."*
— Donald Knuth

