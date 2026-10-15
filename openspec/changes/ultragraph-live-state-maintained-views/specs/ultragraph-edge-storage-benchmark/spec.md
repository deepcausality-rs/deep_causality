## ADDED Requirements

### Requirement: Both edge-storage candidates pass the same suite before they are measured
The Dynamic state's per-node `Vec` lists and RadixGraph's snapshot-log edge arrays SHALL each pass the complete suite of `ultragraph-stable-ids-keyed-edges` and the candidate's own tests before either is measured, so the measurement compares two correct implementations of the same semantics. The temporary Cargo feature `edge-log` selects the snapshot-log arrays.

#### Scenario: The suite runs against each candidate
- **WHEN** `cargo test -p ultragraph` runs without and with `--features edge-log`
- **THEN** both runs pass with the same executed count

### Requirement: The benchmark covers the workloads the decisions depend on
`ultragraph/benches` SHALL measure, at 10, 100 and 1,000 nodes and on a hub of 10⁵ edges where named: building a graph by inserts; keyed update and removal on low-degree nodes and on the hub; outbound and inbound traversal in the Dynamic state over an ordered and a shuffled chain; `freeze` and `unfreeze`; memory per edge on the hub and at 1,000 nodes, after building and after the keyed update workload, with the snapshot-log `MIN_LOG` swept; every algorithm in the Live and the Frozen state; k = 1, 16 and 256 keyed writes followed by `strongly_connected_components`, at 1,000 nodes and on the hub, once in the Live state and once as Dynamic → `freeze` → the algorithm → `unfreeze`; updates in the Live state, in place and with row rebuilds; and the cost a mutation pays with 0, 1, 4 and 16 maintained views.

Memory per edge SHALL be the sum, in bytes, of the capacity of every buffer the edge storage holds, divided by the number of edges, computed by a `#[doc(hidden)]` function in `src/utils/utils_tests.rs`; it is deterministic and needs one run.

#### Scenario: Every workload has a benchmark group
- **WHEN** `cargo bench -p ultragraph -- --test` runs
- **THEN** a group exists and passes for every timed workload listed

#### Scenario: Memory per edge is counted
- **WHEN** the accounting function runs on a graph of three nodes and the edges 0 → 1 and 1 → 2 under each candidate
- **THEN** it returns the byte count derived by hand from each buffer's capacity, written beside the literal

### Requirement: Measurements follow the baseline's three-run protocol
Every timing that feeds a decision SHALL be three runs of `cargo bench -p ultragraph` on the baseline machine, aggregated as in `openspec/notes/ultragraph/BenchBefore/BenchBefore.md` (mean, spread as largest minus smallest over the mean, the median where the spread exceeds 15 %), and recorded under `openspec/notes/ultragraph/BenchStorage/` with the machine, `rustc` version and date.

A candidate *matches* the incumbent on a row when its mean is at most the incumbent's mean times one plus the larger of 5 % and the incumbent row's spread, so a faster candidate matches. It is *clearly faster* when each of its three runs is faster than each of the incumbent's and its mean lies below the incumbent's by more than the incumbent row's spread.

#### Scenario: A decision cites its table
- **WHEN** a decision of this capability is recorded
- **THEN** it cites the aggregated table and the rows it rests on

### Requirement: The edge-storage decision follows its rule
`MIN_LOG` SHALL be the swept value with the fastest keyed update on the hub among those whose memory per edge is at most 1.25 times the `Vec` lists' in every measured configuration; if no value qualifies, the `Vec` lists SHALL stay. At that `MIN_LOG`, the snapshot-log arrays SHALL replace the `Vec` lists if they match them on building, traversal and `freeze`, and are clearly faster on keyed update and removal on the hub; otherwise the `Vec` lists SHALL stay. The losing candidate SHALL be removed from the crate once the decision is recorded and approved.

#### Scenario: The decision and its rows are recorded
- **WHEN** the edge-storage decision is made
- **THEN** `openspec/notes/ultragraph/RadixGraphUpdate.md` §Benchmark records the outcome with the rows it rests on, the `MIN_LOG` chosen and both candidates' memory per edge, and only the winning candidate remains in `ultragraph/src`

### Requirement: The Live state stays only if it earns its place
The Live state SHALL stay if, for k = 1, 16 and 256 keyed writes followed by `strongly_connected_components`, at 1,000 nodes and on the hub, the Live state is clearly faster than Dynamic → `freeze` → the algorithm → `unfreeze`; otherwise the change SHALL stop for the user's decision before Live is removed or maintained views are built.

Live replaces the workflow of writing in the Dynamic state and freezing before each analysis, so it is measured against that workflow. `strongly_connected_components` is linear in the graph, which makes the avoided freeze a large share of the work.

#### Scenario: Live's decision is recorded before views are built on it
- **WHEN** the Live decision is made
- **THEN** it is recorded with its rows before any maintained-view implementation depends on the Live layout
