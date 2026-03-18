# Petri Net Analysis with Kleis

**Formal verification of Petri net properties using Z3.**

Based on **ISO/IEC 15909-2** Petri Net Markup Language (PNML) standard.

---

## 🎯 What Kleis Brings to Petri Nets

| Capability | Kleis Advantage |
|------------|-----------------|
| **Safety properties** | Z3 verifies mutual exclusion, boundedness |
| **Liveness properties** | Deadlock-freedom, transition liveness |
| **Invariants** | Place/transition invariants as axioms |
| **Colored nets** | Algebraic types for token colors |
| **Formal proofs** | Not simulation - actual mathematical proofs |

---

## 📁 Files

### `petri_core.kleis`
Core Petri net library:
- `Place`, `Transition`, `Arc` data types
- `Marking` as function `Place → ℕ`
- `enabled`, `fire` operations with axioms
- `reachable` relation (reflexive-transitive closure)
- Safety: `k_bounded`, `safe`
- Liveness: `dead`, `deadlock`, `live`
- Invariants: `PlaceInvariant`, `valid_invariant`

### `mutex_example.kleis`
Mutual exclusion protocol:
```
   [P1_idle]──▶(enter1)──▶[P1_critical]──▶(exit1)
        │         │              │           │
        │         ▼              ▼           │
        │      [mutex]◀──────────────────────┘
        │         │              │           │
        │         ▼              ▼           │
   [P2_idle]──▶(enter2)──▶[P2_critical]──▶(exit2)
```

Verified properties:
- ✅ Mutual exclusion: both cannot be in critical section
- ✅ Deadlock-freedom
- ✅ All places are 1-bounded (safe net)
- ✅ Token conservation (place invariant)
- ✅ Liveness: all transitions eventually fireable

### `colored_petri.kleis`
High-Level (Colored) Petri Nets:
- Typed tokens with color sets
- Multiset operations (`ms_add`, `ms_sub`, `ms_leq`)
- Arc inscriptions and guards
- Standard color sets: `Dot`, `Bool`, finite enumerations
- Product types for structured tokens

---

## 🔬 Example Verification

```kleis
example "mutual exclusion" {
    // Safety: both processes cannot be in critical section simultaneously
    assert(∀(m : Marking). reachable(initial, m, arcs) → 
        ¬(m(p1_critical) ≥ 1 ∧ m(p2_critical) ≥ 1))
}

example "deadlock-free" {
    // Liveness: there's always some enabled transition
    assert(deadlock_free(initial, all_transitions, arcs))
}

example "token conservation" {
    // Place invariant: total tokens = 3
    assert(∀(m : Marking). reachable(initial, m, arcs) → 
        m(p1_idle) + m(p1_critical) + m(p2_idle) + m(p2_critical) + m(mutex) = 3)
}
```

---

## 🔗 PNML Compatibility

This library is based on the ISO/IEC 15909-2 PNML standard:

| PNML Concept | Kleis Representation |
|--------------|---------------------|
| `<place>` | `data Place = Place(id : String)` |
| `<transition>` | `data Transition = Transition(id : String)` |
| `<arc>` | `data Arc = InputArc(...) \| OutputArc(...)` |
| `<initialMarking>` | `type Marking = Place → ℕ` |
| High-level sorts | Kleis algebraic data types |
| Multisets | `type ColoredMarking(T) = T → ℕ` |

---

## 📚 Applications

Petri nets model:
- **Concurrent systems** - process synchronization, resource sharing
- **Protocols** - communication protocols, handshaking
- **Workflows** - business processes, state machines
- **Hardware** - circuit verification, pipeline analysis
- **Biology** - metabolic pathways, gene regulation

With Kleis + Z3, you can **prove** properties rather than just test them.

---

## 🚀 Future Extensions

- [ ] PNML XML import/export
- [ ] Timed Petri nets
- [ ] Stochastic Petri nets
- [ ] Hierarchical nets (pages)
- [ ] Model checking integration

