# Kleis Grammar v0.96 - Named Arguments

**Date:** 2026-01-01  
**Based on:** v0.95 (Big Operator Syntax)  
**EBNF:** `kleis_grammar_v096.ebnf`

## Summary

v0.96 introduces **named arguments** (keyword arguments) for function calls, primarily to support plotting configuration options.

## New Syntax

```kleis
// Named arguments in function calls
bar(xs, ys, offset = -0.2, width = 0.4, label = "Left")
plot(x, y, yerr = errors, color = "blue", stroke = "none")
diagram(width = 10, height = 7, title = "My Plot", plot(x, y))
```

## Grammar Change

### Previous (v0.95)

```ebnf
arguments ::= expression { "," expression } ;
```

### New (v0.96)

```ebnf
arguments 
    ::= positionalArgs [ "," namedArgs ]
      | namedArgs
      | (* empty *)
      ;

positionalArgs
    ::= expression { "," expression }
      ;

namedArgs
    ::= namedArg { "," namedArg }
      ;

namedArg
    ::= identifier "=" expression   (* Note: '=' not '==' *)
      ;
```

## Parser Transformation

Named arguments are **syntactic sugar** that the parser transforms at parse time:

```
User writes:
  bar(xs, ys, offset = -0.2, width = 0.4)

Parser produces:
  bar(xs, ys, record(
      field("offset", -0.2),
      field("width", 0.4)
  ))
```

This transformation means:
1. **No type system changes** - The type checker sees a regular function call
2. **No unification impact** - The `record` is opaque, not polymorphic
3. **No Z3 changes** - Records are consumed at runtime, never verified

## Why This Doesn't Affect Symbolic Operations

### Design Principle

Named arguments are **consumed by built-in functions at runtime**, never entering the symbolic domain:

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│  Parser         │      │  Type Checker    │      │  Evaluator      │
│                 │      │                  │      │                 │
│  name = value   │──────│  Sees: record    │──────│  Extracts       │
│  ↓              │      │  (opaque type)   │      │  options        │
│  field(...)     │      │  No unification  │      │  Calls Lilaq    │
│  ↓              │      │  No constraints  │      │                 │
│  record(...)    │      │                  │      │                 │
└─────────────────┘      └──────────────────┘      └─────────────────┘
```

### What Stays Unchanged

| Component | Impact |
|-----------|--------|
| Type inference | None - record is opaque |
| Unification | None - record doesn't unify |
| Z3 verification | None - records not sent to Z3 |
| Axioms/theorems | None - records can't appear in axioms |
| Pattern matching | None - records not pattern-matched |
| User-defined functions | None - extra args ignored |

### Example: Type System View

```kleis
// User writes
bar(xs, ys, offset = -0.2, label = "Left")

// Type checker sees
bar : List(ℝ) → List(ℝ) → record → PlotElement
                           ↑
                     Opaque type, not unified with anything
```

## Constraints

1. **Positional before named**: All positional arguments must come before named arguments
2. **Valid identifiers**: Named argument keys must be valid Kleis identifiers
3. **Single equals**: Uses `=` not `==` (parser disambiguates)
4. **Built-in consumption**: Only built-in functions process the record

## Use Cases

### Plotting (Primary)

```kleis
// Lilaq-style grouped bar chart with error bars
diagram(
    bar(xs, ys1, offset = -0.2, width = 0.4, label = "Left"),
    bar(xs, ys2, offset = 0.2, width = 0.4, label = "Right"),
    plot(xs_left, ys1, yerr = yerr1, color = "black", stroke = "none"),
    plot(xs_right, ys2, yerr = yerr2, color = "black", stroke = "none")
)
```

### Diagram Configuration

```kleis
diagram(
    width = 10,
    height = 7,
    title = "My Plot",
    xlabel = "X",
    ylabel = "Y",
    legend = "right + top",
    plot(x, y)
)
```

## Future Work

If user-defined functions with optional parameters are needed:
- Consider full named parameter support in type system
- Implement row polymorphism for record types
- Add default parameter values to function definitions

For now, v0.96 provides a clean, focused solution for the plotting use case without impacting Kleis's symbolic computation capabilities.

## Related Documents

- `ADR-023-Named-Arguments-Parser-Sugar.md` - Architectural decision record
- `kleis_grammar_v095.ebnf` - Previous version (big operators)
- `PLOTTING_ROADMAP.md` - Plotting feature planning

