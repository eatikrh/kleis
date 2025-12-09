# Testing Documentation

Test data, test strategies, and testing infrastructure documentation.

---

## Test Data Sources

### DLMF_INTEGRATION.md (187 lines)
**NIST Digital Library of Mathematical Functions integration**

Contents:
- 36 curated equations from DLMF
- Test data organization
- Equation coverage analysis
- Domain coverage (Bessel, Gamma, Hypergeometric, etc.)
- Integration strategy
- Testing approach

**Purpose:** Comprehensive real-world test cases

**Source:** [NIST DLMF](https://dlmf.nist.gov/)

**Categories covered:**
- Bessel functions
- Gamma function
- Hypergeometric functions
- Legendre polynomials
- Orthogonal polynomials
- Elliptic integrals
- Zeta function
- Special cases

---

## Testing Strategy

The DLMF equations provide:
- **Real-world complexity** - Actual mathematical notation used in research
- **Domain coverage** - Multiple mathematical domains
- **Edge cases** - Complex nested structures
- **Verification** - Authoritative source for correctness

---

## Related Documentation

**Test Implementation:**
- See `tests/` directory in project root for actual test code
- See `examples/` for test examples
- See `docs/guides/TEST_GUIDE.md` for testing guidelines

**Other Test Data:**
- Golden test files in `tests/golden/`
- Custom test equations in `tests/golden/sources/custom/`

---

## Navigation

**Parent:** [docs/README.md](../README.md)  
**Related:**
- [Reference](../reference/) - Technical references
- [Guides](../guides/) - Implementation guides

---

**Last Updated:** December 9, 2024

