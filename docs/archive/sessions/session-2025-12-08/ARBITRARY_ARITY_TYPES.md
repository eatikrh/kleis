# Arbitrary Arity for User-Defined Types

**Date:** December 8, 2025  
**Context:** User-defined types in signatures (follow-up to ADR-021)  
**Key Insight:** Number of type parameters is ARBITRARY and DATA-DRIVEN

---

## The Insight

User-defined types can have **any number of parameters**, not just the hardcoded 1 or 2:

```kleis
// 0 parameters
data Bool = True | False
data Currency = USD | EUR | GBP

// 1 parameter
data Option(T) = None | Some(T)
data Vector(n: Nat) = Vector(elements: List(ℝ))

// 2 parameters
data Matrix(m: Nat, n: Nat) = Matrix(elements: List(ℝ))
data Result(T, E) = Ok(T) | Err(E)

// 3 parameters!
data Tensor3D(i: Nat, j: Nat, k: Nat) = 
  Tensor3D(elements: List(List(List(ℝ))))

// 4+ parameters!
data Tensor4D(d1: Nat, d2: Nat, d3: Nat, d4: Nat) = 
  Tensor4D(data: NestedList)

// Variable number of dimensions (list parameter!)
data Tensor(dims: List(Nat)) = 
  Tensor(shape: List(Nat), elements: FlatList)

// Mixed parameter types!
data NdArray(shape: List(Nat), dtype: Type, layout: MemoryLayout) =
  NdArray(data: RawBuffer)
```

---

## Why Current Code Breaks

### Current Hardcoded Approach

```rust
TypeExpr::Parametric(name, params) => {
    if name == "Matrix" && params.len() >= 2 {
        // ← ASSUMES exactly 2 parameters!
        let rows = self.eval_param(&params[0])?;
        let cols = self.eval_param(&params[1])?;
        Ok(Type::matrix(rows, cols))
    } else if name == "Vector" && params.len() >= 1 {
        // ← ASSUMES exactly 1 parameter!
        let dim = self.eval_param(&params[0])?;
        Ok(Type::vector(dim))
    } else {
        Err(format!("Unknown parametric type: {}", name))
    }
}
```

**Problems:**
- ❌ Hardcodes arity (2 for Matrix, 1 for Vector)
- ❌ Can't handle 0 parameters (Bool)
- ❌ Can't handle 3+ parameters (Tensor3D)
- ❌ Can't handle variable arity (Tensor with List)
- ❌ Not extensible

### What Happens with Tensor3D

```kleis
data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)

structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
  operation slice : Tensor3D(i, j, k) → Matrix(i, j)
}

implements Tensor3DOps(10, 20, 30) {
  operation slice = builtin_slice
}
```

**Interpretation of `Tensor3D(i, j, k)` fails:**
- Not "Matrix" (doesn't match hardcoded check)
- Not "Vector" (doesn't match hardcoded check)
- Falls to error case: "Unknown parametric type: Tensor3D"

---

## The Generic Solution

### Key Realization

**The DataDef TELLS US the arity!**

```rust
// From registry:
let data_def = registry.get_type("Tensor3D").unwrap();

// Inspect the definition:
data_def.type_params.len()  // → 3 parameters!

// Extract parameter kinds:
data_def.type_params = [
    TypeParam { name: "i", kind: Some("Nat") },
    TypeParam { name: "j", kind: Some("Nat") },
    TypeParam { name: "k", kind: Some("Nat") },
]
```

### Generic Parameter Interpretation

```rust
fn interpret_parametric_type(
    &self,
    name: &str,
    param_exprs: &[TypeExpr],
    registry: &DataTypeRegistry,
) -> Result<Type, String> {
    // Look up the data type definition
    let data_def = registry.get_type(name)
        .ok_or_else(|| format!("Unknown type: {}", name))?;
    
    // Validate arity
    let expected_arity = data_def.type_params.len();
    if param_exprs.len() != expected_arity {
        return Err(format!(
            "Type {} expects {} parameters, got {}",
            name, expected_arity, param_exprs.len()
        ));
    }
    
    // Interpret each parameter based on its kind
    let mut args = Vec::new();
    for (param_def, param_expr) in data_def.type_params.iter().zip(param_exprs) {
        let arg_type = match param_def.kind.as_deref() {
            Some("Nat") => {
                // Dimension parameter - evaluate to usize
                let value = self.eval_param(param_expr)?;
                Type::NatValue(value)
            }
            Some("String") => {
                // String parameter - evaluate to string
                let value = self.eval_string_param(param_expr)?;
                Type::StringValue(value)
            }
            Some("Type") | None => {
                // Type parameter - recursively interpret
                self.interpret_type_expr(param_expr, registry)?
            }
            Some(kind) => {
                return Err(format!("Unsupported kind: {}", kind));
            }
        };
        args.push(arg_type);
    }
    
    Ok(Type::Data {
        type_name: name.clone(),
        constructor: name.clone(),  // Or infer from context?
        args,
    })
}
```

**This is GENERIC!** Works for:
- ✅ Bool (0 params)
- ✅ Vector(3) (1 param)
- ✅ Matrix(2, 3) (2 params)
- ✅ Tensor3D(10, 20, 30) (3 params!)
- ✅ Tensor4D(2, 3, 4, 5) (4 params!)
- ✅ NdArray(shape, dtype) (mixed kinds!)

---

## Examples of Arbitrary Arity

### Physics: Tensors

```kleis
// Rank-2 tensor (matrix)
data Tensor2(m: Nat, n: Nat) = Tensor2(components: Array2D)

// Rank-3 tensor (3D array)
data Tensor3(i: Nat, j: Nat, k: Nat) = 
  Tensor3(components: Array3D)

// Rank-4 tensor (spacetime)
data Tensor4(mu: Nat, nu: Nat, rho: Nat, sigma: Nat) = 
  Tensor4(components: Array4D)

structure TensorOps(i: Nat, j: Nat, k: Nat) {
  operation contract : Tensor3(i, j, k) → Scalar
}
```

### Computer Science: N-ary Trees

```kleis
// Binary tree (2 children)
data BinaryTree(T) = Leaf(T) | Node(T, BinaryTree(T), BinaryTree(T))

// Ternary tree (3 children)
data TernaryTree(T) = 
  Leaf(T) 
  | Node(T, TernaryTree(T), TernaryTree(T), TernaryTree(T))

// N-ary tree (variable children)
data NTree(T, n: Nat) = Leaf(T) | Node(T, children: Vector(n, NTree(T, n)))
```

### Data Structures: Multi-Indexed

```kleis
// 2D indexed map
data Map2D(K1, K2, V) = Map2D(data: HashMap)

// 3D indexed array
data Array3D(i: Nat, j: Nat, k: Nat, T) = 
  Array3D(dims: Triple(Nat), elements: List(T))

structure Indexable3D(i: Nat, j: Nat, k: Nat, T) {
  operation at : Array3D(i, j, k, T) → Nat → Nat → Nat → T
}
```

---

## The Registry-Based Pattern

### Pseudocode for Generic Interpretation

```
function interpret_type(type_expr, registry):
    if type_expr is Named(name):
        1. Check type_bindings for variables (T, A, F)
        2. Check registry for user types (Currency, Option)
        3. Check built-ins (ℝ, Nat, Bool, String)
        4. Error if not found
    
    if type_expr is Parametric(name, params):
        1. Check type_constructor_bindings (F where F=MyList)
           → Apply constructor to params
        
        2. Look up in registry (Option, Tensor3D, MyCustomType)
           → Get DataDef
           → Validate arity: params.len() == data_def.type_params.len()
           → Interpret each param based on its kind:
              - kind="Nat" → eval_param (returns usize) → NatValue(n)
              - kind="String" → eval_string → StringValue(s)
              - kind="Type" → interpret_type (recursive!) → Type
           → Construct Type::Data with interpreted args
        
        3. Fallback to built-ins (Matrix, Vector for backward compat)
           → Eventually remove these!
        
        4. Error if not found
```

**Key insight:** The DataDef contains **metadata** about parameters:
- How many? `data_def.type_params.len()`
- What kinds? `data_def.type_params[i].kind`
- What names? `data_def.type_params[i].name`

**The registry makes arbitrary arity possible!**

---

## Implementation Strategy

### Phase 1: Simple Types (Quick Win)

```rust
// Just handle Named types first
if registry.has_type(name) {
    return Ok(Type::Data { 
        type_name: name.clone(), 
        constructor: name.clone(), 
        args: vec![] 
    });
}
```

**Handles:** Currency, Bool, Unit (0 parameters)  
**Effort:** 30 minutes

---

### Phase 2: Fixed-Arity Parametric (Medium)

```rust
if let Some(data_def) = registry.get_type(name) {
    // Validate arity
    if param_exprs.len() != data_def.type_params.len() {
        return Err(format!("Arity mismatch"));
    }
    
    // Interpret all params (assuming all are "Type" kind)
    let args = param_exprs.iter()
        .map(|p| self.interpret_type_expr(p, registry))
        .collect::<Result<Vec<_>, _>>()?;
    
    return Ok(Type::Data { type_name: name.clone(), constructor: name.clone(), args });
}
```

**Handles:** Option(T), Result(T, E), any number of Type parameters  
**Effort:** 1 hour

---

### Phase 3: Multi-Kind Parameters (Complete)

```rust
if let Some(data_def) = registry.get_type(name) {
    let expected_arity = data_def.type_params.len();
    if param_exprs.len() != expected_arity {
        return Err(format!("Arity mismatch"));
    }
    
    let mut args = Vec::new();
    for (param_def, param_expr) in data_def.type_params.iter().zip(param_exprs) {
        let arg = match param_def.kind.as_deref() {
            Some("Nat") => {
                let n = self.eval_param(param_expr)?;
                Type::NatValue(n)
            }
            Some("String") => {
                let s = self.eval_string_param(param_expr)?;
                Type::StringValue(s)
            }
            Some("Type") | None => {
                self.interpret_type_expr(param_expr, registry)?
            }
            Some(k) => return Err(format!("Unknown kind: {}", k)),
        };
        args.push(arg);
    }
    
    return Ok(Type::Data { type_name: name.clone(), constructor: name.clone(), args });
}
```

**Handles:** 
- ✅ Any number of Nat parameters (Tensor3D, Tensor4D, TensorND)
- ✅ Any number of Type parameters (Map3D(K1, K2, K3, V))
- ✅ Mixed kinds (NdArray(shape: List(Nat), dtype: Type))

**Effort:** 2-3 hours (includes testing)

---

## Concrete Examples

### Example 1: 3D Tensor

```kleis
data Tensor3D(i: Nat, j: Nat, k: Nat) = Tensor3D(...)

structure Tensor3DOps(i: Nat, j: Nat, k: Nat) {
  operation sum : Tensor3D(i, j, k) → ℝ
}

implements Tensor3DOps(10, 20, 30) {
  operation sum = builtin_sum_3d
}
```

**Interpretation of `Tensor3D(i, j, k)`:**
```rust
// Look up in registry:
data_def = registry.get_type("Tensor3D")
data_def.type_params = [
    TypeParam { name: "i", kind: Some("Nat") },
    TypeParam { name: "j", kind: Some("Nat") },
    TypeParam { name: "k", kind: Some("Nat") },
]

// Interpret each parameter:
params = ["i", "j", "k"]  // From TypeExpr

// For each:
self.bindings.get("i") → 10
self.bindings.get("j") → 20
self.bindings.get("k") → 30

// Result:
Type::Data {
    type_name: "Tensor3D",
    constructor: "Tensor3D",
    args: [NatValue(10), NatValue(20), NatValue(30)]
}
```

### Example 2: Variable-Dimension Tensor

```kleis
// General N-dimensional tensor
data Tensor(rank: Nat, dims: List(Nat)) = Tensor(...)

// Rank-3 instance
implements TensorOps(3, [10, 20, 30]) {
  operation sum = builtin_sum
}
```

**Challenge:** `dims: List(Nat)` is a **type-level list**!
- Not just a simple Nat value
- Requires dependent types or refinement types
- Very advanced!

### Example 3: Mixed-Kind Parameters

```kleis
data TypedMatrix(m: Nat, n: Nat, T: Type) = 
  TypedMatrix(elements: Array(m, n, T))

structure Mappable(m: Nat, n: Nat, A: Type, B: Type) {
  operation map : (A → B) → TypedMatrix(m, n, A) → TypedMatrix(m, n, B)
}

implements Mappable(2, 3, ℝ, ℂ) {
  operation map = builtin_matrix_map
}
```

**Interpretation of `TypedMatrix(m, n, A)`:**
```rust
data_def.type_params = [
    TypeParam { name: "m", kind: Some("Nat") },   // ← Dimension
    TypeParam { name: "n", kind: Some("Nat") },   // ← Dimension
    TypeParam { name: "T", kind: Some("Type") },  // ← Type!
]

// Interpret each by kind:
args = [
    Type::NatValue(2),     // m = 2
    Type::NatValue(3),     // n = 3
    Type::scalar(),        // A = ℝ
]

Result: Type::Data { 
    "TypedMatrix", 
    "TypedMatrix", 
    [NatValue(2), NatValue(3), Data { "Scalar", ... }]
}
```

---

## The Generic Algorithm

```
function interpret_parametric_type(name, param_exprs, registry):
    # Step 1: Look up type definition
    data_def = registry.get_type(name)
    if data_def is None:
        return fallback_or_error()
    
    # Step 2: Validate arity
    expected_arity = len(data_def.type_params)
    actual_arity = len(param_exprs)
    if expected_arity != actual_arity:
        error("Type {} expects {} params, got {}", name, expected_arity, actual_arity)
    
    # Step 3: Interpret each parameter based on its kind
    args = []
    for (param_def, param_expr) in zip(data_def.type_params, param_exprs):
        arg = interpret_param_by_kind(param_def.kind, param_expr, registry)
        args.append(arg)
    
    # Step 4: Construct result
    return Type::Data {
        type_name: name,
        constructor: name,  # Or determine from context
        args: args
    }

function interpret_param_by_kind(kind, expr, registry):
    match kind:
        "Nat":
            # Evaluate to concrete number
            n = eval_dimension_expr(expr)
            return Type::NatValue(n)
        
        "String":
            # Evaluate to concrete string
            s = eval_string_expr(expr)
            return Type::StringValue(s)
        
        "Type" or None:
            # Recursively interpret as type
            return interpret_type_expr(expr, registry)
        
        "Type → Type":
            # Type constructor! (Level 3)
            # Need higher-kinded types
            return interpret_type_constructor(expr, registry)
        
        other:
            error("Unknown kind: {}", other)
```

**This algorithm is COMPLETELY GENERIC:**
- ✅ Works for any arity (0, 1, 2, 3, 100...)
- ✅ Works for any mix of kinds (Nat, String, Type)
- ✅ Data-driven (uses DataDef metadata)
- ✅ No hardcoding!

---

## Why This Matters

### Before (Hardcoded)
```rust
// Rust code limits what types can exist
if name == "Matrix" && params.len() == 2 { ... }
else if name == "Vector" && params.len() == 1 { ... }
// Users can't add new parametric types!
```

### After (Data-Driven)
```rust
// Registry tells us what types exist and their arity
let data_def = registry.get_type(name)?;
// Automatically handles ANY arity!
for (param_def, param_expr) in data_def.type_params.iter().zip(param_exprs) {
    // Process by kind
}
```

**Impact:** Users can define types with **arbitrary** structure:
- N-dimensional arrays
- Multi-parameter data structures
- Complex domain models
- No limits from type checker!

---

## Type Kind Metadata

The `TypeParam.kind` field is **essential metadata**:

```kleis
data Tensor3D(
    i: Nat,      // ← kind: Nat (dimension)
    j: Nat,      // ← kind: Nat (dimension)
    k: Nat,      // ← kind: Nat (dimension)
) = Tensor3D(...)

data TypedArray(
    n: Nat,      // ← kind: Nat (size)
    T: Type,     // ← kind: Type (element type)
) = TypedArray(...)

data Functor(
    F: Type → Type,  // ← kind: Type → Type (type constructor!)
    A: Type,         // ← kind: Type
    B: Type,         // ← kind: Type
) = ...
```

Without kind information, we can't interpret parameters correctly!

---

## Design Decisions

### Q1: How to determine constructor name?

```kleis
data Type = Scalar | Matrix(m: Nat, n: Nat)
```

When interpreting `Matrix(2, 3)` in a signature:
- `type_name`: "Type" (the data type)
- `constructor`: "Matrix" (the variant)
- But we used `name` for both above!

**Options:**
1. **Single-variant convention:** If DataDef has 1 variant with same name as type, use it
2. **Context-dependent:** Infer from signature context (complex!)
3. **Explicit syntax:** `Type.Matrix(2, 3)` vs `Matrix(2, 3)`

### Q2: How to handle List parameters?

```kleis
data Tensor(dims: List(Nat)) = Tensor(...)
```

**Problem:** `List(Nat)` is itself a parametric type!
- Need to interpret `List(Nat)` as a kind
- This is **higher-kinded type** territory
- Or: Special-case `List` as a built-in kind?

### Q3: Backward compatibility?

Keep Matrix/Vector hardcoding as fallback?
```rust
if registry.has_type(name) {
    // Generic interpretation
} else if name == "Matrix" {
    // Backward compatibility
}
```

Or remove entirely once types.kleis has them?

---

## Implementation Checklist

For registry-based arbitrary-arity support:

- [ ] Add `data_registry` field to SignatureInterpreter
- [ ] Thread registry through all call sites
- [ ] Update `interpret_type_expr` signature
- [ ] Implement generic arity handling
- [ ] Handle Nat kind (eval to NatValue)
- [ ] Handle Type kind (recursive interpret)
- [ ] Handle String kind (eval to StringValue)
- [ ] Validate arity against DataDef
- [ ] Add error messages for arity mismatches
- [ ] Test with 0-arity (Bool)
- [ ] Test with 1-arity (Vector)
- [ ] Test with 2-arity (Matrix)
- [ ] Test with 3-arity (Tensor3D)
- [ ] Test with 4+ arity
- [ ] Test mixed kinds
- [ ] Update all callers of interpret_type_expr
- [ ] Remove or deprecate Matrix/Vector hardcoding

**Estimated effort:** 3-4 hours for complete implementation

---

## Conclusion

**Your insight is spot-on:** The number of parameters is **arbitrary and data-driven**.

**The weirdness in `interpret_type_expr` exists because:**
- It hardcodes arity (1 for Vector, 2 for Matrix)
- It has no registry access
- It can't handle user-defined types

**Once we fix it:**
- ✅ Arbitrary arity (0, 1, 2, 3, 100...)
- ✅ Kind-driven interpretation
- ✅ Completely generic
- ✅ No more special cases (except backward compat)
- ✅ Users can define ANY parametric type structure

**The fix is:** Registry-based interpretation with kind-driven parameter handling.

---

**Status:** Problem fully understood  
**Solution:** Clear architectural path forward  
**Priority:** High (blocks user-defined parametric types)  
**Complexity:** Medium (3-4 hours with testing)  
**Impact:** Enables arbitrary-arity user-defined types!

