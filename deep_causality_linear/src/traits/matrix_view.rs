/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::linear_error::LinearError;
use deep_causality_num::Zero;

/// Read access to a matrix: its shape, and its entries by value.
///
/// Every representation in this crate implements this, the compressed-sparse-row one included. It
/// is the half of the seam that carries no assumption about storage layout.
///
/// # Access is by value
///
/// `get` returns `Self::Scalar` rather than `&Self::Scalar`, because a bit-packed representation has
/// no element to lend a reference to — the entry exists as one bit inside a word, and there is no
/// `Gf2` in memory to point at. Returning by value costs nothing here: the scalars in this workspace
/// are `Clone`, and the ones that matter numerically are `Copy`.
///
/// # A structural zero is a zero
///
/// A sparse matrix stores only its non-zeros. Reading a position outside the stored pattern returns
/// `Self::Scalar::zero()` and is not an error — the entry is genuinely zero, and the caller asking
/// for it has done nothing wrong. Only an index outside the *shape* fails.
///
/// # Why `Zero` on the associated type
///
/// Two things need it, and neither is optional. The structural-zero rule above is one. The other is
/// [`RowOps::pivot_in_column`](crate::traits::row_ops::RowOps::pivot_in_column), whose default
/// implementation has to recognise a non-zero entry, and which would otherwise need a bound that
/// every caller of every elimination would have to repeat.
pub trait MatrixView {
    /// The type of an entry.
    ///
    /// An associated type rather than a type parameter: a representation has exactly one scalar,
    /// and a type parameter would let a caller ask for `MatrixView<f64>` on a matrix of `i64` and
    /// get a confusing error rather than a missing-impl one.
    type Scalar: Zero + Clone;

    /// The number of rows.
    fn rows(&self) -> usize;

    /// The number of columns.
    fn cols(&self) -> usize;

    /// The shape, as `(rows, cols)`.
    #[inline]
    fn shape(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }

    /// The number of entries the shape describes, which is not the number stored.
    #[inline]
    fn len(&self) -> usize {
        self.rows() * self.cols()
    }

    /// Whether the shape describes no entries at all.
    ///
    /// True for `0x0`, and also for `0xn` and `nx0`, which are distinct shapes that both hold
    /// nothing. Elimination on any of them has rank zero rather than an error.
    #[inline]
    fn is_empty(&self) -> bool {
        self.rows() == 0 || self.cols() == 0
    }

    /// Whether the matrix is square.
    ///
    /// `0x0` is square. The determinant of the empty matrix is one, being the empty product.
    #[inline]
    fn is_square(&self) -> bool {
        self.rows() == self.cols()
    }

    /// The entry at `(row, col)`.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the position is outside the shape. A position inside the
    /// shape but outside a sparse matrix's stored pattern is not an error; it is a zero.
    fn get(&self, row: usize, col: usize) -> Result<Self::Scalar, LinearError>;
}
