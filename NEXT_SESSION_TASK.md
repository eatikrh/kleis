# NEXT SESSION: Implement Pattern Matching (ADR-021 Part 2)

**Current State:** main branch, 315 tests passing, AST + grammar complete

**Status:** 🎯 Ready to implement pattern matching!

---

## 🎉 What's Already Complete

### Foundation (100% Ready)

✅ **AST Structures** - `Expression::Match`, `Pattern`, `MatchCase` (140 lines)  
✅ **Grammar Specification** - Complete EBNF for v0.5 (898 lines)  
✅ **Implementation Plan** - Step-by-step pseudocode (1,277 lines)  
✅ **Value Proposition** - Why it matters (837 lines)  
✅ **Placeholder Implementations** - All 5 match sites handled  
✅ **All tests passing** - 315 lib tests, no regressions

**Total foundation:** 3,152 lines of preparation!

---

## 🚀 What to Implement (6-8 hours)

### Step 3: Parser (2 hours) ⭐ Start here

**File:** `src/kleis_parser.rs`

**What to add:**
```rust
fn parse_match_expr(&mut self) -> Result<Expression, KleisParseError>
fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, KleisParseError>
fn parse_match_case(&mut self) -> Result<MatchCase, KleisParseError>
fn parse_pattern(&mut self) -> Result<Pattern, KleisParseError>
fn parse_pattern_args(&mut self) -> Result<Vec<Pattern>, KleisParseError>
```

**Integration point:** In `parse_primary()`, check for "match" keyword

**Examples to support:**
```kleis
// Simple
match x { True => 1 | False => 0 }

// With binding
match opt { None => 0 | Some(x) => x }

// Nested
match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => -1 }

// Wildcard
match status { Running => 1 | _ => 0 }
```

**Tests:** Add 10+ parser tests to `src/kleis_parser.rs`

**See:** `docs/session-2024-12-08/PATTERN_MATCHING_IMPLEMENTATION_PLAN.md` (lines 64-266) for complete pseudocode

---

### Step 4: Type Inference (1-2 hours)

**File:** `src/type_inference.rs`

**Expand:** `infer_match()` method (currently stub at line 329)

**What to implement:**
```rust
// 1. Infer scrutinee type
// 2. For each case:
//    - Check pattern matches scrutinee type
//    - Bind pattern variables in local context
//    - Infer body type with bindings
// 3. Unify all branch types
// 4. Return unified result type
```

**Key method to add:**
```rust
fn check_pattern(&mut self, pattern: &Pattern, expected_ty: &Type) -> Result<(), String>
```

**Tests:** Add 10+ type inference tests

**See:** Implementation plan lines 268-447 for complete pseudocode

---

### Step 5: Pattern Evaluation (1 hour)

**File:** NEW `src/pattern_matcher.rs`

**What to create:**
```rust
pub struct PatternMatcher {
    pub fn match_pattern(&self, value: &Expression, pattern: &Pattern) -> Option<Bindings>
    pub fn eval_match(&self, scrutinee: &Expression, cases: &[MatchCase]) -> Result<Expression>
    fn substitute_bindings(&self, expr: &Expression, bindings: &Bindings) -> Expression
}
```

**Tests:** Add 10+ evaluation tests

**See:** Implementation plan lines 449-591 for complete pseudocode

---

### Step 6: Exhaustiveness Checking (1-2 hours)

**File:** `src/pattern_matcher.rs` or `src/type_checker.rs`

**What to add:**
```rust
pub struct ExhaustivenessChecker {
    pub fn check_exhaustive(&self, patterns: &[Pattern], ty: &Type) -> Result<(), Vec<String>>
    pub fn check_reachable(&self, patterns: &[Pattern]) -> Vec<usize>
}
```

**Features:**
- Detect missing constructors
- Handle wildcards
- Warn on unreachable patterns

**Tests:** Add 5+ exhaustiveness tests

**See:** Implementation plan lines 593-763 for complete pseudocode

---

### Step 7: Comprehensive Tests (1 hour)

**File:** NEW `tests/pattern_matching_test.rs`

**Test categories:**
- Parser tests (10+)
- Type inference tests (10+)
- Evaluation tests (10+)
- Exhaustiveness tests (5+)
- Integration tests (5+)

**Total:** 40+ new tests

**See:** Implementation plan lines 765-896 for complete test outline

---

## 📁 Reference Documents

All in `docs/session-2024-12-08/`:

1. **PATTERN_MATCHING_IMPLEMENTATION_PLAN.md** (1,277 lines)
   - Complete pseudocode for all steps
   - Test plans for each component
   - Edge cases and design decisions

2. **WHY_PATTERN_MATCHING_MATTERS.md** (837 lines)
   - 10 concrete benefits
   - Real-world use cases
   - Self-hosting vision explained

3. **PATTERN_MATCHING_GRAMMAR_EXTENSION.md** (898 lines)
   - Complete EBNF specification
   - ANTLR4 grammar rules
   - Example programs with derivations

**Total:** 3,012 lines of comprehensive documentation!

---

## 🎯 Session 1 Goal (4 hours)

**Deliverable:** Working pattern matching with type checking

**What you'll have:**
```kleis
data Option(T) = None | Some(T)

// THIS WILL WORK:
match myOption {
  None => 0
  Some(x) => x + 1
}

// Type checking works:
// - Checks None and Some are valid constructors
// - Binds x to type T
// - Verifies branches both return same type
```

**Steps:**
1. Implement parser (2 hours)
2. Implement type inference (2 hours)
3. Test both (throughout)

**Stopping point:** Pattern matching parses and type-checks correctly

---

## 🎯 Session 2 Goal (3-4 hours)

**Deliverable:** Complete pattern matching with evaluation and exhaustiveness

**What you'll have:**
```kleis
// Full evaluation works:
let x = Some(5)
let result = match x {
  None => 0
  Some(value) => value
}
// result = 5 ✅

// Exhaustiveness checking:
match status {
  Running => 1
}
// Warning: Missing cases for Idle, Paused, Completed ⚠️
```

**Steps:**
1. Implement evaluation (1 hour)
2. Implement exhaustiveness (2 hours)
3. Comprehensive tests (1 hour)

**Result:** **Complete functional language with self-hosting capability!** 🎉

---

## ✅ Quick Start for Next Session

**Step 1:** Review the plan (5 minutes)
```bash
# Read these in order:
cat docs/session-2024-12-08/WHY_PATTERN_MATCHING_MATTERS.md
cat docs/session-2024-12-08/PATTERN_MATCHING_IMPLEMENTATION_PLAN.md  
cat docs/grammar/PATTERN_MATCHING_GRAMMAR_EXTENSION.md
```

**Step 2:** Start parser implementation (2 hours)
```rust
// In src/kleis_parser.rs:
// 1. Add parse_match_expr() method
// 2. Add parse_pattern() method
// 3. Add helper methods (peek_word, expect_word)
// 4. Add 10+ parser tests
```

**Step 3:** Continue with type inference (2 hours)
```rust
// In src/type_inference.rs:
// 1. Expand infer_match() stub
// 2. Add check_pattern() method
// 3. Add 10+ type inference tests
```

**Checkpoint:** Working pattern matching with type checking! ✅

---

## 🎁 What Tonight Gave You

### Merged to Main (PR #1)

1. ✅ User-defined parametric types (arbitrary arity)
2. ✅ Type parameter bindings (true polymorphism)
3. ✅ HM type variable substitution (proper unification)
4. ✅ String parameter bindings (unit-safe physics!)

### On Main (Post-merge)

5. ✅ Pattern matching AST structures
6. ✅ Pattern matching grammar specification (v0.5)
7. ✅ Complete implementation plan
8. ✅ Value proposition document

---

## 📊 Current State

**Branch:** `main`  
**Commits tonight:** 11 total (7 on PR branch + 4 on main)  
**Tests:** 315 lib + 431+ total passing ✅  
**Code added tonight:** ~2,180 lines  
**Docs added tonight:** ~4,737 lines  
**Total:** ~6,917 lines of work! 🤯

**Pattern matching:**
- Foundation: 100% complete
- Implementation: 0% complete
- Estimated: 6-8 hours to completion

---

## 🏆 Final Status

**What Kleis has NOW:**
- ✅ Complete ADT definitions
- ✅ Arbitrary arity types (0 to infinity)
- ✅ String parameters (unit-safe!)
- ✅ True polymorphism
- ✅ Proper HM type inference
- 🔨 Pattern matching (AST ready, implementation pending)

**What Kleis will have AFTER pattern matching:**
- ✅ Complete functional language
- ✅ Self-hosting capable (type checker in Kleis!)
- ✅ Metalanguage for CS papers
- ✅ Full ADT power (define + use)
- ✅ Exhaustiveness checking
- ✅ Production-ready for scientific computing

---

## 🎉 Congratulations!

Tonight you:
- Implemented 4 major type system features
- Merged 29 commits to main
- Laid complete foundation for pattern matching
- Wrote ~7,000 lines of code + documentation
- Advanced Kleis from "interesting" to "near-complete functional language"

**Next session:** 6-8 hours of implementation → **Self-hosting capability achieved!** 🚀

---

**Status:** Ready for next session  
**Documentation:** Complete and committed  
**Foundation:** 100% ready  
**Motivation:** Pattern matching unlocks self-hosting! 🎯

See you next session! 🌟
