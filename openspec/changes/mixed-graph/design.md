## Context

`deep_causality_topology` already hosts general combinatorial structures beside its algebraic-topology core — `Graph<T>` (undirected) and `Hypergraph<T>` (unordered hyperedges) — each with an HKT witness (`GraphWitness`, `HypergraphWitness`), a `*Topology` trait, and a comonadic cursor. Neither models a graph with mixed directed/undirected edges. Constraint-based causal discovery needs exactly that: a CPDAG. The full rationale, the proof that a CPDAG is a *superset* edge model no existing type can express, and the literature (Andersson–Madigan–Perlman 1997; Meek 1995; Richardson–Spirtes 2002; Zhang 2008; Tetrad / causal-learn) are in `openspec/notes/mixed-graph.md`.

Repo constraints carried in: no external numeric crates, `unsafe_code = "forbid"`, static dispatch (no `dyn`), one-type-one-module layout, full test coverage of new code, and the two writing guides for prose.

## Goals / Non-Goals

**Goals:**
- A `MixedGraph` type whose edges carry a **mark at each endpoint**, expressing undirected, directed, and partially-directed edges under one type — hence DAG, PDAG, CPDAG.
- A single source of truth that makes the **endpoint invariant** (one state per node pair, symmetric consistent marks) a local, checkable property.
- Structural queries the causal algorithms need: adjacency, parents/children (arc projection), undirected neighbors, edge enumeration by kind, and arc-projection acyclicity / topological order.
- Integration with the crate's HKT family (`MixedGraphWitness`, `MixedGraphTopology`).
- A reusable primitive: future PC/GES/FCI work builds on it, not just BRCD.

**Non-Goals:**
- The `Circle` endpoint mark's *semantics* and any MAG/PAG-specific behavior — the variant is reserved but not exercised.
- Causal operations (Meek orientation, unshielded-collider validity, MEC sizing) — those live in `deep_causality_algorithms` and consume this type, in a later change.
- A full comonad instance (see D4).
- Large-graph performance engineering (`ultragraph` remains the tool for that).

## Decisions

**D1. Typed-endpoint model, three marks declared, two implemented.**
Each edge has two endpoints, each marked `Tail`, `Arrow`, or `Circle`. An edge is the pair `(mark at u, mark at v)`: `(Tail, Arrow)` = `u → v`; `(Tail, Tail)` = `u — v`; `(Empty, Empty)` = no edge. This is the Tetrad/causal-learn `Endpoint` model and subsumes DAG/CPDAG/MAG/PAG. We implement and test the `Tail`/`Arrow` combinations (sufficient for DAG/CPDAG today) and **reserve `Circle`** so adding PAGs later needs no breaking change. *Alternative:* a two-mark enum sufficient for CPDAGs only — rejected because the reserved third mark is one enum variant and avoids a future migration.

**D2. Structural by default, optional node payload.**
`MixedGraph<T = ()>`: scalar-free over `usize` indices by default (a CPDAG holds no scalars), with an optional node payload `T` to match the `Graph<T>`/`Hypergraph<T>` family. Edges carry no weight. *Alternative:* mandatory payload like the existing family — rejected; it would force a meaningless `T` on the common structural use, the same category error avoided when placing the stats primitives.

**D3. Dense `n × n` endpoint-mark matrix as the single source of truth.**
Store marks in an `n × n` structure; the edge between `u` and `v` is read from the two symmetric cells. Mutation updates both cells together, so the invariant is a one-line symmetry check and cannot drift. *Alternative:* sparse per-node adjacency sets (`parents`/`children`/`undirected`) — more idiomatic and O(log), but spreads the invariant across three structures that must be kept consistent. For causal graphs (tens–hundreds of nodes) the dense matrix is simplest and safest; revisit only if a large-graph consumer appears.

**D4. HKT witness and topology trait in v1; comonad deferred.**
Add `MixedGraphWitness` (HKT) and a `MixedGraphTopology` trait mirroring `GraphTopology`/`HypergraphTopology`, so the type joins the crate's higher-kinded-type unification. A comonadic cursor + `Comonad` instance is deferred to a follow-up unless it falls out trivially — it is not needed by any planned consumer and would widen v1. *Alternative:* full comonad parity with `Graph`/`Hypergraph` now — deferred to keep v1 focused on the structure and its invariant.

**D5. Self-contained arc-projection acyclicity (Kahn), no `ultragraph` dependency.**
Topological sort and cycle detection over the arc projection use Kahn's algorithm implemented in the type (~20 lines). *Alternative:* materialize an `ultragraph` on demand and delegate — rejected; it reintroduces a freeze/thaw round-trip and a cross-crate dependency to save a trivial, exhaustively-testable algorithm. `ultragraph` stays the right tool for large directed graphs elsewhere.

**D6. Invariant enforced at the mutation boundary.**
All edge mutations go through methods that set both endpoint marks together; constructors validate inputs. There is no way to leave the graph in a half-set state. Orientation (`u — v` → `u → v`) is a mark flip on the two cells.

## Risks / Trade-offs

- **Reimplementing graph algorithms already in `ultragraph`.** → Scope is tiny (Kahn topo-sort + cycle detection over the arc projection); exhaustively unit-tested. The mixed-edge semantics are new regardless, so no full reuse was available.
- **Dense `n²` storage.** → Acceptable for causal-graph sizes; documented, with a sparse-representation escape hatch noted if a large consumer ever appears.
- **A third structure type in `topology` widens its surface.** → It joins an established family (`Graph`/`Hypergraph`) and reuses the same module layout and HKT pattern, so the marginal complexity is low.
- **Reserved-but-unimplemented `Circle`.** → Clearly documented as unimplemented; methods that cannot yet handle it return a documented error rather than silently mis-handling it.

## Migration Plan

This is additive — no existing type changes, nothing to roll back. Sequencing with the BRCD work:
1. Land `mixed-graph` (this change) in `deep_causality_topology`.
2. Revise `brcd-prep-foundations` decision **D2**: its causal-graph *operations* (Meek, unshielded-collider validity, MEC sizing) move to `deep_causality_algorithms` consuming this `MixedGraph`, replacing the plan to define a graph type there. Update that change's proposal/design/`causal-graph` spec/tasks.
3. `brcd-prep-foundations` Tier A (cg_solve lift, tensor stats, parents accessor) is already complete and unaffected.

## Open Questions

- Confirm the v1 HKT scope (D4): witness + topology trait now, comonad later — or full comonad parity now.
- Confirm dense storage (D3) is acceptable, or prefer sparse adjacency from the start.
- Confirm `Circle` is reserved-only in v1 (D1).
