# Type System & Standard Library - Next Steps

**Date:** December 7, 2024  
**Author:** Development Analysis  
**Status:** 🔴 Significant Work Required  
**Priority:** HIGH - Foundation for all type checking

---

## Executive Summary

The Kleis type system has a **solid architectural foundation** (ADR-014 Hindley-Milner, ADR-016 Operations in Structures) but suffers from a critical **disconnect between specification and implementation**. The standard library defines beautiful algebraic structures in Kleis code, but the type inference engine doesn't load or use them. Instead, operations are hardcoded in Rust, violating ADR-016.

**Bottom Line:** ~1-2 weeks of focused work can bridge this gap and make the type system truly ADR-016 compliant.

---

## Current State Assessment

### ✅ What's Working Well

1. **Type Inference Core** (`src/type_inference.rs`)
   - Hindley-Milner implementation: ✅ Solid
   - Unification algorithm: ✅ Working
   - Constraint generation: ✅ Working
   - Test coverage: ✅ Good (4 tests passing)
   - Lines: 550 (well-organized)

2. **Type Context Builder** (`src/type_context.rs`)
   - OperationRegistry: ✅ Implemented
   - Structure parsing: ✅ Working
   - Implements parsing: ✅ Working
   - Operation lookup: ✅ Working
   - Lines: 669 (comprehensive)

3. **Standard Library Definitions**
   - `stdlib/prelude.kleis`: 269 lines
     - Algebraic hierarchy: Semigroup → Monoid → Group → Ring → Field
     - Vector space structures
     - 47+ operations defined
     - 8 mathematical constants
     - 12+ implementations
   - `stdlib/matrices.kleis`: 44 lines
     - Matrix operations following ADR-016
     - Transpose, add, multiply, det, trace
   - **Status:** ✅ Well-designed, follows grammar v0.3

4. **Type Checker** (`src/type_checker.rs`)
   - Bridge between context and inference: ✅ Working
   - Error message generation: ✅ Good
   - Operation support queries: ✅ Working
   - Lines: 258 (clean)

---

## ⚠️ Critical Disconnects

### **Problem 1: Hardcoded Operations (ADR-016 Violation)**

**Severity:** 🔴 HIGH  
**Location:** `src/type_inference.rs` lines 196-380  
**Impact:** Defeats purpose of standard library

**Current State:**
```rust
// type_inference.rs:204-215
fn infer_operation(...) {
    match name {
        "plus" | "minus" => {
            // Hardcoded logic for addition
            let t1 = self.infer(&args[0], context_builder)?;
            let t2 = self.infer(&args[1], context_builder)?;
            self.add_constraint(t1.clone(), t2.clone());
            Ok(t1)
        }
        // ... 150+ more lines of hardcoded operations
    }
}
```

**Operations Hardcoded (Should be in stdlib):**
- ❌ `plus`, `minus` - Should be in `Monoid`/`Group` structure
- ❌ `scalar_divide`, `frac` - Should be in `Field` structure
- ❌ `sqrt` - Should be in `Numeric` structure
- ❌ `sup`, `power` - Should be in `Ring` structure
- ❌ `derivative`, `d_dx`, `partial` - Should be in calculus structures
- ❌ `integral`, `int` - Should be in calculus structures
- ❌ `scalar_multiply`, `times` - Partially delegating but inconsistent

**What Should Happen (ADR-016):**
```rust
fn infer_operation(...) {
    match name {
        // ONLY truly primitive constructors
        "Matrix" | "PMatrix" | "VMatrix" | "BMatrix" => {
            // Matrix literal construction
            self.infer_matrix_constructor(args, context_builder)
        }
        
        // EVERYTHING ELSE delegates to stdlib
        _ => {
            if let Some(builder) = context_builder {
                let arg_types = self.infer_args(args, context_builder)?;
                builder.infer_operation_type(name, &arg_types)
            } else {
                Err("No type context available".to_string())
            }
        }
    }
}
```

**Work Required:**
1. Move operations to `stdlib/prelude.kleis` implementations
2. Refactor `infer_operation()` to delegate by default
3. Keep ONLY matrix literal constructors as special cases
4. Update tests to use context_builder
5. **Estimated Time:** 2-3 days

---

### **Problem 2: Standard Library Not Loaded**

**Severity:** 🔴 HIGH  
**Location:** `src/type_checker.rs`, `src/type_context.rs`  
**Impact:** All stdlib definitions are ignored at runtime

**Current State:**
- `stdlib/prelude.kleis` exists ✅
- `stdlib/matrices.kleis` exists ✅
- `TypeContextBuilder::from_program()` can parse Kleis ✅
- **BUT:** No code loads stdlib on startup ❌

**What's Missing:**
```rust
// type_checker.rs currently has:
impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context_builder: TypeContextBuilder::new(), // Empty!
            inference: TypeInference::new(),
        }
    }
}

// Should have:
impl TypeChecker {
    pub fn with_stdlib() -> Result<Self, String> {
        let mut checker = Self::new();
        
        // Load prelude (always)
        let prelude = include_str!("../stdlib/prelude.kleis");
        checker.load_kleis(prelude)?;
        
        // Load matrices (always)
        let matrices = include_str!("../stdlib/matrices.kleis");
        checker.load_kleis(matrices)?;
        
        Ok(checker)
    }
    
    fn load_kleis(&mut self, code: &str) -> Result<(), String> {
        let program = parse_kleis_program(code)?;
        let new_builder = TypeContextBuilder::from_program(program)?;
        
        // Merge with existing context
        self.context_builder.merge(new_builder)?;
        Ok(())
    }
}
```

**Missing Functionality:**
1. ❌ Stdlib loading mechanism
2. ❌ Context merging (for incremental loading)
3. ❌ Error handling for parse failures
4. ❌ Tests for stdlib initialization

**Work Required:**
1. Add `TypeChecker::with_stdlib()` constructor
2. Implement `TypeContextBuilder::merge()` for combining contexts
3. Add `load_kleis()` method for incremental loading
4. Update all type checker creation to use `with_stdlib()`
5. Add tests verifying stdlib loads correctly
6. Handle parse errors gracefully
7. **Estimated Time:** 1-2 days

---

### **Problem 3: Parser Coverage Gap**

**Severity:** 🟡 MEDIUM (blocking advanced features)  
**Location:** `src/kleis_parser.rs`  
**Impact:** Can't parse full Kleis v0.3 syntax

**From Session 2024-12-06 Analysis:**
> Current parser: ~30% of Kleis v0.3 grammar

**Can Parse (Working):**
- ✅ Structure definitions
- ✅ Implements blocks
- ✅ Operation declarations
- ✅ Type expressions: `ℝ → ℝ`, `Set(T)`, `Matrix(m, n)`
- ✅ Function calls: `abs(x)`, `Res(φ, ω)`
- ✅ Basic expressions

**Cannot Parse (Needed for Full Stdlib):**
- ❌ `extends` clause (syntax recognized but semantics not implemented)
- ❌ Unicode operators: `⟨·,·⟩`, `(⊗)`, `(∘)`
- ❌ Integral syntax: `∫`, with bounds and differentials
- ❌ Universal quantifiers: `∀` in axiom definitions
- ❌ Function definitions with params: `define f(x, y) = expr`
- ❌ Subscripts/superscripts: `aᵢⱼ`, `x₁`
- ❌ Lambda expressions: `λx. expr`
- ❌ Sigma/Pi notation: `Σᵢ`, `∏ᵢ`

**Impact on Current Stdlib:**
- `prelude.kleis` uses some unsupported syntax (quantifiers, operators)
- Parser skips or approximates these parts
- ~60% of prelude.kleis parses correctly
- ~40% needs parser extensions

**Work Required (Not Immediate):**
1. **Phase 2A (2-3 weeks):** Extend parser to ~80% coverage
   - Extends clause semantics
   - Function definitions with parameters
   - Universal quantifiers for axioms
   - Unicode operators
   - Calculus syntax (∫, ∂, ∇)
2. **Phase 2B (1 week):** Update stdlib to use full syntax
3. **Phase 2C (2-3 days):** Test full stdlib loading
4. **Estimated Time:** 3-4 weeks total

---

### **Problem 4: Missing Runtime Implementations**

**Severity:** 🟢 LOW (not needed for type checking)  
**Location:** All `builtin_*` references  
**Impact:** Can't execute expressions, only type-check them

**Current State:**
```kleis
// stdlib/prelude.kleis:131
implements Field(ℝ) {
    operation (+) = builtin_add  // ← Function doesn't exist!
    operation (×) = builtin_mul  // ← Function doesn't exist!
    operation negate(x) = -x     // ← Can't evaluate!
}
```

**What's Missing:**
- Actual Rust implementations of all `builtin_*` functions
- Interpreter to execute Kleis expressions
- Or codegen to compile to executable form
- Evaluation engine

**Why Low Priority:**
- Type checking works without runtime
- Can infer types from signatures alone
- Execution is separate concern from type inference
- Can be added incrementally

**Work Required (Future):**
1. Choose architecture: Interpreter vs Codegen vs Hybrid
2. Implement builtin functions in Rust
3. Add evaluation engine
4. Test with example expressions
5. **Estimated Time:** 1-2 weeks (but defer to Phase 3)

---

## Priority Roadmap

### 🎯 **Phase 1: Connect What Exists** (1-2 weeks)

**Goal:** Make stdlib work with current type system  
**Success Criteria:** Type checker uses stdlib definitions, not hardcoded logic

#### **Task 1.1: Load Stdlib on Startup**
**Time:** 2 days  
**Files:** `src/type_checker.rs`, `src/type_context.rs`

**Steps:**
1. Implement `TypeChecker::with_stdlib()` constructor
2. Add `include_str!()` for `stdlib/prelude.kleis`
3. Add `include_str!()` for `stdlib/matrices.kleis`
4. Parse both files using `parse_kleis_program()`
5. Build `TypeContextBuilder` from parsed programs
6. Handle parse errors gracefully (log warnings, continue)
7. Update all type checker instantiation to use `with_stdlib()`

**Code to Write:**
```rust
// src/type_checker.rs
impl TypeChecker {
    pub fn with_stdlib() -> Result<Self, String> { /* ... */ }
    fn load_kleis(&mut self, code: &str) -> Result<(), String> { /* ... */ }
}

// src/type_context.rs
impl TypeContextBuilder {
    pub fn merge(&mut self, other: TypeContextBuilder) -> Result<(), String> {
        // Merge operation registries
        // Merge structure definitions
        // Merge implements
    }
}
```

**Tests to Add:**
```rust
#[test]
fn test_stdlib_loads_successfully() {
    let checker = TypeChecker::with_stdlib().unwrap();
    // Verify operations from prelude are available
    assert!(checker.types_supporting("+").len() > 0);
}

#[test]
fn test_matrix_operations_from_stdlib() {
    let checker = TypeChecker::with_stdlib().unwrap();
    // Verify matrix operations loaded
    assert!(checker.types_supporting("transpose").contains(&"Matrix"));
}
```

---

#### **Task 1.2: Reduce Hardcoding in Type Inference**
**Time:** 2-3 days  
**Files:** `src/type_inference.rs`, `stdlib/prelude.kleis`

**Steps:**
1. Move arithmetic operations to `stdlib/prelude.kleis`:
   - Add `Arithmetic` structure with `(+)`, `(-)`, `(×)`, `(/)`
   - Implement for `ℝ`, `ℂ`, `Vector(n)`, `Matrix(m,n)`
2. Move calculus operations to `stdlib/prelude.kleis`:
   - Add `Differentiable` structure
   - Add `Integrable` structure
3. Refactor `infer_operation()` to delegate by default:
   - Keep ONLY matrix literal constructors as special cases
   - All other operations → `context_builder.infer_operation_type()`
4. Update tests to pass `context_builder`

**Simplified `infer_operation()`:**
```rust
fn infer_operation(
    &mut self,
    name: &str,
    args: &[Expression],
    context_builder: Option<&crate::type_context::TypeContextBuilder>,
) -> Result<Type, String> {
    // ONLY special case: Matrix constructors (they're literals, not operations)
    match name {
        "Matrix" | "PMatrix" | "VMatrix" | "BMatrix" => {
            self.infer_matrix_constructor(name, args, context_builder)
        }
        
        // EVERYTHING ELSE: Delegate to context_builder (ADR-016!)
        _ => {
            if let Some(builder) = context_builder {
                // Infer argument types
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|arg| self.infer(arg, context_builder))
                    .collect::<Result<Vec<_>, _>>()?;
                
                // Query registry for operation type
                builder.infer_operation_type(name, &arg_types)
            } else {
                // Fallback: No context, return fresh variable
                for arg in args {
                    self.infer(arg, context_builder)?;
                }
                Ok(self.context.fresh_var())
            }
        }
    }
}
```

**Lines to Remove from `type_inference.rs`:**
- Lines 204-289: Hardcoded `plus`, `minus`, `divide`, `sqrt`, `power`, `derivative`, `integral`
- ~80 lines eliminated
- Simpler, cleaner, ADR-016 compliant

---

#### **Task 1.3: Expand TypeContextBuilder Operation Support**
**Time:** 1-2 days  
**Files:** `src/type_context.rs`

**Steps:**
1. Expand `infer_operation_type()` to handle more operations
2. Add support for arithmetic operations from prelude
3. Add support for calculus operations
4. Improve error messages when operation not found
5. Add operation suggestion system

**Current `infer_operation_type()` handles:**
- ✅ Matrix constructors (legacy)
- ✅ `transpose`
- ✅ `add` (with dimension checking)
- ✅ `multiply` (with dimension checking)
- ✅ `det`, `determinant`
- ✅ `trace`
- ❌ Arithmetic: `plus`, `minus`, `times`, `divide`
- ❌ Calculus: `derivative`, `integral`, `partial`
- ❌ Numeric: `sqrt`, `abs`, `floor`, `ceil`

**Extend to:**
```rust
pub fn infer_operation_type(&self, op_name: &str, arg_types: &[Type]) -> Result<Type, String> {
    // Try to find operation in registry
    if let Some(structure_name) = self.registry.structure_for_operation(op_name) {
        // Get structure definition
        let structure = self.get_structure(&structure_name)
            .ok_or_else(|| format!("Structure '{}' not found", structure_name))?;
        
        // Use signature interpreter (ADR-016!)
        let mut interpreter = SignatureInterpreter::new();
        interpreter.interpret_signature(structure, op_name, arg_types)
    } else {
        // Operation not in registry - helpful error
        Err(format!(
            "Unknown operation: '{}'\n\
             Hint: Define it in a structure or import the appropriate library",
            op_name
        ))
    }
}
```

---

#### **Task 1.4: Test End-to-End**
**Time:** 1 day  
**Files:** New test file `tests/type_system_integration_tests.rs`

**Tests to Write:**
1. **Stdlib operations work**
   ```rust
   #[test]
   fn test_addition_from_stdlib() {
       let mut checker = TypeChecker::with_stdlib().unwrap();
       let expr = parse_kleis("a + b").unwrap();
       let result = checker.check(&expr);
       assert!(matches!(result, TypeCheckResult::Success(_)));
   }
   ```

2. **Matrix operations from stdlib**
   ```rust
   #[test]
   fn test_matrix_transpose_from_stdlib() {
       let mut checker = TypeChecker::with_stdlib().unwrap();
       checker.bind("A", &TypeExpr::Parametric(
           "Matrix".to_string(),
           vec![TypeExpr::Nat(3), TypeExpr::Nat(2)]
       ));
       let expr = parse_kleis("transpose(A)").unwrap();
       let result = checker.check(&expr);
       // Should infer: Matrix(2, 3)
       assert!(matches!(result, TypeCheckResult::Success(Type::Matrix(2, 3))));
   }
   ```

3. **Field operations**
   ```rust
   #[test]
   fn test_field_operations() {
       let mut checker = TypeChecker::with_stdlib().unwrap();
       // Test: (a + b) × (c - d) / e
       let expr = parse_kleis("((a + b) * (c - d)) / e").unwrap();
       let result = checker.check(&expr);
       assert!(matches!(result, TypeCheckResult::Success(_)));
   }
   ```

4. **Operation not supported error**
   ```rust
   #[test]
   fn test_helpful_error_for_unsupported_operation() {
       let mut checker = TypeChecker::with_stdlib().unwrap();
       let expr = parse_kleis("nonexistent_op(x)").unwrap();
       let result = checker.check(&expr);
       assert!(matches!(result, TypeCheckResult::Error { message, suggestion } 
           if message.contains("Unknown operation")));
   }
   ```

---

#### **Task 1.5: Fix Any Issues & Buffer**
**Time:** 1-2 days  
**Goal:** Address unexpected problems, edge cases, test failures

**Likely Issues:**
1. Parse errors in stdlib (handle gracefully)
2. Operation name mismatches between parser and stdlib
3. Type representation mismatches
4. Missing structure definitions
5. Registry edge cases

---

### **Phase 1 Success Criteria**

**When Done:**
- ✅ `TypeChecker::with_stdlib()` loads prelude and matrices
- ✅ Type inference delegates to context_builder by default
- ✅ Less than 20 lines of hardcoded operations (only constructors)
- ✅ Addition, subtraction, multiplication work via stdlib
- ✅ Matrix operations work via stdlib
- ✅ All existing tests pass
- ✅ New integration tests pass (10+ tests)
- ✅ ADR-016 compliance: Operations defined in structures ✓

**Measurable Outcomes:**
- `type_inference.rs` reduced from 550 → ~450 lines
- Type system now extensible via Kleis code
- New operations can be added without touching Rust

---

## 🚀 **Phase 2: Expand Parser** (2-3 weeks)

**Goal:** Support ~80% of Kleis v0.3 grammar  
**Success Criteria:** Can parse `mass_from_residue.kleis` and full `prelude.kleis`

### **Task 2.1: Extends Clause Semantics**
**Time:** 2-3 days

**Current:**
```kleis
structure Hont extends HilbertSpace(Hont) {
    // extends recognized but ignored
}
```

**Needed:**
1. Parse extends clause properly
2. Implement inheritance: child inherits parent operations
3. Override semantics: child can override parent operations
4. Test structure hierarchy

---

### **Task 2.2: Function Definitions**
**Time:** 2-3 days

**Current:**
```kleis
define mass_magnitude = abs(x)  // Works
```

**Needed:**
```kleis
define φ_hat(ω) = fourier_transform(φ, ω)  // Doesn't work yet
define f(x, y) = x + y  // Doesn't work yet
```

**Implementation:**
1. Parse function definition syntax
2. Bind parameters in scope
3. Infer function type: `A → B`
4. Test polymorphic functions

---

### **Task 2.3: Quantifiers and Axioms**
**Time:** 3-4 days

**Needed:**
```kleis
axiom mass_is_residue:
    ∀ (particle : Observable) .
        mass(particle) = abs(Res(φ_hat, resonance_frequency(particle)))
```

**Implementation:**
1. Parse `∀` and `∃` syntax
2. Parse axiom propositions
3. Type-check propositions
4. Store axioms in context
5. (Future: Axiom verification/proof checking)

---

### **Task 2.4: Operator Syntax**
**Time:** 2-3 days

**Needed:**
```kleis
operation ⟨·,·⟩ : H × H → ℂ  // Inner product
operation (⊗) : H × H → H    // Tensor product
```

**Implementation:**
1. Unicode operator lexing
2. Operator precedence table
3. Infix/prefix/postfix handling
4. Test operator parsing

---

### **Task 2.5: Calculus Syntax**
**Time:** 2-3 days

**Needed:**
```kleis
define Π(ψ)(x) = ∫_Hont K(x, m) × ψ(m) dm
define total = ∫₀^∞ f(x) dx
define gradient = ∂f/∂x
```

**Implementation:**
1. Parse integral syntax: `∫`, with bounds
2. Parse differential syntax: `∂/∂x`
3. Parse subscript/superscript
4. Test calculus expressions

---

### **Phase 2 Success Criteria**

**When Done:**
- ✅ Parser covers ~80% of Kleis v0.3 grammar
- ✅ Can parse `mass_from_residue.kleis` fully
- ✅ Can parse full `prelude.kleis` without errors
- ✅ Structure inheritance works
- ✅ Function definitions with parameters work
- ✅ Axioms with quantifiers parse correctly

---

## 📊 **Phase 3: Runtime Execution** (1-2 weeks, future)

**Goal:** Execute Kleis expressions, not just type-check  
**Status:** 🟦 LOW PRIORITY (nice to have, not essential)

### **Options:**

**Option A: Interpreter**
- Pros: Simple, flexible, easy to debug
- Cons: Slower, no optimization
- Time: 1 week

**Option B: Codegen to Rust**
- Pros: Fast, can use Rust ecosystem
- Cons: Complex, compilation overhead
- Time: 2 weeks

**Option C: Hybrid**
- Interpret user code, compile builtins
- Best of both worlds
- Time: 1.5 weeks

### **Not Urgent Because:**
- Type checking is valuable on its own
- Can validate mathematical correctness without execution
- Editor feedback works without runtime
- Can be added incrementally later

---

## 📈 **Timeline Summary**

| Phase | Duration | Priority | Blocking |
|-------|----------|----------|----------|
| **Phase 1: Connect stdlib** | 1-2 weeks | 🔴 HIGH | Everything else |
| Phase 2: Expand parser | 2-3 weeks | 🟡 MEDIUM | Advanced features |
| Phase 3: Runtime | 1-2 weeks | 🟢 LOW | Nothing |

**Critical Path:** Phase 1 → Phase 2  
**Total to Full Grammar:** 3-5 weeks  
**Minimum Viable:** Phase 1 (1-2 weeks)

---

## 🎯 **Recommended Action: Start Phase 1 Now**

### **Week 1 Focus: Load Stdlib**

**Days 1-2:**
- Implement `TypeChecker::with_stdlib()`
- Add `TypeContextBuilder::merge()`
- Test stdlib loading

**Days 3-4:**
- Refactor `infer_operation()` to delegate
- Move operations to stdlib implementations
- Update tests

**Day 5:**
- Integration testing
- Fix issues
- Document changes
- Commit with quality checks

### **Week 2 Focus: Stabilize & Test**

**Days 1-2:**
- Write comprehensive tests
- Test edge cases
- Verify ADR-016 compliance

**Days 3-4:**
- Expand `type_context.rs` operation support
- Improve error messages
- Add operation suggestions

**Day 5:**
- Final testing
- Documentation
- Session summary

---

## 📋 **Success Metrics**

### **Phase 1 Complete When:**

1. **Code Quality:**
   - ✅ All tests pass (including new integration tests)
   - ✅ No linter errors
   - ✅ `cargo fmt` clean
   - ✅ `cargo clippy` clean

2. **Functionality:**
   - ✅ Stdlib loads successfully on startup
   - ✅ Type checker uses stdlib definitions
   - ✅ Matrix operations work via stdlib
   - ✅ Arithmetic operations work via stdlib
   - ✅ Operations extensible via Kleis code

3. **Architecture:**
   - ✅ ADR-016 compliant (operations in structures)
   - ✅ Less than 50 lines of hardcoded operations
   - ✅ All operation logic delegated to context_builder

4. **Testing:**
   - ✅ 10+ new integration tests
   - ✅ All tests pass (279 tests → 290+ tests)
   - ✅ Test coverage maintained

---

## 💡 **Key Insights**

### **Why This Matters**

**Before Phase 1:**
- Type system is ~60% hardcoded, ~40% registry-based
- Stdlib exists but isn't used
- Violates ADR-016 (operations should be in structures)
- Can't extend type system without modifying Rust

**After Phase 1:**
- Type system is ~95% registry-based, ~5% constructors
- Stdlib is loaded and actively used
- Complies with ADR-016
- Can extend type system by editing Kleis files
- New operations don't require Rust changes

### **The Architectural Win**

This isn't just about connecting pieces - it's about **achieving self-hosting for types**:

1. **User Extension:** Users can define custom types and operations in Kleis
2. **Stdlib Evolution:** Stdlib can evolve without touching Rust
3. **Domain-Specific:** Easy to create domain-specific type systems
4. **Educational:** Users can read stdlib to understand type system

**Example Future Use Case:**
```kleis
// user_workspace/finance.kleis
structure Currency(C) {
    operation convert : C → C → Exchange → C
}

implements Currency(USD)
implements Currency(EUR)

// Now type checker knows about currency operations!
// No Rust changes needed!
```

---

## 🎓 **Lessons from Session 2024-12-06**

**From POT_TYPE_CHECKING_REALITY.md:**
> "Can our type system type-check this code NOW? ⚠️ No - about 40% of it"

**Why?** Parser coverage + stdlib not connected

**From ADR016_COMPLIANCE_STATUS.md:**
> "✅ Architecture is sound ⚠️ Implementation has gaps"

**This document addresses those gaps.**

---

## 📝 **Notes & Caveats**

### **Parser Limitations**

Even after Phase 1, parser is still ~30% of full grammar. This means:
- Some stdlib syntax won't parse (quantifiers, operators)
- Will need to simplify stdlib or extend parser
- Phase 2 required for full stdlib support

### **Builtin Functions**

All `builtin_*` references in stdlib are still stubs:
```kleis
operation (+) = builtin_add  // ← Doesn't execute, only type-checks
```

This is **OK for Phase 1** because:
- Type inference only needs signatures, not implementations
- Can type-check without executing
- Runtime is separate concern (Phase 3)

### **Performance**

Loading stdlib on every `TypeChecker` creation:
- Parses ~313 lines of Kleis
- Builds registry with ~47 operations
- Should be fast (<10ms) but measure

**Optimization (if needed):**
- Cache parsed stdlib
- Lazy load optional libraries
- Precompile stdlib to binary format

---

## 🔗 **Related Documents**

**ADRs (Architecture):**
- `docs/adr-014-hindley-milner-type-system.md` - HM algorithm
- `docs/ADR-016-operations-in-structures.md` - Operations should be in structures (THIS IS THE ISSUE)
- `docs/adr-015-text-as-source-of-truth.md` - Kleis code is source of truth

**Session Docs:**
- `docs/session-2024-12-06/POT_TYPE_CHECKING_REALITY.md` - Parser coverage analysis
- `docs/session-2024-12-06/ADR016_COMPLIANCE_STATUS.md` - Compliance status
- `docs/session-2024-12-06/FINAL_HONEST_SUMMARY.md` - End of session summary

**Technical Docs:**
- `stdlib/README.md` - Standard library overview
- `docs/type-system/` - Type system documentation

---

## ✅ **Action Items (This Week)**

### **Immediate (Today):**
1. Review this document
2. Confirm Phase 1 priorities
3. Start Task 1.1 (Load stdlib)

### **This Week:**
1. Complete Task 1.1: Load stdlib (2 days)
2. Complete Task 1.2: Reduce hardcoding (2-3 days)
3. Complete Task 1.3: Expand context builder (1-2 days)
4. Begin Task 1.4: Integration tests (1 day)

### **Next Week:**
1. Complete Task 1.4: Integration tests
2. Complete Task 1.5: Fix issues & buffer
3. Document Phase 1 completion
4. Plan Phase 2

---

## 🎯 **Bottom Line**

**The type system architecture is excellent. The implementation is 80% there. Phase 1 bridges the gap in 1-2 weeks.**

**After Phase 1:**
- ✅ Stdlib connected and working
- ✅ ADR-016 compliant
- ✅ Type system extensible via Kleis
- ✅ Foundation solid for all future work

**Let's do this!** 🚀

---

**Document Status:** ✅ Complete  
**Next Update:** After Phase 1 completion  
**Owner:** Type System Team

