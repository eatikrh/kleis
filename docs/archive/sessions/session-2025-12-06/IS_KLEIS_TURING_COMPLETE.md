# Is Kleis Turing Complete?

**Date:** December 6, 2025  
**Question:** Is Kleis Turing complete?  
**Answer:** ⚠️ **It depends on what you mean by "Kleis"**

---

## TL;DR

**Short answer:** ⚠️ **Syntax yes, semantics no (by design)**

**Kleis has Turing-complete SYNTAX but is NOT designed to be a Turing-complete EVALUATOR.**

---

## What Makes a Language Turing Complete?

### Requirements

1. ✅ **Conditional branching** (if/then/else)
2. ✅ **Unbounded memory** (data structures)
3. ✅ **Recursion or loops** (iteration)

### Equivalently

Can simulate a Turing machine OR has:
- Lambda calculus + recursion (Church-Turing thesis)
- Primitive recursion
- While loops + conditionals

---

## What Kleis Grammar Has

### From Kleis_v03.g4

**1. Conditionals** ✅
```antlr
conditional: 'if' expression 'then' expression 'else' expression
```

**Example:**
```kleis
define abs(x) = if x < 0 then -x else x
```

**2. Lambda Calculus** ✅
```antlr
lambda: 'λ' params '.' expression
      | 'lambda' params '.' expression
```

**Example:**
```kleis
define double = λ x . x + x
```

**3. Function Definitions** ✅
```antlr
functionDef: 'define' IDENTIFIER '(' params ')' '=' expression
```

**Example:**
```kleis
define factorial(n) = 
    if n == 0 then 1 else n * factorial(n - 1)
```

**4. Let Bindings** ✅
```antlr
letBinding: 'let' IDENTIFIER '=' expression 'in' expression
```

**Example:**
```kleis
let x = 5 in x * x
```

**5. Data Structures** ✅
```antlr
'[' expressions ']'  // Lists/vectors
```

---

## Theoretical Analysis

### Could You Implement a Turing Machine?

**Yes, syntactically!**

```kleis
// Turing machine state
structure TMState {
    tape : List(Symbol)
    head : ℕ
    state : State
}

// Transition function
define step(tm : TMState) : TMState =
    if tm.state == q0 then
        if read(tm.tape, tm.head) == '0' then
            write(tm.tape, tm.head, '1')
            move_right(tm)
        else ...
    else ...

// Recursion for execution
define run(tm : TMState) : TMState =
    if tm.state == halt then tm
    else run(step(tm))
```

**Syntactically valid Kleis!**

---

## But Here's the Critical Point

### Kleis is NOT an Evaluator (By Design)

**From ADR-002 and design docs:**

> "Kleis maintains expressions as **symbolic**, not evaluated"

**Kleis does:**
- ✅ Parse expressions
- ✅ Type check them
- ✅ Verify axioms
- ✅ Render beautifully
- ❌ **NOT evaluate/execute them**

**Example:**
```kleis
define factorial(n) = if n == 0 then 1 else n * factorial(n - 1)

// Kleis does:
✅ Parse: factorial is a function
✅ Type check: ℕ → ℕ
✅ Render: factorial(n) = { 1 if n=0; n·factorial(n-1) otherwise }

// Kleis does NOT:
❌ Execute: factorial(5) → 120
```

**The expression STAYS SYMBOLIC.**

---

## Three Interpretations

### 1. Kleis Syntax Language: ✅ Turing Complete

**The grammar has:**
- Conditionals
- Lambda calculus
- Recursion
- Data structures

**Theoretically:** Could express any computable function

**Status:** ✅ **Turing complete as a syntax**

---

### 2. Kleis Type System: ❌ Not Turing Complete

**Type checking is decidable:**
- Hindley-Milner is decidable
- Type checking always terminates
- No arbitrary computation in types

**This is GOOD!** Type checking should terminate.

**Status:** ❌ **Not Turing complete (intentionally)**

---

### 3. Kleis Evaluation Engine: ⚠️ Intentionally Limited

**Current design:**
- Symbolic manipulation (not evaluation)
- Partial evaluation possible
- No general recursion execution

**From KLEIS_EVALUATION_SYNTAX.md:**
```kleis
expr.eval(context) → Expression | Value
// Returns symbolic OR numeric, but doesn't execute arbitrary recursion
```

**Status:** ⚠️ **Not Turing complete (by design - it's symbolic, not executable)**

---

## Comparison to Other Systems

| System | Syntax TC? | Evaluation TC? | Purpose |
|--------|-----------|----------------|---------|
| **Haskell** | ✅ Yes | ✅ Yes | Programming language |
| **LaTeX** | ❌ No | N/A | Typesetting |
| **TeX macros** | ✅ Yes | ✅ Yes | Macro expansion (bad!) |
| **Mathematica** | ✅ Yes | ✅ Yes | Computer algebra |
| **SymPy** | N/A (Python) | ✅ Yes | Symbolic computation |
| **Coq/Agda** | ✅ Yes | ⚠️ Terminating | Proof assistants |
| **Kleis** | ✅ Yes | ❌ **No (symbolic)** | **Symbolic verification** |

---

## The Design Philosophy

### Kleis is More Like Coq Than Haskell

**Coq:**
- Turing-complete syntax
- But all functions must TERMINATE
- Purpose: Proofs, not general computation

**Kleis:**
- Turing-complete syntax
- But expressions stay SYMBOLIC
- Purpose: Verification, not execution

### Why Kleis Should NOT Execute Arbitrary Code

```kleis
// Infinite loop in Kleis
define loop(x) = loop(x)

// What should happen?
// Option A: Execute forever ❌ (bad for editor!)
// Option B: Keep symbolic ✅ (Kleis choice)
//   Result: "loop(5)" stays as "loop(5)"
//   Type: loop : ∀T. T → T
```

**If Kleis executed arbitrary recursion:**
- Editor could hang
- Type checking could diverge
- Loading files could freeze

**By keeping symbolic:**
- Always responsive
- Type checking terminates
- Can reason about infinite objects

---

## Could Kleis Become Turing Complete?

### If You Added Evaluation

**Yes! With an interpreter:**

```rust
fn eval(expr: &Expression, ctx: &Context) -> Result<Value, Error> {
    match expr {
        Expression::Const(n) => Ok(Value::Number(n.parse()?)),
        
        Expression::Operation { name: "if_then_else", args } => {
            let condition = eval(&args[0], ctx)?;
            if condition.is_true() {
                eval(&args[1], ctx)  // then branch
            } else {
                eval(&args[2], ctx)  // else branch
            }
        }
        
        Expression::Operation { name: "apply", args } => {
            // Function application
            let func = eval(&args[0], ctx)?;
            let arg = eval(&args[1], ctx)?;
            apply_function(func, arg, ctx)  // Could recurse!
        }
        
        _ => ...
    }
}
```

**With this interpreter + recursion:** ✅ Turing complete!

**But Kleis doesn't do this!**

---

## The Kleis Model

### Symbolic Manipulation, Not Execution

```kleis
// Define factorial
define factorial(n) = if n == 0 then 1 else n * factorial(n - 1)

// Type check
// ✅ factorial : ℕ → ℕ

// Use it
result = factorial(5)

// Kleis returns:
// NOT: 120 (evaluation)
// BUT: "factorial(5)" (symbolic)
//      OR: "5 × factorial(4)" (one step)
//      OR: kept as "factorial(5)" until explicitly evaluated
```

**Key:** Expressions stay symbolic unless user explicitly evaluates.

---

## Answer to Your Question

### Is Kleis Turing Complete?

**Syntax:** ✅ **Yes** - Has lambda, conditionals, recursion

**Semantics:** ❌ **No** - Doesn't execute arbitrary computation

**By Design:** Kleis is a **verification system**, not an execution engine.

### Comparison

**Turing Complete Languages:**
- Haskell, Python, JavaScript
- Execute arbitrary programs
- Can hang, loop forever

**Kleis:**
- Verifies symbolic expressions
- Type checks
- Doesn't execute (stays symbolic)
- Always terminates

**Closer to:**
- Coq (proof assistant - terminating)
- LaTeX (typesetting - not computational)
- Type systems (decidable, terminating)

---

## Could It Be Both?

### Option: Add Evaluation Mode

```kleis
// Symbolic mode (default)
result = factorial(5)  // → "factorial(5)" (symbolic)

// Evaluation mode (explicit)
result = eval(factorial(5))  // → 120 (computed)

// With termination checking
result = eval_with_limit(factorial(5), max_steps: 1000)
```

**This would make Kleis:**
- Turing complete in eval mode
- Terminating in symbolic mode

**Benefit:** Best of both worlds!

---

## Recommendation

### For Now: Keep Symbolic (Not Turing Complete)

**Rationale:**
1. Type checking is the priority
2. Symbolic manipulation is the core
3. Can add evaluation later if needed
4. Keeps system predictable and fast

### Future: Add Optional Evaluation

**When needed:**
- Add `eval()` function
- With termination limits
- Opt-in, not default
- For numerical computation

**Timeline:** After type checking POC is complete

---

## Conclusion

**Question:** Is Kleis Turing complete?

**Answer:** 
- **Syntax:** ✅ Yes (has lambda, conditionals, recursion)
- **Semantics:** ❌ No (symbolic, not evaluating)
- **By design:** Verification system, not programming language
- **Could be:** Add evaluation mode if needed

**Current focus:** Type checking and verification (which should TERMINATE)

**Status:** ⚠️ **Intentionally NOT Turing complete in evaluation semantics**

---

**Comparison:**
- More like Coq (proof assistant) than Haskell (programming)
- More like LaTeX (symbolic) than Python (executable)
- Purpose: Verify mathematics, not execute programs

