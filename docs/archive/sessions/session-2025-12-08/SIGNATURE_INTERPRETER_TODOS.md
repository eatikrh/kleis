# SignatureInterpreter TODOs - Analysis and Recommendations

**Date:** December 8, 2025  
**Context:** After implementing user-defined parametric types and HM type variable substitution

---

## Overview

After implementing proper type parameter bindings and HM substitution, there are **4 remaining TODOs** in `src/signature_interpreter.rs`. This document analyzes why each is more difficult than what we just implemented and provides recommendations.

---

## TODO #1: String Parameter Binding Support

**Location:** `src/signature_interpreter.rs:190-195`

```rust
Type::StringValue(_s) => {
    // TODO: Add string binding support for types like:
    // data Tagged(label: String, T) = Tagged(...)
    // Would need: string_bindings: HashMap<String, String>
    // Currently ignored - string-valued type parameters not validated
}
```

### What It Enables

```kleis
data Tagged(label: String, T) = Tagged(...)
data Metric(unit: String, T) = Metric(...)

// Would enable unit-safe physics:
let velocity: Metric("m/s", ℝ) = ...
let force: Metric("N", ℝ) = ...
velocity + force  // ERROR: unit mismatch!
```

### Why It's Difficult

**1. New Type System Dimension**
- Currently: Nat bindings (dimensions) + Type bindings (polymorphism)
- Need: String bindings (labels/units) as third dimension
- Must thread through entire unification pipeline

**2. Unification Logic**
```rust
// Must implement:
- String equality checking: "velocity" ≟ "position"
- Error messages: "String parameter mismatch: expected 'N', got 'm/s'"
- Substitution: If string vars exist, need string substitution map
```

**3. Parser Changes**
```rust
// TypeExpr needs string literals:
TypeExpr::Parametric("Metric", vec![
    TypeExpr::StringLiteral("m/s"),  // ← Doesn't exist in AST!
    TypeExpr::Named("ℝ")
])
```

Parser currently doesn't support string literals in type positions.

**4. Edge Cases**
- String parameters as variables: `structure Generic(s: String) { ... }`
- String variable unification: How do `s1` and `s2` unify?
- Are strings first-class types or just literal values?

### Implementation Steps

1. Add `TypeExpr::StringLiteral(String)` to AST
2. Update parser to handle string literals in type positions
3. Add `string_bindings: HashMap<String, String>` to SignatureInterpreter
4. Implement `bind_or_check_string()` method
5. Add string comparison in `unify_with_expected()`
6. Write comprehensive tests for string parameter unification

### Complexity

**Medium-High** - Requires parser changes + new unification logic + testing

### Priority

**High** - Enables important use cases:
- Unit-safe physics calculations
- Tagged types for domain modeling
- Type-level string documentation

---

## TODO #2: Strict Type Checking for ℝ Unification

**Location:** `src/signature_interpreter.rs:212-219`

```rust
if param_name == "ℝ" || param_name == "Real" {
    match actual_type {
        Type::Data { constructor, .. } if constructor == "Scalar" => Ok(()),
        _ => {
            // TODO: Should error on type mismatch (e.g., Matrix when expecting ℝ)
            // Currently accepts for backward compatibility with existing tests.
            // Future: Replace Ok() with:
            // Err(format!("Type mismatch: expected ℝ, got {:?}", actual_type))
            Ok(())
        }
    }
}
```

### What It Prevents

```kleis
structure Arithmetic(T) {
    operation plus : ℝ → ℝ → ℝ  // Expects Real numbers
}

// Currently ALLOWED (but wrong!):
plus(Matrix(2,2), Matrix(2,2))  // Should error: Matrix ≠ ℝ

// Should ALLOW:
plus(Scalar, Scalar)  // ✓ Correct
```

### Why It's Difficult

**1. Breaking Backward Compatibility**
- Many tests may implicitly rely on lenient type checking
- Changing to strict checking will cause cascade test failures
- Need to audit and fix ALL affected code

**2. Interaction with Type Variables**
```rust
// During inference:
x + 1  // x is Var(0), 1 is Scalar
       // Signature: ℝ → ℝ → ℝ
       
// If strict: Var(0) ≠ ℝ → ERROR ❌
// Should: Substitute first, THEN check ✓
```

**Correct implementation:**
1. Apply all substitutions first
2. THEN check type compatibility
3. Distinguish "not yet known" (Var) from "wrong type" (Matrix)

**3. Policy Decisions Needed**

What should be accepted?

| Case | Accept? | Rationale |
|------|---------|-----------|
| `Var + ℝ` | ✓ | Unknown type - might resolve to ℝ |
| `Nat + ℝ` | ? | Type coercion? Subtyping? |
| `Vector(3) + ℝ` | ✗ | Clear type error |
| `Matrix(2,2) + ℝ` | ✗ | Clear type error |

Need to decide on:
- Type variable handling policy
- Coercion rules (if any)
- Subtyping hierarchy (if any)

**4. Error Message Quality**
```rust
// Bad:
"Type mismatch"

// Good:
"Type error: operation 'plus' expects ℝ → ℝ → ℝ
 Argument 1: Matrix(2,2) is not compatible with ℝ
 Hint: Use scalar operations for matrices or define matrix addition"
```

### Implementation Steps

1. **Phase 1: Analysis**
   - Run tests with strict checking enabled
   - Document all failures
   - Categorize: legitimate errors vs false positives

2. **Phase 2: Policy**
   - Decide on type variable handling
   - Decide on coercion rules
   - Write ADR documenting decisions

3. **Phase 3: Implementation**
   - Implement `apply_substitution()` before checking
   - Add Var special case handling
   - Improve error messages

4. **Phase 4: Migration**
   - Add feature flag for strict mode
   - Fix all affected tests
   - Enable strict mode by default
   - Remove feature flag after stabilization

### Complexity

**High** - Breaks backward compat + needs substitution ordering + policy decisions

### Priority

**Medium** - Important for type safety, but requires careful rollout

---

## TODO #3: Proper Error Handling for Unbound Type Parameters

**Location:** `src/signature_interpreter.rs:472-481`

```rust
// 4. Unbound type parameters (T, N, S, etc.)
// If we reach here, the parameter wasn't bound during unification.
// This happens with signatures without arrows (e.g., "transpose : Matrix(n, m)")
// where old binding logic is used. For backward compatibility, default to Scalar.
// Note: Type variable substitution IS implemented (see bind_or_check_type),
// so Var types DO resolve correctly when unified with concrete types!
_ if name.len() == 1 && name.chars().next().unwrap().is_uppercase() => {
    Ok(Type::scalar())
}
```

### What It Would Catch

```kleis
// LEGITIMATE (should work):
structure Matrix(m: Nat, n: Nat, T) {
    operation transpose : Matrix(n, m, T)  // No arrows, old binding method
}

// ILLEGITIMATE (should error):
structure Broken(T) {
    operation bad : Unknown → T  // T not properly bound
}

structure Malformed {
    operation broken : T  // T not in structure params!
}
```

### Why It's VERY Difficult

**1. Distinguishing Legitimate from Illegitimate Cases**

Multiple reasons for unbound parameters:

| Reason | Legitimate? | Action |
|--------|-------------|--------|
| Old binding method (no arrows) | ✓ | Accept, default to Scalar |
| Type in signature but not structure params | ✗ | Error |
| Type bound to Var, never substituted | ✓ | Accept (polymorphic) |
| Type never encountered during unification | ✗ | Error |

**How to distinguish?**
- Check `structure.type_params` for parameter existence
- Track which params were "seen" during unification
- Differentiate old vs new binding code paths

**2. Multiple Code Paths**

Unbound params can occur in different places:

```rust
// Path 1: interpret_type_expr (direct)
TypeExpr::Named("T") => // Not in type_bindings

// Path 2: bind_from_args (old method)
// Doesn't use type_bindings at all!

// Path 3: After unification
// T bound to Var(0), Var(0) never substituted
```

Each path needs different handling!

**3. Backward Compatibility Nightmare**

```rust
// Current behavior (permissive):
structure Matrix(m: Nat, n: Nat, T) {
    operation transpose : Matrix(n, m, T)
}
// T defaults to Scalar → WORKS

// Strict behavior (error on unbound):
// T never bound → ERROR
// Breaks ALL stdlib matrices!
```

Would need to:
- Update entire stdlib
- Update all user code
- Provide migration guide
- Maintain compatibility mode

**4. Error Message Complexity**

Need context-aware errors:

```rust
// Context 1: Missing from structure params
"Type 'T' used in signature but not declared in structure Generic()"

// Context 2: Not bound during unification  
"Type parameter 'T' from structure Generic(T) was not constrained by arguments.
 Signature: operation id : T → T
 Arguments: []
 Hint: Type parameters must be determinable from argument types"

// Context 3: Polymorphic (actually OK!)
"Type parameter 'T' remains polymorphic (type variable).
 This is valid for generic operations."
```

**5. Interaction with Substitution**

Timeline confusion:

```
Step 1: T encountered, not in type_bindings → default to Scalar?
Step 2: Actually, T gets bound later during unification
Step 3: T bound to Var(0)
Step 4: Var(0) substituted to Scalar
Step 5: Final: T = Scalar ✓

vs

Step 1: T encountered, not in type_bindings → ERROR?
Step 2: Stop! Never gets to unification
```

Need to defer error checking until AFTER unification completes.

### Implementation Steps

**Phase 1: Analysis & Design**
1. Audit all cases where unbound params occur
2. Categorize: legitimate vs errors vs edge cases
3. Design policy for each category
4. Write ADR documenting decisions

**Phase 2: Tracking Infrastructure**
1. Add `seen_params: HashSet<String>` to track encountered params
2. Add `unification_complete: bool` flag
3. Track binding source (unification vs default)

**Phase 3: Validation**
1. After unification, check each structure param:
   - Was it seen?
   - Is it bound?
   - Is it resolved (not just Var)?
2. Generate appropriate error/warning

**Phase 4: Migration**
1. Start with warnings (non-fatal)
2. Update stdlib to be strict-mode compatible
3. Provide migration guide for users
4. Upgrade warnings to errors in next major version

### Complexity

**Very High** - Multiple code paths + backward compat + policy decisions + migration cost

### Priority

**Low** - Current behavior works, strict checking would help but not critical

---

## TODO #4: Remove Matrix/Vector Fallback After ADR-020

**Location:** `src/signature_interpreter.rs:530-548`

```rust
// 2. Fallback to hardcoded Matrix/Vector (backward compatibility)
// TODO(ADR-020): Remove after type/value separation
// Matrix and Vector are NOT in DataTypeRegistry yet because they're
// currently VALUE constructors (Matrix(...) creates values, not types).
if name == "Matrix" && param_exprs.len() >= 2 {
    Ok(Type::matrix(rows, cols))
} else if name == "Vector" && param_exprs.len() >= 1 {
    Ok(Type::vector(dim))
}
```

### Why It Exists

Matrix and Vector are **not in the DataTypeRegistry** because of the type/value distinction:

```kleis
// Current (ADR-020 not implemented):
Matrix(2, 3, [1, 2, 3, 4, 5, 6])  // VALUE constructor - creates a matrix value

// After ADR-020:
Matrix(2, 3, ℝ)  // TYPE constructor - describes a matrix type
Matrix(2, 3, [1, 2, 3, 4, 5, 6])  // VALUE constructor - creates a value
```

From `stdlib/types.kleis` lines 22-26:

> Matrix is NOT included here because the current Matrix(...) syntax is a VALUE constructor (creates matrix values), not a TYPE constructor. This is the type/value distinction from ADR-020.

### Why We Can't Remove It Yet

Without the fallback:
```rust
// User code:
structure MatrixOps(m: Nat, n: Nat) {
    operation transpose : Matrix(m, n) → Matrix(n, m)
}

// Would fail:
self.data_registry.get_type("Matrix")  // Returns None!
// Falls through to: Err("Unknown parametric type: Matrix")
```

**Result:** All matrix operations would break!

### What ADR-020 Needs to Do

**1. Separate Type Constructors from Value Constructors**

```kleis
// TYPE LEVEL (describes types):
type MatrixType(m: Nat, n: Nat, T: Type)

// VALUE LEVEL (creates values):
data MatrixValue(m: Nat, n: Nat, T: Type) = 
  Matrix(rows: m, cols: n, elements: List(T))
```

**2. Update stdlib/types.kleis**

```kleis
data Type = 
  Scalar
  | Matrix(m: Nat, n: Nat, T: Type)  // ← Add type constructor
  | Vector(n: Nat, T: Type)          // ← Add type constructor
  | Complex
  | ...
```

**3. Register in DataTypeRegistry**

The type checker loads `stdlib/types.kleis` and registers all data types, including Matrix and Vector.

**4. Remove the Fallback**

Once Matrix and Vector are in the registry, the generic lookup will find them:

```rust
if let Some(data_def) = self.data_registry.get_type("Matrix") {
    // Now returns Some(...)! ✓
    // Generic code handles it - no special case needed
}
```

### Implementation Steps

**Phase 1: ADR-020 Design**
1. Write ADR-020 documenting type/value separation
2. Design syntax for type constructors vs value constructors
3. Decide on backward compatibility strategy

**Phase 2: Parser Changes**
1. Update parser to distinguish type contexts from value contexts
2. Add `TypeConstructor` and `ValueConstructor` to AST
3. Update type expression parsing

**Phase 3: Registry Updates**
1. Add Matrix and Vector to `stdlib/types.kleis`
2. Update type checker to load type constructors
3. Test that registry lookup works

**Phase 4: Remove Fallback**
1. Delete the hardcoded Matrix/Vector special case
2. Verify all tests still pass
3. Celebrate clean, generic code! 🎉

### Complexity

**Very High** - Requires ADR-020 implementation (major type system change)

### Priority

**Low (blocked on ADR-020)** - This is a cleanup task that unblocks after ADR-020

### Dependencies

- **Blocks:** Nothing - fallback works fine
- **Blocked by:** ADR-020 (type/value separation)
- **Related:** TODO #1 (string bindings) could inform ADR-020 design

### Impact if Not Fixed

**Low** - The fallback is:
- ✓ Well-documented
- ✓ Harmless (no bugs)
- ✓ Localized (only 2 types)
- ✓ Easy to maintain

**But removing it would:**
- ✓ Make code more consistent (no special cases)
- ✓ Demonstrate true generic type system
- ✓ Enable Matrix/Vector to have full ADT features (variants, pattern matching)

---

## Summary: Difficulty Comparison

### What We Just Implemented (Easier)

✅ **User-defined parametric types**
- Clean addition, no breaking changes
- Well-scoped: registry lookup + arity checking
- Clear success criteria

✅ **Type parameter bindings**  
- Parallel to existing `bindings` field
- Same patterns, just for Type instead of usize
- Localized changes

✅ **HM type variable substitution**
- Well-defined algorithm (textbook HM)
- Local to SignatureInterpreter
- Tests immediately proved correctness

### Remaining TODOs (Harder)

| TODO | Complexity | Why Harder |
|------|-----------|-----------|
| **#1 String Bindings** | Medium-High | Parser changes + new unification logic + new type dimension |
| **#2 Strict ℝ Check** | High | Breaking changes + substitution ordering + policy decisions |
| **#3 Error on Unbound** | Very High | Multiple code paths + backward compat + distinguishing legitimate cases |
| **#4 Remove Matrix/Vector Fallback** | Very High | Blocked on ADR-020 (type/value separation) - major architectural change |

---

## Recommendations

### Priority Order

1. **TODO #1 (String bindings)** - Next priority
   - High value: enables unit-safe types, tagged types
   - Clean addition: doesn't break existing code
   - Moderate complexity: doable with parser changes

2. **TODO #2 (Strict ℝ check)** - Medium term
   - Important for type safety
   - Requires careful rollout with feature flags
   - Need policy decisions first

3. **TODO #3 (Error on unbound)** - Long term
   - Lowest priority: current behavior works
   - Very high migration cost
   - Could be done in next major version

4. **TODO #4 (Remove Matrix/Vector fallback)** - After ADR-020
   - Blocked on ADR-020 implementation
   - Will happen naturally when type/value separation is complete
   - Not a priority - fallback is harmless

### Before Starting Any TODO

1. Write an ADR documenting the design
2. Get consensus on policy decisions
3. Create feature flag for gradual rollout
4. Write comprehensive tests first
5. Plan migration path for existing code

---

## Context for Future Work

**What We Accomplished:**
- Arbitrary arity user-defined types ✓
- Type parameter bindings (polymorphism) ✓  
- HM type variable substitution ✓
- Foundation for future improvements ✓

**What's Left:**
- String parameters (new capability)
- Stricter type checking (safety improvement)
- Better error messages (developer experience)
- Remove Matrix/Vector fallback (blocked on ADR-020)

**Key Insight:**
We chose to implement features that were **additive** rather than **breaking**. This allowed rapid progress without extensive migration work. The remaining TODOs involve breaking changes or new type system features, making them inherently more complex.

---

**Document Status:** Analysis complete, ready for future implementation  
**Next Steps:** Choose TODO #1 for next session, write ADR first

