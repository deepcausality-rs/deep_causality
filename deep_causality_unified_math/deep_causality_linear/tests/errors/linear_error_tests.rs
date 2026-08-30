/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{DenseMatrix, LinearError, LinearErrorEnum, MatrixBuild, MatrixView};

#[test]
fn test_index_out_of_bounds_names_the_position_and_the_shape() {
    let e = LinearError::IndexOutOfBounds((3, 1), (2, 2));
    let s = format!("{e}");
    assert!(s.contains("3"), "must name the offending row: {s}");
    assert!(s.contains('1'), "must name the offending column: {s}");
    assert!(
        s.contains("2x2"),
        "must name the shape it was checked against: {s}"
    );
}

#[test]
fn test_shape_mismatch_names_both_shapes() {
    let e = LinearError::ShapeMismatch((2, 3), (4, 5));
    let s = format!("{e}");
    assert!(s.contains("2x3") && s.contains("4x5"), "{s}");
}

#[test]
fn test_inner_dimension_mismatch_names_both_dimensions() {
    let e = LinearError::InnerDimensionMismatch(3, 7);
    let s = format!("{e}");
    assert!(s.contains('3') && s.contains('7'), "{s}");
}

#[test]
fn test_length_mismatch_names_expected_and_found() {
    let e = LinearError::LengthMismatch(4, 9);
    let s = format!("{e}");
    assert!(s.contains('4') && s.contains('9'), "{s}");
}

#[test]
fn test_not_square_names_the_shape() {
    let e = LinearError::NotSquare((2, 5));
    assert!(format!("{e}").contains("2x5"));
}

#[test]
fn test_singular_names_the_column_elimination_stopped_at() {
    let e = LinearError::Singular(2);
    assert!(format!("{e}").contains('2'));
}

#[test]
fn test_zero_diagonal_names_the_index() {
    let e = LinearError::ZeroDiagonal(5);
    assert!(format!("{e}").contains('5'));
}

#[test]
fn test_wrong_triangle_names_the_first_offending_position() {
    let e = LinearError::WrongTriangle((0, 2));
    let s = format!("{e}");
    assert!(s.contains('0') && s.contains('2'), "{s}");
}

#[test]
fn test_not_binary_names_the_position_so_the_caller_need_not_rescan() {
    let e = LinearError::NotBinary((1, 3));
    let s = format!("{e}");
    assert!(s.contains('1') && s.contains('3'), "{s}");
    assert!(s.contains("GF(2)"), "must say what could not hold it: {s}");
}

#[test]
fn test_overflow_names_the_operation_that_produced_it() {
    let e = LinearError::Overflow("fraction-free determinant");
    let s = format!("{e}");
    assert!(
        s.contains("fraction-free determinant"),
        "must name the operation: {s}"
    );
    assert!(s.contains("Overflow"), "{s}");
}

#[test]
fn test_empty_matrix_says_so() {
    assert!(format!("{}", LinearError::EmptyMatrix()).contains("Empty"));
}

#[test]
fn test_errors_compare_and_clone() {
    let a = LinearError::Singular(1);
    assert_eq!(a.clone(), a);
    assert_ne!(a, LinearError::Singular(2));
    assert_ne!(a, LinearError::EmptyMatrix());
}

#[test]
fn test_errors_are_debug() {
    let s = format!("{:?}", LinearError::NotSquare((1, 2)));
    assert!(s.contains("NotSquare"), "{s}");
}

#[test]
fn test_not_positive_definite_names_the_diagonal_index_and_keeps_invertibility_open() {
    // `diag(1, -1)` is invertible and indefinite, so "no Cholesky factor" and "no inverse" are
    // different failures. The message has to say which one this is.
    let e = LinearError::NotPositiveDefinite(3);
    let s = format!("{e}");
    assert!(s.contains('3'), "must name the diagonal index: {s}");
    assert!(
        s.contains("positive definite"),
        "must name the classification: {s}"
    );
    assert!(
        s.contains("invertible"),
        "must not claim the matrix is singular: {s}"
    );
    assert_ne!(
        e,
        LinearError::Singular(3),
        "the two failures are distinct at the same index"
    );
}

#[test]
fn test_kind_reports_the_classification_of_an_error_an_operation_produced() {
    // The accessor exists so a caller can match without naming the tuple field. Driven through a
    // real failure rather than a hand-built value, since that is how a caller meets it.
    let m: DenseMatrix<f64> = MatrixBuild::identity(2);
    let e = m.get(0, 7).unwrap_err();
    assert!(
        matches!(
            e.kind(),
            LinearErrorEnum::IndexOutOfBounds {
                index: (0, 7),
                shape: (2, 2)
            }
        ),
        "got {:?}",
        e.kind()
    );
}

#[test]
fn test_new_and_the_named_constructor_build_the_same_error() {
    // Two routes to one value: the inner enum is public, so a caller may already hold a variant.
    // They must not drift, or a match on one route stops matching a value built by the other.
    let via_new = LinearError::new(LinearErrorEnum::ZeroDiagonal { at_index: 4 });
    let via_named = LinearError::ZeroDiagonal(4);
    assert_eq!(via_new, via_named);
    assert_eq!(format!("{via_new}"), format!("{via_named}"));
    assert!(matches!(
        via_new.kind(),
        LinearErrorEnum::ZeroDiagonal { at_index: 4 }
    ));
}
