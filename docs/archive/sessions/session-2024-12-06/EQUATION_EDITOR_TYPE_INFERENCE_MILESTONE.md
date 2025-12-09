# Equation Editor Type Inference - Next Milestone

**Date:** December 6, 2024  
**Status:** Infrastructure Complete, Ready for Integration  
**Goal:** Live type feedback in the visual equation editor

---

## The Vision

```
User edits equation in visual editor:
    ↓
    [x + y]  ← User building expression
    ↓
Type inference runs in background
    ↓
Visual feedback appears:
    🔵 "Type: ℝ + ℝ → ℝ" (all good)
    
User tries invalid operation:
    ↓
    [abs(S)] where S : Set(ℤ)
    ↓
Type checker catches error:
    ↓
Visual feedback:
    🔴 "Error: Set(ℤ) doesn't support abs"
    💡 "Did you mean: card(S)?"
```

**This is the next milestone!**

---

## What We Built Today (Infrastructure)

### ✅ Phase 1: Text Representation (ADR-015)
- Text is source of truth
- Explicit forms: `abs(x)`, `frac(a,b)`
- Git-friendly

**Enables:** Clear canonical forms for equation editor to generate

### ✅ Phase 2: Parser
- Expression parser: `abs(x)`, `a + b`
- Structure parser: `structure Numeric { ... }`
- Implements parser: `implements Numeric(ℝ) { ... }`

**Enables:** Parse stdlib to load type definitions

### ✅ Phase 3: Type Context (ADR-016)
- Operations in structures
- Operation registry
- Query interface: "Which types support abs?"

**Enables:** Know what operations are valid for what types

### ✅ Phase 4: Type Checker Connection
- TypeChecker bridges registry ↔ HM inference
- Error suggestions
- Polymorphic operation support

**Enables:** Type check expressions with helpful errors

---

## What We Have Now

### Complete Pipeline (Backend)

```
.kleis file
    ↓
Parser
    ↓
Program { structures, implements }
    ↓
TypeContextBuilder
    ↓
TypeChecker
    ↓
Type Check Result + Suggestions
```

### Working Tests

```bash
# 25+ tests all passing
cargo test kleis_parser::tests --lib
cargo test type_context::tests --lib
cargo test type_checker::tests --lib

# Complete demos
cargo run --bin test_complete_type_checking
cargo run --bin test_adr016_demo
```

---

## Next Milestone: Equation Editor Integration

### What Needs to Be Built

#### 1. Backend API Endpoint (1-2 days)

**File:** `src/main.rs` (server)

```rust
// POST /api/type_check
// Body: { expression: AST, context: { x: "ℝ", y: "Vector(3)" } }
// Response: { result: "success", type: "ℝ", suggestions: [] }

async fn type_check_endpoint(
    Json(payload): Json<TypeCheckRequest>
) -> Json<TypeCheckResponse> {
    // 1. Load stdlib/core.kleis
    let stdlib = load_stdlib().await?;
    
    // 2. Build type checker
    let mut checker = TypeChecker::from_program(stdlib)?;
    
    // 3. Add user context
    for (var, type_str) in payload.context {
        checker.bind(&var, &parse_type(&type_str)?);
    }
    
    // 4. Type check expression
    let result = checker.check(&payload.expression);
    
    // 5. Return result with suggestions
    Json(TypeCheckResponse::from(result))
}
```

---

#### 2. Frontend Integration (2-3 days)

**File:** Frontend equation editor

```javascript
class EquationEditorWithTypes {
    constructor() {
        this.typeChecker = new TypeCheckerClient();
        this.debounceTimer = null;
    }
    
    onExpressionChange(ast) {
        // Debounce type checking (don't spam server)
        clearTimeout(this.debounceTimer);
        this.debounceTimer = setTimeout(() => {
            this.checkTypes(ast);
        }, 300);  // 300ms delay
    }
    
    async checkTypes(ast) {
        try {
            const result = await this.typeChecker.check({
                expression: ast,
                context: this.getContext()
            });
            
            this.displayTypeFeedback(result);
        } catch (error) {
            console.error('Type check failed:', error);
        }
    }
    
    displayTypeFeedback(result) {
        const indicator = document.getElementById('type-indicator');
        
        if (result.result === 'success') {
            indicator.textContent = `🔵 Type: ${result.type}`;
            indicator.className = 'type-success';
        } else {
            indicator.textContent = `🔴 ${result.message}`;
            indicator.className = 'type-error';
            
            if (result.suggestion) {
                this.showSuggestion(result.suggestion);
            }
        }
    }
    
    showSuggestion(suggestion) {
        const tooltip = document.getElementById('type-tooltip');
        tooltip.textContent = `💡 ${suggestion}`;
        tooltip.style.display = 'block';
    }
}
```

---

#### 3. stdlib/core.kleis File (1 day)

**File:** `stdlib/core.kleis`

```kleis
@library("kleis.core")
@version("1.0.0")

// Numeric operations
structure Numeric(N) {
    operation abs : N → N
    axiom abs_non_negative: ∀ (x : N) . abs(x) ≥ 0
}

implements Numeric(ℝ) {
    operation abs = builtin_abs
}

implements Numeric(ℂ) {
    operation abs = complex_modulus
}

// Set operations
structure Finite(C) {
    operation card : C → ℕ
}

implements Finite(Set(T)) {
    operation card = set_cardinality
}

// Vector operations
structure NormedSpace(V) {
    operation norm : V → ℝ
}

implements NormedSpace(Vector(n)) {
    operation norm = euclidean_norm
}

// Display mode
operation frac : ℝ × ℝ → ℝ
```

---

#### 4. UI Visual Feedback (1-2 days)

**Update equation editor UI:**

```html
<div class="equation-editor">
    <!-- Editor canvas -->
    <div id="editor-canvas">
        [Equation being edited]
    </div>
    
    <!-- Type feedback (NEW!) -->
    <div id="type-panel">
        <div id="type-indicator" class="type-unknown">
            ⚪ Type: checking...
        </div>
        <div id="type-tooltip" style="display:none">
            Suggestions appear here
        </div>
    </div>
</div>

<style>
.type-success { color: #0066cc; }  /* Blue */
.type-error { color: #cc0000; }    /* Red */
.type-warning { color: #cc9900; }  /* Yellow */
.type-unknown { color: #999999; }  /* Gray */
</style>
```

---

### Timeline

| Task | Duration | Depends On |
|------|----------|------------|
| Backend API endpoint | 1-2 days | stdlib/core.kleis |
| stdlib/core.kleis | 1 day | - |
| Frontend integration | 2-3 days | Backend API |
| UI visual feedback | 1-2 days | Frontend integration |
| Testing & polish | 1-2 days | All above |

**Total:** 6-10 days (~1.5-2 weeks)

---

## What Today's Work Enables

### Before Today

❌ No way to parse structure definitions  
❌ No type context for user types  
❌ No connection to equation editor  
❌ Type checking only worked with hardcoded types

### After Today

✅ Can parse structures + implements  
✅ Can build type context from .kleis files  
✅ Can query: "Which types support operation X?"  
✅ Can suggest: "Try card instead of abs"  
✅ Ready to integrate with equation editor!

**Today's work was the FOUNDATION for the milestone!**

---

## The Complete Flow (Milestone)

### Backend

```
User opens equation editor
    ↓
Server loads stdlib/core.kleis
    ↓
Parse structures + implements
    ↓
Build TypeChecker
    ↓
Ready for type check requests
```

### Editor

```
User creates expression: abs(x)
    ↓
Send to backend: { expr: "abs(x)", context: { x: "ℝ" } }
    ↓
Backend type checks
    ↓
Response: { result: "success", type: "ℝ" }
    ↓
Display: 🔵 Type: ℝ
```

### Error Case

```
User creates: abs(S) where S : Set(ℤ)
    ↓
Send to backend
    ↓
TypeChecker: Set(ℤ) doesn't implement Numeric
    ↓
Response: { result: "error", message: "...", suggestion: "Try card(S)" }
    ↓
Display: 🔴 Error + 💡 Suggestion
```

---

## What Makes This Milestone Achievable Now

**Before today:** Missing pieces

**After today:**
1. ✅ Parser for structures/implements
2. ✅ Type context builder
3. ✅ Operation registry
4. ✅ Type checker with suggestions
5. ✅ All tested and working

**Only need:**
1. API endpoint (wrap existing code)
2. stdlib file (write structures)
3. Frontend calls (HTTP requests)
4. UI updates (display results)

**All straightforward integration work!**

---

## Success Criteria for Milestone

When complete, user should see:

✅ Type feedback appears as they edit  
✅ Correct operations show green checkmark  
✅ Invalid operations show red error  
✅ Suggestions appear for common mistakes  
✅ Works with user-defined types  
✅ Polymorphic operations work (abs for ℝ and ℂ)  
✅ Response time < 300ms (feels instant)

---

## Next Actions

### Immediate (This Week)

1. **Create stdlib/core.kleis** (1 day)
   - Define Numeric, Finite, NormedSpace structures
   - Implement for ℝ, ℂ, Set, Vector

2. **Add API endpoint** (1-2 days)
   - POST /api/type_check
   - Load stdlib on server start
   - Return type check results

### Next Week

3. **Frontend integration** (2-3 days)
   - Call API on expression change
   - Display type feedback
   - Show suggestions

4. **Testing & polish** (1-2 days)
   - Test with real equations
   - Optimize response time
   - Improve error messages

---

## What Today Achieved

**🎯 Today we built the FOUNDATION:**
- Text representation decided (ADR-015)
- Parser created
- Operations properly designed (ADR-016)
- Type context builder working
- Registry connected to HM inference

**🚀 Next milestone is just INTEGRATION:**
- Backend API (wrap existing code)
- Frontend calls (standard HTTP)
- UI feedback (display results)

**Estimated:** 1.5-2 weeks to live type inference in equation editor! 🎉

---

**Status:** ✅ **Infrastructure complete, ready for editor integration!**  
**Next:** Build API endpoint and integrate with frontend  
**Timeline:** 1.5-2 weeks to working milestone

