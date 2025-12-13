# Z3 Algebraic Data Type Support

**Priority:** High  
**Estimated Effort:** 2-4 hours  
**Status:** ✅ Phase 3 Complete + ADT Constructor Support

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

### ⚠️ Partial Support
- Full Z3 Datatype sorts not yet created (using uninterpreted functions + identity elements)
- Constructor accessors not auto-generated

### ❌ What's Still Missing (Low Priority)
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

## Related

- ADR-021: Algebraic Data Types
- ADR-022: Z3 Integration
- `examples/protocols/ipv4_types.kleis`
