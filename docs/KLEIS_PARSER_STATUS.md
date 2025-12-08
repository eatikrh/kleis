# Kleis Language Parser - Current Status

**Parser:** `src/kleis_parser.rs`  
**Last Updated:** December 8, 2024  
**Grammar Version:** v0.3 (POC - ~30% coverage)  
**Status:** Working for Phase 1, ready for Phase 2 extension

---

## Purpose

This document tracks the **Kleis language parser** (kleis_parser.rs), which parses:
- Structure definitions: `structure Matrix(m, n, T) { ... }`
- Implementations: `implements Matrix(m, n, ℝ) { ... }`
- Operations: `operation transpose : Matrix(n, m, T)`
- Axioms: `axiom commutativity: ...`

**Note:** This is DIFFERENT from the LaTeX parser (parser.rs) which parses math notation.
See [PARSER_TODO.md](../PARSER_TODO.md) for LaTeX parser status.

---

## Current Coverage (~30% of Grammar)

**Design Note:** The parser is intentionally minimal (POC). It covers ~30% of the formal grammar (docs/grammar/kleis_grammar_v03.ebnf) to demonstrate the type system. Full grammar implementation is Phase 2 work.

### ✅ **Working (v0.3)**

**Structure Definitions:**
- [x] `structure Name(params) { members }`
- [x] Type parameters: `(m: Nat, n: Nat, T)`
- [x] Operations: `operation name : signature`
- [x] Fields: `field_name : Type`
- [x] Axioms: `axiom name: expression` (basic)

**Implementations:**
- [x] `implements StructureName(types) { members }`
- [x] Concrete type arguments: `Matrix(2, 3, ℝ)`
- [x] Operation bindings: `operation name = builtin_name`

**Type Expressions:**
- [x] Named types: `ℝ`, `ℂ`, `Nat`, `Bool`
- [x] Parametric types: `Matrix(2, 3, ℝ)`
- [x] Function types: `T → T`
- [x] Product types: `T × T × T`
- [x] Type variables: `α`, `β`, `γ`

**Top-Level:**
- [x] Structure definitions
- [x] Implements blocks
- [x] Operation declarations
- [x] Function definitions (basic)
- [x] Type aliases
- [x] Comments: `/* ... */` and `// ...`

**Unicode Support:**
- [x] Greek letters: α, β, γ, etc.
- [x] Math symbols: ℝ, ℂ, ℕ, ℤ, →, ×, ∀, ∃
- [x] Subscripts/superscripts in identifiers

---

## ❌ **Not Yet Supported (Phase 2)**

### **High Priority (Phase 2 Week 1-2):**

**Operator Symbols:**
- [ ] Operator names as symbols: `operation (+) : T × T → T`
- [ ] Infix notation: `(•)`, `(⊗)`, `(∘)`
- Needed for: Full prelude.kleis loading

**Lowercase Constructors:**
- [ ] Lowercase operation names: `matrix`, `vector`, `list`
- [ ] Distinction from type constructors: `Matrix` vs `matrix`
- Needed for: Proper value constructors (ADR-020)

**Enhanced Axioms:**
- [ ] Quantifiers: `∀(x y : T)`, `∃(x : T)`
- [ ] Proper axiom bodies (currently placeholders)
- [ ] Conditions: `if`, `where` clauses
- Needed for: Full axiom support

**Function Definitions:**
- [ ] Full `define` syntax: `define f(x: T) : U = expr`
- [ ] Pattern matching in definitions
- [ ] Multi-clause definitions
- Needed for: Executable specifications

### **Medium Priority (Phase 2 Week 3-4):**

**Nested Structures:**
- [ ] Structures inside structures
- [ ] Local type definitions
- Needed for: Complex type hierarchies

**List Comprehensions:**
- [ ] `[x * 2 | x <- [1..10]]`
- Needed for: Matrix from lists

**Let Bindings:**
- [ ] `let x = ... in ...`
- Needed for: Complex expressions

### **Low Priority (Phase 3+):**

**Pattern Matching:**
- [ ] `match` expressions
- [ ] Case analysis
- Needed for: ADR-021 (data types)

**Proof Terms:**
- [ ] `theorem` declarations
- [ ] `proof` blocks
- [ ] QED markers
- Needed for: Formal proofs

---

## Phase 2 Roadmap (Parser Extension)

**Goal:** Extend from 30% → 70% grammar coverage

### **Week 1-2: Critical Features**
1. Operator symbol parsing: `(+)`, `(×)`, `(•)`
2. Lowercase value constructors: `matrix`, `vector`
3. Enhanced axiom quantifiers: `∀(x y : T)`
4. Function definition bodies

**Enables:**
- Full prelude.kleis loading
- Proper value constructors
- Rich axiom support

### **Week 3-4: Structure Support**
1. Nested structures
2. Complex type expressions
3. List comprehensions (maybe)

**Enables:**
- User-defined complex types
- Rich type hierarchies

---

## Phase 2.5: Add `data` Keyword (ADR-021)

**After Phase 2 parser is extended:**

**New syntax:**
```kleis
data Type =
  | Scalar
  | Vector(n: Nat)
  | Matrix(m: Nat, n: Nat)
  | Complex
```

**Implementation:** See [session-2024-12-08/ADR021_IMPLEMENTATION_PLAN.md](session-2024-12-08/ADR021_IMPLEMENTATION_PLAN.md)

**Timeline:** 1-2 weeks  
**Impact:** Meta-circular type system

---

## Grammar Conformance

**Formal Grammar:** `docs/grammar/kleis_grammar_v03.ebnf`

**Current parser implements:**
- Structure definitions ✓
- Type expressions ✓
- Basic axioms ✓
- Comments ✓

**Parser does NOT implement:**
- Full axiom syntax (quantifiers, conditions)
- Pattern matching
- Data types
- Module system
- Proof terms

**By design:** Parser is POC covering essentials for Phase 1 type system work.

---

## Testing

**Current:**
- Parser tested indirectly through type system tests
- Loading stdlib exercises parser on real Kleis code
- 288 total tests (many parse Kleis structures)

**Need:**
- Direct kleis_parser.rs unit tests (currently minimal)
- Parse error tests
- Grammar conformance tests

---

## Known Limitations (Phase 1 POC)

### **1. Axiom Bodies Not Parsed**
```kleis
axiom commutativity: ∀(x y). x + y = y + x
//                   ^^^^^^^^^^^^^^^^^^^^ Stored as string, not parsed
```
**Why:** Axiom parsing is complex, Phase 2 work  
**Impact:** Can't validate axiom structure yet

### **2. No Operator Symbols**
```kleis
operation (+) : T × T → T  // ❌ Not supported yet
```
**Why:** Operator precedence parsing is Phase 2  
**Impact:** Must use named operations: `operation plus : T × T → T`

### **3. Uppercase/Lowercase Not Distinguished**
```kleis
Matrix(2, 3, ...)  // Type constructor
matrix(a, b, c)    // Value constructor
// Parser treats both the same
```
**Why:** Distinction not needed for Phase 1  
**Impact:** Type/value conflation (will fix with Phase 2 + ADR-021)

---

## Next Steps

### **Immediate (Phase 2 Start):**
1. Add operator symbol parsing
2. Add lowercase constructor recognition
3. Enhanced axiom parsing with quantifiers

### **Phase 2.5 (ADR-021):**
1. Add `data` keyword parsing
2. Parse variant definitions
3. Support pattern matching syntax

### **Phase 3:**
1. Full grammar implementation
2. Module system
3. Proof terms

---

## Statistics

**Lines of code:** ~1,400 (kleis_parser.rs)  
**Grammar coverage:** ~30% (intentional POC)  
**Test coverage:** Indirect through type system  
**Passing:** All tests that use Kleis structures (288 total)

---

## References

- **Formal Grammar:** docs/grammar/kleis_grammar_v03.ebnf
- **Parser Implementation:** src/kleis_parser.rs
- **Phase 2 Plan:** docs/parser-implementation/
- **ADR-021 Plan:** docs/session-2024-12-08/ADR021_IMPLEMENTATION_PLAN.md

---

**Status:** ✅ Working for Phase 1  
**Next:** Phase 2 extension (30% → 70%)  
**Then:** ADR-021 (data types)  
**Goal:** Full self-hosting (100% grammar in Kleis)


