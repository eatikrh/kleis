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
   │  Parse  │──── Syntax error? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │  Load   │──── Name conflict? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │ Verify  │──── Inconsistent axioms? → reject, theory unchanged
   └────┬────┘
        │
   ┌────▼────┐
   │ Commit  │──── Append to session.kleis, rebuild evaluator
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

## Walkthrough: UN Charter Article 51 — Legal Cross-Examination

The previous walkthrough formalized a physics theory. This one formalizes
international law and uses Z3 as a cross-examiner.

### The Legal Question

Article 2(4) of the UN Charter prohibits the use of force between states.
Article 51 provides one exception: self-defense. But two competing doctrines
disagree on when self-defense is lawful:

- **Strict doctrine**: only after an *armed attack* has occurred
- **Anticipatory doctrine**: also when an attack is *imminent*

This disagreement has real consequences. By formalizing both doctrines as
axiom packs over a shared ontology, Z3 can identify exactly where they
diverge — and prove what follows from each.

### The Ontology

Types and operations that both doctrines share. The key design decision:
`lawful_self_defense` and `status` are parameterized by `Doctrine`, so
both doctrines coexist in the same file without axiom collision:

```kleis
data State = US | Iran | Russia | Ukraine | Israel | Lebanon | CustomState(ℤ)
data Norm = Permitted | Prohibited
data Act = MilitaryAct(ℤ)
data ThreatEvidence = Intel(ℤ)
data Doctrine = Strict | Anticipatory

operation use_of_force   : Act → Bool
operation attacker       : Act → State
operation target         : Act → State
operation sc_authorized  : Act → Bool
operation necessary      : Act → Bool
operation proportional   : Act → Bool
operation armed_attack_occurred : State × State → Bool
operation imminent_attack       : State × State → Bool

operation lawful_self_defense : Doctrine × Act → Bool
operation status              : Doctrine × Act → Norm
```

### Article 2(4): The Baseline

Force is prohibited unless an exception applies. A single axiom encodes
both directions, universally quantified over doctrines:

```kleis
structure Article2_4 {
    axiom force_status : ∀(d : Doctrine, a : Act).
        use_of_force(a) →
            ((status(d, a) = Permitted ↔ (sc_authorized(a) ∨ lawful_self_defense(d, a)))
           ∧ (status(d, a) = Prohibited ↔ ¬(sc_authorized(a) ∨ lawful_self_defense(d, a))))
}
```

### The Two Doctrines

The strict doctrine requires an armed attack. Each axiom is pinned to
its `Doctrine` value — `Strict` or `Anticipatory` — so they never
interfere:

```kleis
structure StrictArticle51 {
    axiom sd_requires : ∀(a : Act).
        lawful_self_defense(Strict, a) →
            (necessary(a) ∧ proportional(a) ∧
             armed_attack_occurred(target(a), attacker(a)))
}
```

The anticipatory doctrine also accepts imminence:

```kleis
structure AnticipatoryArticle51 {
    axiom sd_requires : ∀(a : Act).
        lawful_self_defense(Anticipatory, a) →
            (necessary(a) ∧ proportional(a) ∧
             (armed_attack_occurred(target(a), attacker(a))
              ∨ imminent_attack(target(a), attacker(a))))
}
```

The only difference is one disjunct: `∨ imminent_attack(...)`. The
`Doctrine` parameter ensures each doctrine's axioms operate independently
— no silent domination.

### The Case File and Evidence Profile

Facts of a disputed strike — US attacks Iran, no SC authorization, no prior
armed attack. The `NoEvidence` structure prevents Z3 from inventing
phantom witnesses to satisfy the imminence predicate:

```kleis
structure CaseFacts {
    element case_act : Act
    axiom is_force : use_of_force(case_act)
    axiom who_attacks : attacker(case_act) = US
    axiom who_is_target : target(case_act) = Iran
    axiom no_sc_auth : ¬sc_authorized(case_act)
    axiom no_prior_attack : ¬armed_attack_occurred(Iran, US)
}

structure NoEvidence {
    axiom no_ev : ∀(e : ThreatEvidence). ¬evidence(Iran, US, e)
}
```

Without `NoEvidence`, Z3 could freely instantiate `Intel(k)` for any `k`
and set `credible`, `immediate`, `no_alternative` to true — technically
satisfying the imminence definition. The closed-world guard forces the
case file to supply evidence or accept its absence.

### Verified Theorems

The file lives under `examples/authorization/` alongside Zanzibar and OAuth2
models. This is not an accident — the structure is the same. Authorization
asks "is this action permitted given these rules and this identity?"
The UN Charter asks "is this use of force permitted given these doctrines
and these facts?" Both are judgment functions over acts. The difference
is that authorization is a gate *before* the act; international law is
a judgment *after* the act has already occurred.

Load the theory into the kleis-theory MCP:

```
load_theory(imports: ["examples/authorization/un_charter_article51.kleis"])
→ ✅ 6 structures loaded
```

### Cross-Examination

The human asks questions in natural language. The AI agent inspects the
loaded axioms and translates each question into a Kleis proposition,
which Z3 either verifies or disproves. The human never writes Kleis —
the agent does.

**Human:** *"Is the act a use of force?"*

```
evaluate: ∀(a : Act). a = case_act → use_of_force(a)
→ ✅ VERIFIED
```

**Human:** *"Did the Security Council authorize it?"*

```
evaluate: ∀(a : Act). a = case_act → sc_authorized(a)
→ ❌ DISPROVED
```

**Human:** *"Did an armed attack occur?"*

```
evaluate: ∀(dummy : Act). ¬armed_attack_occurred(Iran, US)
→ ✅ VERIFIED (none occurred)
```

**Human:** *"Does any evidence of imminence exist?"*

```
evaluate: ∀(e : ThreatEvidence). ¬evidence(Iran, US, e)
→ ✅ VERIFIED (none exists)
```

**Human:** *"Is imminence established?"*

```
evaluate: ∀(v : State, a : State). v = Iran ∧ a = US → ¬imminent_attack(v, a)
→ ✅ VERIFIED (impossible without evidence)
```

**Human:** *"Can self-defense be claimed under either doctrine?"*

```
evaluate: ∀(a : Act). a = case_act → lawful_self_defense(Strict, a)
→ ❌ DISPROVED

evaluate: ∀(a : Act). a = case_act → lawful_self_defense(Anticipatory, a)
→ ❌ DISPROVED
```

**Human:** *"What is the legal status?"*

```
evaluate: ∀(a : Act). a = case_act → status(Strict, a) = Prohibited
→ ✅ VERIFIED

evaluate: ∀(a : Act). a = case_act → status(Anticipatory, a) = Prohibited
→ ✅ VERIFIED
```

**Human:** *"Could it ever be Permitted?"*

```
evaluate: ∀(a : Act). a = case_act → status(Strict, a) = Permitted
→ ❌ DISPROVED

evaluate: ∀(a : Act). a = case_act → status(Anticipatory, a) = Permitted
→ ❌ DISPROVED
```

**Human:** *"Do both doctrines agree?"*

```
evaluate: ∀(a : Act). a = case_act →
    (status(Strict, a) = Prohibited ∧ status(Anticipatory, a) = Prohibited)
→ ✅ VERIFIED
```

### The Verdict

Both doctrines produce the same verdict: **Prohibited**. The anticipatory
doctrine *could* diverge, but only if the case file supplies an
`EvidenceSupported` profile instead of `NoEvidence`. Without evidence,
imminence is logically impossible, and both doctrines collapse to the
same judgment. The `Doctrine` parameter makes this comparison explicit
in a single Z3 query.

### Why This Matters

The cross-examination is not a simulation — it is a proof. The human
asked questions in English; the agent translated them into formal
propositions; Z3 checked every possible model. No rhetoric, no
ambiguity, no appeal to authority. Just axioms, facts, and a solver.

Change the case facts (assert `sc_authorized`, or
`armed_attack_occurred`) and the verdict flips — same law, different
facts, different theorem.

The same kleis-theory MCP that built a physics theory in the previous
walkthrough now performs legal reasoning. The substrate is the same:
structures, axioms, Z3 verification. The domain changed; the method
did not.

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

### From Asking a Lawyer (or an LLM)

In the UN Charter cross-examination, the human did not need to know
international law. The human said "cross-examine this case" and the agent
did the rest — it queried the loaded theory via the MCP, discovered which
structures and predicates were available, understood the logical
dependencies between them, and constructed the propositions in Kleis
syntax. The human asked in English; the agent translated to formal logic;
Z3 returned the verdict.

This is different from asking an LLM "is this attack legal?" An LLM
would produce a plausible paragraph citing Article 51, hedging with
"it depends," and possibly hallucinating case law. The agent does
something structurally different: it reads the axioms that are actually
loaded, constructs propositions that reference them precisely, and submits
those propositions to a solver that checks every possible model.
If the agent gets the proposition wrong, Z3 rejects it. If the axioms
are inconsistent, Z3 reports inconsistency. There is no room for
plausible-sounding nonsense.

The axioms are self-describing — `lawful_self_defense`, `necessary`,
`proportional`, `armed_attack_occurred` — the names carry the semantics.
The structures tell the agent which predicates gate which conclusions.
The agent does not "know" international law; it reads the formal encoding
and reasons over it. The domain knowledge lives in the `.kleis` file,
not in the model's training data.

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

-> [Previous: Control Systems](./26-control-systems.md) | [Next: Agent MCP Servers](./28-agent-mcps.md)
