/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{MixedGraph, TopologyError, TopologyErrorEnum};

/// Helper: a structural 3-node graph (unit payload), no edges.
fn structural_graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn new_success_with_payload() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let g = MixedGraph::new(3, data, 0).unwrap();
    assert_eq!(g.num_vertices(), 3);
    assert_eq!(g.num_edges(), 0);
    assert_eq!(g.cursor(), 0);
    assert_eq!(g.data().as_slice(), &[1.0, 2.0, 3.0]);
}

#[test]
fn new_success_structural_unit_payload() {
    // A scalar-free CPDAG: payload is the unit type.
    let g = structural_graph(4);
    assert_eq!(g.num_vertices(), 4);
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn new_rejects_zero_vertices() {
    let data = CausalTensor::new(Vec::<f64>::new(), vec![0]).unwrap();
    let err = MixedGraph::new(0, data, 0).expect_err("zero vertices must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::InvalidInput(_))
    ));
}

#[test]
fn new_rejects_data_length_mismatch() {
    let data = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let err = MixedGraph::new(3, data, 0).expect_err("data/vertex mismatch must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::InvalidInput(_))
    ));
}

#[test]
fn new_rejects_out_of_range_cursor() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let err = MixedGraph::new(3, data, 5).expect_err("out-of-range cursor must be rejected");
    assert!(matches!(
        err,
        TopologyError(TopologyErrorEnum::IndexOutOfBounds(_))
    ));
}

#[test]
fn getters_expose_fields() {
    let data = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let g = MixedGraph::new(2, data, 1).unwrap();
    assert_eq!(g.num_vertices(), 2);
    assert_eq!(g.cursor(), 1);
    assert!(g.edges().is_empty());
    assert!(!g.has_edge(0, 1));
    assert_eq!(g.edge_marks(0, 1), None);
    assert_eq!(g.endpoint_mark(0, 1), None);
    assert_eq!(g.edge_kind(0, 1), None);
}

#[test]
fn clone_shallow_resets_cursor() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let g = MixedGraph::new(3, data, 2).unwrap();
    let c = g.clone_shallow();
    assert_eq!(c.cursor(), 0);
    assert_eq!(c.num_vertices(), 3);
    assert_eq!(c.data().as_slice(), g.data().as_slice());
}

#[test]
fn display_reports_vertex_and_edge_counts() {
    let g = structural_graph(3);
    assert_eq!(format!("{g}"), "MixedGraph { vertices: 3, edges: 0 }");
}
