# Next Session: TODO Review in Rust Files

**Date Created:** December 12, 2024  
**Priority:** High  
**Scope:** All .rs files in the codebase

---

## Task: Review All TODOs in Rust Files

Go through all TODO comments in `.rs` files and:

1. **Categorize** each TODO:
   - Critical (blocking functionality)
   - Important (should be done soon)
   - Nice-to-have (future enhancement)
   - Obsolete (already done or no longer relevant)

2. **Take action:**
   - Fix critical TODOs immediately
   - Create GitHub issues for important ones
   - Document nice-to-have items properly
   - Remove obsolete TODOs

3. **Document:**
   - Create a TODO inventory
   - Prioritize by impact
   - Estimate effort for each

---

## Finding TODOs

```bash
# Find all TODOs in Rust source files
grep -r "TODO\|FIXME\|XXX\|HACK" --include="*.rs" src/ tests/ examples/ | wc -l

# Get detailed list
grep -rn "TODO\|FIXME\|XXX\|HACK" --include="*.rs" src/ tests/ examples/ > /tmp/rust_todos.txt

# Group by file
grep -r "TODO\|FIXME\|XXX\|HACK" --include="*.rs" src/ | cut -d: -f1 | sort | uniq -c | sort -rn
```

---

## Areas to Focus On

1. **src/axiom_verifier.rs** - Any Z3 translation TODOs
2. **src/type_inference.rs** - Type system TODOs  
3. **src/parser.rs** - Parser limitations
4. **src/kleis_parser.rs** - Kleis text parser TODOs
5. **src/template_inference.rs** - Template system TODOs
6. **src/math_layout/*.rs** - Layout and rendering TODOs
7. **tests/*.rs** - Test TODOs (ignored tests, known failures)

---

## Expected Outcomes

- Clear understanding of all pending work
- Prioritized list of TODOs
- Critical issues addressed
- Obsolete TODOs removed
- Clean codebase with actionable TODOs only

---

## Context from This Session

We just completed:
- ✅ Z3 Dynamic type migration
- ✅ Stack overflow fix
- ✅ 50+ clippy warnings fixed
- ✅ Test expectations corrected

The codebase is in good shape - now's a good time to review TODOs systematically.

