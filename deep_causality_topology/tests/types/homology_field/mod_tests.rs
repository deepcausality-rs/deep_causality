/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The one rank helper, and the field it is a rank over.
//!
//! The cases here are chosen so that a test failing tells you *which* field was used, which is the
//! property the retired thresholded-SVD path could not have.

use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{HomologyField, TopologyErrorEnum};

fn csr(rows: usize, cols: usize, triplets: &[(usize, usize, i8)]) -> CsrMatrix<i8> {
    CsrMatrix::from_triplets(rows, cols, triplets).unwrap()
}

// ---- the two fields agree where they must ------------------------------------------------------

#[test]
fn test_rank_of_the_empty_matrix_is_zero_over_both_fields() {
    let m = csr(0, 0, &[]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 0);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 0);
}

#[test]
fn test_rank_of_the_zero_matrix_is_zero_over_both_fields() {
    let m = csr(3, 3, &[]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 0);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 0);
}

#[test]
fn test_rank_of_the_identity_is_full_over_both_fields() {
    let m = csr(3, 3, &[(0, 0, 1), (1, 1, 1), (2, 2, 1)]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 3);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 3);
}

#[test]
fn test_rank_of_a_triangle_boundary_operator_is_two_over_both_fields() {
    // ∂₁ of a triangle: three vertices, three edges, each column a (−1, +1) pair.
    let m = csr(
        3,
        3,
        &[
            (0, 0, -1),
            (1, 0, 1),
            (1, 1, -1),
            (2, 1, 1),
            (0, 2, 1),
            (2, 2, -1),
        ],
    );
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 2);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 2);
}

// ---- and disagree where the field is the whole question ----------------------------------------

#[test]
fn test_a_matrix_whose_rank_differs_between_the_two_fields() {
    // Rows summing to an even vector: independent over ℚ, dependent mod 2.
    //
    //   [1 1 0]
    //   [0 1 1]
    //   [1 0 1]
    //
    // The determinant is 2, so the rank is 3 over ℚ. Mod 2 the third row is the sum of the first
    // two, so the rank is 2. This is the case G-02 records as producing a wrong `k` for a qLDPC
    // code with an even-weight dependency, and it is why the field cannot be implicit.
    let m = csr(
        3,
        3,
        &[
            (0, 0, 1),
            (0, 1, 1),
            (1, 1, 1),
            (1, 2, 1),
            (2, 0, 1),
            (2, 2, 1),
        ],
    );
    assert_eq!(
        HomologyField::Rational.rank_of(&m).unwrap(),
        3,
        "over ℚ the determinant is 2, so the rows are independent"
    );
    assert_eq!(
        HomologyField::Gf2.rank_of(&m).unwrap(),
        2,
        "mod 2 the third row is the sum of the first two"
    );
}

#[test]
fn test_the_gf2_rank_reduces_coefficients_rather_than_reading_signs() {
    // Over ℚ these two rows are independent; mod 2 they are the same row.
    let m = csr(2, 2, &[(0, 0, 1), (0, 1, 1), (1, 0, -1), (1, 1, 1)]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 2);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 1);
}

// ---- the rank is exact, not thresholded --------------------------------------------------------

#[test]
fn test_a_long_path_incidence_matrix_gets_its_exact_rank() {
    // ∂₁ of a path on 40 vertices: 40 rows, 39 columns, rank 39. The smallest singular value of a
    // path incidence matrix falls off as `2 sin(π/2n)`, so the number the retired path counted
    // approached its `1e-5` threshold as the complex grew. There is no threshold on this path to
    // approach — the entries are integers and an integer is zero or it is not.
    let n = 40usize;
    let mut t = Vec::new();
    for e in 0..(n - 1) {
        t.push((e, e, -1i8));
        t.push((e + 1, e, 1i8));
    }
    let m = csr(n, n - 1, &t);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), n - 1);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), n - 1);
}

#[test]
fn test_the_signature_admits_no_tolerance() {
    // A rank taken over a field is an exact question. `rank_of` takes the field and nothing else:
    // there is no epsilon to pass, so there is none to get wrong. This test exists to fail at
    // compile time if one is ever added.
    let m = csr(2, 2, &[(0, 0, 1), (1, 1, 1)]);
    let f: fn(HomologyField, &CsrMatrix<i8>) -> _ = HomologyField::rank_of;
    assert_eq!(f(HomologyField::Rational, &m).unwrap(), 2);
}

#[test]
fn test_a_rectangular_matrix_ranks_by_its_smaller_side() {
    let m = csr(2, 5, &[(0, 0, 1), (0, 3, 2), (1, 1, 1), (1, 4, 3)]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 2);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 2);

    let m = csr(5, 2, &[(0, 0, 1), (3, 0, 2), (1, 1, 1), (4, 1, 3)]);
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 2);
    assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 2);
}

// ---- widening is what keeps the overflow guard from firing --------------------------------------

#[test]
fn test_extreme_i8_entries_do_not_overflow_the_widened_elimination() {
    // The fraction-free intermediates are products of entries. At `i64::MAX` the very first one
    // overflows, and the point is that it says so instead of returning a rank computed from a
    // wrapped value. `i8` entries cannot reach this, which is why `rank_of` widens to `i64`;
    // the guard is the linear crate's and this pins that it survives the conversion.
    let big = i8::MAX;
    let m = csr(
        3,
        3,
        &[
            (0, 0, big),
            (0, 1, big),
            (0, 2, big),
            (1, 0, big),
            (1, 1, -big),
            (1, 2, big),
            (2, 0, big),
            (2, 1, big),
            (2, 2, -big),
        ],
    );
    // `i64` absorbs `i8` products with room to spare, so this one succeeds — which is the
    // measurement, not an assumption: widening is what makes the overflow unreachable in practice.
    assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 3);
}

// ---- the field is a value, and behaves like one ------------------------------------------------

#[test]
fn test_the_field_is_copy_and_comparable() {
    let f = HomologyField::Gf2;
    let g = f;
    assert_eq!(f, g);
    assert_ne!(HomologyField::Rational, HomologyField::Gf2);
}

#[test]
fn test_the_field_names_itself_in_debug() {
    assert_eq!(format!("{:?}", HomologyField::Rational), "Rational");
    assert_eq!(format!("{:?}", HomologyField::Gf2), "Gf2");
}

#[test]
fn test_a_linear_failure_arrives_as_a_linear_algebra_error() {
    // There is no `i8` boundary matrix that overflows, so this checks the wiring rather than the
    // arithmetic: the variant a linear failure converts into is the one that names it.
    let e: deep_causality_topology::TopologyError =
        deep_causality_linear::LinearError::NotSquare((2, 3)).into();
    assert!(matches!(e.0, TopologyErrorEnum::LinearAlgebraError(_)));
    assert!(e.to_string().contains("Linear algebra error"));
}
