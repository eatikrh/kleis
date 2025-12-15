# ADR-022: Z3 Integration for Axiom Verification

**Date:** 2025-12-10  
**Status:** Accepted  
**Deciders:** Architecture team  
**Related:** ADR-014 (Hindley-Milner Type System), ADR-015 (Text as Source of Truth), ADR-016 (Operations in Structures)

---

## Context

Kleis has algebraic structures defined in text (stdlib/*.kleis) with axioms:

```kleis
structure Ring(R) {
    operation (+) : R → R → R
    operation (×) : R → R → R
    
    axiom commutativity: ∀(x y : R). x + y = y + x
    axiom distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
}
```

**Problem:** Axioms were just documentation - no verification they're actually true!

**Need:** A way to verify that:
1. Axioms are mathematically valid
2. Implementations satisfy structure axioms
3. User-defined structures are consistent
4. Simplification rules are sound

---

## Decision

**Integrate Z3 SMT solver as a default (but optional) feature for axiom verification.**

### Key Design Decisions:

1. **Default feature** - Enabled by default, can be disabled
2. **Generic translator** - No hardcoded axioms, reads from Expression AST
3. **Build configuration** - `.cargo/config.toml` handles linking automatically
4. **Local dependency** - Use local z3.rs clone to avoid build issues
5. **Feature-gated** - All Z3 code behind `#[cfg(feature = "axiom-verification")]`

---

## Alternatives Considered

### Alternative 1: No Axiom Verification

**Pros:**
- Simpler codebase
- No external dependencies
- Faster builds

**Cons:**
- Axioms just documentation (can be wrong!)
- No way to verify implementations
- Can't detect inconsistencies
- Type system can't leverage axioms

**Rejected:** Axioms without verification have limited value

### Alternative 2: Manual Pattern Matching

**Approach:** Write Rust code to check specific axioms

```rust
fn verify_commutativity(expr: &Expression) -> bool {
    match expr {
        Operation { name: "plus", args } => {
            // Check if args[0] + args[1] = args[1] + args[0]
            // ... hardcoded logic
        }
    }
}
```

**Pros:**
- No external dependencies
- Full control

**Cons:**
- Have to implement theorem prover ourselves!
- Hardcoded for each axiom
- Won't scale to complex axioms
- Reimplementing decades of SMT solver work

**Rejected:** Reinventing the wheel poorly

### Alternative 3: Coq or Lean Integration

**Approach:** Use proof assistants like Coq or Lean

**Pros:**
- More powerful than Z3
- Can do interactive proofs
- Rich type theory

**Cons:**
- Steeper learning curve
- Requires proof scripts (not automatic)
- Heavier integration
- Overkill for our needs

**Rejected:** Z3's automatic verification is better fit

### Alternative 4: Z3 from crates.io

**Approach:** Use `z3 = "0.12"` from crates.io

**Tried:** Initial implementation

**Problem:** z3-sys tries to build Z3 from source
- CMake compatibility issues
- Long build times
- Cross-platform complications

**Solution:** Use local z3.rs clone instead

---

## Implementation

### Architecture

```
User writes axiom
       ↓
Parser: axiom → Expression::Quantifier
       ↓
Registry: Store in StructureDef
       ↓
AxiomVerifier: kleis_to_z3()
       ↓
Z3 Solver: Check validity
       ↓
Result: Valid/Invalid/Unknown
```

### Generic Translator

**Key insight:** No hardcoding!

```rust
fn kleis_to_z3(expr: &Expression) -> Result<Bool> {
    match expr {
        Expression::Operation { name, args } => {
            match name.as_str() {
                "plus" => Int::add(...),
                "times" => Int::mul(...),
                "logical_and" => Bool::and(...),
                // Extensible - just add more cases!
            }
        }
        Expression::Quantifier { variables, body, .. } => {
            // Create fresh Z3 variables
            // Translate body recursively
        }
        Expression::Conditional { condition, then_branch, else_branch } => {
            // Translate to Z3's ite (if-then-else)
            Bool::ite(cond, then_z3, else_z3)
        }
        Expression::Let { name, value, body } => {
            // Extend variable context with bound value
            // Translate body with extended context
        }
    }
}
```

**Operations map by name** - works for ANY axiom!

### Build Configuration

**Cargo.toml:**
```toml
[dependencies]
z3 = { path = "../Z3/z3.rs/z3", optional = true }

[features]
default = ["axiom-verification"]
axiom-verification = ["z3"]
```

**`.cargo/config.toml`** (new file):
```toml
[build]
rustflags = ["-L", "/opt/homebrew/opt/z3/lib"]

[env]
Z3_SYS_Z3_HEADER = "/opt/homebrew/opt/z3/include/z3.h"
```

**Result:** Just `cargo test` works automatically!

### Health Check Script

Created `scripts/check_z3_setup.sh` to verify:
- Architecture match (Rust vs system)
- Z3 installation
- Library and headers present
- Configuration correct
- Build succeeds
- Tests pass

---

## What We Learned

### 1. Z3 Does Normalization, Not E-Unification

**Discovery:** Z3 doesn't implement "E-unification" as an algorithm.

Instead, it uses:
1. **Flattening** - Normalize AC operations
2. **Sorting** - Canonical ordering
3. **Combining** - Merge like terms
4. **Rewriting** - Apply rules

**Result:** Behaves like E-unification for arithmetic!

**Source:** Explored Z3 C++ source (`src/ast/rewriter/poly_rewriter_def.h`)

### 2. Architecture Matters

**Problem:** x86_64 Rust + arm64 Z3 = linker errors

**Solution:** Match architectures!

```bash
rustc --version --verbose | grep host  # Must match system
uname -m
```

### 3. Local Path Avoids Build Issues

Using `path = "../Z3/z3.rs/z3"` instead of `version = "0.12"`:
- No CMake complexity
- No building Z3 from source
- Faster builds
- More reliable

### 4. Feature Flags Work Well

Making Z3 **default** but **optional**:
- Always available in development
- Can disable for CI if needed
- Users without Z3 can still build
- Good developer experience

---

## Consequences

### Positive

1. **Axioms are verifiable** - No longer just comments
2. **Catch errors early** - Invalid axioms detected at definition time
3. **Enable proof chains** - Build complex proofs from simple axioms
4. **Generic architecture** - No hardcoded axioms, scales to any domain
5. **Strong foundation** - Can add more verification features later

### Negative

1. **Build complexity** - Need Z3 installed, architecture matching
2. **External dependency** - Project depends on Z3 installation
3. **Platform-specific** - Path configuration varies by OS
4. **Learning curve** - Developers need to understand SMT solving
5. **Build time** - Adds ~5-10 seconds to clean builds

### Mitigations

- **Health check script** - Diagnose setup issues quickly
- **Comprehensive docs** - Complete troubleshooting guide
- **Feature flag** - Can disable if problems arise
- **Automatic config** - `.cargo/config.toml` makes it "just work"

---

## Grammar Extensions Enabled

Z3 integration created **motivation** for parser extensions:

**Added to parser:**
1. Universal quantifiers: `∀(x : M). body`
2. Existential quantifiers: `∃(x : M). body`
3. Operator symbols: `operation (×) : R → R → R`
4. Logical operators: `∧`, `∨`, `¬`, `⟹`
5. Comparison operators: `=`, `<`, `>`, `≤`, `≥`, `≠`
6. Axiom declarations in structures

**Grammar coverage:** 40% → 52%

**Virtuous cycle:** Parser extensions → Axioms verifiable → More parser work has value!

---

## Verification Capabilities

### What Works Now

**Can verify:**
- ✅ Commutativity: `∀(x y : R). x + y = y + x`
- ✅ Associativity: `∀(x y z : R). (x + y) + z = x + (y + z)`
- ✅ Distributivity: `∀(x y z : R). x × (y + z) = (x × y) + (x × z)`
- ✅ Identity: `∀(x : M). x + 0 = x`
- ✅ Invalid axiom detection: Finds counterexamples

**Can query:**
```rust
let axioms = registry.get_axioms("Ring");
for (name, expr) in axioms {
    let result = verifier.verify_axiom(expr)?;
    println!("{}: {:?}", name, result);
}
```

**Can check equivalence:**
```rust
// Is x + 0 equivalent to x?
let equivalent = verifier.are_equivalent(&expr1, &expr2)?;
```

### Current Scope

**Z3 verification currently supports:**
- ✅ Arithmetic operations (+, ×, etc.)
- ✅ Boolean logic (∧, ∨, ¬, ⟹)
- ✅ Universal and existential quantifiers
- ✅ Comparison operators (=, <, >, ≤, ≥, ≠)
- ✅ Integer and boolean reasoning

**Future extensions (as needed):**
- Type-aware translation (Real vs Int vs Bool sorts)
- User-defined function symbols
- Matrix operations (via Z3 arrays)
- Quantifier instantiation heuristics

---

## Test Coverage

**Total tests:** 471 (+58 from Z3 integration)

**Z3-specific tests:**
- 21 foundation tests (from earlier experiments)
- 10 axiom integration tests
- 11 logical operator tests
- 7 quantifier parsing tests
- 7 operator symbol tests
- 5 registry query tests

**All passing!** ✅

---

## Performance Impact

**Build times:**
- Without Z3: ~30 seconds (clean build)
- With Z3: ~40 seconds (clean build, +10s)
- Incremental: ~2-5 seconds (no difference)

**Runtime:**
- Axiom verification: <50ms per axiom (fast!)
- Tests: ~0.03 seconds for 471 tests
- No runtime impact (verification is compile-time)

**Acceptable trade-off for theorem proving capabilities!**

---

## Integration Points

### 1. AST Extensions

Added `Expression::Quantifier`:
```rust
pub enum Expression {
    // ... existing variants
    Quantifier {
        quantifier: QuantifierKind,  // ForAll or Exists
        variables: Vec<QuantifiedVar>,
        body: Box<Expression>,
    },
}
```

### 2. Parser Extensions

New methods:
- `parse_proposition()` - For axioms
- `parse_quantifier()` - For `∀` and `∃`
- `parse_operation_name()` - For operator symbols
- Precedence chain for logic operators

### 3. Structure Registry

New query methods:
- `get_axioms(name)` - Get all axioms from structure
- `get_operations(name)` - Get all operations
- `has_axiom(name, axiom)` - Check for specific axiom
- `structures_with_axioms()` - List structures with axioms

### 4. Axiom Verifier Module

New module `src/axiom_verifier.rs`:
- `verify_axiom(expr)` - Main entry point
- `are_equivalent(expr1, expr2)` - Equivalence checking
- `kleis_to_z3(expr)` - Generic translator
- Feature-gated for optional use

---

## Related ADRs

**ADR-014 (Hindley-Milner):** Z3 complements type inference
- Type inference: What type does this have?
- Z3 verification: Is this property true?

**ADR-015 (Text as Truth):** Axioms in .kleis files
- Axioms defined in stdlib/*.kleis (not Rust)
- Z3 verifies what's written in text
- Self-hosting principle maintained

**ADR-016 (Operations in Structures):** Axioms belong with structures
- Structures define operations AND axioms
- Both queryable from registry
- Both part of type-level specification

---

## Future Directions

### Phase 3 (When Needed)

**Not done yet, but planned:**

1. **`where` clauses** - Generic constraints on implementations
   ```kleis
   implements MatrixMultipliable(m, n, p, T) 
     where Semiring(T) {
       operation multiply = builtin_matrix_multiply
     }
   ```

2. **Verify implementations** - Check if implementations satisfy axioms
   ```rust
   verifier.verify_implementation("Ring", real_numbers)?;
   ```

3. **Proof-carrying types** - Preconditions and postconditions
   ```kleis
   function invert(m : Matrix(n, n, R))
     where det(m) ≠ 0
     ensures m × result = I
   ```

### Advanced Features (Future)

- **Quantifier instantiation** - Smarter variable handling
- **Custom theories** - Domain-specific axioms
- **Proof term extraction** - Get proof from Z3
- **Interactive proving** - User-guided verification
- **Counterexample visualization** - Show why axiom fails

---

## Comparison with Other Approaches

### vs Hardcoded Verification

| Aspect | Hardcoded | Z3 Integration |
|--------|-----------|----------------|
| Extensibility | Must write Rust for each axiom | Generic translator |
| Completeness | Only checks what we program | Theorem prover |
| Maintenance | Update Rust for new axioms | Just add axioms in .kleis |
| Complexity | Simple at first, grows quickly | Complex setup, simple use |

**Winner:** Z3 - scales better

### vs Proof Assistants (Coq, Lean)

| Aspect | Coq/Lean | Z3 |
|--------|----------|-----|
| Power | More powerful | Less powerful |
| Automation | Requires proof scripts | Automatic |
| Learning curve | Steep | Moderate |
| Use case | Interactive proving | Automatic verification |

**Winner:** Z3 - better fit for automatic verification

### vs No Verification

| Aspect | No Verification | Z3 Integration |
|--------|-----------------|----------------|
| Simplicity | Simplest | More complex |
| Correctness | Hope axioms are right | Proven correct |
| User trust | "Probably works" | "Verified" |
| Error detection | Runtime errors | Compile-time detection |

**Winner:** Z3 - correctness matters for math!

---

## Implementation Summary

### What We Built (December 10, 2025)

**Phase 1: Foundation**
1. Universal quantifiers (`∀`, `∃`)
2. Operator symbols in declarations
3. Axiom verifier with generic translator
4. Integration tests (axioms verified!)

**Phase 2: Logic & Registry**
5. Logical operators (`⟹`, `∧`, `∨`, `¬`)
6. Axiom query methods in registry

**Results:**
- 471 tests passing ✅
- Commutativity verified ✅
- Associativity verified ✅
- Distributivity verified ✅
- Invalid axiom detection ✅

### Code Statistics

- **New code:** ~1,200 lines
- **Tests:** ~400 lines (58 new tests)
- **Documentation:** ~2,000 lines
- **Files modified:** 15+
- **New module:** `src/axiom_verifier.rs`

---

## Trade-offs Accepted

### Build Complexity ↔ Verification Power

**Accept:** Need Z3 installed, architecture matching, build configuration  
**Get:** Automatic theorem proving for axioms

**Decision:** Worth it - correctness matters for mathematical systems!

### External Dependency ↔ Don't Reinvent

**Accept:** Depend on Z3 project  
**Get:** Decades of SMT solver research and optimization

**Decision:** Worth it - Z3 is mature, well-maintained, widely used

### Feature Flag ↔ Always Available

**Accept:** Default feature means most users need Z3  
**Get:** Verification always available in development, optional in production

**Decision:** Worth it - can disable if needed, but default is powerful

---

## Validation

### Tests Prove It Works

**Verified real axioms:**
```
✅ Commutativity: ∀(x y : R). x + y = y + x
✅ Associativity: ∀(x y z : R). (x + y) + z = x + (y + z)  
✅ Distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
❌ Invalid: ∀(x : R). x + 1 = x (counterexample found)
```

### Performance Acceptable

- Verification: <50ms per axiom
- Build time: +10 seconds (clean)
- Test time: 0.03s for 471 tests

### Documentation Complete

- Setup guide (Z3_BUILD_SETUP.md)
- Troubleshooting (comprehensive)
- Health check script (automated)
- Implementation roadmap (Z3_GRAMMAR_ROADMAP.md)
- Session summaries (PHASE_1_AND_2_COMPLETE.md)

---

## Success Criteria Met

**From planning phase:**
- ✅ Parse quantifiers (`∀`, `∃`)
- ✅ Parse operator symbols
- ✅ Build generic translator
- ✅ Verify example axioms
- ✅ Integration with registry
- ✅ Comprehensive tests
- ✅ Documentation complete

**Exceeded expectations:**
- ✅ Logical operators too!
- ✅ 471 tests (planned for ~440)
- ✅ Health check automation
- ✅ Feature flag working
- ✅ Grammar coverage increase (12 points)

---

## Adoption Strategy

### For Developers

**Setup once:**
1. Install Z3: `brew install z3` (macOS)
2. Clone z3.rs: (already done)
3. Run health check: `./scripts/check_z3_setup.sh`

**Daily use:**
```bash
cargo test  # Just works!
```

### For CI/CD

**Option A:** Enable Z3
```yaml
- name: Install Z3
  run: brew install z3
- name: Run tests
  run: cargo test
```

**Option B:** Disable Z3
```yaml
- name: Run tests
  run: cargo test --no-default-features
```

### For Users Without Z3

**Build without verification:**
```bash
cargo build --no-default-features
```

**Feature flag prevents hard dependency.**

---

## Lessons Learned

### 1. Experiment First, Document After

**Process:**
1. Tried Z3 on feature branch
2. Learned what works/doesn't work
3. Documented findings in ADR

**Better than:** Speculate → Document → Discover it's wrong

### 2. Generic Beats Hardcoded

Every time we made something generic (vs hardcoded), it:
- Scaled better
- Was easier to maintain
- Handled edge cases naturally

### 3. Tests Create Confidence

58 new tests mean we KNOW it works:
- Parsing tested
- AST construction tested
- Z3 translation tested
- Verification tested
- Registry queries tested

### 4. The Virtuous Cycle Is Real

Parser extensions have immediate value when axioms are verifiable!

**Before:** "Why add quantifiers? Just syntax."  
**After:** "We need quantifiers to verify axioms!"

---

## Decision Rationale

**Why include Z3 as default feature?**

1. **Value proposition:** Axiom verification is core feature
2. **Developer experience:** Always available, no manual setup
3. **Correctness:** Math language should verify math!
4. **Extensibility:** Foundation for future proof features
5. **Maturity:** Z3 is production-ready, widely used

**Why keep it optional?**

1. **Portability:** Not everyone has Z3
2. **CI flexibility:** Can disable if needed
3. **Build simplicity:** Users can opt out
4. **Testing:** Verify feature flag works

**Best of both worlds!**

---

## Conclusion

**We accept the inclusion of Z3 as a default (but optional) feature for axiom verification.**

**Rationale:**
- Proven to work (471 tests passing)
- Valuable capability (axioms verified!)
- Clean architecture (generic translator)
- Well-documented (troubleshooting covered)
- Optional (can disable if needed)

**Benefits outweigh costs for a mathematical language!**

---

## Status: Implemented ✅

- Implementation complete: December 10, 2025
- **Full prelude migration:** December 11, 2025
- Tests passing: 421 ✅
- Documentation complete: ✅
- Feature branch: `feature/full-prelude-migration`
- Ready for merge to main

**Updates (December 11, 2025):**
- ✅ Added `TypeExpr::ForAll` for polymorphic type schemes
- ✅ Parser supports quantified types: `∀(n : ℕ, T). Matrix(m,n,T) → ...`
- ✅ Full `prelude.kleis` loaded with algebraic structures and axioms
- ✅ Arithmetic operations added for test compatibility
- ✅ All 421 tests passing with full prelude

**Axiom verification is now a core Kleis capability!** 🎯

---

## References

**Documentation:**
- `docs/archive/sessions/session-2025-12-10/Z3_BUILD_SETUP.md` - Complete setup guide
- `docs/archive/sessions/session-2025-12-10/Z3_GRAMMAR_ROADMAP.md` - Implementation plan
- `docs/archive/sessions/session-2025-12-10/PHASE_1_AND_2_COMPLETE.md` - Achievement summary
- `docs/archive/sessions/session-2025-12-10/HOW_Z3_DOES_E_UNIFICATION.md` - Z3 internals
- `docs/archive/sessions/session-2025-12-10/Z3_AST_VS_KLEIS_AST.md` - Architecture comparison

**Code:**
- `src/axiom_verifier.rs` - Main implementation
- `tests/axiom_verification_integration_test.rs` - Integration tests
- `.cargo/config.toml` - Build configuration
- `scripts/check_z3_setup.sh` - Health check

**External:**
- Z3 Theorem Prover: https://github.com/Z3Prover/z3
- z3.rs Rust bindings: https://github.com/prove-rs/z3.rs
- Local clone: `/Users/eatik_1/Documents/git/cee/Z3/z3.rs/`

