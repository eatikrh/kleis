# Z3 Integration Notes - Dec 10, 2024

## Investigation Results

### What We Found

**Z3 is installed via Homebrew:**
- Version: 4.15.4
- Include: `/opt/homebrew/opt/z3/include` (has z3.h ✅)
- Lib: `/opt/homebrew/opt/z3/lib` (has libz3.dylib ✅)

**Z3 Rust crate issue:**
- Current: z3 v0.12.1 (uses z3-sys v0.8.1)
- Available: z3 v0.19.5 (newer)
- Problem: z3-sys tries to build from source via CMake
- Even with Z3_SYS_Z3_HEADER set, still tries to build

### Build Error

```
CMake Error: Compatibility with CMake < 3.5 has been removed
```

The bundled Z3 source in z3-sys v0.8.1 is old and has CMake compatibility issues.

### Possible Solutions

**Option 1: Try newer z3 crate version**
```toml
z3 = { version = "0.19", optional = true }
```

**Option 2: Use vcpkg feature**
```toml
z3 = { version = "0.12", features = ["vcpkg"], optional = true }
```

**Option 3: Use local z3.rs clone**
```toml
z3 = { path = "../../Z3/z3.rs/z3", optional = true }
```

**Option 4: Set RUSTFLAGS to link system Z3**
```bash
export RUSTFLAGS="-L /opt/homebrew/opt/z3/lib"
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h
```

---

## Experiments Created

**File:** `tests/z3_axiom_experiments.rs` (on feature branch)

**7 tests exploring axiom verification:**
1. `test_z3_basic_arithmetic` - Verify x + 0 = x
2. `test_z3_commutativity` - Verify x + y = y + x
3. `test_z3_associativity` - Verify (x+y)+z = x+(y+z)
4. `test_z3_distributivity` - Verify x(y+z) = xy + xz
5. `test_z3_multiplicative_identity` - Verify x × 1 = x
6. `test_z3_find_counterexample` - Test error detection
7. `test_z3_ring_axioms_together` - Multiple axioms at once

**Pattern used:**
```rust
// To verify axiom: ∀x. P(x)
// Check that ¬P(x) is UNSATISFIABLE
solver.assert(&axiom.not());
match solver.check() {
    SatResult::Unsat => println!("✅ Axiom holds"),
    SatResult::Sat => println!("❌ Counterexample found"),
    ...
}
```

---

## Why Z3 Integration Matters

### The Virtuous Cycle

**Without Z3:**
- Axioms are just documentation
- Parser priority for `∀`, `⟹` is LOW
- Grammar coverage: 40-45%

**With Z3:**
- Axioms become **verifiable**
- Parser priority for `∀`, `⟹` becomes HIGH
- Grammar coverage jumps to 60-65%
- Features interconnect and create momentum

### What It Enables

**1. Verify Structure Implementations:**
```kleis
implements Ring(MyType) {
  operation plus = my_add
  operation times = my_mul
}

verify MyType satisfies Ring
// Check: Does my_add + my_mul satisfy all Ring axioms?
```

**2. Guide Simplification:**
```kleis
expression: (x + 0) * y
// Z3 verifies: x + 0 = x (by identity axiom)
simplified: x * y ✅
```

**3. Find Counterexamples:**
```kleis
claim: ∀x. x + 1 = x
// Z3 finds: No such x exists ❌
```

---

## Next Steps for Next Session

### Investigation Needed (30 min)

Try different approaches to get z3-sys working:

1. **Try newer version:**
   ```bash
   # In Cargo.toml
   z3 = { version = "0.19", optional = true }
   cargo test --test z3_axiom_experiments --features axiom-verification
   ```

2. **Try local path:**
   ```bash
   z3 = { path = "../../Z3/z3.rs/z3", optional = true }
   ```

3. **Check z3-sys documentation:**
   Look at build.rs in z3-sys to see what env vars it checks

### Once Working

1. **Run experiments** - See if basic axioms verify
2. **Build generic translator** - kleis_to_z3() function
3. **Integrate with structure registry** - Make axioms accessible
4. **Extend parser** - Add `∀` and `⟹` syntax
5. **Write ADR-022** - Document decision with real experience

---

## Branch Status

**Branch:** `feature/full-prelude-migration`  
**Commits:** 3
- Z3 task planning documentation
- Z3 dependency and experiments added
- Build issue documented

**Status:** WIP - needs build configuration fix before proceeding

**Tests ready to run once Z3 builds!**

---

## For ADR-022

**Things to document based on this exploration:**

**Pros:**
- Powerful verification capabilities
- Proven technology (Microsoft Research)
- Good Rust bindings (in principle)

**Cons:**
- Build complexity (CMake, system vs static linking)
- External dependency management
- Version compatibility issues

**Decision factors:**
- Is verification worth the build complexity?
- Optional feature vs required?
- Alternative approaches (pattern matching, manual checking)?

**Learn by doing - these real challenges inform the decision!** 🎯

