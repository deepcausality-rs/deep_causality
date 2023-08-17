# ultragraph

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Audit][audit-url]
![Clippy][clippy-url]
![Tests][test-url]
[![OpenSSF Best Practices][ossf-badge]][ossf-url]

[ossf-badge]: https://bestpractices.coreinfrastructure.org/projects/7568/badge

[ossf-url]:https://bestpractices.coreinfrastructure.org/projects/7568

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/ultragraph

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality/latest/ultragraph/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg

[clippy-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust-clippy.yml/badge.svg

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## 📣 Goal

Ultragraph aims to simplify working with directed graph data structures by adding more features such
as storing and retrieving nodes directly from the graph, getting all neighbors of a node, and
some basic algorithm such as shortest path.

## 🎁 Features

* Stores nodes directly in the graph for easy access
* Access to all nodes and edges in the graph (get_node & get_all_nodes)
* Access to all neighbors of a node (outgoing_edges)
* Shortest path algorithm

## ⚡️ Implementation

* Wraps petgraph under the hood
* Stores relations in a matrix graph and nodes in a hashmap
* Supports multiple implementations via storage trait pattern
* Adds proper error handling

## 🚀 Install

Just run:

```toml
cargo add ultragraph
```

Alternatively, add the following to your Cargo.toml

```toml
ultragraph = "current_version"
```

## ⭐ Usage

See:

* [Examples](examples)
* [Test](tests)

```rust
use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}

pub fn main() {
    let mut g = ultragraph::with_capacity::<Data>(10);

    // Add nodes to the graph
    let root_index = g.add_root_node(Data { x: 3 });
    let node_a_index = g.add_node(Data { x: 7 });
    let node_b_index = g.add_node(Data { x: 9 });
    let node_c_index = g.add_node(Data { x: 11 });

    // Link nodes together
    // Link root node to node a
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
    // Link node a to node b
    let res = g.add_edge(node_a_index, node_b_index);
    assert!(res.is_ok());
    // Link node root to node c
    let res = g.add_edge(root_index, node_c_index);
    assert!(res.is_ok());

    // Get node a
    let node = g.get_node(node_a_index);
    assert!(node.is_some());

    let data = node.unwrap();
    assert_eq!(data.x, 7);

    // get all outgoing_edges of root node
    let neighbors = g.outgoing_edges(root_index).unwrap();

    // root node has 2 outgoing_edges: node a and node b
    assert_eq!(neighbors.len(), 2);

    // neighbors is just a vector of indices
    // so you can iterate over them to get the actual nodes
    println!("Neighbors of root node: ");
    for n in neighbors{
        let node = g.get_node(n).unwrap();
        println!("node: {:?}", node);
    }
}
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