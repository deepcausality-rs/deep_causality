/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt;

/// Every way an operation in this crate can fail.
///
/// One enum rather than one per module. The alternative — a `MatrixError`, a `SolveError`, a
/// `ConversionError` — makes every call site that composes two operations write a `From` impl or a
/// `map_err`, and the variants would overlap anyway: a dimension mismatch is the same failure
/// whether it arises in a product or in a solve.
///
/// Variants carry the numbers needed to say what went wrong, because an error that reports only
/// that something was out of bounds sends the reader back to the debugger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearError {
    /// An index outside the matrix or vector shape. Carries the offending `(row, col)` and the
    /// shape it was checked against.
    IndexOutOfBounds {
        index: (usize, usize),
        shape: (usize, usize),
    },

    /// Two matrices whose shapes must agree do not. Carries both shapes.
    ShapeMismatch {
        left: (usize, usize),
        right: (usize, usize),
    },

    /// A product whose inner dimensions do not meet. Carries the left column count and the right
    /// row count.
    InnerDimensionMismatch { left_cols: usize, right_rows: usize },

    /// A vector whose length does not match the dimension it is used against.
    LengthMismatch { expected: usize, found: usize },

    /// An operation defined only on a square matrix, given a rectangular one.
    NotSquare { shape: (usize, usize) },

    /// A matrix that is singular to the precision available: no pivot could be found in a column
    /// that needed one. Carries the column, which is the rank at which elimination stopped.
    Singular { at_column: usize },

    /// A triangular solve given a matrix with a zero on the diagonal, which no substitution can
    /// divide by.
    ZeroDiagonal { at_index: usize },

    /// A triangular solve given a matrix with non-zeros in the wrong triangle. Carries the first
    /// offending position, so that the caller can see which triangle it actually has.
    WrongTriangle { at: (usize, usize) },

    /// A value outside `{0, 1}` offered to the bit-packed 𝔽₂ constructor. Carries the position, so
    /// the caller can find it without re-scanning.
    NotBinary { at: (usize, usize) },

    /// An exact computation whose result does not fit the scalar type.
    ///
    /// ℤ is unbounded and `i64` is not. The determinant of an integer matrix is an integer, but not
    /// necessarily one that fits: `det(diag(i64::MAX, 2))` is `2^64 - 2`. Wrapping would return a
    /// plausible integer that is wrong, and no caller could tell it from a correct one — which is
    /// the failure mode the exact path exists to remove, reappearing one level down.
    ///
    /// Carries the operation that overflowed, because by the time a caller sees this the
    /// intermediate that overflowed is gone.
    Overflow { operation: &'static str },

    /// An operation that has no meaning on an empty matrix.
    EmptyMatrix,
}

impl fmt::Display for LinearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearError::IndexOutOfBounds { index, shape } => write!(
                f,
                "Index out of bounds: ({}, {}) is outside a {}x{} matrix.",
                index.0, index.1, shape.0, shape.1
            ),
            LinearError::ShapeMismatch { left, right } => write!(
                f,
                "Shape mismatch: left is {}x{}, right is {}x{}.",
                left.0, left.1, right.0, right.1
            ),
            LinearError::InnerDimensionMismatch {
                left_cols,
                right_rows,
            } => write!(
                f,
                "Inner dimension mismatch: left has {left_cols} columns, right has {right_rows} rows."
            ),
            LinearError::LengthMismatch { expected, found } => {
                write!(f, "Length mismatch: expected {expected}, found {found}.")
            }
            LinearError::NotSquare { shape } => write!(
                f,
                "Not square: this operation needs a square matrix, got {}x{}.",
                shape.0, shape.1
            ),
            LinearError::Singular { at_column } => {
                write!(f, "Singular: no pivot available in column {at_column}.")
            }
            LinearError::ZeroDiagonal { at_index } => write!(
                f,
                "Zero diagonal: entry {at_index} on the diagonal is zero, so the substitution cannot divide by it."
            ),
            LinearError::WrongTriangle { at } => write!(
                f,
                "Wrong triangle: a non-zero entry at ({}, {}) is outside the expected triangle.",
                at.0, at.1
            ),
            LinearError::NotBinary { at } => write!(
                f,
                "Not binary: the entry at ({}, {}) is outside {{0, 1}} and cannot be packed into GF(2).",
                at.0, at.1
            ),
            LinearError::Overflow { operation } => write!(
                f,
                "Overflow: {operation} produced a value the scalar type cannot hold. The exact result exists in the unbounded structure; it does not fit this representation."
            ),
            LinearError::EmptyMatrix => write!(
                f,
                "Empty matrix: the operation has no meaning on an empty matrix."
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for LinearError {}
