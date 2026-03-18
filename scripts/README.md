# Kleis Scripts

Utility scripts for development and maintenance.

## Z3 Integration

### check_z3_setup.sh

**Purpose:** Verify Z3 integration is configured correctly

**Usage:**
```bash
./scripts/check_z3_setup.sh
```

**What it checks:**
- ✅ System and Rust architecture match
- ✅ Z3 is installed and accessible
- ✅ Z3 library and headers are present
- ✅ `.cargo/config.toml` is configured
- ✅ Local Z3 Rust bindings are available
- ✅ Project builds successfully
- ✅ Z3 tests pass

**When to run:**
- Setting up a new development environment
- After updating Z3 or Rust toolchain
- When Z3 tests fail unexpectedly
- After switching between branches

**Expected output:**
```
✅ All checks passed! Z3 integration ready 🚀
```

**If checks fail:**
See troubleshooting guide in `docs/session-2025-12-10/Z3_BUILD_SETUP.md`

---

## DLMF Fetching

### fetch_dlmf.py / fetch_dlmf_v2.py

Scripts for fetching mathematical content from DLMF (Digital Library of Mathematical Functions).

See inline documentation for usage.

---

## Adding New Scripts

When adding new scripts:

1. **Make them executable:**
   ```bash
   chmod +x scripts/your_script.sh
   ```

2. **Add usage documentation here**

3. **Include error handling and clear output**

4. **Use the project root as working directory:**
   ```bash
   cd "$(dirname "$0")/.."
   ```
