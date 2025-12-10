# Z3 Setup and Next Session Guide

**Date:** December 10, 2024  
**Branch:** `feature/full-prelude-migration` (6 commits)  
**Status:** Z3 working, ready to build translator

---

## ✅ Current Setup (WORKING)

### Environment Configuration

**Critical: Rust must match system architecture!**

**System:**
- macOS on Apple Silicon (ARM64/aarch64)
- Z3 4.15.4 installed via Homebrew (arm64)

**Rust toolchain (MUST BE ARM64):**
```bash
# Check current:
rustc --version --verbose | grep host
# Should show: aarch64-apple-darwin

# If shows x86_64-apple-darwin, switch:
rustup default stable-aarch64-apple-darwin
```

**Environment variables (NOT needed with local path):**
```bash
# These were tried but not necessary when using local Z3 clone:
export Z3_SYS_INCLUDE_DIR="/opt/homebrew/opt/z3/include"
export Z3_SYS_LIB_DIR="/opt/homebrew/opt/z3/lib"
export Z3_SYS_Z3_HEADER="/opt/homebrew/opt/z3/include/z3.h"
```

### Cargo.toml Configuration

**Current (WORKING):**
```toml
[dependencies]
z3 = { path = "../Z3/z3.rs/z3", optional = true }

[features]
axiom-verification = ["z3"]
```

**Why local path:**
- z3-sys from crates.io tries to build from source
- CMake compatibility issues
- Local clone avoids build complexity

### File Locations

**Z3 Rust bindings (API we use):**
```
/Users/eatik_1/Documents/git/cee/Z3/z3.rs/
├── z3/           ← Rust crate we depend on
│   ├── src/
│   │   ├── lib.rs
│   │   └── ast/
│   │       ├── int.rs    ← Int AST
│   │       ├── bool.rs   ← Bool AST
│   │       ├── real.rs   ← Real AST
│   │       └── mod.rs    ← Main AST module
│   └── tests/
│       └── lib.rs        ← Usage examples
└── z3-sys/       ← Low-level C bindings
```

**Z3 C++ source code (implementation details):**
```
/Users/eatik_1/Documents/git/cee/Z3/z3/
├── src/
│   ├── ast/
│   │   ├── ast.h/.cpp                    ← Core AST (107KB)
│   │   ├── arith_decl_plugin.h/.cpp      ← Arithmetic operations (27KB/37KB)
│   │   └── rewriter/
│   │       ├── arith_rewriter.h/.cpp     ← Arithmetic simplification (9KB/82KB)
│   │       ├── poly_rewriter.h           ← AC operation template
│   │       └── poly_rewriter_def.h       ← Flattening, sorting
│   └── smt/
│       └── ... (SMT solver core)
└── doc/
    └── ... (Z3 documentation)
```

**Kleis project:**
```
/Users/eatik_1/Documents/git/cee/kleis/
├── tests/
│   ├── z3_axiom_experiments.rs       ← 7 axiom tests
│   ├── z3_kleis_grammar_tests.rs     ← 7 grammar tests
│   └── z3_e_unification_tests.rs     ← 7 E-unif tests
├── src/
│   └── (future: axiom_verifier.rs)   ← To be created
└── Cargo.toml                         ← Has z3 dependency
```

---

## ✅ What Works Now

### Running Tests

```bash
# Switch to feature branch
git checkout feature/full-prelude-migration

# Run all Z3 tests (21 tests total)
cargo test --features axiom-verification \
    --test z3_axiom_experiments \
    --test z3_kleis_grammar_tests \
    --test z3_e_unification_tests

# All should pass! ✅
```

### Test Coverage

**Axiom verification (7 tests):**
- x + 0 = x (identity)
- x + y = y + x (commutativity)
- (x+y)+z = x+(y+z) (associativity)
- x(y+z) = xy+xz (distributivity)
- x × 1 = x (multiplicative identity)
- Counterexample detection
- Multiple axioms together

**Kleis grammar (7 tests):**
- Matrix dimension checking
- Type unification
- Dimension mismatch detection
- Comparison operators
- Logical operators
- Piecewise condition logic
- Type consistency

**E-unification (7 tests):**
- Built-in commutativity
- Built-in associativity
- Algebraic simplification
- Distributivity as rewrite
- x × 0 = 0
- x × 1 = x
- -(-x) = x

**Total: 21 tests, all passing** ✅

---

## 🎯 Next Session Plan

### Part 1: Build Generic Translator (2-3 hours)

**Create:** `src/axiom_verifier.rs`

**Key functions:**
```rust
pub struct AxiomVerifier {
    // No fields needed - Z3 context is thread-local
}

impl AxiomVerifier {
    /// Verify ANY Kleis axiom
    pub fn verify_axiom(&self, axiom: &Axiom) -> Result<VerificationResult> {
        // 1. Extract quantified variables
        // 2. Create Z3 variables
        // 3. Translate Kleis expression to Z3
        // 4. Check with solver
    }
    
    /// Generic translator: Kleis Expression → Z3
    fn kleis_to_z3(
        &self,
        expr: &Expression,
        vars: &HashMap<String, z3::ast::Int>,
    ) -> Result<z3::ast::Int> {
        // Map operations by name (NO HARDCODING!)
    }
    
    /// Check if two expressions are equivalent
    pub fn are_equivalent(
        &self,
        expr1: &Expression,
        expr2: &Expression,
    ) -> Result<bool> {
        // Use Z3 to check algebraic equivalence
        // KEY for simplification!
    }
}
```

### Part 2: Integrate with Structure Registry (1 hour)

**Add axioms to structure lookup:**
```rust
// In StructureRegistry or TypeChecker:
pub fn get_axioms(&self, structure_name: &str) -> Vec<&Axiom> {
    // Return axioms for a structure
}

pub fn verify_implementation(
    &self,
    impl_name: &str,
) -> Result<Vec<AxiomViolation>> {
    // Check if implementation satisfies all axioms
}
```

### Part 3: Parser Extensions (2-3 hours)

**Add support for:**

1. **Universal quantifiers:**
   ```kleis
   axiom: ∀(x y : R). x + y = y + x
   ```

2. **Operator symbols:**
   ```kleis
   operation (×) : R → R → R
   ```

3. **Implication:**
   ```kleis
   axiom: P ⟹ Q
   ```

### Part 4: Load Full Prelude (1 hour)

**Replace:**
```rust
let minimal_prelude = include_str!("../stdlib/minimal_prelude.kleis");
```

**With:**
```rust
let prelude = include_str!("../stdlib/prelude.kleis");
```

**Requires:** Parser extensions from Part 3

### Part 5: Write ADR-022 (1 hour)

**Document based on real experience:**
- Why Z3? (What we learned from tests)
- How it works (Normalization, not E-unification)
- Architecture (Hybrid approach)
- Trade-offs (Build complexity, external dependency)
- Decision (Include as optional feature)

**Timeline: 7-9 hours total**

---

## 🔑 Critical Setup Reminders

### Before Starting Work

```bash
# 1. Verify Rust architecture
rustc --version --verbose | grep host
# MUST show: aarch64-apple-darwin

# 2. Switch to feature branch
git checkout feature/full-prelude-migration

# 3. Verify Z3 tests pass
cargo test --features axiom-verification \
    --test z3_axiom_experiments -- --nocapture
# Should see: "test result: ok. 7 passed"
```

### If Architecture Wrong

```bash
# Switch to ARM64 Rust:
rustup default stable-aarch64-apple-darwin

# Rebuild everything:
cargo clean
cargo test --features axiom-verification
```

### If Z3 Build Fails

**Check Cargo.toml has:**
```toml
z3 = { path = "../Z3/z3.rs/z3", optional = true }
```

**Not:**
```toml
z3 = { version = "0.12", ... }  # This tries to build from source
```

---

## 📚 Reference Documentation

**On feature branch:**
1. `docs/session-2024-12-10/Z3_AST_VS_KLEIS_AST.md`
   - Comparison of AST structures
   - What each is good for
   - Translation strategy

2. `docs/session-2024-12-10/HOW_Z3_DOES_E_UNIFICATION.md`
   - Normalization approach
   - Flattening + sorting
   - Why it works for AC

3. `NEXT_SESSION_TASK.md`
   - Complete plan
   - Full prelude migration
   - Matrix cleanup
   - Z3 integration

**Test files:**
- `tests/z3_axiom_experiments.rs` - Basic axiom verification
- `tests/z3_kleis_grammar_tests.rs` - Kleis features
- `tests/z3_e_unification_tests.rs` - Simplification rules

---

## 🎯 Success Criteria for Next Session

**When done:**
1. ✅ Generic `kleis_to_z3()` translator working
2. ✅ Can verify any axiom from stdlib
3. ✅ Can check expression equivalence (for simplification)
4. ✅ Parser supports `∀` and `(×)` syntax
5. ✅ Load full `prelude.kleis`
6. ✅ ADR-022 written with real learnings
7. ✅ Merge to main (all tests passing)

---

## 💡 Key Insights to Remember

### 1. Architecture Matters!

**x86_64 Rust + arm64 Z3 = Linker errors**

Solution: Match architectures!

### 2. Z3 Does Normalization, Not E-Unification

**What it really does:**
- Flatten AC operations
- Sort canonically
- Combine like terms
- Apply rewrite rules

**Behaves like E-unification for arithmetic!**

### 3. Hybrid Approach Is Best

**Use Z3 for:** Standard arithmetic (it's built-in)  
**Build ourselves:** Custom operations (domain-specific)

### 4. Tests Before ADR

**Process:**
1. Experiment (feature branch)
2. Learn what works
3. Document decision (ADR)

**Not:** Speculate, decide, implement

---

## 📝 Quick Start for Next Session

```bash
# 1. Verify setup
rustc --version | grep aarch64  # Should match
uname -m                        # Should be arm64

# 2. Switch to branch
cd /Users/eatik_1/Documents/git/cee/kleis
git checkout feature/full-prelude-migration

# 3. Verify tests work
cargo test --features axiom-verification --test z3_axiom_experiments

# 4. Start coding!
# Create src/axiom_verifier.rs
# Implement generic kleis_to_z3() translator
```

---

## 🗂️ Commit History Summary

**Main branch (40 commits today):**
- Formatting fixes
- Math library (70+ operations)
- Piecewise functions (fully parametric)
- Documentation updates

**Feature branch (6 commits):**
1. `d08b4cf` - Z3 foundation and tests
2. `a134d4f` - Build issues documented
3. `ac76cfd` - Try local path
4. `bfd78fa` - Grammar tests + E-unification discovery
5. `0c753b0` - AST comparison doc
6. `b15f6ce` - E-unification explanation

---

## Ready for Next Session! 🚀

**Everything documented:**
- ✅ Environment setup
- ✅ Architecture requirements
- ✅ File locations
- ✅ What works
- ✅ What to build next
- ✅ Why it matters

**You'll be able to jump right in!** 🎯

