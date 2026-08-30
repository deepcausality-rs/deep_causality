/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_topology::SimplicialComplex;
use deep_causality_topology::utils_tests::{create_test_tensor, create_triangle_complex};

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

    // Hodge ⋆ operators: the helper constructs the complex via `SimplicialComplex::new`
    // with an empty Hodge ⋆ vector and no geometric data. Post-H4 lazy refactor, the
    // accessor surfaces a discriminating error for this state because lazy build
    // requires coords + ambient_dim that the helper does not supply.
    let err = complex.hodge_star_operators().unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("geometric data is not available"));
}

#[test]
fn test_create_test_tensor_helper_yields_zeroed_tensor() {
    // Exercises the public `utils_tests::create_test_tensor` helper, which
    // allocates a 1-D zero tensor of the requested length.
    let t = create_test_tensor::<f64>(5);
    assert_eq!(t.shape(), &[5]);
    assert_eq!(t.len(), 5);
    assert!(t.as_slice().iter().all(|&x| x == 0.0));
}

#[test]
fn test_hodge_star_operators_on_empty_complex_is_empty() {
    // A complex with no skeletons takes the `skeletons.is_empty()` fast path:
    // it caches and returns an empty operator vector instead of erroring on
    // missing geometric data.
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![], vec![], vec![], Vec::new());
    let ops = complex
        .hodge_star_operators()
        .expect("empty complex yields empty operator set, not an error");
    assert!(ops.is_empty());

    // A second call returns the same cached empty vector.
    let ops_again = complex.hodge_star_operators().expect("cached empty set");
    assert!(ops_again.is_empty());
}

#[test]
fn test_simplicial_complex_computed_getters() {
    let complex = create_triangle_complex();

    // total_simplices
    assert_eq!(complex.total_simplices(), 7); // 3 + 3 + 1

    // max_simplex_dimension
    assert_eq!(complex.max_simplex_dimension(), 2); // 0, 1, 2 skeletons present
}
