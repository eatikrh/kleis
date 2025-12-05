# Kleis Grammar v0.3 – Type System Integration

**Date:** December 2024  
**Status:** Working Draft  
**Based on:** stdlib/prelude.kleis and ADR-014

---

## Introduction

Kleis Grammar v0.3 extends v0.2 with a formal type system based on Hindley-Milner type inference and Haskell-style algebraic structures. This enables:
- Automatic type inference for symbolic expressions
- Algebraic structure hierarchy (Monoid → Group → Ring → Field)
- User-defined types for any domain (math, business, physics)
- Operation-based type checking with manifests

---

## Core Syntax Elements

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
           | <UserDefinedType>

TypeArgs ::= <Type> [ "," <Type> ]*
TypeVars ::= <TypeVariable> [ <TypeVariable> ]*
Constraints ::= <Constraint> [ "," <Constraint> ]*
Constraint ::= <StructureName> "(" <Type> ")"
```

**Examples:**
```kleis
x : ℝ
v : Vector(3)
A : Matrix(m, n)
f : ℝ → ℝ
id : ∀T. T → T
sum : ∀T. Monoid(T) ⇒ List(T) → T
```

### 2. Structure Definitions

```ebnf
StructureDef ::= "structure" <Name> "(" <TypeParam> ")" 
                 [ "extends" <StructureName> ] 
                 "{" <StructureBody> "}"

StructureBody ::= <StructureMember>*

StructureMember ::= <OperationDecl>
                  | <ElementDecl>
                  | <AxiomDecl>
                  | <NestedStructure>
                  | <SupportsBlock>

OperationDecl ::= "operation" <OpSymbol> ":" <TypeSignature>
ElementDecl ::= "element" <Name> ":" <Type>
AxiomDecl ::= "axiom" <Name> ":" <Proposition>
NestedStructure ::= "structure" <Name> ":" <StructureName> "(" <Type> ")"
SupportsBlock ::= "supports" "{" <OperationDecl>* "}"
```

**Examples:**
```kleis
structure Monoid(M) {
  operation (•) : M × M → M
  element e : M
  
  axiom identity: ∀x ∈ M. e • x = x ∧ x • e = x
  axiom associativity: ∀x y z ∈ M. (x•y)•z = x•(y•z)
}

structure Ring(R) {
  structure additive : AbelianGroup(R)
  structure multiplicative : Monoid(R)
  
  axiom distributivity: ∀x y z. x×(y+z) = x×y + x×z
}

structure Matrix(m : ℕ, n : ℕ) {
  supports {
    (+) : Matrix(m,n) × Matrix(m,n) → Matrix(m,n)
    (×) : Matrix(m,n) × Matrix(n,p) → Matrix(m,p)
    transpose : Matrix(m,n) → Matrix(n,m)
    det : Matrix(n,n) → ℝ (if m = n)
  }
}
```

### 3. Implementation Declarations

```ebnf
ImplementsDef ::= "implements" <StructureName> "(" <TypeArgs> ")"
                  [ "over" <FieldSpec> ]
                  [ "{" <ImplementationBody> "}" ]

ImplementationBody ::= <OperationImpl> | <ElementImpl> | <VerifyStmt>*

OperationImpl ::= "operation" <OpSymbol> "=" <Expression>
ElementImpl ::= "element" <Name> "=" <Expression>
VerifyStmt ::= "verify" <AxiomName>

FieldSpec ::= "Field" "(" <Type> ")"
```

**Examples:**
```kleis
implements Field(ℝ) {
  element zero = 0
  element one = 1
  operation (+) = builtin_add
  operation (×) = builtin_mul
}

implements VectorSpace(Vector(n)) over Field(ℝ) {
  element zero_v = [0, 0, ..., 0]
  operation (+) = vector_add
  operation (·) = scalar_vector_mul
}

implements Monoid(ℤ, +, 0) {
  verify identity
  verify associativity
}
```

### 4. Function Definitions

```ebnf
FunctionDef ::= "define" <Name> [ <TypeAnnotation> ] "=" <Expression>
              | "define" <Name> "(" <Parameters> ")" [ ":" <Type> ] "=" <Expression>

Parameters ::= <Parameter> [ "," <Parameter> ]*
Parameter ::= <Name> [ ":" <Type> ]
```

**Examples:**
```kleis
define π : ℝ = 3.14159265358979323846

define f(x) = x²
// Inferred: f : ℝ → ℝ

define f(x : ℝ) : ℝ = x²
// Explicit annotation

define dot(u, v) = Σᵢ uᵢ × vᵢ
// Inferred: dot : ∀n. Vector(n) × Vector(n) → ℝ

define dot : ∀(n : ℕ). Vector(n) × Vector(n) → ℝ
define dot(u, v) = Σᵢ uᵢ × vᵢ
// Explicit polymorphic type
```

### 5. Operation Declarations (Top-Level)

```ebnf
OperationDecl ::= "operation" <OpSymbol> ":" <TypeSignature>
                  [ "define" <Name> "(" <Parameters> ")" "=" <Expression> ]
```

**Examples:**
```kleis
// Declaration only
operation dot : ∀(n : ℕ). Vector(n) × Vector(n) → ℝ

// Declaration with definition
operation cross : Vector(3) × Vector(3) → Vector(3)
define cross([u₁,u₂,u₃], [v₁,v₂,v₃]) = [u₂v₃ - u₃v₂, u₃v₁ - u₁v₃, u₁v₂ - u₂v₁]

// Polymorphic operation
operation (d/dx) : (ℝ → ℝ) → (ℝ → ℝ)
```

### 6. Axiom Syntax

```ebnf
AxiomDecl ::= "axiom" <Name> ":" <Proposition>

Proposition ::= <ForAll> <Expression>
ForAll ::= "∀" <Variables> [ "where" <Constraint> ] "." <Proposition>
Variables ::= <VarDecl> [ <VarDecl> ]*
VarDecl ::= <Name> [ ":" <Type> ] | "(" <Name>+ ":" <Type> ")"
Constraint ::= <Expression>
```

**Examples:**
```kleis
axiom identity:
  ∀x. e • x = x ∧ x • e = x

axiom associativity:
  ∀(x y z : M). (x • y) • z = x • (y • z)

axiom inverse:
  ∀x where x ≠ zero. ∃y. x × y = one

axiom commutativity:
  ∀x y. x + y = y + x
```

### 7. Library Annotations

```ebnf
LibraryAnnotation ::= "@library" "(" <StringLiteral> ")"
VersionAnnotation ::= "@version" "(" <StringLiteral> ")"
```

**Examples:**
```kleis
@library("std.prelude")
@version("0.1.0")

// ... library contents
```

### 8. Polymorphic Types

```ebnf
PolymorphicType ::= "∀" <TypeVars> [ "." <Constraints> "⇒" ] <Type>

TypeVars ::= <TypeVar> | "(" <TypeVarDecl> [ "," <TypeVarDecl> ]* ")"
TypeVarDecl ::= <TypeVar> [ ":" <Kind> ]
Constraints ::= <Constraint> [ "," <Constraint> ]*
Constraint ::= <StructureName> "(" <Type> ")"
```

**Examples:**
```kleis
∀T. T → T
∀(T : Type). List(T) → ℕ
∀(n : ℕ). Vector(n) → ℝ
∀T. Monoid(T) ⇒ List(T) → T
∀(m n p : ℕ). Matrix(m,n) × Matrix(n,p) → Matrix(m,p)
```

---

## Complete Grammar (EBNF)

### Top-Level Declarations

```ebnf
Program ::= <Declaration>*

Declaration ::= <LibraryAnnotation>
              | <StructureDef>
              | <ImplementsDef>
              | <FunctionDef>
              | <OperationDecl>
              | <ObjectDecl>
              | <TypeAlias>
```

### Structures

```ebnf
StructureDef ::= "structure" <Name> "(" <TypeParams> ")"
                 [ "extends" <StructureName> [ "(" <TypeArgs> ")" ] ]
                 [ "over" <FieldSpec> ]
                 "{" <StructureMember>* "}"

TypeParams ::= <TypeParam> [ "," <TypeParam> ]*
TypeParam ::= <Name> [ ":" <Kind> ]
Kind ::= "Type" | "ℕ" | "Field" | "*"

StructureMember ::= <OperationDecl>
                  | <ElementDecl>
                  | <AxiomDecl>
                  | <NestedStructure>
                  | <SupportsBlock>
                  | <NotationDecl>

SupportsBlock ::= "supports" "{" <OperationDecl>* "}"
NotationDecl ::= "notation" <Name> "(" <Params> ")" "=" <Expression>
```

### Implementations

```ebnf
ImplementsDef ::= "implements" <StructureName> "(" <TypeArgs> ")"
                  [ "over" <FieldSpec> ]
                  [ "{" <ImplMember>* "}" ]

ImplMember ::= "element" <Name> "=" <Expression>
             | "operation" <OpSymbol> "=" <Implementation>
             | "verify" <AxiomName>

Implementation ::= <Expression> | <FunctionName>
```

### Types

```ebnf
Type ::= <PrimitiveType>
       | <ParametricType>
       | <FunctionType>
       | <TypeVariable>
       | <PolymorphicType>

PrimitiveType ::= "ℝ" | "ℂ" | "ℤ" | "ℕ" | "Bool" | "String"

ParametricType ::= <TypeConstructor> "(" <TypeArgs> ")"
TypeConstructor ::= "Vector" | "Matrix" | "Tensor" | "List" | "Set"
                  | <UserDefinedType>

FunctionType ::= <Type> "→" <Type>

TypeVariable ::= <LowerGreek> | <LowerLatin> | <UpperLatin>
LowerGreek ::= "α" | "β" | "γ" | "δ" | "ε" | ...
LowerLatin ::= "a" | "b" | "c" | ...
UpperLatin ::= "T" | "U" | "V" | "S" | "M" | ...

PolymorphicType ::= "∀" <TypeVarList> "." [ <Constraints> "⇒" ] <Type>
TypeVarList ::= <TypeVarDecl> [ <TypeVarDecl> ]*
TypeVarDecl ::= <TypeVar> [ ":" <Kind> ]
Constraints ::= <Constraint> [ "," <Constraint> ]*
Constraint ::= <StructureName> "(" <Type> ")"
             | <Type> "=" <Type>
             | <Expression>
```

### Expressions

```ebnf
Expression ::= <Constant>
             | <Variable>
             | <Operation>
             | <FunctionCall>
             | <Lambda>
             | <LetBinding>
             | <Conditional>

Constant ::= <Number> | <SymbolicConstant>
Number ::= <Integer> | <Decimal> | <ScientificNotation>
SymbolicConstant ::= "π" | "e" | "i" | "ℏ" | "c"

Variable ::= <Name>

Operation ::= <InfixOp> | <PrefixOp> | <PostfixOp>
InfixOp ::= <Expression> <Operator> <Expression>
Operator ::= "+" | "-" | "×" | "/" | "·" | "∧" | "∨" | "∈" | ...

FunctionCall ::= <Name> "(" <Arguments> ")"
Arguments ::= <Expression> [ "," <Expression> ]*

Lambda ::= "λ" <Parameters> "." <Expression>
         | "λ" "(" <Parameters> ")" "." <Expression>

LetBinding ::= "let" <Name> [ ":" <Type> ] "=" <Expression> "in" <Expression>
```

---

## Concrete Examples from stdlib/prelude.kleis

### Structure with Operations

```kleis
structure Monoid(M) extends Semigroup(M) {
  element e : M
  
  axiom left_identity:
    ∀(x : M). e • x = x
    
  axiom right_identity:
    ∀(x : M). x • e = x
}
```

**Grammar match:**
- `structure Monoid(M)` ← StructureDef
- `extends Semigroup(M)` ← extends clause
- `element e : M` ← ElementDecl
- `axiom left_identity: ...` ← AxiomDecl
- `∀(x : M). e • x = x` ← ForAll proposition

### Structure with Nested Structures

```kleis
structure Ring(R) {
  structure additive : AbelianGroup(R) {
    operation (+) : R × R → R
    element zero : R
  }
  
  structure multiplicative : Monoid(R) {
    operation (×) : R × R → R
    element one : R
  }
  
  axiom distributivity:
    ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
}
```

**Grammar match:**
- `structure additive : AbelianGroup(R)` ← NestedStructure
- Nested operations and elements ← StructureMember

### Implementation with Verification

```kleis
implements Numeric(ℝ) {
  element zero = 0
  element one = 1
  operation (+) = builtin_add
  operation (×) = builtin_mul
}

implements Monoid(ℤ, +, 0) {
  verify identity
  verify associativity
}
```

**Grammar match:**
- `implements Numeric(ℝ)` ← ImplementsDef
- `element zero = 0` ← ElementImpl
- `operation (+) = builtin_add` ← OperationImpl
- `verify identity` ← VerifyStmt

### Operation with Polymorphic Type

```kleis
operation dot : ∀(n : ℕ). Vector(n) × Vector(n) → ℝ
define dot(u, v) = Σᵢ uᵢ × vᵢ
```

**Grammar match:**
- `operation dot : ∀(n : ℕ). ...` ← OperationDecl with PolymorphicType
- `∀(n : ℕ)` ← type-level dimension parameter
- `define dot(u, v) = ...` ← function definition

### VectorSpace over Field

```kleis
structure VectorSpace(V) over Field(F) {
  operation (+) : V × V → V
  operation (·) : F × V → V
  element zero_v : V
  
  axiom scalar_distributivity:
    ∀(c : F, u v : V). c · (u + v) = c · u + c · v
}

implements VectorSpace(Vector(n)) over Field(ℝ) {
  element zero_v = [0, 0, ..., 0]
  operation (+) = vector_add
  operation (·) = scalar_vector_mul
}
```

**Grammar match:**
- `over Field(F)` ← field specification for vector spaces
- Multiple variable declarations: `u v : V` ← shorthand syntax

---

## New Keywords Added in v0.3

### Core Type System

| Keyword | Purpose | Example |
|---------|---------|---------|
| `structure` | Define algebraic structure | `structure Monoid(M)` |
| `extends` | Structure inheritance | `extends Group(G)` |
| `implements` | Create instance | `implements Field(ℝ)` |
| `over` | Specify underlying field | `over Field(ℝ)` |
| `operation` | Declare operation | `operation (+) : T × T → T` |
| `element` | Declare constant | `element zero : T` |
| `axiom` | Declare law | `axiom identity: ...` |
| `verify` | Verify axiom holds | `verify associativity` |
| `supports` | Operation manifest | `supports { ... }` |

### Type System

| Keyword | Purpose | Example |
|---------|---------|---------|
| `∀` | Universal quantifier | `∀T. T → T` |
| `∃` | Existential quantifier | `∃y. x × y = one` |
| `⇒` | Constraint implication | `Monoid(T) ⇒ ...` |
| `:` | Type annotation | `x : ℝ` |
| `→` | Function type | `ℝ → ℝ` |
| `where` | Add constraint | `where x ≠ 0` |

### Annotations

| Annotation | Purpose | Example |
|------------|---------|---------|
| `@library` | Mark as library | `@library("std.prelude")` |
| `@version` | Version number | `@version("0.1.0")` |

---

## Kept from v0.2

### Equality Types (Unchanged)

| Syntax | Meaning |
|--------|---------|
| `define A = B` | Definition (by fiat) |
| `assert A == B` | Algebraic equality |
| `equiv A ~ B` | Structural equivalence |
| `approx A ≈ B` | Approximate equality |

### Object Declarations (Unchanged)

```kleis
object ψ : Hilbert(ℂ)
object g : Tensor(M, [↓,↓])
object E : VectorField(ℝ³)
```

---

## Deprecated from v0.2

### Replaced Syntax

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `narrow M -> M` | `operation : M → M` | Clearer |
| `object Monad M` | `structure Monoid(M)` | Types vs values |
| `->` | `→` | Mathematical notation |
| `const Pi` | `define π : ℝ` | Type annotations |

---

## Complete Example: Field Hierarchy

```kleis
@library("std.algebra")

// ============================================
// ALGEBRAIC HIERARCHY
// ============================================

structure Semigroup(S) {
  operation (•) : S × S → S
  axiom associativity: ∀(x y z : S). (x•y)•z = x•(y•z)
}

structure Monoid(M) extends Semigroup(M) {
  element e : M
  axiom identity: ∀(x : M). e•x = x ∧ x•e = x
}

structure Group(G) extends Monoid(G) {
  operation inv : G → G
  axiom inverse: ∀(x : G). inv(x)•x = e ∧ x•inv(x) = e
}

structure AbelianGroup(A) extends Group(A) {
  axiom commutativity: ∀(x y : A). x•y = y•x
}

structure Ring(R) {
  structure additive : AbelianGroup(R)
  structure multiplicative : Monoid(R)
  axiom distributivity: ∀(x y z : R). x×(y+z) = x×y + x×z
}

structure Field(F) extends Ring(F) {
  operation (/) : F × F → F
  axiom inverse: ∀x where x ≠ zero. ∃y. x × y = one
}

// ============================================
// IMPLEMENTATIONS
// ============================================

implements Field(ℝ) {
  element zero = 0
  element one = 1
  operation (+) = builtin_add
  operation (×) = builtin_mul
  operation negate(x) = -x
  operation inverse(x) = 1/x
  
  verify identity
  verify associativity
  verify commutativity
  verify distributivity
}

// ============================================
// OPERATIONS
// ============================================

operation sin : ℝ → ℝ
operation cos : ℝ → ℝ
operation exp : ℝ → ℝ
operation ln : ℝ → ℝ

operation dot : ∀(n : ℕ). Vector(n) × Vector(n) → ℝ
define dot(u, v) = Σᵢ uᵢ × vᵢ

operation norm : ∀(n : ℕ). Vector(n) → ℝ
define norm(v) = √(dot(v, v))

// ============================================
// CONSTANTS
// ============================================

define π : ℝ = 3.14159265358979323846
define e : ℝ = 2.71828182845904523536
define i : ℂ = √(-1)
```

---

## Grammar Evolution

| Version | Date | Changes |
|---------|------|---------|
| v0.1 | 2024-11 | Initial design |
| v0.2 | 2024-12-01 | Objects, morphisms, equality types |
| **v0.3** | **2024-12-05** | **Type system, structures, inference** |

---

## Implementation Status

### Supported in Current Parser ✅
- Basic expressions
- Operations
- Objects and constants

### Need to Implement ⬜
- `structure` keyword parsing
- `implements` keyword parsing
- `axiom` declarations
- `∀` type syntax
- Type annotations with `:`
- `@library` annotations

### Implementation Priority

**Phase 1:** Structure definitions
- Parse `structure Name(Params) { ... }`
- Parse operation declarations
- Parse axioms (as strings for now)

**Phase 2:** Implementations
- Parse `implements Structure(Type)`
- Parse element/operation bindings

**Phase 3:** Type annotations
- Parse `: Type` syntax
- Parse `∀` polymorphic types

**Phase 4:** Library system
- Parse `@library` annotations
- Load and parse stdlib/prelude.kleis

---

## Next Steps

1. Update parser to support v0.3 syntax
2. Implement structure definition parsing
3. Implement type annotation parsing
4. Load stdlib/prelude.kleis at startup
5. Integrate with type inference engine

---

**This grammar now matches the code we've been writing!** 🎯

