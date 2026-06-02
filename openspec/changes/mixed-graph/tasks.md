## 1. Type and module scaffold

- [ ] 1.1 Create `deep_causality_topology/src/types/mixed_graph/` with a folder-based module layout mirroring `graph` (`mod.rs`, `constructors`, `getters`, `mixed_graph_ops`, `api`, `clone`, `display`, `topology`).
- [ ] 1.2 Define the `Mark` enum (`Tail`, `Arrow`, `Circle`) and an `Edge` value holding the marks at the lower- and higher-indexed endpoints. Define the `MixedGraph<T>` struct: node count, the canonical-pair edge map `BTreeMap<(usize, usize), Edge>` (single source of truth, key = `(min, max)`), node payload `CausalTensor<T>`, and the comonadic `cursor: usize`. Document the canonical-pair encoding and the order-agnostic accessor.
- [ ] 1.3 Register the module in `types/mod.rs` and export `MixedGraph`, `Mark`, and `Edge` from `lib.rs`. Confirm the crate builds.

## 2. Constructors and getters

- [ ] 2.1 Add constructors: a graph of `n` nodes with no edges, and construction with node-payload data. Validate inputs.
- [ ] 2.2 Add getters: node count; counts by edge kind (directed, undirected, bidirected, partially directed, nondirected, partially undirected); the order-agnostic endpoint-mark accessor; and edge lookup. Implement `Clone`/`Debug`/`PartialEq` and a `Display` consistent with the crate's style.
- [ ] 2.3 Tests: construction, per-kind counts, the order-agnostic accessor, and the structural (`T = ()`) plus payload forms. Build + test + clippy + fmt green.

## 3. Edge mutation and the endpoint invariant

- [ ] 3.1 Implement edge constructors for every kind — `add_arc`, `add_undirected`, `add_bidirected`, and a general `add_edge(u, v, mark_u, mark_v)` — each writing one canonical `Edge`; reject out-of-range indices and an already-occupied pair per the spec. Implement `remove_edge(u, v)`.
- [ ] 3.2 Implement endpoint orientation: `set_endpoint(u, v, at, mark)` to set/reorient any endpoint to any `Mark` (the full PAG calculus), plus a convenience `orient(u, v)` for `u — v → u → v`. Setting an endpoint of a non-existent edge returns a documented error.
- [ ] 3.3 Add an internal invariant check (single entry per canonical pair; well-formed `Edge`) used in tests/asserts.
- [ ] 3.4 Tests: every endpoint-mark combination representable; circle→arrow orientation; make-bidirected; at-most-one-edge-per-pair; out-of-range rejection; set-endpoint-of-missing-edge rejection; order-agnostic reads after mutation. Build + test + clippy + fmt green.

## 4. Structural queries

- [ ] 4.1 Implement `parents(v)` and `children(v)` over the **directed-arc projection** (`(Tail, Arrow)` only), `undirected_neighbors(v)`, `is_adjacent(u, v)` (any edge kind), and enumeration grouped by edge kind, by scanning the edge map.
- [ ] 4.2 Tests: parents vs. undirected neighbors reported separately; adjacency spans all edge kinds; per-kind enumeration correctness. Build + test + clippy + fmt green.

## 5. Arc-projection acyclicity and topological order

- [ ] 5.1 Implement a self-contained Kahn topological sort over the arc projection (ignoring undirected edges) returning the order or signaling a cycle; implement `has_cycle` / `find_cycle` over the arc projection.
- [ ] 5.2 Tests: acyclic projection yields a valid order (every arc earlier→later), cyclic projection detected, undirected edges do not affect acyclicity. Build + test + clippy + fmt green.

## 6. HKT, topology-trait, and comonad integration

- [ ] 6.1 Add a `MixedGraphWitness` HKT witness under `extensions/`, mirroring `GraphWitness`/`HypergraphWitness`; register and export it.
- [ ] 6.2 Add a `MixedGraphTopology` trait under `traits/` analogous to `GraphTopology`/`HypergraphTopology`; implement it for `MixedGraph` and export it.
- [ ] 6.3 Implement the comonadic interface to full parity with `Graph`/`Hypergraph`: `extract` (focused node's payload), `extend`/`duplicate` (neighborhood-aware map over every focus), driven by the `cursor`.
- [ ] 6.4 Tests: witness + topology trait compose as the other graph types do; `extract` returns the focused payload; `extend`/`duplicate` satisfy the comonad laws (left identity, right identity, associativity). Build + test + clippy + fmt green.

## 7. PAG edge-kind completeness

- [ ] 7.1 Classify every edge by kind (directed, undirected, bidirected, partially directed, nondirected, partially undirected) from its endpoint marks; expose the classifier and ensure orientation across kinds (e.g. `∘→` to `→`, `—` to `↔`) preserves the invariant.
- [ ] 7.2 Tests: round-trip every edge kind through construction, classification, reorientation, and removal; confirm the full PAG calculus is expressible and mutation-stable. Build + test + clippy + fmt green.

## 8. Test registration and hygiene

- [ ] 8.1 Place all new tests under `deep_causality_topology/tests/` mirroring the source layout; register them in the module tree and in `tests/BUILD.bazel`.
- [ ] 8.2 Confirm no external numeric crate was added, `unsafe_code = "forbid"` is intact, no `dyn` was introduced, and new code has full test coverage.

## 9. Verification and handoff

- [ ] 9.1 `cargo build -p deep_causality_topology`, `cargo test -p deep_causality_topology`, `cargo clippy -p deep_causality_topology --all-targets`, `cargo fmt -p deep_causality_topology --check` all clean; then the full-workspace `make build` / `make test`.
- [ ] 9.2 Run `openspec validate mixed-graph` and confirm apply-complete; prepare a commit message and request the owner commit (do not commit).
- [ ] 9.3 After this lands, revise `brcd-prep-foundations` (decision D2 and tasks 2.2–2.7) so its causal-graph operations consume this `MixedGraph` instead of defining a graph type in `deep_causality_algorithms`.
