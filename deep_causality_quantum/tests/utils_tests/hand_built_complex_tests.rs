/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Corner cases of the hand-built complex: (A) the grade past the top, (B) grade zero, (E) a
//! malformed triplet panics at construction.

use deep_causality_homology::{ChainComplex, HomologyField};
use deep_causality_quantum::utils_tests::HandBuiltComplex;

/// A circle: two vertices, two edges each from vertex 0 to vertex 1.
fn circle() -> HandBuiltComplex {
    HandBuiltComplex::new(
        vec![2, 2],
        &[&[(0, 0, -1), (1, 0, 1), (0, 1, -1), (1, 1, 1)]],
    )
}

#[test]
fn test_shapes_and_betti_numbers_of_a_circle() {
    let c = circle();
    assert_eq!(c.num_cells(0), 2);
    assert_eq!(c.num_cells(1), 2);
    assert_eq!(c.num_cells(2), 0);
    assert_eq!(c.max_dim(), 1);
    assert_eq!(
        c.boundary_matrix(0).row_indices().len(),
        1,
        "∂₀ has no rows"
    );
    assert_eq!(
        c.boundary_matrix(2).row_indices().len(),
        3,
        "∂₂ has two rows and no columns"
    );
    assert_eq!(
        c.coboundary_matrix(0).row_indices().len(),
        3,
        "δ₀ is ∂₁ᵀ, two rows"
    );
    assert_eq!(c.betti_number(0), 1);
    assert_eq!(c.betti_number(1), 1);
    assert_eq!(c.betti_number_over(1, HomologyField::Gf2).unwrap(), 1);
}

#[test]
#[should_panic(expected = "one boundary matrix per grade above zero")]
fn test_wrong_boundary_count_panics() {
    let _ = HandBuiltComplex::new(vec![2, 2, 1], &[&[(0, 0, 1)]]);
}

#[test]
#[should_panic(expected = "not well formed")]
fn test_out_of_range_triplet_panics() {
    let _ = HandBuiltComplex::new(vec![1, 1], &[&[(5, 0, 1)]]);
}

/// Two edges on two vertices with a face whose boundary is only one of them: `∂₁ ∂₂` has the
/// entries `(±1, ∓1)` and the fixture is refused.
#[test]
#[should_panic(expected = "not a chain complex")]
fn test_a_nonzero_boundary_square_panics() {
    let _ = HandBuiltComplex::new(
        vec![2, 2, 1],
        &[
            &[(0, 0, -1), (1, 0, 1), (0, 1, -1), (1, 1, 1)],
            &[(0, 0, 1)],
        ],
    );
}

/// The same cells with the face bounded by both edges in opposite orientation: `∂₁ ∂₂ = 0`.
#[test]
fn test_a_zero_boundary_square_builds_and_the_top_coboundary_is_empty() {
    let complex = HandBuiltComplex::new(
        vec![2, 2, 1],
        &[
            &[(0, 0, -1), (1, 0, 1), (0, 1, -1), (1, 1, 1)],
            &[(0, 0, 1), (1, 0, -1)],
        ],
    );
    assert_eq!(complex.max_dim(), 2);
    assert_eq!(complex.coboundary_matrix(2).shape(), (0, 1));
    assert_eq!(complex.coboundary_matrix(usize::MAX).shape(), (0, 0));
    assert_eq!(complex.boundary_matrix(usize::MAX).shape(), (0, 0));
}
