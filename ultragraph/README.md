# ultragraph

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[ossf-badge]: https://bestpractices.coreinfrastructure.org/projects/7568/badge

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/ultragraph

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/ultragraph/latest/ultragraph/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

## Goal

ultragraph provides a directed graph data structure with two states: a mutable graph for construction and an
immutable graph for fast analysis. Users pick the state that fits each phase of their application.

## Features

* **Dual-State Architecture:** A `DynamicGraph` for mutations and a `Static` (frozen) `CsmGraph` for fast reads.
* **Mutations:** `add_node`, `add_edge`, `remove_node`, and others in the dynamic state.
* **Algorithms:** Algorithms (`shortest_path`, `topological_sort`, `has_cycle`) that operate
  on the frozen graph.
* **Efficient Traversals:** Cache-friendly neighbor iteration (`outbound_edges`) on the frozen graph.
* **Full Lifecycle:** `freeze()` a graph for analysis and `unfreeze()` it to resume mutations.

##  Implementation

`ultragraph` separates graph *construction* from graph *analysis* with a state machine.

### 1. The Dynamic State: `DynamicGraph`

The default state, optimized for mutations.

* **Underlying Structure:** A standard adjacency list (`Vec<Vec<...>>`).
* **Best For:** Building and modifying the graph topology: adding, removing, and updating nodes and edges.
* **Performance:** Scattered memory allocation causes CPU cache misses, which slows traversals.

### 2. The Transition: `freeze()`

Calling `g.freeze()` consumes the `DynamicGraph` and transforms it into a `CsmGraph`, a one-time "compilation" step
that prepares the graph for analysis.

### 3. The Static State: `CsmGraph` (Frozen)

The read-only state, optimized for analysis.

* **Underlying Structure:** A **Compressed Sparse Row (CSR)** format that stores all edges in a few large, contiguous
  memory blocks, which keeps the CPU cache warm.
* **Best For:** Running algorithms, complex traversals, and any read-heavy workload.
* **Performance:** Data locality makes traversals and algorithms on a `CsmGraph` orders of magnitude faster than on a
  `DynamicGraph`. All methods on the `GraphAlgorithms` trait require this state.

### 4. The Reverse Transition: `unfreeze()`

`g.unfreeze()` converts the `CsmGraph` back into a `DynamicGraph`, so mutation and analysis can alternate.

##  Graph Algorithms

The `ultragraph` crate provides read-only analytical algorithms, implemented on the static graph
structure.

* **`find_cycle()`**: Finds a single cycle in the
  graph and returns the path of nodes that form it.
  Returns `None` if the graph is a Directed Acyclic Graph
  (DAG).

* **`has_cycle()`**: Checks whether the graph contains a
  directed cycle.

* **`topological_sort()`**: Computes a topological
  sort of the graph if it is a DAG. Returns `None` if the
  graph contains a cycle.

* **`is_reachable(start_index, stop_index)`**: Checks
  whether a path of any length exists from a start node to a
  stop node.

* **`shortest_path_len(start_index, stop_index)`**:
  Returns the length (number of nodes) of the shortest
  path from a start node to a stop node.

* **`shortest_path(start_index, stop_index)`**: Finds
  the complete shortest path (sequence of nodes) from a
  start node to a stop node.

* **`shortest_weighted_path(start_index, stop_
 index)`**: Finds the shortest path in a weighted graph
  using Dijkstra's algorithm, returning the path and its
  total cost.

* **`strongly_connected_components()`**: Finds all
  Strongly Connected Components (SCCs) in the graph using
  Tarjan's algorithm, returning one node set per SCC.

* **`betweenness_centrality()`**: Measures each node's importance with Brandes' algorithm by
  counting how often it appears on the shortest paths between all other pairs of nodes.

## Benchmark Results

### Dynamic Graph

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
* LARGE = 1000;

Benchmark source code in [ultragraph/benches ](../ultragraph/benches/benchmarks)

---

* **`add_node` Performance:** Median times range from **29 to 46 nanoseconds**, consistent with an O(1) operation.
  System-level memory allocation causes the variation.
* **`get_node` Performance:** The median time stays at about **4 nanoseconds** across all graph sizes, an O(1) lookup
  into the underlying `Vec`.
* **Outliers:** Outliers are normal on a non-dedicated machine; `Criterion` identifies and filters these system-level
  interruptions.

### Static CSM Graph

| Operation       | Scale | Graph Configuration                          |  Mean Time  | Throughput (Est.)        |
|:----------------|:------|:---------------------------------------------|:-----------:|:-------------------------|
| **Edge Lookup** | Tiny  | `contains_edge` (Linear Scan, degree < 64)   | **~7.7 ns** | ~130 Million lookups/sec |
|                 | Tiny  | `contains_edge` (Binary Search, degree > 64) | **~8.2 ns** | ~122 Million lookups/sec |
| **Algorithms**  | Small | `shortest_path` (1k nodes)                   | **~5.3 µs** | ~188,000 paths/sec       |
|                 | Small | `topological_sort` (1k nodes, DAG)           | **~5.2 µs** | ~192,000 sorts/sec       |
|                 | Small | `find_cycle` (1k nodes, has cycle)           | **~7.1 µs** | ~140,000 checks/sec      |
|                 | Large | `shortest_path` (1M nodes, 5M edges)         | **~482 µs** | ~2,000 paths/sec         |
|                 | Large | `topological_sort` (1M nodes, 5M edges)      | **~2.9 ms** | ~345 sorts/sec           |
| **Lifecycle**   | Small | `freeze` (1k nodes, 999 edges)               | **~42 µs**  | ~23,800 freezes/sec      |
|                 | Small | `unfreeze` (1k nodes, 999 edges)             | **~12 µs**  | ~81,600 unfreezes/sec    |
|                 | Large | `freeze` (1M nodes, 5M edges)                | **~75 ms**  | ~13 freezes/sec          |
|                 | Large | `unfreeze` (1M nodes, 5M edges)              | **~24 ms**  | ~41 unfreezes/sec        |

*(Note: Time units are nanoseconds (ns), microseconds (µs), and milliseconds (ms). Throughput is an approximate
calculation based on the mean time.)*

Benchmark source code in [deep_causality/benches ](../deep_causality/benches)

## Performance Design

`ultragraph`'s static analysis structure, `CsmGraph`, follows the sparse graph representation in the paper
"NWHy: A Framework for Hypergraph Analytics" (Liu et al.). It adopts the paper's two mutually-indexed Compressed
Sparse Row (CSR) structures for `O(degree)` bidirectional traversal: one for forward (outbound) edges and one for the
transposed graph's backward (inbound) edges.

`ultragraph` extends this baseline in three ways to support dynamically evolving systems.

1. **Struct of Arrays (SoA) Memory Layout:** The internal CSR adjacency structures use two parallel vectors,
   `Vec<target>` and `Vec<weight>`, instead of a single `Vec<(target, weight)>`. Topology-only algorithms (e.g.,
   reachability, cycle detection) iterate over the `targets` vector alone, so they never load unused edge weights into
   the CPU cache, which saves memory bandwidth and reduces cache pollution.

2. **Adaptive Edge Containment Checks:** `contains_edge` checks the source node's degree in `O(1)` and picks a search
   strategy: a cache-friendly linear scan for low-degree nodes (fewer neighbors than a compile-time threshold, e.g.,
   64) and a binary search for high-degree nodes.

3. **Formal Evolutionary Lifecycle:** A two-state model for graph evolution: a mutable `DynamicGraph` with `O(1)` node
   and edge additions, and the immutable `CsmGraph` for analysis. `O(V + E)` `.freeze()` and `.unfreeze()` operations
   move between the states. Systems that evolve their structure, such as models of emergent causality, use this to
   separate the mutation phase from the analysis phase.

## Install

Run:

```bash
cargo add ultragraph
```

Or add the following to your Cargo.toml:

```toml
ultragraph = "current_version"
```

## Usage

See:

* [Examples](examples)
* [Benchmarks](benches)
* [Tests](tests)

```rust
use ultragraph::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}
impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.x)
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = UltraGraph::with_capacity(10, None);
    assert!(g.is_empty());

    // Add nodes to the graph
    let root_index = g.add_root_node(Data { x: 3 })?;
    let node_a_index = g.add_node(Data { x: 7 })?;
    let node_b_index = g.add_node(Data { x: 9 })?;
    let node_c_index = g.add_node(Data { x: 11 })?;

    // Link nodes together
    g.add_edge(root_index, node_a_index, ())?;
    g.add_edge(node_a_index, node_b_index, ())?;
    g.add_edge(root_index, node_c_index, ())?;

    // Get node a
    let node = g.get_node(node_a_index);
    assert!(node.is_some());

    let data = node?;
    assert_eq!(data.x, 7);
    println!("Retrieved Node A with data: {data:?}");

    println!("Freeze the graph to enable high-performance traversal");
    g.freeze(); // This is the crucial step!

    // neighbors is just a vector of indices
    // so you can iterate over them to get the actual nodes
    println!("Neighbors of root node: ");
    println!("Iterating over neighbors of Node A with a for loop:");
    for neighbor_index in g.outbound_edges(root_index)? {
        // You can use the index to get the node's data
        let neighbor_data = g.get_node(neighbor_index)?;
        println!("- Found neighbor: {neighbor_data} at index {neighbor_index}");
    }

    Ok(())
}
```

## Credits

The project took inspiration from:

* [petgraph](https://github.com/petgraph/petgraph)
* [Dachshund](https://github.com/facebookresearch/dachshund)
* [Hypergraph](https://github.com/yamafaktory/hypergraph)

## Contribution

Contributions are welcome, especially documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

See the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

