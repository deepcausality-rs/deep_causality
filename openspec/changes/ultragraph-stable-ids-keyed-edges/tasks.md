## 0. Ground rules for every group

Each group below runs the five phases of `openspec/specs/unified-math-tdd-protocol/spec.md` in order
and does not start a phase before the previous one's exit holds (design D8). Each group's records go
in `openspec/notes/ultragraph/tdd/group-<n>.md`: the A–K corner-case table with a test named per row
(or n/a and why), the phase-2 failing run with its test count, the phase-3 defect audit, and the
phase-5 mutation report. Tests mirror `src` file for file under `tests/`, are registered upward
through `mod.rs`, and are declared to Bazel. Each group ends with `bazel test //...` green, clippy
clean with no `allow`, and a commit message shown to the user; the user commits.

Phase 4 lands one crate per commit, in dependency order, wherever the workspace stays green between
commits; each commit carries the tests it makes pass, and work stops for the user after each. Two
pairs cannot be split: in group 2, `ultragraph`'s permanent positions and the context's remap
removal (the remap would corrupt every stored index once `freeze` stops renumbering); in group 3,
`ultragraph`'s keyed edges and dynamic traversal with the caller tests of task 3.4 that pin the old
behaviour.

- [x] 0.1 Create `openspec/notes/ultragraph/tdd/` with one record file per group, each carrying the
      empty A–K table from design D8

## 1. ultragraph_traits and the EdgeKey bound

- [x] 1.1 Phase 1: add the crate `ultragraph_traits` at the repository root (no `[dependencies]`,
      `[lints] workspace = true`, `no_std`, the crates.io metadata the other crates carry), add it to
      `[workspace.dependencies]` with a two-digit version (`0.1`) that members inherit, declare `EdgeKey` and its impls for `()`, `bool` and the primitive numerics with
      `unimplemented!()` bodies, add it to the workspace members and `BUILD.bazel`, and re-export
      `EdgeKey` from `ultragraph`; `cargo build` and `bazel build` succeed
- [x] 1.2 Phase 2: write `ultragraph_traits/tests` for every scenario of
      `ultragraph-keyed-edges` "ultragraph_traits holds EdgeKey and depends on nothing" and "The unit
      and primitive numeric types take an empty key" (impl coverage only; graph behaviour is group 3),
      plus the `context-store-crate-identity` scenarios on `RelationKind`; run them, observe the
      `unimplemented!()` failures, record the run and count
- [x] 1.3 Phase 3: audit the suite against a throwaway implementation (a missing impl, a non-unit key
      for a numeric type where it compiles, `RelationKind::edge_key` returning a fixed variant, the
      order reversed); record which test caught each
- [x] 1.4 Phase 4: implement the impls; add the `W: EdgeKey` bound to `UltraGraphContainer`,
      `GraphState`, `DynamicGraph` and `CsmGraph`; give `deep_causality_context_store` its dependency
      on `ultragraph_traits` (Cargo, Bazel), implement `PartialOrd` and `Ord` on `RelationKind` in declaration order and
      implement `EdgeKey` for it; implement `EdgeKey` for `TeloidRelation` with `Key = ()` through
      `ultragraph::EdgeKey`
- [x] 1.5 Confirm nothing behaves differently: every existing suite of the five calling crates passes
      unchanged under `cargo test` and `bazel test //...`, and the executed counts match
- [x] 1.6 Phase 5: run `scripts/mutants.sh` over the new and edited files; resolve every survivor
- [x] 1.7 Update `AGENTS.md`: the crate count (34), the root crates (thirteen, and the list of those
      that stayed at the root), the tier block (`ultragraph_traits` Tier 0;
      `deep_causality_context_store` and `ultragraph` Tier 1), the store crate's description line,
      the count of crates without external runtime dependencies (27), and the count of workspace
      members opting into the lints (49)
- [x] 1.8 `make format && make fix`, `make check_no_std`, `make check_publishable`,
      `bazel test //...`, commit message for group 1

## 2. Permanent positions, and every caller that reads an index

- [x] 2.1 Phase 1: declare `compact_ids` on `DynamicGraph` and `UltraGraphContainer` and the provided
      method `CausableGraph::index_bound`, each with an `unimplemented!()` body
- [x] 2.2 Fill the group-2 A–K table (rows A–F, H, K of design D8 apply) before writing tests
- [x] 2.3 Phase 2, UltraGraph: write a test for every scenario of `ultragraph-stable-node-ids`; add a
      model-based test driving seeded random sequences of `add_node`, `remove_node`, `add_edge`,
      `freeze`, `unfreeze`, `compact_ids` and `clear` against a reference model of positions; add the
      generated agreement test between a graph with tombstones and its `compact_ids` image for every
      algorithm
- [x] 2.4 Phase 2, rewrite the existing tests that pin renumbering before touching the code:
      `graph_evolve_tests.rs:91` and `:326`; `contextuable_graph_tests.rs:332` (indices stay; stale
      entries gone at removal); `hypothesis_tests.rs:161` and `config_tests.rs:96-97` (the rewritten
      range messages); and the comment in `context_tests.rs`
      `test_get_data_by_id_reads_the_payload_of_the_node_carrying_the_id`
- [x] 2.5 Phase 2, callers: write tests for every scenario of `causaloid-graph-index-bound` except
      "A repeated edge between two causaloids updates its weight" and "The check examines a dynamic
      graph" (group 3: the first needs keyed edges, the second dynamic traversal, without which
      `check_single_writer` skips every join and returns `Ok(())`), of
      `context-graph-stable-indices` "Freezing and unfreezing keep every index the context stores" and
      "Index entries naming a removed node are dropped at removal", and of the modified `qcl-hypothesis`
      and `qcl-pipeline` requirements; cover `FactorSupports::from_graph`,
      `CausalStructure::from_graph_reachability`, `DemModel::from_graph`, the model-subject validation
      and `graph_has_cycle` each with a removed index below a live node that carries a factor, role,
      edge or cycle
- [x] 2.6 Phase 2 exit: run every group-2 test against the unchanged implementation; each fails with
      the `unimplemented!()` panic or with the old behaviour's wrong answer, never a compile error;
      record the run and the count
- [x] 2.7 Phase 3: on a throwaway implementation inject, one at a time, the design D8 defects of this
      group (freeze renumbers; `add_node` reuses a tombstone; freeze of a graph with no live node
      resets the table; frozen `number_nodes` counts slots; `get_last_index` returns slots minus one;
      betweenness normalised by the slot count; traversal from a tombstone returns no edges; the context remap left in place; index-map entries
      kept at removal; `index_bound` returning `number_nodes`; a QCM reader checking `>= n`) and the
      nine generic classes; confirm each is caught by a test whose subject it is; widen the suite where
      not; record; discard the throwaway
- [x] 2.8 Phase 4, `ultragraph`: `CsmGraph.nodes` as `Vec<Option<N>>` with a live count; `freeze` and
      `unfreeze` keep every position and the whole node table; remove the empty-graph shortcuts; every
      algorithm sizes by `node_slots()` and skips tombstones; traversal checks `contains_node`;
      `compact_ids`; `clear` restarting at 0; rustdoc of every changed item states the new behaviour
- [x] 2.9 Phase 4, `deep_causality`: `index_bound`; size the two reasoning engines,
      `check_single_writer` and `reachable_mask` by it; validate writers with `contains_causaloid`
- [x] 2.10 Phase 4, `deep_causality_context`: delete the rank remap in `contextuable_graph.rs`; add the
      helper dropping index-map entries and call it from `remove_node` and the base-graph `release`;
      rewrite the `freeze` doc in `traits/contextuable_graph/mod.rs`
- [x] 2.11 Phase 4, `deep_causality_quantum`: the four readers and `graph_has_cycle` size by
      `index_bound()`, check declared indices with `contains_causaloid`, and keep the frozen
      precondition with the density rationale removed from docs and errors
- [x] 2.12 Phase 4 exit: every group-2 test green; full line coverage of added and edited files;
      `cargo test` and `bazel test //...` counts match for every affected crate
- [x] 2.13 Phase 5: `scripts/mutants.sh` over the group's edited files in the four crates; kill or
      record every survivor in `.cargo/mutants.toml` with its measurement and `comm` check
- [x] 2.14 `make format && make fix`, `make build`, `bazel test //...`, commit message for group 2

## 3. Keyed edges, dynamic traversal and the projection view

- [x] 3.1 Phase 1: declare `remove_edge_keyed` and `contains_edge_keyed` on `GraphMut` and `GraphView`,
      the `Projection` trait and its impl for closure pairs, `ParallelEdges`, `project` and its view
      type implementing `GraphAlgorithms` and `GraphTraversal`, and `DeonticError::ConflictingRelation`,
      each with an `unimplemented!()` body
- [x] 3.2 Fill the group-3 A–K table (rows A–F, H, K of design D8 apply; E is the 63/64/65 and
      127/128/129 thresholds; K is the four payload kinds)
- [x] 3.3 Phase 2, UltraGraph: write a test for every scenario of `ultragraph-keyed-edges`,
      `ultragraph-dynamic-traversal` and `ultragraph-projection-view`; extend the model-based test with
      keyed `add_edge`, `remove_edge`, `remove_edge_keyed` and both directions; add independent oracles
      for bridges (delete each connection, test connectivity), betweenness (enumerate shortest paths on
      graphs of at most eight nodes) and projections (build the passing subgraph by a separate filter)
- [x] 3.4 Phase 2, rewrite the existing tests whose behaviour changes: `graph_evolve_tests.rs:44`
      (one edge, weight 15); `graph_traversal_tests.rs:26` and `:64` (traversal works dynamic);
      `graph_algo_bridges_tests.rs:71` and `:80` (keyed payload under `Distinct`, plus `Merge`
      counterparts); `deep_causality/src/traits/causable_graph/mod.rs:147` (reach on a dynamic graph);
      `contextuable_graph_tests.rs:270` (dynamic listing equals frozen); `snapshot_tests.rs:246`
      (`remove_edge` removes both kinds)
- [x] 3.5 Phase 2, callers: write tests for every scenario of `teloid-graph-relations`, of
      `context-graph-stable-indices` "Neighbour listings work without freezing", "The context holds
      one relation per pair through the store" and "A pair's relation reads the same in every state
      and context", and of `causaloid-graph-index-bound` "A repeated edge between two causaloids
      updates its weight" and "The check examines a dynamic graph"
- [x] 3.6 Phase 2 exit: run every group-3 test against the unchanged implementation; record the run
      and the count
- [x] 3.7 Phase 3: inject the design D8 edge defects (keyed `add_edge` appends; `remove_edge` removes
      one key; `contains_edge_keyed` ignores the key; the backward list not updated on removal;
      `remove_node` leaves incoming edges; `Merge` and `Distinct` swapped; an edge passing with one end
      failing; `Merge` keeping the first parallel edge instead of the lightest; undirected `Distinct`
      betweenness deduplicating neighbours; the context's `get_edge` reading the first edge of a pair;
      the teloid conflict check omitted) and the nine generic classes; record; discard the throwaway
- [x] 3.8 Phase 4, `ultragraph`: forward and backward lists in `DynamicGraph`; keyed add, remove and
      contains in both directions; `remove_node` removing incoming edges; live `number_edges`; frozen
      adjacency sorted by target then key in both CSRs; dynamic traversal through the container; the
      crate-internal adjacency trait with the frozen graph as the simple projection and the view with
      its tests applied; `Merge` and `Distinct` in every algorithm, in `build_symmetric_adjacency` and in undirected
      Brandes; reads and traversals of a view listing every passing edge;
      correct the module doc of `graph_csm_biconnectivity_common.rs`, whose claim about biconnected
      components does not hold for vertex sets
- [x] 3.9 Phase 4, callers: remove the `GraphNotFrozen` arm of the context's `traversal_error` and
      rewrite the traversal docs in `traits/contextuable_graph/mod.rs`; make `get_edge` and
      `extra_ctx_get_edge` return the lowest `RelationKind` of the pair; implement the teloid conflict
      check and `ConflictingRelation` (`Display`, `source`); rewrite the `reachable_mask` doc on
      dynamic graphs
- [x] 3.10 Phase 4 exit: every group-3 test green; full coverage of added and edited files; matching
      `cargo test` and `bazel test //...` counts
- [x] 3.11 Phase 5: `scripts/mutants.sh` over the group's edited files; resolve every survivor
- [x] 3.12 `make format && make fix`, `make build`, `bazel test //...`, commit message for group 3

## 4. Measurement and close-out

- [x] 4.1 Run `cargo bench -p ultragraph` three times on the baseline machine; aggregate into
      `openspec/notes/ultragraph/BenchAfter/BenchAfter.md` with the baseline's method and table layout
- [x] 4.2 Compare every row with `BenchBefore.md`; explain each mean that moves beyond the baseline
      row's spread by a decision of design D9, and treat an unexplained move in a frozen read or an
      algorithm as a defect to fix before close-out
- [x] 4.3 Run the full workspace: `make format && make fix`, `make build`, `bazel test //...`,
      `make check_examples`
- [x] 4.4 Note for archive: the `context-store-crate-identity` Purpose line still says Tier 0 and
      dependency-free; update it when the change is archived
- [x] 4.5 Commit message for group 4, with the migration notes for external callers (the `EdgeKey`
      bound, permanent positions, keyed edge semantics, `Merge` as the default) in its body
