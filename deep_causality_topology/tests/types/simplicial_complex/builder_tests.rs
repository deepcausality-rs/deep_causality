/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{Simplex, SimplicialComplexBuilder, SimplicialTopology};

#[test]
fn test_builder_new() {
    let builder = SimplicialComplexBuilder::new(2);
    // Cannot inspect builder internals directly easily, but we can verify it builds empty.
    let complex = builder.build::<f64>().unwrap();
    assert_eq!(complex.max_simplex_dimension(), 2); // Complex created with dim 2 capacity
}

#[test]
fn test_builder_add_simplex_success() {
    let mut builder = SimplicialComplexBuilder::new(2);
    // Add vertex
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    // Add edge
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();

    let complex = builder.build::<f64>().unwrap();

    assert!(complex.contains_simplex(&Simplex::new(vec![0])));
    assert!(complex.contains_simplex(&Simplex::new(vec![1]))); // Should be added by closure of edge
    assert!(complex.contains_simplex(&Simplex::new(vec![0, 1])));
}

#[test]
fn test_builder_add_simplex_closure() {
    let mut builder = SimplicialComplexBuilder::new(2);
    // Add only the triangle [0, 1, 2].
    // Expect:
    // Vertices: [0], [1], [2]
    // Edges: [0, 1], [0, 2], [1, 2]
    // Face: [0, 1, 2]
    builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();

    let complex = builder.build::<f64>().unwrap();

    // Check counts
    assert_eq!(complex.num_simplices_at_grade(0).unwrap(), 3);
    assert_eq!(complex.num_simplices_at_grade(1).unwrap(), 3);
    assert_eq!(complex.num_simplices_at_grade(2).unwrap(), 1);

    // Check specific existence
    assert!(complex.contains_simplex(&Simplex::new(vec![0])));
    assert!(complex.contains_simplex(&Simplex::new(vec![0, 2])));
}

#[test]
fn test_builder_add_simplex_error_dim_mismatch() {
    let mut builder = SimplicialComplexBuilder::new(1); // 1D complex
    let res = builder.add_simplex(Simplex::new(vec![0, 1, 2])); // 2D simplex
    assert!(res.is_err());
    match res.err().unwrap().0 {
        deep_causality_topology::TopologyErrorEnum::DimensionMismatch(_) => {}
        _ => panic!("Expected DimensionMismatch error"),
    }
}

#[test]
fn test_builder_add_simplex_error_empty() {
    let mut builder = SimplicialComplexBuilder::new(2);
    let res = builder.add_simplex(Simplex::new(vec![]));
    assert!(res.is_err());
    match res.err().unwrap().0 {
        deep_causality_topology::TopologyErrorEnum::InvalidInput(msg) => {
            assert!(msg.contains("empty simplex"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_builder_build_sorting() {
    let mut builder = SimplicialComplexBuilder::new(1);
    // Add edges in non-sorted order regarding vertices or insertion
    // Simplex::new sorts vertices, so [1, 0] becomes [0, 1].
    // But let's add [2, 3] then [0, 1].
    // Skeleton should be sorted: [0, 1], [2, 3].

    builder.add_simplex(Simplex::new(vec![2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();

    let complex = builder.build::<f64>().unwrap();

    let s0 = complex.get_simplex(1, 0).unwrap();
    let s1 = complex.get_simplex(1, 1).unwrap();

    assert_eq!(s0.vertices(), &vec![0, 1]);
    assert_eq!(s1.vertices(), &vec![2, 3]);
}

#[test]
fn test_builder_operators_correctness() {
    // Triangle [0, 1, 2]
    let mut builder = SimplicialComplexBuilder::new(2);
    builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    // Check Boundary D1 (Edges -> Vertices)
    // Vertices: 0, 1, 2 (Sorted)
    // Edges: (0,1), (0,2), (1,2) (Sorted)

    // Boundary of (0,1) should be 1 - 0.
    // In matrix: Col for (0,1), Row for 1 has +1, Row for 0 has -1.
    // Let's verify via get_simplex indices.
    let idx_edge_01 = complex.skeletons()[1]
        .get_index(&Simplex::new(vec![0, 1]))
        .unwrap();
    let idx_v0 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![0]))
        .unwrap();
    let idx_v1 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![1]))
        .unwrap();

    let d1 = &complex.boundary_operators()[0]; // Maps 1 -> 0
    let val_v1 = d1.get_value_at(idx_v1, idx_edge_01);
    let val_v0 = d1.get_value_at(idx_v0, idx_edge_01);

    // Face (0,1) = {1} - {0} if orientation is standard (-1)^i
    // v0 is removed at index 0 (value 0), sign (-1)^0 = +1? Wait.
    // Boundary formula: sum (-1)^i [v0, ..., no_vi, ..., vn]
    // (0, 1) -> remove 0 (idx 0): (-1)^0 {1} = + {1}
    //        -> remove 1 (idx 1): (-1)^1 {0} = - {0}
    // So {1} has coeff +1, {0} has coeff -1.
    assert_eq!(val_v1, 1);
    assert_eq!(val_v0, -1);
}
