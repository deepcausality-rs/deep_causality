## 1. Type and module scaffold

- [ ] 1.1 Create `deep_causality_topology/src/types/mixed_graph/` with a folder-based module layout mirroring `graph` (`mod.rs`, `constructors`, `getters`, `mixed_graph_ops`, `api`, `clone`, `display`, `topology`).
- [ ] 1.2 Define the `Mark` enum (`Empty`, `Tail`, `Arrow`, `Circle`) and the `MixedGraph<T = ()>` struct holding the node count, the dense `n × n` endpoint-mark store (single source of truth), and the optional node payload. Document the edge encoding `(mark[u][v], mark[v][u])`.
- [ ] 1.3 Register the module in `types/mod.rs` and export `MixedGraph` (and `Mark`) from `lib.rs`. Confirm the crate builds.

## 2. Constructors and getters

- [ ] 2.1 Add constructors: a graph of `n` nodes with no edges, and (for the payload form) construction with node data. Validate inputs.
- [ ] 2.2 Add getters: node count, arc count, undirected-edge count, and accessors over the mark store. Implement `Clone`/`Debug`/`PartialEq` and a `Display` consistent with the crate's style.
- [ ] 2.3 Tests: construction, counts, and the scalar-free default (`T = ()`) plus the payload form. Build + test + clippy + fmt green.

## 3. Edge mutation and the endpoint invariant

- [ ] 3.1 Implement `add_arc(u, v)` and `add_undirected(u, v)` that set both endpoint cells together; reject out-of-range indices and an already-occupied pair per the spec. Implement `remove_edge(u, v)`.
- [ ] 3.2 Implement `orient(u, v)` turning an undirected `u — v` into `u → v`; return a documented error if the pair is not undirected.
- [ ] 3.3 Add an internal invariant check (symmetric, consistent marks) used in tests/asserts.
- [ ] 3.4 Tests: arc vs. undirected distinguishable, at-most-one-edge-per-pair, out-of-range rejection, orientation success and rejection, mark symmetry after every mutation. Build + test + clippy + fmt green.

## 4. Structural queries

- [ ] 4.1 Implement `parents(v)`, `children(v)`, `undirected_neighbors(v)`, `is_adjacent(u, v)`, and enumeration of all arcs and all undirected edges, by scanning the mark store.
- [ ] 4.2 Tests: parents vs. neighbors reported separately, adjacency across both edge kinds, enumeration correctness. Build + test + clippy + fmt green.

## 5. Arc-projection acyclicity and topological order

- [ ] 5.1 Implement a self-contained Kahn topological sort over the arc projection (ignoring undirected edges) returning the order or signaling a cycle; implement `has_cycle` / `find_cycle` over the arc projection.
- [ ] 5.2 Tests: acyclic projection yields a valid order (every arc earlier→later), cyclic projection detected, undirected edges do not affect acyclicity. Build + test + clippy + fmt green.

## 6. HKT and topology-trait integration

- [ ] 6.1 Add a `MixedGraphWitness` HKT witness under `extensions/`, mirroring `GraphWitness`/`HypergraphWitness`; register and export it.
- [ ] 6.2 Add a `MixedGraphTopology` trait under `traits/` analogous to `GraphTopology`/`HypergraphTopology`; implement it for `MixedGraph` and export it.
- [ ] 6.3 Tests: the witness and topology trait compose as the other graph types do. Build + test + clippy + fmt green.

## 7. Reserved Circle endpoint

- [ ] 7.1 Ensure every operation that would need to interpret a `Circle` mark returns a documented "unsupported" error rather than treating it as tail/arrow; the `Tail`/`Arrow` paths are fully implemented.
- [ ] 7.2 Tests: a `Circle` mark triggers the documented error on the relevant operations. Build + test + clippy + fmt green.

## 8. Test registration and hygiene

- [ ] 8.1 Place all new tests under `deep_causality_topology/tests/` mirroring the source layout; register them in the module tree and in `tests/BUILD.bazel`.
- [ ] 8.2 Confirm no external numeric crate was added, `unsafe_code = "forbid"` is intact, no `dyn` was introduced, and new code has full test coverage.

## 9. Verification and handoff

- [ ] 9.1 `cargo build -p deep_causality_topology`, `cargo test -p deep_causality_topology`, `cargo clippy -p deep_causality_topology --all-targets`, `cargo fmt -p deep_causality_topology --check` all clean; then the full-workspace `make build` / `make test`.
- [ ] 9.2 Run `openspec validate mixed-graph` and confirm apply-complete; prepare a commit message and request the owner commit (do not commit).
- [ ] 9.3 After this lands, revise `brcd-prep-foundations` (decision D2 and tasks 2.2–2.7) so its causal-graph operations consume this `MixedGraph` instead of defining a graph type in `deep_causality_algorithms`.
