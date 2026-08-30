/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use deep_causality_algebra::Field;

/// The three row operations every elimination is built from.
///
/// The inner loop of RREF, of rank, of a kernel basis, of an LU factorisation is a row update.
/// Putting that behind a trait method lets a representation that can update a whole row at once do
/// so, while the algorithm driving it never sees an individual entry in the hot path. The prototype
/// measured the cost of that seam at 0.92–0.95× a hand-written non-generic loop over the same packed
/// words — it is slightly *faster*, for the reason `from_col` exists.
///
/// # Implemented by dense layouts only
///
/// `deep_causality_linear` implements this for the dense matrix and the bit-packed 𝔽₂ matrix, and
/// **not** for the compressed-sparse-row matrix.
///
/// `swap_rows` on CSR is fine. `axpy_rows` is not: adding a multiple of one sparse row to another
/// changes that row's non-zero pattern, which in CSR means reallocating every row after it. Sparse
/// elimination needs a fill-reducing ordering and a symbolic factorisation — a different algorithm,
/// not a different implementation of this one. A caller who wants to eliminate on a sparse matrix
/// converts to a dense layout, and that conversion is written at the call site so its cost is
/// visible.
///
/// # `from_col` is not an optimisation hint
///
/// Every operation takes the column to start from. By the time elimination reaches column `k`, the
/// entries to its left are already zero in every row it will touch, so re-reading them is wasted
/// work — and it is the work a hand-written loop over `&mut [u64]` does, which is why the generic
/// version through this trait outruns it. An implementation may ignore `from_col` and still be
/// correct.
pub trait RowOps: MatrixView {
    /// Exchanges two rows.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if either row is outside the shape. Swapping a row with
    /// itself is a no-op rather than an error.
    fn swap_rows(&mut self, a: usize, b: usize) -> Result<(), LinearError>;

    /// Multiplies `row` by `factor`, from column `from_col` onward.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the row is outside the shape.
    fn scale_row(
        &mut self,
        row: usize,
        factor: &Self::Scalar,
        from_col: usize,
    ) -> Result<(), LinearError>;

    /// `dst := dst + factor * src`, from column `from_col` onward.
    ///
    /// The one operation that makes elimination possible and sparse elimination a separate problem.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if either row is outside the shape.
    fn axpy_rows(
        &mut self,
        dst: usize,
        src: usize,
        factor: &Self::Scalar,
        from_col: usize,
    ) -> Result<(), LinearError>;

    /// Chooses a pivot row in `col`, searching at or below `from_row`.
    ///
    /// Returns `None` when the column is entirely zero from `from_row` down, which is what tells
    /// elimination that the column contributes no rank.
    ///
    /// # The search is not optional
    ///
    /// The default searches. An implementation may override this to search *differently* — the
    /// bit-packed matrix scans a word at a time rather than a bit at a time — but it may not
    /// override it to take `self.get(from_row, col)` and stop.
    ///
    /// This is load-bearing rather than a quality nicety. Both Laplace determinants in
    /// `deep_causality_topology` are fed Cayley-Menger matrices, whose `(0,0)` entry is zero by
    /// construction. Elimination that assumes the diagonal returns **zero for every simplex
    /// volume**: measured on a regular unit tetrahedron, the pivoted result is `det = 4.0` and
    /// `vol = √2⁄12`, and the unpivoted one is `det = 0.0` and a NaN volume.
    ///
    /// # This default is exact, and that is a numerical choice
    ///
    /// Taking the first non-zero is correct over any field and needs no ordering and no epsilon,
    /// which is what lets 𝔽₂ and ℚ through — neither has an order, and neither needs one. Over the
    /// floats it is correct but ill-conditioned: a pivot near zero amplifies rounding. The
    /// magnitude-pivoting path is a separate entry point on the algorithms rather than an override
    /// here, because `DenseMatrix<T>` has one impl slot and needs two opposite answers depending on
    /// whether `T` is normed — the same mechanic that made the algebra tower's law markers
    /// parameterised by their operator.
    fn pivot_in_column(&self, col: usize, from_row: usize) -> Option<usize>
    where
        Self::Scalar: Field,
    {
        (from_row..self.rows()).find(|&r| {
            self.get(r, col)
                .map(|v| !crate::traits::row_ops::is_zero(&v))
                .unwrap_or(false)
        })
    }
}

/// Whether a scalar is the additive identity.
///
/// A free function rather than a method call so that the default `pivot_in_column` above reads as
/// one expression. `Zero::is_zero` takes `&self`, and the entry arrives by value from `get`.
#[inline]
pub(crate) fn is_zero<T: deep_causality_num::Zero>(v: &T) -> bool {
    v.is_zero()
}
