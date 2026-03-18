# Z3 Build Setup - Complete Reference

**Date:** December 10, 2025  
**Branch:** `feature/full-prelude-migration`  
**Status:** Working configuration documented ✅

---

## TL;DR - How to Build with Z3

### On This System (macOS ARM64)

```bash
# Just run cargo commands normally - it should "just work"
cargo build
cargo test
cargo test --test z3_axiom_experiments
```

**If you see linking errors**, the `.cargo/config.toml` needs updating (see below).

---

## Architecture Requirements

**CRITICAL: Rust toolchain must match system architecture!**

```bash
# Check system architecture
uname -m
# Should show: arm64 (on Apple Silicon)

# Check Rust architecture
rustc --version --verbose | grep host
# Should show: aarch64-apple-darwin (on Apple Silicon)

# If mismatch, switch Rust toolchain:
rustup default stable-aarch64-apple-darwin
cargo clean
```

**Why this matters:** Linking arm64 Z3 library with x86_64 Rust = linker errors!

---

## Z3 Installation

### macOS (Homebrew)

```bash
# Install Z3
brew install z3

# Verify installation
which z3
# Should show: /opt/homebrew/bin/z3 (ARM) or /usr/local/bin/z3 (Intel)

z3 --version
# Should show: Z3 version 4.15.x

# Check library location
ls -la /opt/homebrew/opt/z3/lib/libz3.dylib    # ARM
# or
ls -la /usr/local/opt/z3/lib/libz3.dylib        # Intel
```

### Linux (Ubuntu/Debian)

```bash
# Install Z3
sudo apt-get install libz3-dev

# Verify
which z3
ls /usr/lib/libz3.so  # or /usr/lib/x86_64-linux-gnu/libz3.so
```

### Windows

```bash
# Download from https://github.com/Z3Prover/z3/releases
# Extract to C:\z3
# Add C:\z3\bin to PATH
```

---

## Project Configuration

### 1. Cargo.toml Configuration

**Location:** `Cargo.toml`

```toml
[dependencies]
# Using local clone to avoid z3-sys build issues
z3 = { path = "../Z3/z3.rs/z3", optional = true }

[features]
# Z3 is enabled by default but can be disabled
default = ["axiom-verification"]
axiom-verification = ["z3"]
```

**Why local path?**
- z3-sys from crates.io tries to build Z3 from source
- CMake compatibility issues
- Local clone avoids build complexity

**Where is the local clone?**
```
/Users/eatik_1/Documents/git/cee/Z3/z3.rs/
├── z3/           ← We depend on this
└── z3-sys/
```

### 2. Cargo Build Configuration

**Location:** `.cargo/config.toml` (created in this session)

```toml
[build]
# Link Z3 library
rustflags = ["-L", "/opt/homebrew/opt/z3/lib"]

[env]
# Z3 header for z3-sys binding generation
Z3_SYS_Z3_HEADER = "/opt/homebrew/opt/z3/include/z3.h"
```

**This file makes it work automatically!** No need for environment variables.

**For different systems:**

| System | Library Path | Header Path |
|--------|--------------|-------------|
| macOS ARM (Homebrew) | `/opt/homebrew/opt/z3/lib` | `/opt/homebrew/opt/z3/include/z3.h` |
| macOS Intel (Homebrew) | `/usr/local/opt/z3/lib` | `/usr/local/opt/z3/include/z3.h` |
| Linux (typical) | `/usr/lib` | `/usr/include/z3.h` |
| Windows | `C:\z3\bin` | `C:\z3\include\z3.h` |

---

## Running Tests

### All Tests (Including Z3)

```bash
# Z3 is enabled by default
cargo test

# Just the Z3 tests (21 tests)
cargo test --test z3_axiom_experiments \
           --test z3_kleis_grammar_tests \
           --test z3_e_unification_tests

# Individual test suites
cargo test --test z3_axiom_experiments       # 7 axiom tests
cargo test --test z3_kleis_grammar_tests     # 7 grammar tests
cargo test --test z3_e_unification_tests     # 7 simplification tests
```

### Without Z3 (Optional)

```bash
# Disable default features
cargo test --no-default-features
cargo build --no-default-features
```

### Verbose Output

```bash
# See Z3 solver output
cargo test --test z3_axiom_experiments -- --nocapture

# See what Z3 is proving
cargo test test_z3_associativity -- --nocapture --show-output
```

---

## Troubleshooting

### Error: "library 'z3' not found"

**Problem:** Linker can't find libz3

**Solution 1:** Check `.cargo/config.toml` has correct path

```bash
# Verify Z3 library exists
ls -la /opt/homebrew/opt/z3/lib/libz3.dylib   # macOS ARM
ls -la /usr/local/opt/z3/lib/libz3.dylib      # macOS Intel
ls /usr/lib/libz3.so                           # Linux
```

**Solution 2:** Set RUSTFLAGS manually (if config.toml doesn't work)

```bash
RUSTFLAGS="-L /opt/homebrew/opt/z3/lib" cargo test
```

### Error: "'z3.h' file not found"

**Problem:** z3-sys can't find Z3 header

**Solution 1:** Check `.cargo/config.toml` has Z3_SYS_Z3_HEADER

**Solution 2:** Set environment variable

```bash
Z3_SYS_Z3_HEADER="/opt/homebrew/opt/z3/include/z3.h" cargo build
```

### Error: Architecture mismatch

**Problem:** Rust is x86_64 but Z3 is arm64 (or vice versa)

**Solution:** Switch Rust toolchain to match system

```bash
# For Apple Silicon (arm64)
rustup default stable-aarch64-apple-darwin

# For Intel Mac (x86_64)
rustup default stable-x86_64-apple-darwin

# Clean and rebuild
cargo clean
cargo build
```

### Error: z3-sys build fails with CMake errors

**Problem:** Trying to use z3 from crates.io instead of local path

**Solution:** Verify Cargo.toml has:

```toml
z3 = { path = "../Z3/z3.rs/z3", optional = true }
```

NOT:
```toml
z3 = { version = "0.12", optional = true }  # ❌ This tries to build from source
```

---

## Build Process Explained

### What Happens When You Run `cargo build`

1. **Feature resolution:**
   - `default` feature is enabled
   - `axiom-verification` feature is enabled
   - `z3` dependency is included

2. **z3-sys build (runs once):**
   - Reads `Z3_SYS_Z3_HEADER` from `.cargo/config.toml`
   - Generates Rust bindings from `z3.h` using bindgen
   - Creates `libz3_sys.rlib`

3. **z3 crate build:**
   - Compiles Rust API wrapper
   - Creates `libz3.rlib`

4. **kleis build:**
   - Compiles kleis with Z3 integration
   - Links against z3, z3-sys, and system libz3

5. **Final linking:**
   - Uses `-L /opt/homebrew/opt/z3/lib` from `.cargo/config.toml`
   - Links system libz3.dylib
   - Produces final binary

### Why Local Path Works

```
Cargo.toml: z3 = { path = "../Z3/z3.rs/z3" }
             ↓
Uses source from: /Users/eatik_1/Documents/git/cee/Z3/z3.rs/z3/
             ↓
That depends on: z3-sys = { path = "../z3-sys" }
             ↓
z3-sys uses: Installed system Z3 (Homebrew)
             ↓
Links against: /opt/homebrew/opt/z3/lib/libz3.dylib
```

**No CMake, no building Z3 from source, just works!**

---

## CI/CD Considerations

### GitHub Actions (Future)

For CI, you'll need:

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest  # or ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      # Install Z3
      - name: Install Z3
        run: brew install z3  # macOS
        # or: sudo apt-get install libz3-dev  # Linux
      
      # No config needed - .cargo/config.toml handles it!
      - name: Build
        run: cargo build --verbose
      
      - name: Run tests
        run: cargo test --verbose
```

### Alternative: Disable Z3 in CI

```yaml
- name: Run tests (without Z3)
  run: cargo test --no-default-features --verbose
```

---

## Environment Summary

### What You Need

✅ **System:** macOS on Apple Silicon (arm64)  
✅ **Rust:** aarch64-apple-darwin toolchain  
✅ **Z3:** Version 4.15.x from Homebrew  
✅ **Config:** `.cargo/config.toml` with paths  
✅ **Source:** Local z3.rs clone in `../Z3/z3.rs/`

### Quick Health Check

```bash
# Run this to verify everything is set up correctly
cd /Users/eatik_1/Documents/git/cee/kleis

# Check architecture
echo "System: $(uname -m)"
echo "Rust: $(rustc --version --verbose | grep host | awk '{print $2}')"

# Check Z3
echo "Z3 installed: $(which z3)"
echo "Z3 version: $(z3 --version)"

# Check library
echo "Library exists: $(ls /opt/homebrew/opt/z3/lib/libz3.dylib 2>/dev/null && echo YES || echo NO)"

# Check config
echo "Config exists: $(ls .cargo/config.toml 2>/dev/null && echo YES || echo NO)"

# Try building
cargo build --quiet && echo "✅ Build successful!" || echo "❌ Build failed"

# Try running Z3 tests
cargo test --quiet --test z3_axiom_experiments && echo "✅ Z3 tests pass!" || echo "❌ Z3 tests failed"
```

**Expected output:**
```
System: arm64
Rust: aarch64-apple-darwin
Z3 installed: /opt/homebrew/bin/z3
Z3 version: Z3 version 4.15.4 ...
Library exists: YES
Config exists: YES
✅ Build successful!
✅ Z3 tests pass!
```

---

## Next Session Quick Start

```bash
# 1. Switch to feature branch
cd /Users/eatik_1/Documents/git/cee/kleis
git checkout feature/full-prelude-migration

# 2. Verify Z3 works (should "just work" with .cargo/config.toml)
cargo test --test z3_axiom_experiments

# 3. If it works, you're ready! 🎉
# 4. If it fails, run the health check above and troubleshoot
```

**That's it! The `.cargo/config.toml` makes it automatic.** 🚀

---

## Files Reference

**Project files:**
- `Cargo.toml` - Z3 dependency and feature flag
- `.cargo/config.toml` - Build configuration (NEW in this session)
- `tests/z3_axiom_experiments.rs` - 7 axiom tests
- `tests/z3_kleis_grammar_tests.rs` - 7 grammar tests
- `tests/z3_e_unification_tests.rs` - 7 simplification tests

**External files:**
- `/Users/eatik_1/Documents/git/cee/Z3/z3.rs/` - Local z3 crate clone
- `/opt/homebrew/opt/z3/` - Homebrew Z3 installation
- `~/.cargo/config.toml` - Global cargo config (not used)

---

## Summary

**Before this session:**
- Had to set `RUSTFLAGS` and `Z3_SYS_Z3_HEADER` manually every time
- Z3 was optional and off by default
- No documentation on setup

**After this session:**
- ✅ `.cargo/config.toml` makes linking automatic
- ✅ Z3 is default feature (on by default)
- ✅ Complete documentation for troubleshooting
- ✅ Clear commands for next time

**Just run `cargo test` and it works!** 🎯

