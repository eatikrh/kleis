# Remaining Broken Links to Fix

**Date:** December 11, 2024  
**Status:** 23 broken links remaining (from 284 original)  
**Priority:** Must reach 0 broken links per quality gate

## Progress So Far

✅ Fixed 261 links (including 240+ false positives from math notation)  
✅ Improved link checker to skip code blocks  
✅ Fixed critical files: README.md, PARSER_TODO.md, ADRs  
⏳ Remaining: 23 real broken links in archived/session folders

## Remaining Broken Links (7 real broken links)

**Note:** Link checker has improved but still flags some mathematical notation. Real broken links below.

### Real Broken Links to Fix:

1. **docs/archive/template-implementation-strategy.md** - Line 5
   - ADR links in header need `../adr/` prefix

2. **docs/archive/sessions/session-2024-12-06/INDEX.md**
   - File references deleted documents (can remove these references)

**False positives (mathematical notation - ignore):**
- `[exp(-t²)](ω)` in TYPST_RENDERING_FIXED.md - This is math, not a link
- Similar patterns in other files

## Action Plan

1. **Archive folder links** - Update to point to correct locations
2. **Session 2024-12-08** - Remove links to deleted TASK files
3. **Parser status** - Fix PARSER_TODO path
4. **Improve link checker** - Better math notation detection

## Quality Gate

**Added to .cursorrules:**
```
Before committing .md files:
→ Run: python3 scripts/check_markdown_links.py
→ Must show 0 broken links
```

## Next Session

Start with: `python3 scripts/check_markdown_links.py` and systematically fix remaining 23 links.

