/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt;

/// Every way an operation in this crate can fail.
///
/// One error type rather than one per module. The alternative — a `MatrixError`, a `SolveError`, a
/// `ConversionError` — makes every call site that composes two operations write a `From` impl or a
/// `map_err`, and the variants would overlap anyway: a dimension mismatch is the same failure
/// whether it arises in a product or in a solve.
///
/// It is also **one type across representations**. `deep_causality_sparse` carried a separate
/// `SparseMatrixError` whose four variants named the same four failures under different names and
/// different payload shapes — a flat `IndexOutOfBounds(index, size)` against a positional
/// `IndexOutOfBounds { index: (row, col), shape }`, and a `DimensionMismatch(a, b)` that conflated
/// a product's inner dimensions with a vector's length. Both are folded in here, which is why a
/// caller that composes a sparse operation with a dense one no longer converts between two error
/// types that were describing the same things.
///
/// # A struct wrapping an enum, not a bare enum
///
/// The public type is a newtype and the classification lives in [`LinearErrorEnum`] behind it.
/// That is what keeps the surface forward compatible: a new failure mode is a new variant on the
/// inner enum, and a caller that matched `LinearError(LinearErrorEnum::NotSquare { .. })` with a
/// wildcard arm keeps compiling. A bare public enum makes every addition a breaking change, since
/// an exhaustive `match` in any downstream crate stops being exhaustive.
///
/// Construct one through the associated functions rather than the inner enum:
///
/// ```
/// use deep_causality_linear::{LinearError, LinearErrorEnum};
///
/// let e = LinearError::NotSquare((2, 3));
/// assert!(matches!(e.kind(), LinearErrorEnum::NotSquare { shape: (2, 3) }));
/// ```
///
/// This is the shape `deep_causality_physics::PhysicsError` uses, for the same reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearError(pub LinearErrorEnum);

/// The classification behind [`LinearError`].
///
/// Variants carry the numbers needed to say what went wrong, because an error that reports only
/// that something was out of bounds sends the reader back to the debugger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearErrorEnum {
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

    /// A Cholesky factorisation whose radicand went non-positive, so the input is not positive
    /// definite. Carries the diagonal index where that was discovered.
    ///
    /// Distinct from [`Singular`](Self::Singular): a matrix can be invertible and indefinite —
    /// `diag(1, -1)` is both — so "no Cholesky factor" and "no inverse" are different failures and
    /// a caller may well recover from one and not the other.
    NotPositiveDefinite { at_index: usize },

    /// An operation that has no meaning on an empty matrix.
    EmptyMatrix,
}

impl LinearError {
    /// The classification behind this error.
    ///
    /// Matching on this rather than on `self.0` keeps a caller working if the field is ever made
    /// private.
    pub fn kind(&self) -> &LinearErrorEnum {
        &self.0
    }

    /// Wraps a classification.
    pub fn new(variant: LinearErrorEnum) -> Self {
        Self(variant)
    }

    #[allow(non_snake_case)]
    pub fn IndexOutOfBounds(index: (usize, usize), shape: (usize, usize)) -> Self {
        Self(LinearErrorEnum::IndexOutOfBounds { index, shape })
    }

    #[allow(non_snake_case)]
    pub fn ShapeMismatch(left: (usize, usize), right: (usize, usize)) -> Self {
        Self(LinearErrorEnum::ShapeMismatch { left, right })
    }

    #[allow(non_snake_case)]
    pub fn InnerDimensionMismatch(left_cols: usize, right_rows: usize) -> Self {
        Self(LinearErrorEnum::InnerDimensionMismatch {
            left_cols,
            right_rows,
        })
    }

    #[allow(non_snake_case)]
    pub fn LengthMismatch(expected: usize, found: usize) -> Self {
        Self(LinearErrorEnum::LengthMismatch { expected, found })
    }

    #[allow(non_snake_case)]
    pub fn NotSquare(shape: (usize, usize)) -> Self {
        Self(LinearErrorEnum::NotSquare { shape })
    }

    #[allow(non_snake_case)]
    pub fn Singular(at_column: usize) -> Self {
        Self(LinearErrorEnum::Singular { at_column })
    }

    #[allow(non_snake_case)]
    pub fn ZeroDiagonal(at_index: usize) -> Self {
        Self(LinearErrorEnum::ZeroDiagonal { at_index })
    }

    #[allow(non_snake_case)]
    pub fn WrongTriangle(at: (usize, usize)) -> Self {
        Self(LinearErrorEnum::WrongTriangle { at })
    }

    #[allow(non_snake_case)]
    pub fn NotBinary(at: (usize, usize)) -> Self {
        Self(LinearErrorEnum::NotBinary { at })
    }

    #[allow(non_snake_case)]
    pub fn Overflow(operation: &'static str) -> Self {
        Self(LinearErrorEnum::Overflow { operation })
    }

    #[allow(non_snake_case)]
    pub fn NotPositiveDefinite(at_index: usize) -> Self {
        Self(LinearErrorEnum::NotPositiveDefinite { at_index })
    }

    #[allow(non_snake_case)]
    pub fn EmptyMatrix() -> Self {
        Self(LinearErrorEnum::EmptyMatrix)
    }
}

impl fmt::Display for LinearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Display for LinearErrorEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearErrorEnum::IndexOutOfBounds { index, shape } => write!(
                f,
                "Index out of bounds: ({}, {}) is outside a {}x{} matrix.",
                index.0, index.1, shape.0, shape.1
            ),
            LinearErrorEnum::ShapeMismatch { left, right } => write!(
                f,
                "Shape mismatch: left is {}x{}, right is {}x{}.",
                left.0, left.1, right.0, right.1
            ),
            LinearErrorEnum::InnerDimensionMismatch {
                left_cols,
                right_rows,
            } => write!(
                f,
                "Inner dimension mismatch: left has {left_cols} columns, right has {right_rows} rows."
            ),
            LinearErrorEnum::LengthMismatch { expected, found } => {
                write!(f, "Length mismatch: expected {expected}, found {found}.")
            }
            LinearErrorEnum::NotSquare { shape } => write!(
                f,
                "Not square: this operation needs a square matrix, got {}x{}.",
                shape.0, shape.1
            ),
            LinearErrorEnum::Singular { at_column } => {
                write!(f, "Singular: no pivot available in column {at_column}.")
            }
            LinearErrorEnum::ZeroDiagonal { at_index } => write!(
                f,
                "Zero diagonal: entry {at_index} on the diagonal is zero, so the substitution cannot divide by it."
            ),
            LinearErrorEnum::WrongTriangle { at } => write!(
                f,
                "Wrong triangle: a non-zero entry at ({}, {}) is outside the expected triangle.",
                at.0, at.1
            ),
            LinearErrorEnum::NotBinary { at } => write!(
                f,
                "Not binary: the entry at ({}, {}) is outside {{0, 1}} and cannot be packed into GF(2).",
                at.0, at.1
            ),
            LinearErrorEnum::Overflow { operation } => write!(
                f,
                "Overflow: {operation} produced a value the scalar type cannot hold. The exact result exists in the unbounded structure; it does not fit this representation."
            ),
            LinearErrorEnum::NotPositiveDefinite { at_index } => write!(
                f,
                "Not positive definite: the Cholesky radicand at diagonal entry {at_index} is not positive, so the matrix has no Cholesky factor. It may still be invertible."
            ),
            LinearErrorEnum::EmptyMatrix => write!(
                f,
                "Empty matrix: the operation has no meaning on an empty matrix."
            ),
        }
    }
}

// Unconditional, not gated on `std`. `deep_causality_sparse::SparseMatrixError` implemented
// `core::error::Error` for every build, and a no-std caller that relied on the bound would lose it
// silently if this were narrower.
impl core::error::Error for LinearError {}
impl core::error::Error for LinearErrorEnum {}
