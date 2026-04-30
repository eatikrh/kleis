#!/usr/bin/env python3
"""
Growing Transformer Brain — A self-growing language model.

A character-level transformer that starts tiny (1 attention head, 32 FFN
neurons) and grows on demand by recruiting from a replenishable freelist.
Trains on .kleis files to learn Kleis syntax.

Key properties:
  - Starts minimal, grows as needed (no pre-decided size)
  - Replenishable freelist: when capacity runs out, expand
  - Attention heads recruited when loss plateaus
  - FFN neurons recruited when more capacity needed
  - Pruning: dead FFN neurons and unhelpful attention heads returned to freelist
  - Training and inference are the same operation — always learning

Architecture:
  tokens → embedding + pos_enc → [attention heads (avg)] + residual
         → [FFN (growable)] + residual → output logits

Usage:
    python examples/mathematics/growing_transformer_brain.py
"""

import numpy as np
import json
import glob
import os
import re
import subprocess
import sys
import tempfile

# ---------------------------------------------------------------------------
# Kleis validation oracle — rich feedback
# ---------------------------------------------------------------------------

_ERR_POS_RE = re.compile(r"Line\s+(\d+),\s+column\s+(\d+)")

class KleisFeedback:
    """Structured feedback from kleis check — not just pass/fail."""
    __slots__ = ("valid", "error_line", "error_col", "error_char_offset",
                 "valid_prefix_ratio", "error_kind", "expected", "message")

    def __init__(self, valid, text, message,
                 error_line=0, error_col=0, expected=""):
        self.valid = valid
        self.error_line = error_line
        self.error_col = error_col
        self.message = message
        self.expected = expected

        if valid:
            self.valid_prefix_ratio = 1.0
            self.error_char_offset = len(text)
            self.error_kind = "none"
        else:
            offset = _char_offset(text, error_line, error_col)
            self.error_char_offset = offset
            self.valid_prefix_ratio = offset / max(len(text), 1)
            if "parse error" in message:
                self.error_kind = "parse"
            elif "type" in message.lower():
                self.error_kind = "type"
            else:
                self.error_kind = "unknown"

    @property
    def reward(self):
        """Continuous reward in [0, 1]: 1 = fully valid, 0 = instant fail."""
        if self.valid:
            return 1.0
        base = self.valid_prefix_ratio
        if self.error_kind == "type":
            base = max(base, 0.7)
        return base

    def short(self, max_len=80):
        if self.valid:
            return "VALID"
        s = self.message.replace("\n", " ")[:max_len]
        return f"{self.error_kind}@L{self.error_line}C{self.error_col} " \
               f"({self.valid_prefix_ratio:.0%} correct): {s}"


def _char_offset(text, line, col):
    """Convert (1-based line, 1-based col) to a character offset."""
    if line <= 0:
        return 0
    current_line = 1
    pos = 0
    for i, ch in enumerate(text):
        if current_line == line:
            return min(pos + max(col - 1, 0), len(text))
        if ch == '\n':
            current_line += 1
            pos = i + 1
    return len(text)


def validate_kleis(text):
    """Check text with `kleis check`. Returns KleisFeedback with rich info."""
    try:
        with tempfile.NamedTemporaryFile(
                mode="w", suffix=".kleis", delete=True) as f:
            f.write(text)
            f.flush()
            result = subprocess.run(
                ["kleis", "check", f.name],
                capture_output=True, text=True, timeout=5)
            combined = (result.stdout + "\n" + result.stderr).strip()
            if result.returncode == 0:
                return KleisFeedback(True, text, combined)
            m = _ERR_POS_RE.search(combined)
            line = int(m.group(1)) if m else 0
            col = int(m.group(2)) if m else 0
            exp_m = re.search(r"Expected (.+?)$", combined, re.MULTILINE)
            expected = exp_m.group(1) if exp_m else ""
            return KleisFeedback(False, text, combined, line, col, expected)
    except (FileNotFoundError, subprocess.TimeoutExpired) as e:
        return KleisFeedback(False, text, str(e))


# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------

def softmax(x, axis=-1):
    x_max = x.max(axis=axis, keepdims=True)
    e = np.exp(x - x_max)
    return e / e.sum(axis=axis, keepdims=True)


# ---------------------------------------------------------------------------
# Adam-wrapped parameter
# ---------------------------------------------------------------------------

class Param:
    """Numpy array with Adam optimizer state."""

    def __init__(self, data):
        self.data = data.astype(np.float32)
        self.m = np.zeros_like(self.data)
        self.v = np.zeros_like(self.data)
        self.grad = np.zeros_like(self.data)

    def zero_grad(self):
        self.grad.fill(0)

    def step(self, lr, t, beta1=0.9, beta2=0.999, eps=1e-8):
        self.m = beta1 * self.m + (1 - beta1) * self.grad
        self.v = beta2 * self.v + (1 - beta2) * self.grad ** 2
        m_hat = self.m / (1 - beta1 ** t)
        v_hat = self.v / (1 - beta2 ** t)
        self.data -= lr * m_hat / (np.sqrt(v_hat) + eps)


# ---------------------------------------------------------------------------
# Attention Head
# ---------------------------------------------------------------------------

class AttentionHead:
    """Single causal self-attention head."""

    def __init__(self, d_model, scale=1.0):
        s = np.sqrt(2.0 / d_model) * scale
        self.W_Q = Param(np.random.randn(d_model, d_model).astype(np.float32) * s)
        self.W_K = Param(np.random.randn(d_model, d_model).astype(np.float32) * s)
        self.W_V = Param(np.random.randn(d_model, d_model).astype(np.float32) * s)
        self.frozen = False
        self._cache = {}
        self.avg_output_norm = 1.0
        self._ema_decay = 0.95

    def forward(self, X):
        Q = X @ self.W_Q.data
        K = X @ self.W_K.data
        V = X @ self.W_V.data
        D = Q.shape[1]

        scores = Q @ K.T / np.sqrt(np.float32(D))
        S = Q.shape[0]
        mask = np.triu(np.full((S, S), -1e9, dtype=np.float32), k=1)
        scores += mask
        A = softmax(scores)
        out = A @ V

        # Track output magnitude for pruning
        norm = float(np.linalg.norm(out) / max(S, 1))
        self.avg_output_norm = (self._ema_decay * self.avg_output_norm
                                + (1 - self._ema_decay) * norm)

        self._cache = {"X": X, "Q": Q, "K": K, "V": V, "A": A}
        return out

    def backward(self, d_out):
        if self.frozen:
            return np.zeros_like(self._cache["X"])

        X, Q, K, V, A = (self._cache[k] for k in ("X", "Q", "K", "V", "A"))
        D = np.float32(Q.shape[1])

        d_V = A.T @ d_out
        d_A = d_out @ V.T
        d_scores = A * (d_A - (d_A * A).sum(axis=-1, keepdims=True))
        d_scores /= np.sqrt(D)

        d_Q = d_scores @ K
        d_K = d_scores.T @ Q

        self.W_Q.grad += X.T @ d_Q
        self.W_K.grad += X.T @ d_K
        self.W_V.grad += X.T @ d_V

        return d_Q @ self.W_Q.data.T + d_K @ self.W_K.data.T + d_V @ self.W_V.data.T

    def params(self):
        return [self.W_Q, self.W_K, self.W_V]

    def zero_grad(self):
        for p in self.params():
            p.zero_grad()


# ---------------------------------------------------------------------------
# Transformer Layer — one attention block + one FFN block
# ---------------------------------------------------------------------------

class TransformerLayer:
    """One transformer layer: multi-head attention + FFN, both with residuals."""

    def __init__(self, d_model, initial_heads=1, initial_ffn=32,
                 max_ffn=512, near_zero=False):
        self.d_model = d_model
        self.max_ffn = max_ffn
        self.active_ffn = initial_ffn

        scale = 0.01 if near_zero else 1.0

        self.heads: list[AttentionHead] = [
            AttentionHead(d_model, scale=scale)
            for _ in range(initial_heads)
        ]

        s = np.sqrt(2.0 / d_model) * scale
        self.W1 = Param(np.random.randn(d_model, max_ffn).astype(np.float32) * s)
        self.b1 = Param(np.zeros(max_ffn, dtype=np.float32))
        self.W2 = Param(
            np.random.randn(max_ffn, d_model).astype(np.float32) * s * 0.1)
        self.b2 = Param(np.zeros(d_model, dtype=np.float32))
        self.W1.data[:, initial_ffn:] = 0
        self.W2.data[initial_ffn:, :] = 0

        self._ffn_activation_ema = np.ones(max_ffn, dtype=np.float32)
        self._cache = {}

        self.total_heads_recruited = 0
        self.total_ffn_recruited = 0
        self.total_ffn_pruned = 0
        self.total_heads_pruned = 0

    def forward(self, x):
        n_heads = len(self.heads)
        head_outs = [h.forward(x) for h in self.heads]
        attn_out = sum(head_outs) / n_heads
        x1 = x + attn_out

        n = self.active_ffn
        ffn_z = x1 @ self.W1.data[:, :n] + self.b1.data[:n]
        ffn_h = np.maximum(ffn_z, 0.01 * ffn_z)
        ffn_out = ffn_h @ self.W2.data[:n, :] + self.b2.data

        act_mag = np.abs(ffn_h).mean(axis=0)
        decay = 0.95
        self._ffn_activation_ema[:n] = (
            decay * self._ffn_activation_ema[:n] + (1 - decay) * act_mag)

        x2 = x1 + ffn_out

        self._cache = {
            "x_in": x, "x1": x1,
            "ffn_z": ffn_z, "ffn_h": ffn_h,
            "n_heads": n_heads,
        }
        return x2

    def backward(self, d_out):
        c = self._cache
        n = self.active_ffn

        d_ffn_out = d_out
        self.W2.grad[:n, :] += c["ffn_h"].T @ d_ffn_out
        self.b2.grad += d_ffn_out.sum(axis=0)
        d_ffn_h = d_ffn_out @ self.W2.data[:n, :].T
        d_ffn_z = d_ffn_h * np.where(c["ffn_z"] > 0, 1.0, 0.01)
        self.W1.grad[:, :n] += c["x1"].T @ d_ffn_z
        self.b1.grad[:n] += d_ffn_z.sum(axis=0)
        d_x1 = d_ffn_z @ self.W1.data[:, :n].T + d_out

        d_attn_per_head = d_x1 / c["n_heads"]
        d_x_in = d_x1.copy()
        for h in self.heads:
            d_x_in += h.backward(d_attn_per_head)
        return d_x_in

    def params(self):
        return [self.W1, self.b1, self.W2, self.b2]

    def zero_grad(self):
        for p in self.params():
            p.zero_grad()
        for h in self.heads:
            h.zero_grad()

    def step(self, lr, adam_t):
        for p in self.params():
            p.step(lr, adam_t)
        for h in self.heads:
            if not h.frozen:
                for p in h.params():
                    p.step(lr, adam_t)

    def clip_gradients(self, max_norm=1.0):
        for p in self.params():
            np.clip(p.grad, -max_norm, max_norm, out=p.grad)
        for h in self.heads:
            for p in h.params():
                np.clip(p.grad, -max_norm, max_norm, out=p.grad)

    def recruit_head(self):
        new = AttentionHead(self.d_model)
        self.heads.append(new)
        self.total_heads_recruited += 1
        return len(self.heads)

    def recruit_ffn(self, n_new=16):
        available = self.max_ffn - self.active_ffn
        if available < n_new:
            self._replenish_ffn(max(n_new - available, 64))

        start = self.active_ffn
        end = min(start + n_new, self.max_ffn)
        actual = end - start

        s = np.sqrt(2.0 / self.d_model)
        self.W1.data[:, start:end] = (
            np.random.randn(self.d_model, actual).astype(np.float32) * s)
        self.W2.data[start:end, :] = (
            np.random.randn(actual, self.d_model).astype(np.float32) * s * 0.1)
        self.b1.data[start:end] = 0
        self.W1.m[:, start:end] = 0
        self.W1.v[:, start:end] = 0
        self.W2.m[start:end, :] = 0
        self.W2.v[start:end, :] = 0

        self.active_ffn = end
        self.total_ffn_recruited += actual
        return actual

    def _replenish_ffn(self, n_extra):
        old = self.max_ffn
        new = old + n_extra

        def _grow(param, axis, new_size):
            shape = list(param.data.shape)
            shape[axis] = new_size
            for attr in ("data", "m", "v", "grad"):
                old_arr = getattr(param, attr)
                new_arr = np.zeros(shape, dtype=np.float32)
                slices = [slice(None)] * len(shape)
                slices[axis] = slice(0, old_arr.shape[axis])
                new_arr[tuple(slices)] = old_arr
                setattr(param, attr, new_arr)

        _grow(self.W1, 1, new)
        _grow(self.b1, 0, new)
        _grow(self.W2, 0, new)

        new_ema = np.ones(new, dtype=np.float32)
        new_ema[:old] = self._ffn_activation_ema
        self._ffn_activation_ema = new_ema

        self.max_ffn = new

    def prune_ffn(self, threshold=0.01, bottom_pct=0.02, ceiling=0.05):
        n = self.active_ffn
        if n <= 8:
            return 0
        active_ema = self._ffn_activation_ema[:n]
        adaptive_thresh = min(
            max(threshold, np.percentile(active_ema, bottom_pct * 100)),
            ceiling)
        pruned = 0
        i = 0
        while i < n:
            if self._ffn_activation_ema[i] < adaptive_thresh:
                last = n - 1
                if i < last:
                    self.W1.data[:, i] = self.W1.data[:, last]
                    self.W2.data[i, :] = self.W2.data[last, :]
                    self.b1.data[i] = self.b1.data[last]
                    self._ffn_activation_ema[i] = self._ffn_activation_ema[last]
                    for attr in ("m", "v"):
                        getattr(self.W1, attr)[:, i] = getattr(self.W1, attr)[:, last]
                        getattr(self.W2, attr)[i, :] = getattr(self.W2, attr)[last, :]
                        getattr(self.b1, attr)[i] = getattr(self.b1, attr)[last]
                self.W1.data[:, last] = 0
                self.W2.data[last, :] = 0
                n -= 1
                pruned += 1
            else:
                i += 1
        self.active_ffn = n
        self.total_ffn_pruned += pruned
        return pruned

    def prune_heads(self, min_heads=1):
        if len(self.heads) <= min_heads:
            return 0
        max_norm = max(h.avg_output_norm for h in self.heads)
        if max_norm < 1e-8:
            return 0
        cutoff = max_norm * 0.10
        ranked = sorted(self.heads, key=lambda h: h.avg_output_norm, reverse=True)
        surviving = []
        pruned = 0
        for h in ranked:
            if len(surviving) < min_heads or h.avg_output_norm >= cutoff:
                surviving.append(h)
            else:
                pruned += 1
        self.heads = surviving
        self.total_heads_pruned += pruned
        return pruned

    @property
    def ffn_freelist(self):
        return self.max_ffn - self.active_ffn

    @property
    def active_params(self):
        n = self.active_ffn
        total = self.d_model * n + n + n * self.d_model + self.d_model
        for h in self.heads:
            total += sum(p.data.size for p in h.params())
        return total


# ---------------------------------------------------------------------------
# Character tokenizer
# ---------------------------------------------------------------------------

class CharTokenizer:

    def __init__(self, text=None, vocab=None):
        if vocab is not None:
            chars = vocab
        elif text is not None:
            chars = sorted(set(text))
        else:
            chars = []
        self.char_to_id = {c: i for i, c in enumerate(chars)}
        self.id_to_char = {i: c for c, i in self.char_to_id.items()}
        self.vocab_size = len(chars)

    def encode(self, text):
        return [self.char_to_id.get(c, 0) for c in text]

    def decode(self, ids):
        return "".join(self.id_to_char.get(i, "?") for i in ids)

    def save(self, path):
        vocab = [self.id_to_char[i] for i in range(self.vocab_size)]
        with open(path, "w") as f:
            json.dump(vocab, f, ensure_ascii=False)

    @classmethod
    def load(cls, path):
        with open(path, "r") as f:
            vocab = json.load(f)
        return cls(vocab=vocab)


# ---------------------------------------------------------------------------
# Growing Transformer
# ---------------------------------------------------------------------------

class GrowingTransformer:
    """
    A multi-layer transformer that grows from small.

    Each layer has its own attention heads and FFN, both growable.
    New layers can be added at runtime initialized near-zero so the
    residual connection preserves the existing model's behaviour.

    Architecture per forward pass:
      embed → [Layer 0: attn + FFN] → [Layer 1: attn + FFN] → ... → logits
    """

    def __init__(self, vocab_size, d_model=48, context_len=64,
                 initial_heads=1, initial_ffn=32, max_ffn=512):
        self.vocab_size = vocab_size
        self.d_model = d_model
        self.context_len = context_len

        self.embedding = Param(
            np.random.randn(vocab_size, d_model).astype(np.float32) * 0.1)
        self.pos_enc = Param(
            np.random.randn(context_len, d_model).astype(np.float32) * 0.02)

        self.layers: list[TransformerLayer] = [
            TransformerLayer(d_model, initial_heads, initial_ffn, max_ffn)
        ]

        self.W_out = Param(
            np.random.randn(d_model, vocab_size).astype(np.float32) * 0.1)
        self.b_out = Param(np.zeros(vocab_size, dtype=np.float32))

        self.adam_t = 0
        self._cache = {}

        self.history = {
            "loss": [],
            "events": [],
            "snapshots": [],
        }

    # ------------------------------------------------------------------
    # d_model expansion (Net2WiderNet)
    # ------------------------------------------------------------------

    def widen_d_model(self, new_d):
        """Widen the model from current d_model to new_d.

        Old weights occupy the top-left block of every expanded matrix.
        New dimensions get near-zero random init so the model's output
        is approximately unchanged.  Adam state for new dims starts at 0.
        """
        old_d = self.d_model
        if new_d <= old_d:
            return

        def _pad_param(param, expand_axes, old_sizes, new_sizes, scale=0.01):
            """Expand a Param along one or more axes with near-zero init."""
            new_shape = list(param.data.shape)
            for ax, ns in zip(expand_axes, new_sizes):
                new_shape[ax] = ns

            for attr in ("data", "m", "v", "grad"):
                old_arr = getattr(param, attr)
                if attr == "data":
                    new_arr = np.random.randn(*new_shape).astype(np.float32) * scale
                else:
                    new_arr = np.zeros(new_shape, dtype=np.float32)
                slices = [slice(None)] * len(new_shape)
                for ax, os_ in zip(expand_axes, old_sizes):
                    slices[ax] = slice(0, os_)
                new_arr[tuple(slices)] = old_arr
                setattr(param, attr, new_arr)

        # -- Embedding: (vocab, old_d) → (vocab, new_d)
        _pad_param(self.embedding, [1], [old_d], [new_d])

        # -- Positional encoding: (context, old_d) → (context, new_d)
        _pad_param(self.pos_enc, [1], [old_d], [new_d])

        # -- Output projection: W_out (old_d, vocab) → (new_d, vocab)
        _pad_param(self.W_out, [0], [old_d], [new_d])

        # -- b_out stays (vocab,) — no change needed

        # -- Per layer
        for layer in self.layers:
            # W1: (old_d, max_ffn) → (new_d, max_ffn)
            _pad_param(layer.W1, [0], [old_d], [new_d])

            # b1: (max_ffn,) — unchanged

            # W2: (max_ffn, old_d) → (max_ffn, new_d)
            _pad_param(layer.W2, [1], [old_d], [new_d])

            # b2: (old_d,) → (new_d,)
            _pad_param(layer.b2, [0], [old_d], [new_d])

            layer.d_model = new_d

            # Attention heads: W_Q, W_K, W_V each (old_d, old_d) → (new_d, new_d)
            for h in layer.heads:
                for wp in [h.W_Q, h.W_K, h.W_V]:
                    _pad_param(wp, [0, 1], [old_d, old_d], [new_d, new_d])

        self.d_model = new_d

    # ------------------------------------------------------------------
    # Layer management
    # ------------------------------------------------------------------

    def add_layer(self, initial_heads=1, initial_ffn=32, max_ffn=256):
        """Add a new transformer layer initialized near-zero.

        Because of the residual connections, a near-zero layer acts as
        an identity — the model's output is unchanged. The new layer
        then gradually learns to refine the representation.
        """
        layer = TransformerLayer(
            self.d_model, initial_heads, initial_ffn, max_ffn, near_zero=True)
        self.layers.append(layer)
        return len(self.layers)

    # ------------------------------------------------------------------
    # Forward
    # ------------------------------------------------------------------

    def forward(self, tokens):
        S = len(tokens)
        x = self.embedding.data[tokens] + self.pos_enc.data[:S]

        for layer in self.layers:
            x = layer.forward(x)

        logits = x @ self.W_out.data + self.b_out.data
        self._cache = {"tokens": tokens, "x_final": x, "S": S}
        return logits

    # ------------------------------------------------------------------
    # Backward
    # ------------------------------------------------------------------

    def backward(self, targets):
        c = self._cache
        S = c["S"]

        probs = softmax(c["x_final"] @ self.W_out.data + self.b_out.data)
        d_logits = probs.copy()
        d_logits[np.arange(S), targets] -= 1.0
        d_logits /= S

        loss = -np.log(probs[np.arange(S), targets] + 1e-10).mean()

        self.W_out.grad += c["x_final"].T @ d_logits
        self.b_out.grad += d_logits.sum(axis=0)
        d_x = d_logits @ self.W_out.data.T

        for layer in reversed(self.layers):
            d_x = layer.backward(d_x)

        np.add.at(self.embedding.grad, c["tokens"], d_x)
        self.pos_enc.grad[:S] += d_x

        return loss

    # ------------------------------------------------------------------
    # Optimizer
    # ------------------------------------------------------------------

    def zero_grad(self):
        for p in self._global_params():
            p.zero_grad()
        for layer in self.layers:
            layer.zero_grad()

    def step(self, lr=0.001):
        self.adam_t += 1
        for p in self._global_params():
            p.step(lr, self.adam_t)
        for layer in self.layers:
            layer.step(lr, self.adam_t)

    def clip_gradients(self, max_norm=1.0):
        for p in self._global_params():
            np.clip(p.grad, -max_norm, max_norm, out=p.grad)
        for layer in self.layers:
            layer.clip_gradients(max_norm)

    def _global_params(self):
        return [self.embedding, self.pos_enc, self.W_out, self.b_out]

    # ------------------------------------------------------------------
    # Convenience delegates — growth targets the last layer
    # ------------------------------------------------------------------

    def most_starved_layer(self):
        """Find the layer whose neurons are most saturated (highest avg EMA).
        This layer needs more capacity the most."""
        best = None
        best_ema = -1.0
        for layer in self.layers:
            n = layer.active_ffn
            if n == 0:
                continue
            avg = float(layer._ffn_activation_ema[:n].mean())
            if avg > best_ema:
                best_ema = avg
                best = layer
        return best if best is not None else self.layers[-1]

    @property
    def heads(self):
        return self.layers[-1].heads

    @property
    def active_ffn(self):
        return self.layers[-1].active_ffn

    @property
    def max_ffn(self):
        return self.layers[-1].max_ffn

    def prune_ffn(self, threshold=0.01):
        total = 0
        for layer in self.layers:
            total += layer.prune_ffn(threshold)
        return total

    def prune_heads(self, min_heads=1):
        total = 0
        for layer in self.layers:
            total += layer.prune_heads(min_heads)
        return total

    # ------------------------------------------------------------------
    # Aggregated counters across all layers
    # ------------------------------------------------------------------

    @property
    def total_heads_recruited(self):
        return sum(l.total_heads_recruited for l in self.layers)

    @property
    def total_ffn_recruited(self):
        return sum(l.total_ffn_recruited for l in self.layers)

    @property
    def total_heads_pruned(self):
        return sum(l.total_heads_pruned for l in self.layers)

    @property
    def total_ffn_pruned(self):
        return sum(l.total_ffn_pruned for l in self.layers)

    # ------------------------------------------------------------------
    # Generation
    # ------------------------------------------------------------------

    def generate(self, tokenizer, prompt, length=100, temperature=0.8):
        tokens = tokenizer.encode(prompt)
        for _ in range(length):
            ctx = tokens[-self.context_len:]
            logits = self.forward(np.array(ctx, dtype=np.int32))
            last_logits = logits[-1] / temperature
            probs = softmax(last_logits)
            probs = np.clip(probs, 1e-10, None)
            probs /= probs.sum()
            next_token = np.random.choice(len(probs), p=probs)
            tokens.append(int(next_token))
        return tokenizer.decode(tokens)

    # ------------------------------------------------------------------
    # Metrics
    # ------------------------------------------------------------------

    @property
    def active_params(self):
        total = self.embedding.data.size + self.pos_enc.data.size
        total += self.W_out.data.size + self.b_out.data.size
        for layer in self.layers:
            total += layer.active_params
        return total

    @property
    def ffn_freelist(self):
        return sum(l.ffn_freelist for l in self.layers)

    def summary(self):
        parts = []
        for i, layer in enumerate(self.layers):
            parts.append(
                f"L{i}({len(layer.heads)}h,{layer.active_ffn}ffn)")
        return (f"layers={len(self.layers)}, {'+'.join(parts)}, "
                f"params={self.active_params:,}")

    # ------------------------------------------------------------------
    # Checkpoint save/load
    # ------------------------------------------------------------------

    def save_checkpoint(self, directory):
        """Save full model state so training can resume later."""
        os.makedirs(directory, exist_ok=True)

        layer_configs = []
        for layer in self.layers:
            layer_configs.append({
                "max_ffn": layer.max_ffn,
                "active_ffn": layer.active_ffn,
                "n_heads": len(layer.heads),
                "total_heads_recruited": layer.total_heads_recruited,
                "total_ffn_recruited": layer.total_ffn_recruited,
                "total_heads_pruned": layer.total_heads_pruned,
                "total_ffn_pruned": layer.total_ffn_pruned,
            })

        config = {
            "vocab_size": self.vocab_size,
            "d_model": self.d_model,
            "context_len": self.context_len,
            "adam_t": self.adam_t,
            "n_layers": len(self.layers),
            "layers": layer_configs,
        }
        with open(os.path.join(directory, "config.json"), "w") as f:
            json.dump(config, f, indent=2)

        with open(os.path.join(directory, "history.json"), "w") as f:
            json.dump(self.history, f)

        def _save_param(prefix, param, arrays):
            for attr in ("data", "m", "v"):
                arrays[f"{prefix}.{attr}"] = getattr(param, attr)

        arrays = {}
        _save_param("embedding", self.embedding, arrays)
        _save_param("pos_enc", self.pos_enc, arrays)
        _save_param("W_out", self.W_out, arrays)
        _save_param("b_out", self.b_out, arrays)

        for li, layer in enumerate(self.layers):
            pfx = f"layer_{li}"
            _save_param(f"{pfx}.W1", layer.W1, arrays)
            _save_param(f"{pfx}.b1", layer.b1, arrays)
            _save_param(f"{pfx}.W2", layer.W2, arrays)
            _save_param(f"{pfx}.b2", layer.b2, arrays)
            arrays[f"{pfx}.ffn_activation_ema"] = layer._ffn_activation_ema

            for hi, h in enumerate(layer.heads):
                hp = f"{pfx}.head_{hi}"
                _save_param(f"{hp}.W_Q", h.W_Q, arrays)
                _save_param(f"{hp}.W_K", h.W_K, arrays)
                _save_param(f"{hp}.W_V", h.W_V, arrays)
                arrays[f"{hp}.frozen"] = np.array(h.frozen)
                arrays[f"{hp}.avg_output_norm"] = np.array(h.avg_output_norm)

        np.savez_compressed(os.path.join(directory, "weights.npz"), **arrays)

    @classmethod
    def load_checkpoint(cls, directory):
        """Restore model from a saved checkpoint.

        Backward compatible: old checkpoints without 'n_layers' are
        loaded as a single-layer model.
        """
        with open(os.path.join(directory, "config.json"), "r") as f:
            config = json.load(f)

        model = cls.__new__(cls)
        model.vocab_size = config["vocab_size"]
        model.d_model = config["d_model"]
        model.context_len = config["context_len"]
        model.adam_t = config["adam_t"]
        model._cache = {}

        w = np.load(os.path.join(directory, "weights.npz"))

        def _load_param(prefix):
            p = Param(w[f"{prefix}.data"])
            p.m = w[f"{prefix}.m"].copy()
            p.v = w[f"{prefix}.v"].copy()
            p.grad = np.zeros_like(p.data)
            return p

        def _load_head(prefix):
            h = AttentionHead.__new__(AttentionHead)
            h.W_Q = _load_param(f"{prefix}.W_Q")
            h.W_K = _load_param(f"{prefix}.W_K")
            h.W_V = _load_param(f"{prefix}.W_V")
            h.frozen = bool(w[f"{prefix}.frozen"])
            h.avg_output_norm = float(w[f"{prefix}.avg_output_norm"])
            h._ema_decay = 0.95
            h._cache = {}
            return h

        def _load_layer(prefix, lc):
            layer = TransformerLayer.__new__(TransformerLayer)
            layer.d_model = model.d_model
            layer.max_ffn = lc["max_ffn"]
            layer.active_ffn = lc["active_ffn"]
            layer.total_heads_recruited = lc["total_heads_recruited"]
            layer.total_ffn_recruited = lc["total_ffn_recruited"]
            layer.total_heads_pruned = lc["total_heads_pruned"]
            layer.total_ffn_pruned = lc["total_ffn_pruned"]
            layer._cache = {}
            layer.W1 = _load_param(f"{prefix}.W1")
            layer.b1 = _load_param(f"{prefix}.b1")
            layer.W2 = _load_param(f"{prefix}.W2")
            layer.b2 = _load_param(f"{prefix}.b2")
            ema_key = f"{prefix}.ffn_activation_ema"
            layer._ffn_activation_ema = w[ema_key].copy() if ema_key in w \
                else np.ones(layer.max_ffn, dtype=np.float32)
            layer.heads = []
            for hi in range(lc["n_heads"]):
                layer.heads.append(_load_head(f"{prefix}.head_{hi}"))
            return layer

        model.embedding = _load_param("embedding")
        model.pos_enc = _load_param("pos_enc")
        model.W_out = _load_param("W_out")
        model.b_out = _load_param("b_out")

        if "n_layers" in config:
            # New multi-layer format
            model.layers = []
            for li, lc in enumerate(config["layers"]):
                model.layers.append(_load_layer(f"layer_{li}", lc))
        else:
            # Old single-layer format — migrate into layer_0
            lc = {
                "max_ffn": config["max_ffn"],
                "active_ffn": config["active_ffn"],
                "n_heads": config["n_heads"],
                "total_heads_recruited": config["total_heads_recruited"],
                "total_ffn_recruited": config["total_ffn_recruited"],
                "total_heads_pruned": config["total_heads_pruned"],
                "total_ffn_pruned": config["total_ffn_pruned"],
            }
            layer = TransformerLayer.__new__(TransformerLayer)
            layer.d_model = model.d_model
            layer.max_ffn = lc["max_ffn"]
            layer.active_ffn = lc["active_ffn"]
            layer.total_heads_recruited = lc["total_heads_recruited"]
            layer.total_ffn_recruited = lc["total_ffn_recruited"]
            layer.total_heads_pruned = lc["total_heads_pruned"]
            layer.total_ffn_pruned = lc["total_ffn_pruned"]
            layer._cache = {}
            layer.W1 = _load_param("W1")
            layer.b1 = _load_param("b1")
            layer.W2 = _load_param("W2")
            layer.b2 = _load_param("b2")
            layer._ffn_activation_ema = w["ffn_activation_ema"].copy()
            layer.heads = []
            for hi in range(lc["n_heads"]):
                layer.heads.append(_load_head(f"head_{hi}"))
            model.layers = [layer]

        history_path = os.path.join(directory, "history.json")
        if os.path.exists(history_path):
            with open(history_path, "r") as f:
                model.history = json.load(f)
        else:
            model.history = {"loss": [], "events": [], "snapshots": []}

        return model


# ---------------------------------------------------------------------------
# Training
# ---------------------------------------------------------------------------

def find_kleis_files(base_dir):
    return sorted(glob.glob(os.path.join(base_dir, "**", "*.kleis"),
                            recursive=True))


def main():
    np.random.seed(42)

    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(script_dir, "..", ".."))
    checkpoint_dir = os.path.join(script_dir, "brain_checkpoint")

    kleis_files = find_kleis_files(project_root)

    print("=" * 70)
    print("  GROWING TRANSFORMER BRAIN — Learning Kleis Syntax")
    print("=" * 70)
    print()

    if not kleis_files:
        print("  No .kleis files found!")
        return

    all_text = ""
    skipped_lines = 0
    kept_lines = 0
    for f in kleis_files:
        try:
            with open(f, "r") as fh:
                for line in fh:
                    stripped = line.lstrip()
                    if stripped.startswith("//"):
                        skipped_lines += 1
                        continue
                    all_text += line
                    kept_lines += 1
            all_text += "\n"
        except Exception:
            pass

    print(f"  Training corpus: {len(kleis_files)} .kleis files, "
          f"{len(all_text):,} characters")
    print(f"  Filtered: kept {kept_lines:,} code lines, "
          f"skipped {skipped_lines:,} comment lines")

    for f in kleis_files[:5]:
        print(f"    {os.path.relpath(f, project_root)}")
    if len(kleis_files) > 5:
        print(f"    ... and {len(kleis_files) - 5} more")
    print()

    CONTEXT = 96
    D_MODEL = 96

    # --- Resume from checkpoint or start fresh ---
    tokenizer_path = os.path.join(checkpoint_dir, "tokenizer.json")
    resumed = False

    if os.path.isdir(checkpoint_dir) and os.path.exists(tokenizer_path):
        print("  Checkpoint found — resuming from previous session!")
        tokenizer = CharTokenizer.load(tokenizer_path)

        new_chars = sorted(set(all_text) - set(tokenizer.char_to_id.keys()))
        if new_chars:
            print(f"  (Corpus has {len(new_chars)} new chars — "
                  f"using saved vocab, new chars mapped to id 0)")

        model = GrowingTransformer.load_checkpoint(checkpoint_dir)
        resumed = True
        print(f"  Loaded: {model.summary()}")
        print(f"  Adam step: {model.adam_t}, "
              f"heads recruited: {model.total_heads_recruited}, "
              f"FFN recruited: {model.total_ffn_recruited}")
        print(f"  FFN pruned: {model.total_ffn_pruned}, "
              f"heads pruned: {model.total_heads_pruned}")

        # Widen d_model if checkpoint is narrower
        if model.d_model < D_MODEL:
            old_d = model.d_model
            model.widen_d_model(D_MODEL)
            print(f"  Widened d_model: {old_d} → {D_MODEL} "
                  f"(old weights preserved, new dims near-zero)")

        # Extend positional encoding if context window grew
        if model.context_len < CONTEXT:
            old_len = model.context_len
            old_pe = model.pos_enc.data
            new_pe = np.random.randn(CONTEXT, model.d_model).astype(np.float32) * 0.02
            new_pe[:old_len] = old_pe
            model.pos_enc = Param(new_pe)
            model.pos_enc.m = np.zeros_like(new_pe)
            model.pos_enc.v = np.zeros_like(new_pe)
            model.pos_enc.m[:old_len] = 0
            model.pos_enc.v[:old_len] = 0
            model.context_len = CONTEXT
            print(f"  Extended context window: {old_len} → {CONTEXT}")

        if len(model.layers) < 2:
            n_layers = model.add_layer(
                initial_heads=1, initial_ffn=32, max_ffn=256)
            print(f"  Added layer 1 (near-zero init) → {n_layers} layers")
            model.history["events"].append(
                [model.adam_t, "+layer 1 (near-zero)"])

        if len(model.layers) < 3:
            n_layers = model.add_layer(
                initial_heads=1, initial_ffn=32, max_ffn=512)
            print(f"  Added layer 2 (near-zero init) → {n_layers} layers")
            model.history["events"].append(
                [model.adam_t, "+layer 2 (near-zero)"])
    else:
        print("  No checkpoint — starting from scratch.")
        tokenizer = CharTokenizer(all_text)
        model = GrowingTransformer(
            vocab_size=tokenizer.vocab_size,
            d_model=D_MODEL,
            context_len=CONTEXT,
            initial_heads=1,
            initial_ffn=32,
            max_ffn=512,
        )
        print(f"  Model: d_model={D_MODEL}, context={CONTEXT}")
        print(f"  Initial: {model.summary()}")

    data = np.array(tokenizer.encode(all_text), dtype=np.int32)
    print(f"  Vocabulary: {tokenizer.vocab_size} unique characters")
    print(f"  Freelist: attention=unlimited, FFN={model.ffn_freelist} neurons")
    print()

    # ----- Training loop -----

    MAX_STEPS = 50000
    LR_START = 0.0001
    LR_END = 0.00002
    GROWTH_CHECK = 2000
    PRUNE_EVERY = 5000
    PRINT_EVERY = 2500
    SAMPLE_EVERY = 10000
    VALIDATE_EVERY = 100
    VALIDATE_PRINT_EVERY = 2500
    LAYER_WARMUP = 10000
    FREEZE_GROWTH = False
    CURRICULUM_MIX = 0.20

    KLEIS_PROMPTS = ["structure ", "define ", "axiom ", "import ",
                     "example {\n  ", "let "]
    CURRICULUM_RATIO_START = 0.85
    CURRICULUM_RATIO_END = 0.50

    # ----- Build curriculum: valid Kleis snippets for completion tasks -----

    curriculum_snippets = []
    for f in kleis_files:
        try:
            with open(f, "r") as fh:
                content = fh.read()
            if len(content) < 50:
                continue
            fb = validate_kleis(content)
            if fb.valid:
                if len(content) <= 2000:
                    curriculum_snippets.append(content)
                else:
                    # Extract structure/example blocks from large files
                    lines = content.split("\n")
                    block, depth = [], 0
                    for line in lines:
                        if depth == 0 and block:
                            chunk = "\n".join(block)
                            if 50 < len(chunk) < 2000:
                                curriculum_snippets.append(chunk)
                            block = []
                        block.append(line)
                        depth += line.count("{") - line.count("}")
                    if block:
                        chunk = "\n".join(block)
                        if 50 < len(chunk) < 2000:
                            curriculum_snippets.append(chunk)
        except Exception:
            pass
    # Pre-encode curriculum snippets for direct training
    curriculum_encoded = []
    for snip in curriculum_snippets:
        ids = tokenizer.encode(snip)
        if len(ids) > CONTEXT + 1:
            curriculum_encoded.append(np.array(ids, dtype=np.int32))
    print(f"  Curriculum: {len(curriculum_snippets)} valid snippets "
          f"({len(curriculum_encoded)} long enough for training)")

    best_loss = float("inf")
    no_improve = 0
    losses: list[float] = []
    growth_log: list[tuple] = []
    prune_log: list[tuple] = []
    valid_count = 0
    invalid_count = 0

    if "validations" not in model.history:
        model.history["validations"] = []

    print("  Training (next-character prediction)...")
    print()

    for step in range(1, MAX_STEPS + 1):
        # Cosine LR decay
        progress = step / MAX_STEPS
        lr = LR_END + 0.5 * (LR_START - LR_END) * (1 + np.cos(np.pi * progress))

        # 20% of steps: train on a known-valid curriculum snippet (teaching)
        # 80% of steps: train on random corpus window (general learning)
        if curriculum_encoded and np.random.random() < CURRICULUM_MIX:
            snip = curriculum_encoded[
                np.random.randint(len(curriculum_encoded))]
            start = np.random.randint(0, len(snip) - CONTEXT - 1)
            tokens = snip[start : start + CONTEXT + 1]
        else:
            start = np.random.randint(0, len(data) - CONTEXT - 1)
            tokens = data[start : start + CONTEXT + 1]
        inp = tokens[:-1]
        tgt = tokens[1:]

        model.zero_grad()
        model.forward(inp)
        loss = model.backward(tgt)
        model.clip_gradients(max_norm=1.0)
        model.step(lr=lr)

        losses.append(loss)
        lifetime_step = model.adam_t + step

        # Record loss periodically for paper
        if step % 100 == 0:
            model.history["loss"].append(
                [lifetime_step, float(np.mean(losses[-100:]))])

        # Growth trigger — smoothed loss, paused during warmup
        smooth_loss = np.mean(losses[-200:]) if len(losses) >= 200 else loss
        if smooth_loss < best_loss - 0.005:
            best_loss = smooth_loss
            no_improve = 0
        else:
            no_improve += 1

        growth_ok = (not FREEZE_GROWTH
                     and (step > LAYER_WARMUP or len(model.layers) == 1))
        if no_improve >= GROWTH_CHECK and growth_ok:
            gl = model.most_starved_layer()
            gl_idx = model.layers.index(gl)
            if len(gl.heads) * 3 <= gl.active_ffn // 16:
                nh = gl.recruit_head()
                growth_log.append((step, f"L{gl_idx} head #{nh}"))
                model.history["events"].append(
                    [lifetime_step, f"+L{gl_idx} head #{nh}"])
                print(f"    Step {step}: L{gl_idx} recruited head #{nh}")
            else:
                added = gl.recruit_ffn(16)
                growth_log.append((step, f"L{gl_idx} +{added} FFN"))
                model.history["events"].append(
                    [lifetime_step, f"+L{gl_idx} {added} FFN"])
                print(f"    Step {step}: L{gl_idx} recruited {added} FFN "
                      f"→ {gl.active_ffn} active")
            no_improve = 0
            best_loss = loss

        # Pruning — return dead capacity to freelist (all layers)
        if step % PRUNE_EVERY == 0 and step >= 500:
            ffn_pruned = model.prune_ffn(threshold=0.01)
            heads_pruned = model.prune_heads(min_heads=1)
            if ffn_pruned > 0:
                per_layer = ", ".join(
                    f"L{i}={l.active_ffn}" for i, l in enumerate(model.layers))
                prune_log.append((step, f"-{ffn_pruned} FFN neurons"))
                model.history["events"].append(
                    [lifetime_step, f"-{ffn_pruned} FFN"])
                print(f"    Step {step}: Pruned {ffn_pruned} FFN neurons "
                      f"({per_layer}), freelist={model.ffn_freelist}")
            if heads_pruned > 0:
                per_layer = ", ".join(
                    f"L{i}={len(l.heads)}h" for i, l in enumerate(model.layers))
                prune_log.append((step, f"-{heads_pruned} attention heads"))
                model.history["events"].append(
                    [lifetime_step, f"-{heads_pruned} heads"])
                print(f"    Step {step}: Pruned {heads_pruned} attention heads "
                      f"({per_layer})")

        # Kleis validation — curriculum: complete real Kleis snippets
        if step % VALIDATE_EVERY == 0:
            lifetime_step_v = model.adam_t + step
            progress = step / MAX_STEPS
            prefix_ratio = (CURRICULUM_RATIO_START
                            + (CURRICULUM_RATIO_END - CURRICULUM_RATIO_START)
                            * progress)

            if curriculum_snippets:
                snippet = curriculum_snippets[
                    np.random.randint(len(curriculum_snippets))]
                cut = int(len(snippet) * prefix_ratio)
                prompt = snippet[:cut]
                gen_len = len(snippet) - cut + 20
                gen = model.generate(
                    tokenizer, prompt, length=gen_len, temperature=0.7)
            else:
                prompt = KLEIS_PROMPTS[
                    step // VALIDATE_EVERY % len(KLEIS_PROMPTS)]
                gen = model.generate(
                    tokenizer, prompt, length=200, temperature=0.7)

            fb = validate_kleis(gen)
            reward = fb.reward

            if fb.valid:
                valid_count += 1
            else:
                invalid_count += 1

            if reward > 0.05:
                prefix_len = fb.error_char_offset
                train_text = gen[:prefix_len] if not fb.valid else gen
                gen_ids = tokenizer.encode(train_text)
                if len(gen_ids) > CONTEXT + 1:
                    gen_ids = gen_ids[:CONTEXT + 1]
                if len(gen_ids) > 2:
                    inp_v = np.array(gen_ids[:-1], dtype=np.int32)
                    tgt_v = np.array(gen_ids[1:], dtype=np.int32)
                    model.zero_grad()
                    model.forward(inp_v)
                    r_loss = model.backward(tgt_v)
                    model.clip_gradients(max_norm=1.0)
                    model.step(lr=lr * 0.5 * reward)
                    if fb.valid and step % VALIDATE_PRINT_EVERY == 0:
                        print(f"    Step {step}: VALID completion "
                              f"(prefix={prefix_ratio:.0%}, "
                              f"loss={r_loss:.3f})")
                    elif step % VALIDATE_PRINT_EVERY == 0:
                        print(f"    Step {step}: {fb.short()}")
                        print(f"      → prefix={prefix_ratio:.0%}, "
                              f"reinforced {prefix_len} chars, "
                              f"reward={reward:.2f}")
            elif step % VALIDATE_PRINT_EVERY == 0:
                print(f"    Step {step}: {fb.short()}")

            model.history["validations"].append(
                [lifetime_step_v, valid_count, invalid_count,
                 round(reward, 3), fb.error_kind,
                 fb.valid_prefix_ratio])

        if step % PRINT_EVERY == 0:
            recent = np.mean(losses[-PRINT_EVERY:])
            ppl = np.exp(min(recent, 20))
            model.history["snapshots"].append(
                [lifetime_step, len(model.heads), model.active_ffn,
                 model.active_params])
            print(f"    Step {step:4d}: loss={recent:.3f}  ppl={ppl:.1f}  "
                  f"| {model.summary()}")

        if step % SAMPLE_EVERY == 0:
            idx = np.random.randint(0, len(data) - 16)
            prompt = tokenizer.decode(data[idx : idx + 16].tolist())
            sample = model.generate(tokenizer, prompt, length=80,
                                    temperature=0.8)
            print()
            print(f"    --- Sample at step {step} ---")
            print(f"    Prompt:    '{prompt}'")
            print(f"    Generated: '{sample[len(prompt):80]}'")
            print()

    # ----- Final generation -----

    print()
    print("=" * 70)
    print("  FINAL SAMPLES (temperature=0.7)")
    print("=" * 70)

    prompts = ["structure ", "define ", "axiom ", "example {"]
    for prompt in prompts:
        if all(c in tokenizer.char_to_id for c in prompt):
            sample = model.generate(tokenizer, prompt, length=150,
                                    temperature=0.7)
            print(f"\n  '{prompt}' →")
            # Print wrapped
            text = sample[len(prompt) : len(prompt) + 150]
            for i in range(0, len(text), 70):
                print(f"    {text[i:i+70]}")

    # ----- Summary -----

    print()
    print("=" * 70)
    print("  SUMMARY")
    print("=" * 70)
    final_loss = np.mean(losses[-100:])
    print(f"  Steps: {MAX_STEPS}")
    print(f"  Final loss: {final_loss:.3f}  "
          f"(perplexity: {np.exp(min(final_loss, 20)):.1f})")
    print(f"  Model: {model.summary()}")
    for i, layer in enumerate(model.layers):
        print(f"    Layer {i}: {len(layer.heads)} heads "
              f"(+{layer.total_heads_recruited}/-{layer.total_heads_pruned}), "
              f"{layer.active_ffn} FFN "
              f"(+{layer.total_ffn_recruited}/-{layer.total_ffn_pruned}), "
              f"freelist={layer.ffn_freelist}")
    print(f"  Total params: {model.active_params:,}")
    total_tried = valid_count + invalid_count
    rate = valid_count / max(total_tried, 1)
    avg_prefix = 0.0
    if model.history.get("validations"):
        recent_v = model.history["validations"][-total_tried:]
        avg_prefix = np.mean([v[5] for v in recent_v if len(v) > 5])
    print(f"  Kleis validation: {valid_count}/{total_tried} valid "
          f"({rate:.1%}), avg prefix correctness: {avg_prefix:.1%}")
    print()

    if growth_log or prune_log:
        events = [(s, d) for s, d in growth_log]
        events += [(s, d) for s, d in prune_log]
        events.sort(key=lambda x: x[0])
        print("  Growth & pruning events:")
        for s, desc in events:
            print(f"    Step {s:4d}: {desc}")
    else:
        print("  No growth/pruning events (model sufficient from start)")

    # ----- Save checkpoint -----

    print()
    model.save_checkpoint(checkpoint_dir)
    tokenizer.save(os.path.join(checkpoint_dir, "tokenizer.json"))
    print(f"  Checkpoint saved to {os.path.relpath(checkpoint_dir, project_root)}/")
    print(f"  Total lifetime training: {model.adam_t} Adam steps")
    print()

    interactive(model, tokenizer)


def interactive(model, tokenizer):
    """Interactive prompt loop — type a prompt, see what the brain generates."""
    print("=" * 70)
    print("  INTERACTIVE MODE — type a prompt, get a completion")
    print("  Commands:  :q quit  :t0.7 set temperature  :l200 set length")
    print("=" * 70)
    print()

    temperature = 0.7
    length = 150

    while True:
        try:
            prompt = input("  prompt> ")
        except (EOFError, KeyboardInterrupt):
            print()
            break

        if not prompt:
            continue
        if prompt == ":q":
            break
        if prompt.startswith(":t"):
            try:
                temperature = float(prompt[2:])
                print(f"  temperature = {temperature}")
            except ValueError:
                print("  usage: :t0.7")
            continue
        if prompt.startswith(":l"):
            try:
                length = int(prompt[2:])
                print(f"  length = {length}")
            except ValueError:
                print("  usage: :l200")
            continue

        unknown = [c for c in prompt if c not in tokenizer.char_to_id]
        if unknown:
            print(f"  (unknown chars mapped to id 0: {unknown})")

        sample = model.generate(tokenizer, prompt, length=length,
                                temperature=temperature)
        text = sample[len(prompt): len(prompt) + length]
        print()
        for i in range(0, len(text), 70):
            print(f"    {text[i:i+70]}")
        print()


def chat_mode():
    """Load checkpoint and go straight to interactive mode (no training)."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    checkpoint_dir = os.path.join(script_dir, "brain_checkpoint")
    tokenizer_path = os.path.join(checkpoint_dir, "tokenizer.json")

    if not os.path.isdir(checkpoint_dir) or not os.path.exists(tokenizer_path):
        print("  No checkpoint found. Run without --chat first to train.")
        return

    print("=" * 70)
    print("  GROWING TRANSFORMER BRAIN — Chat Mode")
    print("=" * 70)
    print()

    tokenizer = CharTokenizer.load(tokenizer_path)
    model = GrowingTransformer.load_checkpoint(checkpoint_dir)
    print(f"  Loaded: {model.summary()}")
    print(f"  Lifetime training: {model.adam_t} Adam steps")
    print()

    interactive(model, tokenizer)


if __name__ == "__main__":
    if "--chat" in sys.argv:
        chat_mode()
    else:
        main()
