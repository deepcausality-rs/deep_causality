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
