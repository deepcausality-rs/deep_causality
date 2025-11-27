/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Topology;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_topology_new() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0; // Vertices
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let cursor = 0;

    let topology = Topology::new(complex.clone(), grade, data.clone(), cursor);

    assert_eq!(topology.complex(), &complex);
    assert_eq!(topology.grade(), grade);
    assert_eq!(topology.data(), &data);
    assert_eq!(topology.cursor(), cursor);
}

#[test]
fn test_topology_getters() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 1;
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let cursor = 1;

    let topology = Topology::new(complex.clone(), grade, data.clone(), cursor);

    assert_eq!(topology.complex(), &complex);
    assert_eq!(topology.grade(), grade);
    assert_eq!(topology.data(), &data);
    assert_eq!(topology.cursor(), cursor);
}

#[test]
fn test_topology_clone_shallow() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topology = Topology::new(complex, grade, data, 0);

    let shallow_clone = topology.clone_shallow();
    assert_eq!(shallow_clone.complex(), topology.complex());
    assert_eq!(shallow_clone.grade(), topology.grade());
    assert_eq!(shallow_clone.data(), topology.data());
    assert_eq!(shallow_clone.cursor(), topology.cursor());
}

#[test]
fn test_topology_cup_product() {
    let complex = Arc::new(create_triangle_complex());

    // 0-form: scalar field on vertices
    let data0 = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topo0 = Topology::new(complex.clone(), 0, data0, 0);

    // 1-form: vector field on edges
    let data1 = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo1 = Topology::new(complex.clone(), 1, data1, 0);

    // cup product of 0-form and 1-form should result in a 1-form
    let cup_product_result = topo0.cup_product(&topo1);

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
#[should_panic(expected = "Cup product dimension exceeds complex dimension")]
fn test_topology_cup_product_dim_exceeds() {
    let complex = Arc::new(create_triangle_complex()); // Max dim 2

    // 2-form: field on faces
    let data2 = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topo2_a = Topology::new(complex.clone(), 2, data2.clone(), 0);
    let topo2_b = Topology::new(complex.clone(), 2, data2, 0);

    // cup product of 2-form and 2-form should result in a 4-form, but max dim is 2.
    topo2_a.cup_product(&topo2_b);
}

#[test]
fn test_topology_display() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let cursor = 1;
    let topology = Topology::new(complex, grade, data, cursor);

    let expected_display = format!(
        "CausalTopology:\n  Grade: {}\n  Cursor: {}\n  Data: {}\n",
        grade,
        cursor,
        CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap()
    );
    assert_eq!(format!("{}", topology), expected_display);
}
