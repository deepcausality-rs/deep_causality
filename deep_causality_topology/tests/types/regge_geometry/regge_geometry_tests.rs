/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::Metric;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{ReggeGeometry, Simplex, SimplicialComplex, Skeleton};
use std::vec;

#[test]
fn test_regge_geometry_metric_at_triangle() {
    let complex = create_triangle_complex();

    // Edge lengths for a right-angle triangle (0,1) length 1, (0,2) length 1, (1,2) length sqrt(2)
    // Edges in create_triangle_complex are (0,1), (0,2), (1,2)
    let edge_lengths =
        CausalTensor::new(vec![1.0, 1.0, std::f64::consts::SQRT_2], vec![3]).unwrap();
    let regge_geometry = ReggeGeometry::new(edge_lengths);

    // Test for a 0-simplex (vertex 0)
    let metric_at_vertex = regge_geometry.metric_at(&complex, 0, 0);
    assert_eq!(metric_at_vertex, Metric::Euclidean(0));

    // Test for a 1-simplex (edge (0,1) - index 0)
    let metric_at_edge = regge_geometry.metric_at(&complex, 1, 0);
    assert_eq!(metric_at_edge, Metric::Euclidean(1));

    // Test for a 2-simplex (face (0,1,2) - index 0)
    let metric_at_face = regge_geometry.metric_at(&complex, 2, 0);
    assert_eq!(metric_at_face, Metric::Euclidean(2));
}

#[test]
#[should_panic(expected = "Edge not found in 1-skeleton")]
fn test_regge_geometry_metric_at_invalid_edge_panic() {
    // Manually create a complex missing an edge
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    // Only add two edges, omitting (1,2) which would be the 3rd edge
    let edges = vec![Simplex::new(vec![0, 1]), Simplex::new(vec![0, 2])];
    let skeleton_1 = Skeleton::new(1, edges);

    let faces = vec![Simplex::new(vec![0, 1, 2])];
    let skeleton_2 = Skeleton::new(2, faces);

    // Minimal boundary operators (they don't need to be correct for this specific panic test)
    let complex = SimplicialComplex::new(
        vec![skeleton_0, skeleton_1, skeleton_2],
        vec![], // Empty boundary ops
        vec![], // Empty coboundary ops
        vec![],
    );

    let edge_lengths =
        CausalTensor::new(vec![1.0, 1.0, std::f64::consts::SQRT_2], vec![3]).unwrap();
    let regge_geometry = ReggeGeometry::new(edge_lengths);

    // Try to compute metric for the (0,1,2) face. The edge (1,2) will not be found.
    regge_geometry.metric_at(&complex, 2, 0);
}

#[test]
fn test_regge_geometry_metric_at_collinear_triangle_is_degenerate() {
    // Vertices 0, 1, 2 sit at 0, 1 and 2 on a line: d(0,1) = 1, d(0,2) = 2,
    // d(1,2) = 1. The Gram matrix built from those squared distances is
    // [[1, 2], [2, 4]], which has rank 1, so the signature is (p, q, r) =
    // (1, 0, 1) and `Metric::from_signature` names a single zero direction PGA.
    let complex = create_triangle_complex();
    // Edge order in the 1-skeleton is (0,1), (0,2), (1,2).
    let edge_lengths = CausalTensor::new(vec![1.0, 2.0, 1.0], vec![3]).unwrap();
    let geometry = ReggeGeometry::new(edge_lengths);

    assert_eq!(geometry.metric_at(&complex, 2, 0), Metric::PGA(2));
}
