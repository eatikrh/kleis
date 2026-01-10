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

## Editor Tensor Format: Update to New Convention

The Equation Editor uses an old tensor convention (`subsup`, `index_mixed`) that causes Z3 verification errors when index names collide with mathematical constants.

**Problem:**
- When entering `T_j^i`, the editor creates `subsup(T, j, i)`
- Z3 interprets `i` as the imaginary unit (Complex type), not as a tensor index
- This causes: `Type mismatch in call to 'subsup': argument 3 has type Complex but expected Int`

**Better fix:** Update the editor to use the new tensor format with `kind: 'tensor'` and `indexStructure` metadata:

```javascript
// Old format (current):
subsup: { Operation: { name: 'subsup', args: [{...}, {...}, {...}] } }

// New format (target):
tensor_mixed: { Operation: { 
    name: 'tensor', 
    kind: 'tensor',
    args: [{Object:'T'}, {Object:'j'}, {Object:'i'}],
    metadata: { indexStructure: ['down', 'up'] }
} }
```

The new format explicitly marks indices vs base symbols, avoiding collision with mathematical constants like `i` (imaginary unit).

**Files to update:**
- `static/index.html` - Update `subsup` template to use new tensor format
- `src/solvers/z3/backend.rs` - Add handling for `kind: 'tensor'` operations (treat as formatting-only, return base type)
