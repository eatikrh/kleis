# Pattern Matching Grammar Extension for Kleis v0.5

> **HISTORICAL DOCUMENT (December 2025).** Pattern matching was fully implemented
> and is part of the mature grammar (now v0.99). All items marked "TODO" or
> "Planned" below are **done**. This document is preserved as a design record.

**Date:** December 8, 2025  
**Status:** ✅ IMPLEMENTED (grammar v0.99)  
**Base:** Kleis Grammar v0.4 (with algebraic data types)

---

## Overview

This document specifies the grammar extensions needed for pattern matching support in Kleis. Pattern matching is the second half of ADR-021 (Algebraic Data Types), enabling users to **use** the data types they define.

**Grammar Version:** v0.4 → **v0.5**  
**New Feature:** Pattern matching expressions

---

## Grammar Extensions (EBNF)

### Expression Extension

**Add to expression alternatives:**

```ebnf
expression
    ::= term
      | operation
      | functionCall
      | matchExpr        (* NEW: Pattern matching *)
      | literal
      | identifier
      | "(" expression ")"
      ;
```

### Pattern Matching Syntax

```ebnf
(* ============================================ *)
(* PATTERN MATCHING (NEW in v0.5)              *)
(* ============================================ *)

matchExpr
    ::= "match" expression "{" matchCases "}"
    ;

matchCases
    ::= matchCase { caseSeperator matchCase }
    ;

caseSeparator
    ::= "|"               (* Pipe separator *)
      | newline           (* Newline separator *)
    ;

matchCase
    ::= pattern "=>" expression
    ;

pattern
    ::= wildcardPattern
      | variablePattern
      | constructorPattern
      | constantPattern
      ;

wildcardPattern
    ::= "_"
    ;

variablePattern
    ::= identifier       (* Must start with lowercase *)
    ;

constructorPattern
    ::= identifier [ "(" patternArgs ")" ]
    ;                    (* Constructor must start with uppercase *)

patternArgs
    ::= pattern { "," pattern }
    ;

constantPattern
    ::= number
      | string
      | boolean
    ;

boolean ::= "True" | "False" ;
```

---

## Grammar Rules

### Constructor vs Variable Disambiguation

**Rule:** First character case determines interpretation

```ebnf
constructorPattern ::= UPPERCASE_ID [ "(" patternArgs ")" ]
variablePattern    ::= LOWERCASE_ID

UPPERCASE_ID ::= [A-Z] [a-zA-Z0-9_]*
LOWERCASE_ID ::= [a-z_] [a-zA-Z0-9_]*
```

**Examples:**
- `Some(x)` - Constructor: `Some`, Variable: `x`
- `None` - Constructor (0-arity)
- `value` - Variable
- `_` - Wildcard (special token)

---

## Complete EBNF for Kleis v0.5

### Top-Level Declarations

```ebnf
program ::= { declaration } ;

declaration
    ::= libraryAnnotation
      | versionAnnotation
      | structureDef
      | implementsDef
      | dataDef              (* v0.4: Algebraic data types *)
      | functionDef
      | operationDecl
      | objectDecl
      | typeAlias
      ;
```

### Data Type Definitions (v0.4)

```ebnf
dataDef
    ::= "data" identifier [ "(" typeParams ")" ] "="
        dataVariant { "|" dataVariant }
    ;

dataVariant
    ::= identifier [ "(" dataFields ")" ]
    ;

dataFields ::= dataField { "," dataField } ;

dataField
    ::= identifier ":" type      (* Named field *)
      | type                      (* Positional field *)
    ;
```

### Pattern Matching (v0.5 - NEW)

```ebnf
matchExpr
    ::= "match" expression "{" matchCases "}"
    ;

matchCases
    ::= matchCase { ( "|" | newline ) matchCase }
    ;

matchCase
    ::= pattern "=>" expression
    ;

pattern
    ::= "_"                                      (* Wildcard *)
      | LOWERCASE_ID                             (* Variable *)
      | UPPERCASE_ID [ "(" patternArgs ")" ]     (* Constructor *)
      | number                                    (* Constant *)
      | string                                    (* String constant *)
    ;

patternArgs
    ::= pattern { "," pattern }
    ;
```

---

## ANTLR4 Grammar Extension

For `docs/grammar/Kleis_v05.g4`:

```antlr
// Pattern Matching (v0.5)

matchExpr
    : 'match' expression '{' matchCases '}'
    ;

matchCases
    : matchCase (('|' | NEWLINE) matchCase)*
    ;

matchCase
    : pattern '=>' expression
    ;

pattern
    : '_'                                         # wildcardPattern
    | LOWERCASE_ID                                # variablePattern
    | UPPERCASE_ID ('(' patternArgs ')')?         # constructorPattern
    | NUMBER                                      # constantPattern
    | STRING                                      # stringConstantPattern
    ;

patternArgs
    : pattern (',' pattern)*
    ;

// Lexer rules
UPPERCASE_ID : [A-Z][a-zA-Z0-9_]* ;
LOWERCASE_ID : [a-z_][a-zA-Z0-9_]* ;
```

---

## Example Programs (Syntactically Valid)

### Example 1: Bool Match

```kleis
data Bool = True | False

operation not : Bool → Bool

define not(b) = match b {
  True => False
  False => True
}
```

**Grammar derivation:**
```
matchExpr → "match" expression "{" matchCases "}"
  expression → identifier("b")
  matchCases → matchCase "|" matchCase
    matchCase₁ → pattern "=>" expression
      pattern → constructorPattern("True")
      expression → identifier("False")
    matchCase₂ → pattern "=>" expression
      pattern → constructorPattern("False")
      expression → identifier("True")
```

---

### Example 2: Option with Variable Binding

```kleis
data Option(T) = None | Some(T)

operation getOrElse : Option(T) → T → T

define getOrElse(opt, default) = match opt {
  None => default
  Some(x) => x
}
```

**Grammar derivation:**
```
matchExpr → "match" expression "{" matchCases "}"
  expression → identifier("opt")
  matchCases → matchCase "|" matchCase
    matchCase₁ → pattern "=>" expression
      pattern → constructorPattern("None")
      expression → identifier("default")
    matchCase₂ → pattern "=>" expression
      pattern → constructorPattern("Some", patternArgs)
        patternArgs → pattern
          pattern → variablePattern("x")
      expression → identifier("x")
```

---

### Example 3: Nested Patterns

```kleis
data Result(T, E) = Ok(T) | Err(E)
data Option(T) = None | Some(T)

operation extract : Result(Option(T), String) → T → T

define extract(result, default) = match result {
  Ok(Some(x)) => x
  Ok(None) => default
  Err(_) => default
}
```

**Grammar derivation:**
```
matchExpr → "match" expression "{" matchCases "}"
  matchCases → matchCase "|" matchCase "|" matchCase
    matchCase₁ → pattern "=>" expression
      pattern → constructorPattern("Ok", patternArgs)
        patternArgs → pattern
          pattern → constructorPattern("Some", patternArgs)
            patternArgs → pattern
              pattern → variablePattern("x")
      expression → identifier("x")
    matchCase₂ → pattern "=>" expression
      pattern → constructorPattern("Ok", patternArgs)
        patternArgs → pattern
          pattern → constructorPattern("None")
      expression → identifier("default")
    matchCase₃ → pattern "=>" expression
      pattern → constructorPattern("Err", patternArgs)
        patternArgs → pattern
          pattern → wildcardPattern("_")
      expression → identifier("default")
```

---

### Example 4: Wildcard Pattern

```kleis
data Status = Idle | Running | Paused | Completed | Failed

operation isActive : Status → Bool

define isActive(status) = match status {
  Running => True
  _ => False
}
```

---

### Example 5: Pattern Matching in Lambda Calculus

```kleis
data LambdaTerm =
  | Var(String)
  | Abs(String, LambdaTerm)
  | App(LambdaTerm, LambdaTerm)

operation eval : LambdaTerm → Value

define eval(term) = match term {
  Var(x) => lookup(context, x)
  Abs(x, body) => Closure(x, body, context)
  App(e1, e2) => apply(eval(e1), eval(e2))
}
```

---

## Grammar Changes Summary

### v0.4 → v0.5 Changes

**Added:**
- `matchExpr` production
- `matchCases` production
- `matchCase` production
- `pattern` production (4 variants)
- `patternArgs` production

**Modified:**
- `expression` - Added `matchExpr` alternative

**Total new productions:** 6  
**Total modified productions:** 1

---

## Integration with Existing Grammar

### Expression Grammar (Extended)

**v0.4 (Before):**
```ebnf
expression
    ::= term
      | operation
      | functionCall
      | literal
      | identifier
      | "(" expression ")"
      ;
```

**v0.5 (After):**
```ebnf
expression
    ::= term
      | operation
      | functionCall
      | matchExpr        (* NEW *)
      | literal
      | identifier
      | "(" expression ")"
      ;
```

### New Non-Terminals

```ebnf
matchExpr      ::= "match" expression "{" matchCases "}"
matchCases     ::= matchCase { ( "|" | newline ) matchCase }
matchCase      ::= pattern "=>" expression
pattern        ::= "_" | LOWERCASE_ID | UPPERCASE_ID [ "(" patternArgs ")" ] | constant
patternArgs    ::= pattern { "," pattern }
```

---

## Lexical Rules

### Case Sensitivity Matters

**New lexical distinction:**

```ebnf
(* Lexical tokens *)
UPPERCASE_ID ::= [A-Z] [a-zA-Z0-9_]*    (* Constructors: Some, None, True *)
LOWERCASE_ID ::= [a-z_] [a-zA-Z0-9_]*   (* Variables: x, value, result *)
WILDCARD     ::= "_"                     (* Wildcard pattern *)
ARROW        ::= "=>"                    (* Pattern arrow *)
```

**This is NEW for Kleis!** Previously, case didn't matter. Now:
- Uppercase = Data constructor (semantic meaning)
- Lowercase = Variable binding (semantic meaning)

---

## Ambiguities and Resolutions

### 1. Brace Ambiguity

**Problem:** Braces used for multiple purposes

```kleis
structure S { ... }   // Structure body
match x { ... }       // Match cases
{ stmt1; stmt2 }      // Block (if we add it later)
```

**Resolution:** Context determines meaning
- After "structure" → structure body
- After "match expr" → match cases
- Grammar is unambiguous due to keyword

---

### 2. Arrow Ambiguity

**Problem:** Two arrow symbols

```kleis
operation f : A → B   // Type arrow (→)
match x { p => e }    // Pattern arrow (=>)
```

**Resolution:** Different symbols, no ambiguity
- `→` (U+2192) for types
- `=>` (ASCII) for patterns

---

### 3. Constructor vs Variable

**Problem:** How to distinguish `Some` from `some`?

```kleis
match x {
  Some(value) => ...  // Some = constructor, value = variable
  some => ...         // some = variable (catches everything!)
}
```

**Resolution:** Case sensitivity rule
- UPPERCASE_ID = Constructor
- LOWERCASE_ID = Variable
- Enforced by lexer/parser

---

### 4. Separator Ambiguity

**Problem:** Cases separated by `|` or newline?

```kleis
// Version 1: Pipe-separated
match x { A => 1 | B => 2 | C => 3 }

// Version 2: Newline-separated  
match x {
  A => 1
  B => 2
  C => 3
}

// Version 3: Mixed
match x {
  A => 1 | B => 2
  C => 3
}
```

**Resolution:** Support both
- Pipe `|` is optional separator
- Newlines are whitespace (automatically separate)
- Grammar: `matchCase { ("|" | newline) matchCase }`

---

## Formal Grammar Files to Update

### Required Updates

**1. `docs/grammar/kleis_grammar_v05.ebnf`** (NEW)
- Copy from v0.4
- Add pattern matching productions
- Add lexical rules for case sensitivity

**2. `docs/grammar/Kleis_v05.g4`** (NEW)
- Copy from v0.4
- Add pattern matching rules
- Add UPPERCASE_ID and LOWERCASE_ID tokens

**3. `docs/grammar/kleis_grammar_v05.md`** (NEW)
- Human-readable explanation
- Examples with derivations
- Ambiguity resolutions

### Version Timeline

- **v0.3:** Bootstrap grammar (original)
- **v0.4:** + Algebraic data types (ADR-021 Part 1) ✅ Implemented
- **v0.5:** + Pattern matching (ADR-021 Part 2) 📋 Planned
- **v0.6:** + String literals, let bindings (future)
- **v0.7:** + Lambda expressions, higher-order functions (future)

---

## Implementation Notes

### Parser Must Handle

1. **Keyword Recognition:** `match` keyword
2. **Case Sensitivity:** UPPERCASE vs lowercase
3. **Nesting:** Patterns can be nested (recursive parsing)
4. **Separator Flexibility:** Both `|` and newlines
5. **Arrow Distinction:** `=>` not `→`

### Type Checker Must Handle

1. **Pattern Type Checking:** Pattern must match scrutinee type
2. **Variable Binding:** Pattern variables get bound to types
3. **Branch Unification:** All branches must have same result type
4. **Exhaustiveness:** Check all constructors covered
5. **Unreachability:** Warn on unreachable patterns

### Evaluator Must Handle

1. **Pattern Matching:** Try patterns in order
2. **Variable Substitution:** Bind matched values
3. **Nested Matching:** Recursive pattern matching
4. **Non-Exhaustive Error:** Runtime error if no match

---

## Compatibility with Existing Grammar

### No Conflicts

Pattern matching integrates cleanly:
- ✅ New keyword (`match`) - no existing usage
- ✅ New arrow (`=>`) - distinct from type arrow `→`
- ✅ Braces after expression - unambiguous context
- ✅ Patterns are distinct from expressions

### Complementary Features

Pattern matching works with:
- ✅ Data types (v0.4) - matches on constructors
- ✅ Type system - type checks patterns
- ✅ Operations - can use match in operation bodies
- ✅ Structures - patterns can appear in implementations

---

## Example: Full Grammar Derivation

**Program:**
```kleis
data Option(T) = None | Some(T)

operation unwrap : Option(ℝ) → ℝ

define unwrap(opt) = match opt {
  None => 0
  Some(x) => x
}
```

**Grammar Derivation:**

```
program
├─ declaration (dataDef)
│  └─ "data" identifier("Option") "(" typeParams ")" "=" dataVariants
│     └─ dataVariant("None") "|" dataVariant("Some", fields)
│
└─ declaration (operationDecl)
   └─ "operation" identifier("unwrap") ":" typeSignature
      └─ type("Option(ℝ)") "→" type("ℝ")

└─ declaration (functionDef)
   └─ "define" identifier("unwrap") "(" params ")" "=" expression
      └─ expression (matchExpr)
         ├─ "match" expression(identifier("opt")) "{"
         ├─ matchCases
         │  ├─ matchCase
         │  │  ├─ pattern (constructorPattern("None"))
         │  │  ├─ "=>"
         │  │  └─ expression (identifier("0"))
         │  ├─ "|"
         │  └─ matchCase
         │     ├─ pattern (constructorPattern("Some", args))
         │     │  └─ patternArgs: pattern(variablePattern("x"))
         │     ├─ "=>"
         │     └─ expression (identifier("x"))
         └─ "}"
```

---

## Testing the Grammar

### Valid Programs

All these should parse successfully:

```kleis
// Simple match
match x { True => 1 | False => 0 }

// With variable binding
match opt { None => 0 | Some(x) => x }

// Nested patterns
match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => -1 }

// Wildcard
match status { Running => 1 | _ => 0 }

// Newline-separated
match value {
  A => 1
  B => 2
  C => 3
}

// Mixed separators
match value {
  A => 1 | B => 2
  C => 3
}
```

### Invalid Programs

All these should produce parse errors:

```kleis
// Missing braces
match x True => 1 | False => 0

// Missing arrow
match x { True 1 | False 0 }

// Wrong arrow
match x { True -> 1 | False -> 2 }

// No cases
match x { }

// Missing expression
match { True => 1 }

// Incomplete case
match x { True => }
```

---

## Changes Required in Parser

### New Methods Needed

**In `src/kleis_parser.rs`:**

```rust
impl KleisParser {
    fn parse_match_expr(&mut self) -> Result<Expression, KleisParseError>
    fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, KleisParseError>
    fn parse_match_case(&mut self) -> Result<MatchCase, KleisParseError>
    fn parse_pattern(&mut self) -> Result<Pattern, KleisParseError>
    fn parse_pattern_args(&mut self) -> Result<Vec<Pattern>, KleisParseError>
    
    // Helpers:
    fn peek_word(&self) -> Option<&str>
    fn expect_word(&mut self, word: &str) -> Result<(), KleisParseError>
    fn expect_char(&mut self, ch: char) -> Result<(), KleisParseError>
    fn consume_str(&mut self, s: &str) -> bool
}
```

### Integration Point

**In `parse_primary()`:**

```rust
fn parse_primary(&mut self) -> Result<Expression, KleisParseError> {
    self.skip_whitespace();
    
    // NEW: Check for match keyword
    if self.peek_word() == Some("match") {
        return self.parse_match_expr();
    }
    
    // ... existing code for other expressions ...
}
```

---

## Backward Compatibility

### No Breaking Changes

**All v0.4 programs remain valid:**
- `match` is a new keyword (no existing usage)
- Pattern syntax is new (no conflicts)
- Grammar is strictly extended (subset property)

**Migration path:** None needed! v0.4 code works in v0.5

---

## Future Extensions (v0.6+)

### Possible Enhancements

**Guards:**
```kleis
match x {
  Some(n) if n > 0 => positive(n)
  Some(n) if n < 0 => negative(n)
  Some(0) => zero
  None => default
}
```

**As-patterns:**
```kleis
match tree {
  node@Node(left, right) => {
    // Both 'node' and 'left'/'right' in scope
    process(node, left, right)
  }
}
```

**List patterns:**
```kleis
match list {
  [] => empty
  [x] => single(x)
  [x, y] => pair(x, y)
  x :: xs => cons(x, xs)
}
```

**Or-patterns:**
```kleis
match status {
  Idle | Paused => inactive()
  Running | Loading => active()
}
```

---

## Grammar Validation

### Ambiguity Check

**The grammar is unambiguous because:**

1. **Keywords are reserved:** `match` can't be an identifier
2. **Arrow is unique:** `=>` only used in patterns
3. **Braces are scoped:** After `match expr`, braces start cases
4. **Case distinction:** UPPERCASE vs lowercase is syntactic

**LL(1) parsable?** Yes, with 1-token lookahead:
- See `match` → parse match expression
- See `{` after expression → match cases
- See `=>` after pattern → match case body
- See `|` or newline → next case

---

## Documentation Artifacts to Create

**When implementing v0.5:**

1. ✅ `docs/grammar/kleis_grammar_v05.ebnf` - EBNF specification
2. ✅ `docs/grammar/Kleis_v05.g4` - ANTLR4 grammar
3. ✅ `docs/grammar/kleis_grammar_v05.md` - Human-readable guide
4. ✅ `CHANGELOG.md` - Document v0.5 release
5. ✅ Update `docs/KLEIS_PARSER_STATUS.md` - Parser coverage

---

## Relation to Formal Specification

### Grammar Hierarchy

```
Kleis v0.3 (Bootstrap)
  ├─ Basic expressions
  ├─ Operations
  └─ Structures
  
Kleis v0.4 (+ ADR-021 Part 1)
  ├─ Everything from v0.3
  └─ Data type definitions     ← Implemented Dec 8, 2025
  
Kleis v0.5 (+ ADR-021 Part 2)
  ├─ Everything from v0.4
  └─ Pattern matching          ← Planned, grammar specified here
  
Kleis v1.0 (Full Grammar)
  ├─ Everything from v0.5
  ├─ Lambda expressions
  ├─ Let bindings
  ├─ Type annotations
  └─ Full operator support
```

### Formal Specification Status

| Feature | Grammar | Parser | Type Checker | Evaluator |
|---------|---------|--------|--------------|-----------|
| Data types | ✅ v0.4 | ✅ Done | ✅ Done | ✅ Done |
| Pattern matching | ✅ v0.5 spec | 📋 TODO | 📋 TODO | 📋 TODO |
| Lambda expressions | 📋 Future | ❌ | ❌ | ❌ |
| Let bindings | 📋 Future | ❌ | ❌ | ❌ |

---

## Implementation Checklist

**When implementing pattern matching:**

- [ ] Create `docs/grammar/kleis_grammar_v05.ebnf`
- [ ] Create `docs/grammar/Kleis_v05.g4`  
- [ ] Create `docs/grammar/kleis_grammar_v05.md`
- [ ] Update `docs/KLEIS_PARSER_STATUS.md`
- [ ] Implement parser methods in `src/kleis_parser.rs`
- [ ] Add parser tests (10+ tests)
- [ ] Implement type inference in `src/type_inference.rs`
- [ ] Add type inference tests (10+ tests)
- [ ] Implement evaluation (new module or extend existing)
- [ ] Add evaluation tests (10+ tests)
- [ ] Implement exhaustiveness checking
- [ ] Add exhaustiveness tests (5+ tests)
- [ ] Update `CHANGELOG.md` with v0.5 release notes

---

## References

- **ADR-021:** Algebraic Data Types (pattern matching is Part 2)
- **Kleis v0.4 Grammar:** `docs/grammar/kleis_grammar_v04.ebnf`
- **Implementation Plan:** `docs/archive/sessions/session-2025-12-08/PATTERN_MATCHING_IMPLEMENTATION_PLAN.md`
- **Value Proposition:** `docs/archive/sessions/session-2025-12-08/WHY_PATTERN_MATCHING_MATTERS.md`

---

**Status:** Grammar extension specified, ready for implementation  
**Next:** Create formal v0.5 grammar files when implementing parser  
**Impact:** Kleis v0.5 will be a complete functional language with ADTs + pattern matching

