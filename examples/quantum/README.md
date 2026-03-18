# Quantum Computing Examples

This directory contains Kleis examples for quantum computing, demonstrating how to express and verify quantum algorithms using formal axioms.

## Files

| File | Description |
|------|-------------|
| `quantum_foundations.kleis` | Core structures: quantum states, unitary gates, measurement, no-cloning theorem, entanglement |
| `bell_state_simulation.kleis` | Numerical simulation of Bell state creation using matrix operations |
| `deutsch_jozsa.kleis` | Deutsch-Jozsa algorithm - exponential speedup for constant vs balanced detection |
| `grover_search.kleis` | Grover's search algorithm - quadratic speedup for unstructured search |

## Key Concepts

### Quantum States
- Represented as complex vectors with unit norm
- Computational basis: |0⟩, |1⟩, |00⟩, |01⟩, etc.
- Superposition: |+⟩ = (|0⟩ + |1⟩)/√2

### Quantum Gates
- Single-qubit: Hadamard (H), Pauli (X, Y, Z), S, T
- Two-qubit: CNOT, CZ, SWAP
- All gates are unitary (U†U = I)

### Measurement
- Born rule: P(i) = |αᵢ|²
- Collapses superposition to definite state

### Entanglement
- Bell states: maximally entangled 2-qubit states
- Non-separable: cannot be written as tensor product

## Running Examples

```bash
# Check syntax
kleis check examples/quantum/quantum_foundations.kleis

# Run examples (with output)
kleis test examples/quantum/bell_state_simulation.kleis
```

## How Kleis Helps with Quantum Computing

1. **Axiom Verification**: Define and verify quantum properties (unitarity, no-cloning)
2. **Algorithm Correctness**: Express algorithm invariants and correctness conditions
3. **Numerical Simulation**: Use LAPACK-backed matrix operations
4. **Educational**: Clear, mathematical notation for understanding quantum concepts

## References

- Nielsen & Chuang, "Quantum Computation and Quantum Information" (2000)
- Deutsch & Jozsa, "Rapid solution of problems by quantum computation" (1992)
- Grover, "A fast quantum mechanical algorithm for database search" (1996)

