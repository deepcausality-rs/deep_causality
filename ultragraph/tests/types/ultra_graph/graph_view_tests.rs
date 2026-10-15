/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

fn get_dynamic_graph() -> UltraGraphContainer<String, u32> {
    let mut g = UltraGraphWeighted::new();
    g.add_node("A".to_string()).unwrap(); // 0
    g.add_node("B".to_string()).unwrap(); // 1
    g.add_node("C".to_string()).unwrap(); // 2
    g.add_edge(0, 1, 10).unwrap();
    g.add_edge(1, 2, 20).unwrap();
    g.add_root_node("ROOT".to_string()).unwrap(); // 3
    g
}

fn get_static_graph() -> UltraGraphContainer<String, u32> {
    let mut g = get_dynamic_graph();
    g.freeze();
    g
}

// Test functions for both dynamic and static graphs
fn test_view_on_graph<F>(graph_provider: F)
where
    F: Fn() -> UltraGraphContainer<String, u32>,
{
    let g = graph_provider();

    // is_empty
    assert!(!g.is_empty());

    // contains_node
    assert!(g.contains_node(0));
    assert!(g.contains_node(3));
    assert!(!g.contains_node(4));

    // get_node
    assert_eq!(g.get_node(0), Some(&"A".to_string()));
    assert_eq!(g.get_node(4), None);

    // number_nodes
    assert_eq!(g.number_nodes(), 4);

    // contains_edge
    assert!(g.contains_edge(0, 1));
    assert!(!g.contains_edge(1, 0));

    // number_edges
    assert_eq!(g.number_edges(), 2);

    // get_edges
    let edges = g.get_edges(1).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0], (2, &20));
    assert!(g.get_edges(0).is_some());
    assert!(g.get_edges(2).unwrap().is_empty()); // No outgoing edges from C
    assert!(g.get_edges(99).is_none());

    // contains_root_node
    assert!(g.contains_root_node());

    // get_root_node
    assert_eq!(g.get_root_node(), Some(&"ROOT".to_string()));

    // get_root_index
    assert_eq!(g.get_root_index(), Some(3));
}

#[test]
fn test_view_on_dynamic_graph() {
    test_view_on_graph(get_dynamic_graph);
}

#[test]
fn test_view_on_static_graph() {
    test_view_on_graph(get_static_graph);
}

#[test]
fn test_is_frozen() {
    let dynamic_g = get_dynamic_graph();
    assert!(!dynamic_g.is_frozen());

    let static_g = get_static_graph();
    assert!(static_g.is_frozen());
}

#[test]
fn test_empty_graph_view() {
    let g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(!g.contains_node(0));
    assert!(!g.contains_edge(0, 0));
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);
}

#[test]
fn test_graph_without_root() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);

    g.freeze();
    assert!(!g.contains_root_node());
    assert_eq!(g.get_root_node(), None);
    assert_eq!(g.get_root_index(), None);
}

// --- Tombstones and live counts (spec: ultragraph-stable-node-ids) ---

/// Nodes A, B, C and D at positions 0 to 3.
fn four_nodes() -> UltraGraph<&'static str> {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g
}

#[test]
fn test_is_empty_counts_live_nodes() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    for position in 0..3 {
        g.remove_node(position).unwrap();
    }

    // Dynamic state.
    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);

    // Frozen state.
    g.freeze();
    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);

    // The node table still holds the three slots: the next position is 3.
    g.unfreeze();
    assert_eq!(g.add_node("D").unwrap(), 3);
}

#[test]
fn test_counting_after_a_removal() {
    let mut g = four_nodes();
    g.remove_node(1).unwrap();

    // Dynamic state: three live nodes, the highest at position 3.
    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.get_last_index(), Some(3));

    // Frozen state: the same, since position 3 is still D.
    g.freeze();
    assert_eq!(g.number_nodes(), 3);
    assert_eq!(g.get_last_index(), Some(3));
    assert_eq!(g.get_node(3), Some(&"D"));
}

/// Row H: a tombstone at the last slot, and the position one past the slot count beside it.
#[test]
fn test_the_highest_live_position_skips_trailing_tombstones() {
    let mut g = four_nodes();
    g.remove_node(3).unwrap();

    // Dynamic state.
    assert_eq!(g.get_last_index(), Some(2));
    assert_eq!(g.number_nodes(), 3);
    assert!(!g.contains_node(3)); // the trailing tombstone
    assert!(!g.contains_node(4)); // one past the slot count

    // Frozen state.
    g.freeze();
    assert_eq!(g.get_last_index(), Some(2));
    assert_eq!(g.number_nodes(), 3);
    assert!(!g.contains_node(3));
    assert!(!g.contains_node(4));

    // The trailing slot survived the freeze: the next position is 4, not 3.
    g.unfreeze();
    assert_eq!(g.add_node("E").unwrap(), 4);
}

#[test]
fn test_a_frozen_tombstone_is_absent() {
    let mut g = four_nodes();
    g.remove_node(1).unwrap();

    g.freeze();

    assert!(!g.contains_node(1));
    assert_eq!(g.get_node(1), None);

    let mut payloads: Vec<&str> = g.get_all_nodes().into_iter().copied().collect();
    payloads.sort_unstable();
    assert_eq!(payloads, vec!["A", "C", "D"]);
}

// --- Keyed edge reads (spec: ultragraph-keyed-edges) ---

/// The two-variant kind that keys the `Keyed` payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    #[default]
    A,
    B,
}

/// A payload whose kind identifies the edge and whose weight is data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Keyed {
    kind: Kind,
    weight: u32,
}

impl EdgeKey for Keyed {
    type Key = Kind;

    fn edge_key(&self) -> Self::Key {
        self.kind
    }
}

/// A payload identified by a 128-bit id; the note is data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Identified {
    id: u128,
    note: &'static str,
}

impl EdgeKey for Identified {
    type Key = u128;

    fn edge_key(&self) -> Self::Key {
        self.id
    }
}

#[test]
fn test_keyed_and_unkeyed_tests_agree_with_the_edges_present() {
    let mut g: UltraGraphWeighted<usize, Keyed> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(
        0,
        1,
        Keyed {
            kind: Kind::A,
            weight: 1,
        },
    )
    .unwrap();

    // Dynamic state.
    assert!(g.contains_edge(0, 1));
    assert!(g.contains_edge_keyed(0, 1, Kind::A));
    assert!(!g.contains_edge_keyed(0, 1, Kind::B));
    assert!(!g.contains_edge(1, 0));
    assert!(!g.contains_edge_keyed(1, 0, Kind::A));

    g.freeze();

    // Frozen state: the same four answers.
    assert!(g.contains_edge(0, 1));
    assert!(g.contains_edge_keyed(0, 1, Kind::A));
    assert!(!g.contains_edge_keyed(0, 1, Kind::B));
    assert!(!g.contains_edge(1, 0));
    assert!(!g.contains_edge_keyed(1, 0, Kind::A));
}

/// Row K: an identity-keyed payload through add, remove, contains, freeze and traversal.
#[test]
fn test_an_identity_keyed_payload() {
    // Ids above `u64::MAX` so that the key is used at its full width.
    let first = 1_u128 << 100;
    let second = (1_u128 << 100) + 1;
    let absent = (1_u128 << 100) + 2;

    let mut g: UltraGraphWeighted<&str, Identified> = UltraGraphWeighted::new();
    g.add_node("source").unwrap(); // 0
    g.add_node("target").unwrap(); // 1

    g.add_edge(
        0,
        1,
        Identified {
            id: first,
            note: "first",
        },
    )
    .unwrap();
    g.add_edge(
        0,
        1,
        Identified {
            id: second,
            note: "second",
        },
    )
    .unwrap();
    assert_eq!(g.number_edges(), 2);

    // Adding the first id again replaces its note and adds no edge.
    g.add_edge(
        0,
        1,
        Identified {
            id: first,
            note: "first, revised",
        },
    )
    .unwrap();
    assert_eq!(g.number_edges(), 2);
    let notes: Vec<&str> = {
        let mut edges = g.get_edges(0).unwrap();
        edges.sort_by_key(|(_, payload)| payload.id);
        edges.into_iter().map(|(_, payload)| payload.note).collect()
    };
    assert_eq!(notes, vec!["first, revised", "second"]);

    assert!(g.contains_edge(0, 1));
    assert!(g.contains_edge_keyed(0, 1, first));
    assert!(g.contains_edge_keyed(0, 1, second));
    assert!(!g.contains_edge_keyed(0, 1, absent));

    // Removing one id leaves the other.
    assert_eq!(g.remove_edge_keyed(0, 1, first), Ok(()));
    assert_eq!(
        g.remove_edge_keyed(0, 1, absent),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );
    assert_eq!(g.number_edges(), 1);
    assert!(!g.contains_edge_keyed(0, 1, first));
    assert!(g.contains_edge_keyed(0, 1, second));

    // Dynamic traversal lists the one remaining edge.
    let out: Vec<usize> = g.outbound_edges(0).unwrap().collect();
    assert_eq!(out, vec![1]);
    let inbound: Vec<usize> = g.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![0]);

    g.freeze();

    assert_eq!(g.number_edges(), 1);
    assert!(g.contains_edge(0, 1));
    assert!(!g.contains_edge_keyed(0, 1, first));
    assert!(g.contains_edge_keyed(0, 1, second));
    assert_eq!(
        g.get_edges(0).unwrap(),
        vec![(
            1,
            &Identified {
                id: second,
                note: "second"
            }
        )]
    );
    let out: Vec<usize> = g.outbound_edges(0).unwrap().collect();
    assert_eq!(out, vec![1]);
    let inbound: Vec<usize> = g.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![0]);
}
