/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{PointCloud, Topology};
use std::sync::Arc;

/// Helper to create a simple topology
fn create_simple_topology() -> Topology<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = point_cloud.triangulate(1.2).unwrap();

    // Values for 3 vertices (grade 0)
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();

    Topology::new(Arc::new(complex), 0, data, 0).unwrap()
}

#[test]
fn test_topology_cup_product() {
    let complex = Arc::new(create_triangle_complex());

    // 0-form: scalar field on vertices
    let data0 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topo0 = Topology::new(complex.clone(), 0, data0, 0).unwrap();

    // 1-form: vector field on edges
    let data1 = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo1 = Topology::new(complex.clone(), 1, data1, 0).unwrap();

    // cup product of 0-form and 1-form should result in a 1-form
    let cup_product_result = topo0.cup_product(&topo1).unwrap();

    assert_eq!(cup_product_result.grade(), 1);
    assert_eq!(
        cup_product_result.complex().skeletons()[1]
            .simplices()
            .len(),
        3
    ); // 3 edges

    // Expected values: (v0 * e01, v0 * e02, v1 * e12)
    // Front face (0..0) for 0-form: simplex (0), (0), (1)
    // Back face (0..1) for 1-form: simplex (0,1), (0,2), (1,2)

    // Simplex 0 (0,1): front (0), back (0,1)
    // topo0 data index of Simplex(0) is 0, value 1.0
    // topo1 data index of Simplex(0,1) is 0, value 0.5
    // Result for Simplex(0,1) should be 1.0 * 0.5 = 0.5

    // Simplex 1 (0,2): front (0), back (0,2)
    // topo0 data index of Simplex(0) is 0, value 1.0
    // topo1 data index of Simplex(0,2) is 1, value 1.5
    // Result for Simplex(0,2) should be 1.0 * 1.5 = 1.5

    // Simplex 2 (1,2): front (1), back (1,2)
    // topo0 data index of Simplex(1) is 1, value 2.0
    // topo1 data index of Simplex(1,2) is 2, value 2.5
    // Result for Simplex(1,2) should be 2.0 * 2.5 = 5.0

    let expected_data = CausalTensor::new(vec![0.5, 1.5, 5.0], vec![3]).unwrap();
    assert_eq!(cup_product_result.data(), &expected_data);
}

#[test]
fn test_topology_cup_product_missing_data_self() {
    let complex = Arc::new(create_triangle_complex());

    // 0-form with insufficient data (only 1 value, but 3 vertices needed)
    let data0 = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    // This should now fail validation
    let topo0_result = Topology::new(complex.clone(), 0, data0, 0);
    assert!(topo0_result.is_err());

    // Check error message contains appropriate info
    if let Err(e) = topo0_result {
        let msg = e.to_string();
        assert!(msg.contains("data length 1 does not match skeleton size 3"));
    }
}

#[test]
fn test_topology_cup_product_missing_data_other() {
    let complex = Arc::new(create_triangle_complex());

    // 0-form with correct data
    let data0 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let _topo0 = Topology::new(complex.clone(), 0, data0, 0).unwrap();

    // 1-form with insufficient data (only 1 value, but 3 edges needed)
    let data1 = CausalTensor::new(vec![0.5], vec![1]).unwrap();
    // This should now fail validation
    let topo1_result = Topology::new(complex.clone(), 1, data1, 0);
    assert!(topo1_result.is_err());
}

#[test]
fn test_topology_cup_product_rejects_complex_mismatch() {
    // Two distinct Arc-allocated complexes — same shape but different identity.
    let complex_a = Arc::new(create_triangle_complex());
    let complex_b = Arc::new(create_triangle_complex());

    let data_a = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let data_b = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo_a = Topology::new(complex_a, 0, data_a, 0).unwrap();
    let topo_b = Topology::new(complex_b, 1, data_b, 0).unwrap();

    let err = topo_a.cup_product(&topo_b).unwrap_err();
    assert!(err.to_string().contains("Complex Mismatch"));
}

#[test]
fn test_topology_cup_product_overflow_grade_returns_zero_field() {
    // Triangle complex has max simplex dimension 2 (a single 2-simplex).
    // A 2 ⌣ 1 cup product produces grade r = 3, which exceeds the max → zero-length
    // result tensor (no 3-skeleton on a triangle).
    let complex = Arc::new(create_triangle_complex());

    let data2 = CausalTensor::new(vec![7.0], vec![1]).unwrap();
    let topo2 = Topology::new(complex.clone(), 2, data2, 0).unwrap();

    let data1 = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo1 = Topology::new(complex.clone(), 1, data1, 0).unwrap();

    let result = topo2.cup_product(&topo1).unwrap();
    assert_eq!(result.grade(), 3);
    // r=3 ≥ skeletons.len()=3 → zero_len = 0 (else branch).
    assert_eq!(result.data().as_slice().len(), 0);
}

#[test]
fn test_topology_cup_product_simplex_not_found_returns_error() {
    use deep_causality_sparse::CsrMatrix;
    use deep_causality_topology::{Simplex, SimplicialComplex, Skeleton};

    // Build a complex with mismatched skeletons: the 1-skeleton lists an edge that is
    // NOT a sub-simplex of the only 2-simplex. The cup-product front-face lookup will
    // succeed for the 0-skeleton but the back-face Simplex([1,2]) is absent from the
    // 1-skeleton → SimplexNotFound.
    //
    // Triangle simplex: (0, 1, 2). 1-skeleton intentionally only contains (0, 1) and
    // (0, 2) — leaving (1, 2) missing.
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let edges = vec![Simplex::new(vec![0, 1]), Simplex::new(vec![0, 2])];
    let faces = vec![Simplex::new(vec![0, 1, 2])];
    let skel = vec![
        Skeleton::new(0, vertices),
        Skeleton::new(1, edges),
        Skeleton::new(2, faces),
    ];

    let d1 =
        CsrMatrix::from_triplets(3, 2, &[(1, 0, 1i8), (0, 0, -1), (2, 1, 1), (0, 1, -1)]).unwrap();
    let d2: CsrMatrix<i8> = CsrMatrix::with_capacity(2, 1, 0);

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skel, vec![d1, d2], vec![], Vec::new());
    let complex = Arc::new(complex);

    let data0 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let data2 = CausalTensor::new(vec![7.0], vec![1]).unwrap();
    let topo0 = Topology::new(complex.clone(), 0, data0, 0).unwrap();
    let topo2 = Topology::new(complex.clone(), 2, data2, 0).unwrap();

    // 0 ⌣ 2: front face is [v0..v0]=[0] (in 0-skeleton, found), back face is [v0..v2]
    // — but we want to trigger SimplexNotFound, so use 1 ⌣ 1 to hit a missing edge.
    // 1 ⌣ 1 → r=2, sum over 2-simplices. For (0,1,2): front [0,1] (present), back [1,2]
    // (MISSING from 1-skeleton). Triggers the `back` ok_or branch.
    let data1 = CausalTensor::new(vec![0.5, 1.5], vec![2]).unwrap();
    let topo1a = Topology::new(complex.clone(), 1, data1.clone(), 0).unwrap();
    let topo1b = Topology::new(complex.clone(), 1, data1, 0).unwrap();
    let err = topo1a.cup_product(&topo1b).unwrap_err();
    // SimplexNotFound is a wrapper struct around an inner enum; check via Display.
    assert!(err.to_string().contains("Simplex not found"));

    drop(topo0);
    drop(topo2);
}

// =============================================================================
// Constructor and validation tests
// =============================================================================

#[test]
fn test_topology_new_success() {
    let topology = create_simple_topology();
    assert_eq!(topology.grade(), 0);
}

#[test]
fn test_topology_grade_getter() {
    let topology = create_simple_topology();
    assert_eq!(topology.grade(), 0, "Grade should be 0 for vertex data");
}

#[test]
fn test_topology_data_getter() {
    let topology = create_simple_topology();
    let data = topology.data();
    assert_eq!(data.len(), 3, "Should have 3 data values");
}

#[test]
fn test_topology_complex_getter() {
    let topology = create_simple_topology();
    let complex = topology.complex();
    assert!(!complex.skeletons().is_empty());
}

// =============================================================================
// Cup product edge cases
// =============================================================================

#[test]
fn test_topology_cursor() {
    let topology = create_simple_topology();
    assert_eq!(topology.cursor(), 0);
}
