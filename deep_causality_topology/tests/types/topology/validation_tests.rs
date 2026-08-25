/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Topology;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_topology_validation_invalid_grade() {
    let complex = Arc::new(create_triangle_complex());
    // Max dimension is 2
    let grade = 5;
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();

    let result = Topology::new(complex, grade, data, 0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("grade 5 exceeds max dimension"));
}

#[test]
fn test_topology_validation_data_mismatch() {
    let complex = Arc::new(create_triangle_complex());
    // Grade 0 has 3 vertices
    let grade = 0;
    // Data has only 1 element
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();

    let result = Topology::new(complex, grade, data, 0);
    assert!(result.is_err());
    let err = result.unwrap_err();
    // "data length 1 does not match skeleton size 3 for grade 0"
    assert!(
        err.to_string()
            .contains("data length 1 does not match skeleton size 3")
    );
}

#[test]
fn test_topology_validation_cursor_oob() {
    let complex = Arc::new(create_triangle_complex());
    // Grade 0 has 3 vertices
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    // Cursor 10 is OOB
    let cursor = 10;

    let result = Topology::new(complex, grade, data, cursor);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("cursor 10 is out of bounds"));
}

#[test]
fn test_topology_validation_success() {
    let complex = Arc::new(create_triangle_complex());
    // Valid case
    let grade = 0;
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let cursor = 0;

    let result = Topology::new(complex, grade, data, cursor);
    assert!(result.is_ok());
}

#[test]
fn test_topology_validation_grade_zero_on_complex_without_skeletons() {
    // `max_simplex_dimension()` is `skeletons.len().saturating_sub(1)`, so an
    // empty complex reports 0 and grade 0 clears the max-dimension guard. The
    // skeleton-count guard is what rejects it, since skeleton 0 does not exist.
    let complex: Arc<deep_causality_topology::SimplicialComplex<f64>> =
        Arc::new(deep_causality_topology::SimplicialComplex::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();

    let err = Topology::new(complex, 0, data, 0).unwrap_err();
    assert!(
        err.to_string()
            .contains("grade 0 exceeds available skeletons 0"),
        "got {err}"
    );
}
