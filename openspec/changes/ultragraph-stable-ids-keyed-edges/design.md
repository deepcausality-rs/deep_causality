## Context

UltraGraph 0.9.9 has two states. `DynamicGraph` holds `Vec<Option<N>>` nodes and one outgoing
`Vec<(usize, W)>` per node; `CsmGraph` holds live nodes only, in a forward and a backward CSR.
`freeze` drops tombstones and renumbers every live node by rank. `add_edge` pushes without a
duplicate check, `remove_edge` removes the first match, traversal fails in the dynamic state, and
every algorithm sizes its arrays by `number_nodes()`.

The design is `openspec/notes/ultragraph/RadixGraphUpdate.md` (§Design 1–3, 5, 6 in the part this
change takes), the per-type migration `openspec/notes/ultragraph/RadixGraphUpdateMigration.md`, and
the measured baseline `openspec/notes/ultragraph/BenchBefore/BenchBefore.md`: three runs on an
Apple M3 Max, rustc 1.99.0, with a per-row spread that is the noise floor for any comparison.

Five crates call UltraGraph: `deep_causality` (the causaloid graph and its reasoning engines),
`deep_causality_context` (the context graph), `deep_causality_ethos` (the teloid graph), and,
through `CausableGraph`, `deep_causality_quantum` (four QCM readers); `deep_causality_context_store`
declares `RelationKind`, the context graph's edge payload.

Decided by Marvin on 2026-10-15, beyond the notes: the work is cut into this change and
`ultragraph-live-state-maintained-views`; this change carries the projection view and
`ParallelEdges`; the context keeps one relation per pair (Path A); a second teloid relation is
refused; stale context index entries are dropped at removal; the QCM readers keep their frozen
precondition; the store crate's no-dependency requirement is relaxed to `ultragraph_traits`; the
work follows the TDD protocol of `openspec/specs/unified-math-tdd-protocol/spec.md`.

## Goals / Non-Goals

**Goals:**
- A position `add_node` returns names its node until removal, in every state.
- Edges identified by source, target and key; traversal in the dynamic state.
- Algorithms on a projection, with the parallel-edge mode chosen per projection.
- Every DeepCausality caller migrated in the same change as the behaviour it depends on, so the
  workspace never gives a wrong answer between commits.
- A suite written before the code, audited against named defects, and hardened by mutation testing.

**Non-Goals:**
- The Live state, `freeze_clone`, `materialize` and maintained views (the second change).
- Replacing the per-node `Vec` lists with RadixGraph's snapshot-log arrays; that decision belongs to
  the second change's benchmark.
- Typed relations between one pair in the context (Path B of the migration note).
- Versioned reads, concurrency, persistence and an external-ID index, which stay in the storage
  layer.

## Decisions

### D1. Positions are slots that only grow
`CsmGraph.nodes` becomes `Vec<Option<N>>` with a live count, so `number_nodes` stays O(1).
`freeze` maps every position to itself and keeps the whole node table, trailing tombstones and a
graph with no live node included; today's empty-graph shortcuts (`graph_freeze.rs:57-59`, `:91-93`)
go. Every algorithm sizes its arrays by a crate-internal `node_slots()`, skips tombstones, and counts
live nodes where the node count enters a result: betweenness normalisation and its
fewer-than-three-nodes case (`graph_csm_algo_centrality.rs:30`, `:59-64`).
*Alternative:* keep renumbering and return the mapping from `freeze`. Rejected: every caller would
still remap, which is the hazard this change removes.

### D2. `compact_ids` and `clear` are the only position changes
`compact_ids` lives on `DynamicGraph`; the container delegates in the dynamic state and returns
`GraphError::GraphIsFrozen` in the frozen one. `clear` keeps today's behaviour, a wipe back to an
empty dynamic graph, and restarts positions at 0.

### D3. `EdgeKey` in a crate of its own
`ultragraph_traits` declares `EdgeKey { type Key: Copy + Ord; fn edge_key(&self) -> Self::Key; }`
and its impls for `()`, `bool` and the primitive numerics. `Ord` is required because the frozen
adjacency sorts by target, then key. The trait sits below `ultragraph` so that
`deep_causality_context_store` can implement it for `RelationKind` without depending on the graph.
*Alternative:* a newtype payload in `deep_causality_context` to keep the store crate
dependency-free. Rejected by decision; the store spec is relaxed instead.

### D4. Edge storage stays per-node `Vec` lists in this change
The dynamic state gains a backward list per node beside the forward list. A keyed insert or
removal scans the source's forward list for `(target, key)` and the target's backward list for
`(source, key)`: O(d). `remove_node` walks its backward list to remove its incoming edges from each
predecessor. The second change benchmarks the snapshot-log arrays against exactly this structure.

### D5. One algorithm body, two adjacency sources
The algorithms read adjacency through a crate-internal trait with static dispatch: the frozen graph
implements it as the simple projection, and the projection view implements it with the node and
edge tests applied. Under `Merge` an algorithm skips repeated targets as it walks a node's sorted
adjacency, and a weighted path takes the lightest of them; under `Distinct` it visits every edge.
Reads and traversals list every passing edge under both modes. `build_symmetric_adjacency` keeps its
`max(forward, reverse)` multiplicity for `Distinct` and emits one undirected edge per pair for
`Merge`. Undirected betweenness deduplicates neighbours today
(`graph_csm_algo_centrality.rs:204-205`); under `Distinct` it counts each neighbour with that
multiplicity instead. The view borrows the frozen graph, so the borrow checker forbids mutation
while a view lives. *Alternative:* filter into a new CSR per projection. That is `materialize`, in the
second change.

### D6. Traversal from a tombstone is NodeNotFound
In both states, traversal checks `contains_node` first, as the frozen state does today, so a
tombstone and a position never issued give the same error.

### D7. Downstream migration in the same commits as the behaviour
- Context: the rank remap in `contextuable_graph.rs:173-207` is deleted in the commit that makes
  positions permanent; a private helper drops index-map entries naming a removed index, called from
  `remove_node` and the base-graph `release`; the `GraphNotFrozen` arm of `traversal_error` goes;
  `get_edge` and `extra_ctx_get_edge` return the lowest `RelationKind` of the pair.
- Causaloid graph: `CausableGraph::index_bound()` is a provided method over `get_last_index`; the two
  engines, `check_single_writer` and `reachable_mask` size by it; writer validity is
  `contains_causaloid`.
- Ethos: `TeloidRelation` implements `EdgeKey` with `Key = ()` through `ultragraph`'s re-export;
  `add_inheritance_edge` and `add_defeasance_edge` read the existing relation of the pair and return
  `DeonticError::ConflictingRelation { from, to }` on a conflict.
- Quantum: the four readers size by `index_bound()`, check declared indices with
  `contains_causaloid`, and keep the frozen precondition with its density rationale removed.

### D8. The TDD protocol, applied to a data structure
Each task group runs the five phases of `unified-math-tdd-protocol` in order.

- **Phase 1.** New API items land with `unimplemented!()` bodies: `ultragraph_traits::EdgeKey` and
  its impls, `compact_ids`, `remove_edge_keyed`, `contains_edge_keyed`, `Projection`,
  `ParallelEdges`, `project` and the view type, `CausableGraph::index_bound`,
  `DeonticError::ConflictingRelation`. Existing functions whose behaviour changes keep their
  current bodies; their phase-2 failure is the old behaviour failing the new assertion, recorded.
- **Phase 2.** Every scenario of the change's specs has a test before any implementing commit. The
  failing run, with its test count, goes in `openspec/notes/ultragraph/tdd/`.
- **Phase 3.** A throwaway implementation receives the defects below one at a time; each must be
  caught by a test whose subject it is.
- **Phase 4.** Implementation against the audited suite, full coverage of added and edited files,
  `cargo test` and `bazel test //...` with matching counts.
- **Phase 5.** `scripts/mutants.sh` over every added or edited source file; each survivor is killed or
  recorded in `.cargo/mutants.toml` with the measurement that settles it.

**Independent sources of expected values.** A reference model, positions to payloads and a set of
`(source, target, key)` triples to payloads, driven by the same operation sequence; naive oracles over
the model (breadth-first reachability, bridges by deleting each connection and testing
connectivity, shortest-path counts by enumerating paths in small graphs); hand derivations written
beside literals, as the betweenness scenario's 1/2 and 2/3. Random graphs and operation sequences come
from `deep_causality_rand::Xoshiro256::from_seed`, already a dev-dependency, so no property-testing
crate is added and every failure reproduces.

**Corner cases A–K for a graph.**

| # | Class | Instance in this change |
|---|---|---|
| A | Empty | Graph with no node; freeze of an empty graph; `compact_ids` on an empty graph; `index_bound` 0 |
| B | Single element | One node, no edge; one node with a self-loop; one edge |
| C | Coinciding quantities | No removal, where position equals rank and the old renumbering is invisible; no parallel edge, where `Merge` equals `Distinct`; every test must also run where they differ |
| D | Degenerate index | Removal at position 0, in the middle, at the last position, and of every node; trailing tombstones; a node whose only neighbour is a tombstone |
| E | Thresholds, both sides | Frozen `contains_edge` at degree 63, 64, 65; freeze sort at degree 127, 128, 129 |
| F | Zero | Zero edges; weight 0; `number_nodes` 0 with slots present |
| G | Negative | n/a: positions are `usize`; negative weights reach `shortest_weighted_path` only through `W: Ord`, which this change does not alter |
| H | Exact boundary | Position equal to the slot count (one past the end) beside a tombstone at the last slot; the root at the last slot |
| I | Non-finite | n/a: `f32` and `f64` payloads have `Key = ()` and are never compared by the graph |
| J | Overflow reach | n/a: no arithmetic on positions beyond `+ 1` on a slot count bounded by memory |
| K | Every payload kind | `W = ()`, a numeric weight, a kind key, and an identity key (`u128`), each through add, remove, contains, freeze and traversal |

**Named defects for phase 3**, beyond the nine generic classes: freeze renumbers by rank; `add_node`
reuses a tombstone; freeze of a graph with no live node resets the table; `number_nodes` counts slots
in the frozen state; `get_last_index` returns the slot count minus one; betweenness normalised by the slot count; keyed `add_edge` appends
instead of replacing; `remove_edge` removes one key only; `contains_edge_keyed` ignores the key; the
backward list is not updated on edge removal; `remove_node` leaves incoming edges; traversal from a
tombstone returns no edges instead of `NodeNotFound`; `Merge` and `Distinct` swapped; an edge passes
a projection with one end failing; `Merge` keeps the first parallel edge instead of the lightest;
undirected `Distinct` betweenness deduplicating neighbours; the context remap left in place; the
context's `get_edge` reading the first edge of a pair; `index_bound` returning `number_nodes`; a QCM reader range-checking with
`>= n`; the teloid conflict check omitted.

### D9. Performance is measured against the baseline, not gated by a threshold
After phase 4, `cargo bench -p ultragraph` runs three times on the same machine and is aggregated
like the baseline into `openspec/notes/ultragraph/BenchAfter/BenchAfter.md`. Every row whose mean
moves beyond the baseline row's spread is explained by a decision of this design. Expected moves:
`add_edge` and `remove_edge` on the hub (D4, an O(d) keyed scan and a second write),
`build_graph` (the backward list), `get_edges` (unchanged list, so within spread). Frozen reads and
the algorithms are expected within spread; a move there without an explanation is a defect.

## Risks / Trade-offs

- [Keyed `add_edge` costs O(d) on a hub, against 4.4 ns today at degree 1,000] → D9 measures it; the
  second change's snapshot-log decision exists to win it back.
- [The dynamic state stores every edge twice] → the frozen state already does; the benchmark records
  memory per edge in the second change.
- [Tombstones accumulate until `compact_ids`] → documented on `remove_node`; DeepCausality's callers
  rarely remove; `compact_ids` returns the mapping.
- [External crates with their own `W` stop compiling until they implement `EdgeKey`] → the commit
  message and rustdoc carry the migration; CHANGELOG.md is generated by release-plz and not edited.
- [Bridges and directed betweenness change for external callers with parallel edges] → `Distinct`
  reproduces today's results; undirected betweenness under `Distinct` counts parallel edges where
  today it deduplicates, recorded in rustdoc.
- [A projection view borrows the graph] → mutation needs the view dropped first; `materialize` in the
  second change gives an owned copy.

## Migration Plan

1. `ultragraph_traits`, the `EdgeKey` bound and every impl (store, ethos); nothing behaves
   differently; `bazel test //...` green.
2. Permanent positions, `compact_ids`, `clear`, and in the same group the context remap removal and
   index-map drops, `index_bound` and its callers, and the QCM readers.
3. Keyed edges, backward lists, dynamic traversal, the projection view and `ParallelEdges`, and in the
   same group the context and teloid edge semantics and the thirteen changed tests.
4. Mutation testing, the after-benchmark, AGENTS.md, and the store spec's Purpose line, which still
   says Tier 0.

Each group ends with its phases recorded, `bazel test //...` green, clippy clean, and a commit message
shown to the user; the user commits. Rollback is reverting the group's commit.

## Open Questions

None blocking. The names `ConflictingRelation`, `index_bound` and `project` are provisional until the
phase-1 API review.
