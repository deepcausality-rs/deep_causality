/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, Mark, MixedGraph, TopologyError, TopologyErrorEnum};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn add_arc_records_directed_edge() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    assert_eq!(g.num_edges(), 1);
    assert_eq!(g.edge_marks(0, 1), Some((Mark::Tail, Mark::Arrow)));
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));
    assert!(g.invariant_holds());
}

#[test]
fn add_undirected_and_bidirected() {
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_bidirected(1, 2).unwrap();
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Undirected));
    assert_eq!(g.edge_kind(1, 2), Some(EdgeKind::Bidirected));
    assert_eq!(g.count_of_kind(EdgeKind::Undirected), 1);
    assert_eq!(g.count_of_kind(EdgeKind::Bidirected), 1);
}

#[test]
fn every_endpoint_combination_is_representable() {
    let mut g = graph(6);
    // (Tail, Arrow), (Tail, Tail), (Arrow, Arrow), (Circle, Arrow), (Circle, Circle), (Circle, Tail)
    g.add_edge(0, 1, Mark::Tail, Mark::Arrow).unwrap();
    g.add_edge(0, 2, Mark::Tail, Mark::Tail).unwrap();
    g.add_edge(0, 3, Mark::Arrow, Mark::Arrow).unwrap();
    g.add_edge(0, 4, Mark::Circle, Mark::Arrow).unwrap();
    g.add_edge(1, 5, Mark::Circle, Mark::Circle).unwrap();
    g.add_edge(2, 4, Mark::Circle, Mark::Tail).unwrap();
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));
    assert_eq!(g.edge_kind(0, 2), Some(EdgeKind::Undirected));
    assert_eq!(g.edge_kind(0, 3), Some(EdgeKind::Bidirected));
    assert_eq!(g.edge_kind(0, 4), Some(EdgeKind::PartiallyDirected));
    assert_eq!(g.edge_kind(1, 5), Some(EdgeKind::Nondirected));
    assert_eq!(g.edge_kind(2, 4), Some(EdgeKind::PartiallyUndirected));
    assert!(g.invariant_holds());
}

#[test]
fn marks_are_order_agnostic() {
    let mut g = graph(2);
    // add as (1, 0): Tail at 1, Arrow at 0  => arc 1 → 0
    g.add_edge(1, 0, Mark::Tail, Mark::Arrow).unwrap();
    assert_eq!(g.edge_marks(1, 0), Some((Mark::Tail, Mark::Arrow)));
    assert_eq!(g.edge_marks(0, 1), Some((Mark::Arrow, Mark::Tail)));
    assert_eq!(g.endpoint_mark(0, 1), Some(Mark::Arrow));
    assert_eq!(g.endpoint_mark(1, 0), Some(Mark::Tail));
}

#[test]
fn orient_undirected_to_directed() {
    let mut g = graph(2);
    g.add_undirected(0, 1).unwrap();
    g.orient(0, 1).unwrap();
    assert_eq!(g.edge_marks(0, 1), Some((Mark::Tail, Mark::Arrow)));
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));
}

#[test]
fn set_endpoint_circle_to_tail_makes_directed() {
    let mut g = graph(2);
    g.add_edge(0, 1, Mark::Circle, Mark::Arrow).unwrap(); // 0 ∘→ 1
    g.set_endpoint(0, 1, Mark::Tail).unwrap(); // 0 → 1
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));
    assert_eq!(g.edge_marks(0, 1), Some((Mark::Tail, Mark::Arrow)));
}

#[test]
fn set_both_endpoints_to_arrow_makes_bidirected() {
    let mut g = graph(2);
    g.add_undirected(0, 1).unwrap();
    g.set_endpoint(0, 1, Mark::Arrow).unwrap();
    g.set_endpoint(1, 0, Mark::Arrow).unwrap();
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Bidirected));
}

#[test]
fn remove_edge_reports_presence() {
    let mut g = graph(2);
    g.add_arc(0, 1).unwrap();
    assert!(g.remove_edge(0, 1).unwrap());
    assert!(!g.remove_edge(0, 1).unwrap());
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn duplicate_edge_is_rejected() {
    let mut g = graph(2);
    g.add_arc(0, 1).unwrap();
    let err = g
        .add_undirected(1, 0)
        .expect_err("a second edge for the pair must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::GraphError(_))
    ));
    assert_eq!(g.num_edges(), 1);
}

#[test]
fn self_loop_is_rejected() {
    let mut g = graph(2);
    let err = g.add_arc(1, 1).expect_err("self-loop must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::GraphError(_))
    ));
}

#[test]
fn out_of_range_node_is_rejected() {
    let mut g = graph(2);
    let err = g
        .add_arc(0, 9)
        .expect_err("out-of-range node must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::IndexOutOfBounds(_))
    ));
}

#[test]
fn set_endpoint_on_missing_edge_is_rejected() {
    let mut g = graph(2);
    let err = g
        .set_endpoint(0, 1, Mark::Arrow)
        .expect_err("orienting a missing edge must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::GraphError(_))
    ));
}

#[test]
fn orient_non_undirected_is_rejected() {
    let mut g = graph(2);
    g.add_arc(0, 1).unwrap();
    let err = g
        .orient(0, 1)
        .expect_err("orienting a non-undirected edge must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::GraphError(_))
    ));
}
