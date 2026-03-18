# Chapter 1: Names and Arrows

In this chapter, we leave behind the world of "things" and enter the world of "relations." To use Kleis, you must accept a radical premise: **an object has no internal meaning.** Its entire existence is defined by how it relates to other names through arrows.

---

## 1.1 The Atom of Kleis: The Name

In a standard programming language, if you write `int x`, you are invoking a massive amount of "hidden" knowledge: 32 or 64 bits, two's complement arithmetic, and memory addresses.

In Kleis, we start with a **Name**. A name is just a label. It is a point on a whiteboard with no thickness.

```kleis
G
```

By writing `G`, we have stipulated that there is a "shape" or a "sort" called $G$. We don't know what is "inside" $G$ because there is no "inside."

---

## 1.2 The Relation: The Arrow

If $G$ is a point, an **Arrow** is a directed connection between points. In Kleis, we call these **operations** or **elements**.

- **Operation**: A mapping from one or more names to another.

```kleis
operation f : G → G
```

- **Element**: A specific "pointer" into a name.

```kleis
element e : G
```

> **Note**: An element is actually just a special kind of arrow that starts from "nothing" (the `Unit`) and points into a name.

---

## 1.3 Building a Pattern: The Structure

A **Structure** is a collection of names and arrows. It doesn't "do" anything yet; it simply describes a requirement. Think of it as a blueprint for a machine that hasn't been built.

Let's define the simplest meaningful structure: a **Magma** (a set with a single binary operation).

```kleis
structure Magma(M) {
    operation binary_op : M × M → M
}
```

**What did we just do?**

1. We declared a parameter `M` (a Name).
2. We declared an operation `binary_op` (an Arrow).
3. We stated that if you give this operation two things of shape `M`, it will point you to a third thing of shape `M`.

---

## 1.4 The Commutative Diagram

How do we reason about these arrows? We use **Diagrams**. If there are two different paths of arrows that lead to the same destination, and we want them to be "the same," we create an **Axiom**.

Consider the **Semigroup**, which is just a Magma that follows the Associative Law:

```kleis
structure Semigroup(S) extends Magma(S) {
    axiom associativity: ∀(a b c : S). 
        binary_op(binary_op(a, b), c) = binary_op(a, binary_op(b, c))
}
```

This axiom is a **Rewrite Rule**. It tells the Kleis engine: "Whenever you see the pattern on the left, you are permitted to replace it with the pattern on the right."

---

## 1.5 The Shift: From "Is" to "Witnesses"

This is where the engineering mindset usually breaks. In Java, you might say `Integer implements Semigroup`. In Kleis, we say:

> "The type $\mathbb{Z}$ **witnesses** the Semigroup structure under the operation of addition."

```kleis
implements Semigroup(ℤ) {
    operation binary_op = builtin_add
}
```

Notice that $\mathbb{Z}$ (integers) is not "a Semigroup" by birthright. It only becomes a witness when we provide the specific arrow (`builtin_add`) that fits the pattern. 

We could just as easily make $\mathbb{Z}$ witness a Semigroup using `builtin_mul`.

**The structure is the master; the data is the servant.**

---

## Summary of Chapter 1

1. **Objects** are opaque points (Names).
2. **Morphisms** are connections (Arrows).
3. **Structures** are patterns of names and arrows.
4. **Axioms** are rules for rewriting paths of arrows.
5. **Implementation** is the act of providing a concrete witness that fits the pattern.
