# Matrix Type Inference - Implementation Plan

**Date:** December 6, 2024  
**Status:** Ready to Implement  
**Context:** Matrix builder provides perfect testbed for type inference  
**Timeline:** 3-4 days

---

## Why Matrices First?

**Matrices have clear, intuitive type rules:**

```kleis
A: Matrix(2, 3)  // 2 rows, 3 columns
B: Matrix(3, 2)  // 3 rows, 2 columns

✅ A · B → Matrix(2, 2)  // Valid: (2×3) · (3×2) = (2×2)
❌ A + B → Type Error!   // Invalid: can't add different dimensions
❌ det(A) → Type Error!  // Invalid: determinant needs square matrix
✅ det(A · B) → ℝ        // Valid: (2×2) is square
```

**User gets instant feedback:**
- 🔵 Green checkmark when dimensions match
- 🔴 Red error when dimensions incompatible
- 💡 Suggestion: "Try transpose(B) to make dimensions match"

---

## What We Have (Built Today)

### 1. Type System with Matrices ✅

```rust
pub enum Type {
    Scalar,
    Vector(usize),
    Matrix(usize, usize),  // ← Perfect for our use case!
    Var(TypeVar),
    Function(Box<Type>, Box<Type>),
}
```

### 2. Type Checker Infrastructure ✅

```rust
pub struct TypeChecker {
    context_builder: TypeContextBuilder,
    inference: TypeInference,
}

impl TypeChecker {
    pub fn check(&mut self, expr: &Expression) -> TypeCheckResult
    pub fn bind(&mut self, name: &str, type_expr: &TypeExpr)
}
```

### 3. Test Infrastructure ✅

```bash
# Working demos
cargo run --bin test_complete_type_checking
cargo run --bin test_adr016_demo
```

---

## What We Need to Build

### 1. Matrix Type Inference Rules (1 day)

**File:** `src/type_inference.rs`

Add matrix operation type rules:

```rust
fn infer_operation(&mut self, name: &str, args: &[Expression]) -> Result<Type, String> {
    match name {
        // Matrix operations
        "matrix2x2" => {
            // All args must be Scalar, result is Matrix(2, 2)
            for arg in args {
                let ty = self.infer(arg)?;
                self.add_constraint(ty, Type::Scalar);
            }
            Ok(Type::Matrix(2, 2))
        }
        
        "matrix2x3" => Ok(Type::Matrix(2, 3)),
        "matrix3x2" => Ok(Type::Matrix(3, 2)),
        "matrix3x3" => Ok(Type::Matrix(3, 3)),
        
        // Parse dynamic matrix names: matrix4x5 → Matrix(4, 5)
        name if name.starts_with("matrix") => {
            if let Some((rows, cols)) = parse_matrix_dims(name) {
                Ok(Type::Matrix(rows, cols))
            } else {
                Err(format!("Invalid matrix operation: {}", name))
            }
        }
        
        // Matrix multiplication
        "matmul" => {
            let t1 = self.infer(&args[0])?;
            let t2 = self.infer(&args[1])?;
            
            match (t1, t2) {
                (Type::Matrix(m, n), Type::Matrix(p, q)) => {
                    if n != p {
                        return Err(format!(
                            "Matrix multiplication: inner dimensions must match! Got ({}×{}) · ({}×{})",
                            m, n, p, q
                        ));
                    }
                    Ok(Type::Matrix(m, q))
                }
                _ => Err("Matrix multiplication requires two matrices".to_string())
            }
        }
        
        // Matrix addition
        "matadd" => {
            let t1 = self.infer(&args[0])?;
            let t2 = self.infer(&args[1])?;
            
            match (t1, t2) {
                (Type::Matrix(m1, n1), Type::Matrix(m2, n2)) => {
                    if m1 != m2 || n1 != n2 {
                        return Err(format!(
                            "Matrix addition: dimensions must match! Got ({}×{}) + ({}×{})",
                            m1, n1, m2, n2
                        ));
                    }
                    Ok(Type::Matrix(m1, n1))
                }
                _ => Err("Matrix addition requires two matrices".to_string())
            }
        }
        
        // Determinant
        "det" => {
            let t = self.infer(&args[0])?;
            match t {
                Type::Matrix(m, n) if m == n => Ok(Type::Scalar),
                Type::Matrix(m, n) => Err(format!(
                    "Determinant requires square matrix! Got {}×{} matrix",
                    m, n
                )),
                _ => Err("Determinant requires a matrix".to_string())
            }
        }
        
        // Transpose
        "transpose" => {
            let t = self.infer(&args[0])?;
            match t {
                Type::Matrix(m, n) => Ok(Type::Matrix(n, m)),  // Flip dimensions!
                _ => Err("Transpose requires a matrix".to_string())
            }
        }
        
        // ... existing cases ...
    }
}
```

**Helper function:**
```rust
fn parse_matrix_dims(name: &str) -> Option<(usize, usize)> {
    // "matrix2x3" → Some((2, 3))
    // Same logic as renderer
}
```

---

### 2. Backend API Endpoint (1 day)

**File:** `src/bin/server.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TypeCheckRequest {
    expression: Expression,
    context: HashMap<String, String>,  // Variable → Type string
}

#[derive(Serialize)]
struct TypeCheckResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

async fn type_check_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TypeCheckRequest>,
) -> Json<TypeCheckResponse> {
    // Create type checker
    let mut checker = TypeChecker::new();
    
    // Bind context variables
    for (var, type_str) in req.context {
        // Parse type string: "Matrix(2, 3)" → TypeExpr
        if let Ok(type_expr) = parse_type_string(&type_str) {
            checker.bind(&var, &type_expr);
        }
    }
    
    // Check expression
    match checker.check(&req.expression) {
        TypeCheckResult::Success(ty) => {
            Json(TypeCheckResponse {
                success: true,
                type_name: Some(format!("{:?}", ty)),
                error: None,
                suggestion: None,
            })
        }
        TypeCheckResult::Error { message, suggestion } => {
            Json(TypeCheckResponse {
                success: false,
                type_name: None,
                error: Some(message),
                suggestion,
            })
        }
        TypeCheckResult::Polymorphic { .. } => {
            Json(TypeCheckResponse {
                success: true,
                type_name: Some("Polymorphic".to_string()),
                error: None,
                suggestion: None,
            })
        }
    }
}
```

Add route:
```rust
.route("/api/type_check", post(type_check_handler))
```

---

### 3. Frontend Integration (1-2 days)

**File:** `static/index.html`

Add type checking to structural editor:

```javascript
// Add type indicator to structural editor
function renderStructuralEditor() {
    // ... existing rendering ...
    
    // After successful render, check types
    if (currentAST) {
        checkTypesDebounced(currentAST);
    }
}

// Debounced type checking
let typeCheckTimeout = null;
async function checkTypesDebounced(ast) {
    clearTimeout(typeCheckTimeout);
    typeCheckTimeout = setTimeout(async () => {
        await checkTypes(ast);
    }, 500);  // 500ms delay
}

async function checkTypes(ast) {
    try {
        const response = await fetch(`${API_BASE}/type_check`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                expression: ast,
                context: {}  // Empty for now, can add variable types later
            })
        });
        
        const result = await response.json();
        displayTypeInfo(result);
    } catch (error) {
        console.error('Type check failed:', error);
    }
}

function displayTypeInfo(result) {
    const indicator = document.getElementById('type-indicator');
    
    if (!indicator) {
        // Create indicator if it doesn't exist
        createTypeIndicator();
    }
    
    if (result.success) {
        indicator.innerHTML = `<span style="color: #4CAF50;">✓ Type: ${result.type_name}</span>`;
    } else {
        let html = `<span style="color: #f44336;">✗ ${result.error}</span>`;
        if (result.suggestion) {
            html += `<br><span style="color: #FF9800;">💡 ${result.suggestion}</span>`;
        }
        indicator.innerHTML = html;
    }
}

function createTypeIndicator() {
    // Add type indicator to structural controls
    const controls = document.getElementById('structuralControls');
    const indicator = document.createElement('div');
    indicator.id = 'type-indicator';
    indicator.style.cssText = `
        margin-top: 10px;
        padding: 10px;
        border-radius: 4px;
        background: #f5f5f5;
        font-size: 14px;
        font-family: monospace;
    `;
    controls.appendChild(indicator);
}
```

**Add to HTML:**
```html
<div id="structuralControls">
    <!-- Existing controls -->
    
    <!-- Type indicator (NEW!) -->
    <div id="type-indicator" style="margin-top: 10px; padding: 10px; background: #f5f5f5; border-radius: 4px; font-size: 14px; display: none;">
        <span style="color: #999;">⏳ Type checking...</span>
    </div>
</div>
```

---

## Example User Experiences

### Example 1: Creating Compatible Matrices

```
User creates: matrix2x3 filled with numbers
Type indicator: 🔵 Type: Matrix(2, 3)

User creates: matrix3x2 filled with numbers  
Type indicator: 🔵 Type: Matrix(3, 2)

User tries to multiply: matrix2x3 · matrix3x2
Type indicator: 🔵 Type: Matrix(2, 2) ✓
```

### Example 2: Dimension Mismatch Error

```
User creates: matrix2x3
Type indicator: 🔵 Type: Matrix(2, 3)

User tries: matrix2x3 + matrix3x2
Type indicator: 🔴 Dimension mismatch! Cannot add (2×3) + (3×2)
                💡 Did you mean: transpose(matrix3x2)?
```

### Example 3: Determinant on Non-Square

```
User creates: matrix2x3
User applies: det(matrix2x3)
Type indicator: 🔴 Determinant requires square matrix! Got 2×3
                💡 Determinant only works on square matrices (n×n)
```

---

## Implementation Phases

### Phase 1: Basic Matrix Type Inference (Day 1)

**Tasks:**
- [ ] Add matrix type rules to `type_inference.rs`
- [ ] Add dimension parsing helper
- [ ] Write unit tests for matrix typing
- [ ] Test with `cargo test`

**Test cases:**
```rust
#[test]
fn test_matrix_type_inference() {
    let mut inf = TypeInference::new();
    
    // matrix2x3 should infer as Matrix(2, 3)
    let expr = parse("matrix2x3(1,2,3,4,5,6)");
    let ty = inf.infer(&expr).unwrap();
    assert_eq!(ty, Type::Matrix(2, 3));
}

#[test]
fn test_matrix_multiplication_types() {
    // (2×3) · (3×2) → (2×2)
    let expr = parse("matmul(matrix2x3(...), matrix3x2(...))");
    let ty = inf.infer(&expr).unwrap();
    assert_eq!(ty, Type::Matrix(2, 2));
}

#[test]
fn test_dimension_mismatch_error() {
    // (2×3) + (3×2) → Error
    let expr = parse("matadd(matrix2x3(...), matrix3x2(...))");
    let result = inf.infer(&expr);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("dimensions must match"));
}
```

### Phase 2: API Endpoint (Day 2)

**Tasks:**
- [ ] Add `/api/type_check` endpoint to server
- [ ] Implement TypeCheckRequest/Response
- [ ] Parse type strings: "Matrix(2, 3)" → TypeExpr
- [ ] Test with curl
- [ ] Document API

**API Testing:**
```bash
curl -X POST http://localhost:3000/api/type_check \
  -H "Content-Type: application/json" \
  -d '{
    "expression": {
      "Operation": {
        "name": "matrix2x3",
        "args": [...]
      }
    },
    "context": {}
  }'

# Expected response:
{
  "success": true,
  "type_name": "Matrix(2, 3)",
  "error": null,
  "suggestion": null
}
```

### Phase 3: Frontend Integration (Days 3-4)

**Tasks:**
- [ ] Add type indicator UI element
- [ ] Implement debounced type checking
- [ ] Display type feedback (success/error/suggestion)
- [ ] Add CSS styling for type indicator
- [ ] Test with matrix builder
- [ ] Document user workflow

**UI Mockup:**
```
┌─────────────────────────────────┐
│ 🔧 Structural Editor            │
├─────────────────────────────────┤
│                                 │
│   [a  b  c]                     │
│   [d  e  f]                     │
│                                 │
├─────────────────────────────────┤
│ 🔵 Type: Matrix(2, 3)           │  ← NEW!
└─────────────────────────────────┘
```

---

## Detailed Implementation: Day by Day

### Day 1: Matrix Type Rules

**Morning (3-4 hours):**
1. Extend `infer_operation()` with matrix cases
2. Add dimension parsing from operation names
3. Implement matrix multiplication rule
4. Implement matrix addition rule
5. Implement determinant rule
6. Implement transpose rule

**Afternoon (2-3 hours):**
7. Write comprehensive unit tests
8. Test all matrix operations
9. Test error messages
10. Document type rules

**Deliverable:** Matrix typing works in Rust

### Day 2: API Endpoint

**Morning (2-3 hours):**
1. Create TypeCheckRequest/Response structs
2. Implement type_check_handler
3. Add route to server
4. Implement parse_type_string() helper

**Afternoon (2-3 hours):**
5. Test API with curl
6. Test with different matrix sizes
7. Test error cases
8. Write API documentation

**Deliverable:** `/api/type_check` endpoint working

### Day 3: Frontend Integration

**Morning (3-4 hours):**
1. Add type indicator HTML/CSS
2. Implement checkTypes() function
3. Add debouncing
4. Integrate with renderStructuralEditor()

**Afternoon (2-3 hours):**
5. Style type indicator (colors, layout)
6. Test with matrix builder
7. Test various scenarios
8. Polish UX

**Deliverable:** Type feedback shows in UI

### Day 4: Testing & Polish

**Morning (2-3 hours):**
1. End-to-end testing
2. Edge case testing
3. Performance testing (check latency)
4. Fix any bugs

**Afternoon (2-3 hours):**
5. Write user documentation
6. Create demo examples
7. Record screenshots/video
8. Update session README

**Deliverable:** Complete, polished feature

---

## Success Criteria

When complete, user should experience:

✅ **Create 2×3 matrix** → See "Type: Matrix(2, 3)"  
✅ **Create 3×2 matrix** → See "Type: Matrix(3, 2)"  
✅ **Multiply compatible** → See "Type: Matrix(2, 2)" ✓  
✅ **Add incompatible** → See error with dimension mismatch  
✅ **Apply determinant to non-square** → See clear error message  
✅ **Response time** < 500ms (feels instant)  
✅ **No UI blocking** (async, debounced)

---

## Example Demonstrations

### Demo 1: Simple Matrix Creation

```
1. Open Matrix Builder
2. Create 2×3 matrix with placeholders
3. See: 🔵 Type: Matrix(2, 3)
4. Fill first cell with "1"
5. Type indicator stays: Matrix(2, 3)
```

### Demo 2: Matrix Multiplication (Success)

```
1. Create matrix A: 2×3
2. Create matrix B: 3×2
3. Try: A · B
4. See: 🔵 Type: Matrix(2, 2) ✓
5. Result is valid!
```

### Demo 3: Dimension Mismatch (Error)

```
1. Create matrix A: 2×3
2. Create matrix B: 3×2
3. Try: A + B
4. See: 🔴 Cannot add matrices with different dimensions!
        Got (2×3) + (3×2)
        💡 Matrices must have same dimensions for addition
```

### Demo 4: Determinant Error (Helpful)

```
1. Create matrix A: 2×3
2. Try: det(A)
3. See: 🔴 Determinant requires square matrix!
        Got 2×3 (non-square)
        💡 Only n×n matrices have determinants
```

---

## Technical Challenges

### Challenge 1: Dynamic Matrix Names

**Problem:** Operation names are dynamic (`matrix2x3`, `matrix10x5`)

**Solution:** Parse dimensions from string (already implemented in renderer!)

```rust
fn parse_matrix_dims(name: &str) -> Option<(usize, usize)> {
    // Reuse existing logic from render.rs
}
```

### Challenge 2: Type String Parsing

**Problem:** Frontend sends types as strings: `"Matrix(2, 3)"`

**Solution:** Simple parser

```rust
fn parse_type_string(s: &str) -> Result<TypeExpr, String> {
    if s == "ℝ" || s == "Real" {
        return Ok(TypeExpr::Named("ℝ".to_string()));
    }
    
    // Match: Matrix(2, 3)
    if s.starts_with("Matrix(") && s.ends_with(")") {
        let inner = &s[7..s.len()-1];  // Extract "2, 3"
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            let rows = parts[0].trim().parse().ok()?;
            let cols = parts[1].trim().parse().ok()?;
            return Ok(TypeExpr::Parametric(
                "Matrix".to_string(),
                vec![
                    TypeExpr::Const(rows.to_string()),
                    TypeExpr::Const(cols.to_string())
                ]
            ));
        }
    }
    
    Err(format!("Cannot parse type string: {}", s))
}
```

### Challenge 3: Performance

**Problem:** Type checking on every edit might be slow

**Solution:**
- Debounce (500ms delay)
- Only check complete expressions (skip if placeholders unfilled)
- Cache results for unchanged AST

---

## Next Milestone After This

Once matrix type inference works, we can expand to:

1. **Vector operations** - dot product, cross product, norms
2. **Scalar operations** - abs, sqrt, trig functions
3. **Set operations** - cardinality, union, intersection
4. **Function types** - f: ℝ → ℝ, composability
5. **User-defined types** - load from stdlib/core.kleis

**Timeline:** 1-2 weeks per category

---

## Integration with Matrix Builder

**The matrix builder already creates typed ASTs!**

When user creates a 2×3 matrix:
```javascript
{
  Operation: {
    name: "matrix2x3",  // ← This encodes the type!
    args: [...]
  }
}
```

**Type inference just needs to:**
1. See `"matrix2x3"`
2. Return `Type::Matrix(2, 3)`
3. Display to user!

**This is the simplest possible type inference case!**

---

## Recommendation

**Start with matrices because:**
1. ✅ Type rules are intuitive (dimensions)
2. ✅ Errors are clear (mismatch is obvious)
3. ✅ Matrix builder provides perfect testbed
4. ✅ Visual feedback is compelling (users will love it!)
5. ✅ Foundation for more complex types later

**Timeline:** 3-4 days for complete matrix type inference

**After this works:** Expand to vectors, scalars, sets, functions

---

**Next Action:** Implement Day 1 (Matrix Type Rules) → Ready to start coding!


