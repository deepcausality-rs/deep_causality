---
title: "UltraGraph 0.8: 1,300× Faster Graph Analytics — No Cluster Needed"
description: "This post summarizes the new features of UltraGraph v.0.8."
date: 2025-07-06
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

Today, the DeepCausality project announces UltraGraph v0.8, a ground-up rewrite of our hypergraph library
delivering up to 1,300x speedups, enabling sub-second analytics on 100-million-node graphs, and making
billion-node analytics economically feasible on a single machine.

This release introduces a new dual-state architecture designed for the complete lifecycle of graph data. Graphs begin in
a flexible DynamicGraph state, optimized for fast, O(1) mutations as the structure evolves. When you're ready for
analysis, a single .freeze() call transforms the graph into a hyper-optimized, immutable CsmGraph based on a
cache-friendly Struct of Arrays (SoA) memory layout. This "compilation" step is the key to our performance, virtually
eliminating cache misses and unlocking near-linear scaling. If the graph needs to evolve further, simply .unfreeze().

The new implementation was born out of necessity. The ongoing work on emergent causality within the DeepCausality
project demanded a system that could be both highly dynamic during evolution and blazing fast during analysis. Our
previous petgraph-based foundation could not keep up any longer, so we built the next generation of UltraGraph, inspired
by the state-of-the-art [NWHypergraph (NWHy)](https://par.nsf.gov/servlets/purl/10381502) architecture and heavily
optimized for Rust's memory model. The key elements of the new UltraGraph implementation are:

* The introduction of a SoA CsrAdjacency type instead of a simple CSR row.
* The separation of forward and backward CsrAdjacency.
* The addition of a new dual-state graph evolution lifecycle.

### SoA CsrAdjacency

In conventional CSR-based graph representations (like NWHypergraph), adjacency information is typically packed together
row-wise in a "single structure" per edge or neighbor, which is technically an Array of Structs (AoS) layout, or in
simple terms, “rows of neighbors.” UltraGraph, however, takes a different approach. UltraGraph introduces a
CsrAdjacency<W> type that implements a Struct of Arrays (SoA) pattern:

```rust 
#[derive(Default)]
pub(crate) struct CsrAdjacency<W> {
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<usize>,
    pub(crate) weights: Vec<W>,
}
```

This layout splits each component of the adjacency data — offsets, targets, and weights — into separate, contiguous
memory regions:

* offsets: Starting positions of each node’s adjacency list.
* targets: The target node indices for each edge.
* weights: Edge weights (optional, type-parametric).

Benchmarks show up to 1,300× speedups compared to the previous implementation, and memory usage
seems to approach the theoretical limit, with the implication that billion-node graph analytics is now possible on a
single commodity workstation instead of a cluster.

This has two critical advantages:

* **Better cache utilization:** When performing traversal or shortest path algorithms, the CPU can stream just the
  fields it
  needs (often offsets and targets) without pulling in unnecessary weight data — avoiding cache pollution and improving
  locality.

* **SIMD-friendliness:** SoA layouts enable vectorized processing (e.g., with AVX) far more easily than AoS.

### Forward and Backward CsrAdjacency

Moreover, UltraGraph uses two separate CsrAdjacency instances: one for successor or outbound edges and another one for
backward or inbound edges. This dual-CSR setup is more explicit and efficient than mixing directions
within a single row layout because it reduces CPU cache pollution and thereby directly supports fast and efficient
algorithm implementations. It is worth noting that some CSR systems only store forward edges and reconstruct backward
edges on the fly, which conserves memory but is computationally inefficient. UltraGraph deliberately traded a bit more
memory for drastically better algorithm performance, as shown in the benchmarks.

The backward node list is particularly useful in causality-based inference algorithms, where backtracking is often
required, and is thus particularly well suited for DeepCausality. Memory usage remains low due to the combined effects
of the Struct of Arrays layout and the clean separation between forward and backward adjacency.
This design leads to a predictable, flat memory layout with minimal overhead:

* No per-node allocation overhead
* No padding, no vtables, no boxed pointers
* No HashMaps or linked lists
* No indexing needed due to simple offset
* Struct of Arrays (SoA) leads to a predictable, flat memory layout that incentivizes CPU prefetching.
* When a node has no outbound nodes or weights, there is zero allocation, thus saving memory.
* Near-zero memory fragmentation

Memory fragmentation is largely prevented because of the freeze/unfreeze operation in the graph evolution lifecycle.
Calling `.freeze()` compacts and linearizes the structure, which removes any prior allocation gaps from the dynamic
phase
and thus results in a clean, continuous memory structure.

### The New Graph Evolution Lifecycle

The single biggest change in UltraGraph 0.8 is its new dual-state architecture. We recognize that graph-based systems
have two distinct phases: a dynamic "Evolve" phase, where the structure is built and modified, and a stable "Analyze"
phase, where high-speed queries are essential.

1) **The DynamicGraph State:** This is now the default state for every new graph. It's an adjacency-list-based structure
   optimized for flexibility. Adding nodes and edges is a cheap O(1) operation, perfect for systems where the graph
   structure emerges dynamically over time.

2) **The Frozen State:** When you're ready for analysis, you call `.freeze()`. This is a one-time "compilation" step
   that transforms your graph into a hyper-optimized, immutable Compressed Sparse Row (CSR) format. This state is
   designed for one thing: raw speed.

This new lifecycle is our answer to the challenges of emergent causality. It provides a controlled, predictable way to
transition between a state of evolution and a state of high-performance analysis. The best part? When your frozen graph
needs
to be modified, you call `.unfreeze()`, and your graph structure can evolve further.

## Performance That Speaks for Itself

All benchmarks were completed on a 2023 Macbook Pro with a M3 Max CPU.

### Dynamic Graph

The dynamic graph structure, when the graph is in an unfrozen state, is optimized for efficient mutation.
The table below summarizes the performance of the key operations.

| Benchmark Name    | Graph Size | Operation  | Estimated Time (Median) | Outliers Detected |
|:------------------|:-----------|:-----------|:------------------------|:------------------|
| `small_add_node`  | 10         | `add_node` | 29.099 ns               | 14% (14 / 100)    |
| `medium_add_node` | 100        | `add_node` | 45.864 ns               | 12% (12 / 100)    |
| `large_add_node`  | 1,000      | `add_node` | 39.293 ns               | 11% (11 / 100)    |
| `small_get_node`  | 10         | `get_node` | 3.9417 ns               | 8% (8 / 100)      |
| `medium_get_node` | 100        | `get_node` | 3.9849 ns               | 2% (2 / 100)      |
| `large_get_node`  | 1,000      | `get_node` | 3.9916 ns               | 7% (7 / 100)      |

* SMALL = 10;
* MEDIUM = 100;
* LARGE = 1,000;

Benchmark source code
in [ultragraph/benches ](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph/benches)

### Static CSM Graph

The impact of the new dual-state architecture is most visible when comparing the new CsmGraph implementation against our
previous MatrixGraph-based engine. By freezing a graph into a stable `CsmGraph`, we eliminate CPU cache misses inherent
to traditional, flexible graph structures, thereby allowing the CPU to operate with maximum efficiency. The following
table compares the performance of graph-based reasoning algorithms in DeepCausality before and after the `UltraGraph`
rewrite. The "After" benchmarks were run on a frozen `CsmGraph`, which leverages a highly efficient Compressed Sparse
Row (CSR) memory layout.

| Benchmark                                            | Time Before (Old) | Time After (New) | Improvement Factor |
|------------------------------------------------------|------------------:|-----------------:|-------------------:|
| `small_linear_graph_reason_all_causes`               |          2.760 µs |         78.79 ns |           **~35x** |
| `small_linear_graph_reason_subgraph_from_cause`      |          1.507 µs |         52.41 ns |           **~28x** |
| `small_linear_graph_reason_shortest_path`            |          1.690 µs |        120.19 ns |           **~14x** |
| `medium_linear_graph_reason_all_causes`              |        509.940 µs |          5.23 µs |           **~97x** |
| `medium_linear_graph_reason_subgraph_from_cause`     |        245.250 µs |          2.63 µs |           **~93x** |
| `medium_linear_graph_reason_shortest_path`           |        286.080 µs |          4.35 µs |           **~65x** |
| `large_linear_graph_reason_all_causes`               |         70.221 ms |         51.70 µs |        **~1,358x** |
| `large_linear_graph_reason_subgraph_from_cause`      |         34.933 ms |         25.79 µs |        **~1,354x** |
| `large_linear_graph_reason_shortest_path`            |         35.424 ms |         43.80 µs |          **~808x** |
| `small_multi_layer_graph_reason_all_causes`          |          1.248 µs |         43.59 ns |           **~28x** |
| `small_multi_layer_graph_reason_subgraph_from_cause` |        489.420 ns |         32.02 ns |           **~15x** |
| `small_multi_layer_graph_reason_shortest_path`       |        427.450 ns |         62.99 ns |            **~7x** |

Average Speedup across all use cases: ~300x

* SMALL = 10;
* MEDIUM = 1,000;
* LARGE = 10,000;

Benchmark source code
in [deep_causality/benches ](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/benches)

The new architecture causes the largest and most significant performance gains for algorithms running over large
graphs (10k or more nodes) because of its close alignment with contemporary hardware.
By combining an instantaneous O(1) lookup with a perfectly linear scan over a node's neighbors, Ultragraph creates the
ideal scenario for the CPU's prefetcher to easily anticipates a straight-line sprint through memory. The result becomes
more notable the more data the prefetcher can load ahead of time, thus the disproportional performance gains on larger
graphs.

### Memory Usage and Scaling

| Number of Nodes | Memory Usage | `evaluate_subgraph_from_root` | `evaluate_shortest_path` | `evaluate_single_cause` |
|:----------------|:-------------|:------------------------------|:-------------------------|:------------------------|
| **100,000**     | 55 MB        | 0.68 ms                       | 0.57 ms                  | **5.4 ns**              |
| **1,000,000**   | 350 MB       | 11.12 ms                      | 6.95 ms                  | **5.5 ns**              |
| **10,000,000**  | 3 GB         | 114 ms                        | 85.80 ms                 | **5.6 ns**              |
| **100,000,000** | 32 GB        | 1.23 s                        | 0.98 s                   | **5.5 ns**              |

**Key Observations:**

* **Constant Time to get a single node**: The benchmark evaluate_single_cause returns always takes about 5.5. ns regardless of
  whether the node lookup happens before or during the benchmark loop and regardless of whether blackbox is used or not. The time does not change with the size of the graph because the implementation of the underlying get_node is just two O(1) array
  lookup to find the index and a straight redirect to a virtual memory address, which in this case, is close to the
  physical limit of Apples UMA architecture. On another architectures, the exact value may differ, but in general should
  remain constant and only be bound by the bandwidth and latency of the physical memory.

* **Sub-Second traversal up to a hundred million nodes:** Notice, this uses a linear graph as base line by purpose to
  estimate the best case scenario. On complex or imbalanced graphs, you will see different results. The key here is that
  a linear graph plays exactly to the strength of the continues memory layout created by the underlying CSR structure.

* **Near-Linear Scalability:** Both the memory usage and the execution time for the subgraph and shortest_path tasks
  appear
  to scale in a roughly linear fashion with the number of nodes. A 10x increase in nodes results in a roughly 10x-15x
  increase in time and memory. This indicates a highly efficient implementation with a complexity close to O(N).

* **All benchmarks are single-threaded.** The performance you see is from a single core. Initial experiments showed that
  for graphs up to 1 million nodes, the overhead of even highly-optimized parallel libraries like rayon resulted in a
  net performance loss of 30% or more compared to the single-threaded version. This is a testament to the extreme
  efficiency of the CSR layout when paired with modern CPU caches and prefetchers.
  The results suggest that meaningful gains from concurrency will only appear on massive graphs (likely 10M-50M nodes
  and above). However, this requires a concurrency model carefully designed to avoid the cache-invalidation issues
  common in work-stealing schedulers (used by rayon and Tokio). Developing such a cache-friendly concurrency
  model was out of scope for this release, but it remains an open challenge for future work. Contributions are
  welcome.

Based on Ultragraph's performance, a simple interpolation shows:

* 1 Billion Nodes: The shortest path on a graph of this size in about 12 seconds, requiring approximately 236 GB of RAM.
* 10 Billion Nodes: Require just under 2 TB of RAM and could run a shortest path query in about 2.5 minutes.

### Economic Impact

This level of performance on a single machine has profound economic implications, which is best demonstrated
by an estimate of the Total Cost of Ownership (TCO) for running a 10-billion node graph on major cloud providers and
bare metal.

**Assumptions:**

* **Instance Type:** AWS `u-3tb1.56xlarge` (2.9 TiB RAM) vs. GCP `m1-megamem-96` (2.9 TiB RAM).
* **Region:** `us-east-1` (N. Virginia) for AWS and `us-central1` (Iowa) for GCP, which are typically cost-effective.
* **Commitment:** **1-Year Committed Plan** (AWS Savings Plan / GCP CUD), as this is the standard for any persistent
  workload and offers significant savings over on-demand pricing.
* **Storage:** 500 GB of high-performance SSD storage (`gp3` on AWS, `pd-ssd` on GCP) for the OS

**Scenario 1: Persistent Workload TCO (1-Year Commitment)**

| Cost Component               | AWS (u-3tb1.56xlarge) | GCP (m1-megamem-96)           | Bare Metal (Dedicated Server) |
|:-----------------------------|:----------------------|:------------------------------|:------------------------------|
| **Commitment Plan**          | 1-Year Savings Plan   | 1-Year Committed Use Discount | 1-Year Contract               |
| **Effective Hourly Rate**    | ~$10.99               | ~$10.57                       | ~$7.53 (Calculated)           |
| **Monthly Billed Cost**      | **~$8,071**           | **~$7,813**                   | **~$5,542**                   |
| **Total Annual Billed Cost** | **~$96,852**          | **~$93,756**                  | **~$66,504**                  |

**Scenario 2: Temporary Workload TCO (On-Demand / Monthly Rate)**

| Cost Component                 | AWS (u-3tb1.56xlarge) | GCP (m1-megamem-96) | Bare Metal (Dedicated Server) |
|:-------------------------------|:----------------------|:--------------------|:------------------------------|
| **Commitment Plan**            | On-Demand             | On-Demand           | Monthly (No Contract)         |
| **Effective Hourly Rate**      | ~$28.77               | ~$25.84             | ~$9.32 (Calculated)           |
| **Cost for 1 Month (730 hrs)** | **~$21,002**          | **~$18,863**        | **~$6,800**                   |
| **Cost for 3 Months**          | **~$63,006**          | **~$56,589**        | **~$20,400**                  |

The most important takeaway is that for an annual cost between $67,000 and $95,000 you can analyze a 10-billion node
graph. For a one-month long project, say a proof of concept, a bare-metal server costs roughly one-third of what an on-demand
cloud instance would cost. Any series A funded startup can now at least explore large scale graph analytics without 
over spending on engineering, hardware or expensive license fees for commercial solutions.  

## Graph Algorithms

The `UltraGraph` crate provides a selection of high-performance algorithms for
graph analysis. These algorithms are implemented on the static, optimized graph structure for fast and
efficient computation.

### 🔄 `find_cycle()`

**Use Case:**

- **Dependency resolution:** Detect circular dependencies in software build systems (e.g., Cargo, Bazel).
- **Process control:** Ensure process chains are acyclic.

### ❓ `has_cycle()`

**Use Case:**

- **Workflow engines:** Verify Directed Acyclic Graph (DAG) constraints in ETL jobs.
- **Graph validation:** Ensure schema or data pipelines conform to top-down execution semantics.

### 🔃 `topological_sort()`

**Use Case:**

- **Build systems / Task scheduling:** Order steps where some tasks depend on others (compilers, project planners).
- **Data science pipelines:** Stage DAG-based machine learning workflows (e.g., feature extraction → training →
  evaluation).

### 📡 `is_reachable(start_index, stop_index)`

**Use Case:**

- **Access control:** Can User A reach Resource B? (e.g., network ACLs or graph-based permission models).
- **Social graph analysis:** Check if two users are connected via follower/following chains.

### 📏 `shortest_path_len(start_index, stop_index)`

**Use Case:**

- **Performance modeling:** Compute minimal latency paths in distributed systems or manufacturing processes.
- **Simplified heuristics:** Estimate costs in search/AI agents without computing full paths.

### 🛣️ `shortest_path(start_index, stop_index)`

**Use Case:**

- **Navigation/routing:** Real-time route finding (e.g., maps, logistics).
- **Interactive systems:** Trace dependency or influence paths between data nodes (e.g., in causal graphs).

### 📉 `shortest_weighted_path(start_index, stop_index)`

**Use Case:**

- **Cost-aware routing:** Find paths in weighted graphs like transport networks, financial transaction flows, or
  workflow runtimes.
- **Risk modeling:** Choose minimum-exposure routes in threat graphs or attack trees.

### 🧩 `strongly_connected_components()`

**Use Case:**

- **Community detection:** Find user clusters in social graphs or discussion networks.
- **Deadlock/root cause detection:** In concurrent systems or process management (e.g., OS schedulers, container
  orchestration).

### 🧠 `betweenness_centrality()`

**Use Case:**

- **Influencer detection:** Identify key nodes in social, transportation, or communication networks.
- **Bottleneck analysis:** Discover chokepoints in network infrastructure or data processing graphs.

## What This Means for DeepCausality

This new version of `UltraGraph` is the engine powering the next version of DeepCausality. It provides the foundation
to:

* **Scale to Massive Graphs:** Analyze systems at a scale that was previously impossible due to memory constraints.
* **Model Emergent Systems:** Directly support the "grow as you go" nature of dynamic causality.
* **Build with Confidence:** Leverage a stable, performant, and dependency-free core for mission-critical applications.

## Conclusion

UltraGraph 0.8 offers unprecedented speed for hypergraph analytics on larger graphs. Try it. The Future is now. 

* Explore [the code and examples on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph).
* [Join the community](https://deepcausality.com/community).

## About

DeepCausality is a dynamic-causality framework that enables fast and deterministic context-aware
causal reasoning in Rust. The DeepCausality project is hosted at the Linux Foundation for Artificial Intelligence and
Data (LF AI & Data). Learn more about DeepCausality on GitHub and join the DeepCausality-Announce Mailing List.

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open-source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all
members of the community. For more information, please visit lfaidata.foundation.

Please give us a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)

