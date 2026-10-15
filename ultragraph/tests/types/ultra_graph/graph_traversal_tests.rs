/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{Rng, Xoshiro256};
use std::collections::BTreeMap;
use ultragraph::*;

fn get_test_graph() -> UltraGraphContainer<i32, ()> {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap(); // 0
    g.add_node(1).unwrap(); // 1
    g.add_node(2).unwrap(); // 2
    g.add_node(3).unwrap(); // 3

    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap(); // Cycle
    g.add_edge(2, 3, ()).unwrap();

    g.freeze();
    g
}

/// Spec `ultragraph-dynamic-traversal`: traversal works in the Dynamic state.
#[test]
fn test_outbound_edges_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    let n0 = g.add_node(0).unwrap();
    let n1 = g.add_node(1).unwrap();
    let n2 = g.add_node(2).unwrap();
    g.add_edge(n0, n1, ()).unwrap();
    g.add_edge(n0, n2, ()).unwrap();

    assert!(!g.is_frozen());
    let mut edges: Vec<_> = g.outbound_edges(n0).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![n1, n2]);

    let edges: Vec<_> = g.outbound_edges(n1).unwrap().collect();
    assert!(edges.is_empty());
}

#[test]
fn test_outbound_edges_on_static_graph() {
    let g = get_test_graph();
    let mut edges: Vec<_> = g.outbound_edges(0).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![1, 2]);

    let mut edges: Vec<_> = g.outbound_edges(1).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);

    let mut edges: Vec<_> = g.outbound_edges(2).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0, 3]);

    let edges: Vec<_> = g.outbound_edges(3).unwrap().collect();
    assert!(edges.is_empty());
}

#[test]
fn test_outbound_edges_invalid_node() {
    let g = get_test_graph();
    assert!(matches!(
        g.outbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}

/// Spec `ultragraph-dynamic-traversal`: traversal works in the Dynamic state.
#[test]
fn test_inbound_edges_on_dynamic_graph() {
    let mut g = UltraGraph::new();
    let n0 = g.add_node(0).unwrap();
    let n1 = g.add_node(1).unwrap();
    let n2 = g.add_node(2).unwrap();
    g.add_edge(n0, n1, ()).unwrap();
    g.add_edge(n2, n1, ()).unwrap();

    assert!(!g.is_frozen());
    let mut edges: Vec<_> = g.inbound_edges(n1).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![n0, n2]);

    let edges: Vec<_> = g.inbound_edges(n0).unwrap().collect();
    assert!(edges.is_empty());
}

#[test]
fn test_inbound_edges_on_static_graph() {
    let g = get_test_graph();

    let mut edges: Vec<_> = g.inbound_edges(0).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);

    let mut edges: Vec<_> = g.inbound_edges(1).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0]);

    let mut edges: Vec<_> = g.inbound_edges(2).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![0, 1]);

    let mut edges: Vec<_> = g.inbound_edges(3).unwrap().collect();
    edges.sort_unstable();
    assert_eq!(edges, vec![2]);
}

#[test]
fn test_inbound_edges_invalid_node() {
    let g = get_test_graph();
    assert!(matches!(
        g.inbound_edges(99),
        Err(GraphError::NodeNotFound(99))
    ));
}

/// Spec `ultragraph-stable-node-ids`: traversal started at a tombstone returns `NodeNotFound`
/// with the requested position in both states, as it does for a position never issued.
#[test]
fn test_traversal_from_a_tombstone_fails_in_both_states() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();

    g.remove_node(1).unwrap();

    // Dynamic state: the tombstone, not the state, is what the error names.
    assert!(matches!(
        g.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));

    g.freeze();

    assert!(matches!(
        g.outbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));
    assert!(matches!(
        g.inbound_edges(1),
        Err(GraphError::NodeNotFound(1))
    ));

    // The live nodes above the tombstone keep their positions and lose only the edges into it.
    let out: Vec<_> = g.outbound_edges(2).unwrap().collect();
    assert_eq!(out, vec![3]);
    let inbound: Vec<_> = g.inbound_edges(2).unwrap().collect();
    assert_eq!(inbound, vec![0]); // the edge 1 -> 2 left with the tombstone
}

// --- Traversal in the Dynamic state (spec: ultragraph-dynamic-traversal) ---

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

/// `outbound_edges(a)` sorted, or the error.
fn sorted_outbound<N, W>(g: &UltraGraphContainer<N, W>, a: usize) -> Result<Vec<usize>, GraphError>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    g.outbound_edges(a).map(|edges| {
        let mut edges: Vec<usize> = edges.collect();
        edges.sort_unstable();
        edges
    })
}

/// `inbound_edges(a)` sorted, or the error.
fn sorted_inbound<N, W>(g: &UltraGraphContainer<N, W>, a: usize) -> Result<Vec<usize>, GraphError>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    g.inbound_edges(a).map(|edges| {
        let mut edges: Vec<usize> = edges.collect();
        edges.sort_unstable();
        edges
    })
}

#[test]
fn test_dynamic_listings_equal_frozen_listings() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();

    // Read off the edge list by hand: successors and predecessors of each node.
    let expected_out: [Vec<usize>; 4] = [vec![1, 2], vec![2], vec![0, 3], vec![]];
    let expected_in: [Vec<usize>; 4] = [vec![2], vec![0], vec![0, 1], vec![2]];

    assert!(!g.is_frozen());
    for node in 0..4 {
        assert_eq!(
            sorted_outbound(&g, node),
            Ok(expected_out[node].clone()),
            "dynamic out {node}"
        );
        assert_eq!(
            sorted_inbound(&g, node),
            Ok(expected_in[node].clone()),
            "dynamic in {node}"
        );
    }

    g.freeze();

    for node in 0..4 {
        assert_eq!(
            sorted_outbound(&g, node),
            Ok(expected_out[node].clone()),
            "frozen out {node}"
        );
        assert_eq!(
            sorted_inbound(&g, node),
            Ok(expected_in[node].clone()),
            "frozen in {node}"
        );
    }
}

/// Row B: one edge, then two edges between one pair.
#[test]
fn test_one_entry_per_edge() {
    let mut g: UltraGraphWeighted<usize, Keyed> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();

    assert_eq!(sorted_outbound(&g, 0), Ok(vec![1]));
    assert_eq!(sorted_inbound(&g, 1), Ok(vec![0]));

    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();

    assert_eq!(sorted_outbound(&g, 0), Ok(vec![1, 1]));
    assert_eq!(sorted_inbound(&g, 1), Ok(vec![0, 0]));
    assert_eq!(sorted_outbound(&g, 1), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 0), Ok(vec![]));

    g.freeze();

    assert_eq!(sorted_outbound(&g, 0), Ok(vec![1, 1]));
    assert_eq!(sorted_inbound(&g, 1), Ok(vec![0, 0]));
    assert_eq!(sorted_outbound(&g, 1), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 0), Ok(vec![]));
}

#[test]
fn test_removing_an_edge_updates_both_directions() {
    let mut g = UltraGraph::new();
    for payload in 0..3 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(2, 1, ()).unwrap();

    g.remove_edge(0, 1).unwrap();

    assert_eq!(sorted_inbound(&g, 1), Ok(vec![2]));
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_outbound(&g, 2), Ok(vec![1]));

    g.freeze();
    assert_eq!(sorted_inbound(&g, 1), Ok(vec![2]));
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
}

/// The keyed form updates the backward list as the pair form does.
#[test]
fn test_removing_one_key_updates_both_directions() {
    let mut g: UltraGraphWeighted<usize, Keyed> = UltraGraphWeighted::new();
    for payload in 0..3 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, keyed(Kind::A, 1)).unwrap();
    g.add_edge(0, 1, keyed(Kind::B, 2)).unwrap();
    g.add_edge(2, 1, keyed(Kind::A, 3)).unwrap();

    g.remove_edge_keyed(0, 1, Kind::A).unwrap();

    assert_eq!(sorted_inbound(&g, 1), Ok(vec![0, 2]));
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![1]));

    g.freeze();
    assert_eq!(sorted_inbound(&g, 1), Ok(vec![0, 2]));
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![1]));
}

/// Row D: a node with edges in and out removed from the middle of the table.
#[test]
fn test_incoming_edges_leave_with_the_node() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();

    g.remove_node(2).unwrap();

    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_outbound(&g, 1), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 3), Ok(vec![]));
    assert!(!g.contains_edge(0, 2));
    assert!(!g.contains_edge(1, 2));
    assert!(!g.contains_edge(2, 3));
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.get_edges(0), Some(vec![]));

    // The tombstone itself is absent in both directions.
    assert_eq!(sorted_outbound(&g, 2), Err(GraphError::NodeNotFound(2)));
    assert_eq!(sorted_inbound(&g, 2), Err(GraphError::NodeNotFound(2)));

    g.freeze();
    assert_eq!(g.number_edges(), 0);
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 3), Ok(vec![]));
}

#[test]
fn test_counting_after_a_node_removal() {
    let mut g = UltraGraph::new();
    for payload in 0..3 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    g.add_edge(2, 0, ()).unwrap();

    g.remove_node(1).unwrap();

    // Of the three edges only 2 -> 0 joins two live nodes.
    assert_eq!(g.number_edges(), 1);
    g.freeze();
    assert_eq!(g.number_edges(), 1);
}

/// Row A: an isolated node lists nothing in either direction, in both states.
#[test]
fn test_an_isolated_node_lists_nothing() {
    let mut g = UltraGraph::new();
    g.add_node(0).unwrap();

    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_outbound(&g, 1), Err(GraphError::NodeNotFound(1)));

    g.freeze();
    assert_eq!(sorted_outbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 0), Ok(vec![]));
    assert_eq!(sorted_inbound(&g, 1), Err(GraphError::NodeNotFound(1)));
}

// --- Agreement over generated mutation sequences ---

/// The reference model: live positions and the edges as `(source, target, key)` triples.
struct EdgeModel {
    live: Vec<bool>,
    edges: BTreeMap<(usize, usize, Kind), u32>,
}

impl EdgeModel {
    fn new() -> Self {
        Self {
            live: Vec::new(),
            edges: BTreeMap::new(),
        }
    }

    fn is_live(&self, position: usize) -> bool {
        self.live.get(position).copied().unwrap_or(false)
    }

    /// The successors of `a`, one entry per edge, sorted.
    fn outbound(&self, a: usize) -> Vec<usize> {
        self.edges
            .keys()
            .filter(|(source, _, _)| *source == a)
            .map(|(_, target, _)| *target)
            .collect()
    }

    /// The predecessors of `a`, one entry per edge, sorted.
    fn inbound(&self, a: usize) -> Vec<usize> {
        let mut sources: Vec<usize> = self
            .edges
            .keys()
            .filter(|(_, target, _)| *target == a)
            .map(|(source, _, _)| *source)
            .collect();
        sources.sort_unstable();
        sources
    }
}

/// Every position up to one past the table, in both directions, against the model.
fn check_traversal(g: &UltraGraphWeighted<u64, Keyed>, m: &EdgeModel, state: &str, step: usize) {
    assert_eq!(
        g.number_edges(),
        m.edges.len(),
        "step {step} {state}: number_edges"
    );
    for p in 0..=m.live.len() {
        if m.is_live(p) {
            assert_eq!(
                sorted_outbound(g, p),
                Ok(m.outbound(p)),
                "step {step} {state}: out {p}"
            );
            assert_eq!(
                sorted_inbound(g, p),
                Ok(m.inbound(p)),
                "step {step} {state}: in {p}"
            );
        } else {
            assert_eq!(
                sorted_outbound(g, p),
                Err(GraphError::NodeNotFound(p)),
                "step {step} {state}: out {p}"
            );
            assert_eq!(
                sorted_inbound(g, p),
                Err(GraphError::NodeNotFound(p)),
                "step {step} {state}: in {p}"
            );
        }
    }
}

/// Drives one seeded sequence of node and edge additions and removals, in both keyed forms,
/// against the model, checking both directions in the Dynamic state and, on a frozen copy, in
/// the Frozen state after every step.
fn drive_traversal_model(seed: u64, steps: usize) {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut g: UltraGraphWeighted<u64, Keyed> = UltraGraphWeighted::new();
    let mut m = EdgeModel::new();

    for step in 0..steps {
        let roll = rng.random_range(0..100usize);
        let bound = m.live.len() + 1;
        let kind = if rng.random_range(0..2usize) == 0 {
            Kind::A
        } else {
            Kind::B
        };

        match roll {
            0..=24 => {
                let p = m.live.len();
                m.live.push(true);
                assert_eq!(g.add_node(p as u64), Ok(p), "step {step}: add_node");
            }
            25..=34 => {
                let p = rng.random_range(0..bound);
                if m.is_live(p) {
                    m.live[p] = false;
                    m.edges
                        .retain(|&(source, target, _), _| source != p && target != p);
                    assert_eq!(g.remove_node(p), Ok(()), "step {step}: remove_node({p})");
                } else {
                    assert_eq!(g.remove_node(p), Err(GraphError::NodeNotFound(p)));
                }
            }
            35..=69 => {
                let a = rng.random_range(0..bound);
                let b = rng.random_range(0..bound);
                let w = rng.random_range(0..100u32);
                if m.is_live(a) && m.is_live(b) {
                    m.edges.insert((a, b, kind), w);
                    assert_eq!(
                        g.add_edge(a, b, keyed(kind, w)),
                        Ok(()),
                        "step {step}: add_edge"
                    );
                } else {
                    assert_eq!(
                        g.add_edge(a, b, keyed(kind, w)),
                        Err(GraphError::EdgeCreationError {
                            source: a,
                            target: b
                        })
                    );
                }
            }
            70..=84 => {
                let a = rng.random_range(0..bound);
                let b = rng.random_range(0..bound);
                if !m.is_live(a) {
                    assert_eq!(g.remove_edge(a, b), Err(GraphError::NodeNotFound(a)));
                } else {
                    let before = m.edges.len();
                    m.edges
                        .retain(|&(source, target, _), _| source != a || target != b);
                    if m.edges.len() < before {
                        assert_eq!(g.remove_edge(a, b), Ok(()), "step {step}: remove_edge");
                    } else {
                        assert_eq!(
                            g.remove_edge(a, b),
                            Err(GraphError::EdgeNotFoundError {
                                source: a,
                                target: b
                            })
                        );
                    }
                }
            }
            _ => {
                let a = rng.random_range(0..bound);
                let b = rng.random_range(0..bound);
                if !m.is_live(a) {
                    assert_eq!(
                        g.remove_edge_keyed(a, b, kind),
                        Err(GraphError::NodeNotFound(a))
                    );
                } else if m.edges.remove(&(a, b, kind)).is_some() {
                    assert_eq!(
                        g.remove_edge_keyed(a, b, kind),
                        Ok(()),
                        "step {step}: remove_edge_keyed"
                    );
                } else {
                    assert_eq!(
                        g.remove_edge_keyed(a, b, kind),
                        Err(GraphError::EdgeNotFoundError {
                            source: a,
                            target: b
                        })
                    );
                }
            }
        }

        assert!(!g.is_frozen());
        check_traversal(&g, &m, "dynamic", step);

        let mut frozen = g.clone();
        frozen.freeze();
        check_traversal(&frozen, &m, "frozen", step);
    }
}

#[test]
fn test_agreement_over_generated_mutation_sequences() {
    for seed in [21, 22, 23] {
        drive_traversal_model(seed, 300);
    }
}

/// Two sources share one key into node 1; removing one key from one source leaves the other
/// source's edge in the inbound list.
#[test]
fn test_removing_one_key_from_one_source_leaves_the_other_source() {
    let mut g = UltraGraphWeighted::new();
    for position in 0..3 {
        assert_eq!(g.add_node(position).unwrap(), position);
    }
    g.add_edge(0, 1, 7u64).unwrap();
    g.add_edge(2, 1, 9u64).unwrap();

    g.remove_edge_keyed(0, 1, ()).unwrap();

    // Only the edge 2 -> 1 remains, so 1's inbound list is [2] in both states.
    let inbound: Vec<usize> = g.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![2]);
    assert!(g.outbound_edges(0).unwrap().next().is_none());
    g.freeze();
    let inbound: Vec<usize> = g.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![2]);
}

/// `from_parts` derives the backward lists from the forward lists, so a graph rebuilt from its
/// parts answers inbound traversal.
#[test]
fn test_from_parts_rebuilds_the_backward_lists() {
    let mut g: DynamicGraph<u8, u64> = DynamicGraph::new();
    for payload in 0..3 {
        g.add_node(payload).unwrap();
    }
    g.add_edge(0, 2, 5u64).unwrap();
    g.add_edge(1, 2, 6u64).unwrap();

    let rebuilt = DynamicGraph::from_parts(g.to_parts());

    // Edges 0 -> 2 and 1 -> 2: 2's inbound list is [0, 1], the count is 2.
    let inbound: Vec<usize> = rebuilt.inbound_edges(2).unwrap().collect();
    assert_eq!(inbound, vec![0, 1]);
    assert_eq!(rebuilt.number_edges(), 2);
    assert!(rebuilt.contains_edge_keyed(1, 2, ()));
}

/// The container's traversal iterator reports the size hint of the active state's iterator:
/// exact in both states, and decreasing as items are taken.
#[test]
fn test_traversal_size_hint_is_exact_in_both_states() {
    let mut g = UltraGraph::new();
    for position in 0..4 {
        assert_eq!(g.add_node(position).unwrap(), position);
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(0, 3, ()).unwrap();

    // 0 has three outgoing edges and 1 has one incoming edge, in both states.
    for _ in 0..2 {
        {
            let mut out = g.outbound_edges(0).unwrap();
            assert_eq!(out.size_hint(), (3, Some(3)));
            out.next();
            assert_eq!(out.size_hint(), (2, Some(2)));
            assert_eq!(g.inbound_edges(1).unwrap().size_hint(), (1, Some(1)));
            assert_eq!(g.outbound_edges(3).unwrap().size_hint(), (0, Some(0)));
        }
        g.freeze();
    }
}
