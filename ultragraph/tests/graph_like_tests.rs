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

    let node = g.get_node(1);
    assert!(node.is_some());

    let data = node.unwrap();
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
fn test_get_node_error() {
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

    // Ensure node doesn't exist
    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    // Try to get a node that doesn't exist
    let node = g.get_node(1);
    assert!(node.is_none());
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
fn test_remove_node_error() {
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

    let result = g.remove_node(1);
    assert!(result.is_err());
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
fn test_add_edge_error() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // There are are three ways adding an edge can fail:
    // 1. Node a doesnt exist
    // 2. Node b doesnt exist
    // 3.An edge from node a to node b already exists

    // 1. Node a doesnt exist
    let res = g.add_edge(23, node_a_index);
    assert!(res.is_err());

    // 2. Node b doesnt exist
    let res = g.add_edge(root_index, 23);
    assert!(res.is_err());

    // 3.An edge from node a to node b already exists
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_err());
}

#[test]
fn test_add_edge_with_weight_ok() {
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
fn test_add_edge_with_weight_error() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 0);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 1);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    // There are are three ways adding an edge can fail:
    // 1. Node a doesnt exist
    // 2. Node b doesnt exist
    // 3.An edge from node a to node b already exists

    let weight = 42;

    // 1. Node a doesnt exist
    let res = g.add_edge_with_weight(23, node_a_index, weight);
    assert!(res.is_err());

    // 2. Node b doesnt exist
    let res = g.add_edge_with_weight(root_index, 23, weight);
    assert!(res.is_err());

    // 3.An edge from node a to node b already exists
    let res = g.add_edge_with_weight(root_index, node_a_index, weight);
    assert!(res.is_ok());

    let res = g.add_edge_with_weight(root_index, node_a_index, weight);
    assert!(res.is_err());
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

#[test]
fn test_remove_edge_error() {
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

    //
    // There are are three ways removing an edge can fail:
    // 1. Node a doesnt exist
    // 2. Node b doesnt exist
    // 3. Edge from node a to node b does not exists
    //

    // 1. Node a doesnt exist
    let res = g.remove_edge(42, root_index);
    assert!(res.is_err());

    // 2. Node b doesnt exist
    let res = g.remove_edge(root_index, 500);
    assert!(res.is_err());

    // 3.Edge from node a to node b does not exists
    let res = g.remove_edge(root_index, root_index);
    assert!(res.is_err());
}

#[test]
fn test_update_node_succeeds_and_preserves_edges() {
    let mut g = get_ultra_graph();
    let node_a = g.add_node(Data { x: 10 });
    let node_b = g.add_node(Data { x: 20 });
    g.add_edge(node_a, node_b).unwrap();

    // Update node A
    let update_res = g.update_node(node_a, Data { x: 99 });
    assert!(update_res.is_ok());

    // Verify the node data changed
    let updated_node_a = g.get_node(node_a).unwrap();
    assert_eq!(updated_node_a.x, 99);

    // CRITICAL: Verify the edge still exists
    assert!(g.contains_edge(node_a, node_b));
}

#[test]
fn test_update_node_fails_on_nonexistent_index() {
    let mut g = get_ultra_graph();
    let update_res = g.update_node(999, Data { x: 1 });
    assert!(update_res.is_err());
}

#[test]
fn test_remove_edge_preserves_nodes() {
    let mut g = get_ultra_graph();
    let node_a = g.add_node(Data { x: 1 });
    let node_b = g.add_node(Data { x: 2 });
    g.add_edge(node_a, node_b).unwrap();

    assert!(g.contains_edge(node_a, node_b));

    // Remove the edge
    let res = g.remove_edge(node_a, node_b);
    assert!(res.is_ok());

    // Assert the edge is gone, but the nodes are NOT.
    assert!(!g.contains_edge(node_a, node_b));
    assert!(g.contains_node(node_a)); // CRITICAL CHECK
    assert!(g.contains_node(node_b)); // CRITICAL CHECK

    // Verify we can still retrieve the node data
    assert_eq!(g.get_node(node_a).unwrap().x, 1);
    assert_eq!(g.get_node(node_b).unwrap().x, 2);
}
