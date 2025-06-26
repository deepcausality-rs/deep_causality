/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}

pub fn main() {
    test_get_node();
    test_outgoing_edges();
    test_shortest_path();
}

fn test_get_node() {
    println!("test_get_node");
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
    for n in neighbors {
        let node = g.get_node(n).unwrap();
        println!("node: {:?}", node);
    }
}

fn test_shortest_path() {
    println!("test_shortest_path");

    let mut g = ultragraph::with_capacity::<Data>(10);
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // Add nodes to the graph
    let root_index = g.add_root_node(Data { x: 1 });
    let node_a_index = g.add_node(Data { x: 6 });
    let node_b_index = g.add_node(Data { x: 9 });
    let node_c_index = g.add_node(Data { x: 11 });
    let node_d_index = g.add_node(Data { x: 13 });
    let node_e_index = g.add_node(Data { x: 17 });
    let node_f_index = g.add_node(Data { x: 23 });

    // Link nodes together
    // Link root node to node a
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
    // Link node a to node b
    let res = g.add_edge(node_a_index, node_b_index);
    assert!(res.is_ok());
    // Link node b to node c
    let res = g.add_edge(node_b_index, node_c_index);
    assert!(res.is_ok());
    // Link node c to node e
    let res = g.add_edge(node_c_index, node_e_index);
    assert!(res.is_ok());
    // Link node a to node d
    let res = g.add_edge(node_a_index, node_d_index);
    assert!(res.is_ok());
    // Link node d to node e
    let res = g.add_edge(node_d_index, node_e_index);
    assert!(res.is_ok());
    // Link node e to node f
    let res = g.add_edge(node_e_index, node_f_index);
    assert!(res.is_ok());

    // Graph represented with the weight of each edge
    // Edges with '*' are part of the optimal path
    // a ----- b ----- c
    // | *     |       |
    // d       f       |
    // | *     | *     |
    // \------ e ------/

    let path = g
        .shortest_path(node_a_index, node_f_index)
        .expect("Failed to get Shortest path");
    assert_eq!(path.len(), 4);
    assert_eq!(
        path,
        vec![node_a_index, node_d_index, node_e_index, node_f_index]
    );

    println!("Shortest path: {:?}", path)
}

fn test_outgoing_edges() {
    println!("test_outgoing_edges");

    let mut g = ultragraph::with_capacity::<Data>(10);
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // Add some nodes to the graph
    let root_index = g.add_root_node(Data { x: 1 });
    let node_a_index = g.add_node(Data { x: 6 });
    let node_b_index = g.add_node(Data { x: 9 });

    // Link root node to node a
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    // check edge between root and node a
    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    // Link root node to node b
    let res = g.add_edge(root_index, node_b_index);
    assert!(res.is_ok());

    // check edge root to node b
    let expected = true;
    let actual = g.contains_edge(root_index, node_b_index);
    assert_eq!(expected, actual);

    // get all outgoing_edges of node a
    let neighbors = g.outgoing_edges(node_a_index).unwrap();

    // Node a has zero outgoing_edges
    assert_eq!(neighbors.len(), 0);

    // get all outgoing_edges of node b
    let neighbors = g.outgoing_edges(node_b_index).unwrap();

    // Node b has zero outgoing_edges
    assert_eq!(neighbors.len(), 0);

    // get all outgoing_edges of root node
    let neighbors = g.outgoing_edges(root_index).unwrap();

    // Root has 2 outgoing_edges: node a and node b
    assert_eq!(neighbors.len(), 2);

    // neighbors is just a vector of indices
    // so you can iterate over them to get the actual nodes
    println!("Neighbors of root node: ");
    for n in neighbors {
        let node = g.get_node(n).unwrap();
        println!("node: {:?}", node);
    }
}
