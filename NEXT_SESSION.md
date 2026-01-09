# Next Session Tasks

## Type System Gap: Tensor Operation Signatures

The tensor operations used in the Equation Editor (`index_mixed`, `tensor_lower_pair`, etc.) render correctly but lack type signatures in the stdlib. This causes the type checker to return unresolved type variables like `Var(TypeVar(1))` instead of proper tensor types.

**Examples that show this gap:**
- `g_{μν}` - renders correctly but type is `Var(TypeVar(1))`
- `T^i_j` - renders correctly but type is `Var(TypeVar(1))`

**Fix needed:** Add tensor operation signatures to `stdlib/tensors.kleis`:

```kleis
// Hypothetical type signatures for tensor operations
operation index_mixed : (T: Tensor(n), upper: Index, lower: Index) → Tensor(n)
operation tensor_lower_pair : (T: Tensor(n), μ: Index, ν: Index) → Tensor(n)
operation tensor_upper_pair : (T: Tensor(n), μ: Index, ν: Index) → Tensor(n)
```

This would allow the type checker to properly infer tensor types for indexed expressions.

