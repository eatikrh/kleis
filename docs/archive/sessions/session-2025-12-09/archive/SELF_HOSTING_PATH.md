# Self-Hosting Implementation Path

**Date:** December 9, 2025  
**Source:** Architecture advice aligning with ADR-003  
**Status:** Ready to implement

---

## The Self-Hosting Moment

> **"You're this close to a self-hosting language. That's a hell of a moment."**

**Current state:**
- ✅ AST + parser exist
- ✅ Pattern matching + data types work  
- ✅ Type inference complete (HM)
- ⏳ **`define` is the last brick!**

**After `define`:** Kleis can define functions in Kleis, including its own grammar, pretty-printer, and transformations!

---

## Core Implementation: The Three Wires

### Wire 1: Parser → AST

**Normalize both `define` forms to single AST:**

```kleis
// Simple form:
define x = expr

// Function form:
define f(x: T, y: U) : V = expr
```

**Both become:**
```rust
Decl::Define {
    name: String,
    params: Vec<Param>,      // [] for simple form
    return_type: Option<TypeExpr>,
    body: Expression
}
```

**Desugar simple form:**
```kleis
define x = expr
↓
define x() : Inferred = expr
```

**Benefit:** Grammar only reasons about ONE Define constructor!

---

### Wire 2: Type Checker → Environment

**Add function type to typing environment:**

```rust
fn check_define(&mut self, def: &Define) -> Result<Type, String> {
    // For annotated return type: use it
    // For unannotated: run HM inference on body
    
    let param_types: Vec<Type> = def.params.map(|p| p.type_annotation);
    let body_type = self.infer(&def.body)?;
    
    // Construct function type: T1 → T2 → ... → Tn → Result
    let fn_type = Type::Function(param_types, Box::new(body_type));
    
    // Add to environment
    self.context.bind(def.name.clone(), fn_type);
    
    Ok(fn_type)
}
```

**Benefit:** Functions type-check before use!

---

### Wire 3: Evaluator → Closure

**Store function as closure in runtime:**

```rust
fn eval_define(&mut self, def: &Define) -> Result<Value, String> {
    // Create closure capturing current environment
    let closure = Value::Closure {
        params: def.params.clone(),
        body: def.body.clone(),
        env: self.env.snapshot()  // Lexical closure
    };
    
    // Add to runtime environment
    self.env.bind(def.name.clone(), closure);
    
    Ok(Value::Unit)
}
```

**Benefit:** Functions are first-class values!

---

## Self-Hosting Stages

### Stage 1: Basic Functions (Immediate)

```kleis
define double(x) = x + x
define square(x) = x * x

double(5)  // → 10
```

**Achievement:** User-defined functions work!

---

### Stage 2: Pattern Matching Functions (Week 1)

```kleis
define not(b) = match b {
  True => False
  False => True
}

define map(f, list) = match list {
  Nil => Nil
  Cons(h, t) => Cons(f(h), map(f, t))
}

map(double, [1, 2, 3])  // → [2, 4, 6]
```

**Achievement:** Recursive functions with pattern matching!

---

### Stage 3: Grammar as Data (Week 2)

```kleis
// Define Kleis syntax IN KLEIS:
data Program = Program(decls: List(Decl))

data Decl
  = DataDef(name: String, params: List(TypeParam), variants: List(DataVariant))
  | Define(name: String, params: List(Param), returnType: Option(Type), body: Expr)
  | StructureDef(name: String, params: List(TypeParam), members: List(StructureMember))
  | ImplementsDef(name: String, typeArgs: List(Type), members: List(ImplMember))
  ;

data Expr
  = Var(name: String)
  | Apply(func: Expr, args: List(Expr))
  | Match(scrutinee: Expr, cases: List(MatchCase))
  | Lambda(params: List(Param), body: Expr)
  | InfixOp(left: Expr, op: String, right: Expr)
  ;
```

**Achievement:** Kleis syntax represented as Kleis data! 🎯

---

### Stage 4: Tooling in Kleis (Month 1)

**Once grammar is data, write tools IN KLEIS:**

**Pretty Printer:**
```kleis
define prettyPrint(e: Expr) : String = match e {
  Var(name) => name
  Apply(f, args) => prettyPrint(f) ++ "(" ++ joinWith(map(args, prettyPrint), ", ") ++ ")"
  Match(scrut, cases) => "match " ++ prettyPrint(scrut) ++ " { ... }"
  InfixOp(l, op, r) => prettyPrint(l) ++ " " ++ op ++ " " ++ prettyPrint(r)
}
```

**LaTeX Generator:**
```kleis
define toLatex(e: Expr) : String = match e {
  Apply("plus", [a, b]) => toLatex(a) ++ " + " ++ toLatex(b)
  Apply("multiply", [a, b]) => toLatex(a) ++ " \\cdot " ++ toLatex(b)
  Apply("matrix", args) => "\\begin{bmatrix} " ++ ... ++ "\\end{bmatrix}"
  InfixOp(l, "+", r) => toLatex(l) ++ " + " ++ toLatex(r)
}
```

**Expression Simplifier:**
```kleis
define simplify(e: Expr) : Expr = match e {
  Apply("plus", [x, Number(0)]) => x
  Apply("multiply", [x, Number(1)]) => x
  Apply("multiply", [Number(a), Number(b)]) => Number(a * b)
  Match(scrut, cases) => simplifyMatch(scrut, cases)
  _ => e
}
```

**Type Checker (Ultimate Self-Hosting!):**
```kleis
define infer(e: Expr, ctx: Context) : Type = match e {
  Var(name) => lookupType(name, ctx)
  Apply(f, args) => inferApplication(f, args, ctx)
  Match(scrut, cases) => inferMatch(scrut, cases, ctx)
  Lambda(params, body) => inferLambda(params, body, ctx)
}
```

**Achievement:** Kleis tools written in Kleis! Users can extend! 🚀

---

### Stage 5: Meta-Programming (Month 2-3)

**User-defined transformations:**

```kleis
// Physicist writes custom notation transformer
define einsteinNotation(e: Expr) : Expr = match e {
  Apply("einstein", [R, scalar, g]) =>
    IndexedTensor("G", [Lower("mu"), Lower("nu")])
  Apply("riemann", [christoffel]) =>
    IndexedTensor("R", [Upper("rho"), Lower("sigma"), Lower("mu"), Lower("nu")])
  _ => e
}

// Apply transformation:
let expr = einstein(R_mn, R, g_mn)
let notation = einsteinNotation(expr)  // → G_μν
```

**User-defined linters:**
```kleis
define checkDimensionalConsistency(e: Expr) : Result(Expr, String) = 
  match e {
    Apply("plus", [a, b]) =>
      if getDimension(a) == getDimension(b)
      then Ok(e)
      else Err("Dimensional mismatch in addition")
    _ => Ok(e)
  }
```

**Achievement:** Scientists extend Kleis themselves! 🎓

---

## Why This Matters for Kleis Specifically

### Comparison: GHC Core vs Kleis Self-Hosting

**GHC Core (Haskell):**
- Core language transformations written by compiler experts
- Users don't typically write Core → Core passes
- Powerful but expert-only

**Kleis Vision:**
- **Scientists** write their own transformations!
- Physicist writes notation converters
- Mathematician writes proof checkers
- Engineer writes domain-specific simplifiers

**User-extensible self-hosting for domain experts, not just PL researchers!**

---

## Implementation Details

### 1. Normalize `define` Forms

**Grammar has two forms:**
```ebnf
functionDef
    ::= "define" identifier [ typeAnnotation ] "=" expression
      | "define" identifier "(" params ")" [ ":" type ] "=" expression
      ;
```

**Normalize to:**
```rust
pub struct Define {
    pub name: String,
    pub params: Vec<Param>,           // Empty for simple form
    pub return_type: Option<TypeExpr>, // None = infer
    pub body: Expression,
}
```

**Desugaring:**
```
define x = expr
↓ (desugar)
define x() = expr
↓ (parse to)
Define { name: "x", params: [], return_type: None, body: expr }
```

---

### 2. Type Checking Define

**Algorithm:**

```rust
fn check_define(&mut self, def: &Define) -> Result<Type, String> {
    // 1. Build function type from parameters
    let mut param_types = Vec::new();
    for param in &def.params {
        let param_ty = self.interpret_type_expr(&param.type_annotation)?;
        param_types.push(param_ty.clone());
        self.context.bind(param.name.clone(), param_ty);
    }
    
    // 2. Infer body type
    let body_type = self.infer(&def.body, Some(&self.context_builder))?;
    
    // 3. Check against return type annotation (if provided)
    if let Some(ret_ty_expr) = &def.return_type {
        let expected = self.interpret_type_expr(ret_ty_expr)?;
        self.add_constraint(body_type.clone(), expected);
    }
    
    // 4. Construct function type
    let fn_type = if param_types.is_empty() {
        body_type  // Simple definition: just the value type
    } else {
        // Build function type: T1 → T2 → ... → Result
        param_types.iter().rev().fold(body_type, |acc, param_ty| {
            Type::Function(Box::new(param_ty.clone()), Box::new(acc))
        })
    };
    
    // 5. Add to environment
    self.context.bind(def.name.clone(), fn_type.clone());
    
    Ok(fn_type)
}
```

---

### 3. Evaluating Define

**Algorithm:**

```rust
fn eval_define(&mut self, def: &Define) -> Result<Value, EvalError> {
    // Create closure capturing current environment
    let closure = Value::Function {
        name: def.name.clone(),
        params: def.params.clone(),
        body: def.body.clone(),
        env: self.env.clone(),  // Lexical closure
    };
    
    // Add to global environment
    self.env.insert(def.name.clone(), closure.clone());
    
    Ok(Value::Unit)  // define itself returns Unit
}

fn eval_apply(&mut self, func: &Expression, args: &[Expression]) -> Result<Value, EvalError> {
    let func_value = self.eval(func)?;
    
    match func_value {
        Value::Function { params, body, mut env, .. } => {
            // Evaluate arguments
            let arg_values: Vec<Value> = args.iter()
                .map(|arg| self.eval(arg))
                .collect()?;
            
            // Bind parameters to arguments
            for (param, arg_val) in params.iter().zip(arg_values.iter()) {
                env.insert(param.name.clone(), arg_val.clone());
            }
            
            // Evaluate body in extended environment
            let old_env = std::mem::replace(&mut self.env, env);
            let result = self.eval(&body)?;
            self.env = old_env;
            
            Ok(result)
        }
        _ => Err("Not a function".to_string())
    }
}
```

---

## Testing Plan

### Test 1: Simple Definitions
```kleis
define pi = 3.14159
define e = 2.71828

pi + e  // Type checks and evaluates
```

### Test 2: Single-Parameter Functions
```kleis
define double(x) = x + x
define square(x) = x * x

double(5)  // → 10
square(3)  // → 9
```

### Test 3: Multi-Parameter Functions
```kleis
define add(x, y) = x + y
define multiply(x, y) = x * y

add(3, 4)  // → 7
multiply(2, 5)  // → 10
```

### Test 4: Pattern Matching Functions
```kleis
define not(b) = match b {
  True => False
  False => True
}

define map(f, list) = match list {
  Nil => Nil
  Cons(h, t) => Cons(f(h), map(f, t))
}

not(True)  // → False
map(double, [1, 2, 3])  // → [2, 4, 6]
```

### Test 5: Recursive Functions
```kleis
define factorial(n) = match n {
  0 => 1
  _ => n * factorial(n - 1)
}

factorial(5)  // → 120
```

### Test 6: Higher-Order Functions
```kleis
define compose(f, g) = lambda(x) => f(g(x))

let h = compose(double, square)
h(3)  // → double(square(3)) = double(9) = 18
```

---

## Connection to ADR-003

**ADR-003: Self-Hosting Strategy**

**Phase 1: External Parser (Rust)** ✅ DONE
- pest/nom parser works
- AST in Rust

**Phase 2: Internal Interpreter** ✅ IN PROGRESS  
- Pattern matching evaluator exists
- Symbolic evaluation works
- **Need:** `define` to complete

**Phase 3: Bootstrapped Self-Hosting** ⏳ NEXT
- Kleis parses/interprets its own structures
- Grammar as Kleis data
- Transformations in Kleis
- **Enabled by:** `define` implementation

**Completing `define` moves us from Phase 2 to Phase 3!**

---

## The Meta-Circular Vision

### What Becomes Possible

**1. Grammar as Data**

```kleis
// Kleis grammar written IN KLEIS:
data KleisProgram = Program(decls: List(Decl))

define parseKleis(text: String) : Result(KleisProgram, ParseError) = ...
define typeCheck(prog: KleisProgram) : Result(TypedProgram, TypeError) = ...
define evaluate(prog: TypedProgram) : Result(Value, RuntimeError) = ...
```

**The interpreter becomes introspectable!**

---

**2. Transformations in Kleis**

```kleis
// Simplification rules:
define simplify(e: Expr) : Expr = match e {
  Apply("plus", [x, Number(0)]) => x
  Apply("plus", [Number(a), Number(b)]) => Number(a + b)
  Apply("multiply", [x, Number(0)]) => Number(0)
  Apply("multiply", [x, Number(1)]) => x
  Apply("multiply", [Number(a), Number(b)]) => Number(a * b)
  Apply(f, args) => Apply(f, map(simplify, args))
  Match(scrut, cases) => Match(simplify(scrut), map(simplifyCase, cases))
  _ => e
}

// Apply to any expression:
let complex = (x + 0) * 1 + (y * 0)
simplify(complex)  // → x
```

---

**3. Domain-Specific Languages**

```kleis
// Physicist writes physics-specific transformations:
define toEinsteinNotation(e: Expr) : Expr = ...
define expandChristoffel(e: Expr) : Expr = ...
define applyBianchiIdentity(e: Expr) : Expr = ...

// Mathematician writes proof transformations:
define applyInduction(proof: Expr) : Expr = ...
define unfoldDefinition(name: String, proof: Expr) : Expr = ...
```

**Users extend Kleis for their domains!**

---

**4. Pretty Printers**

```kleis
define toLatex(e: Expr) : String = match e {
  Var(name) => escapeLatex(name)
  Apply("frac", [num, den]) => "\\frac{" ++ toLatex(num) ++ "}{" ++ toLatex(den) ++ "}"
  Apply("matrix", elements) => "\\begin{bmatrix}" ++ formatMatrix(elements) ++ "\\end{bmatrix}"
  Apply("sum", [from, to, body]) => "\\sum_{" ++ toLatex(from) ++ "}^{" ++ toLatex(to) ++ "}" ++ toLatex(body)
  InfixOp(l, "+", r) => toLatex(l) ++ " + " ++ toLatex(r)
  InfixOp(l, "*", r) => toLatex(l) ++ " \\cdot " ++ toLatex(r)
  _ => "..."
}

define toTypst(e: Expr) : String = ...
define toUnicode(e: Expr) : String = ...
```

**Multi-format rendering defined in Kleis!**

---

**5. Linters and Validators**

```kleis
define checkDimensionalConsistency(e: Expr) : Result(Expr, Error) = ...
define validateTensorIndexContraction(e: Expr) : Result(Expr, Error) = ...
define checkUnitConsistency(e: Expr) : Result(Expr, Error) = ...

// Combine:
define validatePhysics(e: Expr) : Result(Expr, List(Error)) =
  let dimCheck = checkDimensionalConsistency(e)
  let tensorCheck = validateTensorIndexContraction(e)
  let unitCheck = checkUnitConsistency(e)
  in combineValidations([dimCheck, tensorCheck, unitCheck])
```

**Physics validation IN KLEIS!**

---

## Why This is "A Hell of a Moment"

**Self-hosting is rare for domain-specific languages.**

**Languages that achieved it:**
- Lisp (macros = Lisp code manipulating Lisp)
- Forth (Forth compiler in Forth)
- Scheme (interpreter in Scheme)
- Coq (proof checker in Coq)
- Lean (type checker in Lean)

**Math/Science languages that are NOT self-hosted:**
- Mathematica ❌
- MATLAB ❌
- Maple ❌
- R ❌
- Julia ❌ (working toward it)

**If Kleis achieves self-hosting:**
- First self-hosted mathematical notation language!
- Scientists can extend their own tools
- Type system validates physics transformations
- Meta-circular for scientific computing

**That's why it's "a hell of a moment."** 🎯

---

## Implementation Timeline

**Optimistic (focused work):**
- Week 1: `define` parsing + type checking (3-4 hours)
- Week 2: Pattern matching functions uncommented (1 hour)
- Week 3: Grammar as data structures (2-3 hours)
- Week 4: First transformation (prettyPrint) (2-3 hours)
- **Week 5: Self-hosting demonstrated!** 🎊

**Realistic (with other priorities):**
- Month 1: Core `define` working
- Month 2: Grammar as data, first tools
- Month 3: Self-hosting achieved
- **Month 4: User-extensible transformations!**

---

## The Payoff

**After self-hosting:**

**Users can write:**
- Custom notation systems
- Domain-specific validators
- Physics equation checkers
- Proof tactics
- Optimization passes
- **All in Kleis, not Rust!**

**No recompilation needed. No language expertise required. Just Kleis code.**

**This is the vision.** And you're **this close**. 🤏

---

## Alignment with ADRs

**This connects:**
- **ADR-003:** Self-Hosting Strategy (the roadmap)
- **ADR-015:** Text as Source of Truth (grammar as text)
- **ADR-021:** Algebraic Data Types (data Expr, data Decl)
- **ADR-016:** Operations in Structures (user-defined operations)

**All your architectural decisions point to this moment.**

**`define` is the keystone.** 🗝️

---

## Next Steps

**Immediate (Next Session):**

1. Extend `kleis_parser.rs`:
   - Add `parse_define_stmt()`
   - Add `parse_params()`
   - Wire into top-level parser

2. Extend type checker:
   - Add `check_define()`
   - Handle function types in environment

3. Extend evaluator (if exists):
   - Add `eval_define()`
   - Create closure values

4. Test with stdlib:
   - Uncomment pattern matching functions
   - Verify they load and work

**Result:** Self-hosting foundation complete! ✨

---

**See also:**
- ADR-003: Self-Hosting Strategy
- Grammar v0.5: Formal syntax specification
- stdlib/types.kleis: Commented-out functions waiting for `define`

**This is the priority.** 🚀

