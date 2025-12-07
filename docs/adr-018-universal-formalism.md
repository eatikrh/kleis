# ADR-018: Universal Formalism with Domain-Specific Rendering

**Date:** December 7, 2024  
**Status:** Accepted  
**Related:** ADR-014 (Type System), ADR-016 (Operations in Structures), ADR-003 (Self-Hosting)

---

## Context

### The Silo Problem

Different communities use different formalisms to express structure:

| Community | Formalism | Notation | Tool Ecosystem |
|-----------|-----------|----------|----------------|
| **Category theorists** | Objects, morphisms, functors | ∘, ⇒, ⊗ | Proof assistants (Coq, Agda) |
| **Set theorists** | Sets, membership, operations | ∈, ⊆, ∪, ∩ | ZFC axioms, LaTeX |
| **Physicists** | States, operators, observables | \|ψ⟩, Ĥ, ⊗ | Quantum packages, LaTeX |
| **Business analysts** | Records, transactions, rules | Forms, tables, SQL | Excel, databases |
| **Type theorists** | Types, constructors, proofs | →, ∀, Σ | Dependent type systems |

### The Cost of Silos

**1. Communication barriers:**
- Physicist can't easily verify their system is "just a category"
- Business analyst can't leverage mathematical proofs
- Duplication of effort (everyone reinvents structures)

**2. Translation is manual and error-prone:**
- Converting between formalisms requires experts
- Errors introduced during translation
- Can't mechanically verify equivalences

**3. Each domain needs separate tools:**
- Different type checkers
- Different proof assistants
- Different notation systems
- **High switching cost** prevents cross-pollination

**4. Mathematical concepts don't transfer:**
- Physicist proves theorem about operators
- Mathematician has equivalent theorem about matrices
- **They can't recognize the equivalence automatically**

### The Opportunity

All these formalisms share common structure:
- **Types** (objects, sets, states, records)
- **Operations** (morphisms, functions, operators, transactions)
- **Axioms** (category laws, set axioms, conservation, business rules)

**If we provide a common substrate**, they could:
- Express their formalism in one system
- Mechanically verify equivalences
- Translate between domains
- **See it in their own notation**

---

## Decision

**Kleis will provide universal formalism with domain-specific rendering:**

### 1. Universal Semantic Layer

**One type system for all domains** (Hindley-Milner + structures):

```kleis
// Category theorist
structure Category(C) {
    objects: Set(Object)
    morphisms: Set(Morphism)
    operation compose : Morphism → Morphism → Morphism
    
    axiom identity: ∀f. id ∘ f = f ∧ f ∘ id = f
    axiom associativity: ∀f g h. (f ∘ g) ∘ h = f ∘ (g ∘ h)
}

// Set theorist
structure ZFC {
    operation (∈) : Element → Set → Bool
    operation (∪) : Set → Set → Set
    
    axiom extensionality: ∀A B. (∀x. x ∈ A ⟺ x ∈ B) ⟹ A = B
    axiom union: ∀x A B. x ∈ (A ∪ B) ⟺ (x ∈ A ∨ x ∈ B)
}

// Physicist
structure QuantumSystem {
    states: HilbertSpace
    operation evolve : Operator → State → State
    operation measure : Observable → State → Distribution(ℝ)
    
    axiom unitarity: ∀U. U† ∘ U = 𝟙
    axiom born_rule: P(outcome) = |⟨outcome|ψ⟩|²
}

// Business analyst
structure PurchaseOrder {
    orderId: String
    items: List(LineItem)
    total: Money
    
    axiom has_items: length(items) > 0
    axiom total_correct: total = Σᵢ items[i].lineTotal
}
```

**All verified by same type system. All can reference each other.**

### 2. Domain-Specific Rendering (Future)

**Each domain sees their own notation:**

```kleis
// Category theorist sees:
@render(Category) for CategoryNotation {
    compose: "{f} ∘ {g}"
    identity: "id_{A}"
    display: diagram
}

// Physicist sees:
@render(QuantumSystem) for DiracNotation {
    states: "|{ψ}⟩"
    operators: "{Ĥ}"
    inner_product: "⟨{φ}|{ψ}⟩"
    display: bra_ket
}

// Business analyst sees:
@render(PurchaseOrder) for FormView {
    template: """
    ┌─────────────────────┐
    │ Order #{orderId}    │
    ├─────────────────────┤
    │ Items: {items}      │
    │ Total: ${total}     │
    └─────────────────────┘
    """
    display: form
}

// Mathematician sees:
@render(PurchaseOrder) for MathNotation {
    template: "PO_{id}(I, Σᵢ p_i)"
    display: symbolic
}
```

**Same structure, different views.**

### 3. Cross-Domain Translation

**Mechanical equivalence checking:**

```kleis
// Physicist asks: "Is my Hilbert space a category?"
prove QuantumSystem implements Category
// Kleis checks if axioms align:
// - States → Objects ✓
// - Operators → Morphisms ✓
// - Sequential evolution → Composition ✓
// - Identity operator → Identity morphism ✓
// - Unitarity → Satisfies composition laws ✓
// Result: Yes, with this mapping!

// Mathematician asks: "Is accounting just monoid theory?"
prove PurchaseOrder implements Monoid(Money, +, 0)
// Checks:
// - total is monoid element ✓
// - addition is the operation ✓
// - zero is identity ✓
// - Associativity holds ✓
// Result: Yes! Accounting is applied algebra!
```

**No longer philosophical debates - mechanical verification.**

---

## Rationale

### Why This Breaks The Silos

**1. Common Verification Infrastructure**

Instead of:
- Category theory → Coq
- Set theory → Isabelle  
- Physics → Mathematica
- Business → Excel + manual review

**Now:**
- **Everyone → Kleis**
- Same type checker
- Same verification engine
- Same proof capabilities

**Benefit:** Cross-domain collaboration becomes feasible.

### 2. Mechanical Translation

**Current state:**
```
Physicist: "Operators form a category"
Mathematician: "Prove it"
Physicist: [writes 50-page proof]
Mathematician: [takes 6 months to verify]
```

**With Kleis:**
```kleis
prove QuantumOperators implements Category
// ✓ Verified in 10ms
// Shows exact mapping
```

**Benefit:** Cross-domain insights become cheap and fast.

### 3. Notation Freedom Without Fragmentation

**The paradox:**
- Physicists **need** Dirac notation (|ψ⟩)
- Business analysts **need** forms and tables
- They can't both use LaTeX effectively

**Kleis solution:**
- **One formalism** (semantics)
- **Many renderings** (presentation)
- Translation: `physicist_view ⟷ business_view`

**Benefit:** Domain comfort without isolation.

### 4. Knowledge Reuse Across Domains

**Example:** Monoid theory

Mathematicians prove theorems about monoids:
```kleis
theorem fold_fusion: 
    ∀f g. homomorphism(f) ⟹ fold(f, g) = f ∘ fold(id, g)
```

**This theorem applies to:**
- String concatenation (CS)
- Purchase order totaling (Business)
- Observable composition (Physics)
- Free group construction (Algebra)

**In Kleis:** Prove once, **apply everywhere**. Type system guarantees validity.

### 5. Democratization of Formal Methods

**Currently:** 
- Only specialists can use theorem provers
- Each domain needs experts in formal verification
- Expensive and rare

**With Kleis:**
- Business analyst uses same tool as mathematician
- Type checker is the same verification engine
- **Formal methods become accessible**

---

## Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────────┐
│  Presentation Layer (Domain-Specific)      │
│  - Category Diagrams                        │
│  - Dirac Notation                          │
│  - Business Forms                          │
│  - Mathematical Symbols                    │
└─────────────────────────────────────────────┘
                    ↕ Rendering
┌─────────────────────────────────────────────┐
│  Semantic Layer (Universal)                │
│  - Type System (Hindley-Milner)            │
│  - Structures (ADR-016)                     │
│  - Operations                               │
│  - Axioms                                   │
└─────────────────────────────────────────────┘
                    ↕ Verification
┌─────────────────────────────────────────────┐
│  Foundation (Mathematics)                   │
│  - Constraint solving                       │
│  - Unification                              │
│  - Proof checking                           │
└─────────────────────────────────────────────┘
```

### Implementation Phases

#### Phase 1: Universal Semantics (Current)
- ✅ Type system (HM)
- ✅ Structures (ADR-016)
- ✅ Operations
- ⬜ Cross-domain examples

**Status:** Core working!

#### Phase 2: Basic Rendering Extension (6 months)
- ⬜ User-defined templates in `.kleis` files
- ⬜ Multiple render targets (LaTeX, HTML, Typst)
- ⬜ Template registry
- ⬜ Fallback to default notation

#### Phase 3: Domain-Specific Rendering (12 months)
- ⬜ `@render` decorator
- ⬜ Rendering conditions (context-dependent)
- ⬜ Template inheritance
- ⬜ Custom layouts (forms, diagrams, tables)

#### Phase 4: Translation System (18 months)
- ⬜ Mechanical equivalence checking
- ⬜ Mapping inference
- ⬜ Translation proof obligations
- ⬜ Bidirectional translation

---

## Examples

### Example 1: Physicist ⟷ Category Theorist

**Physicist defines:**
```kleis
structure QuantumSystem {
    states: HilbertSpace
    operation (⊗) : State → State → State  // Tensor product
    operation U : State → State             // Unitary evolution
    
    axiom unitarity: ∀ψ. ⟨U(ψ)|U(ψ)⟩ = ⟨ψ|ψ⟩
}

@render(QuantumSystem) for Dirac {
    states: "|{ψ}⟩"
    tensor: "{ψ₁} ⊗ {ψ₂}"
}
```

**Category theorist proves equivalence:**
```kleis
// Show it's a category
mapping QuantumToCategory {
    State → Object
    UnitaryOperator → Morphism
    sequential_evolution → composition
    identity_operator → identity
}

prove QuantumSystem ≅ Category via QuantumToCategory
// Kleis verifies:
// ✓ Composition defined (sequential evolution)
// ✓ Identity exists (𝟙)
// ✓ Associativity (unitarity ensures this)
// ✓ Result: They're the same structure!
```

**Both see their notation, both trust the verification.**

### Example 2: Accountant ⟷ Mathematician

**Accountant defines:**
```kleis
structure GeneralLedger {
    accounts: List(Account)
    operation post : Transaction → GeneralLedger → GeneralLedger
    
    axiom balance: ∀t. sum(debits(t)) = sum(credits(t))
    axiom conservation: assets = liabilities + equity
}

@render(GeneralLedger) for Accounting {
    display: ledger_table
    accounts: "{name}: ${balance}"
}
```

**Mathematician recognizes:**
```kleis
prove GeneralLedger implements Monoid(Account, (+), ZeroBalance)
// ✓ Verified! Accounting is applied monoid theory!

// This means:
// - All monoid theorems apply to accounting
// - Fold/reduce operations work
// - Homomorphism properties guarantee correctness
```

**Accountant doesn't need to know category theory.**  
**Mathematician doesn't need to know accounting.**  
**Kleis connects them mechanically.**

### Example 3: Business Rules ⟷ Physics

**Business analyst:**
```kleis
structure Inventory {
    items: List(Item)
    operation restock : Item → Quantity → Inventory
    
    axiom conservation: 
        incoming + manufactured = outgoing + stock
}
```

**Physicist recognizes:**
```kleis
// This is conservation law!
prove Inventory.conservation ≅ PhysicsConservation
// - incoming ↔ particles_in
// - outgoing ↔ particles_out  
// - stock ↔ field_density
// ✓ Same mathematical structure!

// Insights transfer:
// - Physics: Use Noether's theorem
// - Business: Symmetry → conserved quantity
// - Apply to inventory: Time-invariance → stock levels
```

**Cross-pollination that was impossible before.**

---

## Architecture Details

### Rendering System Design

```rust
// In src/render.rs (future extension)

struct RenderContext {
    // Current: built-in targets
    target: RenderTarget,  // LaTeX, HTML, Typst, Unicode
    
    // Future: user-defined templates
    custom_templates: HashMap<String, CustomTemplate>,
    
    // Future: domain preferences
    domain: Option<DomainContext>,
}

struct CustomTemplate {
    structure_name: String,
    render_target: String,
    template: String,
    styles: HashMap<String, StyleRule>,
}

// Defined in .kleis files:
@render_target("dirac")
@render(QuantumSystem) {
    state: "|{name}⟩"
    operator: "{name}̂"
    inner_product: "⟨{left}|{right}⟩"
}

@render_target("accounting")  
@render(GeneralLedger) {
    template: ledger_table(
        columns: ["Account", "Debit", "Credit", "Balance"],
        rows: accounts,
        total_row: true
    )
}
```

### Translation System Design

```rust
// In src/translation.rs (future)

struct DomainMapping {
    from_structure: String,
    to_structure: String,
    type_mapping: HashMap<String, String>,
    operation_mapping: HashMap<String, String>,
    axiom_mapping: Vec<(String, String)>,
}

// User defines mapping
mapping QuantumToCategory {
    State → Object
    UnitaryOperator → Morphism
    sequential_evolution → compose
    identity_operator → identity
}

// Kleis verifies mapping preserves structure
fn verify_mapping(mapping: DomainMapping) -> Result<Proof, MappingError> {
    // 1. Check all types map
    // 2. Check operations preserve types
    // 3. Check axioms preserved under mapping
    // 4. Return constructive proof or counterexample
}
```

---

## Benefits

### 1. Breaking Knowledge Silos

**Before:** 
- Category theory papers unreadable to physicists
- Business logic inaccessible to mathematicians
- Each domain reinvents the wheel

**After:**
- Physicist can read category paper (rendered in Dirac notation)
- Mathematician can verify business rules
- Patterns recognized across domains

### 2. Mechanical Cross-Domain Verification

**Example applications:**
```kleis
// Check if accounting follows group theory
prove AccountingRules implements AbelianGroup

// Check if quantum mechanics is categorical
prove QuantumMechanics implements MonoidalCategory

// Check if business workflow is algebraic
prove WorkflowSystem implements Lattice
```

**Insights that were impossible to verify mechanically before.**

### 3. Universal Verification Engine

**One tool verifies:**
- Mathematical papers (Category theory)
- Physics theories (Quantum mechanics)
- Business rules (Purchase orders)
- Legal contracts (Conditions and obligations)
- Tax returns (IRS regulations)

**Same verification, different domains.**

### 4. Knowledge Transfer

**Theorem proven in one domain applies to all equivalent structures:**

```kleis
// Mathematician proves:
theorem monoid_fold_fusion: ∀f g. homomorphism(f) ⟹ ...

// Automatically applies to:
- String concatenation (CS)
- Observable composition (Physics)
- Transaction merging (Accounting)
- Inventory combining (Supply chain)

// Because they all implement Monoid!
```

### 5. Democratization of Formal Methods

**Currently:** Only specialists use formal verification  
**With Kleis:** Anyone defining structures gets verification

**Impact:** 
- Business analysts get mathematical rigor
- Physicists get proof automation
- Mathematicians get applied validation

---

## Implementation Roadmap

### Stage 1: Foundation (✅ Complete)
- ✅ Type system working (HM)
- ✅ Structures (ADR-016)
- ✅ Operations defined in structures
- ✅ Basic rendering (LaTeX, Typst, HTML)

### Stage 2: Cross-Domain Examples (Next 3 months)
- ⬜ Implement: Category, Monoid, Group in stdlib
- ⬜ Implement: PurchaseOrder, Invoice in examples
- ⬜ Implement: QuantumSystem in examples
- ⬜ Demonstrate equivalence checking

### Stage 3: User-Defined Templates (6 months)
- ⬜ Parse `@render` decorators
- ⬜ Template registry in TypeContext
- ⬜ Custom rendering pipeline
- ⬜ Fallback to default notation

### Stage 4: Domain-Specific Views (12 months)
- ⬜ Form rendering for business types
- ⬜ Diagram rendering for categories
- ⬜ Table rendering for data
- ⬜ Context-dependent rendering

### Stage 5: Translation System (18 months)
- ⬜ Mapping definitions
- ⬜ Equivalence checking
- ⬜ Proof generation
- ⬜ Bidirectional translation

---

## Consequences

### Positive

1. ✅ **Breaks knowledge silos** - Common formalism enables communication
2. ✅ **Mechanical verification** - No manual translation errors
3. ✅ **Universal tool** - One system, infinite domains
4. ✅ **Knowledge reuse** - Theorems apply across domains
5. ✅ **Accessible formal methods** - Not just for specialists

### Negative

1. ⚠️ **Implementation complexity** - Rendering system is substantial work
2. ⚠️ **Adoption challenge** - Each domain needs convincing separately
3. ⚠️ **Performance** - Multiple renderings may impact speed
4. ⚠️ **Maintenance** - More rendering targets = more code

### Neutral

1. **Notation debates** - Some domains may resist change
2. **Template quality** - Users need to create good templates
3. **Learning curve** - Understanding universal formalism takes time

---

## Related ADRs

**ADR-003: Self-Hosting**
- Kleis defines Kleis (including its own structures)
- Universal formalism applies to Kleis itself

**ADR-014: Hindley-Milner Type System**
- Provides the foundation for universal verification
- Example 4 shows PurchaseOrder (user-defined type)

**ADR-015: Text as Source of Truth**
- Structures defined in .kleis files
- Applies to all domains equally

**ADR-016: Operations in Structures**
- Operations belong to types, not hardcoded
- Enables each domain to define their operations
- Foundation for extensibility

---

## Success Criteria

**By 2026:**
- ✅ 3+ domains demonstrably working (Math, Physics, Business)
- ✅ 1+ mechanical equivalence proof between domains
- ✅ User-defined rendering working for at least one domain
- ✅ Documentation showing cross-domain translation

**By 2028:**
- 10+ domains using Kleis
- Standard library covering major mathematical structures
- Community-contributed domain templates
- Cross-domain theorems being discovered

**By 2030:**
- Kleis as infrastructure for formal verification
- Academic adoption (papers submitted with Kleis verification)
- Industry adoption (contracts, financial statements)
- **"Kleis-Verified" becomes a quality standard**

---

## Conclusion

**Kleis is not a mathematics tool.**  
**Kleis is not a programming language.**  

**Kleis is a universal substrate for formal reasoning with domain-specific presentation.**

By providing:
- **One type system** (verifies all)
- **One structure system** (expresses all)
- **Many renderings** (serves all)

We enable:
- Breaking knowledge silos
- Mechanical cross-domain translation
- Universal verification infrastructure
- Knowledge reuse at scale

**This is infrastructure**, not an application.

Like how:
- **HTTP** unified communication (one protocol, many applications)
- **SQL** unified data (one query language, many databases)
- **Git** unified version control (one system, many projects)

**Kleis unifies formal reasoning** (one verification, many domains).

---

**Status:** Accepted as architectural direction  
**Timeline:** 10-year vision, starting with mathematics  
**Current Phase:** Stage 1 complete, Stage 2 beginning  
**Next Steps:** Implement cross-domain examples (Category, Monoid, PurchaseOrder, QuantumSystem)

---

**Last Updated:** December 7, 2024  
**Decision:** Universal formalism with domain-specific rendering ✅

