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
    run_get_node_example();
    println!("--------------------");
    run_outgoing_edges_example();
    println!("--------------------");
    run_shortest_path_example();
    println!("--------------------");
    run_update_and_remove_example();
}

fn run_get_node_example() {
    println!("Running: Get Node Example");
    let mut g = ultragraph::with_capacity::<Data>(10);

    // Add nodes to the graph
    let root_index = g.add_root_node(Data { x: 3 });
    let node_a_index = g.add_node(Data { x: 7 });
    let node_b_index = g.add_node(Data { x: 9 });
    let node_c_index = g.add_node(Data { x: 11 });

    // Link nodes together
    g.add_edge(root_index, node_a_index).unwrap();
    g.add_edge(node_a_index, node_b_index).unwrap();
    g.add_edge(root_index, node_c_index).unwrap();

    // Get node a
    let node = g.get_node(node_a_index);
    assert!(node.is_some());

    let data = node.unwrap();
    assert_eq!(data.x, 7);
    println!("Retrieved Node A with data: {:?}", data);

    // get all outgoing_edges of root node
    let neighbors = g.outgoing_edges(root_index).unwrap();

    // root node has 2 outgoing_edges: node a and node c
    assert_eq!(neighbors.len(), 2);

    // neighbors is just a vector of indices
    // so you can iterate over them to get the actual nodes
    println!("Neighbors of root node: ");
    for n in neighbors {
        let node = g.get_node(n).unwrap();
        println!("- Neighbor node: {node:?}");
    }
}

fn run_shortest_path_example() {
    println!("Running: Shortest Path Example");

    let mut g = ultragraph::with_capacity::<Data>(10);

    // Add nodes to the graph
    let root_index = g.add_root_node(Data { x: 1 });
    let node_a_index = g.add_node(Data { x: 6 });
    let node_b_index = g.add_node(Data { x: 9 });
    let node_c_index = g.add_node(Data { x: 11 });
    let node_d_index = g.add_node(Data { x: 13 });
    let node_e_index = g.add_node(Data { x: 17 });
    let node_f_index = g.add_node(Data { x: 23 });

    // Link nodes together
    g.add_edge(root_index, node_a_index).unwrap();
    g.add_edge(node_a_index, node_b_index).unwrap();
    g.add_edge(node_b_index, node_c_index).unwrap();
    g.add_edge(node_c_index, node_e_index).unwrap();
    g.add_edge(node_a_index, node_d_index).unwrap();
    g.add_edge(node_d_index, node_e_index).unwrap();
    g.add_edge(node_e_index, node_f_index).unwrap();

    let path = g
        .shortest_path(node_a_index, node_f_index)
        .expect("Failed to get Shortest path");
    assert_eq!(path.len(), 4);
    assert_eq!(
        path,
        vec![node_a_index, node_d_index, node_e_index, node_f_index]
    );

    println!("Shortest path from Node A to Node F: {path:?}")
}

fn run_outgoing_edges_example() {
    println!("Running: Outgoing Edges Example");

    let mut g = ultragraph::with_capacity::<Data>(10);

    // Add some nodes to the graph
    let root_index = g.add_root_node(Data { x: 1 });
    let node_a_index = g.add_node(Data { x: 6 });
    let node_b_index = g.add_node(Data { x: 9 });

    // Link root node to node a
    g.add_edge(root_index, node_a_index).unwrap();
    // Link root node to node b
    g.add_edge(root_index, node_b_index).unwrap();

    // get all outgoing_edges of root node
    let neighbors: Vec<usize> = g.outgoing_edges(root_index).unwrap().collect();
    println!("Neighbors of root node: {:?}", neighbors);
    assert_eq!(neighbors.len(), 2);
}

fn run_update_and_remove_example() {
    println!("Running: Update and Remove Example");
    let mut g = ultragraph::with_capacity::<Data>(10);

    let node_a = g.add_node(Data { x: 10 });
    let node_b = g.add_node(Data { x: 20 });
    g.add_edge(node_a, node_b).unwrap();
    println!("Initial graph with edge from {} -> {}", node_a, node_b);

    // Update node A's data
    println!("Updating Node A's data from 10 to 99...");
    g.update_node(node_a, Data { x: 99 }).unwrap();
    let updated_node_a = g.get_node(node_a).unwrap();
    println!("Retrieved updated Node A: {:?}", updated_node_a);
    assert_eq!(updated_node_a.x, 99);

    // The edge is preserved after the update
    let edge_exists = g.contains_edge(node_a, node_b);
    println!("Edge from A to B still exists after update: {}", edge_exists);
    assert!(edge_exists);

    // Remove the edge
    println!("Removing edge from A to B...");
    g.remove_edge(node_a, node_b).unwrap();
    let edge_exists_after_remove = g.contains_edge(node_a, node_b);
    println!("Edge from A to B exists after removal: {}", edge_exists_after_remove);
    assert!(!edge_exists_after_remove);

    // The nodes are preserved after edge removal
    let node_a_exists = g.contains_node(node_a);
    println!("Node A still exists after edge removal: {}", node_a_exists);
    assert!(node_a_exists);

    // Remove a node
    println!("Removing Node B...");
    g.remove_node(node_b).unwrap();
    let node_b_exists = g.contains_node(node_b);
    println!("Node B exists after removal: {}", node_b_exists);
    assert!(!node_b_exists);
}