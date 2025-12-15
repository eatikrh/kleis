# Kleis Vision: Formal Verification for Engineering

*Last updated: Dec 15, 2025*

---

## The Big Picture

**Kleis aims to be a formal verification tool for safety-critical engineering systems.**

The target: Make mathematical proofs of correctness as accessible as unit tests are today.

---

## Why This Matters

### The Problem

| Domain | Current State | Risk |
|--------|---------------|------|
| Autonomous vehicles | "Trust us, it's safe" | Unknown algorithms, no proof |
| Aerospace software | DO-178C testing ($10k/line) | Expensive, not complete |
| Medical devices | FDA 510(k) submissions | Testing-based, not proof-based |
| Industrial control | PLC logic, ad-hoc verification | Critical infrastructure at risk |

### The Insight

> "I didn't ask LLM to solve my control problem — I made LLM write a program 
> to solve ALL LQG/LTR control problems."

This is the difference between:
- **Compute**: Solve one problem once
- **Specify**: Define axioms that verify all instances of a problem class

---

## The Target: DO-178C / DO-333 Formal Methods

**DO-178C** is the FAA/EASA standard for safety-critical avionics software.

**DO-333** is the formal methods supplement that allows mathematical proofs 
to replace some testing requirements.

### Design Assurance Levels

| Level | Failure Effect | Example |
|-------|----------------|---------|
| A | Catastrophic | Flight control |
| B | Hazardous | Engine control |
| C | Major | Navigation |
| D | Minor | Cabin systems |
| E | No effect | Entertainment |

### What Formal Methods Enable

```
Traditional DO-178C:
  Code → Tests → 100% coverage → Expensive

With DO-333:
  Specification → Proof that code satisfies spec → Reduced testing
```

---

## How Kleis Fits

### The Vision

```
┌─────────────────────────────────────────────────────┐
│  Engineer defines REQUIREMENTS in Kleis            │
│                                                     │
│  structure Controller(n: Nat) {                     │
│      element A : Matrix(n, n, ℝ)                   │
│      element K : Matrix(m, n, ℝ)                   │
│      axiom stable: is_stable(A - B·K)              │
│  }                                                  │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│  Kleis + Z3 VERIFIES the proof                     │
│                                                     │
│  ✓ Type-checked: dimensions match                  │
│  ✓ Axiom verified: stability proven                │
│  ✓ Certificate generated                           │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│  Certification body ACCEPTS the evidence           │
│                                                     │
│  • FAA / EASA (aerospace)                          │
│  • NHTSA (automotive)                              │
│  • FDA (medical)                                   │
└─────────────────────────────────────────────────────┘
```

### Current Capabilities (Dec 2025)

| Capability | Status | Notes |
|------------|--------|-------|
| Formal grammar | ✅ v0.7 | EBNF specification |
| Type system | ✅ | Hindley-Milner with matrices |
| Axiom verification | ✅ | Z3 backend |
| Structure definitions | ✅ | Parameterized types |
| Matrix operations | ✅ | Type-checked dimensions |
| Control theory examples | ✅ | LQG, eigenvalues, stability |
| Beautiful rendering | ✅ | Typst/LaTeX/Unicode |

### Gaps to Fill

| Gap | Priority | Description |
|-----|----------|-------------|
| Import/include | HIGH | Module system for large specs |
| Numerical backend | HIGH | nalgebra for eigenvalues, Riccati |
| Proof certificates | HIGH | Export proofs for auditors |
| Refinement types | MEDIUM | `x : ℝ where 0 ≤ x ≤ 1` |
| Temporal logic | FUTURE | "Always stable", LTL/CTL |
| Code generation | FUTURE | Kleis → verified C/Rust |

---

## Progress Roadmap

```
Kleis Progress Toward Formal Methods Tool:

Foundation:        ████████████████████ 100%
Type System:       ████████████████░░░░  80%
Axiom Proofs:      ████████████░░░░░░░░  60%
Control Theory:    ██████░░░░░░░░░░░░░░  30%
Numerical:         ████░░░░░░░░░░░░░░░░  20%
Certification:     ██░░░░░░░░░░░░░░░░░░  10%
───────────────────────────────────────────
Overall:           ~40% toward MVP
```

### Milestones

| Milestone | Timeline | Description |
|-----------|----------|-------------|
| Kleis 1.0 | 6-12 months | Full parser, modules, numerical |
| Academic adoption | 1-2 years | Teaching formal methods |
| Industry pilot | 2-3 years | Partner with drone/UAV company |
| DO-333 qualification | 3-5 years | FAA/EASA acceptance |

---

## Target Markets

### Near-term (1-2 years)

- **Education**: Teach formal verification with beautiful notation
- **Research**: Publish verified proofs in papers
- **Startups**: Drone/UAV companies needing lightweight certification

### Medium-term (2-5 years)

- **Automotive**: ADAS and autonomous vehicle controllers
- **Aerospace**: Smaller aircraft, eVTOL (Joby, Lilium)
- **Industrial**: PLC replacement for safety systems

### Long-term (5+ years)

- **Major OEMs**: Boeing, Airbus, Tesla formal verification
- **Regulatory bodies**: FAA/EASA tooling certification
- **Medical**: Implantable device control systems

---

## Why Kleis Can Succeed

| Advantage | Description |
|-----------|-------------|
| **Usability first** | Beautiful notation, not just correctness |
| **Incremental adoption** | Use for documentation now, proofs later |
| **Modern stack** | Rust, Z3, Typst — not 1990s tools |
| **Type-safe math** | Catch dimension errors at compile time |
| **Open foundation** | Build on existing proof infrastructure |

### The Key Insight

Most formal methods tools fail because they're:
- Academically pure but unusable
- Require PhD to operate
- Ugly output nobody wants to read

Kleis inverts this:
- **Beautiful first** (you'll want to use it)
- **Correct always** (proofs are non-negotiable)
- **Incremental path** (start with rendering, add verification)

---

## The Scary Part

> "Kleis is becoming scary!"

The scary part isn't the power — it's the **responsibility**.

When Kleis can say:
> "This flight controller is proven stable for all disturbances within bounds"

...that's a statement with real-world consequences.

This is why we build carefully, with:
- Rigorous foundations (Hindley-Milner, Z3)
- Clear separation of concerns (AST, renderer, verifier)
- Comprehensive testing
- Honest documentation of limitations

---

## Honest Limitations: Neural Networks

**Kleis cannot formally verify neural network controllers.**

| Controller Type | Parameters | Formal Verification |
|-----------------|------------|---------------------|
| PID | 3 | ✅ Easy |
| LQR/LQG | O(n²) | ✅ Tractable |
| MPC | O(n²·horizon) | ⚠️ Possible with effort |
| Neural Network | 10⁶ - 10⁹ | ❌ Curse of dimensionality |

### Why Neural Networks Are Different

```
Classical control:
  u = -K·x
  Verify: eigenvalues(A - BK) all have Re(λ) < 0
  Complexity: O(n³) ✅

Neural network:
  u = NN(x) = σ(Wₙ·σ(Wₙ₋₁·...σ(W₁·x)))
  Verify: ∀x ∈ ℝⁿ, system stable with u = NN(x)
  
  Problem: ReLU/activation functions create piecewise-linear regions
           n neurons → up to 2ⁿ linear regions to analyze
           Verification must consider all reachable regions
  
  Complexity: Combinatorial explosion (exponential in network size) ❌
```

Note: Some NN verification problems are formally NP-complete (Katz et al. 2017),
but the practical barrier is the combinatorial explosion from activation 
bifurcations, not just worst-case complexity class.

### What Kleis CAN Do for NN Systems

1. **Verify the safety envelope**
   ```kleis
   axiom envelope_safe:
       ∀ x : State .
       x ∈ safe_region → safe_controller(x) stabilizes
   ```

2. **Verify the fallback controller**
   ```kleis
   axiom fallback_stable:
       is_stable(A - B · K_fallback)
   ```

3. **Verify the switching logic**
   ```kleis
   axiom override_condition:
       NN_output ∉ safe_bounds → use fallback
   ```

### The Hybrid Architecture (Industry Practice)

```
┌─────────────────────────────────────────────────┐
│  VERIFIED SAFETY ENVELOPE (Kleis)               │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │  Neural Network (Black Box)               │ │
│  │  - Perception, planning, prediction       │ │
│  │  - NOT formally verifiable                │ │
│  │  - Tested statistically (millions of mi)  │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Monitor: if output ∉ verified_bounds          │
│           → engage verified_fallback           │
│                                                 │
│  Kleis verifies: bounds + fallback + monitor   │
└─────────────────────────────────────────────────┘
```

This is exactly what **Mobileye's RSS** and **NVIDIA's SFF** do — 
mathematical safety constraints around learning-based components.

### Honest Claim

> "Kleis can verify **classical controllers** and **safety envelopes** 
> around neural networks — not the neural networks themselves."

---

*"We don't know what kind of control algorithms self-driving cars apply. 
With Kleis, we can analyze the stability of classical components and 
verify safety envelopes around neural components."*

— Kleis Vision Statement, Dec 2025

