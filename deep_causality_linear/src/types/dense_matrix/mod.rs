/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A dense, row-major matrix.

pub mod algebra;

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
        let _ = (&data, rows, cols);
        todo!("DenseMatrix::from_vec")
    }

    /// The entries, row-major.
    pub fn as_slice(&self) -> &[T] {
        todo!("DenseMatrix::as_slice")
    }

    /// One row's entries, which are contiguous.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if the row is outside the shape.
    pub fn row(&self, row: usize) -> Result<&[T], LinearError> {
        let _ = row;
        todo!("DenseMatrix::row")
    }
}
