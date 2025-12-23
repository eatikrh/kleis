# Chapter 2: The 17 Rules of the Game

If Chapter 1 was about the anatomy of Kleis (Names and Arrows), Chapter 2 is about its physiology: how it moves, breathes, and reasons.

To the engineer, "logic" often feels like a fuzzy philosophical subject. In Kleis, logic is stripped of its pretension. It is treated as a **String Rewrite System**. There is no "meaning," only patterns and substitutions.

---

## 2.1 The Concept of a Formal System

Kleis is a **Formal System**. This means it is a game played with four components:

1. **Symbols**: The alphabet (`∀`, `∃`, `→`, `=`, keywords).
2. **Syntax**: The grammar that determines if a string is "well-formed."
3. **Axioms**: The starting strings we accept as given.
4. **Inference Rules**: The rules for transforming one string into another.

When Kleis "verifies" your code, it is simply checking if your desired conclusion can be reached from your axioms by following the rules.

---

## 2.2 The 3 Inference Rules (The "Engines")

Almost every proof or verification in Kleis boils down to three mechanical movements. If you understand these three, you understand how Kleis "thinks."

### I. Equality Substitution (=)

**The Rule**: If you have established that `a = b`, then in any string containing `a`, you may replace `a` with `b`.

> **Mindset Shift**: Equality is not "sameness." It is a license to swap.

### II. Arrow Application (→)

**The Rule**: If you have an arrow `f : A → B` and an object `x` of shape `A`, you may produce the string `f(x)` which has shape `B`.

> **Mindset Shift**: This is the only way to "move" through a structure. You cannot jump; you must follow the arrows.

### III. Universal Elimination (∀)

**The Rule**: If a law is stated `∀(x : G). P(x)`, and you have a specific witness `a` that is a `G`, you may write `P(a)`.

> **Mindset Shift**: This is how we move from the general "Pattern" to the specific "Instance."

---

## 2.3 The 14 Definitions (The "Truth Tables")

While the inference rules tell us *how to move*, the Truth Table Axioms define *what the logical symbols actually are*.

In Kleis, the symbol `∧` (AND) isn't a "concept." It is defined by four rewrite rules. These rules are "stipulated"—we agree to play by them because they produce useful results.

| Operator | The Rewrite Rules (The Stipulations) |
| :--- | :--- |
| `∧` (And) | `T∧T=T`; `T∧F=F`; `F∧T=F`; `F∧F=F` |
| `∨` (Or) | `T∨T=T`; `T∨F=T`; `F∨T=T`; `F∨F=F` |
| `¬` (Not) | `¬T=F`; `¬F=T` |
| `⟹` (Implies) | `T⟹T=T`; `T⟹F=F`; `F⟹T=T`; `F⟹F=T` |

**Why 14?**

- 4 rules for `∧`
- 4 rules for `∨`
- 2 rules for `¬`
- 4 rules for `⟹`

**Total: 14 definitions.**

Combined with our 3 inference rules, we have exactly **17 stipulations** that form the bedrock of all mathematical verification.

---

## 2.4 Case Study: The "Implies" Trap

Engineers often struggle with the `false ⟹ true = true` rule. In a programming `if` statement, this feels nonsensical. In Kleis, it is vital.

```kleis
axiom approval_required: amount > 1000 ⟹ approved = true
```

If the `amount` is 500, the left side is `false`. According to the rewrite rules, the entire axiom becomes `true` regardless of whether `approved` is `true` or `false`. 

**The law is not violated by things it does not govern.**

---

## 2.5 Logic as a Compiler

Think of Kleis as a compiler that "lowers" high-level Structures into these 17 raw rules.

When you write a complex structure, Kleis expands it into a massive string of `∀`, `∃`, and `∧`. It then uses the 17 rules to see if that string can be simplified to `true` (consistent) or if it leads to a contradiction (invalid).

---

## Summary of Chapter 2

1. **Logic is Rewriting**: To verify is to substitute symbols according to rules.
2. **The 3 Engines**: Substitution, Application, and Elimination move the gears.
3. **The 14 Definitions**: Truth tables are just "find-and-replace" rules for operators.
4. **Complexity is an Illusion**: Every complex verification is just these 17 rules applied thousands of times.
