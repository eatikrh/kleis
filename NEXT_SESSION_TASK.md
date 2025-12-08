# NEXT SESSION: Implement ADR-021 (Algebraic Data Types)

**Current State:** v0.6.0-adr016-complete (about to be tagged)

**Status:** Ready to implement the dynamic type system! 🚀

---

## What We Accomplished Today (Dec 8, 2024)

### **Phase 1: COMPLETE** ✅
1. Task 1.5 finished (clippy fixes, documentation)
2. **TRUE ADR-016 compliance** achieved:
   - Removed ALL type-specific hardcoding
   - Generic structure validation
   - Zero Type::Matrix or Type::Scalar references in type_context.rs
3. Test coverage expanded: 281 → 288 tests

### **ADR-020 Extended** ✅
- Connected Matrix constructor issue to type/value distinction
- "Practical Application" section added
- Root cause: Type/value conflation

### **ADR-021 Prepared** ✅
- type_inference.rs refactored and documented
- Dead code removed
- Helper functions extracted (generic field inference)
- Vision documented in code comments

### **Implementation Plan Created** ✅
- Complete 11-step plan in ADR021_IMPLEMENTATION_PLAN.md
- Risk assessment
- Timeline (1-2 weeks)
- Rollback strategy

---

## The Vision: What We're Building

### **Current (Hardcoded):**
```rust
pub enum Type {
    Scalar,
    Matrix(usize, usize),  // ← Fixed at compile time
    // Users can't add types!
}
```

### **Target (Dynamic):**
```kleis
// stdlib/types.kleis - Loaded at runtime!
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Complex
  | Currency(code: String)  // ← Users add this!
```

**Benefits:**
- ✅ Type system defined in Kleis (self-hosting Level 2)
- ✅ Users extend types without recompiling
- ✅ Matrix becomes just another data constructor (no special cases!)
- ✅ Path to meta-circularity (Kleis types in Kleis)

---

## Next Session Task: Start ADR-021 Implementation

### **Preparation (5 min):**
1. Review ADR021_IMPLEMENTATION_PLAN.md
2. Create feature branch: `feature/adr-021-data-types`
3. Verify starting point: v0.6.0-adr016-complete

### **Step 1: Add Data Type AST** (2 hours)

**File:** `src/kleis_ast.rs`

**Changes:**
```rust
pub enum TopLevel {
    DataDef(DataDef),  // ← ADD THIS
    // ... existing
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataDef {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<DataVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataVariant {
    pub name: String,
    pub fields: Vec<DataField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataField {
    pub name: Option<String>,
    pub type_expr: TypeExpr,
}
```

**Tests:**
- Create DataDef programmatically
- Verify fields are correct
- Test with/without type params

**Commit:** "feat: Add DataDef AST for ADR-021"

---

### **Step 2: Parser Support** (4 hours)

**File:** `src/kleis_parser.rs`

**Grammar:**
```ebnf
dataDecl ::= "data" identifier [ "(" typeParams ")" ] "=" 
             dataVariant { "|" dataVariant }
```

**Implementation:**
- Add `parse_data_def()` function
- Handle `data` keyword
- Parse variants with "|" separator
- Parse fields (named and positional)

**Tests:**
- `data Bool = True | False`
- `data Option(T) = None | Some(T)`
- `data Type = Scalar | Matrix(Nat, Nat)`

**Commit:** "feat: Add parser support for data keyword"

---

### **Step 3: Data Registry** (3 hours)

**File:** `src/data_registry.rs` (NEW)

**Create registry for data type definitions:**
- Maps type names to definitions
- Maps variant names to (type, variant)
- Lookup functions
- Validation

**Tests:**
- Register data type
- Lookup variants
- Detect conflicts

**Commit:** "feat: Add DataTypeRegistry for ADR-021"

---

### **Remaining Steps:** See ADR021_IMPLEMENTATION_PLAN.md

Steps 4-11 cover:
- Type enum refactoring (biggest change)
- Unification updates
- Generic constructor inference
- Integration with TypeChecker
- stdlib/types.kleis creation
- Backward compatibility
- Migration of tests

---

## Critical Success Factors

### **Must Maintain:**
- ✅ All 288 tests passing (regression prevention)
- ✅ Backward compatibility during transition
- ✅ Error messages quality
- ✅ Performance acceptable

### **Must Achieve:**
- ✅ Load stdlib/types.kleis successfully
- ✅ Matrix as data constructor (no special case)
- ✅ Users can define custom types
- ✅ Unification works with data types

---

## Rollback Strategy

**Safety checkpoints:**
1. **Tag before starting:** `v0.6.0-adr016-complete` ← Safe harbor
2. **Feature branch:** Can abandon if needed
3. **Incremental commits:** Can bisect if issues
4. **Tests guard:** Don't merge until all pass

**If stuck:**
- Check ADR021_IMPLEMENTATION_PLAN.md for detailed guidance
- Revert to v0.6.0-adr016-complete and reassess
- Consider smaller incremental approach

---

## Expected Timeline

**Week 1:** AST, Parser, Registry (Steps 1-3)  
**Week 2:** Type refactoring, Integration, Testing (Steps 4-11)

**Total:** 1-2 weeks depending on complexity

---

## Why This Matters

**From ADR-020/021:**
> "The Matrix bug isn't a bug - it's a symptom of missing algebraic data types!"

**Once we have `data`:**
- Matrix constructor weirdness: SOLVED
- User type extensibility: ENABLED
- Meta-circularity (Level 2): ACHIEVED
- Self-hosting vision: REALIZED

**This is the breakthrough that makes Kleis truly self-hosting!** 🎯

---

## Session Summary (Dec 8, 2024)

**Commits today:** 7 commits
1. Task 1.5 complete (clippy fixes)
2. ADR-020 extended (Matrix analysis)
3. Session README updated
4. ADR-016 completion (remove Matrix hardcoding)
5. Generic validation (structure checks)
6. Validation tests (7 new tests)
7. type_inference.rs prepared for ADR-021

**Tests:** 281 → 288 (7 new validation tests)  
**Code quality:** Clean, all checks pass  
**Documentation:** ~3,000 lines added  
**Ready:** ADR-021 implementation plan complete

---

**Next session: Start implementing ADR-021!** 🚀

**First action:** Create feature branch and start with AST changes.

---

**Status:** ✅ Ready to tag and push  
**Tag:** v0.6.0-adr016-complete  
**Next:** ADR-021 implementation (data types)

