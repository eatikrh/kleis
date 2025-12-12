# Remaining Broken Links to Fix

**Date:** December 11, 2024  
**Status:** 23 broken links remaining (from 284 original)  
**Priority:** Must reach 0 broken links per quality gate

## Progress So Far

✅ Fixed 261 links (including 240+ false positives from math notation)  
✅ Improved link checker to skip code blocks  
✅ Fixed critical files: README.md, PARSER_TODO.md, ADRs  
⏳ Remaining: 23 real broken links in archived/session folders

## Remaining Broken Links (23 total)

### docs/archive/session-2024-12-05/TYPST_RENDERING_FIXED.md (1)
- [exp(-t²)](ω) - False positive (math notation), improve checker

### docs/archive/sessions/README.md (2)
- [Session 2024-12-07](2024-12-07/README.md) - Session archived elsewhere
- [Session 2024-12-06](2024-12-06/README.md) - Session archived elsewhere

### docs/archive/sessions/session-2024-12-06/INDEX.md (2)
- [ADR-015-VALIDATION-REPORT.md](ADR-015-VALIDATION-REPORT.md) - Update path
- [IMPLEMENTATION_NEXT_STEPS.md](IMPLEMENTATION_NEXT_STEPS.md) - File deleted

### docs/archive/template-implementation-strategy.md (6)
- ADR links need ../adr/ prefix (4 occurrences)

### docs/parser-implementation/KLEIS_PARSER_STATUS.md (2)
- [PARSER_TODO.md](../PARSER_TODO.md) - Path incorrect (should be ../../)
- [session-2024-12-08/ADR021_IMPLEMENTATION_PLAN.md] - File doesn't exist

### docs/session-2024-12-08/README.md (10)
- Multiple TASK files don't exist (deleted or moved)
- Options: Remove links or create stub files

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

