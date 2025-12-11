# Quality Gates for Kleis Development

**Purpose:** Ensure code quality before committing  
**Updated:** December 10, 2024 - Added Z3 verification tests

---

## Pre-Commit Quality Gates

**Run these commands before committing any Rust code changes:**

### 1. Format Code (Required)

```bash
cargo fmt --all
```

**Purpose:** Ensures consistent code style across ALL crates  
**CI Check:** `cargo fmt -- --check`  
**CRITICAL:** Use `--all` flag to format workspace members (including render/)

---

### 2. Run Clippy (Required)

```bash
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h && cargo clippy --all-targets --all-features
```

**Purpose:** Catches common mistakes and anti-patterns  
**CI Check:** `cargo clippy --all-targets --all-features -- -D warnings`  
**Action:** Fix all clippy warnings before committing

**Platform-specific Z3 header paths:**
- **macOS ARM:** `/opt/homebrew/opt/z3/include/z3.h`
- **macOS Intel:** `/usr/local/opt/z3/include/z3.h`
- **Linux:** `/usr/include/z3.h`

---

### 3. Run Library Tests (Required)

```bash
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h && cargo test --lib
```

**Purpose:** Ensures no regressions in core functionality  
**CI Check:** `cargo test --lib --verbose`  
**Requirement:** Must pass (currently 421+ tests)

**Platform-specific Z3 header paths:**
- **macOS ARM:** `/opt/homebrew/opt/z3/include/z3.h`
- **macOS Intel:** `/usr/local/opt/z3/include/z3.h`
- **Linux:** `/usr/include/z3.h`

---

### 4. Run Z3 Verification Tests (Required if axiom_verifier.rs changed)

```bash
export Z3_SYS_Z3_HEADER=/opt/homebrew/include/z3.h  # macOS ARM
# OR: export Z3_SYS_Z3_HEADER=/usr/local/include/z3.h  # macOS Intel
# OR: export Z3_SYS_Z3_HEADER=/usr/include/z3.h  # Linux

cargo test --features axiom-verification
```

**Purpose:** Verifies Z3 integration and dependency loading  
**When:** Required if changes to `src/axiom_verifier.rs` or structure loading  
**CI Check:** GitHub CI runs this with Z3 installed

**Critical tests:**
- `tests/extends_z3_test.rs` - Inheritance
- `tests/where_constraint_z3_test.rs` - Constraints
- `tests/nested_structure_z3_test.rs` - Nested structures
- `tests/over_clause_z3_test.rs` - Over clause
- `tests/z3_dependency_proof_tests.rs` - Dependency proofs

**Known issue (Dec 10, 2024):**
- 3 out of 5 proof tests pass rigorously
- 2 tests need operation registration fixes (see NEXT_SESSION_PRIORITY.md)
- All other Z3 tests pass

---

## Quality Gate Process

### Before Committing .rs Files

```bash
# 1. Format
cargo fmt --all

# 2. Lint (with Z3 support)
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h && cargo clippy --all-targets --all-features

# 3. Test core (with Z3 support)
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h && cargo test --lib

# 4. Test Z3 verification (if applicable)
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h && cargo test --features axiom-verification

# 5. Fix any errors/warnings from steps 2-4

# 6. THEN commit
git add <files>
git commit -m "..."
```

---

## Platform-Specific Z3 Setup

### macOS ARM (Apple Silicon)

```bash
# Install Z3
brew install z3

# Set environment variable
export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h

# Add to .zshrc or .bashrc:
echo 'export Z3_SYS_Z3_HEADER=/opt/homebrew/opt/z3/include/z3.h' >> ~/.zshrc
```

### macOS Intel

```bash
# Install Z3
brew install z3

# Set environment variable
export Z3_SYS_Z3_HEADER=/usr/local/opt/z3/include/z3.h

# Add to .bash_profile:
echo 'export Z3_SYS_Z3_HEADER=/usr/local/opt/z3/include/z3.h' >> ~/.bash_profile
```

### Linux (Ubuntu/Debian)

```bash
# Install Z3
sudo apt-get install z3 libz3-dev

# Set environment variable
export Z3_SYS_Z3_HEADER=/usr/include/z3.h

# Add to .bashrc:
echo 'export Z3_SYS_Z3_HEADER=/usr/include/z3.h' >> ~/.bashrc
```

---

## GitHub CI Configuration

GitHub Actions automatically sets Z3 variables per platform. See `.github/workflows/rust.yml`.

---

## Exception Handling

### When to Skip Quality Gates

**Never skip for:**
- Changes to core type system (`src/type_inference.rs`, `src/type_checker.rs`)
- Changes to axiom verification (`src/axiom_verifier.rs`)
- Changes to parser (`src/kleis_parser.rs`)
- Changes to AST (`src/ast.rs`, `src/kleis_ast.rs`)

**May skip clippy warnings for:**
- Legacy code with known issues
- External dependencies

**May skip Z3 tests if:**
- Z3 not installed locally (but CI will run them)
- Changes don't affect axiom verification
- Documentation-only changes

---

## Test Failure Policy

### If Tests Fail

1. **DO NOT** weaken assertions to make tests pass
2. **DO** understand root cause
3. **DO** fix the actual issue
4. **DO** document if genuine limitation found
5. **DO** add TODO if fix is complex

### If Z3 Tests Fail

1. Check Z3_SYS_Z3_HEADER is set correctly
2. Check Z3 is installed: `z3 --version`
3. Check test logic is correct
4. If architectural issue, document in NEXT_SESSION_PRIORITY.md
5. Don't commit if critical tests fail

---

## Current Known Issues (Dec 10, 2024)

### Z3 Proof Tests

**Status:** 3 out of 5 pass rigorously

**Passing:**
- ✅ Where constraints
- ✅ Nested structures
- ✅ Over clause

**Need fixes:**
- ⚠️ Extends clause (test setup issue - operations not registered)
- ⚠️ All dependencies together (same issue)

**Action:** See NEXT_SESSION_PRIORITY.md - fix in next session

**Impact:** Does not block current functionality, but limits mathematical rigor claims

---

## Why This Matters

### Professional Development Practice

- Prevents CI failures
- Maintains code quality
- Catches bugs early
- Ensures consistent style

### Mathematical Rigor

- Z3 tests prove formal correctness
- Can't claim "all dependencies work" without passing tests
- Mathematicians require rigorous proofs
- Credibility depends on test quality

---

## References

- Rust Quality Gates: Per project rules in `.cursor/rules`
- Z3 Setup: `.cargo/config.toml`
- CI Configuration: `.github/workflows/rust.yml`
- Test Standards: `NEXT_SESSION_PRIORITY.md` (current gaps)

---

**Last Updated:** December 10, 2024  
**Next Review:** After fixing extends tests (NEXT_SESSION_PRIORITY.md)

