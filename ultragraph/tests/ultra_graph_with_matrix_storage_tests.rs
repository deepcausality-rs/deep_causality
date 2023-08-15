// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}

fn get_ultra_graph() -> UltraGraph<StorageMatrixGraph<Data>, Data> {
    ultragraph::new_with_matrix_storage::<Data>(100)
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



