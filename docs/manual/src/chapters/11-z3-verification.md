# Z3 Verification

## What is Z3?

[Z3](https://github.com/Z3Prover/z3) is a theorem prover from Microsoft Research. Kleis uses Z3 to:

- **Verify** mathematical statements
- **Find counterexamples** when statements are false
- **Check** that implementations satisfy axioms

## Basic Verification

Use `verify` to check a statement:

```kleis example
axiom commutativity : ∀(x : ℝ)(y : ℝ). x + y = y + x
// Z3 verifies: ✓ Valid

axiom zero_annihilates : ∀(x : ℝ). x * 0 = 0
// Z3 verifies: ✓ Valid

axiom all_positive : ∀(x : ℝ). x > 0
// Z3 finds counterexample: x = -1
```

## Verifying Quantified Statements

Z3 handles universal and existential quantifiers:

```kleis example
axiom additive_identity : ∀(x : ℝ). x + 0 = x
// Z3 verifies: ✓ Valid

axiom squares_nonnegative : ∀(x : ℝ). x * x ≥ 0
// Z3 verifies: ✓ Valid (squares are non-negative)

axiom no_real_sqrt_neg1 : ∃(x : ℝ). x * x = -1
// Z3: ✗ Invalid (no real square root of -1)

axiom complex_sqrt_neg1 : ∃(x : ℂ). x * x = -1
// Z3 verifies: ✓ Valid (x = i works)
```

## Checking Axioms

Verify that definitions satisfy axioms:

```kleis
structure Group(G) {
    e : G
    operation mul : G × G → G
    operation inv : G → G
    
    axiom identity : ∀(x : G). mul(e, x) = x
    axiom inverse : ∀(x : G). mul(x, inv(x)) = e
    axiom associative : ∀(x : G)(y : G)(z : G).
        mul(mul(x, y), z) = mul(x, mul(y, z))
}

// Define integers with addition
implements Group(ℤ) {
    element e = 0
    operation mul = builtin_add
    operation inv = builtin_negate
}

// Kleis verifies each axiom automatically!
```

## Implication Verification

Prove that premises imply conclusions:

```kleis example
// If x > 0 and y > 0, then x + y > 0
axiom sum_positive : ∀(x : ℝ)(y : ℝ). (x > 0 ∧ y > 0) → x + y > 0
// Z3 verifies: ✓ Valid

// Triangle inequality
axiom triangle_ineq : ∀(x : ℝ)(y : ℝ)(a : ℝ)(b : ℝ).
    (abs(x) ≤ a ∧ abs(y) ≤ b) → abs(x + y) ≤ a + b
// Z3 verifies: ✓ Valid
```

## Counterexamples

When verification fails, Z3 provides counterexamples:

```kleis example
axiom square_equals_self : ∀(x : ℝ). x^2 = x
// Z3: ✗ Invalid, Counterexample: x = 2 (since 4 ≠ 2)

axiom positive_greater_than_one : ∀(n : ℕ). n > 0 → n > 1
// Z3: ✗ Invalid, Counterexample: n = 1
```

## Timeout and Limits

Complex statements may time out:

```kleis
// Very complex statement
verify ∀ M : Matrix(100, 100) . det(M * M') ≥ 0
// Result: ⏱ Timeout (statement too complex)
```

## Verifying Nested Quantifiers (Grammar v0.9)

Grammar v0.9 enables nested quantifiers in logical expressions:

```kleis
structure Analysis {
    // Quantifier inside conjunction - Z3 handles this
    axiom bounded: (x > 0) ∧ (∀(y : ℝ). y = y)
    
    // Epsilon-delta limit definition
    axiom limit_def: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
}
```

### Function Types in Verification

Quantify over functions and verify their properties:

```kleis
structure Continuity {
    // Z3 treats f as an uninterpreted function ℝ → ℝ
    axiom continuous_at: ∀(f : ℝ → ℝ, a : ℝ, ε : ℝ). ε > 0 →
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - f(a)) < ε))
}
```

**Note:** Z3 treats function-typed variables as uninterpreted functions, allowing reasoning about their properties without knowing their implementation.

## What Z3 Can and Cannot Do

### Z3 Excels At:
- Linear arithmetic
- Boolean logic
- Array reasoning
- Simple quantifiers
- Algebraic identities
- Nested quantifiers (Grammar v0.9)
- Function-typed variables

### Z3 Struggles With:
- Non-linear real arithmetic (undecidable in general)
- Very deep quantifier nesting (may timeout)
- Transcendental functions (sin, cos, exp)
- Infinite structures
- Inductive proofs over recursive data types

## Practical Workflow

1. **Write structure with axioms**
2. **Implement operations**
3. **Kleis auto-verifies** axioms are satisfied
4. **Use `verify`** for additional properties
5. **Examine counterexamples** when verification fails

```kleis
// Step 1: Define structure
structure Ring(R) {
    zero : R
    one : R
    operation add : R × R → R
    operation mul : R × R → R
    operation neg : R → R
    
    axiom add_assoc : ∀(a : R)(b : R)(c : R).
        add(add(a, b), c) = add(a, add(b, c))
}

// Step 2: Implement for integers
implements Ring(ℤ) {
    element zero = 0
    element one = 1
    operation add = builtin_add
    operation mul = builtin_mul
    operation neg = builtin_negate
}

// Step 3: Auto-verification happens!

// Step 4: Check additional properties
axiom mul_zero : ∀(x : ℤ). mul(x, zero) = zero
// Z3 verifies: ✓ Valid
```

## Solver Abstraction Layer

While this chapter focuses on Z3, Kleis is designed with a **solver abstraction layer** that can interface with multiple proof backends.

### Architecture

```
User Code (Kleis Expression)
         │
    SolverBackend Trait
         │
   ┌─────┴──────┬───────────┬──────────────┐
   │            │           │              │
Z3Backend  CVC5Backend  IsabelleBackend  CustomBackend
   │            │           │              │
   └─────┬──────┴───────────┴──────────────┘
         │
  OperationTranslators
         │
   ResultConverter
         │
User Code (Kleis Expression)
```

### The SolverBackend Trait

The core abstraction is defined in `src/solvers/backend.rs`:

```rust
pub trait SolverBackend {
    /// Get solver name (e.g., "Z3", "CVC5")
    fn name(&self) -> &str;

    /// Get solver capabilities (declared upfront, MCP-style)
    fn capabilities(&self) -> &SolverCapabilities;

    /// Verify an axiom (validity check)
    fn verify_axiom(&mut self, axiom: &Expression) 
        -> Result<VerificationResult, String>;

    /// Check if an expression is satisfiable
    fn check_satisfiability(&mut self, expr: &Expression) 
        -> Result<SatisfiabilityResult, String>;

    /// Evaluate an expression to a concrete value
    fn evaluate(&mut self, expr: &Expression) 
        -> Result<Expression, String>;

    /// Simplify an expression
    fn simplify(&mut self, expr: &Expression) 
        -> Result<Expression, String>;

    /// Check if two expressions are equivalent
    fn are_equivalent(&mut self, e1: &Expression, e2: &Expression) 
        -> Result<bool, String>;

    // ... additional methods for scope management, assertions, etc.
}
```

**Key design principle:** All public methods work with Kleis `Expression`, not solver-specific types. Solver internals never escape the abstraction.

### MCP-Style Capability Declaration

Solvers declare their capabilities upfront (inspired by Model Context Protocol):

```rust
pub struct SolverCapabilities {
    pub solver: SolverMetadata,      // name, version, type
    pub capabilities: Capabilities,   // operations, theories, features
}

pub struct Capabilities {
    pub theories: HashSet<String>,              // "arithmetic", "boolean", etc.
    pub operations: HashMap<String, OperationSpec>,
    pub features: FeatureFlags,                 // quantifiers, evaluation, etc.
    pub performance: PerformanceHints,          // timeout, max axioms
}
```

This enables:
- **Coverage analysis** - Know what operations are natively supported
- **Multi-solver comparison** - Choose the best solver for a program
- **User extensibility** - Add translators for missing operations

### Verification Results

```rust
pub enum VerificationResult {
    Valid,                              // Axiom holds for all inputs
    Invalid { counterexample: String }, // Found a violation
    Unknown,                            // Timeout or too complex
}

pub enum SatisfiabilityResult {
    Satisfiable { example: String },    // Found satisfying assignment
    Unsatisfiable,                      // No solution exists
    Unknown,
}
```

### Why Multiple Backends?

Different proof systems have different strengths:

| Backend | Strength | Best For |
|---------|----------|----------|
| **Z3** | Fast SMT solving, decidable theories | Arithmetic, quick checks, counterexamples |
| **CVC5** | Finite model finding, strings | Bounded verification, string operations |
| **Isabelle** | Structured proofs, automation | Complex inductive proofs, formalization |
| **Coq/Lean** | Dependent types, program extraction | Certified programs, mathematical libraries |

### Current Implementation

Currently implemented in `src/solvers/`:

| Component | Status | Description |
|-----------|--------|-------------|
| `SolverBackend` trait | ✅ Complete | Core abstraction |
| `SolverCapabilities` | ✅ Complete | MCP-style capability declaration |
| `Z3Backend` | ✅ Complete | Full Z3 integration |
| `ResultConverter` | ✅ Complete | Convert solver results to Kleis expressions |
| `discovery` module | ✅ Complete | List available solvers |
| CVC5Backend | 🔮 Future | Alternative SMT solver |
| IsabelleBackend | 🔮 Future | HOL theorem prover |

### Solver Discovery

```rust
use kleis::solvers::discovery;

// List all available backends
let solvers = discovery::list_solvers();  // ["Z3"]

// Check if a specific solver is available
if discovery::is_available("Z3") {
    let backend = Z3Backend::new()?;
}
```

### Benefits of Abstraction

1. **Solver independence** - Swap solvers without code changes
2. **Unified API** - Same methods regardless of backend
3. **Capability-aware** - Know what each solver supports before using it
4. **Extensible** - Add custom backends by implementing the trait
5. **Future-proof** - New provers can be integrated without changing Kleis code

This architecture makes Kleis a **proof orchestration layer** with beautiful notation, not just another proof assistant.

## What's Next?

Try the interactive REPL!

→ [Next: The REPL](./12-repl.md)
