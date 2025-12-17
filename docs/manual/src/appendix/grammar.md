# Appendix A: Grammar Reference

This appendix provides a condensed reference to Kleis syntax.

## Expressions

```ebnf
expression ::= literal
             | identifier
             | expression operator expression
             | function_call
             | if_expression
             | let_expression
             | match_expression
             | quantified_expression
             | lambda_expression
             | ascription_expression
             | '(' expression ')'

literal ::= number | boolean | string

function_call ::= identifier '(' arguments? ')'

arguments ::= expression (',' expression)*
```

## Definitions

```ebnf
definition ::= 'define' identifier parameters? '=' expression
             | 'define' identifier parameters? ':' type '=' expression

parameters ::= '(' parameter_list ')'

parameter_list ::= parameter (',' parameter)*

parameter ::= identifier
            | identifier ':' type
```

## Let Bindings

```ebnf
let_expression ::= 'let' identifier '=' expression 'in' expression
                 | 'let' identifier ':' type '=' expression 'in' expression
```

## Type Ascription

```ebnf
ascription_expression ::= expression ':' type
```

## Conditionals

```ebnf
if_expression ::= 'if' expression 'then' expression 'else' expression
```

## Pattern Matching

```ebnf
match_expression ::= 'match' expression '{' match_arms '}'

match_arms ::= match_arm (',' match_arm)* ','?

match_arm ::= pattern guard? '=>' expression

pattern ::= '_'
          | literal
          | identifier
          | constructor '(' patterns? ')'
          | '(' patterns ')'

guard ::= 'if' expression
```

## Quantifiers

```ebnf
quantified_expression ::= quantifier identifier ':' type '.' expression
                        | quantifier identifier '.' expression

quantifier ::= '∀' | 'forall' | '∃' | 'exists'
```

## Lambda Expressions

```ebnf
lambda_expression ::= 'λ' parameters '.' expression
                    | 'lambda' parameters '.' expression

parameters ::= parameter+
parameter ::= identifier
            | '(' identifier ':' type ')'
```

Examples:
- `λ x . x + 1` - simple lambda
- `λ x y . x * y` - multiple parameters
- `λ (x : ℝ) . x^2` - with type annotation
- `lambda x . x` - using keyword

## Types

```ebnf
type ::= base_type
       | type '->' type                (* function type *)
       | type '×' type                 (* product type *)
       | type_constructor type_args?
       | '(' type ')'

base_type ::= 'ℕ' | 'Nat'
            | 'ℤ' | 'Int'
            | 'ℝ' | 'Real'
            | 'ℂ' | 'Complex'
            | 'Bool'
            | 'Unit'

type_constructor ::= identifier

type_args ::= '(' type (',' type)* ')'
```

## Structures

```ebnf
structure ::= 'structure' identifier type_params? extends? over? where? '{' structure_body '}'

type_params ::= '(' param_decl (',' param_decl)* ')'

param_decl ::= identifier ':' type

extends ::= 'extends' type (',' type)*

over ::= 'over' type

where ::= 'where' constraint (',' constraint)*

constraint ::= identifier ':' type

structure_body ::= (field | operation | axiom | nested_structure)*

field ::= 'field' identifier ':' type

nested_structure ::= 'structure' identifier ':' type ('{' structure_body '}')?

operation ::= 'operation' identifier ':' type
            | 'operation' identifier parameters ':' type

axiom ::= 'axiom' identifier ':' expression
```

## Implements

```ebnf
implements ::= 'implements' type ('as' identifier)? '{' impl_body '}'

impl_body ::= operation_impl*

operation_impl ::= 'operation' identifier parameters? '=' expression
                 | 'operation' identifier '=' 'builtin_' identifier
```

## Operators (by precedence, low to high)

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `↔` `iff` | Left |
| 2 | `→` `implies` | Right |
| 3 | `∨` `or` | Left |
| 4 | `∧` `and` | Left |
| 5 | `¬` `not` | Prefix |
| 6 | `=` `≠` `<` `>` `≤` `≥` | Non-assoc |
| 7 | `+` `-` | Left |
| 8 | `*` `/` | Left |
| 9 | `^` | Right |
| 10 | `-` (unary) | Prefix |
| 11 | Function application | Left |

## Comments

```ebnf
line_comment ::= '//' [^\n]*

block_comment ::= '/*' .* '*/'
```

**Note:** Kleis uses C-style comments (`//` and `/* */`), not Haskell-style (`--` and `{- -}`).

## Unicode Equivalents

| Unicode | ASCII |
|---------|-------|
| `∀` | `forall` |
| `∃` | `exists` |
| `→` | `->` |
| `×` | `*` |
| `∧` | `and`, `/\` |
| `∨` | `or`, `\/` |
| `¬` | `not`, `~` |
| `≤` | `<=` |
| `≥` | `>=` |
| `≠` | `!=`, `/=` |
| `ℕ` | `Nat` |
| `ℤ` | `Int` |
| `ℝ` | `Real` |
| `ℂ` | `Complex` |
| `λ` | `\` |
