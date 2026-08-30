/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `deep_causality_sparse::SparseMatrixError` test suite, ported onto `LinearError`.
//!
//! The sparse crate carried four variants — `ShapeMismatch`, `DimensionMismatch`,
//! `IndexOutOfBounds`, `EmptyMatrix` — and asserted the rendered text of each plus the derived
//! traits. `LinearError` covers the same four failures under different names and a different
//! shape of payload, so each test here states the same claim against the variant that replaced it.
//!
//! Two payloads changed arity and the tests record that rather than hide it:
//!
//! - `DimensionMismatch(left_cols, right_rows)` split into `InnerDimensionMismatch` for a product
//!   and `LengthMismatch` for a vector. The product case is ported; the vector case has no sparse
//!   counterpart to port.
//! - `IndexOutOfBounds(index, size)` carried a flat index into one dimension. The replacement
//!   carries a `(row, col)` position and the full `(rows, cols)` shape, so it names the axis that
//!   the flat form left the reader to infer.
//!
//! The rendered text differs from the sparse wording throughout. These tests pin the wording
//! `LinearError` actually produces, so a change to it is caught here.

use deep_causality_linear::LinearError;

#[test]
fn test_shape_mismatch_display_names_both_shapes() {
    let error = LinearError::ShapeMismatch((2, 3), (3, 2));
    assert_eq!(
        format!("{error}"),
        "Shape mismatch: left is 2x3, right is 3x2."
    );
}

#[test]
fn test_inner_dimension_mismatch_display_names_left_columns_and_right_rows() {
    let error = LinearError::InnerDimensionMismatch(3, 2);
    assert_eq!(
        format!("{error}"),
        "Inner dimension mismatch: left has 3 columns, right has 2 rows."
    );
}

#[test]
fn test_index_out_of_bounds_display_names_the_position_and_the_shape() {
    // The sparse original was `IndexOutOfBounds(5, 3)`: index 5 into a dimension of size 3. The
    // same failure here is row 5 of a 3x3 matrix, and the message names the column too.
    let error = LinearError::IndexOutOfBounds((5, 0), (3, 3));
    assert_eq!(
        format!("{error}"),
        "Index out of bounds: (5, 0) is outside a 3x3 matrix."
    );
}

#[test]
fn test_empty_matrix_display_says_the_operation_has_no_meaning() {
    let error = LinearError::EmptyMatrix();
    assert_eq!(
        format!("{error}"),
        "Empty matrix: the operation has no meaning on an empty matrix."
    );
}

#[cfg(feature = "std")]
#[test]
fn test_linear_error_is_an_error_with_no_source() {
    use std::error::Error;

    let error = LinearError::EmptyMatrix();
    assert!(error.source().is_none());
}

#[test]
fn test_linear_error_debug_shows_the_variant_and_both_shapes() {
    let error = LinearError::ShapeMismatch((1, 1), (2, 2));
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("ShapeMismatch"), "{debug_str}");
    assert!(debug_str.contains("(1, 1)"), "{debug_str}");
    assert!(debug_str.contains("(2, 2)"), "{debug_str}");
}

#[test]
fn test_linear_error_clones_to_an_equal_value() {
    let error = LinearError::InnerDimensionMismatch(10, 5);
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_linear_error_compares_by_payload() {
    let error1 = LinearError::IndexOutOfBounds((0, 0), (10, 10));
    let error2 = LinearError::IndexOutOfBounds((0, 0), (10, 10));
    let error3 = LinearError::IndexOutOfBounds((1, 0), (10, 10));
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_linear_error_payload_free_variants_are_equal() {
    let error1 = LinearError::EmptyMatrix();
    let error2 = LinearError::EmptyMatrix();
    assert!(error1 == error2);
}
