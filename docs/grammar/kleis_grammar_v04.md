# Kleis Grammar v0.4 – Algebraic Data Types

**Date:** December 8, 2024  
**Status:** Working Draft  
**Based on:** ADR-021 (Algebraic Data Types) and v0.3 grammar  
**Major Addition:** Data type definitions for self-hosting

---

## What's New in v0.4

### Algebraic Data Types (ADR-021)

**The breakthrough:** Types can now be defined in Kleis files, not hardcoded in Rust!

```kleis
// Define new types with the 'data' keyword
data Bool = True | False

data Option(T) = None | Some(T)

data Type = 
  | Scalar 
  | Vector(n: Nat) 
  | Matrix(m: Nat, n: Nat)
  | Complex

data Currency = USD | EUR | GBP | JPY
```

This enables:
- ✅ **Self-hosting**: Kleis type system defined in Kleis
- ✅ **User extensibility**: Add types without recompiling
- ✅ **Meta-circularity**: Type checking written in Kleis (future)
- ✅ **Domain-specific types**: Custom types for any domain

### Why This Matters

**Before v0.4:** Types hardcoded in Rust
```rust
// In Rust code - users can't extend this!
pub enum Type {
    Scalar,
    Matrix(usize, usize),
    Vector(usize),
}
```

**After v0.4:** Types defined in Kleis
```kleis
// In stdlib/types.kleis - users CAN extend this!
data Type = Scalar | Matrix(m: Nat, n: Nat) | Vector(n: Nat)

// Users can add their own types!
data MyTypes = Quantity(value: ℝ, unit: String) | Temperature(kelvin: ℝ)
```

---

## Data Type Syntax

### Basic Data Declaration

```ebnf
dataDef ::= "data" identifier [ "(" typeParams ")" ] "=" 
            dataVariant { "|" dataVariant }

dataVariant ::= identifier [ "(" dataFields ")" ]

dataFields ::= dataField { "," dataField }

dataField ::= identifier ":" type      (* Named field *)
            | type                      (* Positional field *)
```

### Examples

**Simple enumeration (no fields):**
```kleis
data Bool = True | False
data Color = Red | Green | Blue | Yellow
```

**Parametric types (type variables):**
```kleis
data Option(T) = 
  | None 
  | Some(T)

data Result(T, E) = 
  | Ok(value: T) 
  | Err(error: E)

data List(T) = 
  | Nil 
  | Cons(head: T, tail: List(T))
```

**Named fields (constructor parameters):**
```kleis
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Tensor(dims: List(Nat))

data Point = Point(x: ℝ, y: ℝ, z: ℝ)
```

**Mixed positional and named fields:**
```kleis
data Expression =
  | Const(ℝ)                          (* Positional *)
  | Variable(name: String)            (* Named *)
  | Binary(op: String, left: Expression, right: Expression)
```

---

## Integration with Existing Features

### Data Types + Structures

Data types define **what types exist**.  
Structures define **what operations those types support**.

```kleis
// Define the type (v0.4)
data Type = Scalar | Matrix(m: Nat, n: Nat)

// Define operations on the type (v0.3)
structure Arithmetic(T) {
  operation plus : T → T → T
  operation times : T → T → T
}

// Implement for specific types
implements Arithmetic(ℝ) {
  operation plus = builtin_add
  operation times = builtin_mul
}

implements Arithmetic(Matrix(m, n, ℝ)) {
  operation plus = builtin_matrix_add
  operation times = builtin_hadamard
}
```

### Type Checking Data Constructors

```kleis
// Type checker validates:
Matrix(2, 3, a, b, c, d, e, f)  // ✓ Valid: 2×3 matrix with 6 elements
Matrix(2, 3, a, b)               // ✗ Error: Need 6 elements for 2×3

Some(42)     // ✓ Valid: Some(T) where T = ℝ
None         // ✓ Valid: None has no fields
Some()       // ✗ Error: Some requires one argument
```

---

## Core Syntax Elements (from v0.3)

### 1. Type Annotations

```ebnf
TypeAnnotation ::= <Name> ":" <Type>
Type ::= <ConcreteType> | <TypeVariable> | <FunctionType> | <PolymorphicType>

ConcreteType ::= <TypeName> [ "(" <TypeArgs> ")" ]
TypeVariable ::= <LowerGreek> | <LowerLatin>
FunctionType ::= <Type> "→" <Type>
PolymorphicType ::= "∀" <TypeVars> "." [ <Constraints> "⇒" ] <Type>

TypeName ::= "ℝ" | "ℂ" | "ℤ" | "ℕ" | "Bool" | "String" 
           | "Vector" | "Matrix" | "Tensor" | "List"
           | <UserDefinedType>   (* Can now be from 'data' declarations! *)

TypeArgs ::= <Type> [ "," <Type> ]*
TypeVars ::= <TypeVariable> [ <TypeVariable> ]*
Constraints ::= <Constraint> [ "," <Constraint> ]*
Constraint ::= <StructureName> "(" <Type> ")"
```

**Examples (now with data types):**
```kleis
x : ℝ                            // Primitive type
v : Vector(3)                    // Parametric (could be from data)
maybe : Option(ℝ)                // User-defined data type
result : Result(ℝ, String)       // Parametric data type
f : ℝ → ℝ                        // Function type
id : ∀T. T → T                   // Polymorphic
```

### 2. Structure Definitions

(Unchanged from v0.3 - see kleis_grammar_v03.md for full details)

```kleis
structure Monoid(M) {
  element identity : M
  operation combine : M → M → M
  axiom left_identity : ∀x. combine(identity, x) = x
  axiom right_identity : ∀x. combine(x, identity) = x
  axiom associativity : ∀x y z. 
    combine(combine(x, y), z) = combine(x, combine(y, z))
}
```

### 3. Top-Level Declarations

```ebnf
declaration ::= 
    libraryAnnotation
  | versionAnnotation
  | structureDef
  | implementsDef
  | dataDef              (* NEW in v0.4 *)
  | functionDef
  | operationDecl
  | objectDecl
  | typeAlias
```

---

## Implementation Status

### What's Implemented (Steps 1-4 of ADR-021)

✅ **AST structures** for data definitions  
✅ **Parser support** for `data` keyword  
✅ **Registry** for type/variant lookups  
✅ **Type enum refactored** to support dynamic types  

### What's Next (Steps 5-11)

- [ ] Generic constructor inference (use registry for type inference)
- [ ] Wire DataTypeRegistry into TypeInference
- [ ] TypeChecker loads data types from files
- [ ] Create stdlib/types.kleis with base types
- [ ] Full integration and testing

### Example stdlib/types.kleis (Coming Soon!)

```kleis
// This will be the foundation of Kleis type system
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Complex
  | Set(T: Type)
  | List(T: Type)
  | Tensor(dims: List(Nat))

data Bool = True | False

data Option(T) =
  | None
  | Some(value: T)

data Result(T, E) =
  | Ok(value: T)
  | Err(error: E)

data List(T) =
  | Nil
  | Cons(head: T, tail: List(T))
```

---

## Comparison with Other Languages

### Haskell
```haskell
data Maybe a = Nothing | Just a
data Either a b = Left a | Right b
```

### OCaml
```ocaml
type 'a option = None | Some of 'a
type ('a, 'b) result = Ok of 'a | Error of 'b
```

### Rust
```rust
enum Option<T> {
    None,
    Some(T),
}
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Kleis (v0.4)
```kleis
data Option(T) = None | Some(T)
data Result(T, E) = Ok(T) | Err(E)
```

**Key difference:** Kleis data types integrate seamlessly with mathematical notation and structures!

---

## Design Principles

### 1. Mathematical First
Data types use mathematical syntax and integrate with structures:
```kleis
data Vector(n: Nat) = Vector(elements: List(ℝ))

structure VectorSpace(V) {
  operation plus : V → V → V
  operation scalar_multiply : ℝ → V → V
}

implements VectorSpace(Vector(n)) {
  operation plus = builtin_vector_add
  operation scalar_multiply = builtin_scalar_mul
}
```

### 2. Self-Hosting
The type system itself is defined in Kleis:
```kleis
// The Type type is itself a data type!
data Type = Scalar | Vector(n: Nat) | Matrix(m: Nat, n: Nat) | ...
```

### 3. User Extensibility
Users can define domain-specific types:
```kleis
// Physics
data Particle = Electron | Proton | Neutron | Photon
data Spin = SpinUp | SpinDown

// Finance  
data Currency = USD | EUR | GBP | JPY
data Trade = Buy(amount: ℝ, price: ℝ) | Sell(amount: ℝ, price: ℝ)

// Logic
data Formula = 
  | Var(name: String)
  | Not(Formula)
  | And(Formula, Formula)
  | Or(Formula, Formula)
  | Implies(Formula, Formula)
```

---

## Grammar Change Log

### Version 0.4 (2024-12-08) – Algebraic Data Types

**Major additions:**
- `dataDef` declaration for algebraic data types
- `dataVariant` for constructor definitions
- `dataField` for named and positional fields
- Foundation for self-hosting type system

**Motivation:** Enable Kleis types to be defined in Kleis (ADR-021)

**Impact:** Transformative - makes Kleis truly self-hosting!

### Version 0.3 (2024-12-05) – Type System

- Hindley-Milner type inference
- Polymorphic types with constraints
- Structure definitions with axioms
- Implements blocks

### Version 0.2 – Mathematical Expressions

- Basic arithmetic and calculus
- Function definitions
- Lambda expressions

---

## See Also

- **ADR-021:** Algebraic Data Types (full design rationale)
- **kleis_grammar_v04.ebnf:** Formal EBNF specification
- **Kleis_v04.g4:** ANTLR4 grammar (coming soon)
- **stdlib/types.kleis:** Base type definitions (Step 8)

---

**Status:** Grammar specification complete, implementation in progress (Steps 5-11)  
**Next:** Complete ADR-021 implementation and create stdlib/types.kleis

