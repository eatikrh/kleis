# Kleis Scripts

Utility scripts for the Kleis project.

## `fetch_dlmf_v2.py` ⭐ **RECOMMENDED**

Generates curated DLMF equations from a hand-picked collection. **This is the recommended approach** as it provides immediate, high-quality test cases without network dependencies.

**Quick start:**
```bash
python3 scripts/fetch_dlmf_v2.py
```

This generates 36 equations across 8 topics:
- Gamma functions (5 equations)
- Bessel functions (5 equations)  
- Hypergeometric functions (4 equations)
- Legendre polynomials (4 equations)
- Zeta function (5 equations)
- Elliptic integrals (4 equations)
- Orthogonal polynomials (4 equations)
- Special cases (5 equations)

---

## `fetch_dlmf.py` (Experimental)

**Status:** Work in progress. DLMF uses complex MathML rendering that's difficult to parse automatically. Use `fetch_dlmf_v2.py` instead.

Attempts to download mathematical equations from the NIST Digital Library of Mathematical Functions (DLMF) by scraping the website.

### Requirements

```bash
# Python 3.7+ (uses only standard library)
python3 --version
```

### Usage

**Basic usage (default chapters: 1, 5, 13, 15, 25):**
```bash
python3 scripts/fetch_dlmf.py
```

**Custom chapters:**
```bash
python3 scripts/fetch_dlmf.py --chapters 1,2,3,10,25
```

**Custom output directory:**
```bash
python3 scripts/fetch_dlmf.py --output tests/golden/sources/handbook/
```

**Limit equations per chapter:**
```bash
python3 scripts/fetch_dlmf.py --max-per-chapter 30
```

**All options:**
```bash
python3 scripts/fetch_dlmf.py \
  --chapters 1,5,13,15,25 \
  --output tests/golden/sources/dlmf/ \
  --max-per-chapter 50 \
  --delay 2.0
```

### Recommended Chapters

| Chapter | Topic | Why Include |
|---------|-------|-------------|
| 1 | Algebraic Functions | Basic operations, polynomials |
| 5 | Gamma Function | Special function syntax, complex notation |
| 8 | Incomplete Gamma Functions | Nested integrals |
| 10 | Bessel Functions | Subscripts, limits |
| 13 | Confluent Hypergeometric | Complex multi-level expressions |
| 15 | Hypergeometric Functions | Generalized notation |
| 18 | Orthogonal Polynomials | Series, summations |
| 25 | Zeta & Related Functions | Greek letters, limits |
| 33 | Coulomb Functions | Physics notation |
| 36 | Integrals with Coalescing Saddles | Advanced integrals |

### Output

Creates LaTeX files in the output directory:
```
tests/golden/sources/dlmf/
  chapter01.tex  # Algebraic Functions
  chapter05.tex  # Gamma Function
  chapter13.tex  # Confluent Hypergeometric
  ...
```

Each file contains:
- Source attribution
- Equation IDs for reference
- Clean LaTeX ready for testing

### Integration with Golden Tests

After fetching equations, run your test suite:

```bash
# Run golden tests with new DLMF examples
cargo test golden_tests

# Or specifically test DLMF equations
cargo test -- --test-threads=1 dlmf
```

### Notes

- **Rate limiting**: Default 2-second delay between requests (adjustable)
- **Polite scraping**: Minimal load on NIST servers
- **No dependencies**: Uses only Python standard library
- **Fallback**: If HTML parsing fails, try alternative chapters

### Troubleshooting

**No equations extracted:**
- DLMF HTML structure may have changed
- Try a different chapter
- Check internet connection

**Malformed LaTeX:**
- Some DLMF equations use specialized macros
- May need manual cleanup for complex cases
- Most equations (90%+) work directly

### Future Enhancements

- [ ] Extract equation descriptions/context
- [ ] Support for specific sections (not just chapters)
- [ ] LaTeX validation before saving
- [ ] Chapter metadata (titles, descriptions)
- [ ] Parallel fetching for speed

