/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! PAG edge-kind completeness: every endpoint-mark combination round-trips
//! through construction, classification, reorientation, and removal, and the
//! invariant holds throughout.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, Mark, MixedGraph};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// The six edge kinds and a representative `(mark_u, mark_v)` for each.
const KINDS: [(EdgeKind, (Mark, Mark)); 6] = [
    (EdgeKind::Directed, (Mark::Tail, Mark::Arrow)),
    (EdgeKind::Undirected, (Mark::Tail, Mark::Tail)),
    (EdgeKind::Bidirected, (Mark::Arrow, Mark::Arrow)),
    (EdgeKind::PartiallyDirected, (Mark::Circle, Mark::Arrow)),
    (EdgeKind::Nondirected, (Mark::Circle, Mark::Circle)),
    (EdgeKind::PartiallyUndirected, (Mark::Circle, Mark::Tail)),
];

#[test]
fn every_kind_round_trips_construct_classify_remove() {
    for (kind, (mu, mv)) in KINDS {
        let mut g = graph(2);
        g.add_edge(0, 1, mu, mv).unwrap();
        assert_eq!(g.edge_kind(0, 1), Some(kind), "classification for {kind:?}");
        assert_eq!(g.edge_marks(0, 1), Some((mu, mv)));
        assert!(g.invariant_holds());
        assert!(g.remove_edge(0, 1).unwrap());
        assert_eq!(g.num_edges(), 0);
    }
}

#[test]
fn reorientation_walks_through_every_kind() {
    // Start nondirected ∘—∘ and reorient endpoint by endpoint, visiting kinds.
    let mut g = graph(2);
    g.add_edge(0, 1, Mark::Circle, Mark::Circle).unwrap();
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Nondirected));

    g.set_endpoint(1, 0, Mark::Arrow).unwrap(); // ∘→
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::PartiallyDirected));

    g.set_endpoint(0, 1, Mark::Tail).unwrap(); // →
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));

    g.set_endpoint(0, 1, Mark::Arrow).unwrap(); // ↔
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Bidirected));

    g.set_endpoint(0, 1, Mark::Tail).unwrap();
    g.set_endpoint(1, 0, Mark::Tail).unwrap(); // —
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Undirected));

    g.set_endpoint(0, 1, Mark::Circle).unwrap(); // ∘—
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::PartiallyUndirected));

    assert!(g.invariant_holds());
}

#[test]
fn full_pag_is_expressible_and_counts_are_consistent() {
    // A graph carrying one edge of every kind.
    let mut g = graph(12);
    for (i, (_, (mu, mv))) in KINDS.iter().enumerate() {
        let (u, v) = (2 * i, 2 * i + 1);
        g.add_edge(u, v, *mu, *mv).unwrap();
    }
    assert_eq!(g.num_edges(), 6);
    for (kind, _) in KINDS {
        assert_eq!(g.count_of_kind(kind), 1, "exactly one {kind:?} edge");
    }
    assert!(g.invariant_holds());
    // Directed-arc projection sees only the directed edge; it is acyclic.
    assert_eq!(g.arcs().len(), 1);
    assert!(!g.has_cycle());
}
