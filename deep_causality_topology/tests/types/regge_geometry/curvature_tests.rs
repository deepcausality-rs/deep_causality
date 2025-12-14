/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::f64::consts::PI;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, ReggeGeometry, Simplex, SimplicialComplexBuilder};

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_2d_flat_triangle_boundary() {
    let mut builder = SimplicialComplexBuilder::new(2);
    builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap(); // Will implicitly add edges and vertices
    let complex = builder.build().unwrap();

    // Metric: 3 edges.
    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();

    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry
        .calculate_ricci_curvature(&complex)
        .expect("Calculation failed");

    // 2D -> Curvature at vertices (0)
    let num_verts = complex.num_elements_at_grade(0).unwrap();
    assert_eq!(curvature.shape(), vec![num_verts]);

    // Boundary vertices have 0 curvature
    for &val in curvature.data() {
        assert_eq!(val, 0.0);
    }
}

#[test]
fn test_3d_flat_tetrahedron_boundary() {
    let mut builder = SimplicialComplexBuilder::new(3);
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    let complex = builder.build().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();

    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry
        .calculate_ricci_curvature(&complex)
        .expect("Calculation failed");

    // 3D -> Curvature at edges (bones are n-2 = 1)
    let num_bones = complex.num_elements_at_grade(1).unwrap();
    assert_eq!(curvature.shape(), vec![num_bones]);

    for &val in curvature.data() {
        assert_eq!(val, 0.0);
    }
}

#[test]
fn test_2d_internal_flat_hexagon() {
    let mut builder = SimplicialComplexBuilder::new(2);
    // Center 0.
    // 6 Triangles around 0.
    let indices = [1, 2, 3, 4, 5, 6];
    for i in 0..6 {
        let v1 = indices[i];
        let v2 = indices[(i + 1) % 6];
        builder.add_simplex(Simplex::new(vec![0, v1, v2])).unwrap();
    }
    let complex = builder.build().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry.calculate_ricci_curvature(&complex).unwrap();

    // Check center (0)
    let idx_0 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![0]))
        .unwrap();
    let k_0 = curvature.data()[idx_0];

    assert!(
        k_0.abs() < 1e-6,
        "Flat hexagon center should have 0 curvature. Got {}",
        k_0
    );
}

#[test]
fn test_2d_internal_positive_curvature_pentagon() {
    let mut builder = SimplicialComplexBuilder::new(2);
    let indices = [1, 2, 3, 4, 5];
    for i in 0..5 {
        let v1 = indices[i];
        let v2 = indices[(i + 1) % 5];
        builder.add_simplex(Simplex::new(vec![0, v1, v2])).unwrap();
    }
    let complex = builder.build().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry.calculate_ricci_curvature(&complex).unwrap();

    let idx_0 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![0]))
        .unwrap();
    let k_0 = curvature.data()[idx_0];

    // 360 - 5*60 = 60 deg = PI/3
    let expected = PI / 3.0;
    assert!((k_0 - expected).abs() < 1e-6, "Expected PI/3, got {}", k_0);
}

#[test]
fn test_dimension_mismatch_error() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build().unwrap();

    let tensor = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    assert!(matches!(
        res.err().unwrap().0,
        deep_causality_topology::TopologyErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_manifold_error_triangle_inequality() {
    // We need an INTERNAL bone for calculation to occur.
    // In 3D, create 3 tets around edge (0,1):
    // T1: 0,1,2,3
    // T2: 0,1,3,4
    // T3: 0,1,4,2
    // Edge (0,1) is shared by faces (0,1,2), (0,1,3), (0,1,4).
    // All these faces are shared by 2 tets, so they are internal.
    // Thus edge (0,1) is internal.

    let mut builder = SimplicialComplexBuilder::new(3);
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 3, 4])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 4, 2])).unwrap();
    let complex = builder.build().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let mut lengths = vec![1.0; num_edges];

    // Set edge 0-2 to 10.0, while 0-1 and 1-2 are 1.0.
    // Triangle (0,1,2) will violate inequality Check.
    // We need to find index of 0-2.
    if let Some(idx) = complex.skeletons()[1].get_index(&Simplex::new(vec![0, 2])) {
        lengths[idx] = 10.0;
    } else {
        panic!("Edge 0-2 not found");
    }

    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err(), "Should return error for impossible triangle");

    let err = res.err().unwrap();
    if let deep_causality_topology::TopologyErrorEnum::ManifoldError(msg) = err.0 {
        assert!(msg.contains("Triangle inequality") || msg.contains("Degenerate"));
    } else {
        panic!("Expected ManifoldError, got {:?}", err);
    }
}

#[test]
fn test_curvature_missing_boundary_operators() {
    // Manually construct a complex without boundary operators
    // Vertices {0, 1, 2}
    // Face {0, 1, 2}
    let s0 = deep_causality_topology::Skeleton::new(
        0,
        vec![
            Simplex::new(vec![0]),
            Simplex::new(vec![1]),
            Simplex::new(vec![2]),
        ],
    );
    let s1 = deep_causality_topology::Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![1, 2]),
        ],
    );
    let s2 = deep_causality_topology::Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]);

    // Empty boundary ops
    let complex =
        deep_causality_topology::SimplicialComplex::new(vec![s0, s1, s2], vec![], vec![], vec![]);

    let num_edges = 3;
    let tensor = CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    if let deep_causality_topology::TopologyErrorEnum::InvalidInput(msg) = res.err().unwrap().0 {
        assert!(msg.contains("boundary operators"));
    } else {
        panic!("Expected InvalidInput for missing boundary operators");
    }
}

#[test]
fn test_3d_degenerate_tetrahedron_face() {
    // Degenerate face: 3 vertices collinear.
    // 0, 1, 2 collinear.
    // Tet T1: 0, 1, 2, 3.
    // Edges of face (0,1,2): (0,1)=1 (Bone), (1,2)=1, (0,2)=2.
    // Area(0,1,2) = 0 because 1+1=2 (Triangle inequality satisfied but area zero/degenerate).
    // Or actually 1+1=2 is degenerate triangle.

    // We need edge (0,1) to be internal to trigger calculation involving face (0,1,2).
    // So we need other tets around (0,1).
    // T2: 0,1,3,4.
    // T3: 0,1,4,2.

    let mut builder = SimplicialComplexBuilder::new(3);
    // Add all 3 tets
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 3, 4])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 4, 2])).unwrap();

    let complex = builder.build().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let mut lengths = vec![1.0; num_edges];

    // Set (0,2) length to 2.0.
    // (0,1) is 1.0 default. (1,2) is 1.0 default.
    // So face (0,1,2) has sides 1, 1, 2 -> Area 0.
    let edge_02 = Simplex::new(vec![0, 2]);
    if let Some(idx) = complex.skeletons()[1].get_index(&edge_02) {
        lengths[idx] = 2.0;
    } else {
        // If (0,2) is not explicitly an edge?
        // Simplex(0,1,2,3) contains (0,2). It should be there.
        panic!("Edge (0,2) not found in complex");
    }

    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    let err = res.err().unwrap();
    let err_msg = format!("{:?}", err);

    // Check for specific error
    if !err_msg.contains("Degenerate face")
        && !err_msg.contains("Triangle inequality")
        && !err_msg.contains("Tetrahedron inequality violated")
    {
        panic!("Test Failed. Unexpected error message: {}", err_msg);
    }
}
