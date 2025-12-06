# Complete Type Checking Roadmap

**Date:** December 6, 2024  
**Status:** Connecting ADR-015 + Type Inference → Full Type Checking

---

## The Big Picture

```
┌─────────────────────────────────────────────────────────────────┐
│                     KLEIS TYPE CHECKING                         │
│                         FULL VISION                             │
└─────────────────────────────────────────────────────────────────┘

Phase 1: Design & Basic Inference (DONE ✅)
├── Type inference engine exists (src/type_inference.rs)
├── Basic operations work (plus, minus, etc.)
└── POC tested and validated

Phase 2: Text Representation (DONE ✅)
├── ADR-015: Text as source of truth
├── Explicit forms decided (abs, card, norm, frac)
└── Git-friendly diffs

Phase 3: Expression Parser (DONE ✅)
├── Kleis parser created (src/kleis_parser.rs)
├── Parses: abs(x), frac(a,b), a + b
└── All tests passing (15 tests)

Phase 4: Structure Parser (NEXT ⬜)
├── Parse: structure Money { ... }
├── Parse: operation (+) : T → T
└── Parse: Type annotations

Phase 5: Type Context Builder (NEXT ⬜)
├── Load parsed structures
├── Register types and operations
└── Build complete context

Phase 6: stdlib Integration (NEXT ⬜)
├── stdlib/core.kleis with abs, card, norm, frac
├── Auto-load on startup
└── Type context ready

Phase 7: Full Type Checking (GOAL 🎯)
├── Parse user code + structures
├── Build context (core + stdlib + user)
├── Type check with helpful errors
└── ADR-015 validated end-to-end!
```

---

## Where We Are

### ✅ Already Complete (80% of foundation)

1. **Type Inference Engine** (`src/type_inference.rs`)
   - Hindley-Milner algorithm
   - Polymorphism support
   - Constraint solving

2. **Expression AST** (`src/ast.rs`)
   - Clean structure
   - Works with parser and renderer

3. **Expression Parser** (`src/kleis_parser.rs`)
   - Parses function calls
   - Handles operators
   - Correct precedence

4. **Design Decisions** (ADR-015)
   - Text representation
   - Explicit forms
   - Display modes

### ⬜ What's Missing (20% - the glue)

1. **Parse structure definitions**
   - `structure TypeName { ... }`
   - `operation name : signature`

2. **Build type context from parsed structures**
   - Load stdlib
   - Register user types
   - Build operation registry

3. **Error generation with suggestions**
   - "abs() expects Number, got Set"
   - "Did you mean card()?"

---

## Next Steps (Prioritized)

### 🎯 Step 1: Extend Parser for Structures (START HERE)

**Goal:** Parse this:

```kleis
structure Money {
    amount : ℝ
    currency : String
}

operation (+) : Money × Money → Money
```

**Implementation:**

Add to `src/kleis_parser.rs`:

```rust
// New AST nodes
pub struct StructureDef {
    pub name: String,
    pub members: Vec<StructureMember>,
}

pub enum StructureMember {
    Field { name: String, type_expr: TypeExpr },
    Operation { name: String, type_signature: TypeExpr },
}

pub enum TypeExpr {
    Named(String),                          // ℝ, Money
    Param(String, Vec<TypeExpr>),           // Vector(3)
    Function(Box<TypeExpr>, Box<TypeExpr>), // ℝ → ℝ
}

// New parser methods
impl KleisParser {
    pub fn parse_program(&mut self) -> Result<Vec<TopLevel>, ParseError> {
        // Parse multiple top-level items
    }
    
    fn parse_structure(&mut self) -> Result<StructureDef, ParseError> {
        // structure Name { members }
    }
    
    fn parse_operation_decl(&mut self) -> Result<OperationDecl, ParseError> {
        // operation name : Type
    }
    
    fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        // ℝ, Vector(n), ℝ → ℝ
    }
}
```

**Test:**
```rust
#[test]
fn test_parse_structure() {
    let code = "structure Money { amount : ℝ }";
    let result = parse_kleis_program(code).unwrap();
    // Should have 1 structure definition
}
```

**Duration:** 2-3 days

---

### 🎯 Step 2: Create Type Context Builder (NEXT)

**Goal:** Build type context from parsed structures

**File:** `src/type_context.rs` (new)

```rust
pub struct TypeContextBuilder {
    context: TypeContext,
}

impl TypeContextBuilder {
    pub fn from_definitions(defs: Vec<TopLevel>) -> Result<TypeContext, String> {
        let mut builder = Self::new();
        
        for def in defs {
            match def {
                TopLevel::StructureDef(s) => builder.add_structure(s)?,
                TopLevel::OperationDecl(op) => builder.add_operation(op)?,
                _ => {}
            }
        }
        
        Ok(builder.context)
    }
}
```

**Test:**
```rust
#[test]
fn test_context_from_structures() {
    let code = r#"
        structure Money {
            amount : ℝ
        }
        operation (+) : Money × Money → Money
    "#;
    
    let defs = parse_kleis_program(code).unwrap();
    let ctx = TypeContextBuilder::from_definitions(defs).unwrap();
    
    // Context should know about Money and (+)
    assert!(ctx.has_type("Money"));
    assert!(ctx.has_operation("+"));
}
```

**Duration:** 2-3 days

---

### 🎯 Step 3: Create stdlib/core.kleis (EASY)

**Goal:** Define standard operations per ADR-015

**File:** `stdlib/core.kleis`

```kleis
@library("kleis.core")
@version("1.0.0")

operation abs : ℝ → ℝ
operation card : ∀T. Set(T) → ℕ
operation norm : ∀(n : ℕ). Vector(n) → ℝ
operation frac : ℝ × ℝ → ℝ

operation (+) : ℝ × ℝ → ℝ
operation (-) : ℝ × ℝ → ℝ
operation (×) : ℝ × ℝ → ℝ
operation (/) : ℝ × ℝ → ℝ

// Add more as needed
```

**Duration:** 1 day (mostly documentation)

---

### 🎯 Step 4: Test User-Defined Types (VALIDATION)

**Goal:** Prove the whole pipeline works

**Test file:** `tests/test_full_type_checking.rs`

```rust
#[test]
fn test_user_defined_type_with_error_detection() {
    // Step 1: Parse stdlib
    let stdlib = parse_kleis_file("stdlib/core.kleis").unwrap();
    
    // Step 2: Build context
    let mut ctx = TypeContextBuilder::from_definitions(stdlib).unwrap();
    
    // Step 3: Parse user code with TYPE ERROR
    let code = r#"
        S : Set(ℤ) = {1, 2, 3}
        n = abs(S)
    "#;
    let user_defs = parse_kleis_program(code).unwrap();
    
    // Step 4: Add user variables to context
    ctx.bind("S", Type::Set(Box::new(Type::Integer)));
    
    // Step 5: Type check the abs(S) expression
    let abs_expr = /* extract from user_defs */;
    let result = ctx.check(&abs_expr);
    
    // Step 6: Verify error message
    assert!(result.is_err());
    assert_eq!(result.error_message(), "abs() expects ℝ, got Set(ℤ)");
    assert_eq!(result.suggestion(), Some("Did you mean card(S)?"));
    
    // Step 7: Test correction
    let corrected = parse_kleis("card(S)").unwrap();
    let result2 = ctx.check(&corrected);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Type::Nat);
}
```

**This validates the ENTIRE pipeline!** 🎉

**Duration:** 1-2 days

---

### 🎯 Step 5: Error Suggestions (ADR-015 Validation)

**Goal:** Implement helpful error messages

**File:** `src/type_checker.rs` (new)

```rust
pub fn check_with_suggestions(
    ctx: &TypeContext,
    expr: &Expression
) -> TypeCheckResult {
    match ctx.infer(expr) {
        Ok(ty) => TypeCheckResult::Ok(ty),
        Err(e) => {
            // Generate suggestion based on expression
            let suggestion = generate_suggestion(expr, &e, ctx);
            TypeCheckResult::Error { message: e, suggestion }
        }
    }
}

fn generate_suggestion(
    expr: &Expression,
    error: &str,
    ctx: &TypeContext
) -> Option<String> {
    if let Expression::Operation { name, args } = expr {
        match name.as_str() {
            "abs" => {
                // If argument is Set, suggest card
                if is_set_type(&args[0], ctx) {
                    return Some("Did you mean card(S)?".to_string());
                }
            }
            "card" => {
                // If argument is Number, suggest abs
                if is_numeric_type(&args[0], ctx) {
                    return Some("Did you mean abs(x)?".to_string());
                }
            }
            _ => {}
        }
    }
    None
}
```

**Duration:** 2-3 days

---

## Timeline

```
Week 1:
├── Mon-Wed: Extend parser for structures (3 days)
└── Thu-Fri: Create type context builder (2 days)

Week 2:
├── Mon: Write stdlib/core.kleis (1 day)
├── Tue-Wed: Integration tests (2 days)
└── Thu-Fri: Error suggestions (2 days)

Result: COMPLETE TYPE CHECKING POC WITH USER TYPES! ✅
```

**Total: 10-13 days (2 weeks)**

---

## Success Criteria

When complete:

✅ Can write structures in `.kleis` files  
✅ Parser reads and understands them  
✅ Type context loads automatically  
✅ Type inference works with user types  
✅ Error messages have suggestions  
✅ ADR-015 validated end-to-end  
✅ `abs(Set)` → helpful error  
✅ `card(Number)` → helpful error  
✅ User types work same as built-in  

---

## Run This Demo (Future)

```bash
# After implementation
cargo run --example type_checking_full_demo

# Expected output:
✅ Parsed stdlib/core.kleis (4 operations)
✅ Built type context (8 types, 12 operations)
✅ Type checking: abs(x) where x : ℝ → ℝ ✓
❌ Type error: abs(S) where S : Set(ℤ)
   Error: abs() expects ℝ, got Set(ℤ)
   Suggestion: Did you mean card(S)?
✅ Type checking: card(S) where S : Set(ℤ) → ℕ ✓
```

---

**Status:** 🎯 **Clear Path Forward**  
**Next:** Implement Phase 4 (Structure Parser)  
**Timeline:** 2 weeks to complete

