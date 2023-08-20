// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    pub x: u8,
}

fn get_ultra_graph() -> UltraGraph<Data> {
    ultragraph::with_capacity::<Data>(10)
}

#[test]
fn test_outgoing_edges() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // Add root node
    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    // Add node a
    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    // Link root node to node a
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    // check edge between root and node a
    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    // Add node b
    let d = Data { x: 42 };
    let node_b_index = g.add_node(d);
    assert_eq!(node_b_index, 2);

    // Link root node to node b
    let res = g.add_edge(root_index, node_b_index);
    assert!(res.is_ok());

    // check edge root to node b
    let expected = true;
    let actual = g.contains_edge(root_index, node_b_index);
    assert_eq!(expected, actual);

    // get all outgoing_edges of root node
    let neighbors = g.outgoing_edges(root_index).unwrap();

    // Root has 2 outgoing_edges: node a and node b
    let expected = 2;
    let actual = neighbors.len();
    assert_eq!(expected, actual);

    // get all outgoing_edges of node a
    let neighbors = g.outgoing_edges(node_a_index).unwrap();

    // Node a has zero outgoing_edges
    let expected = 0;
    let actual = neighbors.len();
    assert_eq!(expected, actual);

    // get all outgoing_edges of node b
    let neighbors = g.outgoing_edges(node_b_index).unwrap();

    // Node b has zero outgoing_edges
    let expected = 0;
    let actual = neighbors.len();
    assert_eq!(expected, actual);
}

#[test]
fn test_outgoing_edges_error() {
    let g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // There is really only one way outgoing_edges can fail:
    // when the node does not exist
    let res = g.outgoing_edges(42);
    assert!(res.is_err());
}
