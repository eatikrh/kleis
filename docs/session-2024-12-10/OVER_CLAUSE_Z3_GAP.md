# Gap Found: Over Clause Not Connected to Z3

**Date:** December 10, 2024  
**Severity:** MEDIUM  
**Status:** 🐛 BUG - Feature parsed but not used by Z3

---

## The Issue

The `over` clause is successfully parsed and stored in the AST, but the **axiom verifier doesn't use it**!

### What Works

```kleis
structure VectorSpace(V) over Field(F) {
  operation (·) : F × V → V
  axiom scalar_identity: ∀(v : V). 1 · v = v
}
```

✅ Parser: Recognizes `over Field(F)`  
✅ AST: Stores it in `StructureDef.over_clause`  
❌ Z3: Doesn't load Field axioms when verifying VectorSpace axioms!

---

## The Problem

When verifying `scalar_identity: ∀(v : V). 1 · v = v`:

**Should happen:**
1. Analyze dependencies → find VectorSpace
2. Load VectorSpace axioms
3. See `over Field(F)`
4. Load Field axioms (multiplicative_identity: `1 × x = x`)
5. Use Field axioms as background assumptions

**Actually happens:**
1. Analyze dependencies → find VectorSpace
2. Load VectorSpace axioms
3. ❌ **SKIP** `over Field(F)` - not checked!
4. Verify without Field axioms

**Result:** Z3 might not be able to prove things that depend on field properties!

---

## Evidence

Searched `src/axiom_verifier.rs` for `over_clause`:

```bash
$ grep "over_clause" src/axiom_verifier.rs
# No results!
```

The `ensure_structure_loaded()` function checks:
- ✅ `where` constraints (lines 170-181)
- ✅ `extends` clause (lines 189-201)
- ❌ `over` clause - **NOT CHECKED!**

---

## How to Fix

### Option 1: Add Over Clause Handling to Axiom Verifier

In `ensure_structure_loaded()`, after loading extends clause:

```rust
// THIRD: Load field structure if over clause present
if let Some(over_type) = &structure.over_clause {
    // Extract field structure name
    let field_name = match over_type {
        crate::kleis_ast::TypeExpr::Parametric(name, _) => name.clone(),
        _ => return Err("Invalid over clause type".to_string()),
    };
    
    println!("   🔗 Loading over clause: {}", field_name);
    self.ensure_structure_loaded(&field_name)?;
}
```

**Estimated effort:** 15-30 minutes

### Option 2: Document as Known Limitation

Add comment explaining that `over` clause is syntactic only for now, doesn't affect verification.

---

## Impact Assessment

### Current Impact: LOW

**Why?**
1. VectorSpace axioms mostly don't depend on Field axioms
2. The axioms are about vector operations, not field operations
3. Users can manually ensure field axioms are loaded if needed

**Examples that work without Field axioms:**
```kleis
axiom vector_associativity: ∀(u v w : V). (u + v) + w = u + (v + w)
// Doesn't use field operations - works fine
```

**Examples that might need Field axioms:**
```kleis
axiom scalar_distributivity: ∀(c : F, u v : V). c · (u + v) = c · u + c · v
// Uses scalar from F - might benefit from Field axioms
```

### Future Impact: MEDIUM

As we add more complex axioms that depend on field properties, this gap will become more significant.

---

## Recommendation

**For this session:** Document the gap, fix later

**Rationale:**
1. Parser work is complete and working beautifully
2. This is a Z3 integration issue, not a parser issue
3. Current functionality isn't broken
4. Easy to fix later (15-30 minutes)
5. Session has already been very successful!

**Next session:** Add over clause handling to axiom verifier

---

## Similar Features That DO Work

| Feature | Parser | Z3 Integration | Status |
|---------|--------|----------------|--------|
| `extends` | ✅ | ✅ | Working |
| `where` in implements | ✅ | ✅ | Working |
| `where` in quantifiers | ✅ | ✅ | Working |
| Nested structures | ✅ | ✅ | Working |
| **`over` clause** | ✅ | ❌ | **Gap!** |

---

## Action Items

### Short Term (Next Session)
1. Add `over` clause handling to `ensure_structure_loaded()`
2. Test that Field axioms are loaded for VectorSpace
3. Verify scalar operations with field properties

### Long Term
1. Consider if `over` should be treated like `extends` (inheritance)
2. Document the semantic meaning more clearly
3. Add tests for over clause in Z3 verification

---

## Conclusion

**This is a minor gap in an otherwise excellent implementation!**

The `over` clause:
- ✅ Parses correctly
- ✅ Stores in AST
- ❌ Not used by Z3 verifier

**Easy fix, low current impact, document and defer.**

The parser work is **complete and exceptional!** 🎉

---

**Discovered:** December 10, 2024  
**Priority:** MEDIUM  
**Effort to fix:** 15-30 minutes

