# Appendix A: Grammar Reference

This appendix provides a reference to Kleis syntax based on the formal grammar specification (v0.7).

> **Complete Grammar:** See `docs/grammar/kleis_grammar_v07.ebnf` for the full EBNF specification.

## Program Structure

```ebnf
program ::= { declaration }

declaration ::= libraryAnnotation
              | versionAnnotation
              | structureDef
              | implementsDef
              | dataDef
              | functionDef
              | operationDecl
              | typeAlias
```

## Annotations

```ebnf
libraryAnnotation ::= "@library" "(" string ")"
versionAnnotation ::= "@version" "(" string ")"
```

Example:
```kleis
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
```kleis
data Bool = True | False
data Option(T) = None | Some(T)
data Type = Scalar | Vector(n: Nat) | Matrix(m: Nat, n: Nat)
```

## Pattern Matching

```ebnf
matchExpr ::= "match" expression "{" matchCases "}"

matchCase ::= pattern "=>" expression

pattern ::= "_"                              // Wildcard
          | identifier                       // Variable
          | identifier [ "(" patternArgs ")" ]  // Constructor
          | number | string | boolean        // Constant
```

Examples:
```kleis
match x { True => 1 | False => 0 }
match opt { None => 0 | Some(x) => x }
match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => -1 }
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
```kleis
structure VectorSpace(V) over Field(F) extends AbelianGroup(V) {
    operation (·) : F × V → V
    
    axiom scalar_distributive : ∀ a b : F . ∀ v : V .
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
```kleis
implements Ring(ℝ) {
    operation (+) = builtin_add
    operation (*) = builtin_mul
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
```kleis
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
                | "Bool" | "String"

parametricType ::= identifier "(" typeArgs ")"

functionType ::= type "→" type | type "->" type

typeAlias ::= "type" identifier "=" type
```

Examples:
```kleis
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
```kleis
λ x . x + 1              // Simple lambda
λ x y . x * y            // Multiple parameters
λ (x : ℝ) . x^2          // With type annotation
lambda x . x             // Using keyword
```

## Let Bindings

```ebnf
letBinding ::= "let" identifier [ typeAnnotation ] "=" expression "in" expression
```

Examples:
```kleis
let x = 5 in x + x
let x : ℝ = 3.14 in x * 2
let s = (a + b + c) / 2 in sqrt(s * (s-a) * (s-b) * (s-c))
```

## Conditionals

```ebnf
conditional ::= "if" expression "then" expression "else" expression
```

Example:
```kleis
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
```kleis
∀ x : ℝ . x + 0 = x
∃ x : ℤ . x * x = 4
∀ (a b : ℝ) where a ≠ 0 . a * (1/a) = 1
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
| 1 | `↔` `iff` | Left |
| 2 | `→` `implies` `⟹` `⇒` | Right |
| 3 | `∨` `or` | Left |
| 4 | `∧` `and` | Left |
| 5 | `¬` `not` (prefix) | Prefix |
| 6 | `=` `≠` `<` `>` `≤` `≥` `≈` `≡` `∈` `∉` | Non-assoc |
| 7 | `+` `-` | Left |
| 8 | `*` `×` `/` `·` | Left |
| 9 | `^` | Right |
| 10 | `-` (unary) | Prefix |
| 11 | Function application | Left |
| 12 | Postfix (`!`, `ᵀ`, `†`) | Postfix |

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
