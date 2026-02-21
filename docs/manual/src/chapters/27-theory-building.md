# Interactive Theory Building

Kleis is not just a language for writing mathematics — it is a substrate for
**producing** mathematics. This chapter shows how an AI agent can co-author
formal theories with a human researcher, using the kleis-theory MCP to submit
definitions, check consistency, and derive verified theorems in real time.

## The Idea

Traditional theorem provers require the human to write everything. The agent
workflow is different:

1. **Human** describes the mathematical concepts informally
2. **Agent** formalizes them as Kleis structures with operations and axioms
3. **Kleis + Z3** validates each submission (parsing, loading, consistency)
4. **Agent** proposes conjectures and Z3 either verifies or rejects them
5. **Human** guides the exploration — "what about nullspaces?" "try composing kernels"

The result is a `.kleis` file containing a verified theory that neither the
human nor the agent could have produced as efficiently alone.

## The kleis-theory MCP

The theory-building workflow is powered by a Model Context Protocol server that
exposes nine tools:

| Tool | Purpose |
|------|---------|
| `submit_structure` | Add a structure (with operations and axioms) to the theory |
| `submit_define` | Add a function definition |
| `submit_data` | Add algebraic data types |
| `try_structure` | Dry-run a submission without modifying the theory |
| `evaluate` | Evaluate an expression or verify a proposition via Z3 |
| `describe_schema` | List everything currently loaded |
| `list_session` | Show the history of agent submissions |
| `load_theory` | Reset and load a saved theory |
| `save_theory` | Save the current theory to a `.kleis` file |

### Try-Before-Commit

Every submission goes through a three-stage pipeline:

```
Agent submits Kleis source
        │
   ┌────▼────┐
   │  Parse   │──── Syntax error? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │  Load    │──── Name conflict? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │ Verify   │──── Inconsistent axioms? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │ Commit   │──── Append to session.kleis, rebuild evaluator
   └─────────┘
```

The agent's proposed code is first written to a scratch file and validated
against a fresh evaluator. Only on success does it become part of the theory.
This prevents a bad submission from corrupting accumulated work.

## Walkthrough: Admissible Kernels in Projected Ontology Theory

This section reproduces an actual theory-building session. The goal: formalize
the concept of admissible kernels from Projected Ontology Theory (POT) and
discover what theorems follow from the axioms.

### Step 1: Data Types

Every formalization starts with the types. POT has three:

```kleis
data GreenKernel = GK(ℤ)
data Flow = Fl(ℤ)
data FieldR4 = FR4(ℤ)
```

These are "uninterpreted" types — Kleis and Z3 know they exist and are distinct,
but nothing about their internal structure. This is deliberate: POT is meant to
be general, not tied to specific representations.

### Step 2: Algebraic Structures

Flows and fields need additive structure:

```kleis
structure FlowAlgebra {
    operation flow_add : Flow -> Flow -> Flow
    operation flow_smul : ℂ -> Flow -> Flow
    element flow_zero : Flow
    axiom flow_add_comm : ∀(a b : Flow). flow_add(a, b) = flow_add(b, a)
    axiom flow_add_id : ∀(a : Flow). flow_add(a, flow_zero) = a
}

structure FieldAlgebra {
    operation field_add : FieldR4 -> FieldR4 -> FieldR4
    operation field_smul : ℂ -> FieldR4 -> FieldR4
    element field_zero : FieldR4
    axiom field_add_comm : ∀(f g : FieldR4). field_add(f, g) = field_add(g, f)
    axiom field_add_id : ∀(f : FieldR4). field_add(f, field_zero) = f
}
```

Each submission is validated before it enters the theory. If you wrote
`flow_add_comm` with a typo in the operation name, the pipeline would catch
it at the Load stage.

### Step 3: The Core Axioms

Now the heart of the theory — what makes a kernel "admissible":

```kleis
operation apply_kernel : GreenKernel -> Flow -> FieldR4

structure AdmissibleKernel {
    operation is_admissible : GreenKernel -> Bool

    axiom kernel_lin_add : ∀(G : GreenKernel, a b : Flow).
        implies(is_admissible(G),
            apply_kernel(G, flow_add(a, b))
              = field_add(apply_kernel(G, a), apply_kernel(G, b)))

    axiom kernel_lin_smul : ∀(G : GreenKernel, c : ℂ, a : Flow).
        implies(is_admissible(G),
            apply_kernel(G, flow_smul(c, a))
              = field_smul(c, apply_kernel(G, a)))

    axiom kernel_maps_zero : ∀(G : GreenKernel).
        implies(is_admissible(G),
            apply_kernel(G, flow_zero) = field_zero)
}
```

Three axioms: linearity over addition (K1a), linearity over scalar
multiplication (K1b), and zero preservation (K2). These are the weakest
constraints that make the kernel "transform-like" without hardcoding any
specific physics.

### Step 4: Projective Equivalence

Two flows are equivalent under a kernel if they project to the same field:

```kleis
structure ProjectiveEquivalence {
    operation equiv : GreenKernel -> Flow -> Flow -> Bool
    axiom equiv_intro : ∀(G : GreenKernel, a b : Flow).
        implies(apply_kernel(G, a) = apply_kernel(G, b), equiv(G, a, b))
    axiom equiv_elim : ∀(G : GreenKernel, a b : Flow).
        implies(equiv(G, a, b), apply_kernel(G, a) = apply_kernel(G, b))
}
```

We axiomatized only the **definition** — `equiv` holds if and only if the
kernel maps agree. We did **not** axiomatize reflexivity, symmetry, or
transitivity. Those should follow.

### Step 5: Derived Theorems

Now the agent asks Z3 to verify consequences:

```
evaluate: ∀(G : GreenKernel, a : Flow). equiv(G, a, a)
→ ✅ VERIFIED (reflexivity)

evaluate: ∀(G : GreenKernel, a b : Flow).
    implies(equiv(G, a, b), equiv(G, b, a))
→ ✅ VERIFIED (symmetry)

evaluate: ∀(G : GreenKernel, a b c : Flow).
    implies(and(equiv(G, a, b), equiv(G, b, c)), equiv(G, a, c))
→ ✅ VERIFIED (transitivity)
```

Z3 proves all three. The proof chain: `equiv(G, a, b)` →
`apply_kernel(G, a) = apply_kernel(G, b)` (by `equiv_elim`) → equality is
an equivalence relation → `equiv(G, a, a)` (by `equiv_intro`). We derived
that projective equivalence is a proper equivalence relation without ever
stating it as an axiom.

### Step 6: Nullspace

The nullspace of a kernel is the set of flows it kills:

```kleis
structure Nullspace {
    operation in_null : GreenKernel -> Flow -> Bool
    axiom null_def : ∀(G : GreenKernel, a : Flow).
        implies(in_null(G, a), apply_kernel(G, a) = field_zero)
    axiom null_intro : ∀(G : GreenKernel, a : Flow).
        implies(apply_kernel(G, a) = field_zero, in_null(G, a))
    axiom null_closed_add : ∀(G : GreenKernel, a b : Flow).
        implies(is_admissible(G),
            implies(and(in_null(G, a), in_null(G, b)),
                in_null(G, flow_add(a, b))))
    axiom null_closed_smul : ∀(G : GreenKernel, c : ℂ, a : Flow).
        implies(is_admissible(G),
            implies(in_null(G, a), in_null(G, flow_smul(c, a))))
}
```

More derived theorems:

```
evaluate: ∀(G : GreenKernel, a b : Flow).
    implies(and(is_admissible(G), and(in_null(G, a), in_null(G, b))),
        equiv(G, a, b))
→ ✅ VERIFIED (nullspace elements are projectively equivalent)

evaluate: ∀(G : GreenKernel). in_null(G, flow_zero)
→ ❌ UNKNOWN
```

The second query returns Unknown. Why? Because `kernel_maps_zero` only applies
to **admissible** kernels. The correct formulation is:

```
evaluate: ∀(G : GreenKernel).
    implies(is_admissible(G), in_null(G, flow_zero))
→ ✅ VERIFIED
```

This is not a failure — it's the system working as intended. Z3 caught a
missing precondition that a human might overlook. The "Unknown" result tells
the theorist: your axioms don't support this claim as stated.

### Step 7: Kernel Composition

Can admissible kernels be composed while preserving admissibility?

```kleis
structure KernelComposition {
    operation compose_kernel : GreenKernel -> GreenKernel -> GreenKernel
    axiom compose_admissible : ∀(G1 G2 : GreenKernel).
        implies(and(is_admissible(G1), is_admissible(G2)),
            is_admissible(compose_kernel(G1, G2)))
}
```

```
evaluate: ∀(G1 G2 : GreenKernel).
    implies(and(is_admissible(G1), is_admissible(G2)),
        in_null(compose_kernel(G1, G2), flow_zero))
→ ✅ VERIFIED
```

Z3 chains the reasoning: two admissible kernels compose to an admissible kernel
(axiom), admissible kernels map zero to zero (axiom), zero-mapping implies
nullspace membership (axiom). Three hops, fully automatic.

### Step 8: Save

```
save_theory(name: "pot_admissible_kernels")
→ ✅ Saved to theories/pot_admissible_kernels.kleis
```

The theory is now a plain `.kleis` file that can be imported, extended,
or loaded into a future session.

## What Makes This Different

### From a Chat About Math

A language model can talk about admissible kernels all day. It can generate
plausible-sounding theorems. But it cannot **verify** them. With kleis-theory,
every claim the agent makes passes through Z3. When the agent proposed
`in_null(G, flow_zero)` without the admissibility precondition, Z3 said no.
The agent had to correct its reasoning.

### From a Traditional Proof Assistant

In Lean, Coq, or Isabelle, the human writes everything. The assistant checks.
With kleis-theory, the agent writes the formalizations. The human says "what
about nullspaces?" and the agent produces the structure, the axioms, and the
verification queries. The human guides; the agent formalizes; Z3 verifies.

### From a Notebook

A Jupyter notebook runs code. It doesn't accumulate verified knowledge. Each
cell is independent. In kleis-theory, each submission builds on everything
before it. The theory grows monotonically. Nothing is forgotten, nothing
contradicts.

## The Theory File

The saved theory is a standard Kleis file:

```kleis
import "stdlib/prelude.kleis"

data GreenKernel = GK(ℤ)
data Flow = Fl(ℤ)
data FieldR4 = FR4(ℤ)

structure FlowAlgebra {
    operation flow_add : Flow -> Flow -> Flow
    operation flow_smul : ℂ -> Flow -> Flow
    element flow_zero : Flow
    axiom flow_add_comm : ∀(a b : Flow). flow_add(a, b) = flow_add(b, a)
    axiom flow_add_id : ∀(a : Flow). flow_add(a, flow_zero) = a
}

// ... AdmissibleKernel, ProjectiveEquivalence, Nullspace, KernelComposition
```

It can be imported into other Kleis files, loaded into the REPL, or used as
the starting point for a future theory session:

```
load_theory(imports: ["theories/pot_admissible_kernels.kleis"])
```

## Summary of Verified Results

From the axioms of admissible kernels, Z3 derived:

| # | Theorem | Status |
|---|---------|--------|
| 1 | `equiv(G, a, a)` — reflexivity | ✅ Verified |
| 2 | `equiv(G, a, b) → equiv(G, b, a)` — symmetry | ✅ Verified |
| 3 | `equiv(G, a, b) ∧ equiv(G, b, c) → equiv(G, a, c)` — transitivity | ✅ Verified |
| 4 | `in_null(G, a) ∧ in_null(G, b) → equiv(G, a, b)` — null equivalence | ✅ Verified |
| 5 | `is_admissible(G) → in_null(G, flow_zero)` — zero in nullspace | ✅ Verified |
| 6 | Composed admissible kernels preserve nullspace of zero | ✅ Verified |
| 7 | `in_null(G, flow_zero)` without admissibility precondition | ❌ Unknown |

The "Unknown" in row 7 is not a bug — it's Z3 correctly identifying that the
axioms don't support the claim as stated. This kind of feedback is what makes
interactive theory building productive.

## Enabling the kleis-theory MCP

Toggle it with:

```bash
scripts/theory.sh on      # Enable
scripts/theory.sh off     # Disable
scripts/theory.sh status  # Check
```

When enabled, the nine tools become available to the AI agent in Cursor.

## What's Next

The theory file is the beginning, not the end. Future sessions can:

- **Extend** the theory with new structures (e.g., `PhaseErasure`, `Residues`)
- **Specialize** to specific physics (GR kernels, QM propagators)
- **Connect** multiple theories via imports
- **Compute** with numerical methods (LAPACK, ODE solver) on concrete instances

The substrate is ready. Build on it.

→ [Previous: Control Systems](./26-control-systems.md)
