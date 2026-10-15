## Why

After `ultragraph-stable-ids-keyed-edges`, a graph is either mutable and unanalysable or analysable
and immutable: every algorithm needs `freeze`, which costs about 36 ns per node against about 9 ns to
build it (`openspec/notes/ultragraph/BenchBefore/BenchBefore.md`), and a projection is a borrow that
blocks every write while it lives. DeepBrain, which UltraGraph is meant to express, captures
continuously and analyses at the same time, and needs scoped views (a zone, validity at a time) that
stay current as it writes. Keyed edges also make a hub's insert and removal an O(d) scan of a `Vec`.
RadixGraph's snapshot-log edge arrays append in amortised O(1) (its Theorem 2), a bound for blind
writes; under the contract of `ultragraph-stable-ids-keyed-edges`, an exact `number_edges` and
`EdgeNotFoundError` for an absent edge, every operation first looks its edge up, a binary search of
the sorted snapshot plus a scan of the log, so whether they win is measured. The design is §Design 4,
6 and 7 and §Benchmark of `openspec/notes/ultragraph/RadixGraphUpdate.md`.

This change depends on `ultragraph-stable-ids-keyed-edges` and is applied after it.

## What Changes

- **BREAKING** `GraphState` gains a third variant, `Live`: rows packed in one arena, sorted by target
  then key, with spare capacity, a `(start, len, cap)` record and a mask bit per node. Live takes
  mutations locally, row by row, and runs every algorithm on current data. Transitions: Dynamic and
  Frozen into Live; Live into Frozen by `freeze`; Live into Dynamic by `unfreeze`.
- `freeze_clone` returns a frozen copy of a Dynamic or Live graph without its maintained views, and
  leaves the source, its state and its views as they were. `freeze` keeps converting in place.
- `materialize` turns a projection view into an owned, frozen CSR of the passing nodes and edges at
  the graph's positions, carrying the view's `ParallelEdges` mode.
- **BREAKING** The container takes a projection type parameter with a default that maintains nothing.
  `maintain(projection, mode)` registers a view the graph keeps current eagerly on every mutation;
  `view(id)` reads it. There is no refresh call.
- A benchmark decides the Dynamic state's edge storage among three candidates: today's per-node
  `Vec` lists, RadixGraph's snapshot-log edge arrays, and the `Vec` lists with an index on
  high-degree nodes (a `BTreeMap` from `(other end, key)` to the entry's position, built once a list
  reaches a degree threshold), each challenger at the parameter that keeps its memory per edge
  within 1.25 times the `Vec` lists'; and whether the Live state earns its place against freezing
  before each analysis. Every candidate passes the same suite before the measurement; the losers are
  removed. The after-benchmark of change 1 (`openspec/notes/ultragraph/BenchAfterChange1StableId/BenchAfterChange1StableId.md`)
  puts the hub's keyed `add_edge` at 353 ns and `remove_edge` at 328 ns against 4 ns and 171 ns
  before, the cost of scanning the hub's list for `(target, key)`; the index candidate exists to
  stop that scan without the snapshot-log's log and compaction.
- The work follows the TDD protocol of `openspec/specs/unified-math-tdd-protocol/spec.md`, and every
  timing that feeds a decision uses the three-run protocol of the baseline.

## Capabilities

### New Capabilities
- `ultragraph-three-states`: the Dynamic, Live and Frozen states, what each allows, the transitions
  between them, `freeze` and `freeze_clone`, and positions and tombstones in Live.
- `ultragraph-materialized-projection`: `materialize`, the owned frozen copy of a projection.
- `ultragraph-maintained-views`: the projection type parameter, `maintain` and `view`, eager
  maintenance, and the predicate contract a maintained view relies on.
- `ultragraph-edge-storage-benchmark`: the workloads, the measurement protocol, the two decision rules
  and their record.

### Modified Capabilities
- `ultragraph-projection-view` (added by `ultragraph-stable-ids-keyed-edges`): a graph's direct
  algorithms run under the graph's `ParallelEdges` mode, which is `Merge` except for a copy
  `materialize` returns, which carries its view's mode.

## Impact

- **Crates:** `ultragraph` (storage states, the container, algorithms, benches). No DeepCausality
  caller changes: they stay in the Dynamic state and use `freeze` as today.
- **Public API:** `GraphState::Live`, the transition into Live, `is_live`, `freeze_clone`,
  `materialize`, `maintain`, `view` and the view identifier type, the container's projection type
  parameter, and `GraphError::GraphIsLive`, which `compact_ids` returns in the Live state.
  `GraphState` and `GraphError` are public and not `#[non_exhaustive]`, so their new variants break
  an exhaustive `match` outside the crate; none exists in the workspace.
- **Benchmarks:** new workloads in `ultragraph/benches`; results in
  `openspec/notes/ultragraph/BenchStorage/` and the decisions in the design notes.
- **Tests:** new suites per capability under the TDD protocol; the change-1 suites run against both
  edge-storage candidates, each selected by its temporary Cargo feature.
