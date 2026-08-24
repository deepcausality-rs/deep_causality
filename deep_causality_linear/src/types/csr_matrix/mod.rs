/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A compressed-sparse-row matrix.

pub mod algebra;

use crate::errors::linear_error::LinearError;
use alloc::vec::Vec;
use deep_causality_algebra::CommutativeSemiring;
use deep_causality_num::Zero;

/// A matrix in compressed sparse row form.
///
/// Moves here from `deep_causality_sparse` with its public surface unchanged, so that code written
/// against the retired crate compiles against the new path and returns identical results.
///
/// # The read side only
///
/// This type implements [`MatrixView`](crate::MatrixView) and deliberately **not**
/// [`RowOps`](crate::RowOps). `swap_rows` on CSR is fine; `axpy_rows` is not, because adding a
/// multiple of one sparse row to another changes that row's non-zero pattern, which in CSR means
/// reallocating every row after it. Sparse elimination needs a fill-reducing ordering and a symbolic
/// factorisation, which is a different algorithm and a separate proposal.
///
/// A caller who wants to eliminate on a sparse matrix converts to a dense layout, and writes that
/// conversion at the call site so its cost is visible rather than hidden inside an algorithm.
///
/// # What it is for
///
/// Matrix–vector products against a matrix that is mostly zeros. `deep_causality_topology`'s
/// boundary and coboundary operators are `CsrMatrix<i8>` with entries in `{-1, 0, 1}`, and
/// `deep_causality_physics` applies Hodge-star and coboundary operators to fields. The work is
/// proportional to the number of stored entries rather than to `rows * cols`, which is the whole
/// reason the representation exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrMatrix<T> {
    row_indices: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<T>,
    shape: (usize, usize),
}

impl<T> CsrMatrix<T> {
    /// An empty matrix with shape `(0, 0)`.
    pub fn new() -> Self {
        todo!("CsrMatrix::new")
    }

    /// A matrix of the given shape with room reserved for `capacity` stored entries.
    pub fn with_capacity(rows: usize, cols: usize, capacity: usize) -> Self {
        let _ = (rows, cols, capacity);
        todo!("CsrMatrix::with_capacity")
    }

    /// The shape, as `(rows, cols)`.
    pub fn shape(&self) -> (usize, usize) {
        todo!("CsrMatrix::shape")
    }

    /// The row-pointer array, of length `rows + 1`.
    pub fn row_indices(&self) -> &Vec<usize> {
        todo!("CsrMatrix::row_indices")
    }

    /// The column index of each stored entry.
    pub fn col_indices(&self) -> &Vec<usize> {
        todo!("CsrMatrix::col_indices")
    }

    /// The stored entries, in row-major order of position.
    pub fn values(&self) -> &Vec<T> {
        todo!("CsrMatrix::values")
    }

    /// Decomposes into the three arrays and the shape.
    pub fn into_parts(self) -> (Vec<usize>, Vec<usize>, Vec<T>, (usize, usize)) {
        todo!("CsrMatrix::into_parts")
    }

    /// Applies `f` to every **stored** entry.
    ///
    /// Structural zeros are not visited. A function that does not fix zero therefore changes the
    /// matrix this represents, and the caller is choosing that by calling this rather than
    /// densifying first.
    pub fn map_values<U, F>(self, f: F) -> CsrMatrix<U>
    where
        F: Fn(T) -> U,
    {
        let _ = f;
        todo!("CsrMatrix::map_values")
    }
}

impl<T> CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    /// Builds from `(row, col, value)` triplets.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if any triplet names a position outside the shape.
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, T)],
    ) -> Result<Self, LinearError> {
        let _ = (rows, cols, triplets);
        todo!("CsrMatrix::from_triplets")
    }

    /// The entry at `(row, col)`, returning the scalar zero for a position outside the stored
    /// pattern.
    pub fn get_value_at(&self, row_idx: usize, col_idx: usize) -> T
    where
        T: Zero,
    {
        let _ = (row_idx, col_idx);
        todo!("CsrMatrix::get_value_at")
    }

    /// The transpose.
    ///
    /// Bounded on `CommutativeSemiring` rather than on `Field`: transposing moves entries and
    /// performs no arithmetic at all, so it is available over ℕ.
    pub fn transpose(&self) -> Self {
        todo!("CsrMatrix::transpose")
    }

    /// The matrix–vector product.
    ///
    /// Work is proportional to the number of stored entries. The result is dense, because a sparse
    /// matrix times a dense vector generally is.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if the vector's length is not the column count.
    pub fn vec_mult(&self, vector: &[T]) -> Result<Vec<T>, LinearError> {
        let _ = vector;
        todo!("CsrMatrix::vec_mult")
    }

    /// The matrix product.
    ///
    /// # Errors
    ///
    /// [`LinearError::InnerDimensionMismatch`] if the inner dimensions do not meet.
    pub fn mat_mult(&self, other: &Self) -> Result<Self, LinearError> {
        let _ = other;
        todo!("CsrMatrix::mat_mult")
    }

    /// Entrywise addition.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the shapes differ.
    pub fn add_matrix(&self, other: &Self) -> Result<Self, LinearError> {
        let _ = other;
        todo!("CsrMatrix::add_matrix")
    }

    /// Multiplies every stored entry by `scalar`.
    pub fn scalar_mult(&self, scalar: T) -> Self {
        let _ = scalar;
        todo!("CsrMatrix::scalar_mult")
    }
}

impl<T> Default for CsrMatrix<T> {
    fn default() -> Self {
        todo!("CsrMatrix::default")
    }
}
