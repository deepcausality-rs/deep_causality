# UltraGraph update: migrating the downstream types

* Date: 2026-10-15
* Parent: `openspec/notes/ultragraph/RadixGraphUpdate.md` (the design). This note covers what
  every type built on UltraGraph must change, in which order, and which tests change with it.
* Grounded in: the source as of this date. Paths are relative to the `deep_causality` repository.
* Decided by Marvin on 2026-10-15: `TeloidRelation` takes `Key = ()`.

## Inventory

Population: every file outside `ultragraph/` that imports `ultragraph`, plus every consumer of
`CausableGraph` that sizes an array or a range by `number_nodes()`. Three crates depend on
`ultragraph` directly (`deep_causality`, `deep_causality_context`, `deep_causality_ethos`);
`deep_causality_quantum` reaches it through `CausableGraph`. Of UltraGraph's algorithms, the
downstream crates call `has_cycle`, `topological_sort` and `shortest_path`, and no others.

| # | Type | Crate | Payload `W` | `Key` | Fails to compile without migration | Wrong at run time without migration |
|---|---|---|---|---|---|---|
| 1 | `Context` (base graph and `ExtraContext`) | `deep_causality_context` | `RelationKind` | `RelationKind` | yes, until `RelationKind: EdgeKey` | yes: the freeze remap corrupts every index map |
| 2 | `CausaloidGraph`, with the default methods of `CausableGraph`, `MonadicCausableGraphReasoning` and `StatefulMonadicCausableGraphReasoning` | `deep_causality` | `u64` | `()` | no (`ultragraph_traits` covers `u64`) | yes: reasoning panics after a removal and a freeze |
| 3 | `TeloidGraph`, `EffectEthos` | `deep_causality_ethos` | `TeloidRelation` | `()` | yes, until `TeloidRelation: EdgeKey` | no; one relation per pair, last write wins (§3) |
| 4 | `FactorSupports`, `CausalStructure`, `DemModel`, the model-subject validation | `deep_causality_quantum` | via `CausableGraph` | n/a | no | yes: live nodes refused as out of range, and silent false negatives, after a removal and a freeze |
| 5 | `RelationKind`, the storage contract | `deep_causality_context_store` | n/a | n/a | gains a dependency | only if the context adopts typed edges (§1.9) |

## Order of work

1. **Traits.** Add `ultragraph_traits` with `EdgeKey` and its impls for `()` and the primitive
   numeric types. Add `impl EdgeKey for RelationKind` and `impl EdgeKey for TeloidRelation`. After
   this step, the workspace compiles against the new bound and nothing behaves differently.
2. **Permanent node IDs**, with §1.2, §1.3, §2.2 and §4 in the same change. Each of those reads
   indices that the old freeze renumbered, so none of them can land after the ID change without a
   window in which the workspace gives wrong answers.
3. **Keyed edges and inbound traversal in the dynamic state**, with §1.4, §1.5, §2.3–§2.5 and §3.
4. **Typed edges in the context**, only if decided (§1.9). A separate change: it alters the storage
   contract.

Every step ends with `bazel test //...` green and its commit message.

## 1. `Context` (`deep_causality_context`)

### 1.1 `EdgeKey` for `RelationKind`

`RelationKind` lives in `deep_causality_context_store`, so the impl goes there, in a file
`edge_key.rs` beside `types/relation_kind/mod.rs`:

```rust
impl EdgeKey for RelationKind {
    type Key = RelationKind;
    fn edge_key(&self) -> RelationKind {
        *self
    }
}
```

`RelationKind` derives `Copy, Clone, Debug, Default, Eq, PartialEq, Hash` today. The frozen
adjacency sorts by target, then kind, so `EdgeKey` bounds `Key: Copy + Ord`, and `RelationKind`
adds `PartialOrd, Ord`. A derived `Ord` on an enum orders the variants by declaration:
`Datial < Temporal < Spatial < SpaceTemporal`.

`deep_causality_context_store` gains its first dependency, `ultragraph_traits`. AGENTS.md
describes the store crate as having "No dependencies, so a backend links this crate alone."
That sentence changes to "a backend links this crate and `ultragraph_traits`, which has no
dependencies". The tier block in AGENTS.md changes: `ultragraph_traits` enters Tier 0, and
`deep_causality_context_store` and `ultragraph` move to Tier 1. The tier table in
`deep_causality_unified_math/README.md` covers only the seventeen crates in that folder, none of
which is affected, and does not change. The workspace has 34 library crates. Both `Cargo.toml` and `BUILD.bazel` carry the new
dependency.

### 1.2 Freeze: delete the remap

**Today.** `ContextuableGraph::freeze`
(`deep_causality_context/src/types/context_types/context_graph/contextuable_graph.rs:173-207`)
computes each live node's rank before freezing, freezes, then rewrites `id_to_index_map` and the
four data and time maps through the rank table, dropping entries that name a removed node.

**After.** Freezing keeps every position, so the rank no longer names the frozen position. Left in
place, the remap points every map at the wrong node. Example: add nodes with ids 10, 20 and 30
(indices 0, 1, 2), remove 20, freeze. The remap writes `id 30 → 1`. Position 1 is a tombstone, so
`get_data_by_id(30)` returns `None` and `get_node_index_by_id(30)` returns an index that names
nothing.

**Migration.** `freeze` becomes `self.base_context.freeze()`, which is already a no-op on a frozen
graph (`ultragraph/src/types/ultra_graph/graph_evolve.rs:18-30`). The early return goes with the
remap.

**Docs.** `deep_causality_context/src/traits/contextuable_graph/mod.rs:82-84` says freezing
compacts and that the context remaps every index. The paragraph becomes: freezing and unfreezing
keep every index.

### 1.3 Index maps: drop stale entries at removal

**Today.** `remove_node` drops the node's entry in `id_to_index_map`. Entries in
`current_data_map`, `previous_data_map`, `current_index_map` and `previous_index_map` that name the
removed index stay until the next freeze, which drops them. The test
`test_freezing_after_a_removal_keeps_every_index_on_its_node` pins that they are gone after a
freeze (`contextuable_graph_tests.rs:380-381`).

**After, without migration.** No freeze drops them. They name a tombstone for good: `get_node`
returns `None` for them, and since positions are never reused they never name another node.
`get_data_index` and `get_time_index` keep returning them.

**Migration.** Drop the entries at removal instead of at freeze. A private helper on `Context`
removes every entry of the four maps whose value is the removed index; `remove_node`
(`contextuable_graph.rs:50-66`) calls it, and so does `release` (`apply.rs:220-231`) when the
container is the base graph. The contract the test states, that entries naming a removed node go,
holds from the moment of removal rather than from the next freeze. This is a behaviour change in
the dynamic state only. Leaving the entries in place is the alternative; it is safe, but it breaks
the stated contract.

### 1.4 Traversal in the dynamic state

**Today.** `outbound_edges` and `inbound_edges` fail with "The context is not frozen; call freeze
first" in the dynamic state (`contextuable_graph.rs:217-247`).

**After.** Both work in either state, so the `GraphError::GraphNotFrozen` arm of `traversal_error`
cannot be reached from them. `GraphNotFrozen` stays a variant of `GraphError`, because the
algorithms still require a frozen graph.

**Migration.**
- Remove the `GraphNotFrozen` arm of `traversal_error`; the `other => other.to_string()` arm
  covers the variant.
- `contextuable_graph/mod.rs:96-110`: the docs of both methods drop "read from the frozen graph"
  and "when the context is not frozen" from their error lists.
- `test_neighbour_listing_needs_a_frozen_context` (`contextuable_graph_tests.rs:270`) becomes a
  test that the listings in the dynamic state equal the listings after `freeze`, on a graph with a
  removed node and edges into it.

### 1.5 Edge semantics through the unkeyed calls

`Context` keeps calling `add_edge`, `remove_edge` and `contains_edge` without a key. With
`Key = RelationKind`:

| Call | Today | After |
|---|---|---|
| `add_edge(a, b, k)` twice, same `k` | Two parallel edges; `snapshot` then fails with "an edge is carried twice" | One edge; `snapshot` succeeds |
| `add_edge(a, b, k1)`, then `add_edge(a, b, k2)` | Two parallel edges; `snapshot` fails | Two edges, one per kind; `snapshot` still fails |
| `remove_edge(a, b)` | Removes the first edge to `b` | Removes every edge to `b`, of every kind |
| `contains_edge(a, b)` | Any edge to `b` | Any edge to `b`, of any kind |
| `get_edge(a, b)` (`contextuable_graph.rs:129-135`) | The first edge to `b` in storage order | With two kinds: the lowest kind, in both states (`contextuable_graph.rs` takes the minimum over the pair's edges) |
| `remove_node(i)` | Clears `i`'s outgoing edges; edges into `i` stay until `freeze` | Removes the edges in both directions |
| `number_of_edges()`, dynamic state | Counts edges into removed nodes | Counts live edges; `Display` and `Debug` print it |

`ContextuableGraph::add_edge` performs no duplicate check today; only `connect` (`apply.rs:241-257`)
and `restore::fill` do. The second row shows a gap that exists today and stays: the in-memory
context accepts a second kind between one pair, and `snapshot` refuses it. §1.9 decides whether to
close the gap and in which direction.

`test_a_parallel_edge_is_refused` (`deep_causality_context/tests/types/context_types/context_graph/snapshot_tests.rs:246-269`)
pins rows 2 and 3. It adds `Datial` and `Spatial` from `a` to `b`, calls `remove_edge(a, b)`,
adds `b → a`, and expects two edges in the snapshot. Today `remove_edge` leaves one of the two
kinds; afterwards it removes both, the snapshot holds one edge, and the test fails. It changes
with this step.

The same table holds for the `extra_ctx_*` methods, which delegate to the extra's graph.
`extra_ctx_remove_edge` (`extendable_contextuable_graph.rs:237-280`) reads any error from
`remove_edge` as "the edge does not exist", so UltraGraph's `remove_edge` must still fail when no
edge of any kind joins the pair.

### 1.6 Scans by position

`apply.rs::index_of`, `ExtraContext::holds` and `snapshot.rs::walk` scan `0..=get_last_index()`
and skip positions where `get_node` is `None`. They stay correct, because `get_last_index` returns
the highest live position in both states. Their cost grows with the tombstones below that position.

### 1.7 Snapshot and restore

Unchanged while the context holds one relation per pair. A context that today fails `snapshot`
because of a same-kind duplicate succeeds after the change, because the duplicate no longer
exists.

### 1.8 `compact_ids`

`Context` does not call it. A `Context::compact` built on it would remap `id_to_index_map` and the
four index maps through the returned table: the work of the code §1.2 deletes, driven by
UltraGraph's table rather than a rank computed beforehand. Extra contexts keep no maps and need
nothing.

### 1.9 Typed edges in the context: open decision

The parent note expects relations of several kinds between two contextoids to be kept once
`connect` and `sever` use the keyed variants. The storage contract rules that out.
`ContextStorage` states that "a relation exists once between two nodes: `create_edge` is idempotent
for an identical `(from, to, kind)`, refused for a different kind between the same pair"
(`deep_causality_context_store/src/traits/context_storage.rs:31-33`), and the in-memory backend
enforces it (`utils_test/memory_storage/memory_state.rs:187-225`). A conforming store never sends a
second kind, so `apply` never receives one.

**Path A, recommended for this update: one relation per pair.** Nothing in §1.1–§1.8 changes.
`connect`, `sever` and `changes_base` keep the unkeyed calls. The gap in §1.5, row 2, stays as it
is today.

**Path B: typed edges end to end.** A separate change, touching:
- the `ContextStorage` rule above, and `create_edge`, which accepts a second kind;
- `retract_edge(from, to)` (`context_storage.rs:87-93`), which gains a kind or retracts every kind;
- `ContextEvent::EdgeRetracted { from, to }` (`types/context_event/mod.rs:29-32`), which gains a
  kind;
- the in-memory backend, whose edge map is keyed by `(from, to)`;
- `snapshot.rs::walk`, which orders edges by `(from, to)` and refuses a second edge per pair, and
  `restore.rs::fill`, which refuses a second edge per pair: both move to `(from, to, kind)`;
- `connect` (`apply.rs:241-257`) and `changes_base` (`apply.rs:139-177`), which both switch to
  `contains_edge_keyed`. If `connect` switches and `changes_base` does not, an `EdgeCreated` of a
  new kind between a connected pair on a frozen context reads as "no change", passes the frozen
  guard, reaches `add_edge` on the frozen graph, and fails as `Identity` instead of `Frozen`;
- `sever`, which follows the new `EdgeRetracted`;
- `get_edge` and `extra_ctx_get_edge`, whose `Option<&RelationKind>` cannot report two kinds;
- `RECORD_VERSION`, from 1 to 2, because a version-1 reader refuses a snapshot carrying two kinds
  between one pair;
- every `ContextStorage` backend outside this repository.

## 2. `CausaloidGraph` and `CausableGraph` (`deep_causality`)

### 2.1 Compile

No change. `CausalGraph<T> = UltraGraphWeighted<T, u64>`
(`deep_causality/src/traits/causable_graph/mod.rs:11`), and `ultragraph_traits` implements
`EdgeKey` for `u64` with `Key = ()`.

### 2.2 The index bound

**Today.** Freezing makes positions dense, so the reasoning code sizes every per-node array by
`number_nodes()` and loops over `0..number_nodes()`:

| Site | Use of `number_nodes()` |
|---|---|
| `traits/causable_graph/graph/mod.rs:137-186`, `check_single_writer` | rejects a writer `>= n_nodes`; sizes the cones; loops joins over `0..n_nodes` |
| `traits/causable_graph/graph_reasoning/mod.rs:160-196` | `reachable_mask`, `pending`, `fired`, `processed`, the parent loop |
| `traits/causable_graph/graph_reasoning/stateful.rs:107-128` | the same, in the stateful engine |
| `traits/causable_graph/mod.rs:46-80`, `reachable_mask` | `seen = vec![false; n_nodes]`, indexed by every node it reaches |

**After, without migration.** `number_nodes()` counts live nodes while positions keep their gaps.
Example: add causaloids 0, 1 and 2, add the edge 0 → 2, remove 1, freeze. `number_nodes()` is 2.
- Evaluating from 0 panics: `reachable_mask` reaches 2 and writes `seen[2]` into a vector of
  length 2.
- `freeze_verified(&[2])` refuses a live writer: "valid writer indices are 0..2".
- `check_single_writer` never inspects node 2 as a join.

**Migration.** Add a provided method to `CausableGraph`, beside `get_last_index`:

```rust
/// One past the highest index of a live causaloid; 0 for an empty graph. Removed causaloids
/// leave gaps below it, so it is at least `number_nodes()`.
fn index_bound(&self) -> usize {
    self.get_last_index().map_or(0, |last| last + 1)
}
```

It needs no change to UltraGraph's public API: UltraGraph's `node_slots()` stays crate-internal,
and `get_last_index` already returns the highest live position in both states. Then:
- size and loop by `index_bound()` at the three sites above, and pass it to `reachable_mask`, whose
  parameter `n_nodes` becomes `bound`;
- `check_single_writer` rejects a writer for which `!self.contains_causaloid(w)`, with a message
  naming the index as holding no causaloid. A tombstone below the bound is refused as well as an
  index past it;
- tombstones need no special case in the loops. No traversal reaches a tombstone, and one started
  at a tombstone returns `NodeNotFound` (parent note, §Design 1). The parent loops read
  `inbound_edges` through `if let Ok` or `Err(_) => continue`, so they skip a tombstone, and no
  child loop reads a tombstone's `outbound_edges` because none is reachable.

### 2.3 Parallel edges

`add_edge(a, b)` writes weight 0 and `add_edg_with_weight(a, b, w)` writes `w`
(`types/causal_types/causaloid_graph/causable_graph.rs:112-129`). Today a second call on the same
pair adds a second edge. Afterwards it replaces the weight, so `number_edges()` drops by one per
former duplicate.

Evaluation results stay the same. Today a duplicate edge counts its parent twice in `pending[c]`
(`graph_reasoning/mod.rs:190-196`), and the child loop decrements `pending[c]` once per outbound
edge (`graph_reasoning/mod.rs:343-361`), so the two cancel. `fired[c]` is keyed by the parent's
index and already holds one entry per parent. The stateful engine has the same structure.

### 2.4 Removal

`remove_causaloid` also removes the edges into the causaloid. No caller in the workspace depends on
those edges surviving: before this change, freezing dropped them.

### 2.5 Traversal in the dynamic state

The engines check `is_frozen()` first and return `GraphNotFrozen`, so their behaviour and the five
tests pinning `CausalityError::GraphNotFrozen` stay as they are.

`check_single_writer` is a public trait method that calls `inbound_edges` directly. Today, on a
dynamic graph, every lookup fails and every join is skipped, so any writer set that passes the
range check returns `Ok` without a join being examined. Afterwards it checks. The doc comment of
`reachable_mask` (`traits/causable_graph/mod.rs:31-35`), which says a dynamic graph "yields no
edges", becomes false and is rewritten, and the unit test
`test_a_dynamic_graph_yields_no_edges_and_stops_at_the_start` (`traits/causable_graph/mod.rs:147`),
which asserts that behaviour, changes to assert the reach on a dynamic graph.

### 2.6 Indices that become stable

These hold node indices across a removal and a freeze, and today silently name a different
causaloid after one: the `(source, target)` keys of `LambdaEdges`
(`types/causal_types/causaloid_graph/lambda_edges.rs`), the `state_writers` argument of
`freeze_verified`, and the target of a `RelayTo` command. After the change they keep naming the
causaloid they were taken from.

### 2.7 Tests

New tests, each failing on the code without §2.2:
- evaluate from a start whose descendant sits above `number_nodes()` after a removal and a freeze,
  in both engines;
- `freeze_verified` with a writer above `number_nodes()`, accepted, and a writer on a tombstone,
  refused;
- a decorated `LambdaEdges` edge on a graph with a removal: the decoration applies to the same edge
  after the freeze.

No existing test in `deep_causality` asserts a renumbering.
`tests/formalization_lean/graph_algebra_tests.rs:164` mentions "valid writer indices are 0..4" in
a comment on a graph without removals, where it stays true.

## 3. `TeloidGraph` and `EffectEthos` (`deep_causality_ethos`)

### 3.1 `EdgeKey` for `TeloidRelation`

`TeloidRelation` (`types/teloid_relation/mod.rs`) takes `Key = ()`. The impl goes in
`types/teloid_relation/edge_key.rs` and names the trait through `ultragraph`'s re-export, so the
crate gains no dependency:

```rust
impl ultragraph::EdgeKey for TeloidRelation {
    type Key = ();
    fn edge_key(&self) {}
}
```

### 3.2 One relation per pair

**Today.** `link_inheritance(a, b)` followed by `link_defeasance(a, b)` stores two edges.
`resolve_conflicts` reads the relation of a pair with `relation(from, to)`
(`types/effect_ethos/resolve_conflicts.rs:103-111`), which returns the first edge to `to`.
Freezing sorts each adjacency by target: an adjacency below 128 edges with an unstable sort
(`ultragraph/src/types/storage/graph_dynamic/graph_freeze.rs:217-226`), one of 128 or more with a
stable LSD radix sort. In an adjacency below 128 edges, which of the two relations comes first is
not specified.

**After.** With `Key = ()` the second call replaces the first. The pair carries `Defeats`, the
last relation written.

**Open decision.** Last write wins, or the second relation is refused. Recommended: refuse. A
norm that both inherits from another norm and defeats it is a contradiction in the norm base, and
an overwrite hides it from the author. If adopted, `add_inheritance_edge` and `add_defeasance_edge`
(`types/teloid_graph/teloidable.rs`) check for an existing edge of the other relation and return a
new `DeonticError` variant naming both indices. Re-adding the same relation stays accepted.

### 3.3 The identifier map in `EffectEthos::from`

`EffectEthos::from` (`types/effect_ethos/mod.rs:71`) builds its identifier-to-index map by
enumerating `get_all_nodes()`. This gives a node's rank among live nodes, which equals its
position only while no node has been removed. `TeloidGraph` exposes no removal, so the map stays
correct. A removal added to `TeloidGraph` later has to switch this map to positions.

### 3.4 Unchanged

`deontic_inference` checks `is_frozen()` itself and keeps returning
`DeonticError::GraphNotFrozen`. `verify` freezes, checks `has_cycle` and unfreezes on a cycle.
`resolve_conflicts` runs `topological_sort`, `inbound_edges` and `outbound_edges` on the frozen
graph. None of these sees a tombstone, because no teloid is ever removed.

### 3.5 Tests

No test in `deep_causality_ethos/tests`, and no link in the drone tutorial's norm base
(`end_to_end_tutorials/dynamic_drone_failsafe/part_4_effect_ethos/model_config.rs`), adds two
relations to one pair successfully. `teloid_graph_tests.rs:95-96` and `:118-119` attempt both on
one pair, and both attempts fail: the nodes are missing, or the graph is frozen. Add a test for
whichever semantics §3.2 settles on.

## 4. The QCM readers (`deep_causality_quantum`)

### 4.1 Sites

Each sizes its node range by `number_nodes()`; all but the model-subject validation also read
edges through `contains_edge`:

| Site | Reads |
|---|---|
| `FactorSupports::from_graph`, `types/qcm/process_factors.rs:166-200` | parents of each factor's node |
| `CausalStructure::from_graph_reachability`, `types/qcm/faithfulness.rs:95-150` | reachability from each input, by BFS |
| `DemModel::from_graph`, `types/qcm/dem_model.rs:141-175` | role assignment of mechanisms, detectors and observables |
| model-subject validation, `types/pipeline/config.rs:532-552` | factor nodes, inputs and outputs in range |
| `graph_has_cycle`, `types/pipeline/config.rs:595-614` | a directed cycle, by search |
| `Hypothesis::structural_from_graph`, `types/qcm/hypothesis.rs:125` | doc only: "a factor keyed past `number_nodes()`" |

The first four refuse an unfrozen graph and name density as the reason. The docs of
`FactorSupports::from_graph` and `from_graph_reachability` state it in full: "only the static
representation guarantees a dense node-id space `0..number_nodes()`". The errors of all four say
"(dense node ids)". `graph_has_cycle` checks nothing itself and runs after the validation. After
the change that guarantee is gone.

### 4.2 Failure without migration

None of the five panics: each loops over `0..n` and asks `contains_edge`. After a removal and a
freeze, a live node at a position at or above `number_nodes()`:
- is refused as out of range when a caller declares it as a factor, input, output or role;
- has its parent edges skipped in `FactorSupports::from_graph`;
- is never reached by the BFS in `from_graph_reachability`, which the doc there calls "an unsound
  false negative in a faithfulness gate";
- hides a cycle through it from `graph_has_cycle`, which then reports the graph acyclic.

### 4.3 Migration

- Size by `graph.index_bound()` (§2.2) at every site.
- Replace each range check `id >= n` with `!graph.contains_causaloid(id)`. The error messages drop
  "valid ids 0..n" and name the index as holding no causaloid.
- Keep the frozen precondition. `from_graph_reachability` runs inside a
  `freeze_verified_with_check` hook (`types/qcm/markov_freeze.rs:397-410`), on the frozen graph;
  the others are called directly, by `Hypothesis` (`types/qcm/hypothesis.rs:135`, `:225`), the
  pipeline (`types/pipeline/validate.rs:325`) and callers, and refuse an unfrozen graph today.
  The docs and errors state the requirement and drop "(dense node ids)" and the paragraph
  explaining density.

Whether to drop the frozen precondition is a question for a later change. Under permanent IDs a
dynamic graph has the same positions as its frozen form, so density no longer justifies the
precondition.

### 4.4 Tests

Two tests match the range messages §4.3 rewrites, and change with them:
- `tests/types/qcm/hypothesis_tests.rs:161` asserts `"node 9"` and `"0..4"`, from
  `FactorSupports::from_graph`'s "valid ids 0..{n}";
- `tests/types/pipeline/config_tests.rs:96-97` asserts `"node 3"` and `"3 nodes"`, from the
  model-subject validation's "the frozen graph has {n} nodes".

`tests/types/qcm/dem_model_tests.rs:233` asserts `"node 9"` only, and holds while the message
names the node. `pipeline_tests.rs:142` and `dem_model_tests.rs:199` assert `"frozen"`, and hold
while the frozen precondition stays.

New tests, each failing on the code without §4.3: for each reader, a graph with a removed node,
frozen, whose highest live node carries a factor, a role or an edge; the reader accepts the node
and reads its edges. `graph_has_cycle` gets a cycle through a node above `number_nodes()`.

## 5. Tests that change, by file

Found by reading every test that removes a node, adds two edges to one pair, lists neighbours on
a dynamic graph, or matches a range message.

| Test | Asserts today | After | Step |
|---|---|---|---|
| `ultragraph/tests/types/ultra_graph/graph_evolve_tests.rs:91`, `test_freeze_graph_with_tombstoned_nodes` | positions shift after a removal and a freeze | positions stay | 2 |
| same file `:326`, `test_unfreeze_graph_with_tombstoned_nodes_from_original` | positions shift, and stay shifted after `unfreeze` | positions stay | 2 |
| same file `:44`, `test_freeze_graph_with_nodes_and_edges` | `add_edge(n0, n1, 10)` and `add_edge(n0, n1, 15)` give two edges | one edge, weight 15 (`u64` has `Key = ()`) | 3 |
| `ultragraph/tests/types/ultra_graph/graph_traversal_tests.rs:26`, `:64` | traversal on a dynamic graph returns `GraphNotFrozen` | traversal works | 3 |
| `ultragraph/tests/types/ultra_graph/graph_algo_bridges_tests.rs:71`, `:80` | two same-direction edges between one pair are not a bridge | with `W = ()` the second `add_edge` updates the first, so the fixtures can no longer build parallel edges; they move to a payload keyed by identity and a `ParallelEdges::Distinct` projection, where the assertions hold as today, and `Merge` gets tests of its own (parent note, §Design 5) | 3 |
| `deep_causality/src/traits/causable_graph/mod.rs:147`, `test_a_dynamic_graph_yields_no_edges_and_stops_at_the_start` | a dynamic graph yields no edges | the reach on a dynamic graph (§2.5) | 3 |
| `deep_causality_context/tests/types/context_types/context_graph/contextuable_graph_tests.rs:270`, `test_neighbour_listing_needs_a_frozen_context` | listing fails unfrozen | listing in the dynamic state equals listing frozen (§1.4) | 3 |
| same file `:332`, `test_freezing_after_a_removal_keeps_every_index_on_its_node` | `assert_ne!(index(40), i40)`; stale map entries gone after freeze | `index(40) == i40`; stale map entries gone at removal (§1.3) | 2 |
| `deep_causality_context/tests/types/context_types/context_graph/snapshot_tests.rs:246`, `test_a_parallel_edge_is_refused` | `remove_edge` removes one of two kinds | `remove_edge` removes both (§1.5) | 3 |
| `deep_causality_quantum/tests/types/qcm/hypothesis_tests.rs:161` | message holds `"0..4"` | the rewritten message (§4.4) | 2 |
| `deep_causality_quantum/tests/types/pipeline/config_tests.rs:96-97` | message holds `"3 nodes"` | the rewritten message (§4.4) | 2 |
| new, `deep_causality` | | §2.7 | 2 |
| new, `deep_causality_ethos` | | §3.5 | 3 |
| new, `deep_causality_quantum` | | §4.4 | 2 |

`graph_evolve_tests.rs:142` (a removed root) and `:162` (every node removed) keep passing: the
first removes the last position, so no live node moves, and the second asserts counts only.
`context_tests.rs`, `test_get_data_by_id_reads_the_payload_of_the_node_carrying_the_id`, keeps
passing, and its comment "Freezing compacts the removed slot away" changes with step 2.

## 6. Verification

1. `bazel test //...` after each step of §Order of work.
2. Each new test is run against the code before its fix and fails there. A test that passes on the
   unfixed code tests nothing about the fix.
3. The suites of `deep_causality`, `deep_causality_context`, `deep_causality_context_store`,
   `deep_causality_ethos` and `deep_causality_quantum` pass with no changes to their outcomes
   beyond the tests in §5.

## 7. Open decisions

| # | Decision | Recommendation |
|---|---|---|
| 1 | Typed edges in the context (§1.9) | Path A now; Path B as its own change if wanted |
| 2 | A second relation on one teloid pair (§3.2) | refuse |
| 3 | Stale index-map entries in the context (§1.3) | drop at removal |
| 4 | The QCM readers' frozen precondition (§4.3) | keep for this update |

Two rulings in the parent note bear on the migration. `clear` wipes the graph and restarts positions
at 0 (§Design 1), so an index held across `CausaloidGraph::clear` or `TeloidGraph::clear` is void.
Each projection chooses whether parallel edges merge or count separately (`ParallelEdges`, §Design
5); algorithms called on a graph directly merge them, which changes no result DeepCausality's
callers read.
