# ADR-037: Graph Editor with Domain-Agnostic Routing

**Status:** Accepted & Implemented (Phase 1)  
**Date:** 2026-04-27  
**Relates to:** ADR-005 (Visual Authoring), ADR-023 (Template Externalization),
ADR-035 (Multi-Domain Template Compiler), ADR-036 (Multi-Domain Template
Generality)

## Context

ADR-036 identified three levels of circuit support and classified "Level 3:
Graph editor — KiCad-like 2D canvas with drag-and-drop wiring" as requiring a
graph AST and deferred it indefinitely. Investigation and prototyping revealed
that graph editing is achievable without a new AST type — graphs are represented
as **incidence matrices** within the existing Kleis AST, and a standalone Graph
Editor application handles the visual editing.

The key insight: the Equation Editor's tree AST and the Graph Editor's graph
model are not competing representations. A graph's topology (incidence matrix)
and its component list are **values inside the AST**, serialized as a `graph()`
operation:

```json
{
  "Operation": {
    "name": "graph",
    "args": [
      { "Operation": { "name": "SparseMatrix", "args": [V, P, [triples...]] } },
      { "List": [component operations...] },
      { "List": [net labels...] },
      { "List": [port labels...] }
    ]
  }
}
```

This representation flows through the existing `editor_node_to_expression`
translation, type inference, and Z3 verification pipelines.

A second insight: different graph domains (electronics, bond graphs, Petri nets,
molecular graphs) share the same incidence-matrix topology model but require
fundamentally different visual behavior — routing algorithms, edge decorations,
junction styles, and directionality. Hardcoding these per domain in JavaScript
would prevent end-user extensibility.

## Decision

### 1. Separate Graph Editor Application

The Graph Editor is a standalone web application (`static/graph_editor.html`,
`static/js/graphEditorMain.js`) that is a sibling to the Equation Editor, not an
extension of it. This separation:

- Avoids shoehorning graph-type UI into equation-type UI
- Allows dedicated interaction paradigms (pan/zoom, wire routing, port
  connections)
- Prevents regressions in the existing Equation Editor
- Enables independent evolution of both editors

### 2. Signed Sparse Port-Based Incidence Matrix

Graphs are represented as **port-based** incidence matrices within the Kleis
AST. The matrix rows correspond to nets and columns to individual **ports**
(not components). A transistor with 3 ports (base, collector, emitter) occupies
3 columns, so port identity is preserved by column position.

Entries are **signed integers**:

| Value | Meaning |
|-------|---------|
| `+1`  | First port of component connected to this net (source / positive) |
| `-1`  | Non-first port connected to this net (sink / negative) |
| `+n`/`-n` | Weighted connection (e.g., double bond in molecular graphs) |
| `0`   | No connection (not stored in sparse format) |

The sign convention follows standard algebraic graph theory: each net assigns
`+1` to its "source" end and `-1` to its "sink" end. For undirected domains the
signs still provide algebraic orientation, enabling axioms like "each row sums
to zero for a simple two-port net." For directed domains (bond graphs, Petri
nets), the signs carry physical meaning.

**Storage format:** COO (Coordinate List) — only non-zero entries are stored as
`(net_index, port_index, value)` triples, flattened into a single list:

```
graph(
  SparseMatrix(V, P, [net0, port0, val0, net1, port1, val1, ...]),
  [component operations...],     // component types
  [net labels...],               // net names
  [port labels...]               // "componentIdx:portName" per column
)
```

The dense V x P matrix can be materialized from COO when needed (e.g., for
display or axiom evaluation), but the canonical AST representation is sparse.
This is efficient for large, sparsely connected graphs and maps directly to
Z3 assertions without iterating over zero entries.

This representation is complete: given the sparse matrix + component list + port
labels, the full port-level connectivity can be reconstructed. It is universal
across domains: electronic circuits, bond graphs, Petri nets, and molecular
graphs all reduce to signed sparse port-based incidence matrices.

**Why port-based, not component-based:** A component-level matrix (nets x
components) with +1/-1 polarity encoding cannot distinguish which port of a
multi-port component (3+ ports) is connected to a given net. Port-based columns
eliminate this ambiguity — the column IS the port.

**Why signed:** Unsigned binary entries cannot represent edge direction (needed
for bond graphs, Petri nets) or bond order (needed for molecular graphs). Signed
integers handle direction, weight, and algebraic orientation in a single entry.
The `domainConfig.edge_direction` field guides interpretation: in `"undirected"`
domains the signs are algebraic bookkeeping; in `"directed"` domains they encode
physical flow direction.

### 3. Domain Configuration via `.kleist` Metadata

Domain-specific routing and rendering behavior is declared in `.kleist` template
files using the `@template __domain_<name>` naming convention. The existing
`/api/templates` endpoint carries this metadata to the client without server
changes.

```kleist
@template __domain_electronics {
    pattern: "__domain_electronics()"
    category: "__domain"
    routing_mode: "orthogonal"
    junction_style: "dot"
    multi_port_strategy: "trunk_branch"
    edge_decoration: "none"
    edge_direction: "undirected"
}
```

Configuration fields:

| Field | Values | Purpose |
|-------|--------|---------|
| `routing_mode` | `"orthogonal"`, `"direct"`, `"curved"` (future) | Wire routing algorithm |
| `junction_style` | `"dot"`, `"none"`, `"bar"` | Visual marker at T-junctions |
| `multi_port_strategy` | `"trunk_branch"`, `"star"`, `"bus"` (future) | How 3+ port nets are routed |
| `edge_decoration` | `"none"`, `"arrow"`, `"half_arrow"`, `"inhibitor"` | Edge visual style |
| `edge_direction` | `"undirected"`, `"directed"` | Whether connections have directionality |

Defaults (when no `__domain_` template exists): `orthogonal`, `dot`,
`trunk_branch`, `none`, `undirected`.

### 4. Trunk+Branch Routing for Multi-Port Nets

Multi-port nets (3+ connections) use a trunk+branch algorithm instead of the
previous star topology:

1. **Pick trunk pair**: The two ports with greatest spatial distance become trunk
   endpoints, maximizing the wire length available for branches.
2. **Route trunk**: Use the standard exit-direction-aware waypoint algorithm
   (`computeDefaultWaypoints`) for the main wire.
3. **Route branches**: For each remaining port, project its position onto the
   trunk polyline to find the nearest point. Route a perpendicular stub from the
   port to that junction point, respecting the port's exit direction.
4. **Render**: Draw the trunk as a single `<path>`, each branch as a short
   `<path>`, and a junction marker (configurable via `junction_style`) at each
   branch point.

The algorithm is generic — it works for any domain using `orthogonal` routing
mode. Domains using `direct` routing get straight-line connections (no
waypoints). The `multi_port_strategy` field selects between `trunk_branch` and
`star` (centroid) strategies.

### 5. Future Domain Support Without JS Changes

New domains require only:

1. A `.kleist` file with component templates (SVGs, ports, metadata)
2. A `@template __domain_<name>` block declaring routing preferences
3. SVG assets in `static/svg/<domain>/`

No JavaScript, no Rust, no recompilation. Examples:

**Bond graphs** (`bond_graph.kleist`):
```kleist
@template __domain_bond_graph {
    routing_mode: "direct"
    junction_style: "none"
    multi_port_strategy: "star"
    edge_decoration: "half_arrow"
    edge_direction: "directed"
}
```

**Petri nets** (`petri_net.kleist`):
```kleist
@template __domain_petri_net {
    routing_mode: "orthogonal"
    junction_style: "none"
    multi_port_strategy: "trunk_branch"
    edge_decoration: "arrow"
    edge_direction: "directed"
}
```

**Molecular graphs** (`molecular.kleist`):
```kleist
@template __domain_molecular {
    routing_mode: "direct"
    junction_style: "none"
    multi_port_strategy: "star"
    edge_decoration: "none"
    edge_direction: "undirected"
}
```

### 6. Rust/WASM for Core Graph Logic

Core graph computations (incidence matrix construction, Editor AST generation)
run in Rust compiled to WebAssembly via `wasm-bindgen`/`wasm-pack`. This:

- Shares types with the Kleis Rust codebase
- Provides performance for large graphs
- Enables future integration with the type system and Z3 verification
- Falls back gracefully when WASM is unavailable

The WASM crate is `graph-editor-wasm/` with entry points `compute_incidence`
and `compute_editor_ast`.

## Implementation Status

### Phase 1 (Implemented)

- Graph Editor PoC with component placement, rotation, and deletion
- Port-based connection model with orthogonal (Manhattan) wire routing
- Exit-direction-aware waypoint generation for 2-port nets
- Trunk+branch routing for multi-port nets with junction dots
- Interactive waypoint manipulation (drag segments, add/remove waypoints)
- Collinear segment merging and auto-rerouting on component drag
- SVG `viewBox`-based pan and zoom
- Domain configuration loading from `__domain_` templates
- Routing mode dispatch (`orthogonal` vs `direct`)
- Typst schematic export with proper scaling and rotation
- Rust/WASM integration for incidence matrix and AST generation
- **Signed sparse incidence matrix** in COO format (WASM + JS fallback)
  - Entries are signed integers: +1 (first port), -1 (non-first port)
  - AST topology node changed from `Matrix(V,P,[dense])` to
    `SparseMatrix(V,P,[net,port,val,...])`
  - `renderMatrixHTML` materializes dense view from COO for display

### Phase 2 (Planned): Edge Decorations and Direction

- SVG marker definitions (arrowhead, half-arrow, causal bar)
- `marker-start`/`marker-end` attributes based on `edge_decoration`
- `direction` field on net connections for directed graphs

### Phase 3 (Planned): Bond Graph Templates

- `bond_graph.kleist` with component and junction templates
- Direct routing mode fully exercised
- Causal stroke rendering

### Phase 4 (Planned): Petri Net Templates

- `petri_net.kleist` with place/transition templates
- Token state rendering inside place SVGs
- Simulation integration with existing ODE solver

## Consequences

### Positive

1. **Level 3 graph editing is no longer deferred.** ADR-036 marked it as "out of
   scope"; this ADR delivers it.
2. **The incidence matrix representation is universal.** Electronics, bond
   graphs, Petri nets, and molecular graphs all use the same data model.
3. **Domain extensibility is data-driven.** New graph domains require only
   `.kleist` files and SVG assets. The routing engine is generic.
4. **The existing AST is unchanged.** Graphs are values inside the tree AST, not
   a competing representation. All existing Kleis infrastructure (type inference,
   Z3, evaluation) continues to work.
5. **Separation of editors prevents regressions.** The Equation Editor is
   untouched by graph editing work.

### Negative

1. **Two editors to maintain.** The Graph Editor and Equation Editor share no
   rendering code. Future changes to common patterns (e.g., template loading)
   must be applied in both places.
2. **Phase 2-4 features are not yet implemented.** Edge decorations, directed
   graphs, and non-electronics domains remain planned but unbuilt.
3. **No persistent trunk waypoints for multi-port nets.** Trunk routing is
   recomputed each time; users cannot manually adjust trunk segments of
   multi-port nets (only 2-port nets have persistent draggable waypoints).

### Neutral

1. **The `__domain_` naming convention avoids parser changes** but is less clean
   than a dedicated `@graph_domain` syntax. A future parser enhancement could
   recognize a first-class `@graph_domain` block.
2. **Pan/zoom uses SVG `viewBox`**, which is performant and well-supported but
   limits future layering (e.g., HTML overlays would not pan with the SVG).

## Files

| File | Role |
|------|------|
| `static/graph_editor.html` | Graph Editor HTML structure |
| `static/js/graphEditorMain.js` | Core editor logic: interaction, routing, rendering |
| `std_template_lib/electronics.kleist` | Electronics component templates + `__domain_electronics` config |
| `graph-editor-wasm/` | Rust/WASM crate for incidence matrix and AST generation |
| `static/svg/electronics/` | SVG assets for electronic components |

## References

- ADR-036: Multi-Domain Template Generality — established the Level 1/2/3
  classification; this ADR implements Level 3
- ADR-035: Multi-Domain Template Compiler — engine fixes for data-driven
  extensibility
- ADR-023: Template Externalization — `.kleist` file format and metadata model
- KiCad schematic architecture — studied for component/netlist model
- draw.io/diagrams.net — studied for interactive connector patterns
