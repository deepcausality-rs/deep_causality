/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The seam that lets `deep_causality_linear`'s algorithms read a `CausalTensor` in place.
//!
//! # Why the impl is here and nowhere else
//!
//! [`MatrixView`] is `deep_causality_linear`'s and [`CausalTensor`] is this crate's, so a third
//! crate writing this impl fails E0117 with both halves foreign. Writing it in
//! `deep_causality_linear` would need that crate to depend on this one, and the delegation runs the
//! other way — `CausalTensor::svd` calls into `deep_causality_linear`, so the edge is fixed as
//! tensor → linear and the reverse would close a cycle. This module is the only place it can live.
//!
//! # What it buys
//!
//! The decompositions are generic over the read trait, so they run against a tensor's own buffer.
//! No rank-2 tensor is built to call them and no dense matrix is built to hold the copy.
//!
//! # A tensor of rank other than two is not a matrix
//!
//! `rows` and `cols` return `usize` and cannot report an error, so a tensor that is not
//! two-dimensional has to present *as* something. It presents as `0 × 1`: empty, and not square.
//!
//! That shape is chosen so the operations refuse it. `0 × 0` would have been the obvious choice and
//! is the wrong one — the determinant of the empty matrix is the empty product, so a 3-D tensor
//! would get a confident `1` back. `0 × 1` is not square, so [`determinant`] rejects it, and
//! [`get`] rejects every position rather than reading through a shape the tensor does not have.
//!
//! [`determinant`]: deep_causality_linear::determinant
//! [`get`]: MatrixView::get

use crate::CausalTensor;
use alloc::vec::Vec;
use deep_causality_linear::{LinearError, MatrixView};
use deep_causality_num::Zero;

/// The shape a tensor that is not two-dimensional presents as: empty, and not square.
const NOT_A_MATRIX: (usize, usize) = (0, 1);

impl<T> MatrixView for CausalTensor<T>
where
    T: Zero + Clone,
{
    type Scalar = T;

    fn rows(&self) -> usize {
        match self.shape() {
            [r, _] => *r,
            _ => NOT_A_MATRIX.0,
        }
    }

    fn cols(&self) -> usize {
        match self.shape() {
            [_, c] => *c,
            _ => NOT_A_MATRIX.1,
        }
    }

    fn get(&self, row: usize, col: usize) -> Result<T, LinearError> {
        let shape = self.shape();
        // The rank check and the bounds check are one check: a tensor that is not 2-D has no
        // position `(row, col)` names, so every position is out of bounds rather than some of them.
        let [r, c] = shape else {
            return Err(LinearError::IndexOutOfBounds((row, col), NOT_A_MATRIX));
        };
        if row >= *r || col >= *c {
            return Err(LinearError::IndexOutOfBounds((row, col), (*r, *c)));
        }
        // `CausalTensor::get` returns `None` only for an out-of-bounds index, which the check above
        // has already excluded. A tensor is dense, so an index inside the shape always has a value.
        self.get(&[row, col])
            .cloned()
            .ok_or(LinearError::IndexOutOfBounds((row, col), (*r, *c)))
    }

    /// A tensor already holds its entries contiguously in row-major order, so this is the copy.
    ///
    /// The default walks every position through [`MatrixView::get`], which costs a rank check and a
    /// bounds check per entry. On the 48x48 QR benchmark that measured 4.7% slower than the memcpy,
    /// which is the whole cost of routing the decompositions through the read trait.
    ///
    /// A tensor of rank other than two presents as `0 x 1` and so has no entries to copy.
    fn to_row_major(&self) -> Result<Vec<T>, LinearError> {
        match self.shape() {
            [_, _] => Ok(self.as_slice().to_vec()),
            _ => Ok(Vec::new()),
        }
    }
}
