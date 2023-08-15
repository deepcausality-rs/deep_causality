// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}

fn get_ultra_graph() -> UltraGraph<StorageMatrixGraph<Data>, Data> {
    ultragraph::new_with_matrix_storage::<Data>(10)
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

#[test]
fn test_add_root_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_contains_root_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    g.add_root_node(d);
    assert!(g.contains_root_node());

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_root_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_root_index() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());

    let result = g.get_root_index().unwrap();
    assert_eq!(root_index, 0);
    assert_eq!(result, 0);
    assert_eq!(root_index, result);
    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_last_index() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = 0;
    let actual = g.get_last_index().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_add_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 1);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_contains_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 1);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 1);

    let data = g.get_node(1).unwrap();
    assert_eq!(*data, d);
    assert_eq!(data.x, d.x);
    assert_eq!(data.x, 42);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}


#[test]
fn test_remove_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 1);

    let data = g.get_node(1).unwrap();
    assert_eq!(*data, d);
    assert_eq!(data.x, d.x);
    assert_eq!(data.x, 42);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let result = g.remove_node(1);
    assert!(result.is_ok());
    assert!(!g.contains_node(1));

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_add_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
}

#[test]
fn test_add_edge_with_weight() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let res = g.add_edge_with_weight(root_index, node_a_index, 42);
    assert!(res.is_ok());
}

#[test]
fn test_contains_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);
}


#[test]
fn test_remove_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = true;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.remove_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);
}