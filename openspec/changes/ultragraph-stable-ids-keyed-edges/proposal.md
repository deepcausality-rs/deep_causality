## Why

UltraGraph renumbers every live node when it freezes a graph that has had a node removed, so an
index a caller kept from `add_node` names a different node, or none, afterwards. The context graph
repairs this with a rank remap on every freeze, the causaloid graph's reasoning engines and the QCM
readers in `deep_causality_quantum` assume dense frozen positions, and none of them can traverse a
graph without freezing it first. Edges carry no identity beyond their position in a list, so two
`add_edge` calls on one pair store two edges, `remove_edge` removes whichever it finds first, and a
relation kind cannot identify an edge. The design is in
`openspec/notes/ultragraph/RadixGraphUpdate.md`, the per-type migration in
`openspec/notes/ultragraph/RadixGraphUpdateMigration.md`, and the benchmark baseline the work is
measured against in `openspec/notes/ultragraph/BenchBefore/BenchBefore.md`.

This is the first of two changes. It delivers what DeepCausality's callers need. The second,
`ultragraph-live-state-maintained-views`, adds the Live state, `freeze_clone`, materialised and
maintained projections, and the decision on RadixGraph's snapshot-log edge storage.

## What Changes

- **BREAKING** Node positions are permanent. `remove_node` leaves a tombstone whose position is
  never issued again; `freeze` and `unfreeze` keep every position; `compact_ids` reclaims tombstones
  on request and returns the old-to-new mapping; `clear` wipes the graph and restarts positions at 0.
  A traversal never reaches a tombstone, and one started at a tombstone returns `NodeNotFound`.
- **BREAKING** Edges are identified by source, target and a key taken from the payload through a
  new trait `EdgeKey`, in a new zero-dependency crate `ultragraph_traits` that `ultragraph`
  re-exports. `W` gains the bound `EdgeKey`. A second `add_edge` with the same key updates the edge;
  `remove_edge` removes every edge to the target; `remove_edge_keyed` and `contains_edge_keyed` act
  on one key. `()` and the primitive numeric types take `Key = ()`, so graphs with no payload or a
  weight hold at most one edge per ordered pair.
- **BREAKING** The dynamic state keeps a backward edge array per node, so `outbound_edges` and
  `inbound_edges` work without freezing, and `remove_node` removes the edges into a node as well as
  out of it. `number_edges` counts live edges only.
- Algorithms run on a projection: a `Projection<N, W>` trait with a node test and an edge test, a
  `project` view over the frozen graph, and a `ParallelEdges` mode, `Merge` or `Distinct`, chosen per
  projection. Algorithms called on the graph directly run on the simple projection, every node and
  edge in `Merge`. Under `Merge`, bridges and directed betweenness change on graphs with parallel
  edges; under `Distinct`, undirected betweenness changes, because today it deduplicates neighbours.
  Every other result is today's under both modes. Reads and traversals list every edge under both.
- **BREAKING** The context graph drops its rank remap; its data and time index maps drop entries
  naming a removed node at removal; its neighbour listings work in the dynamic state. It keeps one
  relation per pair through the store (Path A of the migration note), and `get_edge` reads the
  lowest relation of a pair in every state.
- **BREAKING** `CausableGraph` gains `index_bound()`; the reasoning engines, `check_single_writer`
  and `reachable_mask` size their arrays by it instead of `number_nodes()`.
- **BREAKING** `TeloidGraph` refuses a second relation on a pair that already carries the other
  relation, with a new `DeonticError` variant.
- The QCM readers in `deep_causality_quantum` size by `index_bound()`, check declared nodes with
  `contains_causaloid`, and keep their frozen-graph precondition with a rewritten rationale.
- **BREAKING** `deep_causality_context_store` takes its first dependency, `ultragraph_traits`, so
  that `RelationKind` implements `EdgeKey`; it moves from Tier 0 to Tier 1. `RelationKind` implements
  `PartialOrd` and `Ord` in declaration order.
- The work follows the TDD protocol of `openspec/specs/unified-math-tdd-protocol/spec.md`, and its
  performance is measured against the three-run baseline.

## Capabilities

### New Capabilities
- `ultragraph-stable-node-ids`: permanent positions across removal, freeze and unfreeze;
  tombstones; `compact_ids`; `clear`; counting and the highest live position in every state;
  traversal from a tombstone.
- `ultragraph-keyed-edges`: the `ultragraph_traits` crate and `EdgeKey`; edge identity by source,
  target and key; keyed and unkeyed add, remove and contains; the `()` and numeric impls; the frozen
  adjacency order.
- `ultragraph-dynamic-traversal`: backward edge arrays, traversal in the dynamic state, removal of
  inbound edges, live edge counts.
- `ultragraph-projection-view`: the `Projection` trait, the `project` view, `ParallelEdges`, the
  simple projection, and the results algorithms return on a projection.
- `context-graph-stable-indices`: the context graph's freeze, index maps, neighbour listings and
  edge semantics on the updated graph.
- `causaloid-graph-index-bound`: `index_bound()`, the reasoning engines, `check_single_writer` and
  `reachable_mask` on a graph with tombstones; stable `LambdaEdges` keys, state writers and relay
  targets.
- `teloid-graph-relations`: one relation per teloid pair, with a second relation of the other kind
  refused.

### Modified Capabilities
- `context-store-crate-identity`: the crate may declare exactly one dependency, `ultragraph_traits`,
  and sits at Tier 1; `RelationKind` implements `EdgeKey` and `Ord`.
- `qcl-hypothesis`: a factor is refused when its node holds no causaloid, not when it lies past
  `number_nodes()`; the frozen-graph requirement no longer cites dense node ids.
- `qcl-pipeline`: the frozen-graph requirement keeps its scenario, and its rationale no longer cites
  positions past `number_nodes()`.

## Impact

- **Crates:** new `ultragraph_traits`; `ultragraph` (both storage states, the container, every
  algorithm, the traits); `deep_causality` (`CausableGraph`, both reasoning engines,
  `reachable_mask`); `deep_causality_context` (context graph); `deep_causality_context_store`
  (`RelationKind`, manifest); `deep_causality_ethos` (`TeloidRelation`, `TeloidGraph`,
  `DeonticError`); `deep_causality_quantum` (four QCM readers).
- **Public API:** `EdgeKey`, `Projection`, `ParallelEdges`, `project`, `compact_ids`,
  `remove_edge_keyed`, `contains_edge_keyed`, `CausableGraph::index_bound`, a new `DeonticError`
  variant; the `W: EdgeKey` bound on every graph type. `ultragraph`'s semver moves to its next
  breaking version.
- **Tests:** thirteen existing tests change (migration note, §5); new suites per capability under
  the TDD protocol.
- **Workspace:** AGENTS.md crate count (34) and tier block; `BUILD.bazel` for the new crate and the
  new dependencies; `scripts/crates.sh` picks the new crate up from the manifests.
- **Performance:** keyed insert and removal scan the source's edge list, and every edge mutation
  writes the backward array too, so `add_edge` and `remove_edge` are expected to slow; the after-run
  is measured against `BenchBefore.md` with the same three-run protocol.
