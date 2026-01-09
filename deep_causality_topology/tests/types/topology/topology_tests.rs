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
