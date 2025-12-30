# Kleis Grammar v0.8 - Advanced Pattern Matching

**Date:** 2025-12-18  
**Status:** Official  
**Branch:** `feature/grammar-v08-patterns`

## New in v0.8

This release adds four major features: import statements and three advanced pattern matching capabilities.

### 1. Import Statements

Import statements enable modular code organization:

```kleis
import "stdlib/algebra.kleis"
import "./local/types.kleis"
```

**Semantics:**
- Loads all structures, data types, and functions into current scope
- Cascading: imports in loaded files are also processed
- Circular-safe: each file loaded at most once
- Typically placed at the beginning of a file

**Path resolution:**
- Absolute paths: used as-is
- `stdlib/` prefix: relative to standard library directory
- Relative paths: relative to importing file's directory

**REPL integration:**
- `:load file.kleis` automatically processes all import statements
- Reports total files, functions, structures loaded

### 2. Pattern Guards

Guards allow conditional matching beyond structural patterns:

```kleis
match x {
    n if n < 0 => "negative"
    n if n > 0 => "positive"
    _ => "zero"
}
```

**Semantics:**
- Guard expression must evaluate to a boolean
- Pattern matches only when guard evaluates to `True`
- If guard fails, matching continues to next case

**Use cases:**
- Range checking
- Additional constraints beyond structure
- Complex predicates

### 3. As-Patterns (Alias Binding)

Bind both the destructured parts AND the whole value:

```kleis
match list {
    Cons(h, t) as whole => process(h, t, whole)
    Nil => empty
}
```

**Semantics:**
- `whole` binds to the entire matched value
- `h` and `t` bind to the destructured parts
- All bindings available in the body

**Use cases:**
- When you need both parts and the whole
- Avoiding reconstruction of matched value
- Recursive algorithms that need the original

### 4. Let Destructuring

Use patterns in let bindings, not just variable names:

```kleis
let Point(x, y) = origin in
    x^2 + y^2

let Some(Pair(a, b)) = lookup(key) in
    a + b
```

**Semantics:**
- Pattern must match the value (runtime error if not)
- Bound variables available in body
- Type annotations only valid for simple Variable patterns

**Use cases:**
- Extracting fields from structures
- Unpacking tuple-like values
- Cleaner alternative to nested match

## Grammar Changes

### Pattern Extensions

```ebnf
(* v0.8: Extended pattern with as-binding *)
pattern
    ::= basePattern [ "as" identifier ]
    ;

basePattern
    ::= wildcardPattern
      | variablePattern
      | constructorPattern
      | constantPattern
      ;
```

### Match Case with Guards

```ebnf
(* v0.8: Match case with optional guard *)
matchCase
    ::= pattern [ "if" expression ] "=>" expression
    ;
```

### Let Binding with Patterns

```ebnf
(* v0.8: Let binding supports full patterns *)
letBinding
    ::= "let" pattern [ typeAnnotation ] "=" expression "in" expression
    ;

(* Note: typeAnnotation only valid when pattern is a simple Variable *)
```

## Examples

### Pattern Guards

```kleis
-- Absolute value using guards
define abs(x) = match x {
    n if n < 0 => negate(n)
    n => n
}

-- FizzBuzz with guards
define fizzbuzz(n) = match n {
    x if mod(x, 15) == 0 => "FizzBuzz"
    x if mod(x, 3) == 0 => "Fizz"
    x if mod(x, 5) == 0 => "Buzz"
    x => toString(x)
}
```

### As-Patterns

```kleis
-- Duplicate first element
define duplicate_first(list) = match list {
    Cons(h, t) as whole => Cons(h, whole)
    Nil => Nil
}

-- Process with original
define transform(opt) = match opt {
    Some(x) as original => Pair(x, original)
    None => Pair(0, None)
}
```

### Let Destructuring

```kleis
-- Extract coordinates
define distance(p1, p2) =
    let Point(x1, y1) = p1 in
    let Point(x2, y2) = p2 in
    sqrt((x2 - x1)^2 + (y2 - y1)^2)

-- Unpack result
define process_result(r) =
    let Ok(Pair(status, value)) = r in
    if status then value else 0

-- Nested destructuring
define get_head(list) =
    let Cons(h, _) = list in h
```

## Z3 Integration

All v0.8 features are fully integrated with Z3 verification:

### Guards in Z3

Guards translate to conditional assertions:

```kleis
match x {
    n if n > 0 => f(n)
    _ => g(x)
}
```

Translates to Z3 ITE (if-then-else):
```
(ite (and (= x n) (> n 0))
     (f n)
     (g x))
```

### As-Patterns in Z3

Both bindings are asserted equal to appropriate values:

```kleis
Cons(h, t) as whole
```

Generates:
```
(and (= whole scrutinee)
     (= h (head scrutinee))
     (= t (tail scrutinee)))
```

### Let Patterns in Z3

Destructuring creates appropriate bindings:

```kleis
let Point(x, y) = p in body
```

Generates:
```
(let ((x (point-x p))
      (y (point-y p)))
  body)
```

## Breaking Changes

**None.** v0.8 is fully backward compatible with v0.7.

All existing code continues to work. The new features are purely additive:

- Existing patterns without `as` work unchanged
- Match cases without guards work unchanged  
- Simple `let x = ...` bindings work unchanged

## Implementation Status

| Feature | Parser | Evaluator | Type Inference | Z3 Backend | Pretty Printer |
|---------|--------|-----------|----------------|------------|----------------|
| Pattern Guards | ✅ | ✅ | ✅ | ✅ | ✅ |
| As-Patterns | ✅ | ✅ | ✅ | ✅ | ✅ |
| Let Destructuring | ✅ | ✅ | ✅ | ✅ | ✅ |

## Migration Guide

No migration needed. To adopt v0.8 features:

1. **Pattern Guards**: Add `if condition` after pattern, before `=>`
2. **As-Patterns**: Add `as name` after pattern to bind the whole
3. **Let Destructuring**: Use constructor patterns in let bindings

## Version History

- **v0.8** (2025-12-18): Pattern guards, as-patterns, let destructuring
- **v0.7** (2025-12-13): Mathematica-style calculus (D, Dt, Limit)
- **v0.6** (2025-12-12): Derived operations in structures
- **v0.5** (2025-12-08): Basic pattern matching
- **v0.4** (2025-12-08): Algebraic data types

## See Also

- [kleis_grammar_v08.ebnf](kleis_grammar_v08.ebnf) - Formal EBNF specification
- [PATTERN_MATCHING_GRAMMAR_EXTENSION.md](PATTERN_MATCHING_GRAMMAR_EXTENSION.md) - Original pattern matching design
- [ADR-021](../adr/adr-021-algebraic-data-types.md) - Algebraic data types foundation

