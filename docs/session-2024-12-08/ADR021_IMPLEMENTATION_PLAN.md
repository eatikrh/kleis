# ADR-021 Implementation Plan: Algebraic Data Types

**Date:** December 8, 2024  
**Status:** 📋 READY TO IMPLEMENT  
**Estimated Time:** 1-2 weeks  
**Complexity:** HIGH (500+ lines, 4 files, fundamental refactoring)

---

## Executive Summary

**Goal:** Replace hardcoded `Type` enum with dynamic type system that reads `data` definitions from Kleis code.

**Why Now:**
1. ✅ ADR-016 complete (operations in structures)
2. ✅ Generic validation implemented (structure checks work)
3. ✅ Type/value distinction understood (ADR-020)
4. ✅ Matrix issue needs proper data constructors
5. ✅ Critical for user-defined types

**After this:** Matrix(2, 3, ...) becomes just another data constructor. No special cases!

---

## Current State (Checkpoint)

**Git:** Clean working tree, all committed  
**Tests:** 288 passing (281 lib + 7 validation)  
**Tag:** Will create `v0.6.0-adr016-complete` before starting  
**Branch:** main, ready to push

---

## The Transformation

### **Before (Hardcoded):**

```rust
// src/type_inference.rs - HARDCODED
pub enum Type {
    Scalar,                    // ← Can't be changed
    Vector(usize),             // ← Fixed at compile time
    Matrix(usize, usize),      // ← Requires recompilation to add types
    Var(TypeVar),
    Function(Box<Type>, Box<Type>),
    ForAll(TypeVar, Box<Type>),
}

// Special case for Matrix constructor
fn infer_matrix_constructor(...) {
    // Hardcoded logic for Matrix
}

// Hardcoded pattern matching
fn unify(t1: &Type, t2: &Type) -> Result<...> {
    match (t1, t2) {
        (Type::Scalar, Type::Scalar) => ...   // ← Fixed patterns
        (Type::Matrix(m1,n1), Matrix(m2,n2)) => ...
    }
}
```

### **After (Dynamic):**

```rust
// src/type_inference.rs - DYNAMIC
pub enum Type {
    // Bootstrap types (minimal, for parsing)
    Nat,
    String,
    Bool,
    
    // User-defined data type (loaded from Kleis!)
    Data {
        type_name: String,      // "Type"
        constructor: String,    // "Matrix", "Scalar", "Vector"
        args: Vec<Type>,        // [Nat(2), Nat(3)] for Matrix(2,3)
    },
    
    // Meta-level (for inference)
    Var(TypeVar),
    ForAll(TypeVar, Box<Type>),
}

// Generic data constructor inference
fn infer_data_constructor(
    &mut self,
    name: &str,
    args: &[Expression],
    registry: &DataTypeRegistry,
) -> Result<Type, String> {
    let variant = registry.lookup_variant(name)?;
    let params = self.extract_params(variant, args)?;
    let fields = self.infer_fields(variant, args)?;
    Ok(Type::Data { ... })
}

// Dynamic unification
fn unify(t1: &Type, t2: &Type, registry: &DataTypeRegistry) -> Result<...> {
    match (t1, t2) {
        // Bootstrap types
        (Type::Nat, Type::Nat) => Ok(...),
        
        // User-defined types (generic!)
        (Type::Data { type_name: t1, constructor: c1, args: a1 },
         Type::Data { type_name: t2, constructor: c2, args: a2 }) => {
            if t1 == t2 && c1 == c2 {
                unify_args(a1, a2, registry)  // Recursive!
            } else {
                Err("Different constructors")
            }
        }
        
        (Type::Var(v), t) | (t, Type::Var(v)) => ...
    }
}
```

**Kleis file loaded at startup:**
```kleis
// stdlib/types.kleis
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Complex
  | Currency(code: String)  // ← Users can add this!
```

---

## Implementation Steps

### **Step 1: Add Data Type AST (kleis_ast.rs)**

**Estimated time:** 2 hours

**Changes:**
```rust
// Add to TopLevel enum:
pub enum TopLevel {
    StructureDef(StructureDef),
    ImplementsDef(ImplementsDef),
    DataDef(DataDef),  // ← NEW!
    // ...
}

// New structs:
#[derive(Debug, Clone, PartialEq)]
pub struct DataDef {
    pub name: String,                    // "Type", "Option", "List"
    pub type_params: Vec<TypeParam>,     // (T), (m, n), etc.
    pub variants: Vec<DataVariant>,      // Constructors
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataVariant {
    pub name: String,                    // "Scalar", "Matrix", "None", "Some"
    pub fields: Vec<DataField>,          // Constructor arguments
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataField {
    pub name: Option<String>,            // Named or positional
    pub type_expr: TypeExpr,             // Field type
}
```

**Tests:**
- Create DataDef programmatically
- Verify structure is correct
- Check serialization/deserialization

---

### **Step 2: Add Parser Support (kleis_parser.rs)**

**Estimated time:** 4 hours

**Grammar to implement:**
```ebnf
dataDecl ::= "data" identifier [ "(" typeParams ")" ] "=" 
             dataVariant { "|" dataVariant }

dataVariant ::= identifier [ "(" dataFields ")" ]

dataFields ::= dataField { "," dataField }

dataField ::= [ identifier ":" ] typeExpr
```

**Implementation:**
```rust
fn parse_data_def(&mut self, start_pos: usize) -> Result<TopLevel, String> {
    self.expect_keyword("data")?;
    let name = self.parse_identifier()?;
    let type_params = self.parse_type_params()?;
    self.expect_token("=")?;
    
    let variants = self.parse_variants()?;
    
    Ok(TopLevel::DataDef(DataDef {
        name,
        type_params,
        variants,
    }))
}
```

**Tests:**
- Parse simple: `data Bool = True | False`
- Parse parametric: `data Option(T) = None | Some(T)`
- Parse complex: `data Type = Scalar | Matrix(Nat, Nat)`
- Parse errors: Invalid syntax

---

### **Step 3: Create Data Type Registry**

**Estimated time:** 3 hours

**New file: src/data_registry.rs**
```rust
use crate::kleis_ast::{DataDef, DataVariant};
use std::collections::HashMap;

pub struct DataTypeRegistry {
    /// Maps data type name → definition
    /// Example: "Type" → DataDef { variants: [Scalar, Matrix, ...] }
    types: HashMap<String, DataDef>,
    
    /// Maps variant name → (type name, variant)
    /// Example: "Matrix" → ("Type", DataVariant)
    variants: HashMap<String, (String, DataVariant)>,
}

impl DataTypeRegistry {
    pub fn new() -> Self { ... }
    
    pub fn register(&mut self, def: DataDef) -> Result<(), String> {
        // Register type
        self.types.insert(def.name.clone(), def.clone());
        
        // Register each variant
        for variant in &def.variants {
            self.variants.insert(
                variant.name.clone(),
                (def.name.clone(), variant.clone())
            );
        }
        Ok(())
    }
    
    pub fn lookup_variant(&self, name: &str) -> Option<&(String, DataVariant)> {
        self.variants.get(name)
    }
    
    pub fn get_type(&self, name: &str) -> Option<&DataDef> {
        self.types.get(name)
    }
}
```

**Tests:**
- Register simple data type
- Lookup variants
- Conflict detection (duplicate variants)

---

### **Step 4: Refactor Type Enum (type_inference.rs)**

**Estimated time:** 6 hours (most complex!)

**Replace Type enum:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Bootstrap types (for parsing and basic operations)
    Nat,
    String,
    Bool,
    
    // User-defined data type
    Data {
        type_name: String,      // Which data type: "Type", "Option", etc.
        constructor: String,    // Which variant: "Matrix", "Some", etc.
        args: Vec<Type>,        // Constructor arguments: [Nat(2), Nat(3)]
    },
    
    // Meta-level types (for type inference itself)
    Var(TypeVar),
    ForAll(TypeVar, Box<Type>),
}
```

**Update Display:**
```rust
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Type::Nat => write!(f, "Nat"),
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "Bool"),
            Type::Data { type_name, constructor, args } => {
                if args.is_empty() {
                    write!(f, "{}", constructor)  // "Scalar"
                } else {
                    write!(f, "{}({})", constructor, 
                        args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                }
            }
            Type::Var(TypeVar(n)) => write!(f, "α{}", n),
            Type::ForAll(TypeVar(n), t) => write!(f, "∀α{}. {}", n, t),
        }
    }
}
```

**Update unify():**
```rust
fn unify(t1: &Type, t2: &Type, registry: &DataTypeRegistry) -> Result<...> {
    match (t1, t2) {
        // Bootstrap types
        (Type::Nat, Type::Nat) => Ok(Substitution::empty()),
        (Type::String, Type::String) => Ok(Substitution::empty()),
        (Type::Bool, Type::Bool) => Ok(Substitution::empty()),
        
        // Data types (generic!)
        (Type::Data { type_name: t1, constructor: c1, args: a1 },
         Type::Data { type_name: t2, constructor: c2, args: a2 }) => {
            if t1 != t2 {
                return Err(format!("Cannot unify types from different data types: {} vs {}", t1, t2));
            }
            if c1 != c2 {
                return Err(format!("Cannot unify different constructors: {} vs {}", c1, c2));
            }
            
            // Same constructor - unify arguments recursively
            if a1.len() != a2.len() {
                return Err(format!("Constructor {} has different number of args", c1));
            }
            
            let mut subst = Substitution::empty();
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                let s = unify(&subst.apply(arg1), &subst.apply(arg2), registry)?;
                subst = subst.compose(&s);
            }
            Ok(subst)
        }
        
        // Type variables
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            if occurs(v, t) {
                Err(format!("Occurs check failed"))
            } else {
                Ok(Substitution::singleton(v.clone(), t.clone()))
            }
        }
        
        _ => Err(format!("Cannot unify {:?} with {:?}", t1, t2)),
    }
}
```

**Tests:**
- Unify Scalar with Scalar (bootstrap types)
- Unify Matrix(2,2) with Matrix(2,2) (data types)
- Fail Matrix(2,2) with Matrix(3,3) (different args)
- Fail Scalar with Matrix (different constructors)

---

### **Step 5: Generic Constructor Inference**

**Estimated time:** 4 hours

**Replace infer_matrix_constructor with:**
```rust
fn infer_data_constructor(
    &mut self,
    constructor_name: &str,
    args: &[Expression],
    context_builder: Option<&crate::type_context::TypeContextBuilder>,
    data_registry: &DataTypeRegistry,
) -> Result<Type, String> {
    // Lookup variant definition
    let (type_name, variant) = data_registry
        .lookup_variant(constructor_name)
        .ok_or_else(|| format!("Unknown data constructor: {}", constructor_name))?;
    
    // Validate argument count
    let expected_fields = variant.fields.len();
    if args.len() != expected_fields {
        return Err(format!(
            "Constructor {} expects {} arguments, got {}",
            constructor_name, expected_fields, args.len()
        ));
    }
    
    // Infer each field type and extract constructor parameters
    let mut constructor_args = Vec::new();
    
    for (i, (arg_expr, field_def)) in args.iter().zip(&variant.fields).enumerate() {
        let arg_type = self.infer(arg_expr, context_builder)?;
        
        // Check if this is a type parameter (like dimensions)
        // vs a value field (like matrix elements)
        match &field_def.type_expr {
            TypeExpr::Named(name) if name == "Nat" || name == "String" => {
                // This is a constructor parameter (metadata)
                // Extract the concrete value
                let param_value = match arg_expr {
                    Expression::Const(s) => s.clone(),
                    _ => return Err(format!("Constructor parameter {} must be constant", i)),
                };
                constructor_args.push(self.type_from_const(&field_def.type_expr, &param_value)?);
            }
            _ => {
                // This is a value field
                // Add constraint that it matches field type
                let expected = self.type_from_expr(&field_def.type_expr, data_registry)?;
                self.add_constraint(arg_type, expected);
            }
        }
    }
    
    Ok(Type::Data {
        type_name: type_name.clone(),
        constructor: constructor_name.to_string(),
        args: constructor_args,
    })
}
```

**Tests:**
- Infer Matrix(2, 3, a, b, c, d, e, f) → Type::Data { type_name: "Type", constructor: "Matrix", args: [Nat(2), Nat(3)] }
- Infer Scalar → Type::Data { type_name: "Type", constructor: "Scalar", args: [] }
- Infer Some(x) → Type::Data { type_name: "Option", constructor: "Some", args: [infer(x)] }

---

### **Step 6: Update TypeInference Struct**

**Estimated time:** 2 hours

**Add registry field:**
```rust
pub struct TypeInference {
    context: TypeContext,
    constraints: Vec<Constraint>,
    data_registry: DataTypeRegistry,  // ← NEW!
}

impl TypeInference {
    pub fn new() -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
            data_registry: DataTypeRegistry::new(),
        }
    }
    
    pub fn with_data_registry(registry: DataTypeRegistry) -> Self {
        TypeInference {
            context: TypeContext::new(),
            constraints: Vec::new(),
            data_registry: registry,
        }
    }
}
```

**Update infer_operation:**
```rust
fn infer_operation(...) -> Result<Type, String> {
    // Check if this is a data constructor
    if self.data_registry.lookup_variant(name).is_some() {
        return self.infer_data_constructor(name, args, context_builder, &self.data_registry);
    }
    
    // Otherwise delegate to context_builder (operations)
    // ...
}
```

---

### **Step 7: Update TypeChecker to Load Data Types**

**Estimated time:** 2 hours

```rust
// src/type_checker.rs

impl TypeChecker {
    pub fn with_stdlib() -> Result<Self, String> {
        let mut checker = Self::new();
        
        // PHASE 1: Load data type definitions
        let types_def = include_str!("../stdlib/types.kleis");
        checker.load_data_types(types_def)?;
        
        // PHASE 2: Load structures (uses types from Phase 1)
        checker.load_kleis(include_str!("../stdlib/minimal_prelude.kleis"))?;
        checker.load_kleis(include_str!("../stdlib/matrices.kleis"))?;
        
        Ok(checker)
    }
    
    fn load_data_types(&mut self, source: &str) -> Result<(), String> {
        let program = crate::kleis_parser::parse_kleis_program(source)?;
        
        for item in program.items {
            if let TopLevel::DataDef(data_def) = item {
                self.inference.data_registry.register(data_def)?;
            }
        }
        
        Ok(())
    }
}
```

---

### **Step 8: Create stdlib/types.kleis**

**Estimated time:** 1 hour

```kleis
// stdlib/types.kleis
// Defines the base types for Kleis type system
// 
// This file is loaded FIRST before any other stdlib files.
// It defines the types used throughout Kleis.

// The main Type data type
// This replaces the hardcoded Type enum in Rust!
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Complex
  | Set(T: Type)
  | List(T: Type)
  | Tensor(dims: List(Nat))

// Boolean type
data Bool = True | False

// Optional type
data Option(T) =
  | None
  | Some(value: T)

// Result type  
data Result(T, E) =
  | Ok(value: T)
  | Err(error: E)

// List type
data List(T) =
  | Nil
  | Cons(head: T, tail: List(T))
```

---

### **Step 9: Update type_context.rs**

**Estimated time:** 2 hours

**Update type_to_name() helper:**
```rust
fn type_to_name(&self, ty: &Type, registry: &DataTypeRegistry) -> Option<String> {
    match ty {
        // Bootstrap types
        Type::Nat => Some("Nat".to_string()),
        Type::String => Some("String".to_string()),
        Type::Bool => Some("Bool".to_string()),
        
        // Data types (generic!)
        Type::Data { type_name, constructor, args } => {
            if args.is_empty() {
                Some(constructor.clone())  // "Scalar"
            } else {
                // "Matrix(2, 3, ℝ)"
                Some(format!("{}({})", constructor,
                    args.iter().map(|a| type_to_name(a, registry).unwrap_or("?".to_string()))
                        .collect::<Vec<_>>().join(", ")))
            }
        }
        
        Type::Var(_) => None,
        Type::ForAll(_, _) => None,
    }
}
```

---

### **Step 10: Backward Compatibility**

**Estimated time:** 2 hours

**Keep convenience functions:**
```rust
impl Type {
    // Convenience constructors for common types
    pub fn scalar() -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Scalar".to_string(),
            args: vec![],
        }
    }
    
    pub fn matrix(m: usize, n: usize) -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::Nat, Type::Nat],  // Would store actual values
        }
    }
    
    pub fn vector(n: usize) -> Type {
        Type::Data {
            type_name: "Type".to_string(),
            constructor: "Vector".to_string(),
            args: vec![Type::Nat],
        }
    }
}
```

**Update all test files:**
- Replace `Type::Scalar` with `Type::scalar()`
- Replace `Type::Matrix(2, 3)` with `Type::matrix(2, 3)`
- Or keep both during transition

---

### **Step 11: Migration Strategy**

**Two-phase migration:**

**Phase A: Parallel Support (Week 1)**
- Both old Type enum and new Data type work
- Tests use old syntax
- New code uses data registry
- No functionality broken

**Phase B: Complete Switch (Week 2)**
- Remove old Type variants
- Update all tests
- Pure data type system
- Verify everything works

---

## Testing Strategy

### **Unit Tests (each step)**
1. AST construction
2. Parser functionality
3. Registry operations
4. Constructor inference
5. Unification with data types
6. Type conversion helpers

### **Integration Tests**
1. Load stdlib/types.kleis
2. Infer types using data constructors
3. Full type checking pipeline
4. Error messages still helpful

### **Regression Tests**
1. All 281 existing lib tests still pass
2. All 7 validation tests still pass
3. No performance degradation
4. Error messages as good or better

---

## Risk Assessment

### **High Risk Areas**

**1. Unification Complexity** ⚠️ **HIGH**
- Recursive unification of data type args
- Edge cases: nested types, type variables in args
- Must handle occurs check correctly

**2. Backward Compatibility** ⚠️ **MEDIUM**
- 281 tests reference Type::Scalar, Type::Matrix
- Need migration strategy
- Could break existing code

**3. Bootstrap Chicken-Egg** ⚠️ **MEDIUM**
- Need types to parse Kleis
- Need to parse Kleis to get types
- Bootstrap types must be minimal

**4. Performance** ⚠️ **LOW**
- HashMap lookups vs enum matching
- Should be negligible
- Can optimize later if needed

---

## Rollback Plan

**If things go wrong:**

1. **Git tag before starting:** `v0.6.0-adr016-complete`
2. **Branch strategy:** Work on feature branch first
3. **Incremental commits:** Commit after each step
4. **Tests as safety net:** All tests must pass before merging
5. **Can revert:** `git checkout v0.6.0-adr016-complete`

---

## Success Criteria

**Must achieve:**
- ✅ Load stdlib/types.kleis successfully
- ✅ Matrix(2, 3, ...) works as data constructor
- ✅ All 288 tests pass
- ✅ Unification works with data types
- ✅ Error messages still helpful

**Nice to have:**
- Users can add custom types
- Performance similar to before
- Code is cleaner than before

---

## Timeline

**Aggressive (1 week):**
- Days 1-2: AST + Parser (Steps 1-2)
- Days 3-4: Type enum refactor (Step 4)
- Day 5: Registry + Integration (Steps 3, 5-7)
- Day 6-7: Testing + Polish (Steps 8-11)

**Realistic (2 weeks):**
- Week 1: AST, Parser, Registry (Steps 1-3)
- Week 2: Type refactor, Integration, Testing (Steps 4-11)

**Conservative (3 weeks):**
- Week 1: Design + AST + Parser (Steps 1-2)
- Week 2: Type refactor (Step 4)
- Week 3: Integration + Testing + Polish (Steps 5-11)

---

## Dependencies

### **Before Starting:**
- ✅ ADR-016 complete (operations in structures)
- ✅ ADR-020 complete (type/value distinction)
- ✅ Generic validation implemented
- ✅ Tests comprehensive (288 passing)
- ✅ Code documented with vision

### **Blocks Other Work:**
- Parser extension (can start in parallel)
- Matrix UI fix (will be solved by this)
- Full prelude loading (needs this first)

---

## Expected Outcomes

### **After ADR-021 Implementation:**

**Capabilities:**
```kleis
// Users can define new types:
data Currency = USD | EUR | GBP | JPY

// Users can extend base types:
data MyTypes =
  | Quantity(value: ℝ, unit: String)
  | Temperature(kelvin: ℝ)

// Type system validates automatically!
```

**Code improvements:**
- Matrix constructor: ~60 lines → ~10 lines (generic)
- No special cases for types
- Users extend without recompiling
- True self-hosting (Level 2 of 3)

**Matrix issue resolved:**
- Matrix becomes a data constructor
- No special handling needed
- Works like any other type
- UI dimension issue disappears (proper value constructor)

---

## Next Steps

**Before starting ADR-021:**
1. ✅ Document the plan (this file)
2. ✅ Verify all tests pass (288/288)
3. ✅ Create git tag: `v0.6.0-adr016-complete`
4. ✅ Commit and push

**Starting ADR-021:**
1. Create feature branch: `feature/adr-021-data-types`
2. Start with Step 1 (AST changes)
3. Incremental commits after each step
4. Run tests continuously

---

## Files That Will Change

| File | Changes | Complexity |
|------|---------|------------|
| **src/kleis_ast.rs** | +DataDef, +DataVariant | Low |
| **src/kleis_parser.rs** | +parse_data_def | Medium |
| **src/data_registry.rs** | NEW FILE | Medium |
| **src/type_inference.rs** | Type enum refactor | **HIGH** |
| **src/type_context.rs** | type_to_name update | Low |
| **src/type_checker.rs** | load_data_types | Low |
| **src/lib.rs** | Export data_registry | Low |
| **stdlib/types.kleis** | NEW FILE | Low |
| **tests/*.rs** | Update Type references | Medium |

**Estimated total:** ~600 lines changed

---

## Recommended Approach

### **My Recommendation: Incremental with Feature Branch**

**Week 1:**
```
Day 1: AST (kleis_ast.rs) + basic tests
Day 2: Parser (kleis_parser.rs) + parse tests  
Day 3: Registry (data_registry.rs) + registry tests
Day 4: Start Type refactor (backward compat mode)
Day 5: Buffer / catchup
```

**Week 2:**
```
Day 1: Complete Type refactor
Day 2: Update unify() for data types
Day 3: Generic constructor inference
Day 4: Integration + stdlib/types.kleis
Day 5: Testing + fix issues
Weekend: Polish + documentation
```

---

## Context for Next Session

**Starting point:**
- Tag: `v0.6.0-adr016-complete`
- All 288 tests passing
- Documentation complete
- Plan ready

**First task:**
- Create feature branch
- Add DataDef to kleis_ast.rs
- Write basic tests

**End goal:**
- Type system reads `data` definitions from Kleis
- Matrix is just a data constructor
- Users can define custom types
- Meta-circularity (Level 2 of 3)

---

**This is the path to TRUE self-hosting!** 🎯

**Status:** 📋 PLAN COMPLETE  
**Ready:** Tag and push, then start next session  
**Estimated:** 1-2 weeks for full implementation


