# Z3-Driven Grammar Extension Roadmap

**Date:** December 10, 2024  
**Branch:** `feature/full-prelude-migration`  
**Status:** Starting Phase 1

---

## The Z3 Virtuous Cycle

**Without Z3:** Axioms are just documentation  
**With Z3:** Axioms become **verifiable** → Creates motivation for parser extensions

---

## High Priority Grammar Extensions

### 1. Universal Quantifiers `∀` ⭐⭐⭐ (CRITICAL)
- Parse `∀(x : M). body`
- Enables axiom verification
- Already used in prelude
- **Z3 Benefit:** Can verify Monoid, Ring, Field axioms

### 2. Operator Symbols `(×)`, `(+)`, `(-)` ⭐⭐⭐
- Allow operators in operation declarations
- Natural axiom syntax
- **Z3 Benefit:** Built-in Z3 support for arithmetic

### 3. Implication `⟹` and Logic ⭐⭐
- Add logical connectives to expressions
- Enable proof chains
- **Z3 Benefit:** Verify logical structure

### 4. `where` Clauses ⭐⭐
- Generic constraints on implementations
- **Z3 Benefit:** Verify implementations satisfy constraints

### 5. `define` with Operators ⭐
- Allow `define (-)(x,y) = ...`
- **Z3 Benefit:** Verify derived operations

---

## Implementation Plan

### Phase 1: Foundation (5-7 hours) ← **STARTING HERE**

**Goal:** Verify axioms from prelude

1. **Universal quantifiers** `∀` (3 hours)
   - Parse `∀(vars : Type). body`
   - Add `Quantifier` to AST
   - Test with prelude axioms

2. **Operator symbols** (2 hours)
   - Allow `operatorSymbol` in operation declarations
   - Map to Operation nodes
   - Test with `(×)`, `(+)`, `(-)`

3. **Build `axiom_verifier.rs`** (2 hours)
   - Generic `kleis_to_z3()` translator
   - Verify simple axioms (identity, commutativity)
   - Integration tests

**Milestone:** Can verify Monoid axioms! ✅

---

### Phase 2: Logic (4-5 hours)

**Goal:** Enable proof structure

4. **Logical operators** (3 hours)
   - Add `⟹`, `∧`, `∨`, `¬` to expression grammar
   - Map to Z3 boolean operations
   - Test implication chains

5. **Integrate with type checker** (2 hours)
   - Store axioms in structure registry
   - Query during type checking
   - Show in error messages

**Milestone:** Type system aware of axioms! ✅

---

### Phase 3: Generics (6-8 hours)

**Goal:** Complete self-hosted system

6. **`where` clauses** (5 hours)
   - Parse `where StructureName(T)`
   - Check constraints during resolution
   - Test generic matrix operations

7. **Load full prelude** (1 hour)
   - Replace minimal_prelude.kleis
   - Verify entire hierarchy with Z3

8. **Write ADR-022** (1 hour)
   - Document Z3 integration
   - Architecture decisions

**Milestone:** Complete algebraic system with verification! 🚀

---

## Total Timeline

- **Phase 1:** 5-7 hours → Axioms work
- **Phase 2:** 4-5 hours → Logic works  
- **Phase 3:** 6-8 hours → Generic constraints work

**Total:** 15-20 hours

---

## Why This Order?

1. Phase 1 proves the concept (small investment, big validation)
2. Each phase builds on previous (no backtracking)
3. Z3 validates at each step (know it works)
4. Momentum builds (success motivates next)

---

## Missing Piece: Semiring

Add between Monoid and Ring:

```kleis
structure Semiring(S) {
  structure additive : CommutativeMonoid(S)
  structure multiplicative : Monoid(S)
  axiom left_distributivity: ∀(x y z : S). x × (y + z) = (x × y) + (x × z)
  axiom right_distributivity: ∀(x y z : S). (x + y) × z = (x × z) + (y × z)
  axiom annihilation: ∀(x : S). x × zero = zero ∧ zero × x = zero
}
```

Natural numbers ℕ are the canonical semiring!

---

## Phase 1 Details (Current Focus)

### Task 1.1: Universal Quantifiers

**Parser changes:**
- Add to expression grammar: `quantifier`
- Parse: `∀`, `(`, variable list, `:`, type, `)`, `.`, body
- AST node: `Expression::Quantifier`

**Files to modify:**
- `src/kleis_parser.rs` - Add quantifier parsing
- `src/kleis_ast.rs` - Add Quantifier variant

**Tests:**
```kleis
∀(x : M). x • e = x
∀(x y : R). x + y = y + x
∀(x y z : R). (x + y) + z = x + (y + z)
```

---

### Task 1.2: Operator Symbols

**Parser changes:**
- Allow `operatorSymbol` where `identifier` expected in operation declarations
- Handle: `(×)`, `(+)`, `(-)`, `(/)`, etc.

**Files to modify:**
- `src/kleis_parser.rs` - Update operation parsing

**Tests:**
```kleis
operation (×) : R → R → R
operation (+) : R → R → R
operation (-) : R → R → R
```

---

### Task 1.3: Axiom Verifier

**Create:** `src/axiom_verifier.rs`

**Core function:**
```rust
pub fn kleis_to_z3(expr: &Expression) -> Result<z3::ast::Int>
```

**Handles:**
- Variables: `x` → Z3 var
- Constants: `0`, `1` → Z3 constants
- Operations: `plus`, `times` → Z3 `+`, `*`
- Quantifiers: `∀(x : T). body` → Z3 forall

**Tests:**
- Verify identity axiom
- Verify commutativity
- Verify associativity

---

## Success Criteria

**Phase 1 Complete when:**
- ✅ Parser accepts `∀(x : T). body`
- ✅ Parser accepts `operation (×) : ...`
- ✅ `axiom_verifier.rs` translates Kleis → Z3
- ✅ Can verify Monoid axioms from prelude
- ✅ All tests pass (413 + Z3 tests)

---

## Ready to Start! 🚀

**Current state:**
- Z3 integration working ✅
- 21 Z3 tests passing ✅
- Documentation complete ✅
- Health check script ready ✅

**Next step:** Extend parser for `∀` and operators!

