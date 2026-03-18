# Vendored Dependencies

This directory contains third-party dependencies that are vendored into the Kleis repository.

---

## z3/ and z3-sys/

**Source:** https://github.com/prove-rs/z3.rs  
**Version:** Latest from main branch (as of Dec 10, 2024)  
**License:** MIT

**Why vendored:**
- The published `z3` crate (v0.12) has a different API with explicit Context lifetimes
- The z3.rs main branch has a simpler API without lifetime management
- Our code is written for the simpler API
- Vendoring makes the repo self-contained and CI simpler

**What's included:**
- `z3/` - High-level Rust bindings for Z3
- `z3-sys/` - Low-level FFI bindings to Z3 C library

**Note:** You still need the Z3 C library installed on your system:
- See: `docs/session-2024-12-10/Z3_BUILD_SETUP.md` for installation instructions
- macOS: `brew install z3`
- Ubuntu: `sudo apt-get install libz3-dev z3`

**Updating:**
If z3.rs upstream changes and we need to update:
```bash
cd /path/to/z3.rs
git pull
cd /path/to/kleis
rm -rf vendor/z3 vendor/z3-sys
cp -r /path/to/z3.rs/z3 vendor/
cp -r /path/to/z3.rs/z3-sys vendor/
```

---

**Last updated:** December 10, 2024

