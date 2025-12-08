# User-Defined Parametrized Types in Signatures

**Date:** December 8, 2024  
**Context:** ADR-021 implementation completion  
**Problem:** SignatureInterpreter doesn't handle user-defined types  
**Status:** 🔴 Design problem identified, solution sketched

---

## The Problem

### Current Limitation

After ADR-021, users can define types like:

```kleis
data Currency = USD | EUR | GBP
data MyList(T) = Nil | Cons(head: T, tail: MyList(T))
data Option(T) = None | Some(value: T)
```

But if they try to use these in **operation signatures**:

```kleis
structure Tradeable(C) {
  operation exchange : C → ℝ
}

implements Tradeable(Currency) {
  operation exchange = builtin_exchange
}
```

**What happens:**
```rust
// In signature_interpreter.rs line 320:
TypeExpr::Named("Currency") 
  → Default case: Ok(Type::scalar())  // ❌ WRONG!
```

**Currency gets interpreted as Scalar!** This breaks type checking.

---

## Problem Taxonomy

### **Level 1: Simple User Types** (Not Working)

```kleis
data Currency = USD | EUR | GBP

// This signature interpretation fails:
operation exchange : Currency → ℝ
```

**Why:** `Currency` unknown, defaults to `Scalar`

---

### **Level 2: Parametric User Types** (Not Working)

```kleis
data Option(T) = None | Some(T)

// This signature interpretation fails:
operation unwrap : Option(ℝ) → ℝ
```

**Why:** `Option(ℝ)` unknown, treated as error

---

### **Level 3: Higher-Kinded Type Variables** (Complex!)

```kleis
data MyList(T) = Nil | Cons(T, MyList(T))

structure Functor(F, A, B) {
  operation map : (A → B) → F(A) → F(B)
}

implements Functor(MyList, ℝ, ℂ) {
  operation map = builtin_mylist_map
}
```

**Challenge:** 
- `F` is bound to `MyList` (a **type constructor**, not a type!)
- `F(A)` means "apply MyList to A"
- This is **System F** territory (higher-kinded polymorphism)

---

### **Level 4: Dependent Type Parameters** (Very Complex!)

```kleis
data Matrix(m: Nat, n: Nat) = Matrix(elements: List(ℝ))

structure HasDimensions(M) {
  operation rows : M → Nat
}

implements HasDimensions(Matrix(m, n)) {
  // m, n are type-level variables here!
  operation rows = builtin_matrix_rows
}
```

**Challenge:**
- `m` and `n` are **type-level variables** (not bound to concrete values)
- The signature `Matrix(m, n)` is polymorphic over dimensions
- This is **dependent types** territory!

---

## Current Architecture Gap

### What SignatureInterpreter Has

```rust
pub struct SignatureInterpreter {
    /// Maps dimension variables to concrete values
    /// Example: {m: 2, n: 3}
    bindings: HashMap<String, usize>,
}
```

**Only handles:** Natural number values (dimensions)

### What It Needs

```rust
pub struct SignatureInterpreter {
    /// Dimension bindings (current)
    dim_bindings: HashMap<String, usize>,
    
    /// Type bindings (NEW!)
    /// Maps type variables to concrete types
    /// Example: {T: Scalar, U: Complex}
    type_bindings: HashMap<String, Type>,
    
    /// Type constructor bindings (NEW!)
    /// Maps type constructor variables to data definitions
    /// Example: {F: MyList}
    type_constructor_bindings: HashMap<String, DataDef>,
    
    /// Registry for looking up user-defined types (NEW!)
    data_registry: &DataTypeRegistry,
}
```

---

## Solution Ideas

### Idea 1: Thread Registry (Pragmatic - 90% Solution)

**Approach:** Add `data_registry` parameter, handle Levels 1-2

```rust
fn interpret_type_expr(
    &self, 
    type_expr: &TypeExpr,
    registry: &DataTypeRegistry  // ← Add this!
) -> Result<Type, String> {
    match type_expr {
        TypeExpr::Named(name) => {
            // NEW: Check registry first
            if registry.has_type(name) {
                return Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args: vec![],
                });
            }
            
            // Fallback to built-ins
            match name.as_str() {
                "ℝ" => Ok(Type::scalar()),
                "T" => Ok(Type::scalar()),
                _ => Err(format!("Unknown type: {}", name))
            }
        }
        
        TypeExpr::Parametric(name, params) => {
            // NEW: Check registry for parametric types
            if registry.has_type(name) {
                let args = params.iter()
                    .map(|p| self.interpret_type_expr(p, registry))
                    .collect::<Result<Vec<_>, _>>()?;
                
                return Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args,
                });
            }
            
            // Fallback to Matrix/Vector
            if name == "Matrix" { /* ... */ }
        }
    }
}
```

**Handles:**
- ✅ Level 1: Simple types (`Currency`)
- ✅ Level 2: Parametric with concrete args (`Option(ℝ)`)
- ❌ Level 3: Type constructor variables (`F(A)`)
- ❌ Level 4: Dimension polymorphism (`Matrix(m, n)`)

**Effort:** 1-2 hours

---

### Idea 2: Add Type-Level Bindings (Better - 95% Solution)

**Approach:** Separate dimension bindings from type bindings

```rust
impl SignatureInterpreter {
    fn bind_type_params(
        &mut self,
        structure: &StructureDef,
        type_args: &[TypeExpr],
        registry: &DataTypeRegistry,
    ) -> Result<(), String> {
        for (param, arg) in structure.type_params.iter().zip(type_args) {
            match param.kind.as_deref() {
                Some("Nat") => {
                    // Dimension parameter: extract usize
                    let value = self.eval_param(arg)?;
                    self.dim_bindings.insert(param.name.clone(), value);
                }
                Some("Type") | None => {
                    // Type parameter: interpret as Type
                    let ty = self.interpret_type_expr(arg, registry)?;
                    self.type_bindings.insert(param.name.clone(), ty);
                }
                _ => {
                    return Err(format!("Unknown kind: {:?}", param.kind));
                }
            }
        }
        Ok(())
    }
    
    fn interpret_type_expr(&self, type_expr: &TypeExpr, registry: &DataTypeRegistry) -> Result<Type, String> {
        match type_expr {
            TypeExpr::Named(name) => {
                // Check if it's a type variable binding
                if let Some(ty) = self.type_bindings.get(name) {
                    return Ok(ty.clone());
                }
                
                // Check if it's a user-defined type
                if registry.has_type(name) {
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args: vec![],
                    });
                }
                
                // Check if it's a dimension variable (for Nat)
                if let Some(&value) = self.dim_bindings.get(name) {
                    return Ok(Type::NatValue(value));
                }
                
                // Built-in types
                match name.as_str() {
                    "ℝ" => Ok(Type::scalar()),
                    "Nat" => Ok(Type::Nat),
                    _ => Err(format!("Unknown type: {}", name))
                }
            }
            
            TypeExpr::Parametric(name, params) => {
                // Handle user-defined parametric types
                if registry.has_type(name) {
                    let args = params.iter()
                        .map(|p| self.interpret_type_expr(p, registry))
                        .collect::<Result<Vec<_>, _>>()?;
                    
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args,
                    });
                }
                
                // Fallback to hardcoded
                // ...
            }
        }
    }
}
```

**Handles:**
- ✅ Level 1: Simple types (`Currency`)
- ✅ Level 2: Parametric types (`Option(ℝ)`, `MyList(Currency)`)
- ✅ Type variable substitution (`T` → `Scalar`)
- ❌ Level 3: Type constructor variables (`F` as type constructor)
- ⚠️ Level 4: Partial (dimension variables work, but not polymorphic)

**Effort:** 2-3 hours

---

### Idea 3: Full Higher-Kinded Types (Complete - 100% Solution)

**Approach:** Implement System F-style higher-kinded polymorphism

```rust
#[derive(Debug, Clone)]
pub enum Kind {
    /// Type kind: * (Scalar, ℝ, Currency are of kind Type)
    Type,
    
    /// Natural number kind (for dimensions)
    Nat,
    
    /// Function kind: Type → Type (MyList, Option are of this kind)
    Arrow(Box<Kind>, Box<Kind>),
}

pub enum TypeBinding {
    /// Concrete type
    ConcreteType(Type),
    
    /// Type constructor (takes types, returns type)
    TypeConstructor {
        name: String,
        kind: Kind,  // e.g., Type → Type for Option
        arity: usize,
        apply: Box<dyn Fn(Vec<Type>) -> Type>,  // Or store DataDef
    },
}

impl SignatureInterpreter {
    fn interpret_type_application(
        &self,
        ctor: &TypeBinding,
        args: &[TypeExpr],
        registry: &DataTypeRegistry,
    ) -> Result<Type, String> {
        match ctor {
            TypeBinding::TypeConstructor { name, kind, arity, .. } => {
                // Check arity
                if args.len() != *arity {
                    return Err(format!("{} expects {} type arguments", name, arity));
                }
                
                // Interpret arguments
                let arg_types = args.iter()
                    .map(|a| self.interpret_type_expr(a, registry))
                    .collect::<Result<Vec<_>, _>>()?;
                
                // Apply type constructor
                Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args: arg_types,
                })
            }
            _ => Err("Not a type constructor".to_string()),
        }
    }
}
```

**Handles:**
- ✅ Level 1: Simple types
- ✅ Level 2: Parametric types  
- ✅ Level 3: Type constructor variables (`F(A)`)
- ✅ Level 4: Kind checking
- ✅ Full System F style polymorphism

**Effort:** 1-2 weeks (major undertaking!)

**Complexity:** HIGH - needs kind inference, kind checking, higher-ranked types

---

## Concrete Examples

### Example 1: Currency (Level 1)

**Kleis code:**
```kleis
data Currency = USD | EUR | GBP

structure Tradeable(C) {
  operation exchange : C → ℝ
}

implements Tradeable(Currency) {
  operation exchange = builtin_exchange
}
```

**What interpreter sees:**
```
Structure: Tradeable(C)
TypeArgs: [Currency]
Bindings needed: {C: Currency}

Signature: C → ℝ
Interpret: Look up C → Find Currency type → Return Type::Data { "Currency", "Currency", [] }
```

**Fix:** Check registry in `TypeExpr::Named` case

---

### Example 2: Option(ℝ) (Level 2)

**Kleis code:**
```kleis
data Option(T) = None | Some(T)

structure Unwrappable(M, T) {
  operation unwrap : M(T) → T
}

implements Unwrappable(Option, ℝ) {
  operation unwrap = builtin_option_unwrap
}
```

**What interpreter sees:**
```
Structure: Unwrappable(M, T)
TypeArgs: [Option, ℝ]
Bindings needed: {M: ???, T: Scalar}

Signature: M(T) → T
Problem: What is M? It's a type constructor, not a type!
```

**Fix:** Add `TypeConstructor` binding type, handle application

---

### Example 3: Functor Map (Level 3)

**Kleis code:**
```kleis
data MyList(T) = Nil | Cons(T, MyList(T))

structure Functor(F, A, B) {
  operation map : (A → B) → F(A) → F(B)
}

implements Functor(MyList, ℝ, ℂ) {
  operation map = builtin_mylist_map
}
```

**What interpreter sees:**
```
Structure: Functor(F, A, B)
TypeArgs: [MyList, ℝ, ℂ]

Bindings needed:
  F: MyList (type constructor of kind: Type → Type)
  A: ℝ (concrete type)
  B: ℂ (concrete type)

Signature: (A → B) → F(A) → F(B)
Interpretation:
  A → Scalar
  B → Data { "Complex", ... }
  F(A) → Apply(MyList, [Scalar]) → MyList(ℝ)
  F(B) → Apply(MyList, [Complex]) → MyList(ℂ)
```

**Fix:** Full type application mechanism with kind tracking

---

### Example 4: Dimension Polymorphism (Level 4)

**Kleis code:**
```kleis
data Vec(n: Nat) = Vec(elements: List(ℝ))

structure VectorAddable(n: Nat) {
  operation add : Vec(n) → Vec(n) → Vec(n)
}

// Polymorphic over ALL dimensions!
implements VectorAddable(n) {
  operation add = builtin_vec_add
}
```

**What interpreter sees:**
```
Structure: VectorAddable(n: Nat)
TypeArgs: [n]  // n is a TYPE-LEVEL variable!

Bindings needed:
  n: TypeVariable (not a concrete value!)

Signature: Vec(n) → Vec(n) → Vec(n)
Interpretation:
  Vec(n) where n is variable → Need dependent types or existentials
```

**Fix:** Dependent type support or bounded quantification

---

## Architecture Solutions

### Solution 1: Pragmatic Registry Lookup (Short-term)

**Goal:** Handle Levels 1-2 without major refactoring

**Changes Required:**
1. Thread `data_registry: &DataTypeRegistry` through SignatureInterpreter
2. Update `interpret_type_expr` signature
3. Check registry before defaulting

**Code sketch:**
```rust
impl SignatureInterpreter {
    fn new_with_registry(registry: &DataTypeRegistry) -> Self { ... }
    
    fn interpret_type_expr(
        &self,
        type_expr: &TypeExpr,
        registry: &DataTypeRegistry,
    ) -> Result<Type, String> {
        match type_expr {
            TypeExpr::Named(name) => {
                // 1. Check user-defined simple types
                if registry.has_type(name) {
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args: vec![],
                    });
                }
                // 2. Fallback to built-ins
                // ...
            }
            
            TypeExpr::Parametric(name, params) => {
                // 1. Check user-defined parametric types
                if registry.has_type(name) {
                    let args = params.iter()
                        .map(|p| self.interpret_type_expr(p, registry))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args,
                    });
                }
                // 2. Fallback to Matrix/Vector
                // ...
            }
        }
    }
}
```

**Pros:**
- Quick to implement (1-2 hours)
- Handles most common cases
- No major architecture changes

**Cons:**
- Doesn't handle Level 3 (type constructor variables)
- Still has Matrix/Vector special cases

---

### Solution 2: Type-Level Bindings (Medium-term)

**Goal:** Handle Levels 1-3 with proper type/dimension separation

**Changes Required:**
1. Add `type_bindings: HashMap<String, TypeBinding>` to SignatureInterpreter
2. Create `TypeBinding` enum for different binding kinds
3. Update `bind_from_args` to classify parameters by kind
4. Handle type application

**Type Binding Enum:**
```rust
#[derive(Debug, Clone)]
pub enum TypeBinding {
    /// Dimension value: n = 3
    Dimension(usize),
    
    /// Concrete type: T = Scalar
    ConcreteType(Type),
    
    /// Type constructor: F = MyList
    /// Stores the DataDef for later application
    TypeConstructor {
        name: String,
        arity: usize,
        definition: DataDef,
    },
}
```

**Binding Logic:**
```rust
fn bind_type_params(
    &mut self,
    structure: &StructureDef,
    type_args: &[TypeExpr],
    registry: &DataTypeRegistry,
) -> Result<(), String> {
    for (param, arg_expr) in structure.type_params.iter().zip(type_args) {
        match param.kind.as_deref() {
            Some("Nat") => {
                // Bind dimension
                let value = self.eval_dimension(arg_expr)?;
                self.bindings.insert(param.name.clone(), 
                    TypeBinding::Dimension(value));
            }
            
            Some("Type") | None => {
                // Type parameter - need to determine if it's constructor or concrete
                match arg_expr {
                    TypeExpr::Named(name) if registry.has_type(name) => {
                        let def = registry.get_type(name).unwrap();
                        if def.type_params.is_empty() {
                            // Simple type
                            self.bindings.insert(param.name.clone(),
                                TypeBinding::ConcreteType(Type::Data { ... }));
                        } else {
                            // Type constructor!
                            self.bindings.insert(param.name.clone(),
                                TypeBinding::TypeConstructor {
                                    name: name.clone(),
                                    arity: def.type_params.len(),
                                    definition: def.clone(),
                                });
                        }
                    }
                    
                    TypeExpr::Parametric(name, args) => {
                        // Concrete application like Option(ℝ)
                        let ty = self.interpret_type_expr(arg_expr, registry)?;
                        self.bindings.insert(param.name.clone(),
                            TypeBinding::ConcreteType(ty));
                    }
                    
                    _ => { /* ... */ }
                }
            }
        }
    }
}
```

**Type Application:**
```rust
fn interpret_type_expr(...) -> Result<Type, String> {
    match type_expr {
        TypeExpr::Parametric(ctor_name, args) => {
            // Check if ctor_name is a bound type constructor variable
            if let Some(TypeBinding::TypeConstructor { name, arity, .. }) = 
                self.bindings.get(ctor_name) 
            {
                // Apply type constructor: F(A) where F=MyList, A=ℝ
                if args.len() != *arity {
                    return Err(format!("Wrong arity"));
                }
                
                let arg_types = args.iter()
                    .map(|a| self.interpret_type_expr(a, registry))
                    .collect::<Result<Vec<_>, _>>()?;
                
                return Ok(Type::Data {
                    type_name: name.clone(),
                    constructor: name.clone(),
                    args: arg_types,
                });
            }
            
            // Regular parametric type lookup
            // ...
        }
    }
}
```

**Pros:**
- Handles Levels 1-3
- Proper separation of concerns
- Extensible architecture

**Cons:**
- Moderate complexity
- Doesn't handle Level 4 (dimension polymorphism)

**Effort:** 3-5 hours

---

### Solution 3: Full Dependent/Higher-Kinded Types (Long-term)

**Goal:** Handle ALL levels including dimension polymorphism

**Requires:**
1. **Kind system:**
   ```rust
   enum Kind {
       Type,                    // * in Haskell
       Nat,                     // Index kind
       Arrow(Box<Kind>, Box<Kind>),  // Type → Type
   }
   ```

2. **Kind inference:** Infer kinds for type parameters
   ```kleis
   structure Mappable(F: Type → Type, A: Type, B: Type) { ... }
   ```

3. **Type-level lambdas:** Represent partially applied constructors
   ```rust
   TypeLambda {
       param: "T",
       body: Type::Data { "MyList", [Var("T")] }
   }
   ```

4. **Existential/universal quantification at type level:**
   ```kleis
   implements VectorAddable(n) {
       // ∀(n: Nat). Vec(n) → Vec(n) → Vec(n)
   }
   ```

**Pros:**
- Complete solution
- Handles all cases
- Theoretically sound

**Cons:**
- Very complex (weeks of work)
- Requires kind checker
- Needs type-level evaluation
- May need dependent type theory

**Effort:** 2-4 weeks (major project)

---

## Recommended Path Forward

### Phase 1: Ship Current State (Now)
- ✅ 417/417 tests passing
- ✅ Built-in types work perfectly
- Document limitation for user types in signatures

### Phase 2: Pragmatic Fix (ADR-022)
- Implement Solution 1 (registry threading)
- 1-2 hours of work
- Handles 90% of use cases
- Tag as v0.7.1

### Phase 3: Proper Type Bindings (ADR-023)
- Implement Solution 2 (type-level bindings)
- 3-5 hours of work
- Handles Functor-style abstractions
- Tag as v0.8.0

### Phase 4: Full System (ADR-024+)
- Implement Solution 3 (kinds + higher-kinded types)
- Major research project
- Requires formal type theory design
- Far future

---

## Testing Strategy

### Level 1 Tests (Simple Types)
```rust
#[test]
fn test_user_type_in_signature() {
    let mut checker = TypeChecker::new();
    
    checker.load_data_types("data Currency = USD | EUR").unwrap();
    checker.load_kleis("
        structure Tradeable(C) {
            operation rate : C → ℝ
        }
        implements Tradeable(Currency) {
            operation rate = builtin_rate
        }
    ").unwrap();
    
    // Check that Currency operations work
    assert!(checker.type_supports_operation("Currency", "rate"));
}
```

### Level 2 Tests (Parametric Types)
```rust
#[test]
fn test_parametric_user_type() {
    let mut checker = TypeChecker::new();
    
    checker.load_data_types("data Option(T) = None | Some(T)").unwrap();
    checker.load_kleis("
        structure Unwrappable(M, T) {
            operation unwrap : M(T) → T
        }
        implements Unwrappable(Option, ℝ) {
            operation unwrap = builtin_unwrap
        }
    ").unwrap();
    
    // Should support unwrap on Option(ℝ)
    assert!(checker.type_supports_operation("Option(ℝ)", "unwrap"));
}
```

### Level 3 Tests (Functor)
```rust
#[test]
fn test_functor_implementation() {
    let mut checker = TypeChecker::new();
    
    checker.load_data_types("data MyList(T) = Nil | Cons(T, MyList(T))").unwrap();
    checker.load_kleis("
        structure Functor(F, A, B) {
            operation map : (A → B) → F(A) → F(B)
        }
        implements Functor(MyList, ℝ, ℂ) {
            operation map = builtin_map
        }
    ").unwrap();
    
    // F(A) should resolve to MyList(ℝ)
    let expr = /* map function application */;
    let ty = checker.check(&expr);
    // Should infer MyList(ℂ)
}
```

---

## Open Questions

### Q1: Type Constructor Naming
When we have:
```kleis
data Option(T) = None | Some(T)
```

Is `Option` itself a type or a type constructor?
- As type constructor: `Option : Type → Type`
- As applied: `Option(ℝ) : Type`

**Decision needed:** How to represent unapplied type constructors?

### Q2: Variant vs Type Constructor
```kleis
data Type = Scalar | Matrix(m: Nat, n: Nat)
```

- `Type` is the data type name
- `Scalar` is a variant (constructor)
- `Matrix` is also a variant

When user writes `Matrix(2, 3)` in a signature, do they mean:
- The variant constructor? (current interpretation)
- A type from the Type data type?

**Decision needed:** Clear syntax for type-level vs value-level

### Q3: Kind Annotations
Do we need explicit kind annotations?

```kleis
structure Functor(F: Type → Type, A: Type, B: Type) {
  operation map : (A → B) → F(A) → F(B)
}
```

Or can we infer kinds from usage?

**Decision needed:** Explicit vs inferred kinds

---

## Related ADRs

- **ADR-020:** Metalanguage for Type Theory (type/value distinction)
- **ADR-021:** Algebraic Data Types (foundation - COMPLETE)
- **ADR-022:** (Proposed) User-Defined Types in Signatures
- **ADR-023:** (Proposed) Higher-Kinded Type Polymorphism
- **ADR-024:** (Proposed) Dependent Type Dimensions

---

## References

### Academic Papers
- Pierce, "Types and Programming Languages" (TAPL), Chapter 29-32
- Harper, "Practical Foundations for Programming Languages" (PFPL), Part XV
- "System F-omega" for higher-kinded types
- "Dependent Types at Work" (Bove & Dybjer)

### Similar Systems
- **Haskell:** Type constructors, kind inference, GADTs
- **OCaml:** Higher-kinded polymorphism (limited)
- **Agda/Idris:** Full dependent types
- **Rust:** Trait-based, no higher-kinded types (yet)

---

## Current Workaround

For now, users must use built-in types in operation signatures:
```kleis
// Works:
structure NumericOps(N) {
  operation abs : N → N
}
implements NumericOps(ℝ) { ... }

// Doesn't work yet:
structure CurrencyOps(C) {
  operation rate : C → ℝ
}
implements CurrencyOps(Currency) {  // Currency interpreted as Scalar!
  operation rate = builtin_rate
}
```

**Workaround:** Use type aliases or wait for ADR-022

---

## Implementation Priority

**High Priority** (Solution 1):
- Simple types in signatures
- Essential for user extensibility
- Quick win

**Medium Priority** (Solution 2):
- Parametric types
- Enables Option, Result, List operations
- Moderate effort, high value

**Low Priority** (Solution 3):
- Higher-kinded polymorphism
- Advanced feature
- Large effort, specialized use cases

---

**Status:** Problem documented, solutions sketched  
**Next:** Decide which solution to implement  
**Current:** ADR-021 complete, system functional for built-in types

