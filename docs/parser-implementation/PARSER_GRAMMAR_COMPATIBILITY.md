# Kleis Parser vs Formal Grammar Compatibility

**Date:** December 6, 2024  
**Formal Grammar:** Kleis v0.3 (ANTLR4)  
**Parser Implementation:** `src/kleis_parser.rs`

---

## TL;DR

⚠️ **My parser is a SUBSET of the formal grammar, intentionally simplified for ADR-015 POC**

**Coverage:** ~30% of formal grammar  
**Purpose:** Validate ADR-015 core design decisions  
**Status:** Good enough for POC, needs expansion for production

---

## What's Supported

### ✅ Supported in My Parser

| Feature | Formal Grammar | My Parser | Status |
|---------|----------------|-----------|--------|
| Identifiers | `IDENTIFIER` | ✅ `[a-zA-Z_][a-zA-Z0-9_]*` | ✅ Compatible |
| Numbers | `NUMBER` | ✅ `[0-9]+(\.[0-9]+)?` | ✅ Compatible |
| Function calls | `expression '(' arguments ')'` | ✅ `abs(x)`, `frac(a,b)` | ✅ Compatible |
| Parentheses | `'(' expression ')'` | ✅ `(a + b)` | ✅ Compatible |
| Addition | `'+'` in infixOp | ✅ `a + b` | ✅ Compatible |
| Subtraction | `'-'` in infixOp | ✅ `a - b` | ✅ Compatible |
| Multiplication | `'*'` in infixOp | ✅ `a * b` | ✅ Compatible |
| Division | `'/'` in infixOp | ✅ `a / b` | ✅ Compatible |
| Exponentiation | `'^'` in infixOp | ✅ `a ^ b` | ✅ Compatible |
| Arguments | `expression (',' expression)*` | ✅ Comma-separated | ✅ Compatible |

**Total:** 10/50+ features

---

## What's Missing

### ❌ Not Yet Supported

**From Formal Grammar (Kleis v0.3):**

#### 1. Prefix Operators
```antlr
prefixOp: '-' | '∇' | '∂' | '¬' | '√'
```
**Examples:**
- `-x` (unary minus)
- `∇f` (gradient)
- `∂x` (partial)
- `√x` (sqrt)

**My Parser:** ❌ No prefix operators

---

#### 2. Postfix Operators
```antlr
postfixOp: '!' | '†' | '*' | 'ᵀ' | '^T'
```
**Examples:**
- `n!` (factorial)
- `A†` (conjugate transpose)
- `Aᵀ` (transpose)

**My Parser:** ❌ No postfix operators

---

#### 3. Additional Infix Operators
```antlr
arithmeticOp: '×' | '·' | '⊗' | '∘'  // My parser has ×, ·
relationOp: '=' | '≠' | '<' | '>' | '≤' | '≥' | '≈' | '≡' | '~' | '∈' | '∉' | '⊂' | '⊆'
logicOp: '∧' | '∨' | '⟹' | '⟺' | '→' | '⇒'
calcOp: '∂' | '∫' | '∇' | 'd/dx'
```

**My Parser:** ⚠️ Only has: `+`, `-`, `*`, `/`, `^`, `×`, `·`  
**Missing:** Relations, logic, calculus operators as infix

---

#### 4. Symbolic Constants
```antlr
symbolicConstant: 'π' | 'e' | 'i' | 'ℏ' | 'c' | 'φ' | '∞' | '∅'
```

**My Parser:** ❌ These would be parsed as identifiers

---

#### 5. Vector/List Literals
```antlr
'[' expressions ']'
```
**Examples:**
- `[1, 2, 3]`
- `[x, y, z]`

**My Parser:** ❌ No list syntax

---

#### 6. Lambda Expressions
```antlr
lambda: 'λ' params '.' expression
      | 'lambda' params '.' expression
```
**Examples:**
- `λ x . x^2`
- `lambda x . sin(x)`

**My Parser:** ❌ No lambda support

---

#### 7. Let Bindings
```antlr
letBinding: 'let' IDENTIFIER typeAnnotation? '=' expression 'in' expression
```
**Examples:**
- `let x = 5 in x^2`

**My Parser:** ❌ No let bindings

---

#### 8. Conditionals
```antlr
conditional: 'if' expression 'then' expression 'else' expression
```
**Examples:**
- `if x > 0 then x else -x`

**My Parser:** ❌ No if/then/else

---

#### 9. Type Annotations
```antlr
typeAnnotation: ':' type
```
**Examples:**
- `x : ℝ`

**My Parser:** ❌ No type annotations in expressions

---

#### 10. Placeholders
```antlr
placeholder: '□'
```

**My Parser:** ❌ No placeholder support

---

## Grammar Comparison

### Formal Grammar Structure (v0.3)

```antlr
expression
    : primary
    | prefixOp expression              // ❌ Not in my parser
    | expression postfixOp             // ❌ Not in my parser
    | expression infixOp expression    // ⚠️ Limited operators
    | expression '(' arguments ')'     // ✅ Supported
    | '[' expressions ']'              // ❌ Not in my parser
    | lambda                           // ❌ Not in my parser
    | letBinding                       // ❌ Not in my parser
    | conditional                      // ❌ Not in my parser
```

### My Parser Structure (Simplified)

```rust
expression := term (('+' | '-') term)*           // Only + and -
term       := factor (('*' | '/') factor)*       // Only * and /
factor     := primary ('^' primary)?             // Only ^
primary    := identifier 
            | number 
            | function_call                      // identifier '(' args ')'
            | '(' expression ')'
```

**Intentionally simplified!** Only covers ADR-015 requirements.

---

## Why The Difference?

### My Parser Goals (ADR-015 POC)

1. ✅ Validate that explicit forms work: `abs(x)`, `frac(a,b)`
2. ✅ Show division vs fraction distinction: `a/b` vs `frac(a,b)`
3. ✅ Prove text → AST pipeline works
4. ✅ Test basic arithmetic with proper precedence

**Not trying to implement full Kleis grammar!**

### Formal Grammar Goals (Production)

1. Complete mathematical language
2. Support type system
3. Lambda calculus
4. Vector operations
5. Logical reasoning
6. Calculus operations

**Much more ambitious!**

---

## Compatibility Analysis

### Core Concepts: ✅ Compatible

**Both use same:**
- Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
- Numbers: `[0-9]+(\.[0-9]+)?`
- Function application: `f(args)`
- Parentheses for grouping
- Infix operators (subset)

**My parser is a valid subset of formal grammar!**

### AST Output: ✅ Compatible

**Both produce:**
```rust
Expression::Operation { name: String, args: Vec<Expression> }
Expression::Object(String)
Expression::Const(String)
```

**Perfect compatibility at AST level!**

---

## What Would Full Compatibility Require?

### To match formal grammar 100%:

**Estimated effort: 2-3 weeks**

1. **Prefix operators** (1 day)
   - Unary minus: `-x`
   - Gradient: `∇f`
   - Sqrt: `√x`

2. **Postfix operators** (1 day)
   - Factorial: `n!`
   - Transpose: `Aᵀ`
   - Conjugate: `A†`

3. **Extended operators** (2 days)
   - Relations: `=`, `≠`, `<`, `>`, `≤`, `≥`
   - Logic: `∧`, `∨`, `⟹`, `⟺`
   - Calculus: `∫`, `∂`, `∇`

4. **Vector literals** (1 day)
   - `[1, 2, 3]`
   - `[x, y, z]`

5. **Lambda expressions** (2 days)
   - `λ x . x^2`
   - `lambda (x y) . x + y`

6. **Let bindings** (1 day)
   - `let x = 5 in x^2`

7. **Conditionals** (1 day)
   - `if x > 0 then x else -x`

8. **Type annotations** (3 days)
   - `x : ℝ`
   - Type parsing and validation

9. **Symbolic constants** (0.5 day)
   - `π`, `e`, `i`, etc.

10. **Placeholders** (0.5 day)
    - `□` for structural editing

**Total: ~13 days of work**

---

## Recommendation

### For ADR-015: ✅ Current Parser is Perfect

The simplified parser:
- ✅ Validates core design decisions
- ✅ Tests explicit forms (`abs`, `frac`, etc.)
- ✅ Proves text → AST → render pipeline
- ✅ Is easy to understand and test

**Don't need full grammar for POC!**

### For Production: 🔄 Need Full Grammar

Eventually need:
1. Full ANTLR4 parser from `Kleis_v03.g4`
2. Or expand my parser incrementally
3. Or use parser generator (LALRPOP, pest, etc.)

**Options:**
- **Option A:** Use ANTLR4 to generate parser from grammar
- **Option B:** Expand my recursive descent parser
- **Option C:** Use Rust parser library (pest grammar already exists!)

---

## Using the Existing pest Grammar

**Good news:** There's already a pest grammar at `docs/kleis.pest`!

```pest
program         = { SOI ~ statement* ~ EOI }
expression      = _{ ident | number | "(" ~ expression ~ ")" | expression ~ binary_op ~ expression }
binary_op       = _{ "+" | "-" | "*" | "/" | "×" | "·" }
```

Could use this with the `pest` crate to get a production parser quickly!

---

## Comparison Table

| Feature | Formal Grammar (v0.3) | My Parser | pest Grammar |
|---------|----------------------|-----------|--------------|
| Function calls | ✅ | ✅ | ✅ |
| Basic arithmetic | ✅ | ✅ | ✅ |
| Prefix operators | ✅ | ❌ | ❓ |
| Postfix operators | ✅ | ❌ | ❓ |
| Vector literals | ✅ | ❌ | ❓ |
| Lambda | ✅ | ❌ | ❓ |
| Let bindings | ✅ | ❌ | ❓ |
| Conditionals | ✅ | ❌ | ❓ |
| Type annotations | ✅ | ❌ | ❓ |
| Symbolic constants | ✅ | ❌ | ❌ |
| Implementation | ANTLR4 spec | Rust code | pest spec |
| Status | Reference | POC | Partial |

---

## Conclusion

### ⚠️ My Parser is NOT Fully Compatible with Formal Grammar

**It's intentionally a simplified subset for ADR-015 POC.**

**Coverage:**
- ✅ Core expression parsing (identifiers, numbers, function calls)
- ✅ Basic operators with correct precedence
- ✅ Sufficient for ADR-015 validation
- ❌ Missing ~70% of formal grammar features

**This is OK for POC!** But production needs full grammar.

---

## Next Steps

### For ADR-015 (Now)
✅ **Current parser is sufficient**
- Validates design decisions
- All tests pass
- Proves concept

### For Production (Later)
🔄 **Choose implementation:**

1. **Generate from ANTLR4 grammar** (`Kleis_v03.g4`)
   - Pros: Matches formal spec exactly
   - Cons: ANTLR4 in Rust is less mature

2. **Use pest parser** (`kleis.pest`)
   - Pros: Native Rust, good tooling
   - Cons: Need to update pest grammar to v0.3 spec

3. **Expand my parser** (`kleis_parser.rs`)
   - Pros: Already working, incremental
   - Cons: Manual work, potential bugs

**Recommendation:** Option 2 (pest) for production, keep my simple parser for ADR-015.

---

**Status:** ⚠️ **Subset Only - Good for POC, Not Production Ready**  
**Compatibility:** ~30% of formal grammar  
**Recommendation:** Keep for ADR-015, use pest/ANTLR for production

