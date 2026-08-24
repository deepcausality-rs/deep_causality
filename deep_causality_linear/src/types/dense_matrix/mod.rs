/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A dense, row-major matrix.

pub mod algebra;
pub mod ops;

use crate::errors::linear_error::LinearError;
use alloc::vec::Vec;

/// A dense matrix stored row-major.
///
/// # Why this exists when `CausalTensor` has a rank-2 case
///
/// A rank-2 tensor is a matrix that has to be asked whether it is one. The construction census
/// across the seven consumer crates found 46 rank-2 constructions, and the crates holding them
/// maintain the invariant by hand: `DensityMatrix` stores `dim: usize` beside its tensor because
/// `CausalTensor` cannot express squareness, and topology's `AdjacencyMatrix`, `IncidenceMatrix` and
/// `LaplacianMatrix` are bare aliases of it. Physics, quantum and topology together call 56
/// two-dimensional operations and zero N-dimensional ones.
///
/// Carrying the two dimensions in the type moves rank and squareness out of runtime checks. A
/// determinant asked of this type needs no dimension argument from the caller and cannot be handed
/// a rank-3 object.
///
/// # Row-major
///
/// Elimination works on rows. Row-major puts each row's entries contiguous, so `axpy_rows` walks
/// one slice and `swap_rows` exchanges two, which is what makes the row-operation seam cheap.
///
/// # Fields are private
///
/// Shape and data must agree — `data.len() == rows * cols` — and that is an invariant the type
/// keeps rather than one every caller re-establishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseMatrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> DenseMatrix<T> {
    /// Builds from a row-major buffer.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if `data.len()` is not `rows * cols`.
    pub fn from_vec(data: Vec<T>, rows: usize, cols: usize) -> Result<Self, LinearError> {
        if data.len() != rows * cols {
            return Err(LinearError::ShapeMismatch {
                left: (rows, cols),
                right: (data.len(), 1),
            });
        }
        Ok(Self { data, rows, cols })
    }

    /// The entries, row-major.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// The number of rows.
    pub(crate) fn row_count(&self) -> usize {
        self.rows
    }

    /// The number of columns.
    pub(crate) fn col_count(&self) -> usize {
        self.cols
    }

    /// The row count, for the HKT witness, which must rebuild the shape after mapping.
    pub(crate) fn rows_pub(&self) -> usize {
        self.rows
    }

    /// The column count, for the same reason.
    pub(crate) fn cols_pub(&self) -> usize {
        self.cols
    }

    /// Consumes the matrix and yields its entries, row-major.
    pub(crate) fn into_data(self) -> Vec<T> {
        self.data
    }

    /// The entries, row-major, mutably.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// One row's entries, which are contiguous.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the row is outside the shape.
    pub fn row(&self, row: usize) -> Result<&[T], LinearError> {
        if row >= self.rows {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, 0),
                shape: (self.rows, self.cols),
            });
        }
        Ok(&self.data[row * self.cols..(row + 1) * self.cols])
    }
}
