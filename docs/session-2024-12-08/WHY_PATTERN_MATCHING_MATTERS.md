# Why Pattern Matching Matters for Kleis

**Date:** December 8, 2024  
**Context:** After implementing user-defined parametric types  
**Question:** "What will pattern matching give us?"

---

## The Core Problem

**Without pattern matching, ADTs are "write-only":**

```kleis
data Option(T) = None | Some(T)

// Can CREATE values: ✅
let x = Some(5)
let y = None

// But can't USE them! ❌
// How do we:
// - Extract the value from Some(5)?
// - Check if it's None or Some?
// - Handle both cases?
// - Get type safety?
```

**Pattern matching makes ADTs usable.**

---

## 10 Concrete Benefits

### 1. Complete Algebraic Data Types

**Before (Incomplete):**
```kleis
data Option(T) = None | Some(T)
// Defined but useless - can't extract values
```

**After (Complete):**
```kleis
data Option(T) = None | Some(T)

match x {
  None => 0
  Some(value) => value  // ✅ Extract and use!
}
```

**Impact:** ADTs become fully functional

---

### 2. Metalanguage for CS Papers

**ADR-021 Vision:** "Enable CS researchers to write executable formalisms"

```kleis
// Define lambda calculus in Kleis:
data LambdaTerm =
  | Var(String)
  | Abs(String, LambdaTerm)
  | App(LambdaTerm, LambdaTerm)

// Implement evaluation WITH PATTERN MATCHING:
operation eval : LambdaTerm → Value

define eval(term) = match term {
  Var(x) => lookup(context, x)
  Abs(x, body) => Closure(x, body, context)
  App(e1, e2) => {
    let func = eval(e1)
    let arg = eval(e2)
    apply(func, arg)
  }
}
```

**Impact:** CS papers become executable! The formalism runs!

**Use cases:**
- Lambda calculus semantics
- Type system rules
- Operational semantics
- Abstract machines
- Proof systems

---

### 3. Self-Hosting: Type Checker in Kleis

**Current:** Type checking hardcoded in Rust  
**Vision:** Type checking written in Kleis!

```kleis
data Type =
  | Scalar
  | Vector(Nat)
  | Matrix(Nat, Nat)
  | Var(Nat)
  | Function(Type, Type)

// UNIFICATION ALGORITHM IN KLEIS:
operation unify : Type → Type → Option(Substitution)

define unify(t1, t2) = match (t1, t2) {
  (Scalar, Scalar) => Some(empty_subst)
  
  (Vector(n1), Vector(n2)) if n1 == n2 => Some(empty_subst)
  
  (Matrix(m1,n1), Matrix(m2,n2)) if m1==m2 && n1==n2 => 
    Some(empty_subst)
  
  (Var(id), t) => Some(singleton(id, t))
  (t, Var(id)) => Some(singleton(id, t))
  
  (Function(a1,b1), Function(a2,b2)) => {
    match (unify(a1, a2), unify(b1, b2)) {
      (Some(s1), Some(s2)) => Some(compose(s1, s2))
      _ => None
    }
  }
  
  _ => None  // Type mismatch
}
```

**Impact:** Kleis defines Kleis! (Meta-circularity Level 3)

**Benefits:**
- Type rules are now Kleis code (not Rust)
- Users can extend type system
- Type checker is transparent and modifiable
- True self-hosting achieved

---

### 4. Functional Error Handling

**Without exceptions, using Result type:**

```kleis
data Result(T, E) = Ok(T) | Err(E)

operation safeDivide : ℝ → ℝ → Result(ℝ, String)

define safeDivide(a, b) = match b {
  0 => Err("Division by zero")
  _ => Ok(a / b)
}

// Chain operations safely:
let result = safeDivide(10, x)

match result {
  Ok(value) => value * 2
  Err(msg) => {
    print(msg)
    0
  }
}

// No exceptions, all errors explicit in types!
```

**Impact:** Type-safe error handling without exceptions

**Benefits:**
- All error paths explicit
- Can't forget to handle errors
- Composable error handling
- Railway-oriented programming

---

### 5. Conditional Logic (More Powerful than if/then/else)

**Simple conditions:**
```kleis
data Bool = True | False

match condition {
  True => doThis
  False => doThat
}
```

**Multi-way branches:**
```kleis
match status {
  Idle => "waiting"
  Running => "processing"
  Paused => "on hold"
  Completed => "done"
  Failed => "error"
}
```

**Tuple patterns:**
```kleis
match (isValid, hasPermission) {
  (True, True) => allow()
  (True, False) => deny("No permission")
  (False, _) => deny("Invalid request")
}
```

**Impact:** More expressive than if/then/else

**Why better:**
- Exhaustiveness checking (can't forget cases)
- Pattern destructuring (extract values)
- No deeply nested if/then/else
- Clear, declarative style

---

### 6. State Machines (Type-Safe Transitions)

```kleis
data State =
  | Idle
  | Running(taskId: String)
  | Paused(taskId: String, progress: ℝ)
  | Completed(result: String)

data Event =
  | Start(String)
  | Pause
  | Resume
  | Finish(String)

operation transition : State → Event → State

define transition(state, event) = match (state, event) {
  // Valid transitions:
  (Idle, Start(id)) => Running(id)
  (Running(id), Pause) => Paused(id, getCurrentProgress())
  (Paused(id, prog), Resume) => Running(id)
  (Running(id), Finish(result)) => Completed(result)
  
  // Invalid transitions: no-op
  _ => state
}
```

**Impact:** Explicit, type-safe state machines

**Benefits:**
- All states and transitions visible
- Type system prevents invalid transitions
- Exhaustiveness ensures no forgotten cases
- Clear documentation of behavior

---

### 7. List Processing (Recursive Data Structures)

```kleis
data List(T) = Nil | Cons(head: T, tail: List(T))

// Length (recursive):
operation length : List(T) → Nat

define length(list) = match list {
  Nil => 0
  Cons(_, tail) => 1 + length(tail)
}

// Map (higher-order):
operation map : (T → U) → List(T) → List(U)

define map(f, list) = match list {
  Nil => Nil
  Cons(head, tail) => Cons(f(head), map(f, tail))
}

// Filter:
operation filter : (T → Bool) → List(T) → List(T)

define filter(pred, list) = match list {
  Nil => Nil
  Cons(head, tail) => {
    match pred(head) {
      True => Cons(head, filter(pred, tail))
      False => filter(pred, tail)
    }
  }
}

// Fold:
operation fold : (T → U → U) → U → List(T) → U

define fold(f, acc, list) = match list {
  Nil => acc
  Cons(head, tail) => fold(f, f(head, acc), tail)
}
```

**Impact:** Full functional list processing

**Benefits:**
- Recursive data structures work
- Higher-order functions
- Standard functional patterns
- Efficient implementations

---

### 8. Type-Safe Enum Dispatch

```kleis
data Particle = Electron | Proton | Neutron | Photon

operation charge : Particle → ℝ

define charge(particle) = match particle {
  Electron => -1
  Proton => 1
  Neutron => 0
  Photon => 0
}

operation mass : Particle → ℝ

define mass(particle) = match particle {
  Electron => 9.109e-31
  Proton => 1.673e-27
  Neutron => 1.675e-27
  Photon => 0
}
```

**Impact:** Type-safe lookup tables with exhaustiveness checking

**Benefits:**
- Compiler ensures all cases handled
- Can't forget a particle type
- Add new particle: all match expressions must be updated
- Refactoring is safe

---

### 9. Quantum Mechanics (Physics Domain)

```kleis
data SpinState = Up | Down
data MeasurementResult = Measured(SpinState, Probability)

operation applyGate : Qubit → Gate → Qubit

define applyGate(qubit, gate) = match gate {
  PauliX => flipQubit(qubit)
  PauliY => rotateY(qubit)
  PauliZ => phaseFlip(qubit)
  Hadamard => superposition(qubit)
  CNOT => controlledNot(qubit)
}

// Measure and branch:
let measured = measure(qubit)

match measured {
  Measured(Up, prob) => {
    print("Measured |↑⟩ with probability", prob)
    projectUp(state)
  }
  Measured(Down, prob) => {
    print("Measured |↓⟩ with probability", prob)
    projectDown(state)
  }
}
```

**Impact:** Quantum computing formalism becomes executable

---

### 10. Tree Structures (Hierarchical Data)

```kleis
data Tree(T) =
  | Leaf(value: T)
  | Node(left: Tree(T), right: Tree(T))

operation depth : Tree(T) → Nat

define depth(tree) = match tree {
  Leaf(_) => 1
  Node(left, right) => 1 + max(depth(left), depth(right))
}

operation mapTree : (T → U) → Tree(T) → Tree(U)

define mapTree(f, tree) = match tree {
  Leaf(value) => Leaf(f(value))
  Node(left, right) => Node(mapTree(f, left), mapTree(f, right))
}

// Binary search tree operations:
operation insert : T → Tree(T) → Tree(T)

define insert(value, tree) = match tree {
  Leaf(v) => {
    match compare(value, v) {
      LT => Node(Leaf(value), Leaf(v))
      GT => Node(Leaf(v), Leaf(value))
      EQ => tree
    }
  }
  Node(left, right) => {
    // ... recursive insertion logic
  }
}
```

**Impact:** Hierarchical data structures fully supported

---

## Comparison Table

| Feature | Without Pattern Matching | With Pattern Matching |
|---------|-------------------------|----------------------|
| **ADT Usage** | Can define, can't use ❌ | Fully functional ✅ |
| **Value Extraction** | Impossible ❌ | `Some(x) => x` ✅ |
| **Exhaustiveness** | None ❌ | Compiler-checked ✅ |
| **Self-Hosting** | Impossible ❌ | Achievable ✅ |
| **Error Handling** | Exceptions only ❌ | Result + match ✅ |
| **Recursion** | Limited ❌ | Full support ✅ |
| **State Machines** | Manual dispatch ❌ | Type-safe ✅ |
| **Metalanguage** | Not possible ❌ | CS papers run ✅ |

---

## What You Lose Without It

**Current state (after tonight's work):**

✅ Can define: `data Option(T) = None | Some(T)`  
✅ Can use in types: `structure Optionable(T) { operation unwrap : Option(T) → T }`  
❌ **Can't implement unwrap!** No way to extract value from Some(x)

**With pattern matching:**

✅ Can define data types  
✅ Can use in signatures  
✅ **Can implement operations!** Extract, transform, handle all cases

---

## Real-World Use Cases Enabled

### Academia
- **Executable formalisms** - Lambda calculus, type systems
- **Proof assistants** - Theorem proving with pattern matching
- **Programming language research** - Operational semantics

### Physics
- **Quantum computing** - Gate application, measurement handling
- **Particle physics** - Type-safe particle properties
- **State evolution** - Match on states, compute transitions

### Software Engineering
- **Error handling** - Result types with exhaustiveness
- **State machines** - Type-safe transitions
- **Data processing** - Lists, trees, recursive structures

---

## The Self-Hosting Vision (ADR-021)

### Level 1: Parser in Rust ✅
```
Kleis text → (Rust parser) → AST
```

### Level 2: Types in Kleis ✅ (Done tonight!)
```kleis
// stdlib/types.kleis
data Type = Scalar | Vector(Nat) | Matrix(Nat, Nat) | ...
```

### Level 3: Type Checker in Kleis (Requires Pattern Matching!)
```kleis
// stdlib/type_checker.kleis
operation infer : Expression → Type

define infer(expr) = match expr {
  Const(_) => Scalar
  Object(name) => lookup(context, name)
  Operation(name, args) => inferOperation(name, args)
  Match(scrutinee, cases) => inferMatch(scrutinee, cases)
}

operation unify : Type → Type → Option(Substitution)

define unify(t1, t2) = match (t1, t2) {
  (Scalar, Scalar) => Some(empty)
  (Var(a), t) => Some(bind(a, t))
  (t, Var(a)) => Some(bind(a, t))
  (Function(a1,b1), Function(a2,b2)) => composeUnifications(...)
  _ => None
}
```

**Impact:** The language defines itself! Meta-circularity achieved!

**Why this matters:**
- Type rules are transparent (in Kleis, not hidden in Rust)
- Users can understand and modify type system
- Type system becomes extensible by users
- Ultimate self-hosting goal

---

## Comparison: With vs Without

### Without Pattern Matching (Current)

**What works:**
```kleis
data Option(T) = None | Some(T)  // ✅ Can define

// Can use in types:
structure Optionable(T) {
  operation isSome : Option(T) → Bool  // ✅ Types work
}
```

**What doesn't work:**
```kleis
// Can't implement operations! ❌
implements Optionable(ℝ) {
  operation isSome = ???  // How do we check None vs Some???
}

// Can't extract values:
let x = Some(5)
let value = ???  // No way to get the 5 out! ❌

// Can't handle both cases:
if x is Some {  // No such syntax exists! ❌
  ...
}
```

### With Pattern Matching (Future)

**Everything works:**
```kleis
data Option(T) = None | Some(T)

// Can define types: ✅
structure Optionable(T) {
  operation isSome : Option(T) → Bool
  operation unwrap : Option(T) → T
  operation getOrElse : Option(T) → T → T
}

// Can implement operations: ✅
implements Optionable(ℝ) {
  operation isSome = λ opt. match opt {
    None => False
    Some(_) => True
  }
  
  operation unwrap = λ opt. match opt {
    None => error("Unwrap on None!")
    Some(x) => x
  }
  
  operation getOrElse = λ opt default. match opt {
    None => default
    Some(x) => x
  }
}

// Can use in practice: ✅
let maybeValue = Some(42)

match maybeValue {
  None => print("No value")
  Some(x) => print("Value is", x)
}
```

---

## Technical Benefits

### 1. Exhaustiveness Checking

```kleis
data Status = Idle | Running | Paused | Completed

match status {
  Idle => startTask()
  Running => continueTask()
  Paused => resumeTask()
  // Missing: Completed case!
}

// Compiler WARNING: ⚠️
// Non-exhaustive match: missing case for 'Completed'
```

**Benefit:** Compiler catches forgotten cases - prevents bugs!

---

### 2. Unreachability Detection

```kleis
match value {
  Some(x) => x
  _ => 0
  None => -1  // ⚠️ Unreachable! Wildcard caught everything
}

// Compiler WARNING: ⚠️
// Unreachable pattern: None will never match (caught by wildcard)
```

**Benefit:** Detects dead code automatically

---

### 3. Type-Guided Refactoring

```kleis
// Add new constructor:
data Status = Idle | Running | Paused | Completed | Failed  // + Failed

// Compiler finds ALL match expressions on Status:
// ❌ match in function1: missing Failed case
// ❌ match in function2: missing Failed case  
// ❌ match in function3: missing Failed case
```

**Benefit:** Safe refactoring - compiler guides you

---

### 4. Variable Binding Scope

```kleis
match result {
  Ok(value) => {
    // 'value' is in scope here ✅
    process(value)
  }
  Err(error) => {
    // 'error' is in scope here ✅
    // 'value' is NOT in scope (correctly!)
    handle(error)
  }
}

// Neither 'value' nor 'error' in scope here ✓
```

**Benefit:** Lexically scoped bindings, no variable leakage

---

## Functional Programming Unlocked

With pattern matching, Kleis gains full functional programming power:

### Map/Filter/Reduce

```kleis
data List(T) = Nil | Cons(T, List(T))

// All the classic functions work:
map(f, list)
filter(pred, list)
fold(f, acc, list)
zip(list1, list2)
flatten(nestedList)
```

### Higher-Order Functions

```kleis
operation compose : (B → C) → (A → B) → (A → C)

define compose(g, f) = λ x. g(f(x))

// Pipeline with pattern matching:
let result = 
  data
  |> filter(isValid)
  |> map(transform)
  |> fold(combine, empty)
  |> match {
       Nil => None
       Cons(x, _) => Some(x)
     }
```

### Monadic Operations

```kleis
operation bind : Option(A) → (A → Option(B)) → Option(B)

define bind(opt, f) = match opt {
  None => None
  Some(x) => f(x)
}

// Monadic chaining:
result = bind(step1(), λ x.
         bind(step2(x), λ y.
         Some(combine(x, y))))
```

---

## What Happens If We Don't Implement It?

**ADTs remain incomplete:**
- Users define types but can't use them
- Self-hosting vision blocked
- Metalanguage use cases impossible
- Kleis is "half a language"

**Workarounds would be ugly:**
```kleis
// Without match, would need operations for each constructor:
operation isSome : Option(T) → Bool  // Manual check
operation isNone : Option(T) → Bool  // Manual check
operation unwrapSome : Option(T) → T  // Unsafe! Can fail
operation unwrapOr : Option(T) → T → T  // Need many variants

// Still can't handle user-defined types!
data MyType = A | B | C
// Would need to manually define isA, isB, isC, unwrapA, unwrapB, unwrapC...
// Doesn't scale! ❌
```

---

## Priority Assessment

### Must Have (Core Language Feature)

**Without pattern matching:**
- ADTs are decorative only
- Can't write recursive functions
- Can't implement type checker in Kleis
- Self-hosting vision blocked

**With pattern matching:**
- ADTs fully functional ✅
- Recursive functions work ✅
- Self-hosting possible ✅
- Complete functional language ✅

### Impact on Project Goals

**ADR-021 Vision:** "Meta-circular type system"
- **Blocked** without pattern matching
- **Achievable** with pattern matching

**ADR-003 Self-Hosting:** "Kleis defines Kleis"
- **Incomplete** without pattern matching (can define types, not operations)
- **Complete** with pattern matching (can write type checker in Kleis)

**User Extensibility:**
- **Limited** without pattern matching (define types, can't use them)
- **Full** with pattern matching (define types, write operations on them)

---

## Bottom Line

### Pattern Matching is NOT Optional

**It's the difference between:**
- "Kleis has data types" (nice feature)
- "Kleis is a complete functional language" (transformative)

**It enables:**
1. ✅ Complete ADTs (not just definition)
2. ✅ Self-hosting (type checker in Kleis)
3. ✅ Metalanguage (CS papers become code)
4. ✅ Functional programming (recursion, higher-order functions)
5. ✅ Type safety (exhaustiveness checking)

**Without it:**
- ADTs are incomplete
- Self-hosting blocked
- Half a language

**With it:**
- Full functional language
- Self-hosting possible
- Production-ready

---

## Estimated Value

**Implementation cost:** 6-8 hours  
**Impact:** Transforms Kleis from "partial" to "complete" functional language  
**ROI:** Extremely high

**Comparison:**
- User-defined types (tonight): 4 hours → Extensibility ✅
- Pattern matching (next): 6-8 hours → **Makes types actually usable** ✅✅✅

**Without pattern matching, tonight's work is only 50% complete.**  
**With pattern matching, ADR-021 vision is 100% achieved.**

---

## Recommendation

**Priority:** **HIGHEST** - This is the missing piece

**Timeline:**
- Next session: Implement parser + type inference (4 hours)
- Following session: Evaluation + exhaustiveness (3 hours)  
- Result: Complete, self-hosting-capable functional language

**Impact:** From "interesting project" to "production-ready functional language for scientific computing"

---

**Status:** Critical feature for Kleis completeness  
**Recommendation:** Prioritize for next session  
**Expected outcome:** Self-hosting meta-circular type system! 🎯

