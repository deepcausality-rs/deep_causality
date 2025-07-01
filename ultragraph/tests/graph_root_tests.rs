/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    pub x: u8,
}

fn get_ultra_graph() -> UltraGraph<Data> {
    ultragraph::with_capacity::<Data>(10)
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

    let res = g.get_root_node();
    assert!(res.is_none());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());
    assert_eq!(root_index, 0);

    let res = g.get_root_node();
    assert!(res.is_some());

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

    let res = g.get_root_index();
    assert!(res.is_none());

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

    let res = g.get_last_index();
    assert!(res.is_err());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());
    assert_eq!(root_index, 0);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = 1;
    let actual = g.get_last_index().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_remove_root_node() {
    let mut g = get_ultra_graph();
    let root_index = g.add_root_node(Data { x: 1 });
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(root_index));

    // Remove the root node
    g.remove_node(root_index).unwrap();

    // Assert the root is now gone
    assert!(!g.contains_root_node());
    assert!(g.get_root_index().is_none());
    assert!(g.get_root_node().is_none());
}
