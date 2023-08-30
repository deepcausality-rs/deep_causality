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
fn test_size() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.size();
    assert_eq!(expected, actual);
}

#[test]
fn test_is_empty() {
    let g = get_ultra_graph();

    let expected = true;
    let actual = g.is_empty();
    assert_eq!(expected, actual);
}

#[test]
fn test_number_nodes() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_all_nodes_empty() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.get_all_nodes().len();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_all_nodes() {
    let mut g = get_ultra_graph();

    let _ = g.add_root_node(Data { x: 3 });
    let _ = g.add_node(Data { x: 7 });
    let _ = g.add_node(Data { x: 9 });
    let _ = g.add_node(Data { x: 11 });

    let expected = 4;
    let actual = g.get_all_nodes().len();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_get_all_edges_empty() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.get_all_edges().len();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_get_all_edges() {
    let mut g = get_ultra_graph();

    let root_index = g.add_root_node(Data { x: 3 });
    let node_a_index = g.add_node(Data { x: 7 });
    let node_b_index = g.add_node(Data { x: 9 });
    let node_c_index = g.add_node(Data { x: 11 });

    let expected = 4;
    let actual = g.get_all_nodes().len();
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
    // Link node a to node b
    let res = g.add_edge(node_a_index, node_b_index);
    assert!(res.is_ok());
    // Link node root to node c
    let res = g.add_edge(root_index, node_c_index);
    assert!(res.is_ok());

    let expected = 3;
    let actual = g.get_all_edges().len();
    assert_eq!(expected, actual);
}

#[test]
fn test_number_edges() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.number_edges();
    assert_eq!(expected, actual);
}

#[test]
fn test_clear() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);
    assert!(!g.is_empty());

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    g.clear();

    assert!(g.is_empty());
    let expected = 0;

    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let actual = g.number_edges();
    assert_eq!(expected, actual);
}
