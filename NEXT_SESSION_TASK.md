# NEXT SESSION: Matrix Type Consistency

**Current State:** main branch, 31 commits pushed, 413 tests passing  
**Status:** 🎯 Ready for type signature standardization

---

## 🎯 Priority: Standardize Matrix Type Signatures

### The Inconsistency

**In types.kleis (actual definition):**
```kleis
data Type = ... | Matrix(m: Nat, n: Nat, T)
```
3 parameters: dimensions + element type ✅

**In prelude.kleis (examples):**
```kleis
operation (×) : ∀(m n p : ℕ). Matrix(m,n) × Matrix(n,p) → Matrix(m,p)
operation transpose : ∀(m n : ℕ). Matrix(m,n) → Matrix(n,m)
operation det : ∀(n : ℕ). Matrix(n,n) → ℝ
```
2 parameters: dimensions only, **T is missing!** ❌

**In matrices.kleis (what we use):**
```kleis
structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
  operation multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
}
```
3 parameters: includes T ✅

### The Goal

**Standardize ALL Matrix references to always include T:**
```kleis
Matrix(m, n, T)  // ALWAYS - never just Matrix(m, n)
```

**Remove ALL legacy hardcoded matrix constructors:**
```kleis
// DELETE THESE:
structure LegacyMatrixConstructors(T) {
    operation matrix2x2 : Matrix(2, 2, ℝ)
    operation matrix2x3 : Matrix(2, 3, ℝ)
    operation matrix3x2 : Matrix(3, 2, ℝ)
    // ... etc
}
```

**Use only the parametric constructor:**
```kleis
// The ONE TRUE WAY:
Matrix(2, 2, [a, b, c, d])  // Creates any size matrix
```

Update:
1. `prelude.kleis` - Fix all operation signatures to include T
2. `matrices.kleis` - DELETE LegacyMatrixConstructors structure
3. `src/render.rs` - Remove legacy matrix rendering code
4. Any docs or examples using shorthand
5. Verify consistency across codebase

### Why This Matters

**With T explicit:**
- Can multiply matrices of ANY type (ℝ, ℂ, ℕ, even nested Matrix!)
- Polymorphism is clear
- Type system can properly check element type compatibility

**Example:**
```kleis
// Block matrices work automatically!
Matrix(2, 2, Matrix(3, 3, ℝ))  // 2×2 of 3×3 blocks
```

---

## ⚠️ IMPORTANT: Work on Separate Branch

**Branch name:** `feature/matrix-type-consistency`

**Why separate branch:**

1. **Will cause many errors** while working
2. **Takes significant time** to update all references
3. **Don't want to block main** with broken intermediate states
4. **Can test thoroughly** before merging

### Expected Breakage

While updating, expect:
- Type errors where Matrix(m,n) is used without T
- Signature mismatches in operations
- Tests failing until all references updated
- Parser might need adjustments
- Rendering might break for legacy matrix operations
- Any code using matrix2x2, matrix3x3, etc. will fail

### Timeline

**Estimated:** 3-4 hours (increased due to legacy cleanup)
- Find all Matrix(m,n) usages (~30 min)
- Update signatures systematically (~1 hour)
- Remove LegacyMatrixConstructors (~30 min)
- Clean up renderer legacy code (~30 min)
- Fix resulting type errors (~1 hour)
- Test and verify (~30 min)

---

## Implementation Plan

### Step 1: Create Branch (5 min)

```bash
git checkout -b feature/matrix-type-consistency
```

### Step 2: Update prelude.kleis (30 min)

Change ALL Matrix signatures:
```kleis
operation (×) : ∀(m n p : ℕ, T). Matrix(m,n,T) × Matrix(n,p,T) → Matrix(m,p,T)
operation transpose : ∀(m n : ℕ, T). Matrix(m,n,T) → Matrix(n,m,T)
operation det : ∀(n : ℕ, T). Matrix(n,n,T) → T
operation trace : ∀(n : ℕ, T). Matrix(n,n,T) → T
```

**Note:** Det and trace return **T**, not just ℝ!
- det : Matrix(n,n,ℝ) → ℝ
- det : Matrix(n,n,ℂ) → ℂ
- Generic!

### Step 3: Search All Files (15 min)

```bash
# Find all Matrix references without T
grep -r "Matrix([^,]*,[^,]*)" stdlib/ src/ --include="*.kleis" --include="*.rs"

# Check docs
grep -r "Matrix(m,n)" docs/ --include="*.md"
```

### Step 4: Update Systematically (1 hour)

Go through each file:
- Update type signatures
- Update examples
- Update documentation
- Update comments

### Step 5: Fix Type Errors (1 hour)

Run tests frequently:
```bash
cargo test --lib
```

Fix errors as they appear:
- Missing T parameters
- Type mismatches
- Signature incompatibilities

### Step 6: Verify (30 min)

```bash
# All tests pass
cargo test --lib

# Quality gates
cargo fmt --all
cargo clippy --all-targets --all-features

# No more Matrix(m,n) without T
grep -r "Matrix([^,]*,[^,]*[^T])" stdlib/ src/
```

### Step 7: Merge

```bash
git checkout main
git merge feature/matrix-type-consistency
```

---

## Files Likely to Change

**Stdlib:**
- `stdlib/prelude.kleis` ⭐ (main target)
- `stdlib/tensors.kleis` (might have Matrix references)
- `stdlib/quantum.kleis` (might have Matrix references)

**Docs:**
- `docs/type-system/*.md` (examples might use shorthand)
- `docs/reference/*.md` (any Matrix examples)
- `README.md` (if has Matrix examples)

**Maybe:**
- `src/type_inference.rs` (comments about Matrix)
- Tests with Matrix examples

---

## Benefits After Completion

1. **Consistency** - Matrix(m,n,T) everywhere, no shortcuts
2. **Clarity** - Element type always explicit
3. **Correctness** - Type system can check element type operations
4. **Polymorphism** - Clear that Matrix works for ANY T
5. **Documentation** - Examples are accurate

---

## Related Issues

### Semiring Gap

While working on this, consider adding **Semiring** structure:
```kleis
structure Semiring(S) {
  structure additive : CommutativeMonoid(S)
  structure multiplicative : Monoid(S)
  axiom distributivity: ...
}
```

Natural numbers ℕ are a semiring (can add/multiply but not subtract).

**Decision:** Separate task, don't mix with Matrix consistency work.

---

## Success Criteria

After branch is complete:

✅ **No Matrix(m,n) without T** anywhere in codebase  
✅ **No legacy constructors** (matrix2x2, matrix2x3, etc.)  
✅ **Only parametric Matrix constructor** used throughout  
✅ **All tests pass** (413+)  
✅ **prelude.kleis signatures** are correct and complete  
✅ **Type system** properly checks element types  
✅ **Documentation** is consistent  
✅ **Renderer code** cleaned up (no legacy special cases)  

---

## Notes

**User insight:** "how do we know that Matrix(m,n) and Matrix(m,n,T) same type of things"

**Answer:** They're NOT the same - that's the problem! We need to always use Matrix(m,n,T).

The shorthand Matrix(m,n) is:
- Ambiguous (what's the element type?)
- Incomplete (missing type parameter)
- Inconsistent with our actual definition

Must be fixed for type system correctness!

---

**Ready for next session on feature branch!** 🎯
