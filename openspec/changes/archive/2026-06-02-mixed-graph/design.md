## Context

`deep_causality_topology` already hosts general combinatorial structures beside its algebraic-topology core — `Graph<T>` (undirected) and `Hypergraph<T>` (unordered hyperedges) — each with an HKT witness (`GraphWitness`, `HypergraphWitness`), a `*Topology` trait, and a comonadic cursor. Neither models a graph with mixed directed/undirected edges. Constraint-based causal discovery needs exactly that: a CPDAG. The full rationale, the proof that a CPDAG is a *superset* edge model no existing type can express, and the literature (Andersson–Madigan–Perlman 1997; Meek 1995; Richardson–Spirtes 2002; Zhang 2008; Tetrad / causal-learn) are in `openspec/notes/mixed-graph.md`.

Repo constraints carried in: no external numeric crates, `unsafe_code = "forbid"`, static dispatch (no `dyn`), one-type-one-module layout, full test coverage of new code, and the two writing guides for prose.

## Goals / Non-Goals

**Goals:**
- A `MixedGraph` type whose edges carry a **mark at each endpoint** (`Tail`/`Arrow`/`Circle`), expressing undirected, directed, bidirected, and partially-directed edges under one type — hence DAG, PDAG, CPDAG, MAG, and PAG.
- A single source of truth that makes the **endpoint invariant** (one state per node pair, symmetric consistent marks) a local, checkable property.
- Structural queries the causal algorithms need: adjacency, parents/children (arc projection), undirected neighbors, edge enumeration by kind, and arc-projection acyclicity / topological order.
- Integration with the crate's HKT family (`MixedGraphWitness`, `MixedGraphTopology`).
- A reusable primitive: future PC/GES/FCI work builds on it, not just BRCD.

**Non-Goals:**
- Causal operations (Meek orientation, unshielded-collider validity, MEC sizing) — those live in `deep_causality_algorithms` and consume this type, in a later change.
- The FCI/PAG *learning algorithm* — this change delivers the PAG-capable graph *structure* and its endpoint calculus, not the algorithm that learns a PAG.
- Large-graph performance engineering (`ultragraph` remains the tool for that).

## Decisions

**D1. Full three-mark typed-endpoint model, all marks implemented (CPDAG + PAG).**
Each edge has two endpoints, each marked `Tail`, `Arrow`, or `Circle`. An edge is the pair `(mark at u, mark at v)`, and absence of an entry means no edge. The mark pairs name every edge kind in the literature:

| Endpoint pair | Edge | Used by |
| --- | --- | --- |
| `(Tail, Arrow)` | directed `u → v` | DAG, CPDAG, PAG |
| `(Tail, Tail)` | undirected `u — v` | CPDAG |
| `(Arrow, Arrow)` | bidirected `u ↔ v` | MAG, PAG (latent confounding) |
| `(Circle, Arrow)` | partially directed `u ∘→ v` | PAG |
| `(Circle, Circle)` | nondirected `u ∘—∘ v` | PAG |
| `(Circle, Tail)` | partially undirected `u ∘— v` | PAG |

This is the Tetrad/causal-learn `Endpoint` model and subsumes DAG/CPDAG/MAG/PAG. **All three marks and all combinations are implemented and tested in this change** — the owner's call: PAG support is built now rather than retrofitted, since "add later" tends to arrive sooner than planned and the marginal cost over a two-mark model is small (the storage and accessors are mark-agnostic; only the orientation/query helpers gain Circle-aware arms). *Alternative:* a two-mark `{Tail, Arrow}` model sufficient for CPDAGs today — rejected per the owner; building the full calculus once avoids a later breaking migration.

**D2. Structural by default, optional node payload.**
`MixedGraph<T = ()>`: scalar-free over `usize` indices by default (a CPDAG holds no scalars), with an optional node payload `T` to match the `Graph<T>`/`Hypergraph<T>` family. Edges carry no weight. *Alternative:* mandatory payload like the existing family — rejected; it would force a meaningless `T` on the common structural use, the same category error avoided when placing the stats primitives.

**D3. Canonical-pair edge map as the single source of truth.**
Store edges in one `BTreeMap<(usize, usize), Edge>` keyed by the canonical unordered pair `(min(u, v), max(u, v))`, where `Edge` records the endpoint marks at the lower- and higher-indexed node. There is exactly **one entry per unordered pair**, so the invariant — at most one edge per pair, with consistent endpoint marks — is enforced structurally by the map key and cannot be represented otherwise. Memory is `O(m)` in the number of edges. An order-agnostic accessor hides the canonicalization (`u < v` reads the marks directly, otherwise swaps). Orientation and mark changes rewrite a single `Edge`.

*Alternatives considered:* (a) a dense `n × n` mark matrix — `O(n²)`, and it stores two cells per pair that must be kept in agreement; the edge map is leaner (≈1500 entries vs ≈500K cells at the paper's largest `n ≈ 1000`, `m ≈ 1.5n`) and has a *stronger* invariant (one cell vs two). (b) Sparse per-node adjacency sets (`parents`/`children`/`undirected`) — spreads the invariant across three structures that must be kept consistent; rejected for that reason. If neighbor iteration ever profiles hot, a per-node adjacency index can be added as a **derived cache** rebuilt on mutation — never a second source of truth; for v1 at causal-graph sizes the `O(m)` scan suffices.

**D4. Full HKT + comonad parity with `Graph`/`Hypergraph` in v1.**
Add `MixedGraphWitness` (HKT), a `MixedGraphTopology` trait, **and** a comonadic cursor with a full `Comonad` instance (`extract`/`extend`/`duplicate`), mirroring `Graph`/`Hypergraph` exactly. The graph carries node payload data (`CausalTensor<T>`) and a `cursor: usize` focus; `extract` returns the focused node's payload, `extend`/`duplicate` map a neighborhood-aware function over every focus. This keeps `MixedGraph` a first-class member of the crate's higher-kinded-type unification rather than a partial citizen. *Alternative:* witness + topology trait only, comonad later — rejected per the owner; parity now avoids a second pass over the type and matches the established pattern.

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

All three v1 design questions are **resolved** (owner decisions):
- **HKT scope (D4):** full comonad parity now — witness + topology trait + `Comonad` instance.
- **Storage (D3):** canonical-pair edge map (`O(m)`, single source of truth), not dense or 3-Vec sparse.
- **Endpoint vocabulary (D1):** full three-mark `{Tail, Arrow, Circle}` implemented now for CPDAG **and** PAG support, not Circle-reserved.
