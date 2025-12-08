/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_topology::utils_tests::create_triangle_complex;

#[test]
fn test_simplicial_complex_getters() {
    let complex = create_triangle_complex();
    // Skeletons
    assert_eq!(complex.skeletons()[0].dim(), 0);
    assert_eq!(complex.skeletons()[0].simplices().len(), 3); // 3 vertices
    assert_eq!(complex.skeletons()[1].dim(), 1);
    assert_eq!(complex.skeletons()[1].simplices().len(), 3); // 3 edges
    assert_eq!(complex.skeletons()[2].dim(), 2);
    assert_eq!(complex.skeletons()[2].simplices().len(), 1); // 1 face

    // Boundary operators
    assert_eq!(complex.boundary_operators().len(), 2);
    assert_eq!(complex.boundary_operators()[0].shape(), (3, 3)); // B1: 3x3 (vertices x edges)
    assert_eq!(complex.boundary_operators()[1].shape(), (3, 1)); // B2: 3x1 (edges x faces)

    // Coboundary operators (empty for now based on helper)
    assert!(complex.coboundary_operators().is_empty());

    // Hodge Star operators (empty for now based on helper)
    assert!(complex.hodge_star_operators().is_empty());
}

#[test]
fn test_simplicial_complex_computed_getters() {
    let complex = create_triangle_complex();

    // total_simplices
    assert_eq!(complex.total_simplices(), 7); // 3 + 3 + 1

    // max_simplex_dimension
    assert_eq!(complex.max_simplex_dimension(), 2); // 0, 1, 2 skeletons present
}
