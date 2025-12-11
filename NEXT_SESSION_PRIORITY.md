# NEXT SESSION: First Priority

**Date Created:** December 10, 2024  
**Priority:** 🔴 CRITICAL - Mathematical Rigor  
**Status:** TODO

---

## Task: Fix Extends Tests with Proper Operation Registration

### The Problem

**Current situation:**
- ✅ Extends clause is parsed and stored in AST
- ✅ Code exists to load parent structures (lines 189-201 in axiom_verifier.rs)
- ⚠️ Z3 test does NOT prove it works empirically
- ⚠️ Test only loads 1 structure instead of 2 (Monoid but not Semigroup parent)

**Root cause:**
```rust
// Current test (WRONG):
registry.register(structure);  // Doesn't register operations!

// Needed:
// Properly register operations so dependency analysis can find structures
```

### What This Means

**Mathematically rigorous proof requires:**
1. Monoid extends Semigroup
2. Verify axiom using `plus` operation  
3. Dependency analysis finds Monoid (via `plus` operation)
4. ensure_structure_loaded("Monoid") called
5. Extends clause triggers ensure_structure_loaded("Semigroup")
6. **Both structures' axioms asserted to Z3**
7. Verification succeeds

**Current test stops at step 3** - operations not registered, so dependency analysis fails.

---

## Current Status

### What's Proven

✅ **Where constraints** - Rigorously proven with Z3  
✅ **Nested structures** - Rigorously proven with Z3  
✅ **Over clause** - Rigorously proven with Z3

### What's NOT Proven

⚠️ **Extends clause** - Code exists but test incomplete  
⚠️ **All dependencies together** - Same issue

**Score: 3 out of 5 features rigorously proven**

Mathematicians have no tolerance for this - we need 5 out of 5!

---

## Solution Approaches

### Option 1: Use TypeContextBuilder (Recommended)

**Problem:** `StructureRegistry::register()` doesn't register operations

**Solution:** Use `TypeContextBuilder` which properly registers everything:

```rust
// Instead of:
let mut registry = StructureRegistry::new();
for item in program.items {
    registry.register(structure)?;  // Incomplete!
}

// Do this:
let ctx_builder = TypeContextBuilder::from_program(program)?;
// This registers structures AND operations properly!

// Then get structures for AxiomVerifier
let registry = ctx_builder.get_structures_registry();  // Need to add this method
```

**Challenge:** `TypeContextBuilder` and `StructureRegistry` are separate types. Need to either:
- Add method to extract registry from builder
- Or refactor to share operation registration logic

---

### Option 2: Add Public Operation Registration API

**Add to StructureRegistry:**

```rust
impl StructureRegistry {
    /// Register an operation (for tests and external use)
    pub fn register_operation(&mut self, structure_name: &str, operation_name: &str) {
        // Store operation -> structure mapping
        self.operation_to_structure.insert(
            operation_name.to_string(),
            structure_name.to_string()
        );
    }
}
```

**Then in tests:**
```rust
registry.register(structure)?;

// Manually register operations
for operation in structure.get_operations() {
    registry.register_operation(&structure.name, &operation.name);
}
```

**Pros:** Minimal changes  
**Cons:** Still manual, error-prone

---

### Option 3: Refactor Test Helper

**Create test utility:**

```rust
// In tests/test_utils.rs or similar:
pub fn register_program_fully(
    registry: &mut StructureRegistry, 
    program: Program
) -> Result<(), String> {
    // Register structures
    for item in &program.items {
        if let TopLevel::StructureDef(s) = item {
            registry.register(s.clone())?;
            
            // Register operations from this structure
            for member in &s.members {
                if let StructureMember::Operation { name, .. } = member {
                    registry.register_operation(&s.name, name);
                }
            }
        }
    }
    Ok(())
}
```

**Pros:** Reusable, clear  
**Cons:** Still some refactoring

---

## Estimated Effort

**Time:** 1-2 hours

**Breakdown:**
- Understand operation registration: 20 min
- Choose approach: 10 min
- Implement: 30-45 min
- Fix all extends tests: 30 min
- Verify all 5 features proven: 15 min

---

## Why This Matters

### Mathematical Integrity

> "Mathematicians have no tolerance for such proofs"

**The user is right.** We cannot claim:

> "Z3 has access to all dependency axioms"

Unless we **rigorously prove** it for ALL dependency types.

Currently: 3/5 proven ≠ "all" ⚠️

### Project Credibility

If we claim formal verification but don't have rigorous tests, we undermine:
- Trust in the system
- Academic credibility  
- Research value
- Production readiness

**Fix this first, before adding more features.**

---

## Success Criteria

### Definition of Done

✅ All 5 Z3 proof tests pass:
1. test_proof_extends_makes_parent_axioms_available ✅
2. test_proof_where_makes_constraint_axioms_available ✅ (already passes)
3. test_proof_nested_makes_axioms_available ✅ (already passes)
4. test_proof_over_makes_field_axioms_available ✅ (already passes)
5. test_proof_all_dependencies_together ✅

✅ All tests show `stats.loaded_structures >= 2` (prove dependency loading)

✅ All tests verify axioms successfully (prove axioms are used)

✅ No weakened assertions or workarounds

---

## Context for Next Session

### What Was Built (Dec 10, 2024)

**Parser features (all working):**
- Custom operators
- Element keyword
- Where clauses
- Over clause
- Unary minus
- Comma-separated quantifiers
- Inline implementations

**Tests:**
- 31 parser tests - all passing
- 419 library tests - all passing
- 5 Z3 proof tests - 3 passing, 2 need fixes

### The Gap

Z3 integration code is correct, but tests don't prove it for extends clause.

**Root cause:** Test setup doesn't register operations, so dependency analysis doesn't work.

**Fix:** Use TypeContextBuilder or add public operation registration API.

---

## Acceptance Criteria

**Before claiming "Z3 has all dependency axioms":**

1. ✅ 5 out of 5 Z3 tests pass (not 3 out of 5)
2. ✅ All tests show dependency structures load
3. ✅ All tests verify axioms without errors
4. ✅ No weakened assertions
5. ✅ Mathematically rigorous

**Then and only then can we claim complete Z3 integration.**

---

## Next Session Checklist

- [ ] Read this file first
- [ ] Choose Option 1, 2, or 3 above
- [ ] Implement operation registration in tests
- [ ] Fix `test_proof_extends_makes_parent_axioms_available`
- [ ] Fix `test_proof_all_dependencies_together`
- [ ] Verify all 5 tests pass with strong assertions
- [ ] Run: `cargo test --test z3_dependency_proof_tests --features axiom-verification`
- [ ] Confirm: All tests show ≥2 structures loaded
- [ ] Update documentation with rigorous proof claim
- [ ] Delete HONEST_ASSESSMENT.md and MATHEMATICAL_RIGOR_ASSESSMENT.md (superseded by working tests)

---

**Priority:** 🔴 FIRST THING NEXT SESSION  
**Reason:** Mathematical integrity cannot be compromised  
**Effort:** 1-2 hours  
**Benefit:** Complete, rigorous proof of Z3 integration

**"Fix the extends tests with significant refactoring" - First priority in next session.**

---

**Created:** December 10, 2024  
**User requirement:** Mathematical rigor, no tolerance for incomplete proofs  
**Status:** Ready for next session

