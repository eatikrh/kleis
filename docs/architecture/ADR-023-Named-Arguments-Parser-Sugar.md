# ADR-023: Named Arguments as Parser-Level Sugar

**Status:** Implemented  
**Date:** 2026-01-01  
**Author:** AI Assistant  
**Reviewed by:** Eatik

## Context

Kleis plotting functions need to accept optional configuration parameters like:
- `offset`, `width`, `label` for bar charts
- `yerr`, `color`, `stroke` for line plots
- `legend_position`, `width`, `height` for diagrams

The Lilaq plotting library (which Kleis uses for rendering) supports these extensively.

## Problem

How do we add named/keyword arguments to Kleis without:
1. Complicating the Hindley-Milner type system
2. Interfering with symbolic expression handling
3. Breaking existing unification algorithms

## Decision

**Implement named arguments as parser-level syntactic sugar that produces a `record` expression.**

### Transformation

```kleis
// User writes:
bar(xs, ys, offset = -0.2, width = 0.4, label = "Left")

// Parser transforms to:
bar(xs, ys, record(
    field("offset", -0.2),
    field("width", 0.4),
    field("label", "Left")
))
```

### Implementation

The parser's `parse_arguments` function was modified to:

1. **Detect named argument syntax**: `identifier = expression` where `=` is not `==`
2. **Collect named arguments** into `field(name, value)` expressions
3. **Wrap all named arguments** in a single `record(...)` expression
4. **Append the record** as the last positional argument

```rust
// In kleis_parser.rs
fn parse_arguments(&mut self) -> Result<Vec<Expression>, KleisParseError> {
    let mut positional_args = Vec::new();
    let mut named_args = Vec::new();
    
    // ... parsing loop ...
    
    // If named args exist, wrap in record and append
    if !named_args.is_empty() {
        let record = Expression::Operation {
            name: "record".to_string(),
            args: named_args,
            span: Some(self.current_span()),
        };
        positional_args.push(record);
    }
    
    Ok(positional_args)
}
```

## Why This Doesn't Interfere with Symbolic Operations

### 1. Type System Isolation

Named arguments become a `record` expression which:
- Is opaque to the type checker (not polymorphic)
- Is consumed at runtime by built-in functions
- Never participates in type unification

```
Type inference sees:
  bar : List(ℝ) → List(ℝ) → record → PlotElement
                              ↑
                        Opaque type, not unified
```

### 2. No New Type Constructs

We don't introduce:
- Row polymorphism for records
- Optional parameter types
- Named parameter types in function signatures

The record is just an `Expression::Operation` like any other.

### 3. Consumed by Built-in Functions Only

The `record` expression is:
- Parsed by the evaluator's `parse_element_options` function
- Used only by built-in plotting functions (`bar`, `plot`, `diagram`, etc.)
- Never sent to Z3 for verification
- Never appears in user-defined function signatures

```rust
// In evaluator.rs - options are runtime config, not symbolic
fn parse_element_options(&self, expr: &Expression, options: &mut PlotElementOptions) {
    if let Expression::Operation { name, args, .. } = expr {
        if name == "record" {
            for opt in args {
                // Extract field name and value
                // Apply to options struct
            }
        }
    }
}
```

### 4. Ground Terms Only

Named arguments must be:
- Concrete values (numbers, strings, lists of numbers)
- Not symbolic expressions with free variables
- Immediately evaluable

This ensures they never enter the symbolic domain.

## Comparison with Alternatives

| Approach | Type System Impact | Unification Impact | Effort |
|----------|-------------------|-------------------|--------|
| **Parser sugar (chosen)** | None | None | Low |
| Full named parameters | High | Significant | High |
| Anonymous record types | Medium | Medium | Medium |
| Explicit options struct | None | None | Verbose |

## Examples

### Basic Usage

```kleis
// Before (not possible)
bar(xs, ys)  // No way to set options

// After
bar(xs, ys, offset = -0.2, width = 0.4, label = "Left")
```

### Full Lilaq Example

```kleis
let xs = [0, 1, 2, 3]
let ys1 = [1.35, 3, 2.1, 4]
let ys2 = [1.4, 3.3, 1.9, 4.2]
let yerr1 = [0.2, 0.3, 0.5, 0.4]
let yerr2 = [0.3, 0.3, 0.4, 0.7]

// Compute offset x-coordinates
let xs_left = list_map(λ x . x - 0.2, xs)
let xs_right = list_map(λ x . x + 0.2, xs)

diagram(
    bar(xs, ys1, offset = -0.2, width = 0.4, label = "Left"),
    bar(xs, ys2, offset = 0.2, width = 0.4, label = "Right"),
    plot(xs_left, ys1, yerr = yerr1, color = "black", stroke = "none"),
    plot(xs_right, ys2, yerr = yerr2, color = "black", stroke = "none")
)
```

### Diagram Options

```kleis
diagram(
    width = 10,
    height = 7,
    title = "My Plot",
    xlabel = "X",
    ylabel = "Y",
    legend = "right + top",
    plot(xs, ys)
)
```

## Consequences

### Positive

1. **Clean syntax** - Matches Lilaq documentation 1:1
2. **No type system changes** - Unification unaffected
3. **No Z3 impact** - Symbolic operations work as before
4. **Easy to implement** - Parser change only
5. **Familiar** - Similar to Python, Scala, etc.

### Negative

1. **Built-in only** - User-defined functions can't use this pattern
2. **No type checking** - Invalid option names silently ignored
3. **Positional must come first** - Can't interleave named and positional

### Future Work

If user-defined functions with optional parameters are needed:
- Consider full named parameter support in type system
- Implement row polymorphism for record types
- Add default parameter values to function definitions

## Related ADRs

- ADR-014: Hindley-Milner Type System
- ADR-016: Operations in Structures

## Files Changed

- `src/kleis_parser.rs`: `parse_arguments`, `try_parse_identifier`
- `src/evaluator.rs`: `parse_element_options`, `parse_diagram_options`
- `examples/plotting/basic_plots.kleis`: Updated examples

