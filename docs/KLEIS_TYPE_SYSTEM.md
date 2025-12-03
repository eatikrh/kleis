# Kleis Type System - Design Specification

## Status
**Draft** - Foundation for evaluation and polymorphic dispatch

## Philosophy

The Kleis type system serves three purposes:

1. **Safety**: Prevent meaningless operations (e.g., matrix + scalar)
2. **Dispatch**: Route polymorphic operators to correct implementations
3. **Documentation**: Make mathematical intent explicit

Following ADR-002, types are **first-class** and **inferrable**, not just annotations.

---

## Type Hierarchy

```rust
pub enum Type {
    // Scalar types
    Scalar,              // Real number (ℝ)
    Complex,             // Complex number (ℂ)
    Integer,             // Integer (ℤ)
    
    // Algebraic structures
    Vector(usize),       // n-dimensional vector
    Matrix(usize, usize), // m×n matrix
    Tensor(Vec<usize>),  // Multi-dimensional tensor
    
    // Function types
    Function(Box<Type>, Box<Type>), // Domain → Codomain
    
    // Field theory
    Field {
        space: SpaceType,  // R, R², R³, R⁴, etc.
        value_type: Box<Type>, // Scalar, Vector, Tensor field
    },
    
    // Solution sets
    Set(Box<Type>),         // Set of values
    MultiValue(Box<Type>),  // Multiple values (for ±)
    
    // Special
    Symbolic,            // Unevaluated expression
    Unknown,             // Type inference in progress
    Error(String),       // Type error
}

pub enum SpaceType {
    Real1D,     // ℝ
    Real2D,     // ℝ²
    Real3D,     // ℝ³
    Real4D,     // ℝ⁴ (spacetime)
    RealND(usize), // ℝⁿ
    Manifold(String), // Named manifold
}
```

---

## Type Inference Rules

### Basic Expressions

```rust
// Constants
infer(Const("42")) → Scalar
infer(Const("3.14")) → Scalar
infer(Const("i")) → Complex

// Objects (from context)
context.bind("x", Scalar)
infer(Object("x")) → Scalar

context.bind("E", Field { space: Real3D, value_type: Vector(3) })
infer(Object("E")) → Field(Real3D, Vector(3))

// Placeholders (unknown until filled)
infer(Placeholder) → Unknown
```

### Arithmetic Operations

```rust
// Addition: requires compatible types
infer(plus(Scalar, Scalar)) → Scalar
infer(plus(Vector(n), Vector(n))) → Vector(n)
infer(plus(Vector(3), Vector(4))) → Error("dimension mismatch")
infer(plus(Matrix(m,n), Matrix(m,n))) → Matrix(m,n)

// Subtraction: same as addition
infer(minus(T, T)) → T  // where T supports subtraction

// Negation (unary minus)
infer(minus(Const(0), T)) → T
```

### Multiplication (Polymorphic!)

```rust
// Scalar multiplication
infer(scalar_multiply(Scalar, Scalar)) → Scalar
infer(scalar_multiply(Scalar, Vector(n))) → Vector(n)
infer(scalar_multiply(Vector(n), Scalar)) → Vector(n)
infer(scalar_multiply(Scalar, Matrix(m,n))) → Matrix(m,n)

// Dot product (same operation, different dispatch!)
infer(scalar_multiply(Vector(n), Vector(n))) → Scalar

// Matrix multiplication
infer(scalar_multiply(Matrix(m,n), Matrix(n,p))) → Matrix(m,p)
infer(scalar_multiply(Matrix(m,n), Matrix(k,p))) → Error("inner dimensions must match")

// Matrix-vector
infer(scalar_multiply(Matrix(m,n), Vector(n))) → Vector(m)
```

**Design Decision**: Use single `scalar_multiply` operation; dispatch based on inferred types.

### Division

```rust
infer(scalar_divide(Scalar, Scalar)) → Scalar
infer(scalar_divide(Vector(n), Scalar)) → Vector(n)
infer(scalar_divide(Scalar, Vector(n))) → Error("cannot divide by vector")
infer(scalar_divide(Matrix(m,n), Scalar)) → Matrix(m,n)
```

### Exponentiation

```rust
infer(sup(Scalar, Integer)) → Scalar
infer(sup(Scalar, Scalar)) → Scalar  // General exponentiation
infer(sup(Matrix(n,n), Integer)) → Matrix(n,n)  // Matrix powers
infer(sup(Matrix(m,n), k)) → Error("only square matrices can be exponentiated")
```

### Calculus Operations

```rust
// Differentiation
infer(d_dt(Scalar, Scalar)) → Scalar  // df/dt
infer(d_dt(Vector(n), Scalar)) → Vector(n)  // dr/dt
infer(d_part(Field(space, T), coord)) → Field(space, T)  // ∂F/∂x

// Integration
infer(int_bounds(Scalar, Scalar, Scalar, var)) → Scalar
infer(int_bounds(Vector(n), Scalar, Scalar, var)) → Vector(n)

// Gradient
infer(grad(Field(Real3D, Scalar))) → Field(Real3D, Vector(3))
infer(grad(Scalar)) → Error("gradient requires field or function of position")
```

### Linear Algebra Operations

```rust
// Dot product (explicit)
infer(dot(Vector(n), Vector(n))) → Scalar
infer(dot(Vector(3), Vector(4))) → Error("dimension mismatch")

// Cross product (3D only)
infer(cross(Vector(3), Vector(3))) → Vector(3)
infer(cross(Vector(n), Vector(m))) → Error("cross product only defined in R³")

// Norm
infer(norm(Vector(n))) → Scalar
infer(norm(Matrix(m,n))) → Scalar  // Frobenius norm

// Determinant
infer(det(Matrix(n,n))) → Scalar
infer(det(Matrix(m,n))) → Error("determinant requires square matrix")
```

### Multi-Valued Operations

```rust
// Plus-minus
infer(plus_minus(Scalar, Scalar)) → MultiValue(Scalar)
infer(plus_minus(T, T)) → MultiValue(T)

// In equation context
infer(equals(T, MultiValue(T))) → Set(T)  // Solution set
```

---

## Type Checking Algorithm

### Phase 1: Bottom-Up Inference

```rust
fn infer_type(expr: &Expression, context: &Context) -> Result<Type, TypeError> {
    match expr {
        Expression::Const(_) => Ok(Type::Scalar),
        
        Expression::Object(name) => {
            context.get_type(name)
                .ok_or(TypeError::UnboundSymbol(name.clone()))
        }
        
        Expression::Placeholder { hint, .. } => {
            // Try to infer from hint or context
            Ok(Type::Unknown)
        }
        
        Expression::Operation { name, args } => {
            // Infer argument types
            let arg_types: Vec<Type> = args.iter()
                .map(|arg| infer_type(arg, context))
                .collect::<Result<_, _>>()?;
            
            // Dispatch based on operation + types
            infer_operation_type(name, &arg_types)
        }
    }
}
```

### Phase 2: Constraint Propagation

```rust
fn unify(t1: Type, t2: Type) -> Result<Type, TypeError> {
    match (t1, t2) {
        (Type::Unknown, t) | (t, Type::Unknown) => Ok(t),
        (Type::Scalar, Type::Scalar) => Ok(Type::Scalar),
        (Type::Vector(n1), Type::Vector(n2)) if n1 == n2 => Ok(Type::Vector(n1)),
        (Type::Matrix(m1,n1), Type::Matrix(m2,n2)) if m1==m2 && n1==n2 => Ok(Type::Matrix(m1,n1)),
        _ => Err(TypeError::IncompatibleTypes(t1, t2))
    }
}
```

### Phase 3: Polymorphic Dispatch

```rust
fn infer_operation_type(op: &str, arg_types: &[Type]) -> Result<Type, TypeError> {
    match (op, arg_types) {
        ("scalar_multiply", [Type::Scalar, Type::Scalar]) => Ok(Type::Scalar),
        ("scalar_multiply", [Type::Scalar, Type::Vector(n)]) => Ok(Type::Vector(*n)),
        ("scalar_multiply", [Type::Vector(n), Type::Scalar]) => Ok(Type::Vector(*n)),
        ("scalar_multiply", [Type::Vector(n1), Type::Vector(n2)]) if n1 == n2 => {
            Ok(Type::Scalar)  // Dot product
        }
        ("scalar_multiply", [Type::Matrix(m,n), Type::Matrix(n2,p)]) if n == n2 => {
            Ok(Type::Matrix(*m, *p))
        }
        
        ("plus", [t1, t2]) if can_add(t1, t2) => unify(t1.clone(), t2.clone()),
        ("minus", [t1, t2]) if can_subtract(t1, t2) => unify(t1.clone(), t2.clone()),
        
        ("sup", [Type::Scalar, Type::Scalar]) => Ok(Type::Scalar),
        ("sup", [Type::Matrix(n, m), Type::Scalar]) if n == m => Ok(Type::Matrix(*n, *m)),
        
        ("grad", [Type::Field { space, value_type: box Type::Scalar }]) => {
            Ok(Type::Field {
                space: space.clone(),
                value_type: Box::new(Type::Vector(space.dimension()))
            })
        }
        
        _ => Err(TypeError::NoMatchingOverload(op.to_string(), arg_types.to_vec()))
    }
}
```

---

## Integration with Evaluation

### Type-Directed Evaluation

```rust
impl Expression {
    pub fn eval(&self, context: &Context) -> Result<Value, EvalError> {
        // Step 1: Infer type
        let expr_type = infer_type(self, context)?;
        
        // Step 2: Type-check
        validate_type(&expr_type)?;
        
        // Step 3: Dispatch evaluation
        eval_with_type(self, &expr_type, context)
    }
}

fn eval_with_type(
    expr: &Expression, 
    typ: &Type, 
    context: &Context
) -> Result<Value, EvalError> {
    match (expr, typ) {
        (Operation { name: "scalar_multiply", args: [a, b] }, Type::Scalar) => {
            // Scalar × Scalar
            let va = eval_with_type(a, &Type::Scalar, context)?;
            let vb = eval_with_type(b, &Type::Scalar, context)?;
            Ok(Value::Scalar(va.as_scalar()? * vb.as_scalar()?))
        }
        
        (Operation { name: "scalar_multiply", args: [a, b] }, Type::Vector(n)) => {
            // Scalar × Vector or Vector × Scalar
            // ... dispatch based on arg types
        }
        
        _ => eval_default(expr, context)
    }
}
```

---

## Context with Types

```rust
pub struct Context {
    bindings: HashMap<String, Value>,
    types: HashMap<String, Type>,
}

impl Context {
    pub fn bind(&mut self, name: &str, value: Value) {
        let inferred_type = value.get_type();
        self.types.insert(name.to_string(), inferred_type);
        self.bindings.insert(name.to_string(), value);
    }
    
    pub fn bind_typed(&mut self, name: &str, typ: Type) {
        // Declare type without value (symbolic)
        self.types.insert(name.to_string(), typ);
    }
    
    pub fn get_type(&self, name: &str) -> Option<Type> {
        self.types.get(name).cloned()
    }
}
```

---

## Examples

### Example 1: Scalar Arithmetic

```rust
// Expression: x² + 2x + 1
let expr = plus(
    plus(
        sup(obj("x"), const_("2")),
        scalar_multiply(const_("2"), obj("x"))
    ),
    const_("1")
);

// Context
let mut ctx = Context::new();
ctx.bind_typed("x", Type::Scalar);

// Type inference
assert_eq!(infer_type(&expr, &ctx)?, Type::Scalar);

// Evaluation
ctx.bind("x", Value::Scalar(3.0));
assert_eq!(expr.eval(&ctx)?, Value::Scalar(16.0));
```

### Example 2: Vector Operations

```rust
// Expression: F = ma
// F: Vector(3), m: Scalar, a: Vector(3)
let expr = equals(
    obj("F"),
    scalar_multiply(obj("m"), obj("a"))
);

let mut ctx = Context::new();
ctx.bind_typed("F", Type::Vector(3));
ctx.bind_typed("m", Type::Scalar);
ctx.bind_typed("a", Type::Vector(3));

// Type check
assert!(typecheck(&expr, &ctx).is_ok());

// Infer RHS type
let rhs_type = infer_type(&args[1], &ctx)?;
assert_eq!(rhs_type, Type::Vector(3));
```

### Example 3: Polymorphic Multiply

```rust
// Same AST node, different semantics based on types!

// Scalar context: 2 × 3 = 6
ctx.bind("a", Value::Scalar(2.0));
ctx.bind("b", Value::Scalar(3.0));
let expr = scalar_multiply(obj("a"), obj("b"));
assert_eq!(expr.eval(&ctx)?, Value::Scalar(6.0));

// Vector context: 2 × [1,2,3] = [2,4,6]
ctx.bind("a", Value::Scalar(2.0));
ctx.bind("b", Value::Vector(vec![1.0, 2.0, 3.0]));
assert_eq!(expr.eval(&ctx)?, Value::Vector(vec![2.0, 4.0, 6.0]));

// Dot product: [1,2] · [3,4] = 11
ctx.bind("a", Value::Vector(vec![1.0, 2.0]));
ctx.bind("b", Value::Vector(vec![3.0, 4.0]));
assert_eq!(expr.eval(&ctx)?, Value::Scalar(11.0));
```

### Example 4: Field Theory

```rust
// Expression: ∇·E = ρ/ε₀ (Gauss's law)
let expr = equals(
    div(obj("E")),  // Divergence of E field
    scalar_divide(obj("rho"), obj("epsilon_0"))
);

let mut ctx = Context::new();
ctx.bind_typed("E", Type::Field {
    space: SpaceType::Real3D,
    value_type: Box::new(Type::Vector(3))
});
ctx.bind_typed("rho", Type::Field {
    space: SpaceType::Real3D,
    value_type: Box::new(Type::Scalar)
});
ctx.bind_typed("epsilon_0", Type::Scalar);

// Type check: both sides should be scalar fields over R³
assert!(typecheck(&expr, &ctx).is_ok());
```

---

## Type Errors

```rust
pub enum TypeError {
    UnboundSymbol(String),
    IncompatibleTypes(Type, Type),
    DimensionMismatch { expected: usize, got: usize },
    NoMatchingOverload(String, Vec<Type>),
    InvalidOperation { op: String, types: Vec<Type>, reason: String },
    CyclicType,
    AmbiguousType,
}
```

---

## Implementation Strategy

### Phase 1: Core Types
1. Implement `Type` enum
2. Add type field to `Context`
3. Implement basic inference for Scalar/Vector/Matrix

### Phase 2: Arithmetic Inference
1. Implement inference rules for `plus`, `minus`, `scalar_multiply`, `scalar_divide`
2. Add dimension checking
3. Test with simple expressions

### Phase 3: Polymorphic Dispatch
1. Extend `scalar_multiply` to handle Vector/Matrix cases
2. Implement type-directed evaluation
3. Test dispatch with same AST, different contexts

### Phase 4: Advanced Types
1. Add Field types
2. Implement calculus operations (grad, div, curl)
3. Add Function types
4. Implement MultiValue for `±`

### Phase 5: Integration
1. Connect type inference to structural editor
2. Show type annotations in UI
3. Highlight type errors in real-time
4. Add type-driven autocomplete

---

## Extensible Type System Architecture

### Design Philosophy

The Kleis type system must support **advanced mathematical structures** beyond basic scalars and vectors:

- **Algebra**: Groups, Rings, Fields (algebraic structures)
- **Category Theory**: Categories, Functors, Natural Transformations, Monads
- **Topology**: Topological Spaces, Manifolds, Fiber Bundles
- **Groupoids**: Higher categorical structures
- **Homotopy Type Theory**: ∞-groupoids, identity types

**Key requirement**: Extensible without modifying core type enum.

### Plugin Architecture

```rust
// Core type system (built-in)
pub enum CoreType {
    Scalar,
    Complex,
    Vector(usize),
    Matrix(usize, usize),
    Tensor(Vec<usize>),
    Field { space: SpaceType, value_type: Box<Type> },
    Function(Box<Type>, Box<Type>),
    Set(Box<Type>),
    MultiValue(Box<Type>),
}

// Extensible wrapper
pub enum Type {
    Core(CoreType),
    Extended(Box<dyn TypeClass>),  // Plugin types
}

// Type class trait
pub trait TypeClass: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, value: &Value) -> bool;
    fn infer_operation(&self, op: &str, arg_types: &[Type]) -> Result<Type, TypeError>;
    fn dispatch_eval(&self, expr: &Expression, context: &Context) -> Result<Value, EvalError>;
}
```

### Example: Group Type

```rust
pub struct GroupType {
    pub element_type: Box<Type>,
    pub operation: String,  // e.g., "plus" for additive group
    pub identity: Expression,
    pub inverse_op: String,
}

impl TypeClass for GroupType {
    fn name(&self) -> &str { "Group" }
    
    fn check(&self, value: &Value) -> bool {
        // Verify group axioms: closure, associativity, identity, inverse
        // (This could be symbolic verification or runtime checking)
        true  // Placeholder
    }
    
    fn infer_operation(&self, op: &str, arg_types: &[Type]) -> Result<Type, TypeError> {
        match op {
            // Group operation: G × G → G
            op if op == self.operation => {
                if arg_types.len() == 2 && arg_types[0] == arg_types[1] {
                    Ok(Type::Extended(Box::new(self.clone())))
                } else {
                    Err(TypeError::InvalidGroupOperation)
                }
            }
            // Inverse: G → G
            op if op == self.inverse_op => {
                if arg_types.len() == 1 {
                    Ok(Type::Extended(Box::new(self.clone())))
                } else {
                    Err(TypeError::InvalidInverse)
                }
            }
            _ => Err(TypeError::NoMatchingOverload(op.to_string(), arg_types.to_vec()))
        }
    }
}

// Usage
let additive_group = GroupType {
    element_type: Box::new(Type::Core(CoreType::Scalar)),
    operation: "plus".to_string(),
    identity: Const("0"),
    inverse_op: "negate".to_string(),
};

context.register_type("AdditiveGroup", Type::Extended(Box::new(additive_group)));
```

### Example: Category Type

```rust
pub struct CategoryType {
    pub objects: Vec<Type>,
    pub morphisms: HashMap<(Type, Type), Type>,  // Hom(A, B)
    pub composition: String,  // Morphism composition operation
    pub identity_morphisms: HashMap<Type, Expression>,
}

impl TypeClass for CategoryType {
    fn name(&self) -> &str { "Category" }
    
    fn infer_operation(&self, op: &str, arg_types: &[Type]) -> Result<Type, TypeError> {
        if op == self.composition && arg_types.len() == 2 {
            // Compose f: A → B with g: B → C to get g∘f: A → C
            // Check composition compatibility
            // Return morphism type
            unimplemented!("Category composition")
        } else {
            Err(TypeError::InvalidCategoryOperation)
        }
    }
}
```

### Example: Fiber Bundle Type

```rust
pub struct FiberBundleType {
    pub base_space: SpaceType,      // Base manifold M
    pub fiber: Box<Type>,            // Fiber F
    pub structure_group: Box<Type>,  // Structure group G
    pub projection: Expression,      // π: E → M
}

impl TypeClass for FiberBundleType {
    fn name(&self) -> &str { "FiberBundle" }
    
    fn infer_operation(&self, op: &str, arg_types: &[Type]) -> Result<Type, TypeError> {
        match op {
            // Projection
            "project" => Ok(Type::Core(CoreType::Field {
                space: self.base_space.clone(),
                value_type: self.fiber.clone()
            })),
            // Parallel transport
            "parallel_transport" => {
                // Transport fiber along curve in base space
                unimplemented!()
            }
            // Connection
            "covariant_derivative" => {
                unimplemented!()
            }
            _ => Err(TypeError::NoMatchingOverload(op.to_string(), arg_types.to_vec()))
        }
    }
}

// Usage: Define gauge field as section of fiber bundle
let gauge_field = FiberBundleType {
    base_space: SpaceType::Real4D,  // Spacetime
    fiber: Box::new(Type::Extended(Box::new(LieGroupType { /* U(1) */ }))),
    structure_group: Box::new(Type::Extended(Box::new(LieGroupType { /* U(1) */ }))),
    projection: /* ... */,
};
```

### Example: Monad Type (Category Theory)

```rust
pub struct MonadType {
    pub base_category: Box<Type>,
    pub functor: String,           // T: C → C
    pub unit: Expression,          // η: Id → T (natural transformation)
    pub multiplication: Expression, // μ: T∘T → T (natural transformation)
}

impl TypeClass for MonadType {
    fn name(&self) -> &str { "Monad" }
    
    fn infer_operation(&self, op: &str, arg_types: &[Type]) -> Result<Type, TypeError> {
        match op {
            // Functor application (fmap)
            "map" if arg_types.len() == 2 => {
                // map: (A → B) → T(A) → T(B)
                unimplemented!()
            }
            // Monad bind (>>=)
            "bind" if arg_types.len() == 2 => {
                // bind: T(A) → (A → T(B)) → T(B)
                unimplemented!()
            }
            _ => Err(TypeError::NoMatchingOverload(op.to_string(), arg_types.to_vec()))
        }
    }
}
```

### Type Registry

```rust
pub struct TypeRegistry {
    builtin_types: HashMap<String, CoreType>,
    extended_types: HashMap<String, Box<dyn TypeClass>>,
}

impl TypeRegistry {
    pub fn register(&mut self, name: &str, typ: Box<dyn TypeClass>) {
        self.extended_types.insert(name.to_string(), typ);
    }
    
    pub fn lookup(&self, name: &str) -> Option<Type> {
        if let Some(core) = self.builtin_types.get(name) {
            Some(Type::Core(core.clone()))
        } else if let Some(ext) = self.extended_types.get(name) {
            Some(Type::Extended(ext.clone_box()))
        } else {
            None
        }
    }
    
    pub fn infer_operation(
        &self, 
        op: &str, 
        arg_types: &[Type]
    ) -> Result<Type, TypeError> {
        // Try core type inference first
        if let Ok(typ) = infer_core_operation(op, arg_types) {
            return Ok(typ);
        }
        
        // Try extended types
        for (_, type_class) in &self.extended_types {
            if let Ok(typ) = type_class.infer_operation(op, arg_types) {
                return Ok(typ);
            }
        }
        
        Err(TypeError::NoMatchingOverload(op.to_string(), arg_types.to_vec()))
    }
}
```

### Axiomatic Type Definitions

For advanced structures, define axioms that must be satisfied:

```rust
pub struct AxiomaticType {
    pub name: String,
    pub base_types: Vec<Type>,
    pub operations: Vec<OperationSignature>,
    pub axioms: Vec<Expression>,  // Equations that must hold
}

// Example: Group axioms
let group_axioms = AxiomaticType {
    name: "Group".to_string(),
    base_types: vec![Type::Core(CoreType::Set(Box::new(Type::Unknown)))],
    operations: vec![
        OperationSignature {
            name: "op".to_string(),
            signature: vec![Type::This, Type::This],
            result: Type::This,
        },
        OperationSignature {
            name: "inverse".to_string(),
            signature: vec![Type::This],
            result: Type::This,
        },
    ],
    axioms: vec![
        // Associativity: (a·b)·c = a·(b·c)
        equals(
            op(op(var("a"), var("b")), var("c")),
            op(var("a"), op(var("b"), var("c")))
        ),
        // Identity: ∃e. ∀a. e·a = a·e = a
        exists(
            var("e"),
            forall(var("a"),
                and(
                    equals(op(var("e"), var("a")), var("a")),
                    equals(op(var("a"), var("e")), var("a"))
                )
            )
        ),
        // Inverse: ∀a. ∃a⁻¹. a·a⁻¹ = a⁻¹·a = e
        forall(var("a"),
            equals(
                op(var("a"), inverse(var("a"))),
                identity()
            )
        ),
    ],
};
```

### Integration with Context

```rust
impl Context {
    pub fn register_type_class(&mut self, name: &str, type_class: Box<dyn TypeClass>) {
        self.type_registry.register(name, type_class);
    }
    
    pub fn define_axiomatic_type(&mut self, axioms: AxiomaticType) {
        // Store axioms for verification
        self.axiomatic_types.insert(axioms.name.clone(), axioms);
    }
    
    pub fn verify_axioms(&self, typ: &Type, value: &Value) -> Result<(), TypeError> {
        // Check if value satisfies type's axioms
        // This could be symbolic or runtime verification
        unimplemented!()
    }
}
```

---

## Future Extensions

### Dependent Types
```rust
// Vector length depends on value
Type::Vector(Box<Expression>)  // Vector of length n where n is computed

// Matrix dimensions from context
Type::Matrix(Box<Expr>, Box<Expr>)
```

### Unit Types
```rust
Type::Quantity {
    base: Box<Type>,
    units: Units,  // kg·m/s², etc.
}
```

### Probabilistic Types
```rust
Type::Random(Box<Type>, Distribution)  // Random variable
```

### Higher Structures
```rust
// ∞-Groupoid (HoTT)
Type::InfinityGroupoid {
    objects: Box<Type>,
    paths: Box<Type>,  // Paths between objects
    higher_paths: Vec<Box<Type>>,  // Paths between paths, etc.
}

// Topos
Type::Topos {
    objects: Vec<Type>,
    morphisms: HashMap<(Type, Type), Type>,
    subobject_classifier: Type,
}
```

---

## Connection to ADR-002

This type system maintains the **evaluation/simplification separation**:

- **Type inference**: Deterministic, structural, no heuristics
- **Type checking**: Validates against declared types
- **Dispatch**: Routes to correct implementation based on types
- **Simplification**: Separate layer, type-preserving transforms

Types are **semantic** (what the math means), not syntactic (how it's written).

---

**Next Steps**: Implement `Type` enum and basic inference in `src/types.rs`

