/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Topology;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_topology_display() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let cursor = 1;
    let topology = Topology::new(complex, grade, data, cursor);

    let display_str = format!("{}", topology);

    // Expected format check
    assert!(display_str.contains("CausalTopology:"));
    assert!(display_str.contains("Grade: 0"));
    assert!(display_str.contains("Cursor: 1"));
    assert!(display_str.contains("Data:"));
}
