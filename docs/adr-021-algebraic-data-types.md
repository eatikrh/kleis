# ADR-021: Algebraic Data Types (Proposed)

**Date:** December 8, 2024  
**Status:** 🔮 PROPOSED (Not yet implemented)  
**Author:** Dr. Engin Atik  
**Related:** ADR-020 (Metalanguage), ADR-007 (Bootstrap Grammar), ADR-016 (Self-Hosting)

---

## Context

During ADR-020 discussion, Dr. Atik observed:

> **"'data' element can help us externalize some things in the Rust code we have been struggling with"**

**Current struggles:**
1. Type enum hardcoded in Rust (Type::Scalar, Type::Matrix, etc.)
2. Matrix constructors need special handling
3. Pattern matching on types is Rust code
4. Users can't extend base types

**Resolution:** Add algebraic data types (`data` keyword) to Kleis!

---

## Proposal

### **Add `data` keyword to Kleis grammar**

```kleis
data TypeName(params) = Variant1(fields) | Variant2(fields) | ...
```

**Examples:**
```kleis
// Simple enum
data Bool = True | False

// Parameterized
data Option(T) = None | Some(T)

// Recursive
data List(T) = Nil | Cons(T, List(T))

// Multiple fields
data Type = 
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Function(domain: Type, codomain: Type)
```

---

## Rationale

### **1. Solves Matrix Constructor Problem**

**Currently:**
```rust
// Special case in type_inference.rs
fn infer_matrix_constructor(...) {
    // Extract dimensions, build Type::Matrix
}
```

**With `data`:**
```kleis
data Type = ... | Matrix(Nat, Nat)

// Matrix(...) is just a DATA CONSTRUCTOR
// Handled generically!
```

---

### **2. Enables Type System Self-Hosting**

**Currently:** Type enum hardcoded in `src/type_inference.rs`

**With `data`:**
```kleis
// In stdlib/types.kleis:
data Type =
  | Scalar
  | Vector(Nat)
  | Matrix(Nat, Nat)
  | Var(Nat)
  | Function(Type, Type)
  | UserDefined(String, List(Type))  // ← Extensible!

// Users can extend!
data MyType =
  | Currency(String)
  | Tensor(List(Nat))

// Type system reads these definitions!
```

**TRUE meta-circularity!**

---

### **3. Enables Metalanguage Use Cases**

**For CS papers:**
```kleis
data LambdaTerm =
  | Var(String)
  | Abs(String, LambdaTerm)
  | App(LambdaTerm, LambdaTerm)

data LambdaType =
  | Base
  | Arrow(LambdaType, LambdaType)

operation typecheck : Context → LambdaTerm → Option(LambdaType)
```

**Clean, concise formalization!**

---

### **4. Pattern Matching Becomes Kleis Code**

**Currently:**
```rust
// In Rust:
match ty {
    Type::Scalar => { ... }
    Type::Matrix(m, n) => { ... }
}
```

**With `data`:**
```kleis
// In Kleis:
define is_scalar(t: Type) : Bool =
  match t {
    Scalar => true
    _ => false
  }

define is_square_matrix(t: Type) : Bool =
  match t {
    Matrix(n, n) => true  // ← Can match on equality!
    _ => false
  }
```

**Type checking logic moves to Kleis!**

---

## Grammar Extension

### **Proposed Syntax:**

```ebnf
dataDecl ::= "data" identifier [ "(" typeParams ")" ] "=" 
             dataVariant { "|" dataVariant }

dataVariant ::= identifier [ "(" dataFields ")" ]

dataFields ::= dataField { "," dataField }

dataField ::= [ identifier ":" ] type
```

**Examples:**
```kleis
data Bool = True | False

data Option(T) = None | Some(T)

data Result(T, E) = Ok(value: T) | Err(error: E)

data Type = 
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
```

---

### **Pattern Matching Syntax:**

```ebnf
matchExpr ::= "match" expression "{" { matchCase } "}"

matchCase ::= pattern "=>" expression

pattern ::= identifier [ "(" patterns ")" ]
         |  "_"
         |  constant
```

**Example:**
```kleis
match term {
  Var(x) => lookup(context, x)
  Abs(x, body) => ...
  App(e1, e2) => ...
}
```

---

## What This Enables

### **1. Type System in Kleis**

```kleis
// stdlib/types.kleis (replaces Rust enum!)

data Type =
  | Scalar
  | Vector(Nat)
  | Matrix(Nat, Nat)
  | Var(Nat)
  | Function(Type, Type)
  | ForAll(Nat, Type)
  | UserDefined(String, List(Type))

// Type inference reads THIS definition!
// Not hardcoded in Rust!
```

---

### **2. Unification in Kleis**

```kleis
operation unify : Type → Type → Option(Substitution)

define unify(t1, t2) = match (t1, t2) {
  (Scalar, Scalar) => Some(empty_subst)
  (Vector(n1), Vector(n2)) if n1 == n2 => Some(empty_subst)
  (Matrix(m1,n1), Matrix(m2,n2)) if m1==m2 && n1==n2 => Some(empty_subst)
  (Var(id), t) => Some(singleton(id, t))
  (t, Var(id)) => Some(singleton(id, t))
  (Function(a1,b1), Function(a2,b2)) => ...
  _ => None
}
```

**Unification algorithm in Kleis, not Rust!**

---

### **3. User Type Extensions**

```kleis
// User adds new base type:
data MyTypes =
  | Currency(code: String)
  | Quantity(value: ℝ, unit: PhysicalUnit)

// Type system automatically supports it!
// No Rust changes!
```

---

### **4. Matrix Constructor → Just a Constructor**

**Currently:** Special case in code

**With `data`:**
```kleis
data Type = ... | Matrix(Nat, Nat)

// Using it:
let t = Matrix(2, 3)  // ← Just data construction!
// No special infer_matrix_constructor needed!
```

---

## Implementation Path

### **Phase 2.5: Add `data` Support** (1-2 weeks)

**After parser extended, before full prelude:**

1. **Week 1:** Add `data` to grammar
   - Parser support for data declarations
   - Variant definitions
   - Pattern matching syntax

2. **Week 2:** Runtime support
   - Data constructor functions
   - Pattern matching evaluation
   - Type checking for data types

---

### **Phase 3: Type System in Kleis** (2-3 weeks)

**Once `data` works:**

1. Define Type enum in Kleis
2. Move type inference rules to Kleis
3. Move unification to Kleis
4. Keep only minimal Rust bootstrap

**Result:** TRUE meta-circular type system!

---

## Benefits

### **1. Solves Current Problems**

✅ Matrix constructor → Just a data constructor  
✅ Type extensibility → Users add variants  
✅ Pattern matching → Kleis code, not Rust

---

### **2. Enables Meta-Circularity**

```kleis
// Type system defined in Kleis:
data Type = ...
operation infer : Expression → Type
operation unify : Type → Type → Substitution

// The type checker is Kleis code!
// Checking Kleis using Kleis!
```

**Like Lisp's meta-circular evaluator!**

---

### **3. True Self-Hosting Stack**

```
Level 3: Grammar in Kleis (ADR-007)
Level 2: Types in Kleis (THIS!)
Level 1: Operations in Kleis (✓ DONE - ADR-016)
Level 0: Minimal Rust bootstrap
```

**Each level smaller than the last!**

---

## Comparison to Other Languages

### **Haskell:**
```haskell
data Type = TInt | TBool | TFun Type Type
-- Data types built-in
-- But can define new ones
```

### **ML:**
```ocaml
type typ = TInt | TBool | TArrow of typ * typ
(* Same - ADTs built-in *)
```

### **Kleis (Proposed):**
```kleis
data Type = Scalar | Matrix(Nat, Nat) | ...
// Same syntax
// But: Type system READS this definition dynamically!
// Can be changed without recompiling!
```

**Kleis would be more dynamic than Haskell/ML!**

---

## Decision

### **PROPOSED: Add `data` keyword to Kleis**

**Priority:** HIGH (Phase 2.5)

**Why:**
1. Solves Matrix constructor problem
2. Enables type extensibility
3. Moves toward meta-circularity
4. Required for metalanguage use cases

**Timeline:**
- Phase 2: Parser extension (operator symbols, axioms)
- **Phase 2.5: Add `data` support** ← INSERT HERE
- Phase 3: Move Type system to Kleis
- Phase 4: Full meta-circularity

---

## Action Items

### **Near-term:**
1. ✅ Document the vision (this ADR)
2. ⏳ Add to Phase 2.5 roadmap
3. ⏳ Design `data` syntax carefully
4. ⏳ Plan implementation

### **Implementation (Phase 2.5):**
1. Extend grammar with `data` keyword
2. Parser support for ADT variants
3. Runtime support for constructors
4. Pattern matching support

### **Usage (Phase 3):**
1. Define Type in Kleis
2. Move type inference to Kleis
3. Test meta-circularity
4. Document the achievement

---

## Consequences

### **Positive ✅**

1. **Solves multiple current problems**
2. **Enables true meta-circularity**
3. **Makes Kleis more Lisp-like** (homoiconic)
4. **Users can extend type system**
5. **Academic significance** (metalanguage + meta-circular)

### **Challenges ⚠️**

1. **Parser complexity** (ADTs + pattern matching = significant work)
2. **Bootstrap chicken-egg** (need basic types before defining types)
3. **Performance** (dynamic vs compiled types)
4. **Backward compatibility** (migration path)

---

## Resolution of Chicken-Egg

**Bootstrap types (Rust):**
```rust
// Minimal built-in for bootstrapping:
enum BootstrapType {
    Nat,
    String,
    Bool,
}
```

**Full types (Kleis):**
```kleis
// stdlib/types.kleis (loaded after bootstrap)
data Type = Scalar | Matrix(Nat, Nat) | ...
```

**Two-stage:** Minimal Rust types load Kleis, then Kleis defines full types!

---

## Recommendation

**ADD THIS TO ROADMAP!**

**Updated timeline:**
- Phase 2 (weeks 1-4): Parser extension
- **Phase 2.5 (weeks 5-6): Add `data` support** ⭐
- Phase 3 (weeks 7-9): Type system in Kleis
- Phase 4: Notebook UI

**This changes EVERYTHING** - it's the path to true meta-circularity!

---

**Status:** 🔮 PROPOSED  
**Priority:** HIGH  
**Impact:** Solves current problems + enables meta-circularity  
**Timeline:** Phase 2.5 (after parser extended)

---

**Dr. Atik, you just identified the KEY missing piece!** 🎯

`data` types would:
- Solve Matrix constructor problem ✓
- Enable type extensibility ✓
- Move toward Lisp-level meta-circularity ✓

**This should be in Phase 2.5 roadmap!**

