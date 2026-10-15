<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 2: Permanent positions, and every caller that reads an index

Change `ultragraph-stable-ids-keyed-edges`, design D8. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran on 2026-10-15.

## Corner cases A–K

One test named per row, or `n/a` with the reason.

| # | Class | Instance in this group | Test |
|---|---|---|---|
| A | Empty | A graph with no node: freeze, unfreeze, `compact_ids`, `number_nodes`, `get_last_index`, `index_bound` 0 | `graph_compact_tests::test_compact_ids_on_an_empty_graph`; `causable_graph_tests::test_index_bound_of_an_empty_graph` |
| B | Single element | One node, removed and the graph frozen; one node kept through a round trip | `graph_evolve_tests::test_freeze_keeps_a_single_removed_slot` |
| C | Coinciding quantities | No removal, where position equals rank and the old renumbering is invisible; every scenario also runs with a removal below the node it reads | `graph_evolve_tests::test_freeze_without_removal_keeps_positions` beside `test_freezing_after_a_removal_moves_no_node` |
| D | Degenerate index | Removal at position 0, in the middle, at the last position, of every node; trailing tombstones; a node whose only neighbour is a tombstone | `graph_evolve_tests::test_round_trips_keep_trailing_tombstones`, `test_a_graph_with_no_live_node_keeps_its_table`; the model-based test |
| E | Thresholds, both sides | the small-graph betweenness case at `live <= 3`: 2, 3 and 4 live nodes (the search and sort thresholds belong to group 3) | the betweenness tests with a removal, among them `test_betweenness_with_exactly_three_live_nodes_scores_the_middle` |
| F | Zero | Zero edges; `number_nodes` 0 with slots present; `index_bound` 0 | `graph_view_tests::test_is_empty_counts_live_nodes`; `causable_graph_tests::test_index_bound_of_an_empty_graph` |
| G | Negative | n/a: positions are `usize` | — |
| H | Exact boundary | A position one past the slot count beside a tombstone at the last slot; the root at the last slot | `graph_view_tests::test_the_highest_live_position_skips_trailing_tombstones`; `graph_evolve_tests::test_the_root_keeps_its_position` |
| I | Non-finite | n/a: no floating-point input in this group | — |
| J | Overflow reach | n/a: no arithmetic on positions beyond `+ 1` on a slot count bounded by memory | — |
| K | Every payload kind | `()`, `u64` and a `&str` node payload through add, remove, freeze, unfreeze and `compact_ids` | the model-based test runs over `W = ()` and `W = u64` |

## Phase 1 — the API, every new body `unimplemented!()`

`DynamicGraph::compact_ids(&mut self) -> Vec<Option<usize>>` in
`ultragraph/src/types/storage/graph_dynamic/graph_compact.rs`; `UltraGraphContainer::compact_ids(&mut
self) -> Result<Vec<Option<usize>>, GraphError>` in `ultragraph/src/types/ultra_graph/graph_compact.rs`,
whose Frozen arm already returns `GraphIsFrozen`; `CausableGraph::index_bound(&self) -> usize` as a
provided method in `deep_causality/src/traits/causable_graph/graph/mod.rs`. Exit: `cargo build -p
ultragraph -p deep_causality` succeeds.

Before the suite, a site map: twelve readers (six subsystems, each read twice, the second pass
refuting the first) listed 228 sites that assume dense ids, size by `number_nodes()`, renumber or
enter the node count into a result. Two decisions it forced, both within the spec: the container's
Dynamic traversal arm checks the position first (a tombstone is `NodeNotFound` in both states, a
live node stays `GraphNotFrozen` until group 3 adds the backward lists), and the Dynamic
`number_edges`, which counts edges into tombstones, is group 3's requirement and untouched here.

## Phase 2 — the suite, written first, observed failing

Four writers, one per crate, then one auditor per crate checking scenario coverage, independent
expected values, no trailing-tombstone-only fixtures, and that each test fails today for the
intended reason. The ultragraph auditor was lost to a session limit; its suite was audited by hand
against the same list and stands.

| Crate | Tests added or rewritten | Phase-2 run against the old implementation |
|---|---|---|
| `ultragraph` | 30 (`graph_evolve`, `graph_view`, `graph_traversal`, `graph_mut`, new `graph_compact_tests.rs`, new `graph_algo_tests.rs`); every scenario of `ultragraph-stable-node-ids`; a model-based driver over seeded sequences of every mutation, freeze, unfreeze, `compact_ids` and `clear` for `W = ()` and `W = u64`; the generated agreement test over every algorithm of `GraphAlgorithms` | 132 passed, **26 failed** |
| `deep_causality` | 18 (`causality_graph_nodes_tests`, new `monadic_reasoning_index_bound_tests.rs`, `stateful_tests`, `graph_algebra_tests`, the inline `reachable_mask_tests`); every scenario of `causaloid-graph-index-bound` except the two group-3 ones | 704 passed, **18 failed** |
| `deep_causality_context` | 8 (`contextuable_graph_tests`, `context_tests`, `apply_tests`, `snapshot_tests`); the two group-2 requirements of `context-graph-stable-indices` | 796 passed, **8 failed** |
| `deep_causality_quantum` | 17 (`hypothesis_tests`, `config_tests`, `markov_freeze_tests`, `process_factors_tests`, `dem_model_tests`, `faithfulness_tests`); the modified `qcl-hypothesis` and `qcl-pipeline` requirements, each reader with a removed middle index below a live one | 872 passed, **17 failed** |

69 failures, every one the `not implemented` panic, the old renumbering answer (a position holding
the wrong payload, `get_last_index` one too low, a writer or factor at a live index above
`number_nodes()` refused) or the old range check accepting a removed index; no compile error. The
tests that pass today are the unchanged-behaviour scenarios (row C's no-removal control, the
trailing-tombstone freeze tests that both behaviours satisfy, `clear`, a never-issued position),
each named in the writers' reports.

Expected values: every literal position comes from the stated insertion order, every betweenness
score from a hand count of shortest paths beside the literal (1/3 = 2 paths / (4 − 1)(4 − 2)), every
engine value from the causaloid arithmetic in a comment (18 = (3 + 1) × 2 + 10, against 8 if the
node above `number_nodes()` did not fire and 800 if the tombstone did), the context indices from
`add_node`'s returns. The model-based test and the agreement test build their expectation from a
reference model and a removal set, never from the graph.

## Phase 3 — the defect audit

The throwaway was the candidate implementation itself; each defect was injected, the targeted
suite run, the file restored (`audit_g2.py`, scratch). The old renumbering freeze, the old
empty-graph shortcut and the readers' `>= n` checks are the pre-change code, so the phase-2 run is
their audit; the rest:

| Defect | Class | Caught by |
|---|---|---|
| D2 `add_node` reuses a tombstone | plausible neighbour | 12 tests, among them the model-based driver and `test_freeze_keeps_a_single_removed_slot` |
| D3 freeze of a graph with no live node resets the table | early return skipping a case | 6, among them `test_a_graph_with_no_live_node_keeps_its_table` |
| D4 frozen `number_nodes` counts slots | node count in a result | 18, among them `test_normalised_betweenness_counts_live_nodes` and the agreement test |
| D5 frozen `get_last_index` returns slots − 1 | off-by-one | 8, among them `test_the_highest_live_position_skips_trailing_tombstones` |
| D6 betweenness normalised by the slot count | constant factor | `test_normalised_betweenness_counts_live_nodes`, the agreement test |
| D7 frozen traversal from a tombstone yields no edges | guard replaced by a bound | `test_traversal_from_a_tombstone_fails_in_both_states` |
| D8 the context remap left in place | renumbering | 4, among them `test_freezing_after_a_removal_keeps_every_index_on_its_node` |
| D9 index-map entries kept at removal | early return skipping a case | `test_index_entries_naming_a_removed_node_go_at_removal` and the rewritten freeze test |
| D10 `index_bound` returning `number_nodes` | plausible neighbour | 9, both engines and the single-writer join scan |
| D10b `index_bound` returning the highest live index | off-by-one | 46, every engine test |
| D11 `FactorSupports::from_graph` checking `>= n` | guard replaced by a bound | the two removed-index factor tests |
| D12 tombstones seed the topological queue | early return skipping a case | `test_algorithm_results_name_no_tombstone`, the agreement test |
| D13 a tombstone starts a Tarjan DFS | guard removed | the same two and `test_algorithms_on_a_frozen_graph_with_no_live_node` |
| D14 `compact_ids` keeps an edge into a tombstone | plausible neighbour | the agreement test and both model-based drivers |
| D15 `topological_sort` compares the sorted count with slots | flipped comparison | `test_algorithm_results_name_no_tombstone`, the agreement test |
| D16 the small-graph betweenness case at `live <= 3` | threshold moved | **survived**: no test had exactly three live nodes (row E pinned on one side). Added `test_betweenness_with_exactly_three_live_nodes_scores_the_middle` (node 2 scores 1 unnormalised, 1/2 normalised); re-run: caught |
| D17 `check_single_writer` validates writers by range | guard replaced by a bound | `test_a_state_writer_on_a_removed_index_is_refused` |

Not admitted: a tolerance (none in this group). Input variety: row C has the no-removal control
beside every removal case; row E now has 2, 3 and 4 live nodes around the threshold.

## Phase 4 — implementation against the audited suite

`ultragraph`: `CsmGraph.nodes` is `Vec<Option<N>>` with a `live_nodes` count and a crate-internal
`node_slots()`; `freeze` keeps the node table whole, counts and places edges by position, drops the
edges that touch a tombstone, and no longer has an empty-graph shortcut; `unfreeze` moves the table
back unchanged; the frozen reads answer by slot (`contains_node`, `get_node`, `get_all_nodes`,
`get_last_index`, `is_empty`, the root reads, the `contains_edge` guard); every algorithm sizes its
arrays by `node_slots()` and skips tombstones at its DFS roots, the topological queue and the
Brandes sources; `topological_sort` compares with the live count, betweenness normalises by it and
lists live positions only (`live_scores`); the container's Dynamic traversal arm checks the position
first; `compact_ids` renumbers in order, drops edges into tombstones, remaps the root and returns
the table. `deep_causality`: `index_bound` over `get_last_index`; the two engines,
`check_single_writer` and `reachable_mask` size by it; writers are validated by
`contains_causaloid`. `deep_causality_context`: the rank remap is gone; `drop_index_entries` runs
from `remove_node` and the base-graph `release`; the trait doc and README say so.
`deep_causality_quantum`: the four readers and `graph_has_cycle` size by `index_bound()` and validate
declared indices by `contains_causaloid`; the density rationale left the docs and messages.

The suite went green on the first run after the audit. `make format && make fix`: clean.

| Crate | Cargo | Bazel |
|---|---|---|
| `ultragraph` | 159 | 159 |
| `deep_causality` | 722 | 720 |
| `deep_causality_context` | 804 | 804 |
| `deep_causality_quantum` | 889 | 950 |

The `deep_causality` and `deep_causality_quantum` offsets (−2, +61) are the ones group 1 recorded
before any test of theirs changed: Bazel builds quantum with more features and Cargo sums the
doc-test line. `bazel test` over the four crates: 323 targets, all passing.

Coverage: every added or changed line is reached by a named test above; no coverage tool was run.

## Phase 5 — mutation testing

`cargo mutants --in-diff <git diff HEAD -- <crate>/src>` per crate, so only mutants on changed lines
run (the edited files hold 894 mutants, most in code this group did not touch):

| Crate | Mutants | Caught | Unviable | Missed |
|---|---|---|---|---|
| `ultragraph` | 332 | 322 | 3 | 7 |
| `deep_causality` | 14 | 9 | 5 | 0 |
| `deep_causality_context` | 8 | 8 | 0 | 0 |
| `deep_causality_quantum` | 11 | 10 | 1 | 0 |

The seven survivors, each resolved:

- `-` → `/` in the betweenness scale `(live - 1) * (live - 2)`, both branches: a real gap. Under
  integer division `(live / 2)` equals `(live - 2)` at 3 and 4 live nodes, and every normalised test
  had 3 or 4. Added `test_normalised_betweenness_on_five_live_nodes_pins_the_scale` (directed 1/4,
  1/3, 1/4 over 12; undirected 1/2, 2/3, 1/2 over 6); re-run: 4 mutants, 4 caught.
- `||` → `&&` in the root-loop skip of `articulation_points_impl`, `bridges_impl` and
  `biconnected_components_impl`, and `&&` → `||` in `find_cycle`'s start loop: equivalent. The
  mutant re-enters a finished live root, which gains no tree child and reports nothing new, and
  enters a tombstone, which has no half-edge and no neighbour; only `time` advances. Recorded in
  `.cargo/mutants.toml`, one entry per mutant, each confirmed by the file's `comm` check to match
  exactly that mutant and nothing else.
- `CsmGraph::with_capacity` → `Default::default()`: equivalent, the capacity of the node vector
  being the only difference and unobservable. Recorded in `.cargo/mutants.toml` the same way.

No survivor is left unaccounted for.
