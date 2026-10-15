/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{Rng, Xoshiro256};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt::Debug;
use ultragraph::*;

// Freezable Trait Tests
#[test]
fn test_freeze_empty_graph() {
    let mut g: UltraGraphContainer<usize, _> = UltraGraph::new();
    g.freeze();

    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(g.is_empty());
    assert_eq!(g.get_last_index(), None);
    assert!(!g.contains_root_node());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_graph_with_nodes_no_edges() {
    let mut g: UltraGraph<String> = UltraGraph::with_capacity(10, None);
    g.add_node("A".to_string()).expect("Failed to add node");
    g.add_node("B".to_string()).expect("Failed to add node");
    g.add_root_node("C".to_string())
        .expect("Failed to add node");

    g.freeze();

    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.number_edges(), 0);
    assert!(g.contains_node(0));
    assert!(g.contains_node(1));
    assert!(g.contains_node(2));
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), Some(&"B".to_string()));
    assert_eq!(g.get_node(2), Some(&"C".to_string()));
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_node(), Some(&"C".to_string()));
    assert_eq!(g.get_root_index(), Some(2));
}

#[test]
fn test_freeze_graph_with_nodes_and_edges() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);

    let n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let n1 = g.add_node("B".to_string()).expect("Failed to add node");
    let n2 = g.add_node("C".to_string()).expect("Failed to add node");
    let n3 = g.add_node("D".to_string()).expect("Failed to add node");

    g.add_edge(n0, n1, 10).unwrap();
    g.add_edge(n0, n2, 20).unwrap();
    g.add_edge(n1, n2, 30).unwrap();
    g.add_edge(n2, n3, 40).unwrap();
    g.add_edge(n0, n1, 15).unwrap(); // Parallel edge

    g.freeze();

    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.number_edges(), 5);

    assert!(g.contains_edge(n0, n1));
    assert!(g.contains_edge(n0, n2));
    assert!(g.contains_edge(n1, n2));
    assert!(g.contains_edge(n2, n3));

    // Check parallel edge
    let edges_from_n0 = g.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3); // (n0,n1,10), (n0,n2,20), (n0,n1,15)
    let mut targets_from_n0: Vec<usize> = edges_from_n0.iter().map(|(t, _)| *t).collect();
    targets_from_n0.sort_unstable();
    assert_eq!(targets_from_n0, vec![n1, n1, n2]);

    // Check specific edge weights (requires iterating, as `contains_edge` doesn't check weight)
    let has_edge_0_1_10 = g
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &10);
    let has_edge_0_1_15 = g
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &15);
    assert!(has_edge_0_1_10);
    assert!(has_edge_0_1_15);
}

#[test]
fn test_freeze_graph_with_tombstoned_nodes() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);
    let n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let n1 = g.add_node("B".to_string()).expect("Failed to add node"); // Will be tombstoned
    let n2 = g.add_node("C".to_string()).expect("Failed to add node");
    let n3 = g.add_node("D".to_string()).expect("Failed to add node");

    g.add_edge(n0, n1, 10).unwrap(); // Edge to tombstoned node
    g.add_edge(n0, n2, 20).unwrap();
    g.add_edge(n2, n3, 30).unwrap();
    g.add_edge(n1, n3, 40).unwrap(); // Edge from tombstoned node

    let root_idx = g
        .add_root_node("ROOT".to_string())
        .expect("Failed to add node");
    assert_eq!(root_idx, 4);

    g.remove_node(n1).unwrap(); // Tombstone n1 using the public API
    g.update_node(root_idx, "NEW_ROOT".to_string()).unwrap(); // Update root node

    g.freeze();

    // Live: A (0), C (2), D (3), NEW_ROOT (4). Position 1 is a tombstone and stays one.
    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.number_edges(), 2); // (0,2) and (2,3); both edges touching position 1 are gone
    assert_eq!(g.get_all_nodes().len(), 4);
    assert_eq!(g.get_last_index(), Some(4));

    assert!(g.contains_node(0));
    assert!(!g.contains_node(1));
    assert!(g.contains_node(2));
    assert!(g.contains_node(3));
    assert!(g.contains_node(4));
    assert!(!g.contains_node(5));
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), None);
    assert_eq!(g.get_node(2), Some(&"C".to_string()));
    assert_eq!(g.get_node(3), Some(&"D".to_string()));
    assert_eq!(g.get_node(4), Some(&"NEW_ROOT".to_string()));

    assert!(!g.contains_edge(0, 1));
    assert!(g.contains_edge(0, 2));
    assert!(g.contains_edge(2, 3));
    assert!(!g.contains_edge(1, 3));

    assert!(g.contains_root_node());
    assert_eq!(g.get_root_node(), Some(&"NEW_ROOT".to_string()));
    assert_eq!(g.get_root_index(), Some(4));

    // The table keeps five slots through the freeze: the next position is 5.
    g.unfreeze();
    assert_eq!(g.add_node("E".to_string()).unwrap(), 5);
}

#[test]
fn test_freeze_graph_with_tombstoned_root_node() {
    let mut g = UltraGraph::with_capacity(10, None);
    let _n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let _n1 = g.add_node("B".to_string()).expect("Failed to add node");
    let root_idx = g
        .add_root_node("ROOT".to_string())
        .expect("Failed to add node");
    assert_eq!(root_idx, 2);

    g.remove_node(root_idx).unwrap(); // Tombstone the root node

    g.freeze();

    assert_eq!(g.number_nodes(), 2);
    assert!(!g.is_empty());
    assert_eq!(g.get_last_index(), Some(1)); // the highest live position; slot 2 is a tombstone
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), Some(&"B".to_string()));
    assert!(!g.contains_node(2));
    assert!(!g.contains_root_node()); // Root should be gone
    assert_eq!(g.get_root_index(), None);
    assert_eq!(g.get_root_node(), None);

    // The trailing tombstone keeps its slot: the next position is 3.
    g.unfreeze();
    assert_eq!(g.add_node("C".to_string()).unwrap(), 3);
}

#[test]
fn test_freeze_graph_with_all_nodes_tombstoned() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);
    let n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let n1 = g.add_node("B".to_string()).expect("Failed to add node");
    g.add_edge(n0, n1, 10).unwrap();

    g.remove_node(n0).unwrap();
    g.remove_node(n1).unwrap();
    g.freeze();

    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(g.is_empty());
    assert_eq!(g.get_last_index(), None);
    assert!(!g.contains_root_node());

    // Both slots survive the freeze of a graph with no live node: the next position is 2.
    g.unfreeze();
    assert_eq!(g.add_node("C".to_string()).unwrap(), 2);
}

#[test]
fn test_freeze_with_super_node_triggers_radix_sort() {
    // This constant must be greater than the `RADIX_SORT_THRESHOLD` (128)
    // in `graph_freeze.rs` to ensure the Radix Sort path is triggered.
    const NUM_SPOKES: usize = 250;
    let mut graph = UltraGraph::with_capacity(NUM_SPOKES, Some(NUM_SPOKES));

    let hub_node = graph.add_node(()).expect("Failed to add node");
    let mut spoke_nodes = Vec::new();

    // Add all the spoke nodes first.
    for _ in 0..NUM_SPOKES {
        // We use a payload to make debugging easier if the test fails.
        let node_idx = graph.add_node(()).expect("Failed to add node");
        spoke_nodes.push(node_idx);
    }

    // Add edges from the hub to the spokes in a non-sorted order (reverse)
    // to ensure the sorting logic is robustly exercised.
    for &spoke_node in spoke_nodes.iter().rev() {
        graph.add_edge(hub_node, spoke_node, ()).unwrap();
    }

    // Freeze the graph. This is the operation under test.
    // It should trigger the Radix Sort path for the hub node's adjacency list.
    graph.freeze();

    // --- Verification ---

    // 1. Basic structural verification.
    assert_eq!(graph.number_nodes(), NUM_SPOKES + 1);
    assert_eq!(graph.number_edges(), NUM_SPOKES);

    // 2. Verify the sorting of the super-node's adjacency list.
    //    `get_edges` on a CsmGraph returns edges based on the sorted CSR data.
    let edges = graph.get_edges(hub_node).unwrap();
    assert_eq!(edges.len(), NUM_SPOKES);

    // Extract just the target node indices from the returned edges.
    let target_indices: Vec<usize> = edges.iter().map(|(target, _)| *target).collect();

    // Create the expected, correctly sorted list of spoke node indices.
    let mut expected_sorted_spokes = spoke_nodes;
    expected_sorted_spokes.sort_unstable();

    // 3. The final, critical assertion.
    //    This confirms that the adjacency list was correctly sorted during the freeze.
    assert_eq!(
        target_indices, expected_sorted_spokes,
        "Adjacency list for the super-node was not sorted correctly during freeze."
    );
}

#[test]
fn test_unfreeze_empty_graph() {
    let mut g: UltraGraphContainer<usize, _> = UltraGraph::new();
    g.freeze();

    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(g.is_empty());
    assert_eq!(g.get_last_index(), None);
    assert!(!g.contains_root_node());
    assert!(g.is_frozen());

    g.unfreeze();

    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(g.is_empty());
    assert_eq!(g.get_last_index(), None);
    assert!(!g.contains_root_node());
    assert!(!g.is_frozen());
}

#[test]
fn test_unfreeze_graph_with_nodes_no_edges() {
    let mut g: UltraGraphContainer<String, _> = UltraGraph::new();
    g.add_node("A".to_string()).expect("Failed to add node");
    g.add_node("B".to_string()).expect("Failed to add node");
    g.add_root_node("C".to_string())
        .expect("Failed to add node");

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.number_edges(), 0);
    assert!(g.contains_node(0));
    assert!(g.contains_node(1));
    assert!(g.contains_node(2));
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), Some(&"B".to_string()));
    assert_eq!(g.get_node(2), Some(&"C".to_string()));
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_node(), Some(&"C".to_string()));
    assert_eq!(g.get_root_index(), Some(2));
}

#[test]
fn test_unfreeze_graph_with_nodes_and_edges() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);
    let n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let n1 = g.add_node("B".to_string()).expect("Failed to add node");
    let n2 = g.add_node("C".to_string()).expect("Failed to add node");
    let n3 = g.add_node("D".to_string()).expect("Failed to add node");

    g.add_edge(n0, n1, 10).expect("Failed to add edge");
    g.add_edge(n0, n2, 20).expect("Failed to add edge");
    g.add_edge(n1, n3, 30).expect("Failed to add edge");
    g.add_edge(n2, n3, 40).expect("Failed to add edge");
    g.add_edge(n0, n1, 15).expect("Failed to add edge"); // Parallel edge

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.number_edges(), 5);

    assert!(g.contains_edge(n0, n1));
    assert!(g.contains_edge(n0, n2));
    assert!(g.contains_edge(n1, n3));
    assert!(g.contains_edge(n2, n3));

    // Check parallel edge
    let edges_from_n0 = g.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3); // (n0,n1,10), (n0,n2,20), (n0,n1,15)
    let mut targets_from_n0: Vec<usize> = edges_from_n0.iter().map(|(t, _)| *t).collect();
    targets_from_n0.sort_unstable();
    assert_eq!(targets_from_n0, vec![n1, n1, n2]);

    // Check specific edge weights (requires iterating, as `contains_edge` doesn't check weight)
    let has_edge_0_1_10 = g
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &10);
    let has_edge_0_1_15 = g
        .get_edges(n0)
        .unwrap()
        .iter()
        .any(|&(t, w)| t == n1 && w == &15);
    assert!(has_edge_0_1_10);
    assert!(has_edge_0_1_15);
}

#[test]
fn test_unfreeze_graph_with_tombstoned_nodes_from_original() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);
    let n0 = g.add_node("A".to_string()).expect("Failed to add node");
    let n1 = g.add_node("B".to_string()).expect("Failed to add node");
    let n2 = g.add_node("C".to_string()).expect("Failed to add node");
    let n3 = g.add_node("D".to_string()).expect("Failed to add node");

    g.add_edge(n0, n1, 10).expect("Failed to add edge");
    g.add_edge(n0, n2, 20).expect("Failed to add edge");
    g.add_edge(n2, n3, 30).expect("Failed to add edge");
    g.add_edge(n1, n3, 40).expect("Failed to add edge");

    let root_idx = g
        .add_root_node("ROOT".to_string())
        .expect("Failed to add root node");
    assert_eq!(root_idx, 4);

    g.remove_node(n1).unwrap();
    g.update_node(root_idx, "NEW_ROOT".to_string()).unwrap();

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    // Live: A (0), C (2), D (3), NEW_ROOT (4). Position 1 is a tombstone in both states.
    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.get_all_nodes().len(), 4);
    assert_eq!(g.get_last_index(), Some(4));

    assert!(g.contains_node(0));
    assert!(!g.contains_node(1));
    assert!(g.contains_node(2));
    assert!(g.contains_node(3));
    assert!(g.contains_node(4));
    assert!(!g.contains_node(5));
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), None);
    assert_eq!(g.get_node(2), Some(&"C".to_string()));
    assert_eq!(g.get_node(3), Some(&"D".to_string()));
    assert_eq!(g.get_node(4), Some(&"NEW_ROOT".to_string()));

    assert!(!g.contains_edge(0, 1));
    assert!(g.contains_edge(0, 2));
    assert!(g.contains_edge(2, 3));
    assert!(!g.contains_edge(1, 3));

    assert!(g.contains_root_node());
    assert_eq!(g.get_root_node(), Some(&"NEW_ROOT".to_string()));
    assert_eq!(g.get_root_index(), Some(4));

    // Five slots survived the round trip: the next position is 5.
    assert_eq!(g.add_node("E".to_string()).unwrap(), 5);
}

#[test]
fn test_unfreeze_single_node_no_edges() {
    let mut g: UltraGraphContainer<String, _> = UltraGraph::new();
    g.add_node("Single".to_string())
        .expect("Failed to add node");

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 1);
    assert_eq!(g.number_edges(), 0);
    assert!(g.contains_node(0));
    assert_eq!(g.get_node(0), Some(&"Single".to_string()));
    assert!(!g.contains_root_node());
}

#[test]
fn test_unfreeze_single_node_with_self_loop() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);

    let n0 = g.add_node("Loop".to_string()).expect("Failed to add node");
    g.add_edge(n0, n0, 100).unwrap();

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 1);
    assert_eq!(g.number_edges(), 1);
    assert!(g.contains_node(0));
    assert_eq!(g.get_node(0), Some(&"Loop".to_string()));
    assert!(g.contains_edge(n0, n0));
    assert_eq!(g.get_edges(n0).unwrap().len(), 1);
    assert_eq!(g.get_edges(n0).unwrap()[0], (n0, &100));
}

#[test]
fn test_unfreeze_graph_with_root_not_last_node() {
    let mut g: UltraGraphContainer<String, _> = UltraGraph::new();
    g.add_node("A".to_string()).expect("Failed to add node");
    g.add_root_node("ROOT".to_string())
        .expect("Failed to add root node"); // Root is at index 1
    g.add_node("B".to_string()).expect("Failed to add node");

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(1), Some(&"ROOT".to_string()));
    assert_eq!(g.get_node(2), Some(&"B".to_string()));
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(1));
    assert_eq!(g.get_root_node(), Some(&"ROOT".to_string()));
}

#[test]
fn test_unfreeze_graph_with_complex_edges() {
    let mut g = UltraGraphWeighted::with_capacity(10, None);
    let n0 = g.add_node("Node0".to_string()).expect("Failed to add node");
    let n1 = g.add_node("Node1".to_string()).expect("Failed to add node");
    let n2 = g.add_node("Node2".to_string()).expect("Failed to add node");
    let n3 = g.add_node("Node3".to_string()).expect("Failed to add node");

    g.add_edge(n0, n1, 1).unwrap();
    g.add_edge(n0, n2, 2).unwrap();
    g.add_edge(n1, n2, 3).unwrap();
    g.add_edge(n2, n3, 4).unwrap();
    g.add_edge(n0, n1, 5).unwrap(); // Parallel edge

    g.freeze();
    assert!(g.is_frozen());

    g.unfreeze();
    assert!(!g.is_frozen());

    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.number_edges(), 5);

    // Verify edges from n0
    let edges_from_n0 = g.get_edges(n0).unwrap();
    assert_eq!(edges_from_n0.len(), 3);
    let mut expected_n0_edges = vec![(n1, &1), (n2, &2), (n1, &5)];
    expected_n0_edges.sort_unstable();
    let mut actual_n0_edges = edges_from_n0
        .iter()
        .map(|&(t, w)| (t, w))
        .collect::<Vec<_>>();
    actual_n0_edges.sort_unstable();
    assert_eq!(actual_n0_edges, expected_n0_edges);

    // Verify edges from n1
    let edges_from_n1 = g.get_edges(n1).unwrap();
    assert_eq!(edges_from_n1.len(), 1);
    assert_eq!(edges_from_n1[0], (n2, &3));

    // Verify edges from n2
    let edges_from_n2 = g.get_edges(n2).unwrap();
    assert_eq!(edges_from_n2.len(), 1);
    assert_eq!(edges_from_n2[0], (n3, &4));

    // Verify edges from n3 (no outgoing edges)
    assert!(g.get_edges(n3).unwrap().is_empty());
}

// --- Permanent positions (spec: ultragraph-stable-node-ids) ---

/// Nodes A, B, C and D at positions 0 to 3.
fn four_nodes() -> UltraGraph<&'static str> {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g
}

#[test]
fn test_freezing_after_a_removal_moves_no_node() {
    let mut g = four_nodes();
    g.remove_node(1).unwrap();

    g.freeze();

    assert_eq!(g.get_node(0), Some(&"A"));
    assert!(!g.contains_node(1));
    assert_eq!(g.get_node(1), None);
    assert_eq!(g.get_node(2), Some(&"C"));
    assert_eq!(g.get_node(3), Some(&"D"));
    assert_eq!(g.number_nodes(), 3);
}

#[test]
fn test_unfreezing_keeps_every_position() {
    let mut g = four_nodes();
    g.remove_node(1).unwrap();
    g.freeze();

    g.unfreeze();

    assert_eq!(g.get_node(0), Some(&"A"));
    assert!(!g.contains_node(1));
    assert_eq!(g.get_node(1), None);
    assert_eq!(g.get_node(2), Some(&"C"));
    assert_eq!(g.get_node(3), Some(&"D"));
    assert_eq!(g.number_nodes(), 3);
}

/// Row C: with no removal, position equals rank and the old renumbering is invisible. This test
/// passes under both behaviours; `test_freezing_after_a_removal_moves_no_node` is the one that
/// tells them apart.
#[test]
fn test_freeze_without_removal_keeps_positions() {
    let mut g = four_nodes();
    g.freeze();

    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.get_node(position), Some(&payload));
    }
    assert_eq!(g.number_nodes(), 4);
    assert_eq!(g.get_last_index(), Some(3));

    g.unfreeze();
    assert_eq!(g.add_node("E").unwrap(), 4);
}

#[test]
fn test_round_trips_keep_trailing_tombstones() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D", "E"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.remove_node(3).unwrap();
    g.remove_node(4).unwrap();

    g.freeze();
    g.unfreeze();
    g.freeze();

    assert_eq!(g.get_node(0), Some(&"A"));
    assert_eq!(g.get_node(1), Some(&"B"));
    assert_eq!(g.get_node(2), Some(&"C"));
    assert!(!g.contains_node(3));
    assert!(!g.contains_node(4));
    assert_eq!(g.get_node(3), None);
    assert_eq!(g.get_node(4), None);
    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.get_last_index(), Some(2));

    // The two trailing slots survive both freezes: the next position is 5, not 3.
    g.unfreeze();
    assert_eq!(g.add_node("F").unwrap(), 5);
}

#[test]
fn test_the_root_keeps_its_position() {
    let mut g = four_nodes();
    assert_eq!(g.add_root_node("ROOT").unwrap(), 4);
    g.remove_node(1).unwrap();

    g.freeze();

    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(4));
    assert_eq!(g.get_root_node(), Some(&"ROOT"));
    assert_eq!(g.get_last_index(), Some(4));
}

/// Row B: one node, removed and the graph frozen; one node kept through a round trip.
#[test]
fn test_freeze_keeps_a_single_removed_slot() {
    let mut g = UltraGraph::new();
    assert_eq!(g.add_node("A").unwrap(), 0);
    g.remove_node(0).unwrap();

    g.freeze();

    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert!(!g.contains_node(0));
    assert_eq!(g.get_node(0), None);
    assert_eq!(g.get_last_index(), None);

    // The removed slot survives the freeze: the next position is 1, not 0.
    g.unfreeze();
    assert_eq!(g.add_node("B").unwrap(), 1);

    // One node kept through a round trip.
    g.freeze();
    g.unfreeze();
    assert_eq!(g.get_node(1), Some(&"B"));
    assert!(!g.contains_node(0));
    assert_eq!(g.add_node("C").unwrap(), 2);
}

#[test]
fn test_a_graph_with_no_live_node_keeps_its_table() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    for position in 0..3 {
        g.remove_node(position).unwrap();
    }

    g.freeze();

    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.get_last_index(), None);
    for position in 0..3 {
        assert!(!g.contains_node(position));
    }

    g.unfreeze();

    // Three slots survived: the next position is 3.
    assert_eq!(g.add_node("D").unwrap(), 3);
}

// --- Model-based test: seeded operation sequences against a reference model ---

/// The reference model: positions to payloads, the edges between live positions with their
/// weights, the root, and the state. Positions are permanent until `compact` or `clear`.
struct Model<W> {
    slots: Vec<Option<u64>>,
    edges: BTreeMap<(usize, usize), W>,
    root: Option<usize>,
    frozen: bool,
    next_payload: u64,
}

impl<W: Copy> Model<W> {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            edges: BTreeMap::new(),
            root: None,
            frozen: false,
            next_payload: 0,
        }
    }

    fn live(&self) -> Vec<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(position, slot)| slot.map(|_| position))
            .collect()
    }

    fn is_live(&self, position: usize) -> bool {
        self.slots.get(position).is_some_and(Option::is_some)
    }

    /// A fresh payload at the next slot; no position is ever issued twice before `clear`.
    fn add_node(&mut self) -> (usize, u64) {
        let position = self.slots.len();
        let payload = self.next_payload;
        self.next_payload += 1;
        self.slots.push(Some(payload));
        (position, payload)
    }

    fn remove_node(&mut self, position: usize) {
        self.slots[position] = None;
        self.edges
            .retain(|&(source, target), _| source != position && target != position);
        if self.root == Some(position) {
            self.root = None;
        }
    }

    /// Renumbers the live nodes in order and returns the old-to-new table.
    fn compact(&mut self) -> Vec<Option<usize>> {
        let mut table = Vec::with_capacity(self.slots.len());
        let mut compacted = Vec::new();
        for slot in &self.slots {
            match slot {
                Some(payload) => {
                    table.push(Some(compacted.len()));
                    compacted.push(Some(*payload));
                }
                None => table.push(None),
            }
        }
        self.edges = self
            .edges
            .iter()
            .map(|(&(source, target), &weight)| {
                ((table[source].unwrap(), table[target].unwrap()), weight)
            })
            .collect();
        self.root = self.root.map(|root| table[root].unwrap());
        self.slots = compacted;
        table
    }

    fn clear(&mut self) {
        self.slots.clear();
        self.edges.clear();
        self.root = None;
        self.frozen = false;
    }
}

/// Every read of the graph against the model, for every position up to one past the table.
fn check_against_model<W>(g: &UltraGraphContainer<u64, W>, m: &Model<W>, step: usize)
where
    W: Clone + Default + EdgeKey + Copy + PartialEq + Debug,
{
    assert_eq!(g.is_frozen(), m.frozen, "step {step}: is_frozen");

    let live = m.live();
    assert_eq!(g.number_nodes(), live.len(), "step {step}: number_nodes");
    assert_eq!(g.is_empty(), live.is_empty(), "step {step}: is_empty");
    assert_eq!(
        g.get_last_index(),
        live.last().copied(),
        "step {step}: get_last_index"
    );

    assert_eq!(g.get_root_index(), m.root, "step {step}: get_root_index");
    assert_eq!(
        g.contains_root_node(),
        m.root.is_some(),
        "step {step}: contains_root_node"
    );
    assert_eq!(
        g.get_root_node().copied(),
        m.root.map(|root| m.slots[root].unwrap()),
        "step {step}: get_root_node"
    );

    let mut all: Vec<u64> = g.get_all_nodes().into_iter().copied().collect();
    all.sort_unstable();
    let mut expected_all: Vec<u64> = live.iter().map(|&p| m.slots[p].unwrap()).collect();
    expected_all.sort_unstable();
    assert_eq!(all, expected_all, "step {step}: get_all_nodes");

    let bound = m.slots.len() + 1; // one past the end reads as absent
    for p in 0..bound {
        let expected_payload = m.slots.get(p).copied().flatten();
        assert_eq!(
            g.contains_node(p),
            expected_payload.is_some(),
            "step {step}: contains_node({p})"
        );
        assert_eq!(
            g.get_node(p).copied(),
            expected_payload,
            "step {step}: get_node({p})"
        );

        match (g.get_edges(p), expected_payload) {
            (None, None) => {}
            (Some(edges), Some(_)) => {
                let mut got: Vec<(usize, W)> = edges.into_iter().map(|(t, w)| (t, *w)).collect();
                got.sort_by_key(|(t, _)| *t);
                let want: Vec<(usize, W)> = m
                    .edges
                    .range((p, 0)..(p + 1, 0))
                    .map(|(&(_, t), &w)| (t, w))
                    .collect();
                assert_eq!(got, want, "step {step}: get_edges({p})");
            }
            (got, _) => panic!("step {step}: get_edges({p}) is {got:?}, live {expected_payload:?}"),
        }

        for q in 0..bound {
            assert_eq!(
                g.contains_edge(p, q),
                m.edges.contains_key(&(p, q)),
                "step {step}: contains_edge({p}, {q})"
            );
        }
    }
}

/// Drives one seeded sequence of `add_node`, `add_root_node`, `remove_node`, `add_edge`,
/// `freeze`, `unfreeze`, `compact_ids` and `clear` against the model, checking every read after
/// every step. Draws one past the table so every mutation also meets an absent position.
fn drive_model<W, F>(seed: u64, steps: usize, mut weight: F)
where
    W: Clone + Default + EdgeKey + Copy + PartialEq + Debug,
    F: FnMut(&mut Xoshiro256) -> W,
{
    let mut rng = Xoshiro256::from_seed(seed);
    let mut g: UltraGraphContainer<u64, W> = UltraGraphContainer::new();
    let mut m: Model<W> = Model::new();

    for step in 0..steps {
        let roll = rng.random_range(0..100usize);
        let bound = m.slots.len() + 1;

        if m.frozen {
            match roll {
                0..=69 => {
                    g.unfreeze();
                    m.frozen = false;
                }
                70..=84 => assert_eq!(
                    g.compact_ids(),
                    Err(GraphError::GraphIsFrozen),
                    "step {step}: compact_ids while frozen"
                ),
                85..=94 => {
                    let p = rng.random_range(0..bound);
                    assert_eq!(g.remove_node(p), Err(GraphError::GraphIsFrozen));
                    assert_eq!(g.add_node(u64::MAX), Err(GraphError::GraphIsFrozen));
                }
                _ => {
                    g.clear().unwrap();
                    m.clear();
                }
            }
        } else {
            match roll {
                0..=39 => {
                    let (p, payload) = m.add_node();
                    assert_eq!(g.add_node(payload), Ok(p), "step {step}: add_node");
                }
                40..=47 => {
                    if m.root.is_none() {
                        let (p, payload) = m.add_node();
                        m.root = Some(p);
                        assert_eq!(
                            g.add_root_node(payload),
                            Ok(p),
                            "step {step}: add_root_node"
                        );
                    } else {
                        assert_eq!(
                            g.add_root_node(u64::MAX),
                            Err(GraphError::RootNodeAlreadyExists)
                        );
                    }
                }
                48..=59 => {
                    let p = rng.random_range(0..bound);
                    if m.is_live(p) {
                        m.remove_node(p);
                        assert_eq!(g.remove_node(p), Ok(()), "step {step}: remove_node({p})");
                    } else {
                        assert_eq!(g.remove_node(p), Err(GraphError::NodeNotFound(p)));
                    }
                }
                60..=84 => {
                    let a = rng.random_range(0..bound);
                    let b = rng.random_range(0..bound);
                    let w = weight(&mut rng);
                    if m.is_live(a) && m.is_live(b) {
                        // One edge per pair: a repeated pair is left alone (keyed edges are group 3).
                        if let Entry::Vacant(slot) = m.edges.entry((a, b)) {
                            slot.insert(w);
                            assert_eq!(g.add_edge(a, b, w), Ok(()), "step {step}: add_edge");
                        }
                    } else {
                        assert_eq!(
                            g.add_edge(a, b, w),
                            Err(GraphError::EdgeCreationError {
                                source: a,
                                target: b
                            })
                        );
                    }
                }
                85..=92 => {
                    g.freeze();
                    m.frozen = true;
                }
                93..=97 => {
                    let table = m.compact();
                    assert_eq!(g.compact_ids(), Ok(table), "step {step}: compact_ids");
                }
                _ => {
                    g.clear().unwrap();
                    m.clear();
                }
            }
        }

        check_against_model(&g, &m, step);
    }
}

#[test]
fn test_model_based_positions_with_unit_weights() {
    for seed in [1, 2, 3] {
        drive_model::<(), _>(seed, 400, |_| ());
    }
}

#[test]
fn test_model_based_positions_with_u64_weights() {
    for seed in [11, 12, 13] {
        drive_model::<u64, _>(seed, 400, |rng| rng.random_range(1..1000u64));
    }
}
