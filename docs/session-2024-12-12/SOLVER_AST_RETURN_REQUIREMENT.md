# Solver Module Must Return Kleis AST

**Date:** December 12, 2024  
**Critical Requirement:** Solver backends MUST return Kleis AST, not solver-specific types

---

## 🎯 The Requirement

**User's insight:** "The solver module needs to return Kleis AST"

This is **architecturally critical** for:
1. ✅ Solver independence
2. ✅ User-facing API consistency
3. ✅ Composability
4. ✅ REPL support
5. ✅ Future extensibility

---

## ❌ Wrong Approach: Expose Solver Types

**Bad API (leaky abstraction):**
```rust
pub fn evaluate(&mut self, expr: &Expression) -> Result<z3::ast::Int, String>
//                                                        ^^^^^^^^^^^^^ ❌ Z3-specific!

// User code forced to deal with Z3 types:
let result = solver.evaluate(&expr)?;  // Returns z3::ast::Int
let value = result.as_i64()?;          // User deals with Z3 API!
let kleis_expr = Expression::Const(value.to_string());  // Manual conversion
```

**Problems:**
- ❌ User code depends on Z3
- ❌ Can't swap solvers without changing user code
- ❌ Leaks implementation details
- ❌ Violates abstraction principle

---

## ✅ Correct Approach: Return Kleis AST

**Good API (proper abstraction):**
```rust
pub fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String>
//                                                        ^^^^^^^^^^ ✅ Kleis AST!

// User code is solver-agnostic:
let result = solver.evaluate(&expr)?;  // Returns Expression
match result {
    Expression::Const(value) => println!("Result: {}", value),
    _ => println!("Symbolic: {:?}", result),
}
// User never sees Z3 types!
```

**Benefits:**
- ✅ Solver-independent user code
- ✅ Can swap Z3 for CVC5 transparently
- ✅ Clean abstraction
- ✅ Composable (Kleis AST → Kleis AST)

---

## 🏗️ Architecture: AST as Boundary

### Abstraction Layers

```
┌─────────────────────────────────────────┐
│  User Code (Kleis AST only)            │
├─────────────────────────────────────────┤
│  Solver Interface (returns AST)        │  ← Abstraction boundary
├─────────────────────────────────────────┤
│  Z3 Backend (internal Z3 types)        │
│  - Translates: AST → Z3                │
│  - Converts back: Z3 → AST             │  ← Conversion happens here
├─────────────────────────────────────────┤
│  Z3 Solver (z3::ast::Int, etc.)        │
└─────────────────────────────────────────┘
```

**Key principle:** Users only see Kleis `Expression`, never Z3/CVC5 types!

---

## 🔧 Implementation: ResultConverter

### The Converter is Critical

```rust
/// Converts solver-specific results back to Kleis AST
/// This is the abstraction boundary!
pub trait ResultConverter {
    /// PRIMARY METHOD: Convert to Kleis Expression
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String>;
}

/// Z3-specific implementation
pub struct Z3ResultConverter;

impl ResultConverter for Z3ResultConverter {
    fn to_expression(&self, value: &SolverValue) -> Result<Expression, String> {
        match value {
            // Numeric values → Const
            SolverValue::Integer(i) => {
                Ok(Expression::Const(i.to_string()))
            }
            
            SolverValue::Real(r) => {
                Ok(Expression::Const(r.to_string()))
            }
            
            // Boolean values → constructors
            SolverValue::Boolean(true) => {
                Ok(Expression::Object("True".to_string()))
            }
            
            SolverValue::Boolean(false) => {
                Ok(Expression::Object("False".to_string()))
            }
            
            // Symbolic/uninterpreted → keep symbolic
            SolverValue::Symbolic(s) => {
                // Parse SMT-LIB back to Kleis AST
                parse_smt_lib_to_ast(s)
            }
        }
    }
}
```

---

## 📊 API Examples

### Example 1: Evaluation Returns AST

```rust
// User API
let expr = parse("f(5)")?;
let result = solver.evaluate(&expr)?;

// result is Expression, not z3::ast::Int!
match result {
    Expression::Const(val) => {
        println!("Computed: {}", val);  // "26"
        
        // Can use in another expression
        let next_expr = Expression::Operation {
            name: "g".to_string(),
            args: vec![result],  // ← result IS an Expression!
        };
    }
    _ => {}
}
```

### Example 2: Simplification Returns AST

```rust
// User API
let expr = parse("(3 + 2) * x")?;
let simplified = solver.simplify(&expr)?;

// simplified is Expression: Operation("*", [Const("5"), Object("x")])
render(&simplified);  // Can render it
type_check(&simplified);  // Can type check it
evaluate(&simplified);  // Can evaluate further
```

### Example 3: Verification Still Returns Result Enum

```rust
// Verification doesn't need AST return
let result = solver.verify_axiom(&axiom)?;

// result is VerificationResult enum (not AST)
match result {
    VerificationResult::Valid => println!("✅ Proven"),
    VerificationResult::Invalid { counterexample } => {
        // counterexample is a String description
        println!("❌ Counterexample: {}", counterexample);
    }
    _ => {}
}
```

---

## 🎯 Solver API Design

### Public Interface (All Returns Kleis Types)

```rust
impl SolverBackend for Z3Backend {
    /// Verify axiom - returns verification result
    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String> {
        // Internal: uses Z3
        // Returns: VerificationResult (not AST)
    }
    
    /// Evaluate expression - RETURNS KLEIS AST ← CRITICAL
    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String> {
        // 1. Translate: Expression → z3::ast::Dynamic
        let z3_expr = self.translate(expr)?;
        
        // 2. Solve and get model
        let model = self.get_model()?;
        
        // 3. Extract value: z3::ast::Dynamic
        let z3_value = model.eval(&z3_expr, true)?;
        
        // 4. Convert to Kleis AST ← CONVERSION HAPPENS HERE
        self.converter.to_expression(&z3_value)
        
        // Returns: Expression::Const("26") ← Kleis AST!
    }
    
    /// Simplify expression - RETURNS KLEIS AST ← CRITICAL
    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String> {
        // Internal: uses Z3 simplification
        // Returns: Expression (simplified AST)
    }
}
```

---

## 🔄 Data Flow: Kleis AST → Solver → Kleis AST

### Complete Pipeline

```
┌─────────────────────────────────────────────────────┐
│ User provides: Expression                           │
│   parse("f(5)") → Expression::Operation(...)        │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ Solver Interface (SolverBackend trait)              │
│   fn evaluate(expr: &Expression) → Expression      │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ Internal Translation (solver-specific)              │
│   Expression → z3::ast::Dynamic                     │
│   (hidden from user)                                │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ Z3 Computation                                      │
│   model.eval(...) → z3::ast::Int                    │
│   (hidden from user)                                │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ ResultConverter (solver-specific)                   │
│   z3::ast::Int → Expression::Const("26")            │
│   (hidden from user)                                │
└────────────────────┬────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────┐
│ User receives: Expression::Const("26")              │
│   (Pure Kleis AST, no solver types!)                │
└─────────────────────────────────────────────────────┘
```

**Key: Solver-specific types NEVER escape the backend!**

---

## 📝 Updated SolverBackend Trait

### Complete Interface

```rust
/// Solver backend trait - ALL public methods return Kleis types!
pub trait SolverBackend: Send + Sync {
    // ===== Metadata =====
    
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn capabilities(&self) -> &SolverCapabilities;
    fn supports_operation(&self, op: &str) -> bool;
    
    // ===== Verification (returns enum, not AST) =====
    
    /// Verify an axiom is valid
    /// Returns: Valid/Invalid/Unknown (not a value)
    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String>;
    
    /// Check if two expressions are equivalent
    /// Returns: bool or VerificationResult
    fn are_equivalent(&mut self, e1: &Expression, e2: &Expression) -> Result<bool, String>;
    
    // ===== Evaluation (RETURNS KLEIS AST!) =====
    
    /// Evaluate expression to concrete value
    /// MUST return Kleis AST!
    /// Example: f(5) → Expression::Const("26")
    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String>;
    
    /// Evaluate with given bindings
    /// Example: evaluate(x + y, {x: 3, y: 4}) → Expression::Const("7")
    fn evaluate_with_bindings(
        &mut self,
        expr: &Expression,
        bindings: &HashMap<String, Expression>,
    ) -> Result<Expression, String>;
    
    // ===== Simplification (RETURNS KLEIS AST!) =====
    
    /// Simplify expression symbolically
    /// Example: (3 + 2) * x → 5 * x (as Kleis AST)
    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String>;
    
    /// Expand functions and simplify
    /// Example: f(x) where define f(x) = x + 1 → x + 1
    fn expand_and_simplify(&mut self, expr: &Expression) -> Result<Expression, String>;
    
    // ===== Solving (RETURNS KLEIS AST!) =====
    
    /// Find value(s) that satisfy constraints
    /// Returns: Vec<(String, Expression)> - variable bindings as AST
    fn solve(
        &mut self,
        constraints: &[Expression],
    ) -> Result<Vec<(String, Expression)>, String>;
}
```

---

## 🔍 Why This Matters

### Use Case 1: REPL

```rust
// User in REPL:
> define f(x) = x² + 1
> f(5)
26                  ← Must be displayable

> let prev = _      ← Must capture previous result
> 2 * prev
52                  ← Must work in expressions

// Implementation:
let result = solver.evaluate(&parse("f(5)"))?;
// result is Expression::Const("26") ✅
// Can store, display, use in next expression!
```

### Use Case 2: Expression Builder

```rust
// Build complex expressions programmatically
let f_of_5 = solver.evaluate(&parse("f(5)"))?;  // Expression
let g_of_result = Expression::Operation {
    name: "g".to_string(),
    args: vec![f_of_5],  // ← Needs to be Expression!
};
let final_result = solver.evaluate(&g_of_result)?;  // Expression
```

### Use Case 3: Symbolic Computation Pipeline

```rust
// Chain operations
let expr = parse("(3 + 2) * x")?;
let simplified = solver.simplify(&expr)?;      // Expression: 5 * x
let with_x_3 = substitute(&simplified, "x", &Const("3"))?;  // 5 * 3
let result = solver.evaluate(&with_x_3)?;     // Expression::Const("15")

// All returns are Expression! ✅
```

---

## 📊 Comparison

| Return Type | Pros | Cons | Verdict |
|-------------|------|------|---------|
| **Kleis AST** | ✅ Solver-independent<br>✅ Composable<br>✅ User-friendly | ⚠️ Requires conversion | ✅ **CORRECT** |
| **i64/f64** | ✅ Simple for numerics | ❌ Not composable<br>❌ Loses structure | ❌ Too limited |
| **Solver types (z3::ast::Int)** | ✅ No conversion | ❌ Leaks abstraction<br>❌ Solver-dependent | ❌ **WRONG** |
| **Custom enum** | ✅ Flexible | ⚠️ Another layer<br>⚠️ More conversion | ⚠️ Overkill |

**Winner: Kleis AST** ✅

---

## 🔧 Implementation Strategy

### Internal vs External Types

**Internal (hidden from users):**
```rust
// Inside Z3Backend
struct Z3Backend {
    solver: z3::Solver,           // Z3-specific
    converter: Z3ResultConverter, // Converts Z3 → AST
}

// Internal method
fn translate_to_z3(&self, expr: &Expression) -> z3::ast::Dynamic {
    // Z3 types used internally
}
```

**External (public API):**
```rust
// Public trait implementation
impl SolverBackend for Z3Backend {
    fn evaluate(&mut self, expr: &Expression) -> Result<Expression, String> {
        // 1. Internal: Kleis → Z3
        let z3_expr = self.translate_to_z3(expr)?;
        
        // 2. Internal: Z3 solving
        let z3_result = self.solve_internal(&z3_expr)?;
        
        // 3. Internal: Z3 → Kleis
        let kleis_expr = self.converter.to_expression(&z3_result)?;
        
        // 4. Public: Return Kleis AST
        Ok(kleis_expr)  // ← Expression, not z3::ast::Int!
    }
}
```

---

## 🎯 Conversion Examples

### Z3 Integer → Kleis Const

```rust
// Z3 returns: z3::ast::Int with value 26
let z3_int: z3::ast::Int = model.eval(&expr, true)?;

// Extract value
let value: i64 = z3_int.as_i64()?;

// Convert to Kleis AST
let kleis_expr = Expression::Const(value.to_string());

// Return: Expression::Const("26") ✅
```

### Z3 Boolean → Kleis Constructor

```rust
// Z3 returns: z3::ast::Bool with value true
let z3_bool: z3::ast::Bool = model.eval(&expr, true)?;

// Extract value
let value: bool = z3_bool.as_bool()?;

// Convert to Kleis AST (data constructor)
let kleis_expr = if value {
    Expression::Object("True".to_string())
} else {
    Expression::Object("False".to_string())
};

// Return: Expression::Object("True") ✅
```

### Z3 Symbolic → Kleis Symbolic

```rust
// Z3 returns: uninterpreted function application
// (f (+ x 1))

// Parse SMT-LIB back to Kleis
let kleis_expr = Expression::Operation {
    name: "f".to_string(),
    args: vec![
        Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        },
    ],
};

// Return: Expression (symbolic) ✅
```

---

## ✅ Benefits of AST Return

### 1. Solver Independence ✅

```rust
// User code doesn't care which solver:
fn compute_something<S: SolverBackend>(solver: &mut S) -> Expression {
    solver.evaluate(&parse("f(5)")).unwrap()
    // Works with Z3, CVC5, or any backend!
}
```

### 2. Composability ✅

```rust
// Results can be used in further expressions
let r1 = solver.evaluate(&expr1)?;  // Expression
let r2 = solver.evaluate(&expr2)?;  // Expression

let combined = Expression::Operation {
    name: "+".to_string(),
    args: vec![r1, r2],  // ← Both are Expressions!
};

let final_result = solver.evaluate(&combined)?;  // Expression
```

### 3. REPL Support ✅

```rust
// REPL needs to store history as Expressions
struct ReplState {
    history: Vec<Expression>,  // ← Not Vec<z3::ast::Int>!
}

// User can reference previous results
> f(5)
26
> 2 * _
52

// Implementation:
let result = solver.evaluate(&expr)?;  // Expression
repl.history.push(result);  // Can store it!
```

### 4. Type Safety ✅

```rust
// Kleis type checker works on Expressions
let result = solver.evaluate(&expr)?;  // Expression

// Can type check the result!
let result_type = type_checker.infer(&result)?;
println!("Result type: {:?}", result_type);
```

---

## 📋 Next Session Tasks (Updated)

### High Priority: AST Conversion

**1. Implement ResultConverter trait**
- [ ] Define trait with `to_expression()` primary method
- [ ] Implement Z3ResultConverter
- [ ] Handle: Int, Real, Bool, Symbolic
- [ ] Test all conversions

**2. Update SolverBackend trait**
- [ ] Change `evaluate()` return type to `Expression`
- [ ] Add `simplify()` returning `Expression`
- [ ] Add `solve()` returning `Vec<(String, Expression)>`
- [ ] Keep `verify_axiom()` returning `VerificationResult`

**3. Implement conversion in Z3Backend**
- [ ] Create `Z3ResultConverter`
- [ ] Use in `evaluate()` method
- [ ] Test round-trip: Expression → Z3 → Expression

**4. Test AST return values**
- [ ] Test: `evaluate("f(5)")` returns `Expression::Const("26")`
- [ ] Test: Can use result in another expression
- [ ] Test: REPL-style chaining works
- [ ] Test: Type checking result works

---

## ✅ Critical Design Decision

**DECISION:** Solver module MUST return Kleis AST, not solver-specific types!

**Rationale:**
- ✅ Maintains proper abstraction boundary
- ✅ Enables solver independence  
- ✅ Supports composability
- ✅ REPL-friendly
- ✅ Type-safe

**Implementation:**
- ResultConverter trait handles conversion
- Z3Backend hides Z3 types internally
- Public API only exposes `Expression`

**This is architecturally critical for the modularization!** 🏗️

---

**Next session: Implement MCP-style solver modularization WITH AST return requirement!** ✅

