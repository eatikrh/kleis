#!/usr/bin/env python3
"""
Toy Learning Brain — A self-growing, multi-skill neural network.

Starts with 0 hidden neurons and a freelist of 50 available neurons.
Learns multiple skills (e.g. multiplication, addition) by dynamically
recruiting separate subnetworks for each skill, then freezing them.

Key properties:
  - NO catastrophic forgetting: frozen skills survive new learning
  - Skill gating: only the active skill's neurons fire during inference
  - Demand-driven neurogenesis: recruit neurons when stuck
  - Layered topology: L1 (feature extraction) → L2 (combination) → output
  - Sparse final topology (lottery ticket)

Architecture per skill:
  input_a ──→ [L1 neurons] ──→ [L2 neurons] ──→ output
  input_b ──↗                ↗
  (shared)        (skill-owned)         (shared)

Usage:
    python examples/mathematics/toy_learning_brain.py
"""

import math
import random
from dataclasses import dataclass

# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

@dataclass
class Neuron:
    id: int
    bias: float = 0.0
    activation: float = 0.0
    _z: float = 0.0
    delta: float = 0.0
    layer: int = 0            # 0=input, -1=output, 1,2,...=hidden
    skill: str | None = None  # which skill owns this neuron
    frozen: bool = False
    m_bias: float = 0.0
    v_bias: float = 0.0


@dataclass
class Connection:
    source: int
    target: int
    weight: float
    usage_count: int = 0
    grad: float = 0.0
    skill: str | None = None
    frozen: bool = False
    m: float = 0.0
    v: float = 0.0


# ---------------------------------------------------------------------------
# Brain
# ---------------------------------------------------------------------------

class Brain:

    def __init__(self, freelist_size: int = 50):
        self._next_id = 0

        self.input_a = self._make_neuron(layer=0)
        self.input_b = self._make_neuron(layer=0)
        self.output = self._make_neuron(layer=-1)
        self.output.frozen = True  # shared output bias is never updated

        self.active: dict[int, Neuron] = {
            self.input_a.id: self.input_a,
            self.input_b.id: self.input_b,
            self.output.id: self.output,
        }

        self.freelist: list[Neuron] = [
            self._make_neuron() for _ in range(freelist_size)
        ]

        self.connections: list[Connection] = []
        self.hidden_ids: set[int] = set()
        self.input_ids: set[int] = {self.input_a.id, self.input_b.id}

        self.active_skill: str | None = None
        self.frozen_skills: set[str] = set()
        self.adam_t = 0
        self.total_recruited = 0
        self.total_returned = 0

    def _make_neuron(self, layer: int = 0) -> Neuron:
        n = Neuron(id=self._next_id, layer=layer,
                   bias=random.gauss(0, 0.1))
        self._next_id += 1
        return n

    def _connect(self, src: int, tgt: int) -> Connection:
        fan_in = sum(1 for c in self.connections if c.target == tgt) + 1
        std = math.sqrt(2.0 / fan_in)
        c = Connection(source=src, target=tgt,
                       weight=random.gauss(0, std),
                       skill=self.active_skill)
        self.connections.append(c)
        return c

    def _has_connection(self, src: int, tgt: int) -> bool:
        return any(c.source == src and c.target == tgt for c in self.connections)

    # ------------------------------------------------------------------
    # Topology
    # ------------------------------------------------------------------

    def _topo_order(self) -> list[int]:
        in_deg = {nid: 0 for nid in self.active}
        adj: dict[int, list[int]] = {nid: [] for nid in self.active}
        for c in self.connections:
            if c.source in self.active and c.target in self.active:
                adj[c.source].append(c.target)
                in_deg[c.target] += 1
        queue = [nid for nid, d in in_deg.items() if d == 0]
        order: list[int] = []
        while queue:
            nid = queue.pop(0)
            order.append(nid)
            for nb in adj[nid]:
                in_deg[nb] -= 1
                if in_deg[nb] == 0:
                    queue.append(nb)
        for nid in self.active:
            if nid not in order:
                order.append(nid)
        return order

    def _neurons_at_layer(self, layer: int, skill: str | None = None) -> list[int]:
        return [nid for nid, n in self.active.items()
                if n.layer == layer and (skill is None or n.skill == skill)]

    def _skill_max_layer(self, skill: str) -> int:
        layers = [n.layer for n in self.active.values()
                  if n.layer > 0 and n.skill == skill]
        return max(layers) if layers else 0

    # ------------------------------------------------------------------
    # Activation
    # ------------------------------------------------------------------

    @staticmethod
    def _relu(z: float) -> float:
        return z if z > 0 else 0.01 * z

    @staticmethod
    def _relu_deriv(z: float) -> float:
        return 1.0 if z > 0 else 0.01

    # ------------------------------------------------------------------
    # Forward — with skill gating
    # ------------------------------------------------------------------

    def forward(self, a: float, b: float) -> float:
        self.input_a.activation = a
        self.input_b.activation = b
        for n in self.active.values():
            n.delta = 0.0

        order = self._topo_order()
        for nid in order:
            if nid in self.input_ids:
                continue
            neuron = self.active[nid]

            # GATING: deactivate neurons from other skills
            if neuron.skill and neuron.skill != self.active_skill:
                neuron.activation = 0.0
                neuron._z = 0.0
                continue

            z = neuron.bias
            for c in self.connections:
                if c.target == nid:
                    src = self.active.get(c.source)
                    if src is not None:
                        z += c.weight * src.activation
                        c.usage_count += 1
            neuron._z = z
            neuron.activation = z if neuron.layer == -1 else self._relu(z)

        return self.output.activation

    # ------------------------------------------------------------------
    # Backward — two-pass, respects gating
    # ------------------------------------------------------------------

    def backward(self, target: float) -> float:
        error = self.output.activation - target
        loss = 0.5 * error * error
        self.output.delta = error

        order = self._topo_order()
        order.reverse()

        # Pass 1: deltas
        for nid in order:
            if nid in self.input_ids or nid == self.output.id:
                continue
            neuron = self.active[nid]
            if neuron.skill and neuron.skill != self.active_skill:
                neuron.delta = 0.0
                continue
            raw = 0.0
            for c in self.connections:
                if c.source == nid:
                    tgt = self.active.get(c.target)
                    if tgt is not None:
                        raw += tgt.delta * c.weight
            neuron.delta = raw * self._relu_deriv(neuron._z)

        # Pass 2: connection gradients
        for c in self.connections:
            src = self.active.get(c.source)
            tgt = self.active.get(c.target)
            if src is not None and tgt is not None:
                c.grad = tgt.delta * src.activation
            else:
                c.grad = 0.0

        return loss

    # ------------------------------------------------------------------
    # Adam — respects frozen flags
    # ------------------------------------------------------------------

    def adam_step(self, lr: float = 0.01, beta1: float = 0.9,
                  beta2: float = 0.999, eps: float = 1e-8):
        self.adam_t += 1
        bc1 = 1 - beta1 ** self.adam_t
        bc2 = 1 - beta2 ** self.adam_t

        for c in self.connections:
            if c.frozen:
                continue
            g = c.grad
            c.m = beta1 * c.m + (1 - beta1) * g
            c.v = beta2 * c.v + (1 - beta2) * g * g
            c.weight -= lr * (c.m / bc1) / (math.sqrt(c.v / bc2) + eps)

        for n in self.active.values():
            if n.frozen or n.layer == 0:
                continue
            g = n.delta
            n.m_bias = beta1 * n.m_bias + (1 - beta1) * g
            n.v_bias = beta2 * n.v_bias + (1 - beta2) * g * g
            n.bias -= lr * (n.m_bias / bc1) / (math.sqrt(n.v_bias / bc2) + eps)

    # ------------------------------------------------------------------
    # Hebbian
    # ------------------------------------------------------------------

    def hebbian_strengthen(self, strength: float = 0.001):
        for c in self.connections:
            if c.frozen:
                continue
            src = self.active.get(c.source)
            tgt = self.active.get(c.target)
            if src and tgt:
                coact = abs(src.activation * tgt.activation)
                if coact > 0.05:
                    c.weight += strength * math.copysign(min(coact, 0.5), c.weight)

    # ------------------------------------------------------------------
    # Neurogenesis — skill-tagged
    # ------------------------------------------------------------------

    def recruit_neuron(self):
        if not self.freelist:
            return
        skill = self.active_skill

        new = self.freelist.pop()
        new.bias = random.gauss(0, 0.3)
        new.skill = skill
        new.frozen = False
        new.m_bias = 0.0
        new.v_bias = 0.0

        l1 = self._neurons_at_layer(1, skill=skill)
        l2 = self._neurons_at_layer(2, skill=skill)

        add_l2 = len(l1) >= 3 and len(l2) < len(l1) and random.random() < 0.5

        if add_l2:
            new.layer = 2
            self.active[new.id] = new
            self.hidden_ids.add(new.id)
            k = min(len(l1), random.choice([2, 3]))
            for sid in random.sample(l1, k):
                self._connect(sid, new.id)
            self._connect(new.id, self.output.id)
        else:
            new.layer = 1
            self.active[new.id] = new
            self.hidden_ids.add(new.id)
            self._connect(self.input_a.id, new.id)
            self._connect(self.input_b.id, new.id)
            self._connect(new.id, self.output.id)
            if l2:
                for tid in random.sample(l2, min(len(l2), random.choice([1, 2]))):
                    self._connect(new.id, tid)

        self.total_recruited += 1

    # ------------------------------------------------------------------
    # Pruning — only prune active skill's connections
    # ------------------------------------------------------------------

    def prune(self, weight_threshold: float = 0.01, min_usage: int = 5):
        skill = self.active_skill
        surviving = []
        for c in self.connections:
            if c.skill != skill:
                surviving.append(c)  # don't touch other skills
            elif abs(c.weight) > weight_threshold or c.usage_count >= min_usage:
                surviving.append(c)
        self.connections = surviving

        connected: set[int] = set()
        for c in self.connections:
            connected.add(c.source)
            connected.add(c.target)

        for hid in list(self.hidden_ids):
            n = self.active.get(hid)
            if n and n.skill == skill and hid not in connected:
                self.active.pop(hid, None)
                self.hidden_ids.discard(hid)
                self.freelist.append(n)
                self.total_returned += 1

    def reset_usage_counts(self, skill: str | None = None):
        for c in self.connections:
            if skill is None or c.skill == skill:
                c.usage_count = 0

    # ------------------------------------------------------------------
    # Freeze / unfreeze
    # ------------------------------------------------------------------

    def freeze_skill(self, name: str):
        for n in self.active.values():
            if n.skill == name:
                n.frozen = True
        for c in self.connections:
            if c.skill == name:
                c.frozen = True
        self.frozen_skills.add(name)

    # ------------------------------------------------------------------
    # Evaluate
    # ------------------------------------------------------------------

    def evaluate_skill(self, examples, in_scale, out_scale, tol):
        correct = 0
        for a, b, target in examples:
            pred = self.forward(a / in_scale, b / in_scale) * out_scale
            if abs(pred - target) < tol:
                correct += 1
        return correct

    # ------------------------------------------------------------------
    # Learn a skill
    # ------------------------------------------------------------------

    def learn_skill(self, name: str, examples: list[tuple],
                    in_scale: float, out_scale: float,
                    max_epochs: int = 3000, lr: float = 0.01,
                    tol: float = 0.5, growth_patience: int = 30,
                    prune_interval: int = 500, verbose: bool = True):

        self.active_skill = name
        self.recruit_neuron()  # start with 1 hidden neuron

        best_correct = 0
        epochs_no_improve = 0
        n_examples = len(examples)

        for epoch in range(1, max_epochs + 1):
            random.shuffle(examples)
            epoch_loss = 0.0

            for a, b, target in examples:
                self.forward(a / in_scale, b / in_scale)
                loss = self.backward(target / out_scale)
                self.adam_step(lr=lr)
                epoch_loss += loss

            correct = self.evaluate_skill(examples, in_scale, out_scale, tol)

            if correct > 0:
                for a, b, target in examples:
                    pred = self.forward(a / in_scale, b / in_scale) * out_scale
                    if abs(pred - target) < tol:
                        self.hebbian_strengthen(strength=0.0005)

            if correct > best_correct:
                best_correct = correct
                epochs_no_improve = 0
            else:
                epochs_no_improve += 1

            if epochs_no_improve >= growth_patience and self.freelist:
                self.recruit_neuron()
                epochs_no_improve = 0

            if epoch % prune_interval == 0:
                self.prune(weight_threshold=0.005, min_usage=3)
                self.reset_usage_counts(skill=name)

            avg_loss = epoch_loss / n_examples

            if verbose:
                show = (epoch <= 100 and epoch % 20 == 0) or \
                       (100 < epoch <= 500 and epoch % 50 == 0) or \
                       (epoch > 500 and epoch % 100 == 0) or \
                       correct == n_examples
                if show:
                    ls = self._skill_layer_summary(name)
                    nc = sum(1 for c in self.connections if c.skill == name)
                    print(
                        f"    E{epoch:4d}: {correct:2d}/{n_examples} | "
                        f"loss={avg_loss:.6f} | {ls} | "
                        f"conn={nc} | free={len(self.freelist)}"
                    )

            if correct == n_examples:
                if verbose:
                    print(f"\n    LEARNED all {n_examples} facts at epoch {epoch}!")
                break

        self.freeze_skill(name)
        return best_correct

    def _skill_layer_summary(self, skill: str) -> str:
        counts: dict[int, int] = {}
        for n in self.active.values():
            if n.skill == skill and n.layer > 0:
                counts[n.layer] = counts.get(n.layer, 0) + 1
        return ", ".join(f"L{k}={v}" for k, v in sorted(counts.items())) or "empty"

    # ------------------------------------------------------------------
    # Metrics
    # ------------------------------------------------------------------

    def skill_neurons(self, skill: str) -> list[int]:
        return sorted(nid for nid, n in self.active.items() if n.skill == skill)

    def skill_connections(self, skill: str) -> list[Connection]:
        return [c for c in self.connections if c.skill == skill]


# ---------------------------------------------------------------------------
# Demo
# ---------------------------------------------------------------------------

def print_test_table(brain, skill_name, examples, in_scale, out_scale, tol):
    """Print predicted vs actual for a skill."""
    brain.active_skill = skill_name
    total_ok = 0
    for a, b, target in sorted(examples):
        pred = brain.forward(a / in_scale, b / in_scale) * out_scale
        err = pred - target
        ok = abs(err) < tol
        total_ok += int(ok)
        sym = "×" if skill_name == "multiplication" else "+"
        print(f"    {a} {sym} {b} = {target:5.1f}  pred={pred:6.2f}  "
              f"err={err:+5.2f}  {'OK' if ok else 'MISS'}")
    print(f"    Score: {total_ok}/{len(examples)}")
    return total_ok


def print_topology(brain, skill_name):
    """Print a skill's subnetwork."""
    neurons = brain.skill_neurons(skill_name)
    conns = brain.skill_connections(skill_name)

    l1 = [nid for nid in neurons if brain.active[nid].layer == 1]
    l2 = [nid for nid in neurons if brain.active[nid].layer == 2]

    print(f"    Neurons: {len(neurons)} (L1={len(l1)}, L2={len(l2)})")
    print(f"    Connections: {len(conns)}")
    if neurons:
        n_possible = (len(neurons) + 3) * (len(neurons) + 2)  # with shared neurons
        density = len(conns) / n_possible if n_possible > 0 else 0
        print(f"    Density: {density:.3f}")

    def lbl(nid):
        if nid == brain.input_a.id: return "a"
        if nid == brain.input_b.id: return "b"
        if nid == brain.output.id:  return "out"
        n = brain.active.get(nid)
        return f"L{n.layer}:{nid}" if n else f"?{nid}"

    print(f"    Wiring:")
    for c in sorted(conns, key=lambda x: (
            brain.active.get(x.source, Neuron(0)).layer, x.source, x.target)):
        print(f"      {lbl(c.source):>6} → {lbl(c.target):<8} w={c.weight:+.4f}")


def main():
    random.seed(42)
    brain = Brain(freelist_size=50)

    IN_SCALE = 4.0
    TOL = 0.5

    mult_examples = [(a, b, a * b) for a in range(5) for b in range(5)]
    add_examples = [(a, b, a + b) for a in range(5) for b in range(5)]

    MULT_SCALE = 16.0  # max(a*b) = 16
    ADD_SCALE = 8.0     # max(a+b) = 8

    # ==================================================================
    # SKILL 1: MULTIPLICATION
    # ==================================================================

    print("=" * 70)
    print("  SKILL 1: MULTIPLICATION (a × b)")
    print("=" * 70)
    print(f"  25 facts: 0×0 through 4×4, targets 0..16")
    print(f"  Freelist: {len(brain.freelist)} neurons")
    print()

    brain.learn_skill("multiplication", mult_examples,
                      in_scale=IN_SCALE, out_scale=MULT_SCALE,
                      max_epochs=3000, lr=0.01, tol=TOL)

    print()
    print("  Test after learning:")
    mult_score_1 = print_test_table(
        brain, "multiplication", mult_examples, IN_SCALE, MULT_SCALE, TOL)

    # ==================================================================
    # SKILL 2: ADDITION
    # ==================================================================

    print()
    print("=" * 70)
    print("  SKILL 2: ADDITION (a + b)")
    print("=" * 70)
    print(f"  25 facts: 0+0 through 4+4, targets 0..8")
    print(f"  Freelist: {len(brain.freelist)} neurons")
    print(f"  Multiplication is FROZEN ({len(brain.skill_neurons('multiplication'))} neurons)")
    print()

    brain.learn_skill("addition", add_examples,
                      in_scale=IN_SCALE, out_scale=ADD_SCALE,
                      max_epochs=3000, lr=0.01, tol=TOL)

    print()
    print("  Test after learning:")
    add_score = print_test_table(
        brain, "addition", add_examples, IN_SCALE, ADD_SCALE, TOL)

    # ==================================================================
    # RETENTION TEST — does multiplication survive?
    # ==================================================================

    print()
    print("=" * 70)
    print("  RETENTION TEST — Is multiplication still intact?")
    print("=" * 70)
    print()

    mult_score_2 = print_test_table(
        brain, "multiplication", mult_examples, IN_SCALE, MULT_SCALE, TOL)

    retained = mult_score_2 >= mult_score_1
    if retained:
        print(f"\n    NO CATASTROPHIC FORGETTING! "
              f"({mult_score_1}/25 before → {mult_score_2}/25 after)")
    else:
        print(f"\n    Some forgetting occurred: "
              f"{mult_score_1}/25 → {mult_score_2}/25")

    # ==================================================================
    # TOPOLOGY
    # ==================================================================

    print()
    print("=" * 70)
    print("  NETWORK TOPOLOGY")
    print("=" * 70)

    print()
    print("  Shared neurons: input_a(0), input_b(1), output(2)")
    print(f"  Total active: {len(brain.active)} neurons, "
          f"{len(brain.connections)} connections")
    print(f"  Freelist: {len(brain.freelist)} neurons remaining")
    print(f"  Recruited: {brain.total_recruited}, Returned: {brain.total_returned}")

    for skill_name in ["multiplication", "addition"]:
        print()
        print(f"  --- {skill_name.upper()} subnetwork ---")
        print_topology(brain, skill_name)

    # ==================================================================
    # SUMMARY
    # ==================================================================

    print()
    print("=" * 70)
    print("  SUMMARY")
    print("=" * 70)
    mult_n = len(brain.skill_neurons("multiplication"))
    add_n = len(brain.skill_neurons("addition"))
    print(f"  Multiplication: {mult_score_2}/25 correct, "
          f"{mult_n} neurons, "
          f"{len(brain.skill_connections('multiplication'))} connections")
    print(f"  Addition:       {add_score}/25 correct, "
          f"{add_n} neurons, "
          f"{len(brain.skill_connections('addition'))} connections")
    print(f"  Total hidden:   {mult_n + add_n} neurons "
          f"(freelist: {len(brain.freelist)})")
    print(f"  Catastrophic forgetting: {'NO' if retained else 'YES'}")
    print()


if __name__ == "__main__":
    main()
