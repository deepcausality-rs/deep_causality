/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Topology;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_topology_getters() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 1;
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let cursor = 1;

    let topology = Topology::new(complex.clone(), grade, data.clone(), cursor).unwrap();

    assert_eq!(topology.complex(), &complex);
    assert_eq!(topology.grade(), grade);
    assert_eq!(topology.data(), &data);
    assert_eq!(topology.cursor(), cursor);
}
