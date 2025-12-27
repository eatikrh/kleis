# Kleis Grammar v0.94 - N-ary Product Types

**Date:** 2025-12-26  
**Based on:** v0.93 (example blocks)

## Overview

v0.94 adds **n-ary product types** to Kleis, allowing type signatures with multiple factors:

```kleis
-- Binary (v0.93):
operation f : A × B → C

-- N-ary (v0.94):
operation f : A × B × C × D → E
```

## Motivation: POT Formalization

The Projected Ontology Theory (POT) formalization requires multi-argument operations:

```kleis
-- These couldn't be expressed in v0.93:
operation metric_probe : FieldR4 × Point × Point → ℝ
operation mass_at : GreenKernel × Flow × Event → ℝ
operation residue : FieldR4 × Event × Channel → ℝ
```

**Workaround in v0.93:** Bundle arguments into structures (verbose):
```kleis
structure MetricProbeArgs { element field : FieldR4; element u : Point; element v : Point }
operation metric_probe : MetricProbeArgs → ℝ
```

**v0.94 solution:** Direct n-ary product types (clean):
```kleis
operation metric_probe : FieldR4 × Point × Point → ℝ
```

## Grammar Changes

### v0.93 (Binary only)
```ebnf
productType ::= type "×" type ;

type
    ::= primitiveType
      | parametricType
      | functionType
      | tupleType
      | productType
      | typeVariable
      | "(" type ")"
      ;
```

### v0.94 (N-ary, right-associative)
```ebnf
(* Top-level type: function types bind loosest *)
type
    ::= functionType
      | productType
      ;

(* Function types: right-associative *)
functionType
    ::= productType "→" type
      | productType "->" type
      ;

(* Product types: right-associative (NEW)
 * A × B × C  parses as  A × (B × C)
 *)
productType
    ::= simpleType "×" productType
      | simpleType
      ;

(* Simple types: no function or product *)
simpleType
    ::= primitiveType
      | parametricType
      | tupleType
      | typeVariable
      | "(" type ")"
      ;
```

## Semantics

### Right-associative
Product types are **right-associative**:

```kleis
A × B × C × D  ≡  A × (B × C × D)  ≡  A × (B × (C × D))
```

This matches nested pairs in type theory.

### Precedence
Function types (`→`) bind **looser** than product types (`×`):

```kleis
A × B → C × D → E

-- Parses as:
(A × B) → ((C × D) → E)
```

### Explicit Grouping
Use parentheses for left-associative grouping:

```kleis
(A × B) × C    -- Left-grouped pair of pairs
A × B × C      -- Right-grouped: A × (B × C)
```

## Examples

### POT Formalization
```kleis
structure GreenKernel {
    operation project : Flow → FieldR4
}

structure ResidueOperations {
    -- N-ary product type: 3 arguments
    operation residue : FieldR4 × Event × Channel → ℝ
    
    -- 4 arguments
    operation mass_at : GreenKernel × Flow × Event → ℝ
}

structure KernelMetricInterface {
    -- Metric probe with 3 type arguments
    operation metric_probe : FieldR4 × Vector(4, ℝ) × Vector(4, ℝ) → ℝ
}
```

### Mixed with Functions
```kleis
-- Higher-order function with product domain
operation apply : (A × B → C) × A × B → C

-- Curried equivalent
operation apply_curried : (A × B → C) → A → B → C
```

## Parser Implementation Notes

The key change is in `parse_type()`:

1. **Entry point:** Try function type first
2. **Function type:** Parse product type, then check for `→`
3. **Product type:** Parse simple type, then loop on `×`
4. **Simple type:** Everything else (primitives, parametric, parenthesized)

```rust
fn parse_type(&mut self) -> Result<Type, Error> {
    self.parse_function_type()
}

fn parse_function_type(&mut self) -> Result<Type, Error> {
    let left = self.parse_product_type()?;
    if self.peek_is("→") || self.peek_is("->") {
        self.advance();
        let right = self.parse_function_type()?;  // Right-assoc
        Ok(Type::Function(Box::new(left), Box::new(right)))
    } else {
        Ok(left)
    }
}

fn parse_product_type(&mut self) -> Result<Type, Error> {
    let left = self.parse_simple_type()?;
    if self.peek_is("×") {
        self.advance();
        let right = self.parse_product_type()?;  // Right-assoc
        Ok(Type::Product(Box::new(left), Box::new(right)))
    } else {
        Ok(left)
    }
}
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.94 | 2025-12-26 | N-ary product types |
| v0.93 | 2025-12-24 | Example blocks, assert |
| v0.92 | 2025-12-22 | Type-level arithmetic |
| v0.91 | 2025-12-22 | Parameterized type aliases |
| v0.8 | 2025-12-18 | Pattern matching, imports |

