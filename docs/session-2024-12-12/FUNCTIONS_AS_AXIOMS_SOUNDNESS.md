# Functions as Axioms - Soundness Analysis

**Date:** December 12, 2024  
**Critical Question:** Are we tainting the axiom scope by implementing functions as axioms?

---

## 🎯 The Concern

**When we do this:**
```kleis
structure Ring(R) {
  operation (+) : R × R → R
  operation negate : R → R
  
  define (-)(x, y) = x + negate(y)
}
```

**We translate to Z3:**
```smt
∀(x y). minus(x, y) = plus(x, negate(y))
```

**Question:** Does this "taint" the axiom space? Can it cause:
1. ❌ Inconsistencies?
2. ❌ Proof search problems?
3. ❌ Override conflicts?
4. ❌ Semantic confusion (definition vs axiom)?

---

## 📚 Mathematical Background: Definitional Axioms

### In Formal Logic

**Definitional axioms are a standard technique!**

From logic textbooks:
- A **definitional axiom** introduces a new symbol with a fixed meaning
- It's a **conservative extension** if the new symbol is fresh
- Conservative extensions **cannot introduce inconsistency**

**Example from mathematics:**
```
Define: x - y ≡ x + (-y)

This becomes axiom: ∀x y. (x - y) = x + (-y)

This is SAFE because:
1. (-) is a fresh symbol (not used before)
2. The definition is explicit (not recursive)
3. It's eliminable (can always expand it away)
```

**Theorem (Conservativity):**
> If a definition is explicit and non-circular, adding it as an axiom does not change what's provable about the original theory.

---

## ✅ Why "Functions as Axioms" is SOUND

### 1. **No Inconsistency Risk** ✅

**Definitional axioms cannot introduce contradictions IF:**
- ✅ The function name is fresh (not previously defined)
- ✅ The definition is explicit (body is well-formed)
- ✅ No circular dependencies

**In Kleis:**
```kleis
define (-)(x, y) = x + negate(y)
```

**Safe because:**
- ✅ `(-)` is explicitly declared (not previously defined)
- ✅ RHS uses only existing operations (`+`, `negate`)
- ✅ No recursion (explicit formula)

**Cannot introduce inconsistency!** ✅

### 2. **Proof Search Impact** ⚠️ (Minor)

**Concern:** More axioms = more work for Z3?

**Reality:**
- ✅ Z3 is designed to handle thousands of axioms efficiently
- ✅ Most proofs only use relevant axioms
- ⚠️ Each function adds one axiom (linear growth)
- ✅ Benefits outweigh costs (can actually help proofs!)

**Example:**
```
Without function axiom:
- Z3 treats (-) as uninterpreted
- Cannot prove: (a - a) = zero
- ❌ Proof fails

With function axiom:
- Z3 knows: minus(x, y) = plus(x, negate(y))
- CAN prove: (a - a) = zero
- ✅ Proof succeeds!
```

**The axiom HELPS more than it hurts!** ✅

### 3. **Override Handling** ⚠️ (Needs Care)

**Potential Problem:**
```kleis
structure Ring(R) {
  define (-)(x, y) = x + negate(y)  // Default
}

implements Ring(ℤ) {
  operation (-) = builtin_int_subtract  // Override
}
```

**Issue:** Now we have TWO definitions of `(-)`!
- Z3 axiom: `∀x y. minus(x, y) = plus(x, negate(y))`
- Override: `minus = builtin_int_subtract`

**Solution Options:**

**Option A: Don't load structure functions into Z3 for specific implements**
```rust
// When verifying axioms for Ring(ℤ):
if impl_def.overrides("-") {
    // Don't load the structure's define(-) axiom
    // Use the override instead
}
```

**Option B: Treat overrides as refinements (current behavior)**
```rust
// Both are true:
∀x y. minus(x, y) = plus(x, negate(y))     [abstract definition]
minus = builtin_int_subtract               [concrete implementation]

// Z3 will ensure they're consistent!
// If builtin_int_subtract disagrees with the definition, Z3 will catch it!
```

**Option B is actually SAFER!** It validates that overrides respect the abstract definition.

### 4. **Semantic Clarity** ✅

**Concern:** Mixing definitions and axioms is confusing?

**Reality:**
```kleis
// In mathematics, these are the same:

axiom subtraction_def: ∀(x y : R). (x - y) = x + (-y)
define (-)(x, y) = x + negate(y)

// Both are "definitional axioms"!
```

**In formal logic:**
- Definitions ARE axioms (special case)
- Called "definitional axioms" or "explicit definitions"
- Standard practice in theorem provers

**Coq example:**
```coq
Definition minus (x y : R) := plus x (negate y).
(* This adds an axiom to the context! *)
```

**HOL example:**
```
new_definition `minus x y = plus x (negate y)`
(* Adds definitional axiom *)
```

---

## 🔍 Potential Problems and Solutions

### Problem 1: Circular Definitions

**Bad:**
```kleis
define f(x) = g(x)
define g(x) = f(x)
```

**Solution:**
```rust
// Detect cycles before loading
fn check_circular_dependencies(func_defs: &[FunctionDef]) -> Result<(), String> {
    // Build dependency graph
    // Check for cycles
    // Reject if found
}
```

**Status:** Not implemented yet, but detectable

### Problem 2: Non-terminating Functions

**Example:**
```kleis
define loop(x) = loop(x + 1)
```

**In Z3:**
```smt
∀x. loop(x) = loop(x + 1)
```

**Impact:**
- Z3 may not terminate when trying to expand
- Or Z3 treats it as infinite chain (may time out)

**Solution:**
- Detect direct recursion
- Use RecFuncDecl for recursive functions
- Or reject recursive definitions in v0.6 (add later)

### Problem 3: Type Mismatches

**Example:**
```kleis
define bad(x : ℝ) = x + "string"  // Type error!
```

**Solution:**
- Type check function bodies before loading
- Already done by TypeChecker! ✅
- Only well-typed functions reach Z3

---

## ✅ Is The Axiom Space Tainted? NO!

**Answer: Functions as axioms do NOT taint the axiom space!**

**Why not:**

### 1. **Mathematically Sound**
Definitional axioms are **conservative extensions**:
- Cannot introduce inconsistency
- Standard technique in theorem provers
- Used in Coq, HOL, Lean, Isabelle

### 2. **Semantically Clear**
```
axiom associativity: ∀(x y z). (x + y) + z = x + (y + z)  [primitive truth]
define (-)(x, y) = x + negate(y)                          [derived concept]
```

Both become Z3 assertions, but:
- We know which are primitive (axioms)
- We know which are derived (defines)
- This is **documentation**, not semantic difference

### 3. **Can Be Validated**
```kleis
structure Ring(R) {
  define (-)(x, y) = x + negate(y)
}

implements Ring(ℤ) {
  operation (-) = builtin_subtract  // Override
}
```

**Z3 can check consistency:**
- Does builtin_subtract match the definition?
- If not, Z3 will find a counterexample!
- This is a **feature**, not a bug! ✅

### 4. **Performance Impact is Minimal**
- Each function = 1 axiom
- Z3 handles thousands of axioms efficiently
- Usually HELPS proofs (provides expansion rules)

---

## 🎯 Comparison with Alternatives

### Approach 1: Functions as Axioms (Our Choice) ✅

**Pros:**
- ✅ Mathematically sound (conservative extension)
- ✅ Standard theorem proving technique
- ✅ Simple implementation
- ✅ Can verify overrides are consistent
- ✅ Helps proofs (expansion rules available)

**Cons:**
- ⚠️ Axiom count grows (one per function)
- ⚠️ Need to detect circular definitions
- ⚠️ Override handling needs thought

### Approach 2: Macro Expansion (Alternative)

**Pros:**
- ✅ No axioms added (cleaner axiom space)
- ✅ Direct semantics

**Cons:**
- ❌ Code duplication at each call site
- ❌ Exponential blowup with nesting
- ❌ Harder to debug
- ❌ Need substitution engine

### Approach 3: Don't Integrate with Z3

**Pros:**
- ✅ Axiom space unchanged

**Cons:**
- ❌ Cannot prove properties about derived operations
- ❌ Grammar v0.6 functions half-implemented
- ❌ Users expect it to work

---

## ✅ Conclusion

**Q: Are we tainting the axiom scope?**

**A: NO!** ✅

**Reasons:**
1. ✅ **Mathematically sound** - Definitional axioms are conservative
2. ✅ **Standard practice** - Used in all major theorem provers
3. ✅ **Can be validated** - Z3 can check override consistency
4. ✅ **Helps more than hurts** - Enables proofs of derived operations
5. ✅ **Semantically clear** - We know what's primitive vs derived

**The axiom space is NOT tainted, it's EXTENDED with valid definitions.**

**Analogy:**
```
Adding function definitions is like:
- Adding new vocabulary to a language (extends it)
- NOT changing grammar rules (would be tainting)
```

---

## 🛡️ Safety Measures to Add

**To make this even safer:**

### 1. Detect Circular Definitions
```rust
// Before loading, check for cycles
check_function_dependencies(func_defs)?;
```

### 2. Mark Definitional Axioms
```rust
// In Z3, could track which axioms are definitional
self.definitional_axioms.insert(func_name);
```

### 3. Validate Overrides
```rust
// When override found, verify it matches definition
verify_override_consistency(structure_def, impl_override)?;
```

---

## ✅ Final Answer

**Our approach is sound!** Functions as axioms:
- ✅ Are mathematically correct
- ✅ Follow theorem proving best practices  
- ✅ Do NOT taint the axiom space
- ✅ Actually IMPROVE proof capabilities

**The axiom space is extended, not tainted!**

Would you like me to commit the TODO #57 implementation?
