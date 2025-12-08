/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::BaseTopology;
use deep_causality_topology::utils_tests::create_triangle_complex;

#[test]
fn test_simplicial_complex_dimension() {
    let complex = create_triangle_complex();
    // Triangle complex has 3 skeletons (0, 1, 2). Max dimension is 2.
    assert_eq!(complex.dimension(), 2);
}

#[test]
fn test_simplicial_complex_len() {
    let complex = create_triangle_complex();
    // 3 vertices + 3 edges + 1 face = 7 simplices
    assert_eq!(complex.len(), 7);
}

#[test]
fn test_simplicial_complex_num_elements_at_grade() {
    let complex = create_triangle_complex();

    // Grade 0: 3 vertices
    assert_eq!(complex.num_elements_at_grade(0), Some(3));

    // Grade 1: 3 edges
    assert_eq!(complex.num_elements_at_grade(1), Some(3));

    // Grade 2: 1 face
    assert_eq!(complex.num_elements_at_grade(2), Some(1));

    // Grade 3: None
    assert_eq!(complex.num_elements_at_grade(3), None);
}
