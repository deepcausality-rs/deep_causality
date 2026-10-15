/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

#[test]
fn test_add_node() {
    let mut g = UltraGraph::new();
    let id1 = g.add_node(10).unwrap();
    let id2 = g.add_node(20).unwrap();
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(g.number_nodes(), 2);
    assert_eq!(*g.get_node(id1).unwrap(), 10);
}

#[test]
fn test_add_node_to_frozen_graph() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(g.add_node(2), Err(GraphError::GraphIsFrozen)));
}

#[test]
fn test_update_node() {
    let mut g = UltraGraph::new();
    let id = g.add_node(10).unwrap();
    g.update_node(id, 100).unwrap();
    assert_eq!(*g.get_node(id).unwrap(), 100);
}

#[test]
fn test_update_node_on_frozen_graph() {
    let mut g = UltraGraph::new();
    let id = g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(
        g.update_node(id, 2),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_update_non_existent_node() {
    let mut g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.update_node(0, 100).is_err());
}

#[test]
fn test_remove_node() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert_eq!(g.number_nodes(), 2);
    assert_eq!(g.number_edges(), 1);

    g.remove_node(id0).unwrap();
    assert_eq!(g.number_nodes(), 1);
    assert!(!g.contains_node(id0));
    assert_eq!(g.number_edges(), 0); // Edge should be removed
}

#[test]
fn test_remove_node_from_frozen_graph() {
    let mut g = UltraGraph::new();
    let id = g.add_node(1).unwrap();
    g.freeze();
    assert!(matches!(g.remove_node(id), Err(GraphError::GraphIsFrozen)));
}

#[test]
fn test_remove_non_existent_node() {
    let mut g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.remove_node(0).is_err());
}

#[test]
fn test_add_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert_eq!(g.number_edges(), 1);
    assert!(g.contains_edge(id0, id1));
}

#[test]
fn test_add_edge_to_frozen_graph() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.freeze();
    assert!(matches!(
        g.add_edge(id0, id1, ()),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_add_edge_invalid_node() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    assert!(g.add_edge(id0, 99, ()).is_err());
    assert!(g.add_edge(99, id0, ()).is_err());
}

#[test]
fn test_remove_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    assert!(g.contains_edge(id0, id1));

    g.remove_edge(id0, id1).unwrap();
    assert!(!g.contains_edge(id0, id1));
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_remove_edge_from_frozen_graph() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    g.add_edge(id0, id1, ()).unwrap();
    g.freeze();
    assert!(matches!(
        g.remove_edge(id0, id1),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_remove_non_existent_edge() {
    let mut g = UltraGraph::new();
    let id0 = g.add_node(10).unwrap();
    let id1 = g.add_node(20).unwrap();
    assert!(g.remove_edge(id0, id1).is_err());
}

#[test]
fn test_add_root_node() {
    let mut g = UltraGraph::new();
    let id = g.add_root_node(100).unwrap();
    assert!(g.contains_root_node());
    assert_eq!(g.get_root_index(), Some(id));
    assert_eq!(*g.get_root_node().unwrap(), 100);
}

#[test]
fn test_add_root_node_to_frozen_graph() {
    let mut g = UltraGraph::new();
    g.freeze();
    assert!(matches!(
        g.add_root_node(100),
        Err(GraphError::GraphIsFrozen)
    ));
}

#[test]
fn test_clear() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.add_edge(0, 0, ()).unwrap();
    g.add_root_node(100).unwrap();
    g.clear().unwrap();
    assert!(g.is_empty());
    assert!(!g.contains_root_node());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(!g.is_frozen());
}

#[test]
fn test_clear_frozen_graph() {
    let mut g = UltraGraph::new();
    g.add_node(1).unwrap();
    g.freeze();
    assert!(g.is_frozen());
    g.clear().unwrap();
    assert!(g.is_empty());
    assert!(!g.is_frozen());
}

// --- Positions are never reissued (spec: ultragraph-stable-node-ids) ---

#[test]
fn test_removing_the_last_node_does_not_free_its_position() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        assert_eq!(g.add_node(payload).unwrap(), payload);
    }
    g.remove_node(3).unwrap();

    // The freed slot 3 is not reused: the next position is 4.
    assert_eq!(g.add_node(4).unwrap(), 4);
    assert!(!g.contains_node(3));
    assert_eq!(g.get_node(4), Some(&4));
    assert_eq!(g.number_nodes(), 4);
}

#[test]
fn test_clearing_a_frozen_graph_restarts_positions_at_zero() {
    let mut g = UltraGraph::new();
    for payload in 0..5 {
        assert_eq!(g.add_node(payload).unwrap(), payload);
    }
    g.freeze();
    assert!(g.is_frozen());

    g.clear().unwrap();

    assert!(!g.is_frozen());
    assert!(g.is_empty());
    assert_eq!(g.get_last_index(), None);
    assert_eq!(g.add_node(10).unwrap(), 0);
    assert_eq!(g.number_nodes(), 1);
}

// --- Keyed edges (spec: ultragraph-keyed-edges) ---

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

fn keyed(kind: Kind, weight: u32) -> Keyed {
    Keyed { kind, weight }
}

/// Nodes at positions 0 to `n - 1`, payload equal to position, in the Dynamic state.
fn keyed_nodes(n: usize) -> UltraGraphWeighted<usize, Keyed> {
    let mut g = UltraGraphWeighted::new();
    for position in 0..n {
        assert_eq!(g.add_node(position).unwrap(), position);
    }
    g
}

/// The `(target, key)` pairs of `get_edges(source)` in the order listed.
fn targets_and_keys(g: &UltraGraphWeighted<usize, Keyed>, source: usize) -> Vec<(usize, Kind)> {
    g.get_edges(source)
        .unwrap()
        .into_iter()
        .map(|(target, payload)| (target, payload.kind))
        .collect()
}

/// Spec `ultragraph-keyed-edges`, "Every graph payload implements EdgeKey": the unit and the
/// primitive numeric payloads build and accept `add_edge` in every graph type.
#[test]
fn test_the_weight_and_unit_payloads_compile() {
    fn one_edge<W: Clone + Default + EdgeKey>(w: W) -> UltraGraphContainer<u8, W> {
        let mut g = UltraGraphContainer::new();
        g.add_node(0).unwrap();
        g.add_node(1).unwrap();
        g.add_edge(0, 1, w).unwrap();
        g
    }

    assert!(one_edge(()).contains_edge(0, 1));
    assert!(one_edge(7_u64).contains_edge(0, 1));
    assert!(one_edge(-7_i64).contains_edge(0, 1));
    assert!(one_edge(0.5_f64).contains_edge(0, 1));

    // The same payloads through the two storage types: `DynamicGraph` directly, and
    // `CsmGraph` through `Freezable::freeze`.
    let mut dynamic: DynamicGraph<u8, f64> = DynamicGraph::new();
    dynamic.add_node(0).unwrap();
    dynamic.add_node(1).unwrap();
    dynamic.add_edge(0, 1, 1.5).unwrap();
    assert_eq!(dynamic.number_edges(), 1);
    let mut dynamic: DynamicGraph<u8, i64> = DynamicGraph::new();
    dynamic.add_node(0).unwrap();
    dynamic.add_node(1).unwrap();
    dynamic.add_edge(0, 1, -1).unwrap();
    let frozen: CsmGraph<u8, i64> = dynamic.freeze();
    assert!(frozen.contains_edge(0, 1));
}

#[test]
fn test_a_second_weight_updates_the_first() {
    let mut g: UltraGraphWeighted<usize, u64> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();

    g.add_edge(0, 1, 10_u64).unwrap();
    g.add_edge(0, 1, 15_u64).unwrap();

    // `u64` has `Key = ()`, so the second call replaces the first edge's payload.
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &15_u64)]);
}

#[test]
fn test_a_unit_payload_collapses_a_repeated_edge() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();

    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 1, ()).unwrap();

    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap().len(), 1);
}

#[test]
fn test_edges_with_different_keys_coexist() {
    let mut g = keyed_nodes(2);
    g.add_edge(0, 1, keyed(Kind::A, 3)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 4)).unwrap();

    assert_eq!(g.number_edges(), 2);
    let mut edges: Vec<(usize, Keyed)> = g
        .get_edges(0)
        .unwrap()
        .into_iter()
        .map(|(target, payload)| (target, *payload))
        .collect();
    edges.sort_by_key(|(target, payload)| (*target, payload.kind));
    assert_eq!(edges, vec![(1, keyed(Kind::A, 3)), (1, keyed(Kind::B, 4))]);
}

#[test]
fn test_an_edge_with_the_same_key_is_replaced() {
    let mut g = keyed_nodes(2);
    g.add_edge(0, 1, keyed(Kind::A, 3)).unwrap();
    g.add_edge(0, 1, keyed(Kind::A, 7)).unwrap();

    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &keyed(Kind::A, 7))]);
}

#[test]
fn test_an_edge_to_or_from_an_absent_node_is_refused() {
    let mut g = keyed_nodes(3);
    g.remove_node(2).unwrap(); // position 2 is a tombstone; position 3 was never issued
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();

    for (source, target) in [(0, 2), (2, 0), (0, 3), (3, 0), (2, 3)] {
        assert_eq!(
            g.add_edge(source, target, keyed(Kind::A, 1)),
            Err(GraphError::EdgeCreationError { source, target }),
            "add_edge({source}, {target})"
        );
    }

    // Nothing was added: the one edge from before is all there is.
    assert_eq!(g.number_edges(), 1);
    assert!(!g.contains_edge(0, 2));
    assert!(!g.contains_edge(0, 3));
}

/// Row F: zero edges after the pair is removed.
#[test]
fn test_removing_a_pair_removes_both_keys() {
    let mut g = keyed_nodes(2);
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();

    assert_eq!(g.remove_edge(0, 1), Ok(()));

    assert!(!g.contains_edge(0, 1));
    assert_eq!(g.number_edges(), 0);
    assert!(g.get_edges(0).unwrap().is_empty());
}

#[test]
fn test_removing_one_key_leaves_the_other() {
    let mut g = keyed_nodes(2);
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();

    assert_eq!(g.remove_edge_keyed(0, 1, Kind::A), Ok(()));

    assert!(!g.contains_edge_keyed(0, 1, Kind::A));
    assert!(g.contains_edge_keyed(0, 1, Kind::B));
    assert!(g.contains_edge(0, 1));
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &keyed(Kind::B, 2))]);
}

/// Row A: a pair with no edge, in both forms.
#[test]
fn test_removing_an_absent_edge_is_reported() {
    let mut g = keyed_nodes(2);

    assert_eq!(
        g.remove_edge(0, 1),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );

    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    assert_eq!(
        g.remove_edge_keyed(0, 1, Kind::B),
        Err(GraphError::EdgeNotFoundError {
            source: 0,
            target: 1
        })
    );

    // The edge with the other key is untouched.
    assert!(g.contains_edge_keyed(0, 1, Kind::A));
    assert_eq!(g.number_edges(), 1);
}

/// Row D: the source is a tombstone.
#[test]
fn test_removing_from_an_absent_source_is_reported() {
    let mut g = keyed_nodes(6);
    g.add_edge(5, 1, keyed(Kind::A, 1)).unwrap();
    g.remove_node(5).unwrap();

    assert_eq!(g.remove_edge(5, 1), Err(GraphError::NodeNotFound(5)));
    assert_eq!(
        g.remove_edge_keyed(5, 1, Kind::A),
        Err(GraphError::NodeNotFound(5))
    );

    // A position never issued is reported the same way.
    assert_eq!(g.remove_edge(6, 1), Err(GraphError::NodeNotFound(6)));
    assert_eq!(
        g.remove_edge_keyed(6, 1, Kind::A),
        Err(GraphError::NodeNotFound(6))
    );
}

// --- The frozen adjacency is sorted by target, then key ---

#[test]
fn test_edges_inserted_out_of_order_read_back_sorted() {
    let mut g = keyed_nodes(4);
    g.add_edge(0, 3, keyed(Kind::B, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();
    g.add_edge(0, 3, keyed(Kind::A, 3)).unwrap();
    g.add_edge(0, 1, keyed(Kind::A, 4)).unwrap();

    g.freeze();

    // Ascending target, and within a target ascending key: A < B.
    assert_eq!(
        targets_and_keys(&g, 0),
        vec![(1, Kind::A), (1, Kind::B), (3, Kind::A), (3, Kind::B)]
    );
}

/// A hub at position 0 with `degree` outgoing edges: the targets `1..` each carry an `A` and a
/// `B` edge, in that order of insertion, and target 2 is skipped so that an absent target sits
/// inside the searched range. Returns the graph, frozen, with the present targets.
fn frozen_hub(degree: usize) -> (UltraGraphWeighted<usize, Keyed>, Vec<usize>) {
    let targets_needed = degree.div_ceil(2) + 1; // one slot is the skipped target 2
    let mut g = keyed_nodes(targets_needed + 2);
    let mut present = Vec::new();
    let mut added = 0;
    let mut target = 1;
    while added < degree {
        if target == 2 {
            target += 1;
            continue;
        }
        g.add_edge(0, target, keyed(Kind::A, 1)).unwrap();
        added += 1;
        if added < degree {
            g.add_edge(0, target, keyed(Kind::B, 2)).unwrap();
            added += 1;
        }
        present.push(target);
        target += 1;
    }
    g.freeze();
    assert_eq!(g.get_edges(0).unwrap().len(), degree, "degree {degree}");
    (g, present)
}

/// Row E, the `contains_edge` search threshold at 64 outgoing edges.
#[test]
fn test_lookups_across_the_search_threshold() {
    for degree in [63_usize, 64, 65] {
        let (g, present) = frozen_hub(degree);
        for &target in &present {
            assert!(
                g.contains_edge(0, target),
                "degree {degree}, target {target}"
            );
            assert!(
                g.contains_edge_keyed(0, target, Kind::A),
                "degree {degree}, target {target}, key A"
            );
        }
        // The last present target has a `B` edge only when the degree is even.
        let last = *present.last().unwrap();
        assert_eq!(
            g.contains_edge_keyed(0, last, Kind::B),
            degree % 2 == 0,
            "degree {degree}, target {last}, key B"
        );
        // Absent inside the range, and absent past it.
        assert!(!g.contains_edge(0, 2), "degree {degree}, skipped target");
        assert!(!g.contains_edge_keyed(0, 2, Kind::A), "degree {degree}");
        assert!(
            !g.contains_edge(0, last + 1),
            "degree {degree}, past the range"
        );
    }
}

/// Row E, the freeze sort's radix threshold at 128 outgoing edges. Targets are added in
/// descending order and, within each target, `B` before `A`, so that neither the targets nor the
/// keys are in their sorted order at insertion.
#[test]
fn test_sorting_across_the_radix_threshold() {
    for degree in [127_usize, 128, 129] {
        let targets = degree.div_ceil(2);
        let mut g = keyed_nodes(targets + 1);
        let mut added = 0;
        for target in (1..=targets).rev() {
            if added < degree {
                g.add_edge(0, target, keyed(Kind::B, 2)).unwrap();
                added += 1;
            }
            if added < degree {
                g.add_edge(0, target, keyed(Kind::A, 1)).unwrap();
                added += 1;
            }
        }
        assert_eq!(added, degree);

        // By hand: every target carries A then B, except that at an odd degree the last target
        // inserted, target 1, received its B edge alone.
        let expected: Vec<(usize, Kind)> = (1..=targets)
            .flat_map(|target| {
                if target == 1 && degree % 2 == 1 {
                    vec![(1, Kind::B)]
                } else {
                    vec![(target, Kind::A), (target, Kind::B)]
                }
            })
            .collect();
        assert_eq!(expected.len(), degree, "degree {degree}: the plan");

        g.freeze();

        let listed = targets_and_keys(&g, 0);
        assert_eq!(
            listed, expected,
            "degree {degree}: ascending target, then key"
        );
        let listed_targets: Vec<usize> = listed.iter().map(|(target, _)| *target).collect();
        let mut ascending = listed_targets.clone();
        ascending.sort_unstable();
        assert_eq!(
            listed_targets, ascending,
            "degree {degree}: ascending targets"
        );
    }
}

/// `DynamicGraph::clear` is reached directly: the container replaces its state instead of calling
/// it. Every node, edge and the root go, and the next position is 0.
#[test]
fn test_dynamic_graph_clear_empties_it() {
    let mut g: DynamicGraph<u8, ()> = DynamicGraph::new();
    g.add_node(1).unwrap();
    g.add_root_node(2).unwrap();
    g.add_edge(0, 1, ()).unwrap();

    g.clear().unwrap();

    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert!(!g.contains_root_node());
    assert_eq!(g.add_node(3).unwrap(), 0);
}

/// Replacing the payload of an edge keeps one backward entry for it, so inbound traversal lists
/// the source once and a keyed removal finds the entry by its key.
#[test]
fn test_replacing_a_payload_keeps_one_backward_entry() {
    let mut g = keyed_nodes(2);
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::A, 2)).unwrap();

    // One edge 0 -> 1 with key A, payload weight 2: 1's inbound list is [0].
    assert_eq!(g.number_edges(), 1);
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &keyed(Kind::A, 2))]);
    let inbound: Vec<usize> = g.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![0]);

    g.remove_edge_keyed(0, 1, Kind::A).unwrap();
    assert_eq!(g.number_edges(), 0);
    assert!(g.inbound_edges(1).unwrap().next().is_none());
}

/// Removing a node joined to one neighbour by several keyed edges, in both directions and with a
/// self-loop, takes every one of them and leaves the other edges in place.
#[test]
fn test_remove_node_with_many_keyed_edges_to_one_neighbour() {
    let mut g = keyed_nodes(3);
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();
    g.add_edge(1, 0, keyed(Kind::A, 3)).unwrap();
    g.add_edge(1, 0, keyed(Kind::B, 4)).unwrap();
    g.add_edge(1, 1, keyed(Kind::A, 5)).unwrap();
    g.add_edge(2, 1, keyed(Kind::A, 6)).unwrap();
    g.add_edge(0, 2, keyed(Kind::A, 7)).unwrap();
    assert_eq!(g.number_edges(), 7);

    g.remove_node(1).unwrap();

    // Out of 1: (0, A), (0, B), (1, A), three edges. Into 1 from another node: (0, A), (0, B),
    // (2, A), three edges. The self-loop is one edge, so 6 of the 7 go and 0 -> 2 remains.
    assert_eq!(g.number_edges(), 1);
    assert_eq!(targets_and_keys(&g, 0), vec![(2, Kind::A)]);
    assert!(g.get_edges(2).unwrap().is_empty());
    assert!(g.inbound_edges(0).unwrap().next().is_none());
    let inbound: Vec<usize> = g.inbound_edges(2).unwrap().collect();
    assert_eq!(inbound, vec![0]);
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
}

/// `with_capacity(_, Some(k))` reserves `k` entries for each node's edge list as the node is
/// added; `new` reserves none. The reservation is observable through `to_parts`.
#[test]
fn test_with_capacity_reserves_each_edge_list() {
    let mut g: DynamicGraph<u8, ()> = DynamicGraph::with_capacity(2, Some(8));
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    let (nodes, edges, _) = g.to_parts();
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|list| list.capacity() >= 8));

    let mut g: DynamicGraph<u8, ()> = DynamicGraph::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    let (_, edges, _) = g.to_parts();
    assert!(edges.iter().all(|list| list.capacity() == 0));
}
