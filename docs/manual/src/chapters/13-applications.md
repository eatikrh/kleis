# Applications

Kleis is designed for mathematical verification, but its power extends far beyond pure mathematics. This chapter showcases applications across multiple domains.

## AI/ML Verification

Kleis aligns with the research agenda outlined in **"Toward Verified Artificial Intelligence"** (Seshia et al., Communications of the ACM, 2022) — a seminal paper from UC Berkeley on using formal methods to verify AI systems.

### The Verified AI Challenge

The paper identifies five key challenges for AI verification:

| Challenge | Description | Kleis Approach |
|-----------|-------------|----------------|
| **Environment Modeling** | Formally specify the world | `structure` with `axiom`s |
| **Specification** | Define "correct" behavior | First-order logic assertions |
| **Computational Engines** | SAT/SMT solving | Z3 backend |
| **Correct-by-Construction** | Build verified from start | Axiom-first development |
| **Compositional Reasoning** | Modular proofs | `implements` contracts |

### Neural Network Properties

Kleis can express and verify properties of neural networks:

```kleis
// Robustness: Small input changes → small output changes
structure RobustnessProperty {
    axiom local_robustness: ∀ x : Input . ∀ δ : Input .
        norm(δ) < ε → 
        abs(network(x + δ) - network(x)) < bound
}

// Monotonicity: Larger input → larger output (for certain features)
structure MonotonicityProperty {
    axiom monotonic_feature: ∀ x1 : Input . ∀ x2 : Input .
        x1.feature ≤ x2.feature → 
        network(x1) ≤ network(x2)
}

// Safety envelope: Output always in safe region
structure SafetyProperty {
    axiom output_bounded: ∀ x : Input .
        valid_input(x) → 
        min_safe ≤ network(x) ∧ network(x) ≤ max_safe
}
```

### Safe Reinforcement Learning

```kleis
// Safety envelope for learning-based controllers
structure SafeController(state_dim: Nat, action_dim: Nat) {
    // Pre-computed safety region
    element safe_set : Set(State)
    
    // Learned policy
    operation policy : State → Action
    
    // Safety invariant: policy never leaves safe set
    axiom safety_invariant: ∀ s : State .
        s ∈ safe_set → 
        next_state(s, policy(s)) ∈ safe_set
    
    // Backup controller for edge cases
    operation backup : State → Action
    
    axiom backup_safe: ∀ s : State .
        next_state(s, backup(s)) ∈ safe_set
}
```

### Compositional Verification

The paper emphasizes **assume-guarantee reasoning** — exactly what Kleis structures provide:

```kleis
// Perception module contract
structure PerceptionContract {
    operation detect : Image → List(BoundingBox)
    
    // Guarantee: no false negatives within sensor range
    axiom no_miss_close: ∀ img : Image . ∀ obj : Object .
        distance(obj) < sensor_range ∧ visible(obj, img) →
        ∃ box : BoundingBox . box ∈ detect(img) ∧ contains(box, obj)
}

// Planning module contract
structure PlanningContract {
    // Assume: perception is correct within range
    // Guarantee: plan avoids all detected obstacles
    axiom collision_free: ∀ boxes : List(BoundingBox) . ∀ plan : Trajectory .
        plan = plan_path(boxes) →
        ∀ t : Time . ∀ box : BoundingBox . 
            box ∈ boxes → ¬intersects(plan(t), box)
}

// System-level property emerges from composition
structure SystemSafety {
    axiom no_collision: ∀ img : Image .
        let boxes = PerceptionContract.detect(img) in
        let plan = PlanningContract.plan_path(boxes) in
        collision_free(plan)
}
```

### Why Kleis for AI Verification?

| Approach | Limitation | Kleis Advantage |
|----------|------------|-----------------|
| Testing | Samples finite cases | Proves ∀ inputs |
| Fuzzing | Random, no guarantees | Exhaustive (for decidable) |
| Static analysis | Over-approximates | Precise via Z3 |
| Runtime monitoring | Reactive only | Proactive verification |

### Further Reading

- **Paper**: [Toward Verified Artificial Intelligence](https://cacm.acm.org/research/toward-verified-artificial-intelligence/) (CACM 2022)
- **VerifAI**: Toolkit for formal AI verification (same authors)
- **Scenic**: Domain-specific language for scenario specification

Kleis provides a **general-purpose substrate** for expressing the same verification concepts in a unified, mathematically rigorous framework.

---

## Hardware Verification

Kleis can formally verify hardware designs using Z3's bitvector theory. This section explains how Kleis compares to the industry-standard **Universal Verification Methodology (UVM)** defined in IEEE 1800.2-2020.

### Kleis vs UVM: Different Approaches to the Same Goal

| Aspect | UVM (IEEE 1800.2) | Kleis |
|--------|-------------------|-------|
| **Verification method** | Simulation (random sampling) | Formal proof (Z3) |
| **Coverage** | Statistical: "did we test this?" | Exhaustive: "is this state reachable?" |
| **Assertions** | SVA (temporal, simulation-checked) | First-order logic (mathematically proven) |
| **Language** | SystemVerilog class library | Native Kleis |
| **Cost** | Requires commercial simulators | Open source |

### What Kleis Proves

Kleis can **formally prove** properties for ALL possible inputs:

```kleis
// Bitvector operations (maps to SystemVerilog bit[7:0])
operation bvadd : BitVec8 × BitVec8 → BitVec8
operation bvxor : BitVec8 × BitVec8 → BitVec8
operation bv_zero : BitVec8

// PROVE: Addition is commutative for ALL 2^16 input pairs
example "ADD is commutative" {
    assert(∀ a : BitVec8 . ∀ b : BitVec8 . bvadd(a, b) = bvadd(b, a))
}

// PROVE: XOR of same value always yields zero
example "XOR self is zero" {
    assert(∀ a : BitVec8 . bvxor(a, a) = bv_zero)
}

// PROVE: Subtraction inverts addition
example "SUB inverts ADD" {
    assert(∀ a : BitVec8 . ∀ b : BitVec8 . bvsub(bvadd(a, b), b) = a)
}
```

A UVM testbench would run random tests and **hope** to find bugs. Kleis **proves** correctness mathematically.

### Reachability Analysis (Related to Coverage)

UVM coverage answers: "Did our tests exercise this state?"

Kleis reachability answers: "Is this state possible at all?"

```kleis
// Can ADD ever produce zero? (Yes: 0+0, or 128+128 with overflow)
example "Zero is reachable via ADD" {
    assert(∃ a : BitVec8 . ∃ b : BitVec8 . bvadd(a, b) = bv_zero)
}

// Can ADD produce non-zero? (Yes: 1+0, etc.)
example "Non-zero is reachable via ADD" {
    assert(∃ a : BitVec8 . ∃ b : BitVec8 . bvadd(a, b) ≠ bv_zero)
}
```

### Kleis Complements UVM, Not Replaces It

Kleis and UVM solve **related but different problems**:

```
┌─────────────────────────────────────────────────────────────┐
│                    VERIFICATION WORKFLOW                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   KLEIS (Pre-RTL)              UVM (With RTL)               │
│   ┌────────────────┐           ┌────────────────┐           │
│   │ Define ALU     │           │ Verilog ALU    │           │
│   │ specification  │    →      │ implementation │           │
│   └────────────────┘           └────────────────┘           │
│          ↓                            ↓                     │
│   ┌────────────────┐           ┌────────────────┐           │
│   │ PROVE correct  │           │ SIMULATE with  │           │
│   │ for ALL inputs │           │ random tests   │           │
│   └────────────────┘           └────────────────┘           │
│          ↓                            ↓                     │
│   Mathematical                 "Probably correct"           │
│   certainty                    (statistical)                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Use Kleis FIRST:** Prove your algorithm is correct before writing HDL.

**Use UVM AFTER:** Validate your Verilog/VHDL implements the algorithm correctly.

### What UVM Has That Kleis Doesn't

| UVM Feature | Kleis Status | Notes |
|-------------|--------------|-------|
| DUT connection | Not applicable | Kleis doesn't connect to HDL |
| Timing/clocks | Different paradigm | Model time explicitly if needed |
| Sequences over time | Not needed | Kleis proves for all states |
| Transaction-level modeling | Can model | Use `data` types |
| Waveform output | Not applicable | No simulation |

### Example: ALU Verification

See `examples/hardware/alu_verification.kleis` for a complete example that proves:

- ADD/AND/OR/XOR commutativity
- Addition associativity  
- Subtraction inverts addition
- AND idempotence
- XOR self produces zero
- Additive inverse
- Reachability of zero and non-zero results

All properties are proven for **all 65,536 possible 8-bit input pairs** — not sampled, but exhaustively verified.

### When to Use Kleis for Hardware

| Use Case | Kleis? | Why |
|----------|--------|-----|
| Algorithm correctness before RTL | Yes | Prove before you code |
| Property verification | Yes | Mathematical proof |
| Bug hunting in existing RTL | No | Use UVM/formal tools |
| Coverage closure | Partial | Reachability, not statistical |
| Integration with HDL flow | No | Standalone verification |

## Business Process Modeling

Model and verify business workflows with formal guarantees:

```kleis
// Order-to-Cash (O2C) Business Process
// Models the complete lifecycle from order to payment

// Order lifecycle states
data OrderStatus = 
    Draft | Pending | CreditApproved | CreditDenied 
  | Allocated | Fulfilled | Shipped | Invoiced 
  | Paid | Complete | Cancelled

// Credit check decision based on utilization
define credit_check_decision(utilization) =
    if utilization <= 100 then 1      // Approved
    else if utilization < 125 then 2  // PendingReview
    else 0                            // Denied

// Can order be cancelled from current state?
define can_cancel(status) = match status {
    Draft => 1
  | Pending => 1
  | CreditApproved => 1
  | Allocated => 1
  | _ => 0  // Can't cancel after fulfillment
}

// INVARIANT: No shipment without credit approval
define shipment_requires_credit(order_status, credit_approved) =
    if order_status = 6 then credit_approved = 1 else true

// INVARIANT: Order completion requires full payment
define completion_requires_payment(order_status, payment_status) =
    if order_status = 9 then payment_status >= 2 else true
```

## Network Protocol Verification

Verify protocol correctness with formal methods:

```kleis
// Stop-and-Wait Protocol - Reliable Data Transfer

// Sequence numbers alternate between 0 and 1
define next_seq(seq) = if seq = 0 then 1 else 0

// ACK is valid if it matches sent sequence
define valid_ack(sent, ack) = if ack = sent then 1 else 0

// Sender advances state only on valid ACK
define sender_next_state(current_seq, ack_received) = 
    if valid_ack(current_seq, ack_received) = 1 
    then next_seq(current_seq) 
    else current_seq

// VERIFIED: Double alternation returns to original
// next_seq(next_seq(0)) = 0  ✓
// next_seq(next_seq(1)) = 1  ✓

// SAFETY: No duplicate delivery when synchronized
// LIVENESS: Progress guaranteed when channel delivers
```

### IPv4 Packet Validation

```kleis
// IPv4 Header Validation (RFC 791)

// Version must be 4 for IPv4
define valid_version(v) = if v = 4 then 1 else 0

// IHL (Internet Header Length): 5-15 words
define valid_ihl(ihl) = ihl >= 5 and ihl <= 15

// Header length in bytes
define header_length(ihl) = ihl * 4

// Common protocols: 1=ICMP, 6=TCP, 17=UDP
define is_tcp(proto) = proto = 6
define is_udp(proto) = proto = 17

// Private address ranges
define is_private_class_a(o1) = o1 = 10
define is_private_class_c(o1, o2) = o1 = 192 and o2 = 168

// Full packet validation
define valid_packet(version, ihl, total, ttl, proto) = 
    valid_version(version) = 1 and
    valid_ihl(ihl) = 1 and
    ttl > 0 and
    total >= header_length(ihl)
```

## Authorization & Access Control

Model Zanzibar-style relationship-based access control (like Google Drive):

```kleis
// Permission Levels: 0=None, 1=Viewer, 2=Commenter, 3=Editor, 4=Owner

define has_at_least(user_perm, required_perm) = user_perm >= required_perm

define can_read(perm) = has_at_least(perm, 1)
define can_edit(perm) = has_at_least(perm, 3)
define can_delete(perm) = has_at_least(perm, 4)

// Folder inheritance (like Google Drive)
define inherited_permission(child_perm, parent_perm) = 
    if child_perm > 0 
    then child_perm      // Explicit permission overrides
    else parent_perm     // Inherit from parent

// Multi-group permission: take highest
define effective_permission(direct, group) = 
    if direct >= group then direct else group

// Security invariant: can_edit implies can_read
// ∀ p . can_edit(p) = 1 → can_read(p) = 1
```

## Security Analysis

Use Z3 string theory for static security analysis:

```kleis
// SQL Injection Detection using String Operations

// Vulnerable pattern: string concatenation + SQL execution
// :verify and(
//   contains("SELECT * FROM users WHERE id=" + userId, "+ userId"),
//   contains(code, "executeQuery")
// )
// If Valid → VULNERABLE!

// Safe pattern: parameterized queries
// :verify and(
//   contains(code, "PreparedStatement"),
//   not(contains(code, "+ userId +"))
// )
// If Valid → SAFE

// XSS Detection: innerHTML with user input
// :verify and(
//   contains(code, "innerHTML"),
//   contains(code, "userData")
// )
```

## Control Systems Engineering

Design optimal controllers with verified stability:

```kleis
// LQG Controller: Linear Quadratic Gaussian

structure LinearSystem(n: Nat, m: Nat, p: Nat) {
    element A : Matrix(n, n, ℝ)   // State dynamics
    element B : Matrix(n, m, ℝ)   // Input matrix
    element C : Matrix(p, n, ℝ)   // Output matrix
    element W : Matrix(n, n, ℝ)   // Process noise covariance
    element V : Matrix(p, p, ℝ)   // Measurement noise covariance
}

// LQR: Optimal state feedback
operation lqr_gain : LQRProblem(n, m) → Matrix(m, n, ℝ)

axiom lqr_stability:
    ∀ prob : LQRProblem(n, m) .
    let K = lqr_gain(prob) in
    let A_cl = prob.A - prob.B · K in
    is_stable(A_cl)

// Kalman Filter: Optimal state estimation
operation kalman_gain : KalmanProblem(n, p) → Matrix(n, p, ℝ)

// LQG combines LQR + Kalman via Separation Principle
structure LQGController(n: Nat, m: Nat, p: Nat) {
    element K : Matrix(m, n, ℝ)   // LQR gain
    element L : Matrix(n, p, ℝ)   // Kalman gain
}
```

## Dimensional Analysis (Physical Units)

Prevent unit mismatch bugs at compile time - like the Mars Climate Orbiter disaster ($327M lost due to imperial/metric confusion):

```kleis
// Physical dimensions as exponent tuples [Length, Mass, Time]
structure Dimension(L : ℤ, M : ℤ, T : ℤ) {
    axiom equal : ∀(d1 d2 : Dimension). 
        d1 = d2 ↔ (L(d1) = L(d2) ∧ M(d1) = M(d2) ∧ T(d1) = T(d2))
}

// Named dimensions
define Length = Dimension(1, 0, 0)
define Mass = Dimension(0, 1, 0)
define Time = Dimension(0, 0, 1)
define Velocity = Dimension(1, 0, -1)      // L·T⁻¹
define Force = Dimension(1, 1, -2)         // M·L·T⁻²
define Energy = Dimension(2, 1, -2)        // M·L²·T⁻²

// Physical quantity = value + dimension
structure Quantity(value : ℝ, dim : Dimension) {
    // Addition: dimensions must match
    axiom add_same_dim : ∀(q1 q2 : Quantity)(d : Dimension).
        dim(q1) = d ∧ dim(q2) = d → dim(q1 + q2) = d
    
    // Multiplication: dimensions compose
    axiom mul_composes : ∀(q1 q2 : Quantity).
        dim(q1 * q2) = Dimension(
            L(dim(q1)) + L(dim(q2)), 
            M(dim(q1)) + M(dim(q2)), 
            T(dim(q1)) + T(dim(q2)))
}

// Unit constructors
define meter(x : ℝ) = Quantity(x, Length)
define kilogram(x : ℝ) = Quantity(x, Mass)
define second(x : ℝ) = Quantity(x, Time)
define newton(x : ℝ) = Quantity(x, Force)

// Physics axioms verify dimensional consistency
structure Mechanics {
    // F = ma: [M·L·T⁻²] = [M] × [L·T⁻²] ✓
    axiom newton_second_law : ∀(m : Quantity)(a : Quantity).
        dim(m) = Mass ∧ dim(a) = Dimension(1, 0, -2) →
        dim(m * a) = Force
    
    // E = ½mv²: [M·L²·T⁻²] = [M] × [L·T⁻¹]² ✓
    axiom kinetic_energy : ∀(m : Quantity)(v : Quantity).
        dim(m) = Mass ∧ dim(v) = Velocity →
        dim(m * v * v) = Energy
}
```

**Type-safe physics:**
- `meter(100) + meter(50)` → `Quantity(150, Length)` ✓
- `meter(100) / second(10)` → `Quantity(10, Velocity)` ✓
- `meter(100) + second(10)` → ❌ Type error: `Length ≠ Time`

See `examples/physics/dimensional_analysis.kleis` for the full example.

## Differential Geometry

Kleis excels at differential geometry calculations:

```kleis
// Christoffel symbols for spherical coordinates
structure SphericalMetric {
    operation metric : (ℝ, ℝ) → Matrix(2, 2, ℝ)
    operation christoffel : (ℝ, ℝ) → Tensor(1, 2)
}

implements SphericalMetric {
    // Metric tensor: ds² = r²(dθ² + sin²θ dφ²)
    operation metric(r, θ) = Matrix [
        [r^2, 0],
        [0, r^2 * sin(θ)^2]
    ]
    
    // Christoffel symbols Γⁱⱼₖ
    operation christoffel(r, θ) = 
        let g = metric(r, θ) in
        let g_inv = inverse(g) in
        // ... compute from metric derivatives
}
```

## Tensor Calculus

```kleis
// Einstein field equations
structure EinsteinEquations {
    // Ricci tensor
    operation ricci : Manifold → Tensor(0, 2)
    // Scalar curvature
    operation scalar : Manifold → ℝ
    // Einstein tensor
    operation einstein : Manifold → Tensor(0, 2)
    
    axiom einstein_tensor : ∀(M : Manifold).
        einstein(M) = ricci(M) - (scalar(M) / 2) * metric(M)
}
```

## Symbolic Differentiation

```kleis
data Expr = Const(value : ℝ) 
          | Var(name : String) 
          | Add(left : Expr, right : Expr) 
          | Mul(left : Expr, right : Expr) 
          | Pow(base : Expr, exp : Expr)
          | Sin(arg : Expr)
          | Cos(arg : Expr)
          | Exp(arg : Expr)
          | Ln(arg : Expr)

define diff(e, x) =
    match e {
        Const(_) => Const(0)
        Var(y) => if y = x then Const(1) else Const(0)
        
        Add(f, g) => Add(diff(f, x), diff(g, x))
        
        Mul(f, g) =>
            Add(Mul(diff(f, x), g), Mul(f, diff(g, x)))
            
        Pow(f, Const(n)) =>
            Mul(Mul(Const(n), Pow(f, Const(n - 1))), diff(f, x))
            
        Sin(f) => Mul(Cos(f), diff(f, x))
        Cos(f) => Mul(Mul(Const(-1), Sin(f)), diff(f, x))
        Exp(f) => Mul(Exp(f), diff(f, x))
        Ln(f) => Mul(Pow(f, Const(-1)), diff(f, x))
        
        _ => Const(0)
    }
```

## Linear Algebra

```kleis
structure LinearSystem(n : ℕ) {
    operation solve : Matrix(n, n, ℝ) × Vector(n, ℝ) → Vector(n, ℝ)
    
    // Solution satisfies Ax = b
    axiom solution_correct : ∀(A : Matrix(n, n, ℝ))(b : Vector(n, ℝ)).
        det(A) ≠ 0 → mul(A, solve(A, b)) = b
}

// Eigenvalue problem
structure Eigen(n : ℕ) {
    operation eigenvalues : Matrix(n, n, ℂ) → List(ℂ)
    operation eigenvectors : Matrix(n, n, ℂ) → List(Vector(n, ℂ))
    
    axiom eigenpair : ∀(A : Matrix(n, n, ℂ))(i : ℕ).
        let lam = nth(eigenvalues(A), i) in
        let v = nth(eigenvectors(A), i) in
            mul(A, v) = scale(lam, v)
}
```

## Quantum Mechanics

```kleis
structure QuantumState(n : ℕ) {
    operation amplitudes : Vector(n, ℂ)
    
    // States must be normalized
    axiom normalized : ∀(psi : QuantumState(n)).
        sum(map(λ a . abs(a)^2, amplitudes(psi))) = 1
}

structure Observable(n : ℕ) {
    operation matrix : Matrix(n, n, ℂ)
    
    // Observables are Hermitian
    axiom hermitian : ∀(O : Observable(n)).
        matrix(O) = conjugate_transpose(matrix(O))
}

// Expectation value
define expectation(psi, O) =
    real(inner_product(amplitudes(psi), mul(matrix(O), amplitudes(psi))))
```

## Category Theory

```kleis
structure Category(Obj, Mor) {
    operation id : Obj → Mor
    operation compose : Mor × Mor → Mor
    operation dom : Mor → Obj
    operation cod : Mor → Obj
    
    axiom identity_left : ∀(f : Mor).
        compose(id(cod(f)), f) = f
        
    axiom identity_right : ∀(f : Mor).
        compose(f, id(dom(f))) = f
        
    axiom associativity : ∀(f : Mor)(g : Mor)(h : Mor).
        compose(compose(h, g), f) = compose(h, compose(g, f))
}

structure Functor(C, D) {
    operation map_obj : C → D
    operation map_mor : C → D
    
    axiom preserves_id : ∀(x : C).
        map_mor(id(x)) = id(map_obj(x))
        
    axiom preserves_compose : ∀(f : C)(g : C).
        map_mor(compose(g, f)) = compose(map_mor(g), map_mor(f))
}
```

## Physics: Classical Mechanics

```kleis
structure LagrangianMechanics(n : ℕ) {
    // Generalized coordinates and velocities
    operation q : ℕ → ℝ     // Position
    operation q_dot : ℕ → ℝ  // Velocity
    operation t : ℝ          // Time
    
    // Lagrangian L = T - V
    operation lagrangian : ℝ
    
    // Euler-Lagrange equations
    // Using Mathematica-style: Dt for total derivative, D for partial
    axiom euler_lagrange : ∀ i : ℕ . i < n →
        Dt(D(lagrangian, q_dot(i)), t) = D(lagrangian, q(i))
}
```

## Language Implementation

Kleis can serve as a **meta-language** — a language for implementing other languages. See the complete LISP interpreter in Kleis:

```
λ> :load examples/meta-programming/lisp_parser.kleis

λ> :eval run("(letrec ((fib (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))))) (fib 10))")
✅ VNum(55)

λ> :eval run("(letrec ((fact (lambda (n) (if (<= n 1) 1 (* n (fact (- n 1))))))) (fact 5))")
✅ VNum(120)
```

The complete implementation (parser + evaluator) is ~560 lines of pure Kleis code.

→ [Appendix: LISP Interpreter](../appendix/lisp-interpreter.md) — Full source code with explanation

## What's Next?

Check out the reference appendices!

→ [Appendix A: Grammar Reference](../appendix/grammar.md)
→ [Appendix B: Operators](../appendix/operators.md)
→ [Appendix C: Standard Library](../appendix/stdlib.md)
→ [Appendix D: LISP Interpreter](../appendix/lisp-interpreter.md)
