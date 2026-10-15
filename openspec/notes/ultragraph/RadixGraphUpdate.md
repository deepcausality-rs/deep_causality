# UltraGraph update: stable node IDs, keyed edges, three states and projections, after RadixGraph

* Date: 2026-10-15
* Scope: `deep_causality/ultragraph` (version 0.9.9) and the DeepCausality crates that call it.
* Grounded in: UltraGraph's source as of this date (paths below are relative to the `deep_causality`
  repository); RadixGraph's paper (`ultragraph/papers/RadixGraph.pdf`) and code at commit `9feba53`
  (§Sources).
* Decided by Marvin on 2026-10-15: parallel edges keyed by target and key (§Design 2), inbound
  traversal in the dynamic state (§Design 3), the benchmark that decides whether snapshot-log edge
  arrays replace today's edge lists (§Benchmark), algorithms on the simple projection (§Design 5),
  projections that filter nodes and edges by predicate, as a view, as a frozen view on a
  materialised CSR and as a view maintained eagerly on every mutation, each projection choosing
  whether parallel edges merge or count separately (`ParallelEdges`, §Design 5), the graph holding its
  maintained views through one `Projection<N, W>` type, with pattern matching left to the storage
  layer (§Design 6), three states (Dynamic, Live, Frozen) with `freeze` in place and `freeze_clone`
  as a frozen copy that leaves the maintained views behind (§Design 7), the `ultragraph_traits` crate holding `EdgeKey`
  (§Crates), the causaloid graph's `u64` as a weight (§Changes visible to callers), and
  `TeloidRelation` with `Key = ()` (§Crates).
* Migration of every downstream type: `openspec/notes/ultragraph/RadixGraphUpdateMigration.md`.

## Summary

1. **Node IDs are permanent.** A node keeps the position `add_node` returned for its whole life,
   through any sequence of mutations, freezes and unfreezes. A removed node's position stays a
   tombstone and is never issued again until `clear` wipes the graph. `freeze` stops renumbering. `compact_ids` reclaims the space
   of removed nodes on request and returns the mapping from old to new positions.
2. **Edges are keyed by target and key.** A graph holds at most one edge per source, target and
   key, where the key comes from the edge's payload `W` through a trait `EdgeKey`: nothing for a
   weight, a kind for a typed relation, or the edge's own identity where several relations of one
   kind may join one pair. Edges with different keys between the same two nodes coexist; a second
   edge with the same key updates the first. `EdgeKey` lives in a new zero-dependency crate,
   `ultragraph_traits`.
3. **The dynamic state traverses in both directions.** Every node keeps a forward and a backward edge
   array; `outbound_edges` and `inbound_edges` work without freezing; `remove_node` removes the edges
   into the node as well as out of it.
4. **Algorithms run on a projection.** A projection keeps the nodes and edges that pass two
   predicates the caller gives, one on `N` and one on `W`, and a mode for parallel edges:
   `ParallelEdges::Merge` counts the edges between two nodes as one connection (a weighted path
   takes the lightest), `ParallelEdges::Distinct` counts each as its own link (§Design 5, §Design
   6). Algorithms called on the graph directly run on the simple projection: every node and edge,
   `Merge`. The algorithms DeepCausality calls (`has_cycle`, `topological_sort`, `shortest_path`)
   return what they return today. Under `Merge`, bridges and directed betweenness change on graphs
   with parallel edges; `Distinct` keeps today's results for them.
5. **A benchmark decides the edge structure.** RadixGraph's per-node snapshot-log arrays replace
   today's per-node `Vec` only if they match it on building, traversal and freezing and win on
   updates and removals; it also tests whether the Live state earns its place (§Benchmark). Items
   1–4 hold under either structure.
6. **Versioned reads, concurrent readers, persistence and an external-ID index stay out of
   UltraGraph.** They belong to a storage layer built on it. UltraGraph stays a single-owner,
   deterministic, `no_std`-capable data structure with unsafe code forbidden (the workspace lint `unsafe_code = "forbid"`, which `ultragraph`
   inherits through `[lints] workspace = true`), whose only dependency is `ultragraph_traits`. It
   has no runtime dependency today.
7. **The graph has three states.** Dynamic, the default, serves the application: cheapest
   mutations, traversal in both directions. Live keeps the graph in read-optimised rows that take
   changes locally, so algorithms run on current data. Frozen is one contiguous CSR for batch
   analytics. `freeze` converts the graph in place; `freeze_clone` returns a frozen copy and leaves
   the mutable graph as it was. Every state keeps the same positions (§Design 7).

Item 1 fixes a correctness hazard callers work around today, so it lands first, together with the
downstream changes that read node indices (§Changes visible to callers). Without them, once a node
has been removed and the graph frozen, the causaloid graph's reasoning can panic and the context's
index maps name the wrong nodes.

## RadixGraph in brief

RadixGraph (Xie, Liu, Luo, Wang; accepted at SIGMOD 2026; arXiv 2601.01444; C++, Apache-2.0) is
an in-memory dynamic graph store. The paper describes these parts:

| Part | What it does, per the paper |
|---|---|
| SORT, a space-optimised radix tree | Maps an external vertex ID to the vertex's offset in a vertex table of 32-byte blocks. For a fixed number of layers, the fan-out of each layer comes from a dynamic program that minimises expected space for n uniformly distributed IDs of x bits. Re-tuning happens roughly when n doubles; a transformation took 0.23 s to 1.96 s for n from 0.31M to 2.7M (Fig. 12d) |
| Snapshot-log edge arrays | Each vertex's edge array, of capacity 2d, is split evenly into a snapshot segment and a log segment. Inserts, updates and deletes append to the log through an atomic `fetch_add`; a delete is a log entry with a NULL weight. A full log triggers compaction of that vertex alone: a scan from the newest entry to the oldest keeps the latest entry per destination, using a per-thread bitmap (the "duplicate checker"), into a new array of capacity 2d. Insertion, update and deletion cost amortised O(1) on the edge array, independent of degree (proved for single-threaded execution); each also locates its two vertices in SORT, at O(lg lg u). Reading the neighbours costs O(d) |
| Edge chain | An edge stores the destination's offset, not its ID, so a traversal looks up SORT once, at the start vertex |
| Versions | Log entries carry timestamps, and compactions create versioned arrays kept in a linked list. A reader at time t reads the newest version created before t and filters the log to entries at or before t. A version is freed once it has been compacted and no reader holds it |

Measured by the authors on a dual-socket Xeon Gold 6326 machine (16 cores per socket) with 250 GB
of memory, using 64 threads:
- single-edge insertion up to 16.27× the throughput of Spruce, on Twitter, where Spruce was the only
  one of Teseo, Sortledton, Spruce and GTX to finish; Spruce is fastest on the uniform graph u24;
- 40.1 % less memory, on average, than the second-lowest of those four in the same experiment;
- in the batch-update experiment (Table 7), the Twitter graph (41M vertices, 1.46B edges) in
  26.25 G, against 47.06 G (Terrace), 66.78 G (Aspen) and 69.60 G (CPAM), and LiveJournal batch
  insertion from 6.58E5 at batch size 10 to 6.18E7 at batch size 10⁴ (the table states no unit);
- without the edge chain (Table 6): BFS up to 27.67× slower (LiveJournal), 2-hop queries 1.04× to
  2.10× slower (the maximum on u24), SSSP up to 5.90× and betweenness 1.12× to 1.56×.

The experiments treat every graph as undirected and insert each edge in both directions, so each
logical edge costs two edge writes, as it will in UltraGraph's dynamic state with backward arrays.

The code (commit `9feba53`) differs from the paper where UltraGraph would care:
- an edge holds an `int` index into the vertex table, not a byte offset, so the table holds at most
  about 2³¹ vertices; external IDs are `uint32_t`;
- a weight of 0 marks a delete, so an edge cannot carry a weight of 0;
- `InsertEdge` does not look for an existing edge and adds every insert to the degree
  (`src/radixgraph.cpp:61-66`); when every log entry is an insert, compaction copies the array
  without the duplicate checker (`:206-212`), so a duplicate insert survives. The code calls
  duplicate edges invalid operations (`:223`);
- SORT is built once for a given n; `SORT::Transform` exists but is called only from a test;
- versioning (timestamps, multi-versioned arrays) runs only when the runtime flag
  `global_info.is_mixed_workloads` is set, which is off by default;
- vertex-offset reuse is compiled out by default (`ENABLE_GARBAGE_COLLECTION 0`).

Its limits: memory only; no multi-operation transactions, which the paper names an open problem;
a `float` weight as the only edge payload, with multigraphs described as an extension; `int`
timestamps. Its baselines differ in transactional support, as the paper notes: Teseo, Sortledton
and GTX are transactional, Spruce is concurrent, and Terrace, Aspen and CPAM target batch updates.
None is a plain per-vertex vector, which is what UltraGraph's dynamic state is; the benchmark below
measures RadixGraph's edge arrays against that.

## UltraGraph today

| Aspect | Behaviour | Where |
|---|---|---|
| States | `GraphState` is either `Dynamic(DynamicGraph)` or `Static(CsmGraph)`; `freeze` and `unfreeze` convert the whole graph, each O(V+E) | `ultragraph/src/types/storage/graph_state/mod.rs:11-17` |
| Dynamic nodes | `nodes: Vec<Option<N>>`; `remove_node` writes `None` and clears the node's own outgoing list | `graph_dynamic/mod.rs:28`, `graph_dynamic/graph_mut.rs:57-65` |
| Dynamic edges | `edges: Vec<Vec<(usize, W)>>`, one outgoing list per node, holding target positions; no backward lists | `graph_dynamic/mod.rs:32` |
| `add_node` | Always appends; a removed position is never reused while the graph stays dynamic | `graph_dynamic/graph_mut.rs:16-27` |
| `add_edge` | Pushes, without a duplicate check, so `add_edge(a, b)` twice stores two edges | `graph_dynamic/graph_mut.rs:83-95` |
| `remove_edge` | Scans the source's list for the first edge to the target, then `swap_remove`, which moves the list's last edge into the gap: O(d) | `graph_dynamic/graph_mut.rs:105-122` |
| `clear` | Empties the node table, so the next `add_node` returns 0 again; in the frozen state the container replaces the graph with an empty dynamic one | `graph_dynamic/graph_mut.rs:137-142`, `ultra_graph/graph_mut.rs:93-98` |
| Edges into a removed node | Stay in their sources' lists; `contains_edge` and `get_edges` filter them, `number_edges` counts them, and `freeze` drops them | `graph_dynamic/graph_view.rs:46-91`, `graph_dynamic/graph_freeze.rs:99-111` |
| Traversal | `outbound_edges` and `inbound_edges` return `GraphNotFrozen` in the dynamic state, and `NodeNotFound` for an index the frozen graph does not hold | `ultra_graph/graph_traversal.rs:30,52`, `graph_csm/graph_traversal.rs:21-23,42-44` |
| Frozen nodes | `CsmGraph.nodes: Vec<N>`, holding live nodes only; `number_nodes` is `nodes.len()` | `graph_csm/mod.rs:43`, `graph_csm/graph_csm_view.rs:41` |
| **`freeze` renumbers** | Drops tombstoned nodes and gives every live node its rank among live nodes (`remapping_table`), remapping edges and the root; a graph with no live node freezes to `CsmGraph::new()` | `graph_dynamic/graph_freeze.rs:55-149`; empty: `:57-59`, `:91-93` |
| Frozen adjacency order | Sorted by target: an unstable comparison sort below 128 edges, a stable LSD radix sort at 128 or more | `graph_dynamic/graph_freeze.rs:213-236` |
| Frozen `contains_edge` | A linear scan below 64 neighbours, a binary search at 64 or more | `graph_csm/graph_csm_view.rs:47-70` |
| `unfreeze` | Keeps frozen positions one-to-one | `graph_csm/graph_csm_unfreeze.rs:49` |
| Algorithms | Every algorithm sizes its arrays and loops by `0..number_nodes()`, assuming every frozen position holds a live node | `graph_csm/graph_csm_algo_*.rs`, `graph_csm_biconnectivity_common.rs` |
| Parallel edges in the algorithms | Bridges, articulation points and biconnected components run on an undirected view that keeps same-direction parallel edges with their multiplicity and joins `u → v` with `v → u` into one undirected edge, so two parallel edges are not a bridge; directed betweenness counts each parallel edge as its own path; undirected betweenness deduplicates neighbours | `graph_csm_biconnectivity_common.rs:9-22,51-86`, `graph_csm_algo_centrality.rs:182-205,219-220`; tests `graph_algo_bridges_tests.rs:64-84` |

**The ID problem.** After `remove_node` and `freeze`, every node above the removed position moves
down. Any index a caller kept from `add_node` names a different node, or none. UltraGraph's own test
pins this behaviour (`ultragraph/tests/types/ultra_graph/graph_evolve_tests.rs:91-139`: "n2 (idx 2)
… New: … n2 (idx 1)"; `:326-378` asserts the same after an `unfreeze`). DeepCausality's context graph repairs it on every freeze: it computes each
live node's rank before freezing and rewrites five maps through it
(`deep_causality_context/src/types/context_types/context_graph/contextuable_graph.rs:173-207`).
Every other caller that keeps indices across a removal and a freeze carries the same hazard without
the repair.

**The edge-kind problem.** The context graph's edges carry a `RelationKind` (Datial, Temporal,
Spatial, SpaceTemporal), and the in-memory graph and the store disagree on what identifies an edge.
`Context::add_edge` performs no duplicate check, so a second edge between one pair, of the same
kind or another, is stored as a parallel edge; `snapshot` then refuses the context with "an edge is
carried twice". The store holds one relation per pair: `ContextStorage::create_edge` refuses a
second kind between the same pair
(`deep_causality_context_store/src/traits/context_storage.rs:31-33`), and `EdgeRetracted` names a
pair without a kind. `connect` and `sever`
(`deep_causality_context/src/types/context_types/context_graph/apply.rs:241-273`) follow the store:
`connect` writes nothing when `contains_edge(a, b)` holds, and `sever` removes by pair. The
duplicate checks the context relies on sit in `connect` and `restore`, not in the graph.

## What transfers from RadixGraph

| RadixGraph | UltraGraph | Decision |
|---|---|---|
| SORT, an external-ID index | UltraGraph addresses nodes by position, and callers keep their own ID maps | Stays out: an ID index belongs to the caller or the store |
| Edge chain | Already present: edges hold target positions in both states | Unchanged |
| Snapshot-log edge arrays with per-node compaction | Per-node `Vec`, push O(1), removal O(d), no update | Adopted if the benchmark favours it |
| Versioned reads | None | Stays out of UltraGraph; the store keeps versions by sharing frozen snapshots and its log |
| Lock-free appends for concurrent readers | One owner; the borrow checker separates `&mut` from `&` | Stays out |
| Offset reuse through a free queue (in the paper; compiled out by default in the code) | Not present | Stays out: a reused position makes a stale index name another node |

## Design

### 1. Permanent node IDs

**Positions are permanent and never reused.** The node table only grows. `remove_node` turns a
position into a tombstone for good. `contains_node` is false for it, `get_node` returns `None`, and
`add_node` never returns it again. This matches the rule DeepBrain's `IdAllocator` trait states for
its own identities (in DeepBrain's code, outside this repository): "none ever reissued".

**A tombstone is a removed node's slot, excluded from every traversal.** `remove_node` drops the
node's payload, as it does today (`graph_dynamic/graph_mut.rs:62`); the slot stays so that no other
position moves, and only `compact_ids` or `clear` reclaims it. No traversal reaches a tombstone: in
the Dynamic state its edges are removed with it (§Design 3), in the Live state its mask bit is clear
and readers skip it (§Design 7), and in the Frozen state its rows are empty. A traversal started at a
tombstone returns `GraphError::NodeNotFound`, as one started at a position never issued does, in
every state.

**`clear` wipes the graph.** It drops every node, edge, tombstone and the root, and returns the graph
to an empty Dynamic state from any state, as it does today (`graph_dynamic/graph_mut.rs:137-142`,
`ultra_graph/graph_mut.rs:93-98`). Positions restart at 0, so every index a caller holds is void
after `clear`; in DeepCausality the callers are `CausaloidGraph::clear` and `TeloidGraph::clear`.
Maintained views stay registered and become empty, as invariant 8 requires.

**Every state keeps tombstones.** The bullets below name the Frozen state; the Live state (§Design 7)
keeps them the same way, as a cleared mask bit and an empty row.
- `CsmGraph.nodes` becomes `Vec<Option<N>>`. The CSR offsets cover every position; a tombstone has an
  empty adjacency in both directions. `freeze` maps every position to itself.
- `freeze` keeps the full node table, trailing tombstones included, also when no node is live.
  Today's shortcuts that return `CsmGraph::new()` for a graph with no live node
  (`graph_freeze.rs:57-59`, `:91-93`) would reset the table, and the next `add_node` after
  `unfreeze` would return position 0 again, breaking invariant 2.
- `unfreeze` carries the `Option`s across, so `unfreeze(freeze(g))` and `freeze(unfreeze(g))` keep
  every position.
- `contains_node`, `get_node`, `get_all_nodes` and `is_empty` treat a tombstone as absent in both
  states. Today the frozen versions read `nodes` directly (`graph_csm_view.rs:24-43,79-81`) and
  change with the `Option`.
- The root keeps its position, and a removed root reads as no root, as today.

**Counting and iteration.**
- `number_nodes` means the number of live nodes in every state. `CsmGraph` keeps a live count, so it
  stays O(1).
- A crate-internal `node_slots()` (the length of the node table) sizes the algorithms' arrays. Every
  algorithm loops over `0..node_slots()`, skips tombstones, and leaves them out of its results:
  topological order, SCCs, articulation points, bridges, biconnected components, centrality scores,
  paths.
- `get_last_index` returns the highest live position in every state (the dynamic state already does).
  `get_last_index() + 1` is the bound a caller outside the crate sizes per-node arrays by;
  `number_nodes()` undercounts it after a removal. `CausableGraph` exposes it as `index_bound()`
  (migration note, §2.2).

**Reclaiming space is explicit.** `compact_ids(&mut self) -> Vec<Option<usize>>` drops tombstones,
renumbers live nodes in order and returns, for each old position, its new position or `None`. It
lives on `DynamicGraph`, where tombstones are created, and `UltraGraphContainer` delegates to it in
the dynamic state. A caller that wants the space back calls it and remaps its own indices with the
table: the work `contextuable_graph.rs:173-207` does today, moved behind a call the caller makes on
purpose.

**What a tombstone costs.** About `size_of::<Option<N>>()` plus two CSR offsets, one in the forward
and one in the backward `offsets` array (each a `usize`, 8 bytes on a 64-bit target), in the frozen
state, and two empty edge arrays in the dynamic state. A graph that removes most of what it adds
grows without bound until `compact_ids` runs.

### 2. Keyed parallel edges

**An edge is identified by its source, its target and its key.** The key comes from the payload,
through a trait defined in `ultragraph_traits` and re-exported by `ultragraph`:

```rust
pub trait EdgeKey {
    type Key: Copy + Ord;
    fn edge_key(&self) -> Self::Key;
}
```

`Key` is `Ord` because the frozen adjacency sorts by target, then key. `RelationKind` derives
`PartialOrd` and `Ord` for it.

The payload chooses what identifies an edge between two nodes:

| `Key` | One pair holds | Example |
|---|---|---|
| `()` | at most one edge; a second `add_edge` updates it | a weight (`u64` in the causaloid graph), no payload (`UltraGraph<T>`), `TeloidRelation` |
| a kind | at most one edge per kind | `RelationKind` in the context graph; edges written with `MERGE`-style semantics, one per pair and type |
| the edge's own identity | any number of edges, of any kinds | edges that carry their own identity, provenance and validity window, such as two reports asserting the same cause, or a closed and an open relation after a revision (an identifier such as a ULID as `u128`) |

A kind or an identity identifies an edge; a weight describes one. A payload that carries a weight
and a kind is a struct whose `Key` is the kind; one that carries an identity keys by the identity
and keeps its kind and weight as data.

- **`add_edge(a, b, w)`** inserts the edge `(a, b, w.edge_key())`, or replaces its payload when that
  edge exists. Edges with other keys between `a` and `b` stay.
- **`remove_edge(a, b)`** removes every edge from `a` to `b`, whatever its key.
  **`remove_edge_keyed(a, b, k)`** removes the edge with key `k` alone.
- **`contains_edge(a, b)`** is true when an edge with any key exists;
  **`contains_edge_keyed(a, b, k)`** tests one key.
- **`get_edges(a)`** and the traversals return every edge, keys included through `W`.
- `ultragraph_traits` implements `EdgeKey` for `()` and the primitive numeric types with `Key = ()`.
- In the Live and the Frozen states, each adjacency is sorted by target, then key, so `contains_edge` keeps
  today's search (a linear scan below 64 neighbours, a binary search above) and the edges of one
  pair sit together.
- With `Key = ()` a graph cannot hold two edges between one ordered pair. `UltraGraph<T>`
  (`W = ()`) and every graph with a numeric weight lose the parallel edges they can hold today;
  the tests that build such multigraphs change (migration note, §5).

### 3. Inbound traversal in the dynamic state

**Every node has a forward and a backward edge array.** Every edge write appends to the source's
forward array and the target's backward array, so `outbound_edges` and `inbound_edges` both work in
the dynamic state, and `UltraGraphContainer` delegates them in every state; `GraphNotFrozen` no
longer comes from traversal. The algorithms of `GraphAlgorithms` run in the Live and the Frozen
states (§Design 7), not in the Dynamic state.

With the backward array:
- `remove_node` removes every edge into the node from its predecessors' forward arrays, so no edge
  in the dynamic state points at a tombstone, and `number_edges` counts live edges only.
- `freeze` reads each node's in-degree from the length of its backward array. Today it counts
  in-degrees alongside out-degrees in one pass over the forward lists (`graph_freeze.rs:96-111`).

The cost is a second write per edge mutation and twice the edge memory in the dynamic state. The
frozen state already stores both directions.

### 4. Edge storage: two candidates, chosen by the benchmark

Both candidates implement §1–3; they differ in how a node's edges are held.

- **Vec lists (today's structure, extended).** A forward and a backward `Vec<(usize, W)>` per node.
  Keyed insert and removal scan the list for `(target, key)`: O(d).
- **Snapshot-log arrays (RadixGraph).** Each array is
  `EdgeArray { snapshot: Vec<(usize, W)>, log: Vec<LogEntry<W>> }`, with
  `LogEntry { target: usize, op: EdgeOp<W> }` and `EdgeOp::{Put(W), Remove(W::Key), RemoveAll}`; an enum
  carries removal in place of the paper's NULL weight. Inserts, updates and removals append one
  entry: amortised O(1). When `log.len()` reaches `max(snapshot.len(), MIN_LOG)`, the node compacts
  its own array: a newest-first scan of the log decides each `(target, key)` once, through a bitset
  the graph owns and reuses over targets plus a comparison of keys among the entries for one
  target; the latest `Put` survives, a `Remove` or `RemoveAll` suppresses older entries, and snapshot
  entries the log decided, or whose target is a tombstone, are dropped. `freeze` compacts every node
  and concatenates the snapshots; `unfreeze` turns each CSR slice into a snapshot with an empty log.

Both are `Vec`s and a `Vec<u64>` bitset: `no_std` with `alloc`, no dependencies, no `unsafe`.

### 5. Algorithms on the simple projection

**Every algorithm in `GraphAlgorithms` runs on a projection of the graph in the Live or the Frozen
state (§Design 6, §Design 7); the
simple projection is the one whose predicates pass every node and edge:** one connection per
ordered pair of nodes, whatever the number of edges between them. The undirected
algorithms (articulation points, bridges, biconnected components, undirected betweenness) use one
undirected connection per unordered pair, so `u → v` and `v → u` count once, as they do today. A
weighted path takes, for each connection, the lightest of its parallel edges, by `W`'s order. The
frozen adjacency is sorted by target, then key, so the simple projection costs nothing to build: an
algorithm skips repeated targets as it iterates a node's edges.

Compared with today, the simple projection changes the result of two kinds of algorithm on a graph
with parallel edges and leaves the rest alone:

| Algorithm | On the simple projection (`Merge`, below), against today |
|---|---|
| Reachability, cycle detection, topological sort, SCCs, unweighted shortest path, undirected betweenness | Same result |
| Articulation points, biconnected components | Same result: whether two nodes are joined by one edge or several does not change which nodes separate the graph, and `biconnected_components` returns vertex sets. Measured on the fixtures below |
| Weighted shortest path | Same result: today's Dijkstra relaxes every parallel edge and keeps the lightest (`graph_csm_algo_pathfinder.rs:165-174`) |
| `betweenness_centrality` and `pathway_betweenness_centrality` with `directed = true` | Changes: today each parallel edge adds its own path to the counts (`graph_csm_algo_centrality.rs:182-187,219-220`); on the projection a pair contributes one |
| Bridges | Changes: today two same-direction edges between `u` and `v` are not a bridge; on the projection they are one connection, which is a bridge when removing it disconnects its ends |

Run against `ultragraph` 0.9.9 on graphs that differ only by a doubled edge, outside the repository
(a scratch crate with a path dependency):

| Graph | Bridges | Biconnected components | Articulation points |
|---|---|---|---|
| `0–1`, `1–2` | `(0,1)`, `(1,2)` | `{0,1}`, `{1,2}` | `1` |
| `0–1` twice, `1–2` | `(1,2)` | `{0,1}`, `{1,2}` | `1` |
| triangle `0,1,2`, `2–3` | `(2,3)` | `{0,1,2}`, `{2,3}` | `2` |
| triangle `0,1,2`, `2–3` twice | none | `{0,1,2}`, `{2,3}` | `2` |
| `0→1`, `1→0`, `1–2` | `(0,1)`, `(1,2)` | `{0,1}`, `{1,2}` | `1` |

The doubled edge changes the bridges and nothing else. A pair stored as `u → v` and `v → u` already
counts as one undirected edge today.

**Each projection chooses how parallel edges count.** Whether several edges between two nodes are
one relationship or independent links depends on what the edges mean: two relations of different
kinds, or two reports of one cause, are one relationship; two cables, each keyed by its own
identity, are two links. So the projection carries a mode:

```rust
pub enum ParallelEdges {
    /// One connection per pair, the lightest edge for a weighted path.
    Merge,
    /// Every passing edge counts as its own link.
    Distinct,
}
```

| Algorithm | `Merge` | `Distinct` |
|---|---|---|
| Bridges | Parallel edges between `u` and `v` are one connection, a bridge when removing it disconnects its ends | Two parallel edges between `u` and `v` are not a bridge, as today; `u → v` with `v → u` still counts once, as today |
| `betweenness_centrality`, `pathway_betweenness_centrality`, `directed = true` | A pair contributes one path | Each parallel edge adds its own path, as today |
| The same, `directed = false` | Neighbours deduplicated, as today | Each parallel edge adds its own path; today deduplicates, so this changes |
| Every other algorithm | Same result in both modes (the table above) | |

Algorithms called on the graph directly run on the simple projection: every node and edge, in
`Merge`. A caller who needs multiplicity builds a projection in `Distinct` (§Design 6). No algorithm
signature changes.

`graph_csm_biconnectivity_common.rs:16-22` keeps same-direction parallel edges with their
multiplicity on purpose, because "collapsing them would turn a multigraph into a simple graph and
yield wrong bridges/biconnected components (e.g., two parallel edges between `u` and `v` are not a
bridge, but a collapsed single edge would be)". `Distinct` keeps that behaviour available;
`build_symmetric_adjacency` (`:51-86`) counts multiplicity in `Distinct` and one undirected edge per
pair in `Merge`. The measurement above shows the choice matters for bridges only; the comment's
claim about biconnected components does not hold for the vertex sets `biconnected_components`
returns, and the module doc is corrected. The tests `parallel_multi_edges_are_not_bridges` and
`bridge_plus_parallel_multi_edge_isolates_correctly` (`graph_algo_bridges_tests.rs:71-84`) build
their parallel edges with `W = ()`, which `Key = ()` now collapses into one edge; they move to a
payload keyed by identity and a `Distinct` projection, and `Merge` gets tests of its own.

This follows established practice: an algorithm runs on a graph whose edge semantics the caller
chose, not on the multiplicity storage holds. NetworkX separates simple graph types (`Graph`,
`DiGraph`) from multigraph types (`MultiGraph`, `MultiDiGraph`); igraph's `simplify()` collapses
parallel edges into one, dropping their attributes unless `combine_edges` says how to combine
them; Neo4j's Graph Data Science runs every algorithm on a projection that names the relationship
types it includes and aggregates parallel relationships (`NONE`, `SINGLE`, `COUNT`, `MIN`, `MAX`,
`SUM`).

DeepCausality's callers can build parallel edges today: `Context::add_edge`, a second
`CausaloidGraph::add_edge` on one pair, and `TeloidGraph` with both relations on one pair all store
a second edge. The algorithms they call, across the crates, examples and tutorials, are
`has_cycle`, `topological_sort` and `shortest_path`, whose results on the simple projection equal
their results on the edges as stored, so those results stay the same. What changes for them is the
edge count where a same-key duplicate collapses into one edge.

### 6. Projections that filter nodes and edges

**An algorithm runs on a projection: the nodes and edges of the graph that pass two predicates.**

```rust
let scope = (|n: &N| n.zone() == zone, |w: &W| w.valid_at(t));
let view = graph.project(scope, ParallelEdges::Merge)?;
let path = view.shortest_path(a, b)?;
```

- A node passes when the node predicate holds for its payload. A tombstone never passes.
- An edge passes when the edge predicate holds for its payload and both of its endpoints pass.
- Parallel edges that pass count as the projection's `ParallelEdges` mode says: one connection
  (`Merge`) or one link each (`Distinct`), §Design 5.
- A projection renumbers nothing: every result names positions as `add_node` issued them.
- `project` requires the Live or the Frozen state (§Design 7) and returns an error in the Dynamic
  state, as the algorithms do. `GraphAlgorithms` and the traversals run on the projection.
- The predicates are a `Projection` (below): a pair of closures or a type the caller defines,
  statically dispatched either way. They read only the payload; a predicate that depends on a time,
  a zone or a set of kinds captures it.

**Relation to `MATCH`.** In a labelled property graph, a projection is the subgraph that two
single-element patterns select: `MATCH (n) WHERE p(n)` for the nodes and
`MATCH (a)-[e]->(b) WHERE q(e) AND p(a) AND p(b)` for the edges. Neo4j GDS builds its Cypher
projections the same way, from a `MATCH (source)-[r]->(target)` query filtered with `WHERE` and
passed to `gds.graph.project`, and runs its algorithms on the in-memory graph that results. Labels
and properties are fields of `N` and `W`. General pattern matching (shapes of several hops,
variable-length paths, conditions that relate two matched elements, results as tables of bindings)
is a query engine. It belongs to the storage layer built on UltraGraph, not to UltraGraph.

**A validity window is payload, not a version.** An edge whose payload carries `valid_from` and
`valid_until` is filtered by an edge predicate that reads them, so "valid now" and "valid at time
t" are projections. Versioned reads, which return the graph as it stood at an earlier state, stay in
the storage layer (Summary, item 6).

**What projections serve.** DeepBrain's scoped questions: a zone (a node predicate on zone
membership; an edge between two zones fails because one endpoint fails), validity at a time (an
edge predicate on the window), and a walk along one relation kind (an edge predicate on the kind).

**A projection comes in three forms.**

```rust
let view = graph.project(scope, ParallelEdges::Merge)?;
let frozen = view.materialize();                        // keeps the view's mode
let id = graph.maintain(Scope::Zone(zone), ParallelEdges::Distinct);   // kept current
let live = graph.view(id);
```

| Form | Holds | Costs | Fits |
|---|---|---|---|
| View (`project`) | A borrow of the graph and the two predicates | Nothing to build; every node and edge an algorithm visits is tested | One query, or predicates that change from query to query |
| Frozen view on a materialised CSR (`materialize`) | A CSR of its own holding the passing nodes and edges, frozen | O(slots + passing edges) to build, payloads cloned (`N: Clone`, `W: Clone`); no predicate cost afterwards | Repeated queries over one scope that need not follow later writes |
| Maintained view (`maintain`) | Rows of its own, in the Live layout (§Design 7), owned by the graph | O(slots + passing edges) to build; then, on every mutation, a predicate test and at most a local row change per registered view | Real-time analytics over one scope that must follow every write |

The materialised CSR and the maintained view keep the graph's positions: each covers every slot,
and a node that fails the node predicate reads as absent, as a tombstone does. The materialised CSR
is a copy and does not follow later changes. All three forms run `GraphAlgorithms` and the
traversals, and give the same results on the same graph (invariant 7).

**Maintenance is eager.** The graph owns its maintained views. Every mutation, in the Dynamic or the
Live state, is tested against each registered view's predicates and applied to the rows it affects:
an inserted or removed edge changes at most the one connection it belongs to; a node whose payload
starts or stops passing flips one bit of the view's node mask. There is no refresh call. The write
cost of a mutation grows with the number of registered views. Because the graph owns the views, a
mutation takes `&mut self` once and updates the graph and its views together, and a read borrows the
graph and the view together; the single-owner rule holds.

**One projection type per graph.** The graph stores its maintained views under one type that
implements a trait defined in `ultragraph` (it moves to `ultragraph_traits` only if a crate needs it
without the in-memory graph, §Crates):

```rust
pub trait Projection<N, W> {
    fn node(&self, node: &N) -> bool;
    fn edge(&self, edge: &W) -> bool;
}
```

A caller that maintains several scopes implements it on an enum, one variant per scope, so every
test is statically dispatched. The container takes the type as a parameter with a default that
maintains nothing; the aliases `UltraGraph<T>` and `UltraGraphWeighted<T, W>` keep that default, so
code that names the graph through them compiles unchanged. A pair of closures, a node test and an
edge test, also implements `Projection`, so `project` and `materialize` take either. Boxed closures
would need `dyn`, which the code conventions exclude, and `fn` pointers cannot carry a zone or a
time.

A maintained view's predicates are re-tested only when a mutation touches an element, so they must
depend on payloads and on parameters fixed when the view is registered. "Valid at time t" and
"open" (no `valid_until`) stay correct; "valid now", read from a clock, would go stale as time
passes without a write. Closing a relation is a write that sets `valid_until`, so a view of open
relations follows it.

### 7. Three states

**The graph is in one of three states at a time, and every state keeps the same positions.**

| State | Layout | Mutations | Traversal | Algorithms | Serves |
|---|---|---|---|---|---|
| Dynamic (default) | Per-node forward and backward edge arrays (§Design 3, §Design 4) | Yes, cheapest | Yes | No | The application |
| Live | Rows packed in one arena, sorted by (target, key), with spare capacity; per node a `(start, len, cap)` record and a mask bit | Yes, local | Yes | Yes, on current data | Real-time analytics |
| Frozen | One contiguous CSR, as today | No | Yes | Yes, fastest | Batch analytics over large graphs |

**Live rows change locally.** A plain CSR cannot: when one row grows, every later offset and edge
moves, O(E). A Live row has its own `(start, len, cap)`, so it need not sit next to its neighbour.
A change that fits the row's spare capacity is written in place. A row that outgrows its capacity
is rebuilt alone at a new place with twice the capacity: O(d) for that row, amortised O(1) per
change, by the argument of RadixGraph's Theorem 2. An inserted or removed edge touches two rows, the
source's forward row and the target's backward row. A removed node flips its mask bit; readers skip
targets whose bit is clear, as `get_edges` skips tombstones in the dynamic state today, and the
stale entries leave at the next rebuild of their rows. Moved rows leave holes, which close when the
graph freezes.

**Transitions.** The costs are estimates; the benchmark measures them.

| From → to | Operation | Cost |
|---|---|---|
| Dynamic → Live | `live` (name provisional): pack and sort every row | O(V + E log d), as `freeze` today |
| Frozen → Live | `live`: give each CSR row a record with `cap = len`, in place; the first write to a row moves it | O(V) |
| Dynamic → Frozen, in place | `freeze`, as today | O(V + E log d) |
| Live → Frozen, in place | `freeze`: close holes, drop spare capacity; rows are already sorted | O(V + E) |
| Dynamic or Live → a frozen copy | `freeze_clone`: copy the graph without its maintained views, freeze the copy; the source keeps its state, its views, and stays mutable | As `freeze` from that state, plus the copy; twice the memory while both exist |
| Live or Frozen → Dynamic | `unfreeze`: split rows into per-node arrays | O(V + E) |

`freeze` in place suits a graph whose writes can stop for the analysis, or one too large to copy.
`freeze_clone` suits a graph small enough to double whose mutable state must carry on; it needs
`N: Clone` and `W: Clone`, bounds the container already has. A frozen copy exists to run
algorithms on, so it leaves the maintained views behind; a second operation that copies them too
can be added if a use appears.

`GraphState` gains a `Live` variant. It is a public enum without `#[non_exhaustive]`, so the new
variant breaks an exhaustive `match` outside the crate; no crate in this workspace matches on it.
`is_frozen` stays true for Frozen only. DeepCausality's callers never enter Live, so their checks of
`is_frozen` before running an algorithm keep their meaning.

**Live must earn its place.** If the benchmark adopts snapshot-log arrays, the Dynamic state already
holds sorted snapshot rows plus a short log per node, and algorithms could read them by merging the
log. Live would then differ from Dynamic by packing rows into one arena instead of one allocation
per node. The benchmark runs the algorithms in all three states. If Live is not clearly faster than
Dynamic at reading and clearly cheaper than Frozen to keep current, the smaller design is two states
with algorithms on Dynamic.

### Invariants

1. A position returned by `add_node` names the same node until that node is removed, across any
   sequence of mutations and state transitions, `freeze_clone` included.
2. No position is returned by `add_node` twice between two calls of `clear`.
3. Only `compact_ids` changes a live node's position, and it returns the mapping.
4. At most one edge exists per source, target and key.
5. In every state, the edges a reader sees are the edges a model applying the same operations in
   order would hold, and the backward edges are exactly the forward edges reversed.
6. No edge and no algorithm result names a tombstone.
7. Every algorithm's result on a projection equals its result on a graph built from the nodes and
   edges that pass the projection's predicates, at the same positions, holding one edge per
   connection, the lightest where weights differ, under `Merge`, and every passing edge under
   `Distinct`.
8. After every mutation, each maintained view equals the projection of the current graph under its
   predicates and its mode.
9. `freeze_clone` leaves the source graph, its state, its positions and its maintained views
   unchanged; the frozen copy holds the same positions and no maintained views.

## Crates

| Crate | Holds | Depends on |
|---|---|---|
| `ultragraph_traits` | `EdgeKey`, with its impls for `()` and the primitive numeric types | nothing |
| `ultragraph` | The in-memory graph: three states and their transitions, projections, algorithms; re-exports `EdgeKey` | `ultragraph_traits` |
| `deep_causality_context_store` | `RelationKind` and its `impl EdgeKey` (`Key = RelationKind`) | `ultragraph_traits`, added |
| `deep_causality_ethos` | `TeloidRelation` and its `impl EdgeKey` (`Key = ()`) | `ultragraph`, as today; the trait through its re-export |

Rust's orphan rule lets a crate implement a trait for a type only if it defines one of them.
`deep_causality_context_store` defines `RelationKind` and depends on the trait crate alone, so it
implements `EdgeKey` for its own type, `RelationKind` stays where it is, no import changes in the
places that use it (`deep_causality`, `deep_causality_context`, `deep_causality_context_store`,
`deep_causality_quantum`, and the drone tutorial under `end_to_end_tutorials/dynamic_drone_failsafe`),
and the store crate does not depend on the graph. It does gain its first dependency: a backend links
`deep_causality_context_store` and `ultragraph_traits`. In the tier block of AGENTS.md,
`ultragraph_traits` enters Tier 0, and `deep_causality_context_store` and `ultragraph` move to
Tier 1. The tier table in `deep_causality_unified_math/README.md` covers only the crates in that
folder, none of which is affected, and does not change.

`deep_causality_ethos` defines `TeloidRelation` and already depends on `ultragraph`, so it
implements the re-exported trait and gains no dependency. With `Key = ()` a teloid pair carries one
relation; whether a second relation replaces the first or is refused is decided in the migration
note, §3.2.

`ultragraph_traits` starts with `EdgeKey` alone. It is the bottom of the layering the storage work
continues: storage crates (the log, checkpoints, deterministic apply, and anything that brings file
formats or I/O) sit above `ultragraph`, and a trait moves into `ultragraph_traits` when a crate needs
it without the in-memory graph.

## Benchmark

**What it decides:** whether snapshot-log arrays replace Vec lists as `DynamicGraph`'s edge storage,
and whether the Live state earns its place (§Design 7).

**Setup:** both candidates carry §1–3, so only the edge structure differs, and both run the same
operations with the same semantics.

**Workloads:**
- building a graph by inserts;
- keyed update and removal, on low-degree nodes and on a hub of 10⁵ edges;
- outbound and inbound traversal in the dynamic state;
- `freeze` and `unfreeze`;
- memory per edge, with `MIN_LOG` swept;
- the algorithms in the Live and the Frozen states, and over the Dynamic state's rows with their
  logs merged: whole-graph sweeps (every row in order) and walks from one source (rows in random
  order), on deep, wide walks as well as short ones;
- updates in the Live state: in-place writes, row rebuilds, and the transitions of §Design 7;
- maintained views: the cost a mutation pays with 0, 1, 4 and 16 registered views.

Graph shapes: the shapes DeepCausality's tests build, and one power-law graph.

**Decision rules:**
- Snapshot-log arrays replace Vec lists if they come within about 5 % of them on building,
  traversal and `freeze`, and are clearly faster on update and removal. Otherwise Vec lists stay,
  with §1–3.
- Live stays if its algorithms run clearly faster than over the Dynamic state's rows and it stays
  current at clearly less cost than re-freezing. Otherwise the graph keeps two states, Dynamic and
  Frozen, with algorithms also over the Dynamic rows, and maintained views use the Dynamic layout.

**API impact, under either outcome:**
- Signatures: `W` gains the bound `EdgeKey`, from `ultragraph_traits`; `remove_edge_keyed`,
  `contains_edge_keyed`, `compact_ids`, `ParallelEdges`, `project`, `materialize`, `maintain`, `view`,
  `freeze_clone` and the transition into Live are added; `GraphState` gains `Live`.
- Behaviour: as in §Changes visible to callers. The edge structure itself is invisible to callers.

## Changes visible to callers

| Behaviour | Today | After |
|---|---|---|
| Positions after `remove_node` and `freeze` | Live nodes renumbered by rank | Unchanged |
| `CsmGraph::number_nodes` | `nodes.len()`, equal to live nodes after the renumbering | Live nodes |
| Reclaiming removed positions | Implicit in every `freeze` | Explicit, by `compact_ids`, which returns the mapping |
| Algorithm results on a graph with removals | Positions of the renumbered graph | Positions as issued, tombstones absent |
| A second `add_edge(a, b, w)` | Adds a parallel edge | Updates the edge with `w`'s key; adds one when the key differs |
| `remove_edge(a, b)` | Removes the first edge found to `b` | Removes every edge to `b`; `remove_edge_keyed` removes the edge with one key |
| `remove_node` | Clears the node's outgoing edges; incoming edges dangle until `freeze` | Removes its outgoing and incoming edges |
| `number_edges` in the dynamic state | Counts edges into removed nodes | Counts live edges |
| Traversal in the dynamic state | `GraphNotFrozen` | Outbound and inbound supported |
| Edge payload type | `W: Clone + Default` | `W: Clone + Default + EdgeKey` |
| Algorithm results | Computed on the edges as stored | Computed on the simple projection (`Merge`); the same on a graph without parallel edges; on a graph with them, bridges and directed betweenness change. A projection in `Distinct` gives today's results for them (§Design 5) |
| Graphs with `W = ()` or a numeric weight | Can hold parallel edges | Hold at most one edge per ordered pair (`Key = ()`) |
| Algorithms on a subset of nodes or edges | Not available; a caller builds a second graph | `project` with a node and an edge predicate, as a view, as a frozen copy through `materialize`, or as a view the graph keeps current through `maintain` (§Design 6) |
| States | Dynamic and Static (Frozen); `freeze` converts in place | Dynamic, Live and Frozen; `freeze` converts in place, `freeze_clone` returns a frozen copy (§Design 7) |
| Algorithms on a graph that keeps changing | Not available; every change needs `unfreeze` and a new `freeze` | The Live state, or a maintained view |

**DeepCausality.** The migration note (`RadixGraphUpdateMigration.md`) gives each type's changes,
the order they land in, and the tests that change. In brief:
- **Context graph.** Its freeze computes each node's rank and rewrites its maps by it
  (`contextuable_graph.rs:173-207`). With positions permanent, that rank no longer matches the frozen
  position, so the remap points the maps at the wrong nodes and must be removed in the same change.
  The data and time index maps then drop entries naming a removed node at removal rather than at
  freeze. Neighbour listing works in the dynamic state. Its edge payload `RelationKind` takes
  `Key = RelationKind`, implemented in `deep_causality_context_store` (§Crates). Relations of several
  kinds between two contextoids are not kept by switching `connect` and `sever` to the keyed
  variants: the storage contract allows one relation per pair, and `EdgeRetracted`, `snapshot` and
  `restore` identify an edge by its pair. Typed context edges need a change to that contract and a
  new `RECORD_VERSION`. The migration note, §1.9, lists that change; it is an open decision, and
  until it is made the context keeps one relation per pair through the store.
- **Causaloid graph.** Its `u64` payload is a weight: written by `add_edg_with_weight`
  (`causable_graph.rs:119-129`) and read nowhere in the causaloid graph. It takes `Key = ()`, so a
  second `add_edge(a, b, w)` updates the weight instead of adding a parallel edge; evaluation results
  stay the same. The reasoning engines, `check_single_writer` and `reachable_mask` size their arrays
  by `number_nodes()`, which after a removal and a freeze falls below the highest position. Without
  migration, evaluation panics on an index past the array, and `freeze_verified` refuses a live
  writer. They move to `index_bound()`, one past the highest live index. Indices held across a
  removal and a freeze (`LambdaEdges` keys, state writers, relay targets) stop shifting.
  `check_single_writer` on a dynamic graph checks instead of returning `Ok` unchecked.
- **Effect ethos.** `TeloidRelation` takes `Key = ()`. A second relation between one teloid pair
  replaces the first, where today it adds a parallel edge whose order after freezing is not
  specified. Whether it is refused instead is open (migration note, §3.2).
- **QCM readers in `deep_causality_quantum`.** `FactorSupports::from_graph`,
  `CausalStructure::from_graph_reachability`, `DemModel::from_graph` and the model-subject
  validation rely on frozen positions being dense in `0..number_nodes()`. After a removal and a
  freeze they refuse live nodes, skip their edges, and miss cycles through them. They move to
  `index_bound()` and `contains_causaloid`.
- **Tests.** Those that pin renumbering, traversal failing on a dynamic graph, parallel edges with a
  weight or no payload, multigraph bridges, `remove_edge` removing one of two kinds, and the
  quantum range messages change. The migration note, §5, lists each by file and line: thirteen
  tests across `ultragraph`, `deep_causality`, `deep_causality_context` and
  `deep_causality_quantum`.

## Verification

1. **Model-based property test.** Random sequences of `add_node`, `remove_node`, keyed `add_edge`,
   `remove_edge`, `remove_edge_keyed`, every transition of §Design 7, `freeze_clone`, `maintain`,
   `compact_ids` and `clear`, applied to the new graph and to a reference model: a map from
   position to node and a set of `(source, target, key)` edges with payloads. The sequences include
   removing every node and then freezing, and Live rows that outgrow their capacity. After every
   step: invariants 1–6, 8 and 9, identical outbound and inbound edges per node in every state, and
   `compact_ids` agreeing with the model's renumbering.
2. **Algorithm equivalence.** On graphs with tombstones, every algorithm's result equals its result on
   the `compact_ids` graph mapped back through the returned table. On graphs with parallel edges of
   several keys, and under random node and edge predicates, every algorithm's result on the view,
   on its materialised CSR and on a maintained view, in the Live and the Frozen states and in both
   `ParallelEdges` modes, equals its result on a graph built from the passing nodes and edges with
   one edge per connection carrying the lightest weight (`Merge`) or with every passing edge
   (`Distinct`) (invariant 7).
3. **The benchmark** above.
4. **The DeepCausality suites unchanged in outcome:** `deep_causality`, `deep_causality_context`,
   `deep_causality_context_store`, `deep_causality_ethos` and `deep_causality_quantum`, after the
   migration, apart from the tests the migration note lists (§5 there), and with the new regression
   tests it lists failing on the unmigrated code.

## Later

- Versioned reads (`outbound_edges_at(node, version)`, log entries stamped with a transaction's op
  number) and the retention of old versions for concurrent readers, in the storage layer.
- Persistence: the replicated log and checkpoints of frozen CSR segments, in the storage layer; a
  snapshot-log entry corresponds one-to-one with a record in that log.
- Weight aggregations of parallel edges beyond the lightest, after Neo4j's Graph Data Science
  (maximum, sum, count), as further `ParallelEdges` modes.
- A pattern-matching query engine (`MATCH` with shapes of several hops and variable-length paths),
  in the storage layer built on UltraGraph (§Design 6).

## Sources

- Haoxuan Xie, Junfeng Liu, Siqiang Luo, Kai Wang. *RadixGraph: A Fast, Space-Optimized Data
  Structure for Dynamic Graph Storage* (extended version). SIGMOD 2026.
  [arXiv 2601.01444](https://arxiv.org/abs/2601.01444).
- RadixGraph source: [github.com/ForwardStar/RadixGraph](https://github.com/ForwardStar/RadixGraph),
  commit `9feba53` (2026-05-27): `src/radixgraph.h`, `src/radixgraph.cpp`, `src/sort.h`,
  `src/sort.cpp`, `src/utils.h`, read on 2026-10-15.
- Pradeep Kumar, H. Howie Huang. *GraphOne: A Data Store for Real-time Analytics on Evolving
  Graphs*. FAST 2019; ACM Transactions on Storage 15(4), 2019
  ([10.1145/3364180](https://doi.org/10.1145/3364180)).
- Jixian Su, Chiyu Hao, Shixuan Sun, Hao Zhang. *Revisiting the Design of In-Memory Dynamic Graph
  Storage*. Proceedings of the ACM on Management of Data 3(1), 2025 (SIGMOD 2025)
  ([10.1145/3709720](https://doi.org/10.1145/3709720); arXiv 2502.10959). It finds that
  fine-grained concurrency-control methods "suffer from severe efficiency and space issues due to
  maintaining versions and performing checks for each neighbor" and "experience significant
  contention on high-degree vertices", which supports keeping versions and concurrency out of the
  per-node structure.
- NetworkX graph types: [networkx.org, Graph types](https://networkx.org/documentation/stable/reference/classes/index.html).
  igraph `simplify()`: [python.igraph.org, GraphBase](https://python.igraph.org/en/stable/api/igraph.GraphBase.html).
  Neo4j Graph Data Science native projection: [neo4j.com, Project a graph](https://neo4j.com/docs/graph-data-science/current/management-ops/graph-creation/graph-project/).
