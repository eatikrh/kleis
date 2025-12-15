# Type Checking - Next Steps After ADR-015

**Date:** December 6, 2025  
**Status:** Roadmap  
**Context:** Connects ADR-015 (text/parsing) → Type Inference POC

---

## The Journey So Far

### ✅ What We Have

1. **Type Inference Engine** (`src/type_inference.rs`)
   - Hindley-Milner type inference
   - Works on Expression AST
   - Handles basic operations

2. **Text Representation Decided** (ADR-015)
   - Text is source of truth
   - Explicit forms for operations
   - Git-friendly

3. **Kleis Parser** (`src/kleis_parser.rs`)
   - Parses expressions: `abs(x)`, `frac(a, b)`
   - Generates AST
   - Tested and working

### ❓ What's Missing

**The connection!** We need to parse **structure definitions** to build the **type context**.

---

## The Problem: Type Context Bootstrap

### Current Type Inference Works Like This

```rust
// Type inference engine (EXISTS)
let mut ctx = TypeContext::new();

// Hardcoded types (EXISTS)
ctx.register("plus", /* signature */);
ctx.register("scalar_divide", /* signature */);

// Infer! (EXISTS)
let expr = parse("x + 1");
let ty = ctx.infer(&expr);  // ✅ Works!
```

### What We Need for User-Defined Types

```kleis
// User writes in .kleis file:
structure Money {
    amount : ℝ
    currency : String
    axiom non_negative: amount ≥ 0
}

operation (+) : Money × Money → Money

// Now type inference should know about Money!
define total = myMoney + yourMoney
// Infer: total : Money ✅
```

**The missing link:** Parse structure definitions → Build type context

---

## Next Steps

### Phase 1: Parse Structure Definitions ⬜

**Extend the parser to handle:**

```kleis
structure TypeName {
    field1 : Type1
    field2 : Type2
}

operation opname : Type1 → Type2
```

**New parser rules needed:**
```rust
// Add to kleis_parser.rs
fn parse_structure_def(&mut self) -> Result<StructureDef, ParseError>
fn parse_operation_decl(&mut self) -> Result<OperationDecl, ParseError>
fn parse_type(&mut self) -> Result<Type, ParseError>
```

**Grammar from Kleis v0.3:**
```antlr
structureDef
    : 'structure' IDENTIFIER '(' typeParams ')' 
      '{' structureMember* '}'
    ;

operationDecl
    : 'operation' operatorSymbol ':' typeSignature
    ;
```

**Estimated:** 2-3 days

---

### Phase 2: Build Type Context from Parsed Structures ⬜

**Connect parser output → Type inference:**

```rust
// Parse structure definitions
let structures = parse_kleis_file("stdlib/core.kleis")?;

// Build type context
let mut ctx = TypeContext::new();
for structure in structures {
    ctx.register_structure(structure);
}

// Now type inference knows about user types!
```

**Implementation:**
```rust
// src/type_inference.rs - extend TypeContext

impl TypeContext {
    /// Load structure definition into context
    pub fn register_structure(&mut self, structure: StructureDef) {
        // Add type to context
        self.types.insert(structure.name.clone(), structure);
        
        // Register operations
        for operation in structure.operations {
            self.register_operation(operation.name, operation.signature);
        }
    }
    
    /// Load operation signature
    pub fn register_operation(&mut self, name: String, sig: TypeSignature) {
        self.operations.insert(name, sig);
    }
}
```

**Estimated:** 2-3 days

---

### Phase 3: Test with User-Defined Types ⬜

**Create test file:**

```kleis
// test_user_types.kleis

// Define user type
structure Money {
    amount : ℝ
    currency : String
}

// Define operation
operation (+) : Money × Money → Money

// Use it
define price1 : Money = Money { amount: 10.0, currency: "USD" }
define price2 : Money = Money { amount: 5.0, currency: "USD" }
define total = price1 + price2

// Type check should infer: total : Money ✅
```

**Test:**
```rust
// tests/test_user_defined_types.rs

#[test]
fn test_money_type() {
    // Parse file
    let defs = parse_kleis_file("test_user_types.kleis").unwrap();
    
    // Build context
    let mut ctx = TypeContext::from_definitions(defs);
    
    // Find the "total" expression
    let total_expr = /* ... */;
    
    // Infer type
    let ty = ctx.infer(&total_expr).unwrap();
    
    // Should be Money!
    assert_eq!(ty, Type::UserDefined("Money".to_string()));
}
```

**Estimated:** 1-2 days

---

### Phase 4: Integrate with ADR-015 Explicit Forms ⬜

**Now we can validate ADR-015's type checking promise!**

```kleis
// User writes
S: Set(ℤ) = {1, 2, 3}
n = abs(S)  // ERROR!

// Type checker (with stdlib loaded):
// 1. Parser: abs(S) → Operation { name: "abs", ... }
// 2. Type context: abs : ℝ → ℝ (from stdlib)
// 3. Type inference: S : Set(ℤ), abs expects ℝ
// 4. Error: "abs() expects Number, got Set(ℤ)"
// 5. Suggestion: "Did you mean card(S)?"
```

**Implementation:**
```rust
// src/type_checker.rs (new file)

pub struct TypeChecker {
    context: TypeContext,
}

impl TypeChecker {
    /// Check expression and generate helpful errors
    pub fn check(&mut self, expr: &Expression) -> TypeCheckResult {
        match self.context.infer(expr) {
            Ok(ty) => TypeCheckResult::Success(ty),
            Err(e) => {
                // Generate helpful error with suggestions
                self.generate_error_with_suggestion(expr, e)
            }
        }
    }
    
    fn generate_error_with_suggestion(&self, expr: &Expression, error: String) 
        -> TypeCheckResult 
    {
        // Check for common mistakes
        if let Expression::Operation { name, args } = expr {
            match name.as_str() {
                "abs" => {
                    // Check if argument is actually a Set
                    if let Some(arg_ty) = self.infer_silently(&args[0]) {
                        if matches!(arg_ty, Type::Set(_)) {
                            return TypeCheckResult::Error {
                                message: format!("abs() expects Number, got {}", arg_ty),
                                suggestion: Some("Did you mean card(S)?".to_string()),
                            };
                        }
                    }
                }
                "card" => {
                    // Check if argument is actually a Number
                    if let Some(arg_ty) = self.infer_silently(&args[0]) {
                        if matches!(arg_ty, Type::Scalar) {
                            return TypeCheckResult::Error {
                                message: format!("card() expects Set, got {}", arg_ty),
                                suggestion: Some("Did you mean abs(x)?".to_string()),
                            };
                        }
                    }
                }
                _ => {}
            }
        }
        
        TypeCheckResult::Error {
            message: error,
            suggestion: None,
        }
    }
}
```

**This proves ADR-015 Decision #3 (Explicit Forms)!**

**Estimated:** 2-3 days

---

### Phase 5: Load stdlib Automatically ⬜

**Bootstrap sequence:**

```rust
// On startup
pub fn initialize_type_system() -> TypeContext {
    let mut ctx = TypeContext::new();
    
    // Step 1: Core types (hardcoded)
    ctx.register_core_types();
    
    // Step 2: Parse stdlib
    let stdlib = parse_kleis_file("stdlib/core.kleis")?;
    for def in stdlib {
        ctx.register_definition(def);
    }
    
    // Step 3: Parse prelude
    let prelude = parse_kleis_file("stdlib/prelude.kleis")?;
    for def in prelude {
        ctx.register_definition(def);
    }
    
    // Ready!
    ctx
}
```

**stdlib/core.kleis content (from ADR-015):**
```kleis
@library("kleis.core")
@version("1.0.0")

// Absolute value
operation abs : ℝ → ℝ
axiom abs_non_negative: ∀ (x : ℝ) . abs(x) ≥ 0

// Cardinality
operation card : ∀T. Set(T) → ℕ
axiom card_empty: card(∅) = 0

// Norm
operation norm : ∀(n : ℕ). Vector(n) → ℝ
axiom norm_non_negative: ∀ (v : Vector(n)) . norm(v) ≥ 0

// Fraction (display mode)
operation frac : ℝ × ℝ → ℝ
define frac(a, b) = a / b
```

**Estimated:** 1-2 days

---

## Complete Pipeline

### What We'll Have

```
User writes:
    ↓
.kleis file (text)
    ↓
Parser (kleis_parser.rs)
    ↓
AST (Expression + StructureDef + OperationDecl)
    ↓
Type Context Builder
    ↓
Type Context (with stdlib + user types)
    ↓
Type Inference (type_inference.rs)
    ↓
Type Check Result
    ↓
Visual Feedback (with suggestions!)
```

---

## Proof-of-Concept Test

### Test: User-Defined Type with Error Detection

**Input file:** `test_type_error.kleis`

```kleis
// Step 1: Define stdlib operations (would be in stdlib/core.kleis)
operation abs : ℝ → ℝ
operation card : ∀T. Set(T) → ℕ

// Step 2: User writes code with error
define S : Set(ℤ) = {1, 2, 3}
define n = abs(S)  // ERROR!

// Step 3: User fixes it
define m = card(S)  // ✓ Correct!
```

**Expected behavior:**
```rust
// Parse file
let file = parse_kleis_file("test_type_error.kleis")?;

// Build context from operations
let mut ctx = TypeContext::new();
ctx.register_operation("abs", "ℝ → ℝ");
ctx.register_operation("card", "∀T. Set(T) → ℕ");

// Type check expressions
let abs_expr = /* abs(S) */;
let result = ctx.check(&abs_expr);

// Should produce error:
assert!(matches!(result, TypeCheckResult::Error { .. }));
assert_eq!(result.message, "abs() expects ℝ, got Set(ℤ)");
assert_eq!(result.suggestion, Some("Did you mean card(S)?"));

// Check corrected version
let card_expr = /* card(S) */;
let result2 = ctx.check(&card_expr);

// Should succeed:
assert!(matches!(result2, TypeCheckResult::Success(_)));
assert_eq!(result2.ty, Type::Nat);
```

**This validates the entire ADR-015 → Type Checking pipeline!**

---

## Implementation Timeline

| Phase | Task | Duration | Depends On |
|-------|------|----------|------------|
| 1 | Parse structure defs | 2-3 days | Kleis parser |
| 2 | Build type context | 2-3 days | Phase 1 |
| 3 | Test user types | 1-2 days | Phase 2 |
| 4 | Error suggestions | 2-3 days | Phase 3 |
| 5 | Stdlib loading | 1-2 days | Phase 4 |

**Total:** 8-13 days (~2 weeks)

---

## What ADR-015 Enables

### Before ADR-015
- ❓ How to store structure definitions?
- ❓ How to edit them?
- ❓ What's the canonical form?
- ❓ How to parse them?

### After ADR-015
- ✅ Store as text in `.kleis` files
- ✅ Edit with visual editor (generates text) or text editor
- ✅ Canonical forms defined (`abs`, `frac`, etc.)
- ✅ Parser exists (can be extended)

**Now we can build the type context!**

---

## Priority Order

### Immediate (This Week)

**1. Extend parser for structure definitions** (High Priority)

Current parser handles:
```kleis
abs(x)      // ✅ Function calls
x + y       // ✅ Operators
```

Need to add:
```kleis
structure Money { ... }     // ⬜ Structure definitions
operation (+) : T → T → T   // ⬜ Operation declarations  
define x : ℝ = 5           // ⬜ Type annotations
```

**Start here:** This unblocks everything else.

---

### Next (Next Week)

**2. Connect parser → type context**

```rust
// Parse
let defs = parse_kleis_file("stdlib/core.kleis")?;

// Build context
let ctx = TypeContext::from_definitions(defs);

// Now type inference can use stdlib types!
```

---

### Then (Week After)

**3. Implement ADR-015 error messages**

```kleis
S: Set(ℤ) = {1, 2, 3}
n = abs(S)  // Type error with helpful message!
```

This validates ADR-015's promise of better error messages with explicit forms.

---

## Detailed Task Breakdown

### Task 1: Extend Parser for Structures (2-3 days)

**File:** `src/kleis_parser.rs`

**Add support for:**

```rust
// Structure definitions
pub struct StructureDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<StructureMember>,
}

pub enum StructureMember {
    Field { name: String, ty: TypeExpr },
    Operation { name: String, signature: TypeExpr },
    Axiom { name: String, proposition: Expression },
}

// Top-level items
pub enum TopLevel {
    StructureDef(StructureDef),
    OperationDecl { name: String, signature: TypeExpr },
    FunctionDef { name: String, params: Vec<String>, body: Expression },
}

// Type expressions
pub enum TypeExpr {
    Named(String),                  // ℝ, Money
    Parametric(String, Vec<TypeExpr>), // Vector(3), Set(ℤ)
    Function(Box<TypeExpr>, Box<TypeExpr>), // ℝ → ℝ
}
```

**Grammar to implement:**
```
program := top_level*
top_level := structure_def | operation_decl | function_def
structure_def := 'structure' identifier '{' member* '}'
member := field_decl | operation_decl | axiom_decl
field_decl := identifier ':' type
operation_decl := 'operation' identifier ':' type
type := identifier | identifier '(' type_list ')' | type '→' type
```

---

### Task 2: Create Type Context Builder (2-3 days)

**File:** `src/type_context.rs` (new file)

```rust
use crate::kleis_parser::{TopLevel, StructureDef, OperationDecl};
use crate::type_inference::{TypeContext, Type};

pub struct TypeContextBuilder {
    context: TypeContext,
}

impl TypeContextBuilder {
    pub fn new() -> Self {
        let mut ctx = TypeContext::new();
        
        // Register core types (hardcoded)
        ctx.register_type("ℝ", Type::Scalar);
        ctx.register_type("ℂ", Type::Complex);
        ctx.register_type("ℤ", Type::Integer);
        ctx.register_type("ℕ", Type::Nat);
        
        TypeContextBuilder { context: ctx }
    }
    
    /// Load definitions from parsed file
    pub fn load_definitions(&mut self, defs: Vec<TopLevel>) -> Result<(), String> {
        for def in defs {
            match def {
                TopLevel::StructureDef(s) => self.add_structure(s)?,
                TopLevel::OperationDecl { name, signature } => {
                    self.add_operation(name, signature)?
                }
                TopLevel::FunctionDef { .. } => {
                    // Functions are typed via inference
                }
            }
        }
        Ok(())
    }
    
    fn add_structure(&mut self, structure: StructureDef) -> Result<(), String> {
        // Register the type
        self.context.register_type(&structure.name, Type::UserDefined(structure.name.clone()));
        
        // Register operations
        for member in structure.members {
            match member {
                StructureMember::Operation { name, signature } => {
                    self.context.register_operation(&name, signature)?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    pub fn build(self) -> TypeContext {
        self.context
    }
}
```

**Usage:**
```rust
// Build type context
let mut builder = TypeContextBuilder::new();
builder.load_stdlib("stdlib/core.kleis")?;
builder.load_file("user_code.kleis")?;
let ctx = builder.build();

// Now ready for type checking!
```

---

### Task 3: Create stdlib/core.kleis (1 day)

**File:** `stdlib/core.kleis`

From ADR-015, we need:

```kleis
@library("kleis.core")
@version("1.0.0")

// ============================================
// NUMERICAL OPERATIONS
// ============================================

operation abs : ℝ → ℝ
axiom abs_non_negative: ∀ (x : ℝ) . abs(x) ≥ 0
axiom abs_symmetric: ∀ (x : ℝ) . abs(-x) = abs(x)

// ============================================
// SET OPERATIONS
// ============================================

operation card : ∀T. Set(T) → ℕ
axiom card_empty: card(∅) = 0

// ============================================
// VECTOR OPERATIONS
// ============================================

operation norm : ∀(n : ℕ). Vector(n) → ℝ
axiom norm_non_negative: ∀ (v : Vector(n)) . norm(v) ≥ 0

// ============================================
// DISPLAY MODE OPERATIONS
// ============================================

operation frac : ℝ × ℝ → ℝ
// Note: Semantically identical to (/), signals display mode
```

---

### Task 4: Integration Test (1-2 days)

**File:** `tests/test_type_checking_with_stdlib.rs`

```rust
#[test]
fn test_adr015_error_detection() {
    // Load stdlib
    let stdlib_defs = parse_kleis_file("stdlib/core.kleis").unwrap();
    
    // Build context
    let mut builder = TypeContextBuilder::new();
    builder.load_definitions(stdlib_defs).unwrap();
    let mut ctx = builder.build();
    
    // Parse user code with error
    let user_code = parse_kleis("abs(S)").unwrap();
    
    // Add variable S to context
    ctx.bind("S", Type::Set(Box::new(Type::Integer)));
    
    // Type check
    let result = ctx.infer(&user_code);
    
    // Should fail with helpful message
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("abs() expects ℝ, got Set(ℤ)"));
}

#[test]
fn test_adr015_correction() {
    // Same setup
    let mut ctx = /* ... */;
    ctx.bind("S", Type::Set(Box::new(Type::Integer)));
    
    // Parse corrected code
    let corrected = parse_kleis("card(S)").unwrap();
    
    // Should succeed
    let result = ctx.infer(&corrected);
    assert_eq!(result.unwrap(), Type::Nat);
}
```

---

## Summary

### The Connection

```
Previous Work: Type Inference POC
    ↓
    Missing: How to define user types?
    ↓
ADR-015: Text representation decided
    ↓
Kleis Parser: Can parse expressions
    ↓
Next Step: Parse structure definitions
    ↓
Build Type Context from structures
    ↓
Type Checking with stdlib + user types!
```

### Next Actions (Priority Order)

1. **Extend parser** for structure definitions (2-3 days)
2. **Create type context builder** (2-3 days)
3. **Write stdlib/core.kleis** (1 day)
4. **Test with user types** (1-2 days)
5. **Integrate error messages** (2-3 days)

**Total:** ~2 weeks to complete type checking POC with user-defined types

---

## Success Criteria

When complete, we should be able to:

✅ Define structures in `.kleis` files  
✅ Parser reads and understands them  
✅ Type context loads stdlib automatically  
✅ Type inference works with user types  
✅ Error messages suggest corrections (ADR-015 validated!)  
✅ Visual editor → text → parse → type check pipeline works  

**This completes the full vision!**

---

**Status:** 🎯 **Roadmap Complete - Ready to Implement**  
**Start with:** Task 1 (Extend parser for structures)  
**Goal:** Complete type checking POC with user-defined types

