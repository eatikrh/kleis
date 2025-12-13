# Z3 Algebraic Data Type Support

**Priority:** High  
**Estimated Effort:** 2-4 hours  
**Status:** ✅ Phase 4 Complete - Symbolic ADT Matching Fixed!

## Goal

Enable Z3 to verify properties of Kleis Algebraic Data Types (ADTs), including:
- Constructor verification
- Pattern matching translation
- Type-safe protocol verification

## Current State (Updated Dec 13, 2024)

### ✅ What Works
- ADT definitions parse correctly: `data Protocol = ICMP | TCP | UDP`
- Constructors are recognized: `Packet(4, 5, 1500, 64, TCP, ...)`
- **Pattern matching translation to Z3** - `match` expressions now translate to nested `ite`
- **Variable binding in patterns** - `match pkt { Packet(v, _, _, ttl, _, _, _) => ttl }`
- **Constructor pattern matching** - `match x { Some(a) => a | None => 0 }`
- **Nullary ADT constructors** - `TCP`, `UDP`, `ICMP` loaded as Z3 identity elements
- File loading preserves ADTs and registers constructors
- REPL correctly expands functions with match before Z3 verification
- **✅ Symbolic ADT matching** - `perm_level(Owner) = 4` now verifies correctly!

### ✅ Fixed: Symbolic ADT Matching (Dec 13, 2024)

**The bug (discovered via Zanzibar example):**
```kleis
:verify perm_level(Owner) = 4
❌ Invalid  // Was failing!
```

**The fix (two parts):**

1. **`pattern_to_condition()` in `src/solvers/z3/backend.rs`:**
   - When matching nullary constructor patterns against symbolic scrutinee
   - Now looks up the constructor in `identity_elements` HashMap
   - Uses Z3 equality to compare scrutinee with the registered identity element

2. **`load_identity_element()` asserts distinctness:**
   - When loading a new identity element (ADT constructor)
   - Asserts it's distinct from all previously loaded elements
   - This ensures `Owner ≠ Editor ≠ Viewer` in Z3

**Now works:**
```kleis
:verify perm_level(Owner) = 4   // ✅ Valid
:verify perm_level(Editor) = 3  // ✅ Valid
:verify perm_level(Viewer) = 1  // ✅ Valid
```

### ⚠️ Partial Support
- Full Z3 Datatype sorts not yet created (using uninterpreted functions + identity elements)
- Constructor accessors not auto-generated

#### Low Priority
- Full Z3 Datatype sort creation (for exhaustiveness checking)
- Auto-generated accessor functions (e.g., `Packet.version(pkt)`)

## Verified Examples

These now work in the REPL (including `TCP`, `UDP`, `ICMP` constructor names):

```
λ> :load examples/protocols/ipv4_types.kleis
✅ Loaded: 3 functions, 0 structures, 3 data types

λ> :verify get_ttl(Packet(4, 5, 100, 64, TCP, Address(192, 168, 1, 1), Address(10, 0, 0, 1))) = 64
   📌 Loaded identity element: ICMP
   📌 Loaded identity element: TCP
   📌 Loaded identity element: UDP
✅ Valid

λ> :verify get_ttl(Packet(4, 5, 100, 128, ICMP, Address(1, 1, 1, 1), Address(2, 2, 2, 2))) = 128
✅ Valid

λ> :verify is_valid_version(Packet(4, 5, 100, 64, UDP, Address(1, 1, 1, 1), Address(2, 2, 2, 2))) = 1
✅ Valid

λ> :verify ∀(ttl : ℤ). get_ttl(Packet(4, 5, 100, ttl, TCP, Address(1,1,1,1), Address(2,2,2,2))) = ttl
✅ Valid
```

## Implementation Progress

### ✅ Phase 1: Z3 Datatype Creation (Deferred)
Using uninterpreted functions for now. Full Z3 Datatype sorts can be added later for:
- Better error messages
- Exhaustiveness checking
- Accessor functions

### ✅ Phase 2: Constructor Translation
Constructors like `Packet(...)` and `Address(...)` are declared as uninterpreted functions:
```
🔧 Declaring uninterpreted function: Packet with arity 7
🔧 Declaring uninterpreted function: Address with arity 4
```

### ✅ Phase 3: Match Expression Translation (COMPLETE)
Match expressions now translate to nested Z3 `ite`:
```rust
// In src/solvers/z3/backend.rs
Expression::Match { scrutinee, cases } => {
    self.translate_match(scrutinee, cases, vars)
}
```

Supports:
- Wildcard patterns: `_`
- Variable binding: `x`
- Constant patterns: `5`
- Constructor patterns: `Some(x)`, `Pair(a, b)`
- Nested patterns

### ✅ Phase 4: Testing (8 tests pass)
- `tests/match_translation_test.rs` - 8 integration tests
- All patterns tested: wildcard, variable, constant, constructor, nested

## Files Modified

1. `src/solvers/z3/backend.rs` - Added `translate_match()`, `translate_match_case()`, `bind_pattern_vars()`, `pattern_to_condition()`
2. `src/bin/repl.rs` - Added Match support to `expand_user_functions()` and `substitute_var()`
3. `tests/match_translation_test.rs` - 8 integration tests

## Future Enhancements

### Full Z3 Datatype Sorts
```rust
fn create_z3_datatype(&self, data_def: &DataDef) -> z3::Sort {
    let datatype = z3::Datatype::new(ctx, &data_def.name);
    for variant in &data_def.variants {
        datatype.variant(&variant.name, &variant.fields...);
    }
    datatype.create()
}
```

Benefits:
- Constructor name matching (`TCP`, `UDP`, etc.)
- Accessor functions (`Packet.version(pkt)`)
- Exhaustiveness checking by Z3

## Theoretical Background

The type inference system (Equation Editor) uses concepts from:

### Hindley-Milner Type Inference
- **Principal types**: Always infer the most general type
- **Unification**: Make two types equal by finding substitutions
- **Let-polymorphism**: Functions can be used at multiple types

### E-Unification (Equational Unification)
- Standard unification: `f(x) = f(y)` implies `x = y`
- E-unification: Unification modulo equational theories
- Relevant for ADTs: Constructor equality depends on type theory

### Pattern Matching Theory
- **Exhaustiveness checking**: All cases covered?
- **Reachability**: Any unreachable patterns? (dead code)
- **Constructor families**: `True`/`False` both belong to `Bool`

### Key Insight for Z3

The gap between type inference and Z3 is:

| System | How it handles `Owner` |
|--------|------------------------|
| Type Inference | Looks up in `DataTypeRegistry`, knows it's a `Permission` constructor |
| Z3 Backend | Creates fresh constant, doesn't know it's a constructor |

**Fix:** Share the constructor registry between type inference and Z3 translation.

## Related

- ADR-021: Algebraic Data Types
- ADR-022: Z3 Integration
- `examples/protocols/ipv4_types.kleis`
- `examples/authorization/zanzibar.kleis` (discovered the symbolic ADT bug)
