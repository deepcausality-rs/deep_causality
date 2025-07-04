# ultragraph

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[ossf-badge]: https://bestpractices.coreinfrastructure.org/projects/7568/badge

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/ultragraph

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/ultragraph/latest/ultragraph/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## 📣 Goal

`ultragraph` provides a high-performance, ergonomic, and directed graph data structure. It is designed around a
state-machine architecture that offers both a flexible, mutable graph and a blazing-fast, immutable graph, allowing
users to choose the right tool for the right phase of their application.

## 🎁 Features

* **Dual-State Architecture:** A `DynamicGraph` for easy mutations and a `Static` (frozen) `CsmGraph` for extreme read
  performance.
* **Ergonomic Mutations:** Simple `add_node`, `add_edge`, `remove_node`, etc., in the dynamic state.
* **High-Performance Algorithms:** A suite of algorithms (`shortest_path`, `topological_sort`, `has_cycle`) that operate
  on the frozen graph.
* **Efficient Traversals:** Cache-friendly neighbor iteration (`outbound_edges`) on the frozen graph.
* **Full Lifecycle:** Seamlessly `freeze()` a graph for analysis and `unfreeze()` it to resume mutations.

## ⚡️ Implementation

`ultragraph`'s power comes from its state-machine design, which separates the concerns of graph *construction* from
graph *analysis*.

### 1. The Dynamic State: `DynamicGraph`

This is the default state, optimized for flexibility and mutations.

* **Underlying Structure:** A standard adjacency list (`Vec<Vec<...>>`).
* **Best For:** Building and modifying your graph topology. Adding, removing, and updating nodes and edges is
  straightforward.
* **Performance:** While flexible, this representation is not ideal for high-speed traversals due to scattered memory
  allocation, which can lead to CPU cache misses.

### 2. The Transition: `freeze()`

This is the bridge between the two states. Calling `g.freeze()` consumes the `DynamicGraph` and transforms it into a
`CsmGraph`. Think of this as a one-time "compilation" step that prepares your graph for high-speed analysis.

### 3. The Static State: `CsmGraph` (Frozen)

This is the high-performance, read-only state.

* **Underlying Structure:** A **Compressed Sparse Row (CSR)** format. This layout stores all graph edges in a few large,
  contiguous memory blocks, making it extremely cache-friendly for the CPU.
* **Best For:** Running algorithms, performing complex traversals, and any read-heavy workload.
* **Performance:** Because of its exceptional data locality, traversals and algorithms on a `CsmGraph` are orders of
  magnitude faster than on a `DynamicGraph`. All methods on the `GraphAlgorithms` trait require the graph to be in this
  state.

### 4. The Reverse Transition: `unfreeze()`

If you need to make further changes after a period of analysis, `g.unfreeze()` efficiently converts the `CsmGraph` back
into a `DynamicGraph`, allowing the cycle of mutation and analysis to begin again.

## 🚀 Performance

The CSR format of the frozen `CsmGraph` provides exceptional performance for analytical workloads. The one-time cost of
the `freeze()` operation unlocks repeatable, high-speed analysis.

| Benchmark                                       |     Time |
|-------------------------------------------------|---------:|
| `small_linear_graph_reason_all_causes`          | 78.79 ns |
| `medium_linear_graph_reason_all_causes`         |  5.23 µs |
| `large_linear_graph_reason_all_causes`          | 51.70 µs |
| `large_linear_graph_reason_subgraph_from_cause` | 25.79 µs |
| `large_linear_graph_reason_shortest_path`       | 43.80 µs |
| `large_reason_single_cause`                     |  4.86 ns |

## 🚀 Install

Just run:

```bash
cargo add ultragraph
```

Alternatively, add the following to your Cargo.toml

```toml
ultragraph = "current_version"
```

## ⭐ Usage

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
```

## 🙏 Credits

The project took inspiration from:

* [petgraph](https://github.com/petgraph/petgraph)
* [Dachshund](https://github.com/facebookresearch/dachshund)
* [Hypergraph](https://github.com/yamafaktory/hypergraph)

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 💻 Author

* Marvin Hansen, [GitHub](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC