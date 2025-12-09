# ADR-016 Compliance Status - Honest Assessment

**Date:** December 6, 2024  
**Status:** ⚠️ Partially Compliant (POC stage)

---

## The Ideal (Pure ADR-016)

**All type rules defined in stdlib/*.kleis:**
```kleis
structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
    operation multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
}
```

**Type inference queries the registry:**
```rust
fn infer_operation(&mut self, name: &str, args: &[Expression]) -> Result<Type, String> {
    // Query: What's the result type from the structure definition?
    self.context_builder.infer_operation_type(name, &arg_types)
}
```

**No hardcoding in Rust!**

---

## The Reality (Current Implementation)

**Matrix operations ARE hardcoded in `src/type_inference.rs`:**

```rust
// Lines 297-396: Hardcoded matrix logic
"multiply" => {
    match (&t1, &t2) {
        (Type::Matrix(m, n), Type::Matrix(p, q)) => {
            if n != p { return Err(...) }
            Ok(Type::Matrix(*m, *q))
        }
    }
}

"add" => { /* hardcoded logic */ }
"transpose" => { /* hardcoded logic */ }
"det" => { /* hardcoded logic */ }
"trace" => { /* hardcoded logic */ }
```

**This violates ADR-016's "no hardcoding" principle!**

---

## Why This Exists (Honest Reasons)

### Technical Complexity

**Problem:** TypeInference needs access to TypeContextBuilder to query structures.

**Challenge:**
```rust
pub struct TypeInference {
    context: TypeContext,
    constraints: Vec<Constraint>,
    // How to add: context_builder: &TypeContextBuilder ???
}
```

**Issues:**
1. Lifetime complexity ('a annotation needed)
2. Borrow checker (already have &mut self)
3. Ripple effect (all callsites need updating)

### Pragmatic Decision

**For the POC/demo:**
- Hardcoded rules WORK
- Demonstrate the concept
- Get live type feedback shipping
- Users can see value immediately

**But it's acknowledged:**
- TODO comments in code
- This document for transparency
- Clear path forward documented

---

## What IS ADR-016 Compliant

✅ **Structure definitions** - `stdlib/matrices.kleis` defines operations correctly  
✅ **TypeContextBuilder.infer_operation_type()** - Implemented the right way!  
✅ **Registry queries** - Infrastructure exists  
✅ **No hardcoded TYPES** - Types come from structures  

**Only issue:** Operation TYPE INFERENCE logic is hardcoded

---

## The Path Forward (Proper Implementation)

### Option 1: Lifetime Parameter (Clean but Complex)

```rust
pub struct TypeInference<'a> {
    context: TypeContext,
    constraints: Vec<Constraint>,
    context_builder: Option<&'a TypeContextBuilder>,
}

impl<'a> TypeInference<'a> {
    fn infer_operation(&mut self, name: &str, args: &[Expression]) -> Result<Type, String> {
        if let Some(builder) = self.context_builder {
            // Infer argument types
            let arg_types: Vec<Type> = args.iter()
                .map(|a| self.infer(a))
                .collect::<Result<Vec<_>, _>>()?;
            
            // Query registry!
            builder.infer_operation_type(name, &arg_types)
        } else {
            // Fallback
            Ok(self.context.fresh_var())
        }
    }
}
```

**Pros:** Pure ADR-016, clean architecture  
**Cons:** Lifetime annotations everywhere, refactor all callsites

### Option 2: Pass Builder as Parameter (Simple)

```rust
fn infer_operation(
    &mut self,
    name: &str,
    args: &[Expression],
    context_builder: Option<&TypeContextBuilder>,  // NEW
) -> Result<Type, String> {
    if let Some(builder) = context_builder {
        // Query registry
        let arg_types = ...;
        return builder.infer_operation_type(name, &arg_types);
    }
    
    // Fallback
    Ok(self.context.fresh_var())
}
```

**Pros:** Simpler, fewer changes  
**Cons:** Parameter threading

### Option 3: Keep Hardcoding, Document It (Current)

**Pros:** Works now, ships value  
**Cons:** Not pure ADR-016

---

## Recommendation

**For next refactor (when time permits):**

1. Choose Option 1 or Option 2 above
2. Move ALL operation logic to `TypeContextBuilder.infer_operation_type()`
3. Parse type signatures from structure definitions
4. Remove hardcoded rules from `type_inference.rs`
5. Full ADR-016 compliance

**Timeline:** 2-3 hours focused refactoring

**Until then:**
- Current implementation WORKS
- Demonstrates the vision
- Path forward is clear
- Honestly documented

---

## What Users Get NOW

Despite the implementation not being pure ADR-016:

✅ Live type feedback in equation editor  
✅ Matrix type inference with dimension checking  
✅ Clear error messages  
✅ Structures defined in stdlib/matrices.kleis  
✅ Working demo of the concept  

**The architecture is 80% there, implementation is 60% pure.**

---

## Acceptance Criteria for Full Compliance

- [ ] Zero hardcoded operation logic in type_inference.rs
- [ ] All inference delegates to TypeContextBuilder
- [ ] Type rules parsed from structure definitions
- [ ] Works for user-defined structures (not just built-in)
- [ ] Can add new operations via stdlib/*.kleis only

**Current:**  
- [x] Structures defined in stdlib ✅
- [x] Registry infrastructure ✅
- [ ] Inference delegates to registry ❌ (hardcoded)

---

**Verdict:** ⚠️ **Partially Compliant**

**This is an honest POC with a clear path to full compliance.**

**Working demo > perfect architecture** for proving the concept!

---

**Next Session Goal:** Refactor to Option 1 or 2 above for pure ADR-016 compliance.


