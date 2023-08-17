// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ultragraph::prelude::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
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

    let expected = 1;
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
fn test_add_edge_with_weight_err() {
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

    // When both nodes and the edges exist, then removal succeeds
    let res = g.remove_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    //
    // There are are three ways removing an edge can fail:
    // 1. Node a doesnt exist
    // 2. Node b doesnt exist
    // 3. Edge from node a to node b does not exists
    //

    // 1. Node a doesnt exist
    let res = g.remove_edge(42, node_a_index);
    assert!(res.is_err());

    // 2. Node b doesnt exist
    let res = g.remove_edge(root_index, 23);
    assert!(res.is_err());

    // 3.Edge from node a to node b does not exists
    let res = g.add_edge(23, 42);
    assert!(res.is_err());
}

#[test]
fn test_shortest_path() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    //
    // Add nodes to the graph
    //
    // Add root node
    let root_index = g.add_root_node(Data { x: 1 });
    assert_eq!(root_index, 0);
    // Add node a
    let node_a_index = g.add_node(Data { x: 6 });
    assert_eq!(node_a_index, 1);
    // Add node b
    let node_b_index = g.add_node(Data { x: 9 });
    assert_eq!(node_b_index, 2);
    // Add node c
    let node_c_index = g.add_node(Data { x: 11 });
    assert_eq!(node_c_index, 3);
    // Add node d
    let node_d_index = g.add_node(Data { x: 13 });
    assert_eq!(node_d_index, 4);
    // Add node e
    let node_e_index = g.add_node(Data { x: 17 });
    assert_eq!(node_e_index, 5);
    // Add node f
    let node_f_index = g.add_node(Data { x: 23 });
    assert_eq!(node_f_index, 6);

    //
    // Link nodes together
    //
    // Link root node to node a
    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(root_index, node_a_index));

    // Link node a to node b
    let res = g.add_edge(node_a_index, node_b_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_a_index, node_b_index));

    // Link node b to node c
    let res = g.add_edge(node_b_index, node_c_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_b_index, node_c_index));

    // Link node c to node e
    let res = g.add_edge(node_c_index, node_e_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_c_index, node_e_index));

    // Link node a to node d
    let res = g.add_edge(node_a_index, node_d_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_a_index, node_d_index));

    // Link node d to node e
    let res = g.add_edge(node_d_index, node_e_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_d_index, node_e_index));

    // Link node e to node f
    let res = g.add_edge(node_e_index, node_f_index);
    assert!(res.is_ok());
    assert!(g.contains_edge(node_e_index, node_f_index));

    // Graph represented with the weight of each edge
    // Edges with '*' are part of the optimal path
    // a ----- b ----- c
    // | *     |       |
    // d       f       |
    // | *     | *     |
    // \------ e ------/

    let path = g.shortest_path(node_a_index, node_f_index).expect("Failed to get Shortest path");
    assert_eq!(path.len(), 4);
    assert_eq!(path, vec![node_a_index, node_d_index, node_e_index, node_f_index]);
}

#[test]
fn test_shortest_path_error() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let root_index = g.add_root_node(Data { x: 1 });
    assert_eq!(root_index, 0);

    // There are two ways shortest path can fail:
    // 1. Node a doesnt exist
    // 2. Node b doesnt exist

    // 1. Node a doesnt exist
    let res = g.shortest_path(42, root_index);
    assert!(res.is_none());

    // 2. Node b doesnt exist
    let res = g.shortest_path(root_index, 23);
    assert!(res.is_none());
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
