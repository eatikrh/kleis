# Type Context and Operation Registry

**Date:** December 2025  
**Status:** Design Specification  
**Consolidated from:** TYPE_CONTEXT_BOOTSTRAP.md + OPERATION_BASED_TYPE_INFERENCE.md

---

## Table of Contents

1. [The Bootstrap Problem](#the-bootstrap-problem)
2. [Three-Tier Bootstrap Strategy](#three-tier-bootstrap-strategy)
3. [Operation-Based Type Inference](#operation-based-type-inference)
4. [Operation Registry](#operation-registry)
5. [Loading and Initialization](#loading-and-initialization)
6. [Examples](#examples)

---

## The Bootstrap Problem

### The Challenge

```kleis
// User opens editor and types:
a + b

// Questions:
// - How does Kleis know what (+) means?
// - What types support (+)?
// - How do we infer types for a and b?
```

**Solution:** Three-tier context initialization with operation manifests.

---

## Three-Tier Bootstrap Strategy

### Tier 1: Core Types (Hardcoded in Rust)

**Only primitives - ~50 lines in `src/type_inference.rs`**

```rust
impl EditorTypeContext {
    pub fn core() -> Self {
        let mut ctx = EditorTypeContext::new();
        
        // Primitives only
        ctx.register_builtin("Scalar", Type::Scalar);
        ctx.register_builtin("Bool", Type::Bool);
        ctx.register_builtin("String", Type::String);
        ctx.register_builtin("Nat", Type::Nat);
        
        // Type constructors
        ctx.register_constructor("Vector", 1);  // Takes 1 param
        ctx.register_constructor("Matrix", 2);  // Takes 2 params
        ctx.register_constructor("List", 1);
        
        ctx
    }
}
```

### Tier 2: Standard Library (Kleis Code)

**Everything else - `stdlib/prelude.kleis` (~500 lines)**

```kleis
@library("std.prelude")

// Algebraic structures
structure Monoid(M) { ... }
structure Group(G) extends Monoid(G) { ... }
structure Field(F) extends Ring(F) { ... }

// Implementations
implements Field(ℝ)
implements VectorSpace(Vector(n)) over Field(ℝ)

// Operations
operation dot : ∀n. Vector(n) × Vector(n) → ℝ
operation det : ∀n. Matrix(n,n) → ℝ

// Constants
define π : ℝ = 3.14159...
```

**Loaded at server startup (~15ms)**

### Tier 3: User Workspace (Per-Document)

**User-specific types - `workspace/user_types.kleis`**

```kleis
// Mathematical extensions
structure Spinor(n) { ... }

// Physics types
structure Particle { ... }

// Business types
structure PurchaseOrder { ... }
```

**Loaded on demand**

---

## Operation-Based Type Inference

### The Key Question

> "Given `a + b`, what types are valid?"

**Answer:** Query which types define the `+` operation!

### How It Works

#### 1. Structures Declare Operations

```kleis
structure Monoid(M) {
  operation (+) : M × M → M
  element zero : M
  axiom identity: ∀x. zero + x = x
}

// Implementations
implements Monoid(ℝ)
implements Monoid(Vector(n))
implements Monoid(Matrix(m,n))
```

#### 2. Build Operation Registry

```rust
OperationRegistry {
  "+": [
    (ℝ, "Monoid(ℝ)"),
    (Vector(n), "Monoid(Vector(n))"),
    (Matrix(m,n), "Monoid(Matrix(m,n))"),
  ]
}
```

#### 3. Query on Inference

```rust
// Expression: a + b
query_types_supporting("+") → [ℝ, Vector(n), Matrix(m,n)]

// Generate constraint:
a : α where α ∈ {ℝ, Vector(n), Matrix(m,n)}
```

#### 4. Result

```kleis
Type: ∀T. Monoid(T) ⇒ T
Feedback: 🟢 "Polymorphic - works for ℝ, Vector, Matrix"
```

---

## Operation Registry

### Data Structure

```rust
/// Maps operations to types that support them
pub struct OperationRegistry {
    operations: HashMap<String, Vec<OperationSupport>>,
}

pub struct OperationSupport {
    ty: Type,
    structure: String,
    signature: OperationSignature,
}

pub struct OperationSignature {
    inputs: Vec<Type>,
    output: Type,
    constraints: Vec<Constraint>,
}
```

### Querying Operations

```rust
impl OperationRegistry {
    /// Which types support this operation?
    pub fn types_supporting(&self, op: &str) -> Vec<Type> {
        self.operations.get(op)
            .map(|supports| supports.iter().map(|s| s.ty.clone()).collect())
            .unwrap_or_default()
    }
    
    /// Get operation signature for specific type
    pub fn get_signature(&self, op: &str, ty: &Type) 
        -> Option<OperationSignature> {
        self.operations.get(op)?
            .iter()
            .find(|s| s.ty == *ty)
            .map(|s| s.signature.clone())
    }
    
    /// Get all multiplication rules (polymorphic!)
    pub fn get_multiplication_rules(&self) -> Vec<MultiplicationRule> {
        vec![
            // Scalar × Scalar → Scalar
            (Type::Scalar, Type::Scalar, Type::Scalar),
            // Scalar × Vector → Vector
            (Type::Scalar, Type::Vector(n), Type::Vector(n)),
            // Vector × Vector → Scalar (dot)
            (Type::Vector(n), Type::Vector(n), Type::Scalar),
            // Matrix × Matrix
            (Type::Matrix(m,n), Type::Matrix(n,p), Type::Matrix(m,p)),
        ]
    }
}
```

### Building the Registry

```rust
impl OperationRegistry {
    pub fn from_stdlib(defs: &[StructureDef]) -> Self {
        let mut registry = OperationRegistry::new();
        
        // For each structure definition
        for def in defs {
            // For each operation in the structure
            for op in &def.operations {
                // Find implementations of this structure
                for impl in find_implementations(def.name) {
                    // Register: this type supports this operation
                    registry.register(
                        op.name,
                        impl.type_arg,
                        op.signature,
                        def.name
                    );
                }
            }
        }
        
        registry
    }
}
```

---

## Loading and Initialization

### Server Startup Sequence

```rust
#[tokio::main]
async fn main() {
    println!("Initializing Kleis...");
    
    // Step 1: Core types (instant)
    let mut ctx = EditorTypeContext::core();
    println!("✓ Core types loaded");
    
    // Step 2: Load & parse stdlib (15ms)
    let prelude = include_str!("../stdlib/prelude.kleis");
    let defs = parse_kleis_source(prelude)?;
    ctx.load_definitions(defs)?;
    println!("✓ Standard library loaded");
    
    // Step 3: Build operation registry
    ctx.build_operation_registry()?;
    println!("✓ Operation registry built");
    
    // Step 4: Verify stdlib (optional)
    if cfg!(debug_assertions) {
        ctx.verify_all_axioms()?;
        println!("✓ Stdlib axioms verified");
    }
    
    // Store in global
    *GLOBAL_CONTEXT.lock().unwrap() = ctx;
    
    println!("Ready! Context has:");
    println!("  - {} types", ctx.type_count());
    println!("  - {} structures", ctx.structure_count());
    println!("  - {} operations", ctx.operation_count());
    
    start_server().await;
}
```

### Expected Output

```
Initializing Kleis...
✓ Core types loaded
✓ Standard library loaded (12ms)
✓ Operation registry built
✓ Stdlib axioms verified

Ready! Context has:
  - 8 types (ℝ, ℂ, ℤ, ℕ, Vector, Matrix, List, Set)
  - 12 structures (Semigroup, Monoid, Group, Ring, Field, ...)
  - 47 operations (+, ×, ∂, ∫, ∇, dot, cross, det, ...)
  
Server listening on http://localhost:3000
```

---

## Examples

### Example 1: User Types `a + b`

**With no context:**

```
Query: types_supporting("+")
Result: [ℝ, ℂ, Vector(n), Matrix(m,n)]

Inference:
  a : α where α ∈ {ℝ, ℂ, Vector(n), Matrix(m,n)}
  b : α
  Result: α

Feedback: 🟢 "Type: α where Monoid(α) (polymorphic)"
```

**User adds context: `a : Vector(3)`**

```
Constraint: a : Vector(3), b : Vector(3)
Result: Vector(3)

Feedback: 🔵 "Type: Vector(3) + Vector(3) → Vector(3)"
```

### Example 2: Multiplication Ambiguity

**Expression:** `v × w`

```
Query: types_supporting("×")
Result: [ℝ, ℂ, Vector(n), Matrix(m,n)]

But (×) has multiple meanings:
  - Scalar × Scalar → Scalar
  - Vector × Vector → Scalar (dot product)
  - Matrix × Matrix → Matrix

Feedback: 🟡 "Multiple interpretations possible"
```

**User adds context: `v : Vector(3)`**

```
Rules for Vector(3) × ?:
  - Vector(3) × Vector(3) → Scalar (dot)
  - Vector(3) × Scalar → Vector(3) (scale)

Feedback: 🟢 "Type depends on w: Scalar or Vector(3)"
```

### Example 3: Business Type

**stdlib loaded, user defines:**

```kleis
structure PurchaseOrder {
  total : Money
  items : List(LineItem)
  
  supports {
    combine : PurchaseOrder × PurchaseOrder → PurchaseOrder
  }
}

implements Monoid(PurchaseOrder) {
  element zero = emptyOrder
  operation (+) = combine
}
```

**Now registry has:**
```rust
operations["+"].push((PurchaseOrder, "Monoid(PurchaseOrder)"))
```

**User types:** `order1 + order2`

```
Query: types_supporting("+")
Result: [..., PurchaseOrder]  // Now includes business type!

Inference: PurchaseOrder + PurchaseOrder → PurchaseOrder

Feedback: 🔵 "Type: PurchaseOrder"
```

---

## Implementation Summary

### Context Initialization

```rust
pub struct EditorTypeContext {
    /// Tier 1: Hardcoded primitives
    builtin_types: HashMap<String, Type>,
    
    /// Tier 2: Loaded from stdlib
    structures: HashMap<String, StructureDef>,
    implementations: Vec<Implementation>,
    
    /// Tier 3: User workspace
    user_types: HashMap<String, TypeDefinition>,
    
    /// Operation registry (built from above)
    operation_registry: OperationRegistry,
    
    /// Current scope variables
    variables: HashMap<String, Type>,
}
```

### Loading Sequence

1. ✅ Core types (hardcoded)
2. ✅ Parse stdlib/prelude.kleis
3. ✅ Build operation registry from structures
4. ✅ Verify stdlib axioms (optional)
5. ✅ Ready for user input

---

## API Endpoints

### GET `/api/type_context/summary`

**Returns context status**

```json
{
  "ready": true,
  "types_count": 8,
  "structures_count": 12,
  "operations_count": 47,
  "stdlib_loaded": true
}
```

### GET `/api/type_context/types`

**Returns available types**

```json
["ℝ", "ℂ", "ℤ", "ℕ", "Vector", "Matrix", "List"]
```

### GET `/api/type_context/operations?type=Matrix`

**Returns operations for a type**

```json
["+", "-", "×", "det", "trace", "transpose"]
```

### GET `/api/type_context/types?operation=+`

**Returns types supporting an operation**

```json
["ℝ", "ℂ", "Vector(n)", "Matrix(m,n)", "Polynomial"]
```

---

## Benefits

✅ **Self-hosting** - stdlib is Kleis code  
✅ **Queryable** - Operation registry enables type inference  
✅ **Extensible** - Users can add types/operations  
✅ **Fast** - Loaded once at startup (~15ms)  
✅ **Universal** - Works for math AND business types  

---

## Files

**Implementation:**
- `src/type_inference.rs` - Core context + primitives
- `stdlib/prelude.kleis` - Standard library (Kleis code!)
- `stdlib/README.md` - Overview

**Grammar:**
- `docs/grammar/Kleis_v03.g4` - ANTLR4 grammar
- `docs/grammar/kleis_grammar_v03.ebnf` - EBNF grammar

**Documentation:**
- This file - Complete context & operations guide
- `docs/adr-014-hindley-milner-type-system.md` - Architectural decision

---

**This is how Kleis knows what types work with what operations!** 🎯

