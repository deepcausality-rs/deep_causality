rustc -V
rustc 1.99.0 (b940084d7 2026-09-28)

sysctl machdep.cpu.brand_string hw.ncpu hw.memsize
machdep.cpu.brand_string: Apple M3 Max
hw.ncpu: 16
hw.memsize: 137438953472 (128GB)

uname -a
27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:42 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6031 arm64

-----------------------------------------------------------------
-----------------------------------------------------------------

## After the update: permanent positions, keyed edges, dynamic traversal, the projection view

Three runs of `cargo bench -p ultragraph` at commit `eb660b8ea` (change
`ultragraph-stable-ids-keyed-edges`, groups 1 to 3); the logs are `BenchAfterRun-1.txt` to
`BenchAfterRun-3.txt`. The method is the baseline's (`../BenchBefore/BenchBefore.md`): each run
value is criterion's point estimate, the mean is over the three runs, the spread is (largest −
smallest) / mean, and a row whose spread exceeds 15 % gives the median instead. The six shuffled
walks the baseline listed as pending have their first three runs here. The `add_edge`, `remove_node`
and `remove_edge` rows are from three later runs of those three groups alone, after the review
fixes (logs `BenchAfterReviewRun-1.txt` to `BenchAfterReviewRun-3.txt`): the `add_edge` fixture
inserts an edge the hub does not carry, where the earlier fixture replaced one it did and its scan
stopped at the matched leaf, halfway on average; `remove_node` cleans each neighbour's list once.

### Create

Fixtures: `build_graph` and `add_node` on the chain; `add_edge` on the star (hub degree `n`), inserting an edge to a node outside the star. Per element: per node.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `build_graph` | 10 | Dynamic | 205.79 ns | 203.42 ns | 193.59 ns | 200.93 ns | 6.1 % | 18.27 ns |
| `build_graph` | 100 | Dynamic | 1.66 µs | 1.76 µs | 1.64 µs | 1.69 µs | 7.0 % | 16.72 ns |
| `build_graph` | 1,000 | Dynamic | 16.61 µs | 16.09 µs | 15.84 µs | 16.18 µs | 4.8 % | 16.16 ns |
| `add_node` | 10 | Dynamic | 5.12 ns | 5.10 ns | 5.04 ns | 5.09 ns | 1.5 % | |
| `add_node` | 100 | Dynamic | 6.03 ns | 6.02 ns | 5.94 ns | 5.99 ns | 1.4 % | |
| `add_node` | 1,000 | Dynamic | 8.57 ns | 8.75 ns | 8.67 ns | 8.66 ns | 2.0 % | |
| `add_edge` | 10 | Dynamic | 18.16 ns | 18.53 ns | 18.09 ns | 18.26 ns | 2.4 % | |
| `add_edge` | 100 | Dynamic | 51.23 ns | 53.13 ns | 51.91 ns | 52.09 ns | 3.6 % | |
| `add_edge` | 1,000 | Dynamic | 352.16 ns | 354.08 ns | 352.17 ns | 352.80 ns | 0.5 % | |

### Read

`get_node` on the chain; `contains_edge_*` and `get_edges` on the star (hub degree `n`); the walks follow the chain from end to end, in insertion order or shuffled. Per element: per edge listed, or per step walked.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `get_node` | 10 | Dynamic | 2.42 ns | 2.38 ns | 2.36 ns | 2.38 ns | 2.5 % | |
| `get_node` | 100 | Dynamic | 2.31 ns | 2.32 ns | 2.32 ns | 2.32 ns | 0.4 % | |
| `get_node` | 1,000 | Dynamic | 2.31 ns | 2.38 ns | 2.32 ns | 2.34 ns | 3.0 % | |
| `contains_edge_dynamic` | 10 | Dynamic | 2.82 ns | 2.85 ns | 2.88 ns | 2.85 ns | 2.0 % | |
| `contains_edge_dynamic` | 100 | Dynamic | 21.03 ns | 21.15 ns | 21.28 ns | 21.15 ns | 1.2 % | |
| `contains_edge_dynamic` | 1,000 | Dynamic | 145.11 ns | 146.50 ns | 147.66 ns | 146.42 ns | 1.7 % | |
| `contains_edge_frozen` | 10 | Frozen | 5.01 ns | 5.05 ns | 5.11 ns | 5.06 ns | 2.0 % | |
| `contains_edge_frozen` | 100 | Frozen | 4.86 ns | 4.91 ns | 4.94 ns | 4.90 ns | 1.8 % | |
| `contains_edge_frozen` | 1,000 | Frozen | 6.94 ns | 6.99 ns | 7.05 ns | 6.99 ns | 1.7 % | |
| `get_edges` | 10 | Dynamic | 11.55 ns | 11.58 ns | 11.96 ns | 11.70 ns | 3.5 % | 1.17 ns |
| `get_edges` | 100 | Dynamic | 38.64 ns | 38.97 ns | 39.03 ns | 38.88 ns | 1.0 % | 0.39 ns |
| `get_edges` | 1,000 | Dynamic | 270.15 ns | 285.91 ns | 278.68 ns | 278.25 ns | 5.7 % | 0.28 ns |
| `outbound_walk` | 10 | Frozen | 12.42 ns | 12.54 ns | 13.01 ns | 12.66 ns | 4.7 % | 1.27 ns |
| `outbound_walk` | 100 | Frozen | 216.79 ns | 217.78 ns | 208.20 ns | 214.26 ns | 4.5 % | 2.14 ns |
| `outbound_walk` | 1,000 | Frozen | 1.74 µs | 1.75 µs | 1.76 µs | 1.75 µs | 1.1 % | 1.75 ns |
| `inbound_walk` | 10 | Frozen | 12.44 ns | 12.52 ns | 12.70 ns | 12.55 ns | 2.1 % | 1.26 ns |
| `inbound_walk` | 100 | Frozen | 202.75 ns | 202.23 ns | 201.11 ns | 202.03 ns | 0.8 % | 2.02 ns |
| `inbound_walk` | 1,000 | Frozen | 1.00 µs | 1.01 µs | 1.07 µs | 1.03 µs | 6.4 % | 1.03 ns |
| `outbound_walk_shuffled` | 10 | Frozen | 12.25 ns | 12.22 ns | 12.23 ns | 12.23 ns | 0.2 % | 1.22 ns |
| `outbound_walk_shuffled` | 100 | Frozen | 209.47 ns | 214.16 ns | 214.16 ns | 212.60 ns | 2.2 % | 2.13 ns |
| `outbound_walk_shuffled` | 1,000 | Frozen | 2.13 µs | 2.15 µs | 2.15 µs | 2.14 µs | 0.9 % | 2.14 ns |
| `inbound_walk_shuffled` | 10 | Frozen | 12.59 ns | 12.22 ns | 12.61 ns | 12.47 ns | 3.1 % | 1.25 ns |
| `inbound_walk_shuffled` | 100 | Frozen | 202.70 ns | 200.29 ns | 199.67 ns | 200.89 ns | 1.5 % | 2.01 ns |
| `inbound_walk_shuffled` | 1,000 | Frozen | 2.18 µs | 2.15 µs | 2.15 µs | 2.16 µs | 1.6 % | 2.16 ns |

### Update

Chain.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread |
|---|---:|---|---:|---:|---:|---:|---:|
| `update_node` | 10 | Dynamic | 2.29 ns | 2.26 ns | 2.25 ns | 2.27 ns | 1.9 % |
| `update_node` | 100 | Dynamic | 2.25 ns | 2.25 ns | 2.25 ns | 2.25 ns | 0.2 % |
| `update_node` | 1,000 | Dynamic | 2.24 ns | 2.24 ns | 2.26 ns | 2.25 ns | 1.1 % |

### Delete

`remove_node` on the chain; `remove_edge` on the star (hub degree `n`).

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread |
|---|---:|---|---:|---:|---:|---:|---:|
| `remove_node` | 10 | Dynamic | 24.18 ns | 23.12 ns | 23.43 ns | 23.58 ns | 4.5 % |
| `remove_node` | 100 | Dynamic | 24.84 ns | 25.21 ns | 24.92 ns | 24.99 ns | 1.5 % |
| `remove_node` | 1,000 | Dynamic | 29.01 ns | 31.65 ns | 29.91 ns | 30.19 ns | 8.8 % |
| `remove_edge` | 10 | Dynamic | 17.78 ns | 18.56 ns | 18.19 ns | 18.18 ns | 4.3 % |
| `remove_edge` | 100 | Dynamic | 47.70 ns | 46.55 ns | 47.62 ns | 47.29 ns | 2.4 % |
| `remove_edge` | 1,000 | Dynamic | 331.72 ns | 331.23 ns | 322.07 ns | 328.34 ns | 2.9 % |

### State transitions

Chain. Per element: per node.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `freeze` | 10 | Dynamic → Frozen | 495.93 ns | 495.78 ns | 498.20 ns | 496.64 ns | 0.5 % | 45.15 ns |
| `freeze` | 100 | Dynamic → Frozen | 4.02 µs | 4.03 µs | 4.10 µs | 4.05 µs | 2.0 % | 40.09 ns |
| `freeze` | 1,000 | Dynamic → Frozen | 39.22 µs | 39.36 µs | 40.69 µs | 39.76 µs | 3.7 % | 39.72 ns |
| `unfreeze` | 10 | Frozen → Dynamic | 205.91 ns | 199.46 ns | 206.58 ns | 203.98 ns | 3.5 % | 18.54 ns |
| `unfreeze` | 100 | Frozen → Dynamic | 1.49 µs | 1.49 µs | 1.49 µs | 1.49 µs | 0.2 % | 14.75 ns |
| `unfreeze` | 1,000 | Frozen → Dynamic | 13.89 µs | 13.84 µs | 14.49 µs | 14.07 µs | 4.6 % | 14.06 ns |

### Algorithms

Triangle chain of `n` triangles: `2n + 1` nodes, `3n` edges. Per element: per edge.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `articulation_points` | 10 | Frozen | 1.24 µs | 1.33 µs | 1.40 µs | 1.32 µs | 12.2 % | 44.16 ns |
| `articulation_points` | 100 | Frozen | 10.48 µs | 10.60 µs | 10.57 µs | 10.55 µs | 1.1 % | 35.17 ns |
| `articulation_points` | 1,000 | Frozen | 130.80 µs | 130.78 µs | 141.76 µs | 134.45 µs | 8.2 % | 44.82 ns |
| `articulation_points` | 5,000 | Frozen | 878.40 µs | 856.34 µs | 971.89 µs | 902.21 µs | 12.8 % | 60.15 ns |
| `bridges` | 10 | Frozen | 1.08 µs | 1.21 µs | 1.26 µs | n/a (median 1.21 µs) | 15.1 % | 40.32 ns |
| `bridges` | 100 | Frozen | 10.42 µs | 10.41 µs | 10.49 µs | 10.44 µs | 0.7 % | 34.80 ns |
| `bridges` | 1,000 | Frozen | 127.62 µs | 126.06 µs | 130.24 µs | 127.97 µs | 3.3 % | 42.66 ns |
| `bridges` | 5,000 | Frozen | 856.20 µs | 851.22 µs | 882.39 µs | 863.27 µs | 3.6 % | 57.55 ns |
| `biconnected_components` | 10 | Frozen | 1.93 µs | 1.95 µs | 2.01 µs | 1.96 µs | 4.6 % | 65.47 ns |
| `biconnected_components` | 100 | Frozen | 17.24 µs | 17.03 µs | 17.53 µs | 17.27 µs | 2.9 % | 57.55 ns |
| `biconnected_components` | 1,000 | Frozen | 191.77 µs | 186.61 µs | 186.50 µs | 188.29 µs | 2.8 % | 62.76 ns |
| `biconnected_components` | 5,000 | Frozen | 1.198 ms | 1.154 ms | 1.180 ms | 1.177 ms | 3.8 % | 78.49 ns |


## Comparison with the baseline

A row has moved when its mean differs from the baseline's by more than the larger of 5 % and the
two rows' spreads; the baseline's two unstable `inbound_walk` rows are compared by their medians.
Of the 54 rows both notes share, 24 are within spread, 25 are slower and 5 are faster.

| Benchmark | n | Before | Spread | After | Spread | Change | |
|---|---:|---:|---:|---:|---:|---:|---|
| `build_graph` | 10 | 117.73 ns | 3.2 % | 200.93 ns | 6.1 % | +70.7 % | slower |
| `build_graph` | 100 | 931.35 ns | 3.0 % | 1.69 µs | 7.0 % | +81.3 % | slower |
| `build_graph` | 1,000 | 8.87 µs | 3.3 % | 16.18 µs | 4.8 % | +82.3 % | slower |
| `add_node` | 10 | 3.41 ns | 0.6 % | 5.09 ns | 1.5 % | +49.3 % | slower |
| `add_node` | 100 | 3.55 ns | 5.4 % | 5.99 ns | 1.4 % | +68.7 % | slower |
| `add_node` | 1,000 | 4.65 ns | 6.6 % | 8.66 ns | 2.0 % | +86.2 % | slower |
| `get_node` | 10 | 2.29 ns | 3.9 % | 2.38 ns | 2.5 % | +4.2 % | within spread |
| `get_node` | 100 | 2.24 ns | 0.5 % | 2.32 ns | 0.4 % | +3.5 % | within spread |
| `get_node` | 1,000 | 2.29 ns | 5.2 % | 2.34 ns | 3.0 % | +2.0 % | within spread |
| `articulation_points` | 10 | 1.34 µs | 6.6 % | 1.32 µs | 12.2 % | -1.1 % | within spread |
| `articulation_points` | 100 | 10.77 µs | 4.8 % | 10.55 µs | 1.1 % | -2.0 % | within spread |
| `articulation_points` | 1,000 | 126.29 µs | 10.5 % | 134.45 µs | 8.2 % | +6.5 % | within spread |
| `articulation_points` | 5,000 | 898.62 µs | 4.4 % | 902.21 µs | 12.8 % | +0.4 % | within spread |
| `bridges` | 10 | 1.11 µs | 5.5 % | 1.21 µs | 15.1 % | +8.8 % | within spread |
| `bridges` | 100 | 10.30 µs | 2.5 % | 10.44 µs | 0.7 % | +1.3 % | within spread |
| `bridges` | 1,000 | 121.60 µs | 3.0 % | 127.97 µs | 3.3 % | +5.2 % | slower |
| `bridges` | 5,000 | 858.07 µs | 1.4 % | 863.27 µs | 3.6 % | +0.6 % | within spread |
| `biconnected_components` | 10 | 1.93 µs | 2.6 % | 1.96 µs | 4.6 % | +1.7 % | within spread |
| `biconnected_components` | 100 | 17.10 µs | 4.1 % | 17.27 µs | 2.9 % | +1.0 % | within spread |
| `biconnected_components` | 1,000 | 189.48 µs | 3.4 % | 188.29 µs | 2.8 % | -0.6 % | within spread |
| `biconnected_components` | 5,000 | 1.193 ms | 5.7 % | 1.177 ms | 3.8 % | -1.3 % | within spread |
| `add_edge` | 10 | 3.56 ns | 4.9 % | 18.26 ns | 2.4 % | +412.8 % | slower |
| `add_edge` | 100 | 4.03 ns | 6.6 % | 52.09 ns | 3.6 % | +1192.6 % | slower |
| `add_edge` | 1,000 | 4.38 ns | 6.1 % | 352.80 ns | 0.5 % | +7954.9 % | slower |
| `contains_edge_dynamic` | 10 | 3.38 ns | 0.7 % | 2.85 ns | 2.0 % | -15.7 % | faster |
| `contains_edge_dynamic` | 100 | 22.51 ns | 0.6 % | 21.15 ns | 1.2 % | -6.0 % | faster |
| `contains_edge_dynamic` | 1,000 | 150.92 ns | 1.8 % | 146.42 ns | 1.7 % | -3.0 % | within spread |
| `contains_edge_frozen` | 10 | 4.82 ns | 1.6 % | 5.06 ns | 2.0 % | +5.0 % | within spread |
| `contains_edge_frozen` | 100 | 4.81 ns | 1.2 % | 4.90 ns | 1.8 % | +1.8 % | within spread |
| `contains_edge_frozen` | 1,000 | 6.99 ns | 0.7 % | 6.99 ns | 1.7 % | +0.0 % | within spread |
| `get_edges` | 10 | 66.97 ns | 1.2 % | 11.70 ns | 3.5 % | -82.5 % | faster |
| `get_edges` | 100 | 260.60 ns | 10.0 % | 38.88 ns | 1.0 % | -85.1 % | faster |
| `get_edges` | 1,000 | 1.19 µs | 1.8 % | 278.25 ns | 5.7 % | -76.6 % | faster |
| `outbound_walk` | 10 | 11.42 ns | 1.2 % | 12.66 ns | 4.7 % | +10.8 % | slower |
| `outbound_walk` | 100 | 201.47 ns | 6.8 % | 214.26 ns | 4.5 % | +6.3 % | within spread |
| `outbound_walk` | 1,000 | 1.57 µs | 3.6 % | 1.75 µs | 1.1 % | +11.7 % | slower |
| `inbound_walk` | 10 | 11.49 ns | 1.7 % | 12.55 ns | 2.1 % | +9.2 % | slower |
| `inbound_walk` | 100 | 216.74 ns | 77.7 % | 202.03 ns | 0.8 % | -6.8 % | within spread |
| `inbound_walk` | 1,000 | 868.09 ns | 85.6 % | 1.03 µs | 6.4 % | +18.4 % | within spread |
| `update_node` | 10 | 2.29 ns | 3.0 % | 2.27 ns | 1.9 % | -0.8 % | within spread |
| `update_node` | 100 | 2.25 ns | 0.9 % | 2.25 ns | 0.2 % | -0.2 % | within spread |
| `update_node` | 1,000 | 2.23 ns | 0.7 % | 2.25 ns | 1.1 % | +0.5 % | within spread |
| `remove_node` | 10 | 2.82 ns | 0.2 % | 23.58 ns | 4.5 % | +736.0 % | slower |
| `remove_node` | 100 | 2.43 ns | 8.6 % | 24.99 ns | 1.5 % | +928.4 % | slower |
| `remove_node` | 1,000 | 4.15 ns | 3.0 % | 30.19 ns | 8.8 % | +627.5 % | slower |
| `remove_edge` | 10 | 13.27 ns | 1.4 % | 18.18 ns | 4.3 % | +37.0 % | slower |
| `remove_edge` | 100 | 32.19 ns | 0.4 % | 47.29 ns | 2.4 % | +46.9 % | slower |
| `remove_edge` | 1,000 | 171.29 ns | 2.1 % | 328.34 ns | 2.9 % | +91.7 % | slower |
| `freeze` | 10 | 465.45 ns | 1.3 % | 496.64 ns | 0.5 % | +6.7 % | slower |
| `freeze` | 100 | 3.64 µs | 1.7 % | 4.05 µs | 2.0 % | +11.1 % | slower |
| `freeze` | 1,000 | 35.74 µs | 5.8 % | 39.76 µs | 3.7 % | +11.3 % | slower |
| `unfreeze` | 10 | 106.81 ns | 5.2 % | 203.98 ns | 3.5 % | +91.0 % | slower |
| `unfreeze` | 100 | 662.66 ns | 2.1 % | 1.49 µs | 0.2 % | +124.9 % | slower |
| `unfreeze` | 1,000 | 5.91 µs | 2.5 % | 14.07 µs | 4.6 % | +138.0 % | slower |

## Every move, explained by a design decision

**Slower, as design D9 expected.** `add_edge` on the hub is the keyed insert of D4: it scans the
hub's forward list for `(target, key)` before it appends, O(d), so at degree 1,000 an insert
scans the whole list, 353 ns, about twice the 146 ns of `contains_edge_dynamic`, whose scan stops
at the match halfway on average, where the unchecked push cost 4 ns. `build_graph` and `add_node` pay the second list per node: `add_node` pushes a forward
and a backward list where it pushed one, and the chain's `build_graph` adds the keyed scan of a
list of length one per edge.

**Slower, from the backward list D4 adds and D9 did not list.** `unfreeze` rebuilds the backward
lists from the forward lists, one allocation per node, about 8 ns per node; `freeze` consumes the
dynamic graph and so drops them, about 4 ns per node, which is its whole move. `remove_node` on
the chain went from clearing one list to taking the edges into the node out of their sources'
lists: a constant 24 to 30 ns in place of 3 ns. `remove_edge` on the hub removes every edge of the pair,
an order-preserving `retain` over the hub's list plus the write to the target's backward list,
where it was a `position` and a `swap_remove`; the scan is the 92 % at degree 1,000.

**The walks, 9 to 12 % slower at 10 and 1,000 nodes.** Traversal answers in both states, so the
container's `outbound_edges` and `inbound_edges` return one iterator type over two: an enum whose
`next` matches on the state, a branch per step of about 0.2 ns on a 1.6 ns step. The 100-node rows
moved within their spread.

**Faster.** `get_edges` on the hub, −77 to −85 %, and `contains_edge_dynamic`, −6 to −16 % at 10
and 100: no edge touches a tombstone any more (`remove_node` takes the edges into the node with
it), so the dynamic reads no longer test `contains_node` on every target.

**At the floor.** `bridges` at 1,000 triangles, +5.2 % against a floor of 5 %: the three
biconnectivity algorithms now read the adjacency through the projection trait, and the other two
on the same symmetric adjacency moved within their spread (`articulation_points` +6.5 % against
10.5 %, `biconnected_components` −0.6 %). The row is at the noise edge, not evidence of a cost.

**Within spread**, as D9 expected: every `get_node`, `update_node`, `contains_edge_frozen` and
`contains_edge_dynamic` at 1,000, and every algorithm row but the one above.

## Observations

**The two-speed `inbound_walk` is gone.** The baseline's 100- and 1,000-node rows had spreads of
78 % and 86 %; here they have 0.8 % and 6.4 %. At 100 nodes the walk sits at the baseline's slower
level (202 ns beside `outbound_walk`'s 214 ns), at 1,000 at its faster one (1.03 µs beside
1.75 µs), each steady across the three runs. The enum iterator changed the loop the core predicts
through; whichever level it settles on, it no longer alternates.

**Shuffled chains walk slower per step than ordered ones at 1,000 nodes**: 2.14 µs against
1.75 µs outbound, and 2.16 µs against 1.03 µs inbound. The two shuffled walks land on one figure
while the ordered inbound walk keeps its fast level, which is what a walk with no next address to
predict looks like; at 10 and 100 nodes shuffled and ordered agree within spread.

**A swap-remove in `remove_edge` was tried and is slower.** Three runs of the `remove` rows with
every removal done by swapping the last entry into the hole instead of `retain`: `remove_edge`
51.7 ns against 43.3 ns at degree 100 (+19 %) and 349 ns against 312 ns at 1,000 (+12 %),
`remove_node` within spread. The cost of `remove_edge` is not the compaction: removing every key
of a pair has to scan the whole list, where the old `position` stopped at the first match, halfway
on average, and `retain` is already a tight loop over that scan. The `retain` version stays. The
scan itself is the lever, and only an index on high-degree nodes removes it.

**What the second change measures against.** The hub's keyed `add_edge` at 353 ns and `remove_edge`
at 328 ns are the rows the snapshot-log candidate of `ultragraph-live-state-maintained-views` has
to win; `unfreeze` at 14 ns per node and the 5 ns per node of `add_node` are the price of the
backward list, which both candidates pay.
