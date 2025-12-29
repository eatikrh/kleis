# Appendix A: Grammar Reference

This appendix provides a reference to Kleis syntax based on the formal grammar specification (v0.95).

> **Complete Grammar:** See `docs/grammar/kleis_grammar_v095.md` for the full specification.

## Program Structure

```ebnf
program ::= { declaration }

declaration ::= importDecl              // v0.8: Module imports
              | libraryAnnotation
              | versionAnnotation
              | structureDef
              | implementsDef
              | dataDef
              | functionDef
              | operationDecl
              | typeAlias
              | exampleBlock            // v0.93: Executable documentation
```

## Import Statements (v0.8)

```ebnf
importDecl ::= "import" string
```

Example:
```text
import "stdlib/prelude.kleis"
import "stdlib/complex.kleis"
```

## Annotations

```ebnf
libraryAnnotation ::= "@library" "(" string ")"
versionAnnotation ::= "@version" "(" string ")"
```

Example:
```text
@library("stdlib/algebra")
@version("0.7")
```

## Data Type Definitions

```ebnf
dataDef ::= "data" identifier [ "(" typeParams ")" ] "="
            dataVariant { "|" dataVariant }

dataVariant ::= identifier [ "(" dataFields ")" ]

dataField ::= identifier ":" type    // Named field
            | type                   // Positional field
```

Examples:
```text
data Bool {
    True
    False
}
data Option(T) {
    None
    Some(value : T)
}
```

## Pattern Matching

```ebnf
matchExpr ::= "match" expression "{" matchCases "}"

matchCase ::= pattern [ "if" guardExpression ] "=>" expression   // v0.8: guards

pattern ::= basePattern [ "as" identifier ]  // v0.8: as-patterns

basePattern ::= "_"                              // Wildcard
              | identifier                       // Variable
              | identifier [ "(" patternArgs ")" ]  // Constructor
              | number | string | boolean        // Constant
              | tuplePattern                     // v0.8: Tuple sugar

tuplePattern ::= "()"                            // Unit
               | "(" pattern "," pattern { "," pattern } ")"  // Pair, Tuple3, etc.
```

Examples:
```text
match x { True => 1 | False => 0 }
match opt { None => 0 | Some(x) => x }
match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => -1 }

// v0.8: Pattern guards
match n { x if x < 0 => "negative" | x if x > 0 => "positive" | _ => "zero" }

// v0.8: As-patterns
match list { Cons(h, t) as whole => process(whole) | Nil => empty }
```

## Structure Definitions

```ebnf
structureDef ::= "structure" identifier "(" typeParams ")"
                 [ extendsClause ] [ overClause ]
                 "{" { structureMember } "}"

extendsClause ::= "extends" identifier [ "(" typeArgs ")" ]
overClause ::= "over" "Field" "(" type ")"

structureMember ::= operationDecl
                  | elementDecl
                  | axiomDecl
                  | nestedStructure
                  | functionDef
```

Example:
```text
structure VectorSpace(V) over Field(F) extends AbelianGroup(V) {
    operation (·) : F × V → V
    
    axiom scalar_distributive : ∀(a : F)(b : F)(v : V).
        (a + b) · v = a · v + b · v
}
```

## Implements

```ebnf
implementsDef ::= "implements" identifier "(" typeArgs ")"
                  [ overClause ]
                  [ "{" { implMember } "}" ]

implMember ::= elementImpl | operationImpl | verifyStmt

operationImpl ::= "operation" operatorSymbol "=" implementation
                | "operation" operatorSymbol "(" params ")" "=" expression
```

Example:
```text
implements Ring(ℝ) {
    operation add = builtin_add
    operation mul = builtin_mul
    element zero = 0
    element one = 1
}
```

## Function Definitions

```ebnf
functionDef ::= "define" identifier [ typeAnnotation ] "=" expression
              | "define" identifier "(" params ")" [ ":" type ] "=" expression

param ::= identifier [ ":" type ]
```

Examples:
```text
define pi = 3.14159
define square(x) = x * x
define add(x: ℝ, y: ℝ) : ℝ = x + y
```

## Type System

```ebnf
type ::= primitiveType
       | parametricType
       | functionType
       | typeVariable
       | "(" type ")"

primitiveType ::= "ℝ" | "ℂ" | "ℤ" | "ℕ" | "ℚ"
                | "Real" | "Complex" | "Integer" | "Nat" | "Rational"
                | "Bool" | "String" | "Unit"

parametricType ::= identifier "(" typeArgs ")"
                 | "BitVec" "(" number ")"      // Fixed-size bit vectors

functionType ::= type "→" type | type "->" type

typeAlias ::= "type" identifier "=" type
```

Examples:
```text
ℝ                    // Real numbers
Vector(3)            // Parameterized type
ℝ → ℝ               // Function type
(ℝ → ℝ) → ℝ         // Higher-order function
type RealFunc = ℝ → ℝ  // Type alias
```

## Expressions

```ebnf
expression ::= primary
             | matchExpr
             | prefixOp expression
             | expression postfixOp
             | expression infixOp expression
             | expression "(" [ arguments ] ")"
             | "[" [ expressions ] "]"           // List literal
             | lambda
             | letBinding
             | conditional

primary ::= identifier | number | string | symbolicConstant
          | "(" expression ")" | placeholder

symbolicConstant ::= "π" | "e" | "i" | "ℏ" | "c" | "φ" | "∞" | "∅"

placeholder ::= "□"
```

## Lambda Expressions

```ebnf
lambda ::= "λ" params "." expression
         | "lambda" params "." expression
```

Examples:
```text
λ x . x + 1              // Simple lambda
λ x y . x * y            // Multiple parameters
λ (x : ℝ) . x^2          // With type annotation
lambda x . x             // Using keyword
```

## Let Bindings

```ebnf
letBinding ::= "let" pattern [ typeAnnotation ] "=" expression "in" expression
// Note: typeAnnotation only valid when pattern is a simple Variable
```

Examples:
```text
let x = 5 in x + x
let x : ℝ = 3.14 in x * 2
let s = (a + b + c) / 2 in sqrt(s * (s-a) * (s-b) * (s-c))

// v0.8: Let destructuring
let Point(x, y) = origin in x^2 + y^2
let Some(Pair(a, b)) = opt in a + b
let Cons(h, _) = list in h
```

## Conditionals

```ebnf
conditional ::= "if" expression "then" expression "else" expression
```

Example:
```text
if x > 0 then x else -x
```

## Quantifiers

```ebnf
forAllProp ::= ("∀" | "forall") variables [ whereClause ] "." proposition
existsProp ::= ("∃" | "exists") variables [ whereClause ] "." proposition

varDecl ::= identifier [ ":" type ]
          | identifier "∈" type
          | "(" identifier { identifier } ":" type ")"

whereClause ::= "where" expression
```

Examples:
```text
∀(x : ℝ). x + 0 = x
∃(x : ℤ). x * x = 4
∀(a : ℝ)(b : ℝ) where a ≠ 0 . a * (1/a) = 1
```

## v0.9 Enhancements

### Nested Quantifiers in Expressions

Quantifiers can now appear as operands in logical expressions:

```text
// v0.9: Quantifier inside conjunction
axiom nested: (x > 0) ∧ (∀(y : ℝ). y > 0)

// Epsilon-delta limit definition
axiom epsilon_delta: ∀(ε : ℝ). ε > 0 → 
    (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
```

### Function Types in Type Annotations

Function types are now allowed in quantifier variable declarations:

```text
// Function from reals to reals
axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)

// Higher-order function
axiom compose: ∀(f : ℝ → ℝ, g : ℝ → ℝ). compose(f, g) = λ x . f(g(x))

// Topology: continuity via preimages
axiom continuity: ∀(f : X → Y, V : Set(Y)). 
    is_open(V) → is_open(preimage(f, V))
```

## v0.95 Big Operators

Big operators (Σ, Π, ∫, lim) can be used with function call syntax:

```ebnf
bigOpExpr ::= "Σ" "(" expr "," expr "," expr ")"
            | "Π" "(" expr "," expr "," expr ")"
            | "∫" "(" expr "," expr "," expr "," expr ")"
            | "lim" "(" expr "," expr "," expr ")"
            | ("Σ" | "Π" | "∫") primaryExpr      // prefix form
```

### Summation: Σ

```text
// Sum of f(i) from 1 to n
Σ(1, n, λ i . f(i))

// Parsed as: sum_bounds(λ i . f(i), 1, n)
```

### Product: Π

```text
// Product of g(i) from 1 to n
Π(1, n, λ i . g(i))

// Parsed as: prod_bounds(λ i . g(i), 1, n)
```

### Integral: ∫

```text
// Integral of x² from 0 to 1
∫(0, 1, λ x . x * x, x)

// Parsed as: int_bounds(λ x . x * x, 0, 1, x)
```

### Limit: lim

```text
// Limit of sin(x)/x as x approaches 0
lim(x, 0, sin(x) / x)

// Parsed as: lim(sin(x) / x, x, 0)
```

### Prefix Forms

Simple prefix forms are also supported:

```text
Σf        // Parsed as: Sum(f)
∫g        // Parsed as: Integrate(g)
```

## Calculus Notation (v0.7)

Kleis uses Mathematica-style notation for calculus operations:

```ebnf
// Derivatives (function calls)
D(f, x)              // Partial derivative ∂f/∂x
D(f, x, y)           // Mixed partial ∂²f/∂x∂y
D(f, {x, n})         // nth derivative ∂ⁿf/∂xⁿ
Dt(f, x)             // Total derivative df/dx

// Integrals
Integrate(f, x)           // Indefinite ∫f dx
Integrate(f, x, a, b)     // Definite ∫[a,b] f dx

// Sums and Products
Sum(expr, i, 1, n)        // Σᵢ₌₁ⁿ expr
Product(expr, i, 1, n)    // Πᵢ₌₁ⁿ expr

// Limits
Limit(f, x, a)            // lim_{x→a} f
```

Note: Legacy notation like `∂f/∂x` and `df/dx` is deprecated. Use `D(f, x)` and `Dt(f, x)` instead.

## Operators

### Prefix Operators

```ebnf
prefixOp ::= "-" | "¬" | "∇" | "√" | "∫" | "∬" | "∭" | "∮" | "∯"
```

### Postfix Operators

```ebnf
postfixOp ::= "!" | "†" | "*" | "ᵀ" | "^T" | "^†"
```

### Infix Operators (by precedence, low to high)

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `↔` `⇔` `⟺` (biconditional) | Left |
| 2 | `→` `⇒` `⟹` (implication) | Right |
| 3 | `∨` `or` | Left |
| 4 | `∧` `and` | Left |
| 5 | `¬` `not` (prefix) | Prefix |
| 6 | `=` `==` `≠` `<` `>` `≤` `≥` | Non-assoc |
| 7 | `+` `-` | Left |
| 8 | `*` `×` `/` `·` | Left |
| 9 | `^` | Right |
| 10 | `-` (unary) | Prefix |
| 11 | Postfix (`!`, `ᵀ`, `†`) | Postfix |
| 12 | Function application | Left |

> **Note:** Set operators (`∈`, `∉`, `⊆`, `≈`, `≡`) are not implemented. Use function-call syntax instead.

## Comments

```ebnf
lineComment ::= "//" { any character except newline } newline
blockComment ::= "/*" { any character } "*/"
```

**Note:** Kleis uses C-style comments (`//` and `/* */`), not Haskell-style (`--` and `{- -}`).

## Unicode Equivalents

| Unicode | ASCII | Description |
|---------|-------|-------------|
| `∀` | `forall` | Universal quantifier |
| `∃` | `exists` | Existential quantifier |
| `→` | `->` | Function type / implies |
| `×` | `*` | Product type / multiplication |
| `∧` | `and`, `/\` | Logical and |
| `∨` | `or`, `\/` | Logical or |
| `¬` | `not`, `~` | Logical not |
| `≤` | `<=` | Less or equal |
| `≥` | `>=` | Greater or equal |
| `≠` | `!=`, `/=` | Not equal |
| `ℕ` | `Nat` | Natural numbers |
| `ℤ` | `Int` | Integers |
| `ℚ` | `Rational` | Rational numbers |
| `ℝ` | `Real` | Real numbers |
| `ℂ` | `Complex` | Complex numbers |
| `λ` | `lambda` | Lambda |
| `π` | `pi` | Pi constant |
| `∞` | `infinity` | Infinity |

## Lexical Elements

```ebnf
identifier ::= letter { letter | digit | "_" }

number ::= integer | decimal | scientific
integer ::= digit { digit }
decimal ::= digit { digit } "." { digit }
scientific ::= decimal ("e" | "E") ["+"|"-"] digit { digit }

string ::= '"' { character } '"'

letter ::= "a".."z" | "A".."Z" | greekLetter
digit ::= "0".."9"

greekLower ::= "α" | "β" | "γ" | "δ" | "ε" | "ζ" | "η" | "θ"
             | "ι" | "κ" | "λ" | "μ" | "ν" | "ξ" | "ο" | "π"
             | "ρ" | "σ" | "τ" | "υ" | "φ" | "χ" | "ψ" | "ω"
```
