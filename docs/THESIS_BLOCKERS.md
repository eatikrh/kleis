# Thesis Writing Blockers

**Last Updated:** 2026-01-03

What's preventing PhD students from writing their thesis in Kleis today?

---

## Critical Blockers

### 1. Edit Existing Content

**What:** `doc.update_equation()`, `doc.update_section()`, `doc.remove_section()`

**Why:** PhD thesis writing is iterative. Students revise equations after advisor feedback, 
rewrite sections multiple times, and reorganize chapters. Without edit/delete, they must:
- Manually edit the `.kleis` file (breaks abstraction)
- Delete and re-add content (loses history, error-prone)

**Challenge this:** "Can't they just edit the .kleis file directly?"
- Yes, but that defeats the purpose of the Python API
- The .kleis format is for persistence, not human editing
- AST structures for equations are complex to edit by hand

---

### 2. Cross-References in Text

**What:** Replace `@eq:label`, `@fig:label`, `@sec:label` in prose with Typst refs

**Why:** Academic writing requires inline references like:
> "As shown in Equation 3, the loss function..."
> "See Figure 2.1 for the architecture..."

Currently `doc.cite()` and `doc.ref_equation()` return strings, but they're not 
automatically replaced in section content.

**Challenge this:** "Can students write Typst refs manually?"
- Yes: `"As shown in @eq:loss, the loss function..."`
- But this requires knowing Typst syntax
- Mixing LaTeX equations with Typst refs is confusing

---

## Important but Not Blocking

### 3. Visual Equation Editing in Jupyter

**What:** Open Equation Editor, edit equation, save back to document

**Why:** The Equation Editor provides a visual way to build complex equations.
Currently students must write LaTeX strings.

**Challenge this:** "LaTeX is the standard. Why do they need visual editing?"
- Many students know LaTeX well enough
- Visual editor is a convenience, not a requirement
- They can always use external tools and paste LaTeX

**Verdict:** Nice-to-have, not a blocker.

---

### 4. Version Control / Backups

**What:** `doc.save_version("v1-before-advisor-meeting")`

**Why:** Thesis work spans months. Students want checkpoints.

**Challenge this:** "Can't they use git?"
- Yes! Git works perfectly with `.kleis` files (they're text)
- `git commit -m "before advisor meeting"` is the standard workflow
- No need to reinvent version control

**Verdict:** NOT a blocker. Use git.

---

### 5. Complex LaTeX Edge Cases

**What:** Handle every possible LaTeX command

**Why:** Some equations may not convert correctly.

**Challenge this:** "How common are edge cases?"
- The converter handles ~90% of common academic math
- Students can write Typst directly for edge cases
- Or use the AST-based Equation Editor (renders perfectly)

**Verdict:** Partial blocker. Document known limitations.

---

## Summary: True Blockers

| Blocker | Effort | Impact |
|---------|--------|--------|
| Edit existing content | Medium | High - can't iterate on work |
| Cross-refs in text | Low | Medium - workaround exists |

Everything else has workarounds.

---

## Recommended Next Steps

1. **Implement `update_*` methods** - Enables iterative workflow
2. **Auto-replace `@ref:label` in section content** - Cleaner authoring
3. **Document LaTeX limitations** - Set expectations

## Not Recommended

- Version control in KleisDoc (use git)
- Full LaTeX parser (diminishing returns)
- Jupyter magic commands (user rejected complexity)

