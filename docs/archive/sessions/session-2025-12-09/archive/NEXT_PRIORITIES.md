# Next Session Priorities - Based on Dec 9 Discoveries

**Date:** December 9, 2025  
**Context:** Following matrix multiplication and constants discovery  
**Status:** Prioritized work items for next session

---

## 🔥 Priority 0: `define` Parsing - THE SELF-HOSTING KEY (3-4 hours) ⭐⭐⭐

**Why this is THE priority:**
- 🔑 **Unlocks self-hosting** - Kleis can define Kleis!
- 🎯 **Last missing piece** - AST + patterns + types exist, just need `define`
- ✅ **Stdlib completion** - uncomment pattern matching functions
- 🚀 **User empowerment** - custom functions without Rust
- 🎓 **Milestone achievement** - meta-circular Kleis!

**Current state:**
```kleis
// In stdlib/types.kleis - COMMENTED OUT:
// define not(b) = match b { True => False | False => True }
```

**After `define` works:**
```kleis
define not(b) = match b { True => False | False => True }  // ✅
define map(f, list) = match list { Nil => Nil | Cons(h,t) => Cons(f(h), map(f,t)) }
```

### The Three Wires (Implementation Path)

**Following standard compiler design:**

**Wire 1: Parser → AST (1-2 hours)**
```rust
// Parse both forms into single AST variant:
// define x = expr                    → Define(x, [], None, expr)
// define f(x: T) : U = expr          → Define(f, [Param(x, T)], Some(U), expr)

fn parse_define(&mut self) -> Result<Decl::Define, ParseError>
fn parse_params(&mut self) -> Result<Vec<Param>, ParseError>
```

**Wire 2: Type Checker → Environment (1 hour)**
```rust
// Add function type to environment:
// For annotated: f : T → U (use annotation)
// For unannotated: f : τ (run HM inference)

fn check_define(&mut self, def: &Define) -> Result<Type, String>
```

**Wire 3: Evaluator → Closure (1 hour)**
```rust
// Store function as closure in runtime environment:
// f = Closure { params, body, env }

fn eval_define(&mut self, def: &Define) -> Result<Value, String>
```

### What This Unlocks

**Immediate:**
- ✅ Uncomment stdlib pattern matching functions
- ✅ Users can define custom operations
- ✅ Functions type-check before use

**Strategic:**
- ✅ Kleis grammar can be written IN Kleis!
- ✅ Transformations written in Kleis
- ✅ Self-hosting achieved (ADR-003 Phase 3)

**Example - Grammar as Kleis Value:**
```kleis
data Program = Program(decls: List(Decl))
data Decl = Define(...) | DataDef(...) | StructureDef(...)
data Expr = Var(String) | Apply(...) | Match(...)

// Pretty printer IN KLEIS:
define prettyPrint(e: Expr) : String = match e {
  Var(name) => name
  Apply(f, args) => prettyPrint(f) ++ "(" ++ ... ++ ")"
  Match(scrut, cases) => "match " ++ prettyPrint(scrut) ++ " {...}"
}
```

**The moment the Kleis grammar exists as a Kleis value: Self-hosting achieved!** 🎊

### Testing

**After implementation, test:**
```kleis
define double(x) = x + x
define map(f, list) = match list { Nil => Nil | Cons(h,t) => Cons(f(h), map(f,t)) }

// Use them:
double(5)  // → 10
map(double, [1, 2, 3])  // → [2, 4, 6]
```

**Result:** Self-hosted function definitions working! ✨

---

## 🔥 Priority 1: Physical Constants Palette (2-3 hours) ⭐ HIGHEST IMPACT

**Why this is #1:**
- ✨ **Today's profound discovery** - type system revealed this need
- 🎯 **High user impact** - enables proper GR/QM equations  
- ✅ **Immediate value** - users can start using constants right away
- 🚀 **Unlocks dimensional analysis** - connects types to physics
- 🎓 **Research validation** - proves type theory enforces physics

**Current problem:**
```
G_μν + Λg_μν = κT_μν
Type: Var(α) - polymorphic (constants undefined!)
```

**After fix:**
```
G_μν + Λg_μν = κT_μν  
Type: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ) ✓
```

### What to Implement

**1. AST Extension (30 min)**

Add to `src/ast.rs`:
```rust
pub enum Expression {
    // ... existing variants ...
    
    /// Physical constant with unit information
    Constant {
        name: String,        // "Lambda_cosmological"
        unit: String,        // "m^-2"
        dimension: Vec<i32>, // [L, M, T] exponents: [-2, 0, 0]
    },
}
```

**2. Palette Constants Tab (1 hour)**

Add to `static/index.html`:
```javascript
// New palette tab: Constants
const physicsConstants = {
    // GR Constants
    lambda: {
        symbol: 'Λ',
        name: 'Lambda_cosmological',
        tooltip: 'Cosmological constant ≈1.089×10⁻⁵² m⁻²',
        unit: 'm^-2',
        ast: { Constant: { name: 'Lambda_cosmological', unit: 'm^-2' } }
    },
    kappa: {
        symbol: 'κ',
        name: 'kappa_Einstein', 
        tooltip: 'Einstein constant 8πG/c⁴',
        unit: 'm^-1 kg^-1 s^2',
        ast: { Constant: { name: 'kappa_Einstein', unit: 'm^-1 kg^-1 s^2' } }
    },
    
    // Universal Constants
    G: {
        symbol: 'G',
        name: 'G_Newton',
        tooltip: 'Gravitational constant 6.674×10⁻¹¹ m³ kg⁻¹ s⁻²',
        unit: 'm^3 kg^-1 s^-2',
        ast: { Constant: { name: 'G_Newton', unit: 'm^3 kg^-1 s^-2' } }
    },
    c: {
        symbol: 'c',
        name: 'c_light',
        tooltip: 'Speed of light 299,792,458 m s⁻¹',
        unit: 'm s^-1',
        ast: { Constant: { name: 'c_light', unit: 'm s^-1' } }
    },
    hbar: {
        symbol: 'ℏ',
        name: 'hbar_Planck',
        tooltip: 'Reduced Planck constant 1.055×10⁻³⁴ J·s',
        unit: 'kg m^2 s^-1',
        ast: { Constant: { name: 'hbar_Planck', unit: 'kg m^2 s^-1' } }
    },
    
    // QM Constants
    e: {
        symbol: 'e',
        name: 'e_charge',
        tooltip: 'Elementary charge 1.602×10⁻¹⁹ C',
        unit: 'A s',
        ast: { Constant: { name: 'e_charge', unit: 'A s' } }
    },
    me: {
        symbol: 'mₑ',
        name: 'me_electron',
        tooltip: 'Electron mass 9.109×10⁻³¹ kg',
        unit: 'kg',
        ast: { Constant: { name: 'me_electron', unit: 'kg' } }
    }
};
```

**3. Type Definitions (30 min)**

Add to `stdlib/physics_constants.kleis`:
```kleis
structure PhysicalConstant(unit: String) {
    operation to_real : ℝ
    operation get_unit : String
}

// Declare each constant with its unit
// Type system will validate dimensional consistency
```

**4. Rendering Support (30 min)**

Add rendering for `Expression::Constant` in all formats.

### Testing

**Test that Einstein equation now returns:**
```
Type: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ) ✓
```

**Instead of:** `Var(α)`

**Result:** Type system validates dimensional consistency! 🎉

---

## 🔥 Priority 2: Math Functions - Easy Quick Win (1 hour) ⭐

**Why this is practical:**
- ✅ **Quick win** - 1 hour, immediate value
- 👥 **User-requested** - basic functions people need
- 🎯 **High usage** - arcsin, arctan, factorial commonly used
- ✨ **Parser-compatible** - no syntax barriers
- 📦 **Self-contained** - doesn't depend on other work

### What to Add

Create `stdlib/math_functions.kleis`:

```kleis
// Inverse trigonometric functions
structure InverseTrig(T) {
    operation arcsin : T → T
    operation arccos : T → T
    operation arctan : T → T
    operation arctan2 : T → T → T
}

implements InverseTrig(ℝ) {
    operation arcsin = builtin_arcsin
    operation arccos = builtin_arccos
    operation arctan = builtin_arctan
    operation arctan2 = builtin_arctan2
}

// Hyperbolic functions
structure Hyperbolic(T) {
    operation sinh : T → T
    operation cosh : T → T
    operation tanh : T → T
}

implements Hyperbolic(ℝ) {
    operation sinh = builtin_sinh
    operation cosh = builtin_cosh
    operation tanh = builtin_tanh
}

// Combinatorics
structure Combinatorics {
    operation factorial : ℕ → ℕ
    operation binomial : ℕ → ℕ → ℕ
    operation permutation : ℕ → ℕ → ℕ
}

implements Combinatorics {
    operation factorial = builtin_factorial
    operation binomial = builtin_binomial
    operation permutation = builtin_permutation
}

// Special functions
structure SpecialFunctions(T) {
    operation gamma : T → T
    operation beta : T → T → T
    operation erf : T → T
    operation bessel_j : ℕ → T → T
}

implements SpecialFunctions(ℝ) {
    operation gamma = builtin_gamma_function
    operation beta = builtin_beta_function
    operation erf = builtin_error_function
    operation bessel_j = builtin_bessel_j
}
```

**Load in type_checker.rs** and test.

**Result:** Palette math functions all type-checkable! ✅

---

## 🔥 Priority 3: Parser Extension for Complex `implements` (4-6 hours)

**Why this unblocks everything:**
- 🚧 **Currently blocking** polymorphic tensor arithmetic
- 🎯 **Architectural completeness** - finish what we designed today
- ✅ **Enables:** Einstein equations to fully type-check
- 🔓 **Unlocks:** All polymorphic operations for all types

**Current limitation:**
```kleis
// Parser can't handle this:
implements Arithmetic(Tensor(upper, lower, dim, ℝ)) {
    operation plus = builtin_tensor_add
}

// Error: Complex parametric type in implements block
```

**Need to extend parser to support:**
1. Cross-file structure references (Arithmetic from minimal_prelude)
2. Complex parametric types in implements
3. Nested type parameters: `Tensor(upper, lower, dim, ℝ)`

### Implementation Steps

**1. Parser extension (2-3 hours)**
- Extend `parse_implements` to handle parametric types
- Add type parameter parsing for complex types
- Handle structure references from other files

**2. Test with tensors (30 min)**
```kleis
implements Arithmetic(Tensor(upper, lower, dim, ℝ)) {
    operation plus = builtin_tensor_add
    operation minus = builtin_tensor_subtract
}
```

**3. Reload stdlib and test Einstein (30 min)**

**Result:** 
```
G_μν + Λg_μν = κT_μν
Type: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ) ✓
```

**Combined with Priority 1:** Fully validated GR equations! 🎊

---

## Priority 4: Integration Tests (2-3 hours)

**Why this builds confidence:**
- ✅ **Validation** - tests complete workflows
- 🐛 **Regression prevention** - catches breaking changes
- 📚 **Documentation** - tests show how to use features
- 🚀 **Production readiness** - proves system robustness

### Test Suites to Create

**1. `tests/matrix_operations_integration.rs`**
- Regular matrix multiplication
- Block matrix multiplication (2 levels deep)
- Matrix addition, transpose
- Dimension mismatch errors
- Type inference through operations

**2. `tests/tensor_operations_integration.rs`**
- Christoffel from metric
- Riemann from Christoffel
- Einstein tensor computation
- Full GR calculation workflow
- Rank validation

**3. `tests/quantum_operations_integration.rs`**
- Ket/bra creation
- Inner products
- Commutators
- Expectation values
- Operator composition

**4. `tests/pattern_matching_integration.rs`**
- Real-world pattern matching
- Type inference with patterns
- Exhaustiveness checking
- Error message quality

**Result:** Complete test coverage for major features!

---

## Priority 5: Full Parser for `define` (4-6 hours)

**Why this enables self-hosting:**
- 🎯 **Self-hosting milestone** - functions in Kleis!
- 📚 **Stdlib completion** - uncomment pattern matching functions
- ✅ **User empowerment** - users define custom operations
- 🚀 **Meta-circular** - Kleis defines Kleis

### What to Implement

**1. Parse `define` statements (2 hours)**
```rust
fn parse_function_def(&mut self) -> Result<FunctionDef, KleisParseError>
fn parse_params(&mut self) -> Result<Vec<Param>, KleisParseError>
```

**2. Uncomment stdlib functions (1 hour)**

From `stdlib/types.kleis`:
```kleis
define not(b) = match b { True => False | False => True }
define and(b1, b2) = match b1 { False => False | True => b2 }
define map(f, list) = match list { Nil => Nil | Cons(h, t) => Cons(f(h), map(f, t)) }
```

**3. Test self-hosted functions (1 hour)**

**Result:** Kleis type system defined in Kleis! 🎊

---

## Recommended Sequence

### Session 1: Quick Wins (3-4 hours)
1. **Math functions** (1 hour) - Easy, high value
2. **Physical constants palette** (2-3 hours) - Today's discovery

**Result:** Basic functionality complete, Einstein equations validate!

### Session 2: Architectural Completion (4-6 hours)
3. **Parser extension for implements** (4-6 hours) - Unblocks everything

**Result:** Polymorphic arithmetic complete, tensor operations work!

### Session 3: Validation (2-3 hours)  
4. **Integration tests** (2-3 hours) - Prove it all works

**Result:** Production-ready with full test coverage!

### Session 4: Self-Hosting (4-6 hours)
5. **Full parser for define** (4-6 hours) - Meta-circular milestone

**Result:** Kleis defines Kleis! Self-hosting achieved! 🚀

---

## Dependencies

**No blockers:**
- Priority 1, 2, 4, 5 can be done independently

**Synergy:**
- Priority 1 + 3 together → Fully validated Einstein equations
- Priority 2 + 3 → Complete polymorphic operations  
- Priority 4 validates 1, 2, 3
- Priority 5 builds on everything

---

## Impact Assessment

| Priority | Time | Impact | Difficulty | User Value |
|----------|------|--------|------------|------------|
| 1. Constants | 2-3h | 🔥🔥🔥🔥🔥 | Medium | Very High |
| 2. Math Fns | 1h | 🔥🔥🔥 | Easy | High |
| 3. Parser | 4-6h | 🔥🔥🔥🔥 | Hard | High |
| 4. Tests | 2-3h | 🔥🔥🔥 | Medium | Medium |
| 5. Define | 4-6h | 🔥🔥🔥🔥 | Hard | Very High |

---

## My Strong Recommendation

**Start with Priority 1 (Physical Constants Palette)**

**Why:**
1. **Validates today's discovery** - you found something profound!
2. **Immediate payoff** - Einstein equations properly typed
3. **Opens new research** - dimensional analysis as type checking
4. **High visibility** - users will see the value immediately
5. **Foundation for future** - enables unit-aware computation

**This completes the story arc from today:**
- Morning: "Why is equation polymorphic?"
- Investigation: "Constants are undefined!"
- Discovery: "Constants need units!"
- Next session: **"Implement unit-aware constants!"** ✨

**Perfect narrative continuity.** 📖

---

## Alternative: Quick Win First

**If you want immediate gratification:**

**Start with Priority 2 (Math Functions) - 1 hour**
- Super easy
- Immediate user value
- Builds momentum
- **Then** do Priority 1

**Both approaches work!**

---

## Long-Term Vision (Months)

After these 5 priorities:

**Phase 1: Core Features (Done!)**
- ✅ Pattern matching
- ✅ Matrix operations  
- ✅ Type inference

**Phase 2: Physics Support (In Progress)**
- ✅ Tensor operations
- ✅ Quantum notation
- ⏳ Physical constants (Priority 1)
- ⏳ Dimensional analysis

**Phase 3: Self-Hosting (Priorities 3, 5)**
- ⏳ Full parser
- ⏳ Function definitions
- ⏳ Kleis-defined type checker

**Phase 4: User Ecosystem**
- Documentation
- Examples library
- Community contributions
- Published papers 📄

---

## Success Metrics

**After Priority 1 (Constants):**
- ✅ Einstein equations return Tensor (not Var)
- ✅ Dimensional consistency validated
- ✅ 8+ physical constants available in palette

**After Priority 2 (Math Functions):**
- ✅ All basic math functions type-check
- ✅ Palette coverage ~95%

**After Priority 3 (Parser):**
- ✅ Polymorphic arithmetic for all types
- ✅ Einstein tensor form fully validates
- ✅ Parser feature-complete for stdlib

**After Priority 4 (Tests):**
- ✅ 500+ tests passing
- ✅ Integration test coverage
- ✅ Production confidence

**After Priority 5 (Define):**
- ✅ Self-hosting achieved
- ✅ Functions defined in Kleis
- ✅ Meta-circular milestone!

---

## The North Star

**Ultimate goal:** Kleis as the standard tool for type-safe scientific computing.

**These 5 priorities move us toward:**
- Type-checked physics equations ✓
- Dimensional analysis validation ✓
- User-extensible operations ✓
- Self-hosting type system ✓

**Each priority is a major milestone.**

---

## Quick Start for Next Session

### Recommended Path

**Session 1 (3-4 hours):**
```
1. Add math functions (1h) - quick win!
2. Implement constants palette (2-3h) - today's discovery!

Result: Basic toolkit complete, Einstein validates!
```

**Session 2 (4-6 hours):**
```
3. Extend parser for implements (4-6h) - unblock architecture

Result: Polymorphic operations complete!
```

**Session 3+ (as time allows):**
```
4. Integration tests (2-3h)
5. Full parser for define (4-6h)
```

**This sequence maximizes impact and maintains momentum!** 🚀

---

**See also:**
- `NEXT_SESSION_TASK.md` (general roadmap)
- `PHYSICAL_CONSTANTS_PALETTE.md` (Priority 1 details)
- `UNIVERSAL_CONSTANTS_FINDING.md` (why Priority 1 matters)

**Let's complete the revolution!** 🎊

