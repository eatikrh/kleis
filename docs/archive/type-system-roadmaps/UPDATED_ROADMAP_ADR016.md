# Updated Roadmap: Operations in Structures (ADR-016)

**Date:** December 6, 2025  
**Decision:** ADR-016 - Operations belong in structures  
**Impact:** Updated implementation plan

---

## Design Change

### Before (Top-Level Operations)

```kleis
operation abs : ℝ → ℝ
operation card : Set(T) → ℕ
```

### After (Structures + Implements) ✅

```kleis
structure Numeric(N) {
    operation abs : N → N
}

implements Numeric(ℝ) {
    operation abs = builtin_abs
}
```

---

## Updated Implementation Plan

### Phase 1: Extend AST for Implements ⬜ (1 day)

**Add to `src/kleis_ast.rs`:**

```rust
/// Implements block: implements StructureName(Type) { ... }
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementsDef {
    pub structure_name: String,
    pub type_arg: TypeExpr,
    pub members: Vec<ImplMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplMember {
    /// Element binding: element zero = 0
    Element {
        name: String,
        value: Expression,
    },
    
    /// Operation implementation: operation abs = builtin_abs
    Operation {
        name: String,
        implementation: Implementation,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Implementation {
    /// Builtin function: builtin_abs
    Builtin(String),
    
    /// Defined inline: (x) = x^2
    Inline {
        params: Vec<String>,
        body: Expression,
    },
}

// Add to TopLevel enum
pub enum TopLevel {
    StructureDef(StructureDef),
    ImplementsDef(ImplementsDef),  // ← NEW
    OperationDecl(OperationDecl),
    FunctionDef(FunctionDef),
    TypeAlias(TypeAlias),
}
```

---

### Phase 2: Extend Parser for Implements ⬜ (2-3 days)

**Add to `src/kleis_parser.rs`:**

```rust
impl KleisParser {
    /// Parse implements block
    /// Example: implements Numeric(ℝ) { operation abs = builtin_abs }
    pub fn parse_implements(&mut self) -> Result<ImplementsDef, KleisParseError> {
        // Expect 'implements'
        let keyword = self.parse_identifier()?;
        if keyword != "implements" {
            return Err(/* ... */);
        }
        
        // Parse structure name
        let structure_name = self.parse_identifier()?;
        
        // Parse type argument: (ℝ) or (Vector(n))
        if self.advance() != Some('(') {
            return Err(/* ... */);
        }
        let type_arg = self.parse_type()?;
        if self.advance() != Some(')') {
            return Err(/* ... */);
        }
        
        // Parse members in { }
        if self.advance() != Some('{') {
            return Err(/* ... */);
        }
        
        let mut members = Vec::new();
        while self.peek() != Some('}') {
            members.push(self.parse_impl_member()?);
        }
        
        if self.advance() != Some('}') {
            return Err(/* ... */);
        }
        
        Ok(ImplementsDef {
            structure_name,
            type_arg,
            members,
        })
    }
    
    fn parse_impl_member(&mut self) -> Result<ImplMember, KleisParseError> {
        self.skip_whitespace();
        let keyword = self.parse_identifier()?;
        
        match keyword.as_str() {
            "element" => {
                // element zero = 0
                let name = self.parse_identifier()?;
                self.expect('=')?;
                let value = self.parse_expression()?;
                Ok(ImplMember::Element { name, value })
            }
            "operation" => {
                // operation abs = builtin_abs
                // or operation abs(x) = x^2
                let name = self.parse_identifier()?;
                self.expect('=')?;
                
                // Check if it's inline definition or builtin
                if self.peek() == Some('(') {
                    // Inline: operation abs(x) = x^2
                    // (parse params and body)
                } else {
                    // Builtin: operation abs = builtin_abs
                    let builtin_name = self.parse_identifier()?;
                    Ok(ImplMember::Operation {
                        name,
                        implementation: Implementation::Builtin(builtin_name),
                    })
                }
            }
            _ => Err(/* ... */)
        }
    }
}
```

**Update `parse_program()`:**
```rust
if self.peek_word("implements") {
    let impl_def = self.parse_implements()?;
    program.add_item(TopLevel::ImplementsDef(impl_def));
}
```

---

### Phase 3: Create stdlib/core.kleis (Redesigned) ⬜ (1 day)

**File:** `stdlib/core.kleis`

```kleis
@library("kleis.core")
@version("1.0.0")

// ============================================
// NUMERIC OPERATIONS
// ============================================

structure Numeric(N) {
    operation abs : N → N
    
    axiom abs_non_negative: ∀ (x : N) . abs(x) ≥ 0
    axiom abs_positive_definite: ∀ (x : N) . abs(x) = 0 ⟺ x = 0
    axiom abs_symmetric: ∀ (x : N) . abs(-x) = abs(x)
    axiom abs_triangle: ∀ (x y : N) . abs(x + y) ≤ abs(x) + abs(y)
}

implements Numeric(ℝ) {
    operation abs = builtin_abs_real
}

implements Numeric(ℂ) {
    operation abs = complex_modulus
}

// ============================================
// SET OPERATIONS
// ============================================

structure Finite(C) {
    // C is a "container" type
    operation card : C → ℕ
    
    axiom card_non_negative: ∀ (c : C) . card(c) ≥ 0
}

implements Finite(Set(T)) {
    operation card = builtin_set_cardinality
}

implements Finite(List(T)) {
    operation card = builtin_list_length
}

// ============================================
// VECTOR OPERATIONS
// ============================================

structure NormedSpace(V) {
    operation norm : V → ℝ
    
    axiom norm_non_negative: ∀ (v : V) . norm(v) ≥ 0
    axiom norm_positive_definite: ∀ (v : V) . norm(v) = 0 ⟺ v = 0⃗
    axiom norm_scalar: ∀ (α : ℝ) (v : V) . norm(α × v) = abs(α) × norm(v)
    axiom norm_triangle: ∀ (u v : V) . norm(u + v) ≤ norm(u) + norm(v)
}

implements NormedSpace(Vector(n)) {
    operation norm(v) = √(dot(v, v))
}

// ============================================
// DISPLAY MODE (Special Case - Top Level)
// ============================================

// frac is a rendering hint, not a mathematical structure operation
// Keep as top-level utility
operation frac : ℝ × ℝ → ℝ
define frac(a, b) = a / b
```

---

### Phase 4: Type Context Builder ⬜ (2-3 days)

**Create `src/type_context.rs`:**

```rust
pub struct TypeContextBuilder {
    structures: HashMap<String, StructureDef>,
    implements: Vec<ImplementsDef>,
    context: TypeContext,
}

impl TypeContextBuilder {
    pub fn from_program(program: Program) -> Result<TypeContext, String> {
        let mut builder = Self::new();
        
        // Step 1: Register all structures (abstract operations)
        for item in &program.items {
            if let TopLevel::StructureDef(s) = item {
                builder.register_structure(s)?;
            }
        }
        
        // Step 2: Register all implements (concrete bindings)
        for item in &program.items {
            if let TopLevel::ImplementsDef(impl_def) = item {
                builder.register_implements(impl_def)?;
            }
        }
        
        // Step 3: Build operation registry
        builder.build_operation_registry()?;
        
        Ok(builder.context)
    }
    
    fn register_structure(&mut self, structure: &StructureDef) -> Result<(), String> {
        // Register abstract operations
        for member in &structure.members {
            if let StructureMember::Operation { name, type_signature } = member {
                self.context.register_abstract_operation(
                    &structure.name,
                    name,
                    type_signature
                );
            }
        }
        self.structures.insert(structure.name.clone(), structure.clone());
        Ok(())
    }
    
    fn register_implements(&mut self, impl_def: &ImplementsDef) -> Result<(), String> {
        // Find the structure
        let structure = self.structures.get(&impl_def.structure_name)
            .ok_or_else(|| format!("Unknown structure: {}", impl_def.structure_name))?;
        
        // Register concrete operations for this type
        for member in &impl_def.members {
            if let ImplMember::Operation { name, implementation } = member {
                self.context.register_concrete_operation(
                    &impl_def.type_arg,
                    name,
                    implementation
                );
            }
        }
        
        self.implements.push(impl_def.clone());
        Ok(())
    }
    
    fn build_operation_registry(&mut self) -> Result<(), String> {
        // For each implements:
        //   - Find structure operations
        //   - Register type as supporting those operations
        
        for impl_def in &self.implements {
            let structure = &self.structures[&impl_def.structure_name];
            
            for member in &structure.members {
                if let StructureMember::Operation { name, type_signature } = member {
                    self.context.registry.add_support(
                        name,
                        impl_def.type_arg.clone(),
                        type_signature.clone()
                    );
                }
            }
        }
        
        Ok(())
    }
}
```

---

### Phase 5: Test Complete Pattern ⬜ (1-2 days)

**Test file:** `tests/test_operations_in_structures.rs`

```rust
#[test]
fn test_abs_polymorphic() {
    // Parse stdlib/core.kleis
    let stdlib = parse_kleis_file("stdlib/core.kleis").unwrap();
    
    // Build context
    let ctx = TypeContextBuilder::from_program(stdlib).unwrap();
    
    // Test: abs works for ℝ
    let real_expr = parse_kleis("abs(x)").unwrap();
    ctx.bind("x", Type::Real);
    let ty = ctx.infer(&real_expr).unwrap();
    assert_eq!(ty, Type::Real);
    
    // Test: abs works for ℂ  
    let complex_expr = parse_kleis("abs(z)").unwrap();
    ctx.bind("z", Type::Complex);
    let ty = ctx.infer(&complex_expr).unwrap();
    assert_eq!(ty, Type::Real);  // Complex abs returns Real!
    
    // Test: abs doesn't work for Set
    let set_expr = parse_kleis("abs(S)").unwrap();
    ctx.bind("S", Type::Set(Box::new(Type::Integer)));
    let result = ctx.infer(&set_expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Set(ℤ) does not implement Numeric"));
}

#[test]
fn test_card_for_sets() {
    let ctx = /* ... */;
    
    // card works for Set
    let set_expr = parse_kleis("card(S)").unwrap();
    ctx.bind("S", Type::Set(Box::new(Type::Integer)));
    let ty = ctx.infer(&set_expr).unwrap();
    assert_eq!(ty, Type::Nat);
    
    // card doesn't work for Number
    let num_expr = parse_kleis("card(x)").unwrap();
    ctx.bind("x", Type::Real);
    let result = ctx.infer(&num_expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ℝ does not implement Finite"));
}
```

---

## Updated Timeline

```
Week 1:
├── Day 1: Add ImplementsDef to AST
├── Day 2-3: Parse implements blocks
└── Day 4: Redesign stdlib/core.kleis

Week 2:
├── Day 1-3: Type context builder (structures + implements)
└── Day 4-5: Testing and validation

Result: ✅ Operations in structures pattern working!
```

**Total:** 1.5-2 weeks

---

## Benefits of This Approach

### 1. Polymorphism ✅
```kleis
// abs works for ANY Numeric type
structure Numeric(N) { operation abs : N → N }
implements Numeric(ℝ)
implements Numeric(ℂ)
implements Numeric(Quaternion)  // User can add!
```

### 2. Better Error Messages ✅
```
Error: Set(ℤ) does not implement Numeric
  Required for: abs operation
  Available structures for Set: Finite (provides card)
  Suggestion: Did you mean card(S)?
```

### 3. Extension by Users ✅
```kleis
// User adds their type
structure Matrix3D { ... }

// User adds to existing structure!
implements Numeric(Matrix3D) {
    operation abs = matrix_frobenius_norm
}

// Now abs(matrix) works!
```

### 4. Type-Driven Dispatch ✅
```kleis
define magnitude<T: Numeric>(x: T) = abs(x)
// Works for ℝ, ℂ, or any Numeric!
```

---

## What This Changes

### Parser (kleis_parser.rs)
- ✅ Already parses structures
- ⬜ Need to parse implements blocks
- ⬜ Need to parse implementation members

### Type Context (type_context.rs - new file)
- ⬜ Load structures (abstract operations)
- ⬜ Load implements (concrete operations)
- ⬜ Build operation registry
- ⬜ Query: "Which types support operation X?"

### stdlib/core.kleis (new design)
- ⬜ Define Numeric(N) structure
- ⬜ Define Finite(C) structure  
- ⬜ Define NormedSpace(V) structure
- ⬜ Implement for ℝ, ℂ, Set, Vector

### Type Checking
- ⬜ Check if type implements required structure
- ⬜ Generate suggestions based on available structures

---

## Next Immediate Steps

### Step 1: Extend AST (Today)

Add `ImplementsDef` and related types to `kleis_ast.rs`

**File:** `src/kleis_ast.rs`  
**Time:** 1 hour  
**Test:** Compiles without errors

---

### Step 2: Parse Implements (Tomorrow)

Add parsing for:
```kleis
implements Numeric(ℝ) {
    operation abs = builtin_abs
}
```

**File:** `src/kleis_parser.rs`  
**Time:** 2-3 hours  
**Test:** `cargo test parse_implements`

---

### Step 3: Test Parsing (Tomorrow)

Create test file and verify:
```kleis
structure Numeric(N) { operation abs : N → N }
implements Numeric(ℝ) { operation abs = builtin_abs }
```

**File:** `tests/test_parse_implements.rs`  
**Time:** 1 hour  
**Test:** Parses correctly

---

### Step 4: Design stdlib/core.kleis (Day 3)

Write the structures and implements:

**File:** `stdlib/core.kleis`  
**Time:** 2-3 hours  
**Test:** Parses without errors

---

### Step 5: Type Context Builder (Days 4-5)

Connect parsed structures → type context

**File:** `src/type_context.rs`  
**Time:** 1-2 days  
**Test:** Can load stdlib and query operations

---

## Success Criteria

When complete:

✅ Can parse structure definitions  
✅ Can parse implements blocks  
✅ stdlib/core.kleis uses structures pattern  
✅ Type context understands implements  
✅ Can query "which types support abs?"  
✅ Error messages reference structures  
✅ Users can extend with their types  

---

**Status:** 🎯 **Plan Updated for ADR-016**  
**Start:** Extend AST for ImplementsDef  
**Timeline:** 1.5-2 weeks to complete

