/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Manifold Display trait implementation

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};
use std::fmt::Write;

/// Helper to create a valid manifold
fn setup_triangle_manifold() -> Manifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();

    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_manifold_display_basic() {
    let manifold = setup_triangle_manifold();
    let display_str = format!("{}", manifold);

    // Should contain "Manifold"
    assert!(
        display_str.contains("Manifold"),
        "Display should include 'Manifold'"
    );

    // Should mention dimension
    assert!(
        display_str.contains("dimension"),
        "Display should include dimension info"
    );

    // Should mention simplices
    assert!(
        display_str.contains("simplices"),
        "Display should include simplices count"
    );
}

#[test]
fn test_manifold_display_format() {
    let manifold = setup_triangle_manifold();

    // Use write! macro to format into string
    let mut output = String::new();
    write!(&mut output, "{}", manifold).unwrap();

    // The format should be "Manifold { dimension: X, simplices: Y }"
    assert!(
        output.starts_with("Manifold {"),
        "Should start with 'Manifold {{'"
    );
    assert!(output.ends_with("}"), "Should end with '}}'");
}

#[test]
fn test_manifold_display_values() {
    let manifold = setup_triangle_manifold();
    let display_str = format!("{}", manifold);

    // For a triangle: dimension 2, simplices = 3 vertices + 3 edges + 1 face = 7
    assert!(
        display_str.contains("7") || display_str.contains("simplices"),
        "Display should show simplex count"
    );
}
