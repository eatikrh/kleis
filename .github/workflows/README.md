# GitHub Actions CI/CD

This directory contains GitHub Actions workflows for automated testing and building.

## Workflows

### `ci.yml` - Continuous Integration

**Triggers:**
- Push to `main` branch
- Pull requests to `main`

**Jobs:**

#### 1. Test (Multi-platform)
Runs on: Ubuntu, macOS

- ✅ Format check (`cargo fmt`)
- ✅ Clippy linting (`cargo clippy`)
- ✅ Build project (`cargo build`)
- ✅ Run tests (`cargo test --lib`)
- ✅ Build examples
- 💾 Caches: Cargo registry, dependencies, build artifacts

#### 2. Build Binaries
Runs on: Ubuntu

- ✅ Build server in release mode
- ✅ Verify binary created

#### 3. Documentation Check
Runs on: Ubuntu

- ✅ Build Rust docs (`cargo doc`)
- ✅ Verify key markdown files exist
- ✅ Check documentation structure

#### 4. Template Coverage
Runs on: Ubuntu

- ✅ Run template-specific tests
- ✅ Report template statistics
- ✅ Count template functions and tests

## Status Badge

Add to README.md:
```markdown
[![CI](https://github.com/YOUR_USERNAME/kleis/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/kleis/actions)
```

## Local Testing

Run the same checks locally before pushing:

```bash
# Format check
cargo fmt -- --check

# Clippy
cargo clippy --all-targets --all-features

# Build
cargo build

# Tests
cargo test --lib

# Examples
cargo build --examples
```

## Caching Strategy

The workflow caches:
1. **Cargo registry** (~/.cargo/registry)
2. **Cargo git index** (~/.cargo/git)
3. **Build artifacts** (target/)

This speeds up CI runs significantly (2-5x faster after first run).

## Notes

- Clippy is set to `continue-on-error: true` to avoid blocking on warnings
- Examples build with `continue-on-error: true` in case of optional dependencies
- Tests run only on lib (`--lib`) to avoid bin/example compilation issues
- Multi-platform testing ensures compatibility

## Future Enhancements

- [ ] Add Windows testing
- [ ] Code coverage reporting (tarpaulin)
- [ ] Benchmark tracking
- [ ] Release automation
- [ ] Documentation deployment
- [ ] Security audit (cargo-audit)

