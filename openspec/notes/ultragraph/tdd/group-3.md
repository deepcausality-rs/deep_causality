<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 3: Keyed edges, dynamic traversal and the projection view

Change `ultragraph-stable-ids-keyed-edges`, design D8. The five phases of
`openspec/specs/unified-math-tdd-protocol/spec.md`, recorded as they ran on 2026-10-15.

## Corner cases A–K

One test named per row, or `n/a` with the reason.

| # | Class | Instance in this group | Test |
|---|---|---|---|
| A | Empty | A node with no edge; a projection passing nothing; `remove_edge` on a pair with no edge; traversal of an isolated node in the Dynamic state | `graph_mut_tests::test_removing_an_absent_edge_is_reported`; `graph_project_tests::test_a_projection_passing_nothing` |
| B | Single element | One edge; one node with a self-loop; a projection passing one node | `graph_traversal_tests::test_one_entry_per_edge`; `graph_project_tests::test_a_projection_passing_one_node` |
| C | Coinciding quantities | No parallel edge, where `Merge` equals `Distinct`; every mode scenario also runs with a parallel edge where they differ | `graph_project_tests::test_results_that_do_not_depend_on_the_mode` beside `test_bridges_under_each_mode` |
| D | Degenerate index | `remove_node` of a node with edges in and out, in the middle of the table; a removal from a tombstone source | `graph_traversal_tests::test_incoming_edges_leave_with_the_node`; `graph_mut_tests::test_removing_from_an_absent_source_is_reported` |
| E | Thresholds, both sides | Frozen `contains_edge` at degree 63, 64, 65; the freeze sort at degree 127, 128, 129 | `graph_mut_tests::test_lookups_across_the_search_threshold`, `test_sorting_across_the_radix_threshold` |
| F | Zero | Zero edges after a pair removal; weight 0 as a payload | `graph_mut_tests::test_removing_a_pair_removes_both_keys`; the model-based driver draws weight 0 |
| G | Negative | n/a: positions are `usize`; negative weights reach `shortest_weighted_path` only through `W: Ord`, unchanged | — |
| H | Exact boundary | An edge whose target is the last position; a projection whose node test flips on the last node | `graph_project_tests::test_positions_are_not_renumbered` (fails node 0, keeps the last) |
| I | Non-finite | n/a: `f32`/`f64` payloads have `Key = ()` and are never compared by the graph | — |
| J | Overflow reach | n/a: no arithmetic on positions | — |
| K | Every payload kind | `W = ()`, `u64`, a kind key (`Kind`), an identity key (`u128` field), each through add, remove, contains, freeze and traversal | the model-based driver over `()` and `u64`; `graph_mut_tests` over `Kind`; `graph_view_tests::test_an_identity_keyed_payload` |

## Phase 1 — the API, every new body `unimplemented!()`

`GraphView::contains_edge_keyed(a, b, key: W::Key)` and `GraphMut::remove_edge_keyed(a, b, key)`, which
put the `W: EdgeKey` bound on every graph trait; the `Projection<N, W>` trait with its impl for every
`(F, G)` closure pair; `ParallelEdges::{Merge, Distinct}`; `UltraGraphContainer::project(projection,
mode)`, whose Dynamic arm already returned `GraphNotFrozen`; the `ProjectionView` type with every read,
traversal and algorithm stubbed; `GraphTraversal for DynamicGraph`; `DeonticError::ConflictingRelation
{ from, to }` with its `Display`. The traversal stubs return an iterator that panics on its first
`next()`, so a test of them fails by the intended panic. Exit: `cargo build` and `bazel build` of
`ultragraph` and `deep_causality_ethos` succeed, with one dead-field warning on the view until phase 4.

## Phase 2 — the suite, written first, observed failing

Four writers and four auditors, one crate each.

| Crate | Tests added or rewritten | Run against the stubs |
|---|---|---|
| `ultragraph` | 55 (`graph_mut`, `graph_view`, `graph_traversal`, `graph_evolve`, `graph_algo_bridges`, new `graph_project_tests.rs`): every scenario of `ultragraph-keyed-edges`, `ultragraph-dynamic-traversal` and `ultragraph-projection-view`; the model-based driver extended with keyed `add_edge`, `remove_edge`, `remove_edge_keyed` and both traversal directions; oracles for bridges (delete each connection, test connectivity) and betweenness (shortest-path enumeration on at most eight nodes), each calibrated against the spec's hand counts | 157 passed, **49 failed** |
| `deep_causality_ethos` | 13 (`teloidable_tests`, `effect_ethos_linking_tests`, `deontic_error_tests`): every scenario of `teloid-graph-relations` | 135 passed, **7 failed** |
| `deep_causality_context` | 8 (`contextuable_graph_tests`, `snapshot_tests`, `extendable_contextuable_graph_tests`): the three group-3 requirements of `context-graph-stable-indices` | 802 passed, **7 failed** |
| `deep_causality` | 3 (`causality_graph_edges_tests`, `graph_algebra_tests`, the inline `reachable_mask_tests`): the two deferred scenarios of `causaloid-graph-index-bound` and the rewritten dynamic-mask test | 721 passed, **3 failed** |

66 failures: the `not implemented` panic, the old appending count (2 where 1 is expected),
`remove_edge` leaving the second key, `GraphNotFrozen` from a dynamic traversal, the first-match
relation, the conflict not reported; no compile error. The passing tests pin unchanged behaviour
(different keys coexist, an absent end refused, a dynamic `project` refused, the opposite-direction
teloid pair, the oracle calibrations). The auditors corrected two tests: a weighted-path assertion
that compared the whole payload where the spec names the weight, and a radix-threshold expectation
built from the output itself, replaced by one derived from the insertion plan.

## Phase 3 — the defect audit

The throwaway was the candidate implementation itself (`audit_g3.py`, scratch); each defect was
injected alone, the targeted suite run, the file restored.

| Defect | Class | Caught by |
|---|---|---|
| D1 keyed `add_edge` appends | plausible neighbour | 13, among them `test_a_second_weight_updates_the_first` and both model-based drivers |
| D2 `remove_edge` removes one key | early return skipping a case | `test_removing_a_pair_removes_both_keys` |
| D3 `contains_edge_keyed` ignores the key, frozen and dynamic | guard removed | 5 and 3, among them `test_keyed_and_unkeyed_tests_agree_with_the_edges_present` |
| D4 the backward list not updated on edge removal | early return skipping a case | `test_removing_an_edge_updates_both_directions`, the generated mutation-sequence test |
| D5 `remove_node` leaves the incoming edges | early return skipping a case | 11, among them the model-based drivers and the betweenness oracle |
| D6 `Merge` and `Distinct` swapped, in the symmetric adjacency and in directed Brandes | flipped comparison | 6 (`test_bridges_under_each_mode`, `test_distinct_counts_the_larger_direction`, …) and 2 (`test_directed_betweenness_under_each_mode`, the oracle) |
| D7 an edge passes with its target failing | guard removed | 5, among them `test_an_edge_between_scopes_fails_with_its_end` |
| D8 the weighted path keeps the first parallel edge | plausible neighbour | `test_the_weighted_path_takes_the_lightest_parallel_edge_in_both_modes` and a pre-existing Dijkstra test |
| D9 undirected `Distinct` betweenness deduplicates neighbours | constant factor | `test_undirected_betweenness_under_each_mode`, the oracle |
| D10 the context's `get_edge` reads the first edge | plausible neighbour | `test_the_relation_read_does_not_depend_on_the_state` |
| D11 the teloid conflict check omitted | guard removed | 5, the three `teloidable_tests` conflicts and both `EffectEthos` links |
| D12 the keyed lookup's run ends one entry early (`<` for `<=`) | off-by-one | 5, among them `test_lookups_across_the_search_threshold` |
| D13 the key pre-sort skipped at the radix threshold | threshold | `test_sorting_across_the_radix_threshold` |
| D14 the view's `number_edges` counts every edge of the graph | plausible neighbour | 8 |
| D15 dynamic `inbound_edges` reads the forward list | plausible neighbour | 7 |
| D16 the view's node test ignored | guard removed | 9 |
| D17 a replaced edge keeps its old payload | early return skipping a case | 8 |
| D18 the view's edge test ignored | guard removed | `test_an_edge_test_filters_by_payload`, the generated agreement test |

No survivor. Not admitted: a tolerance (none). Input variety: row C's no-parallel-edge control sits
beside every mode scenario; row E has 63/64/65 and 127/128/129.

## Phase 4 — implementation against the audited suite

`ultragraph`: a crate-internal `Adjacency<W>` trait (slots, passes, mode, out and in edges) with
two implementors, `CsmGraph` as the simple projection under `Merge` and `ProjectionView` with its
tests and mode; every algorithm body is a generic function over it, called by both; the mode enters
only where multiplicity counts: directed Brandes deduplicates targets under `Merge`, undirected
Brandes and the symmetric adjacency give an unordered pair one edge under `Merge` and `max(forward,
reverse)` under `Distinct`, Dijkstra relaxes every edge so the lightest wins under both. The
dynamic graph keeps a backward list and a live edge count; `add_edge` replaces by key, `remove_edge`
removes every key, `remove_edge_keyed` one, `remove_node` takes the edges into the node with it, so
no edge touches a tombstone and `freeze` filters nothing; `freeze` sorts each list by target then
key (a comparison sort below 128, a stable key sort then the stable radix sort at 128 or more);
`contains_edge_keyed` finds the target's run by two partition points. The container's traversal
answers in both states through one enum iterator. The view's reads answer for passing nodes and
list every passing edge. `deep_causality_context`: `get_edge` and `extra_ctx_get_edge` return the
lowest kind; the `GraphNotFrozen` arm of `traversal_error` is gone; docs and README say both states
traverse. `deep_causality_ethos`: `add_relation` refuses the other relation on a pair.
`deep_causality`: the `reachable_mask` doc no longer claims a dynamic graph yields no edges.

The suite went green on the first run after the audit: `ultragraph` 205 (214 after phase 5's additions),
`deep_causality_ethos` 142, `deep_causality_context` 809, `deep_causality` 724,
`deep_causality_quantum` 889 (unchanged).
`make format && make fix`: clean. Bazel, `tests(//<crate>/...)` of one `bazel test //...` run (1,528
tests pass): `ultragraph` 205 (214 after phase 5), `deep_causality_ethos` 142,
`deep_causality_context` 809, all equal
to Cargo; `deep_causality` 722 and `deep_causality_quantum` 950 carry the offsets recorded since
group 1 (−2, +61), which no test of this group touched.

Coverage: every added or changed line is reached by a named test above; no coverage tool was run.

## Phase 5 — mutation testing

`cargo mutants --in-diff <git diff HEAD -- <crate>/src>` per crate:

| Crate | Mutants | Caught | Timeouts | Unviable | Missed |
|---|---|---|---|---|---|
| `ultragraph` | 573 | 536 | 10 | 8 | 19 |
| `deep_causality_ethos` | 6 | 6 | 0 | 0 | 0 |
| `deep_causality_context` | 7 | 6 | 0 | 1 | 0 |
| `deep_causality` | 0 (docs and an inline test only) | — | — | — | — |

A timeout is a mutant that hangs the suite (an unterminated BFS or run scan); the suite never
passes, so it counts as noticed. The nineteen survivors, grouped by resolution (a guard yields several mutants):

- Six real gaps, closed by tests: `remove_edge_keyed`'s backward retain by source (two sources, one
  key), `DynamicGraph::clear` (the container replaces its state and never calls it, so it is
  reachable only directly), `from_parts` rebuilding the backward lists, path queries with a
  never-issued position at either end (row H), `find_cycle`'s exact closed walk on a triangle and a
  self-loop beside a diamond DAG, and, from the second run, pathway betweenness through a diamond
  (halves) with a self-pathway counting toward the normalisation.
- Dead code removed rather than excused: the backward entry's payload update on replace (only
  the entry's source and key are ever read), the guards `scale > 0.0` (at least three live nodes
  reach it) and `sigma[w] > 0.0` (every node on the stack was reached), and the `while`
  condition of `undirected_neighbours`, which duplicated its `(None, None)` arm.
- Three equivalent, recorded in `.cargo/mutants.toml` with their arguments and confirmed by the
  file's `comm` check to match exactly the intended mutants: Dijkstra's stale-entry skip (`>` →
  `<` skips nothing and improves no neighbour), the sort threshold (both branches sort by target
  then key; the 127/128/129 test pins the order on both sides), and `find_cycle_impl`'s root
  guard (group 2's entry, renamed with the function).
- Two first recorded as equivalent and then found not to be, their entries removed and each
  closed by a test: `DynamicGraph::with_capacity` → `Default` (the reserved capacity reaches
  `to_parts()`; `test_with_capacity_reserves_each_edge_list` reads it), and Dijkstra's relaxation
  on a tie (`<` → `<=` re-enqueues a node along a zero-weight self-loop or cycle without end;
  `test_shortest_weighted_path_terminates_on_zero_weight_loops` runs into the timeout, which
  `cargo mutants` counts as a kill).

Second run after the fixes: 552 mutants, 532 caught, 10 timeouts, 8 unviable, 2 missed, both the
`valid_pathways += 1` of a self-pathway, closed by the normalisation test above and re-run: 8 of 8
pathway mutants caught.

**A defect found by the closed-walk test**, fixed: `find_cycle` reconstructed a self-loop reached
from a predecessor as `[0, 1, 1]`, walking the predecessors before checking that the cycle closes
at the node itself; the doc already promised `[n, n]`. The walk now stops when it reaches `v`.

## Close-out checks

`make format && make fix`: clean. `bazel test //...`: 1,528 tests pass. Ultragraph's example and
benches build.
