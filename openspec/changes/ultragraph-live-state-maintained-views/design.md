## Context

`ultragraph-stable-ids-keyed-edges` leaves UltraGraph with permanent positions, keyed edges held in a
forward and a backward `Vec` per node, traversal in the dynamic state, and projections as views that
borrow a frozen graph. Every algorithm still needs `freeze`, a projection blocks writes while it
lives, and a keyed insert or removal on a hub scans the hub's list.

The baseline (`openspec/notes/ultragraph/BenchBefore/BenchBefore.md`, Apple M3 Max, rustc 1.99.0,
three runs) puts `freeze` at about 36 ns per node at 1,000 nodes against about 9 ns per node to build,
`remove_edge` on a hub of 1,000 at 171 ns, and `contains_edge_dynamic` at 151 ns. Change 1's
after-run, `BenchAfter/BenchAfter.md`, is the incumbent this change measures against.

Decided by Marvin on 2026-10-15: three states, Dynamic (default), Live and Frozen, are one
`GraphState`; `freeze` stays in place and `freeze_clone` copies; a frozen copy leaves maintained
views behind; projections are a view, a frozen materialised copy, and a view maintained eagerly on
every mutation, with no refresh call; one projection type per graph through a `Projection` trait; the
snapshot-log arrays replace the `Vec` lists only if the benchmark says so; the query engine stays in
the storage layer; the work follows the TDD protocol of
`openspec/specs/unified-math-tdd-protocol/spec.md`.

## Goals / Non-Goals

**Goals:**
- Analysis on current data without a full re-freeze (Live).
- Scoped views that follow every write (maintained views) and owned frozen scopes (`materialize`).
- A measured decision on the Dynamic state's edge storage, and on whether Live earns its place.
- The same suite passing on every candidate before anything is measured.

**Non-Goals:**
- Pattern matching (`MATCH` of several hops), versioned reads, concurrent readers, persistence: the
  storage layer.
- Weight aggregations beyond the lightest edge (maximum, sum, count): later `ParallelEdges` modes.
- Any change to a DeepCausality caller.

## Decisions

### D1. Live is a row arena
A Live graph keeps one arena of `(target, W)` per direction, a per-node record `(start, len, cap)`
and a mask of live nodes. Rows are sorted by target, then key, so `Merge` skips repeated targets as
in the Frozen state. A change that fits the row's spare capacity is written in place, keeping the
order; a row that outgrows its capacity is rebuilt alone at the arena's end with twice the capacity,
leaving a hole. A change is a binary search and a shift within the row, O(d); a rebuild costs O(d)
once per doubling of the row, amortised O(1) per change by the argument of RadixGraph's Theorem 2. An edge change touches the source's forward row and the target's backward row. A
removed node clears its mask bit; readers skip targets whose bit is clear, and stale entries leave at
the next rebuild of their rows. Holes close at `freeze`.
*Alternative:* a packed-memory array (Wheatman and Xu's PCSR), which rebalances windows instead of
relocating rows. Rejected for now: more code, and no per-row isolation for the maintained views.

### D2. Transitions reuse the structures they start from
Frozen → Live adopts the CSR arrays as the arena and gives each row `cap = len`, O(V); the first
write to a row moves it. Dynamic → Live packs and sorts every row, the work of today's `freeze`.
Live → Frozen closes holes and drops spare capacity, O(V + E), with no sort. Live → Dynamic splits rows
into per-node lists. `freeze_clone` clones the graph without its maintained views and freezes the
clone, so its cost is the source state's `freeze` plus the copy.

### D3. Algorithms read one adjacency trait
Change 1's crate-internal adjacency trait gains a Live implementation that reads rows through the
records and the mask. No algorithm body changes; `Merge` and `Distinct` behave as in the Frozen
state.

### D4. Maintained views live inside the graph
The container's third type parameter `P: Projection<N, W>` defaults to a type with no views.
`maintain(p, mode)` builds the view's rows in the Live layout (D1), stores `(p, mode, rows)` in the
graph, and returns a `ViewId`. Every mutation runs once over the graph and once per view: an edge
mutation tests the edge and its two ends against the view and updates at most one connection; a node
payload change tests the node and flips its mask bit, its edges following through the mask. Owning
the views keeps the single-owner rule: a mutation takes `&mut self` once; `view(id)` borrows the graph
and the view together. Static dispatch through `P` follows the code conventions; boxed closures would
need `dyn`. `compact_ids` stays a Dynamic operation: the Live state refuses it with
`GraphError::GraphIsLive`, as the Frozen state refuses it with `GraphIsFrozen`, and in the Dynamic
state it rebuilds every maintained view from its projection at the new positions, under its
identifier.

### D5. The snapshot-log candidate
Per node and direction, `EdgeArray { snapshot: Vec<(usize, W)>, log: Vec<LogEntry<W>> }` with
`LogEntry { target, op: Put(W) | Remove(W::Key) | RemoveAll }`. Inserts, updates and removals append
after a lookup of `(target, key)`, which the exact `number_edges` and the `EdgeNotFoundError` of
change 1 require: a binary search of the snapshot, sorted by target then key, and a newest-first scan
of the log. The append is amortised O(1) by RadixGraph's Theorem 2, which covers blind writes; the
lookup is O(log d + L) for a log of length L, and L reaches the snapshot's length before a
compaction. When `log.len()` reaches `max(snapshot.len(), MIN_LOG)` the node compacts its own array by a
newest-first scan that decides each `(target, key)` once, through a bitset the graph owns and reuses.
Unlike RadixGraph's code, a duplicate insert is resolved as an update in every compaction, including
an all-insert log, so invariant 4 of change 1 holds; RadixGraph's insert-only shortcut keeps
duplicates (`src/radixgraph.cpp:206-212` at commit `9feba53`). A removal is an enum variant, not a
zero weight, so a weight of 0 stays a weight. Both candidates sit behind one crate-internal storage
trait until the decision; a temporary Cargo feature, `edge-log`, off by default, selects the
snapshot-log arrays and leaves with the losing candidate. The repository's tests reach only the public
API, so the candidate's tests drive a node through the graph across each compaction trigger.

### D6. Measurement and decision rules
Three runs per candidate on the baseline machine, aggregated like the baseline. The rules in
`ultragraph-edge-storage-benchmark` refine the note's "within about 5 %": the baseline shows a
run-to-run spread above 5 % for 16 of its 54 rows. A candidate *matches* the incumbent on a row when
its mean is at most the incumbent's mean times one plus the larger of 5 % and the incumbent row's
spread, so a faster candidate matches. It is *clearly faster* when each of its three runs is faster
than each of the incumbent's and its mean lies below the incumbent's by more than the incumbent row's
spread. Memory per edge is deterministic: it is computed once per configuration from the capacity of
every buffer the storage holds, through a `#[doc(hidden)]` function in `src/utils/utils_tests.rs`. A
counting allocator would need `unsafe impl GlobalAlloc`, which `unsafe_code = "forbid"` refuses in
the bench target as everywhere else. `MIN_LOG` is the swept value with the fastest hub update among
those whose memory per edge stays at most 1.25 times the `Vec` lists'; if none does, the `Vec` lists
stay.

Live is measured against the workflow it replaces, k writes followed by one analysis: for k = 1, 16
and 256, Live must be clearly faster than Dynamic → `freeze` → `strongly_connected_components` →
`unfreeze`. A linear algorithm makes the avoided freeze a large share of the work. No algorithm runs
over the Dynamic rows. The Live decision is taken before maintained views are built on the Live
layout; if Live fails its rule, work stops for the user's decision.

### D7. The TDD protocol around a benchmark gate
Each implementation group runs phases 1 to 5. The candidates are compared only after both pass the
same audited suite (phase 4 complete for both); the measurement is not a phase but a gate between
groups.

**Independent sources.** The change-1 reference model, extended with views: a view's expected content
is built by filtering the model afresh after every step, an algorithm that shares no code with the
incremental maintenance; Live results are compared with a Dynamic graph frozen after every step;
`materialize` results with the view they came from and with a graph built from the passing elements by
a separate filter. Random sequences come from `Xoshiro256::from_seed`.

**Corner cases A–K.**

| # | Class | Instance in this change |
|---|---|---|
| A | Empty | Live and Frozen empty graphs; `maintain` on an empty graph; `materialize` of a view passing nothing |
| B | Single element | One node in Live; a view passing one node; a row of capacity one |
| C | Coinciding quantities | A view passing everything, where the view equals the graph; `MIN_LOG` equal to the snapshot length; every such test also runs where they differ |
| D | Degenerate index | A row rebuilt at the arena's end; the last node's row; a node removed and its mask bit cleared while its entries remain in neighbours' rows |
| E | Thresholds, both sides | A row at capacity, one below and one above; the log at `max(snapshot.len(), MIN_LOG)`, one below and one above |
| F | Zero | A weight of 0, which the snapshot-log candidate must not read as a removal; zero maintained views |
| G | Negative | n/a: positions and capacities are `usize` |
| H | Exact boundary | A row filling its capacity exactly; a view whose test flips on the last node |
| I | Non-finite | n/a: payloads with `f32`/`f64` weights have `Key = ()` and are not compared by the storage |
| J | Overflow reach | n/a: capacities double from a row's length, bounded by memory |
| K | Every payload kind | `()`, a weight, a kind key, an identity key, through every state and both candidates |

**Named defects for phase 3:** a row rebuilt without its spare capacity; a relocated row's record not
updated; the mask bit not cleared on removal; stale entries read through a cleared mask; Frozen → Live
copying instead of adopting (caught by the transition tests on content, measured by the benchmark);
`freeze_clone` copying the views or changing the source; a view not updated on an edge mutation, on a
node payload change, or on `clear`; a view updated on a mutation outside its scope; `compact_ids` leaving a view on the old positions;
`compact_ids` accepted in the Live state; `materialize`
sharing storage with the source; a materialised `Distinct` copy answering under `Merge`; a
transition dropping a maintained view; the snapshot-log compaction keeping a duplicate insert; a zero weight
read as a removal; compaction triggered one entry early or late.

### D8. A materialised copy carries its mode
The container keeps a `ParallelEdges` mode that its direct algorithms read: `Merge` for every graph
built by `new` or `with_capacity`, the view's mode for a copy `materialize` returns. Every transition
and `freeze_clone` keep it. The copy holds every passing edge, so a `Distinct` copy answers as its
view. Change 1's requirement that direct calls run under `Merge` becomes "under the graph's mode"
(the delta on `ultragraph-projection-view`). There is no setter.

## Risks / Trade-offs

- [Live may not beat freezing before each analysis] → D6's rule decides, and the change stops for
  the user before removing Live.
- [Write cost grows with every maintained view] → measured at 0, 1, 4 and 16 views; documented on
  `maintain`.
- [Holes in the Live arena grow until `freeze`] → measured as memory per edge; a later change can add
  compaction in Live if the measurement asks for it.
- [A third type parameter on the container is a breaking change for code naming the struct directly]
  → the aliases keep the default; the commit message carries the migration.
- [The snapshot-log candidate is new code that may lose] → it is written to the same audited suite,
  measured, and removed with the user's approval if it loses.

## Migration Plan

1. Benchmark workloads for every decision, against the incumbent.
2. The snapshot-log candidate behind the storage trait, through phases 1–5.
3. Measurement and the edge-storage decision; the loser removed with approval.
4. The Live state, its transitions and `freeze_clone`, through phases 1–5.
5. Measurement and the Live decision; stop for the user if Live fails.
6. `materialize` and maintained views, through phases 1–5.
7. Final measurement, documentation, close-out.

Each group ends with `bazel test //...` green, clippy clean, and a commit message for the user.

## Open Questions

- The transition into Live is named `live` provisionally; the phase-1 API review settles it.
- Unregistering a maintained view is not specified; add it if a caller needs it.
