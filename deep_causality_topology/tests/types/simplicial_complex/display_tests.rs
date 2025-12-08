/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_topology::utils_tests::create_triangle_complex;

#[test]
fn test_simplicial_complex_display() {
    let complex = create_triangle_complex();
    let display_str = format!("{}", complex);

    assert!(display_str.contains("CausalComplex:"));
    assert!(display_str.contains("Number of Skeletons: 3"));
    assert!(display_str.contains("Skeleton 0: (Dim: 0, Num Simplices: 3)"));
    assert!(display_str.contains("Skeleton 1: (Dim: 1, Num Simplices: 3)"));
    assert!(display_str.contains("Skeleton 2: (Dim: 2, Num Simplices: 1)"));
    assert!(display_str.contains("Number of Boundary Operators: 2"));
    assert!(display_str.contains("Boundary Operator 0: (Shape: 3x3, Num Non-Zeros: 6)"));
    assert!(display_str.contains("Boundary Operator 1: (Shape: 3x1, Num Non-Zeros: 3)"));
    assert!(display_str.contains("Number of Coboundary Operators: 0"));
}
