/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Topology;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_topology_clone_shallow() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topology = Topology::new(complex, grade, data, 0).unwrap();

    let shallow_clone = topology.clone_shallow();

    // Complex should share the same Arc
    assert!(Arc::ptr_eq(shallow_clone.complex(), topology.complex()));

    assert_eq!(shallow_clone.grade(), topology.grade());
    // Data in CausalTensor is generally owned, so equality check compares values.
    assert_eq!(shallow_clone.data(), topology.data());

    assert_eq!(shallow_clone.cursor(), topology.cursor());
}
