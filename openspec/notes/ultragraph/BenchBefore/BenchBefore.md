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

## Baseline before the update

Three runs of `cargo bench -p ultragraph` on unchanged code; the logs are `BenchBeforeRun-1.txt`,
`BenchBeforeRun-2.txt` and `BenchBeforeRun-3.txt`. Each run value is criterion's point estimate,
the middle figure of `time: [low estimate high]`. The mean is over the three runs, and the spread
is (largest − smallest) / mean. Where the spread exceeds 15 %, the runs do not cluster around one
value, so the row gives the median instead of the mean, and the per-element figure uses it.

38 of the 54 benchmarks have a spread of 5 % or less, and 52 of 54 at most 10.5 %. A row's spread
is the noise floor for that row on this machine: a before/after difference smaller than it is not
evidence of a change.

### Create

Fixtures: `build_graph` and `add_node` on the chain; `add_edge` on the star (hub degree `n`). Per element: per node.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `build_graph` | 10 | Dynamic | 115.95 ns | 117.46 ns | 119.77 ns | 117.73 ns | 3.2 % | 10.70 ns |
| `build_graph` | 100 | Dynamic | 924.10 ns | 949.09 ns | 920.85 ns | 931.35 ns | 3.0 % | 9.22 ns |
| `build_graph` | 1,000 | Dynamic | 8.82 µs | 9.05 µs | 8.76 µs | 8.87 µs | 3.3 % | 8.87 ns |
| `add_node` | 10 | Dynamic | 3.42 ns | 3.40 ns | 3.40 ns | 3.41 ns | 0.6 % | |
| `add_node` | 100 | Dynamic | 3.46 ns | 3.65 ns | 3.55 ns | 3.55 ns | 5.4 % | |
| `add_node` | 1,000 | Dynamic | 4.55 ns | 4.86 ns | 4.55 ns | 4.65 ns | 6.6 % | |
| `add_edge` | 10 | Dynamic | 3.49 ns | 3.52 ns | 3.67 ns | 3.56 ns | 4.9 % | |
| `add_edge` | 100 | Dynamic | 3.91 ns | 4.18 ns | 4.01 ns | 4.03 ns | 6.6 % | |
| `add_edge` | 1,000 | Dynamic | 4.54 ns | 4.28 ns | 4.30 ns | 4.38 ns | 6.1 % | |

### Read

`get_node` on the chain; `contains_edge_*` and `get_edges` on the star (hub degree `n`); the walks follow the chain from end to end. Per element: per edge listed, or per step walked.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `get_node` | 10 | Dynamic | 2.27 ns | 2.34 ns | 2.25 ns | 2.29 ns | 3.9 % | |
| `get_node` | 100 | Dynamic | 2.24 ns | 2.25 ns | 2.23 ns | 2.24 ns | 0.5 % | |
| `get_node` | 1,000 | Dynamic | 2.24 ns | 2.28 ns | 2.36 ns | 2.29 ns | 5.2 % | |
| `contains_edge_dynamic` | 10 | Dynamic | 3.39 ns | 3.39 ns | 3.37 ns | 3.38 ns | 0.7 % | |
| `contains_edge_dynamic` | 100 | Dynamic | 22.43 ns | 22.57 ns | 22.53 ns | 22.51 ns | 0.6 % | |
| `contains_edge_dynamic` | 1,000 | Dynamic | 150.47 ns | 152.51 ns | 149.79 ns | 150.92 ns | 1.8 % | |
| `contains_edge_frozen` | 10 | Frozen | 4.85 ns | 4.83 ns | 4.77 ns | 4.82 ns | 1.6 % | |
| `contains_edge_frozen` | 100 | Frozen | 4.84 ns | 4.81 ns | 4.79 ns | 4.81 ns | 1.2 % | |
| `contains_edge_frozen` | 1,000 | Frozen | 6.99 ns | 7.01 ns | 6.96 ns | 6.99 ns | 0.7 % | |
| `get_edges` | 10 | Dynamic | 67.19 ns | 67.27 ns | 66.44 ns | 66.97 ns | 1.2 % | 6.70 ns |
| `get_edges` | 100 | Dynamic | 247.33 ns | 273.36 ns | 261.11 ns | 260.60 ns | 10.0 % | 2.61 ns |
| `get_edges` | 1,000 | Dynamic | 1.18 µs | 1.20 µs | 1.18 µs | 1.19 µs | 1.8 % | 1.19 ns |
| `outbound_walk` | 10 | Frozen | 11.42 ns | 11.49 ns | 11.35 ns | 11.42 ns | 1.2 % | 1.14 ns |
| `outbound_walk` | 100 | Frozen | 197.73 ns | 210.19 ns | 196.49 ns | 201.47 ns | 6.8 % | 2.01 ns |
| `outbound_walk` | 1,000 | Frozen | 1.56 µs | 1.60 µs | 1.54 µs | 1.57 µs | 3.6 % | 1.57 ns |
| `inbound_walk` | 10 | Frozen | 11.43 ns | 11.62 ns | 11.43 ns | 11.49 ns | 1.7 % | 1.15 ns |
| `inbound_walk` | 100 | Frozen | 217.98 ns | 83.73 ns | 216.74 ns | n/a (median 216.74 ns) | 77.7 % | 2.17 ns |
| `inbound_walk` | 1,000 | Frozen | 719.87 ns | 1.64 µs | 868.09 ns | n/a (median 868.09 ns) | 85.6 % | 0.87 ns |

### Update

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread |
|---|---:|---|---:|---:|---:|---:|---:|
| `update_node` | 10 | Dynamic | 2.26 ns | 2.33 ns | 2.27 ns | 2.29 ns | 3.0 % |
| `update_node` | 100 | Dynamic | 2.25 ns | 2.27 ns | 2.25 ns | 2.25 ns | 0.9 % |
| `update_node` | 1,000 | Dynamic | 2.24 ns | 2.23 ns | 2.23 ns | 2.23 ns | 0.7 % |

### Delete

`remove_node` on the chain; `remove_edge` on the star (hub degree `n`).

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread |
|---|---:|---|---:|---:|---:|---:|---:|
| `remove_node` | 10 | Dynamic | 2.82 ns | 2.82 ns | 2.82 ns | 2.82 ns | 0.2 % |
| `remove_node` | 100 | Dynamic | 2.36 ns | 2.57 ns | 2.36 ns | 2.43 ns | 8.6 % |
| `remove_node` | 1,000 | Dynamic | 4.08 ns | 4.17 ns | 4.20 ns | 4.15 ns | 3.0 % |
| `remove_edge` | 10 | Dynamic | 13.39 ns | 13.21 ns | 13.21 ns | 13.27 ns | 1.4 % |
| `remove_edge` | 100 | Dynamic | 32.25 ns | 32.20 ns | 32.13 ns | 32.19 ns | 0.4 % |
| `remove_edge` | 1,000 | Dynamic | 171.01 ns | 173.23 ns | 169.63 ns | 171.29 ns | 2.1 % |

### State transitions

Chain. Per element: per node.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `freeze` | 10 | Dynamic → Frozen | 462.24 ns | 468.19 ns | 465.92 ns | 465.45 ns | 1.3 % | 42.31 ns |
| `freeze` | 100 | Dynamic → Frozen | 3.62 µs | 3.63 µs | 3.68 µs | 3.64 µs | 1.7 % | 36.09 ns |
| `freeze` | 1,000 | Dynamic → Frozen | 35.08 µs | 37.11 µs | 35.02 µs | 35.74 µs | 5.8 % | 35.70 ns |
| `unfreeze` | 10 | Frozen → Dynamic | 104.97 ns | 104.96 ns | 110.50 ns | 106.81 ns | 5.2 % | 9.71 ns |
| `unfreeze` | 100 | Frozen → Dynamic | 655.03 ns | 668.75 ns | 664.21 ns | 662.66 ns | 2.1 % | 6.56 ns |
| `unfreeze` | 1,000 | Frozen → Dynamic | 5.82 µs | 5.97 µs | 5.96 µs | 5.91 µs | 2.5 % | 5.91 ns |

### Algorithms

Triangle chain of `n` triangles: `2n + 1` nodes, `3n` edges. Per element: per edge.

| Benchmark | n | State | Run 1 | Run 2 | Run 3 | Mean | Spread | Per element |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| `articulation_points` | 10 | Frozen | 1.29 µs | 1.34 µs | 1.38 µs | 1.34 µs | 6.6 % | 44.64 ns |
| `articulation_points` | 100 | Frozen | 10.53 µs | 10.73 µs | 11.04 µs | 10.77 µs | 4.8 % | 35.89 ns |
| `articulation_points` | 1,000 | Frozen | 128.55 µs | 131.77 µs | 118.55 µs | 126.29 µs | 10.5 % | 42.10 ns |
| `articulation_points` | 5,000 | Frozen | 912.50 µs | 910.79 µs | 872.58 µs | 898.62 µs | 4.4 % | 59.91 ns |
| `bridges` | 10 | Frozen | 1.09 µs | 1.10 µs | 1.15 µs | 1.11 µs | 5.5 % | 37.06 ns |
| `bridges` | 100 | Frozen | 10.16 µs | 10.42 µs | 10.34 µs | 10.30 µs | 2.5 % | 34.35 ns |
| `bridges` | 1,000 | Frozen | 123.80 µs | 120.17 µs | 120.82 µs | 121.60 µs | 3.0 % | 40.53 ns |
| `bridges` | 5,000 | Frozen | 863.95 µs | 857.99 µs | 852.27 µs | 858.07 µs | 1.4 % | 57.20 ns |
| `biconnected_components` | 10 | Frozen | 1.96 µs | 1.90 µs | 1.93 µs | 1.93 µs | 2.6 % | 64.36 ns |
| `biconnected_components` | 100 | Frozen | 17.43 µs | 17.14 µs | 16.73 µs | 17.10 µs | 4.1 % | 57.00 ns |
| `biconnected_components` | 1,000 | Frozen | 191.28 µs | 191.84 µs | 185.32 µs | 189.48 µs | 3.4 % | 63.16 ns |
| `biconnected_components` | 5,000 | Frozen | 1.183 ms | 1.165 ms | 1.232 ms | 1.193 ms | 5.7 % | 79.56 ns |

### Observations

**Every walk runs on the frozen graph.** The benchmarks call `freeze` before walking, because
`outbound_edges` and `inbound_edges` return `GraphNotFrozen` in the dynamic state today. The State
column gives the state of every row.

**`inbound_walk` at 100 and 1,000 nodes runs at two speeds.** Per step walked:

| Walk | n | Run 1 | Run 2 | Run 3 |
|---|---:|---:|---:|---:|
| `outbound_walk` | 100 | 1.98 ns | 2.10 ns | 1.96 ns |
| `outbound_walk` | 1,000 | 1.56 ns | 1.60 ns | 1.54 ns |
| `inbound_walk` | 100 | 2.18 ns | 0.84 ns | 2.17 ns |
| `inbound_walk` | 1,000 | 0.72 ns | 1.64 ns | 0.87 ns |

The slower level of `inbound_walk` matches `outbound_walk`; the level that varies is faster, about
twice as fast. Cache misses do not explain it: a walk reads one direction's offsets and targets,
about 16 KB at 1,000 nodes, which fits in the first-level data cache, and a miss can only slow a
walk down. Each step makes two dependent loads, the node's offset and then the target it points
at, so a step cannot finish faster than those loads take unless the core predicts the next address
and starts early. On the ordered chain the addresses fall by exactly 8 bytes per step, the most
predictable pattern there is. Why only the inbound walk reaches the fast level is not settled by
three runs.

A real graph's traversal does not step through memory in a fixed stride, so the ordered walks are
the best case. `outbound_walk_shuffled` and `inbound_walk_shuffled`, added after these runs, walk
the same chain laid out in a seeded random order of positions (`utils::build_shuffled_chain`),
where the next address cannot be predicted.

**Pending:** rows for `outbound_walk_shuffled` and `inbound_walk_shuffled`, from three runs of
`cargo bench -p ultragraph -- walk`.
