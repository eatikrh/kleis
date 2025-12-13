# Z3 Algebraic Data Type Support

**Priority:** High  
**Estimated Effort:** 2-4 hours  
**Status:** Planned

## Goal

Enable Z3 to verify properties of Kleis Algebraic Data Types (ADTs), including:
- Constructor verification
- Pattern matching translation
- Type-safe protocol verification

## Current State

### ✅ What Works
- ADT definitions parse correctly: `data Protocol = ICMP | TCP | UDP`
- Constructors are recognized: `Packet(4, 5, 1500, 64, TCP, ...)`
- Pattern matching in functions: `match pkt { Packet(v, ...) => ... }`
- File loading preserves ADTs

### ❌ What's Missing
- Z3 translation of `match` expressions
- Z3 Datatype sort creation
- Constructor/accessor functions in Z3

## Use Case: Protocol Verification

```kleis
// Define packet type
data IPv4Packet = Packet(version: ℕ, ihl: ℕ, total: ℕ, ttl: ℕ, proto: Protocol)

// Validation with pattern matching
define is_valid(pkt) = match pkt {
    Packet(4, ihl, total, ttl, _) => 
        if ihl >= 5 then if ttl > 0 then 1 else 0 else 0
  | _ => 0
}

// Verify: All valid packets have version 4
:verify ∀(pkt : IPv4Packet). is_valid(pkt) = 1 → get_version(pkt) = 4
```

## Implementation Plan

### Phase 1: Z3 Datatype Creation (1 hour)
```rust
// In src/solvers/z3/backend.rs
fn create_z3_datatype(&self, data_def: &DataDef) -> z3::Sort {
    let datatype = z3::Datatype::new(ctx, &data_def.name);
    for variant in &data_def.variants {
        datatype.variant(&variant.name, &variant.fields...);
    }
    datatype.create()
}
```

### Phase 2: Constructor Translation (30 min)
```rust
// Translate Packet(4, 5, ...) to Z3 constructor call
Expression::Operation { name, args } if is_constructor(name) => {
    let constructor = get_z3_constructor(name);
    constructor.apply(&translated_args)
}
```

### Phase 3: Match Expression Translation (1-2 hours)
```rust
// Translate match to nested Z3 ite
Expression::Match { scrutinee, cases } => {
    // For each case:
    //   1. Check if constructor matches
    //   2. Bind pattern variables
    //   3. Translate body
    // Combine with nested ite
}
```

### Phase 4: Testing (30 min)
- Unit tests for each ADT operation
- Integration test with IPv4 packet validation
- Protocol verification tests

## Z3 API Reference

Z3 supports algebraic datatypes natively:
```python
# Python Z3 example (Rust API similar)
Packet = Datatype('Packet')
Packet.declare('mk_packet', 
    ('version', IntSort()),
    ('ihl', IntSort()),
    ('ttl', IntSort()))
Packet = Packet.create()

# Constructor
pkt = Packet.mk_packet(4, 5, 64)

# Accessor
version = Packet.version(pkt)
```

## Files to Modify

1. `src/solvers/z3/backend.rs` - Main translation logic
2. `src/solvers/z3/translators/` - Add `adt.rs` module
3. `tests/adt_verification_test.rs` - Integration tests

## Success Criteria

```
λ> :load examples/protocols/ipv4_types.kleis
✅ Loaded: 3 functions, 0 structures, 3 data types

λ> :verify is_valid_version(Packet(4, 5, 1500, 64, TCP, src, dst)) = 1
✅ Valid

λ> :verify ∀(pkt : IPv4Packet). is_valid(pkt) = 1 → get_version(pkt) = 4
✅ Valid
```

## Related

- ADR-021: Algebraic Data Types
- ADR-022: Z3 Integration
- `examples/protocols/ipv4_types.kleis`

