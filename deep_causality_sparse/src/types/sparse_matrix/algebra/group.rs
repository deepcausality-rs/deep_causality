/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::{Additive, Associative, Commutative, Multiplicative};

use crate::CsrMatrix;
use core::ops::Sub;
use deep_causality_algebra::AbelianGroup;

// AbelianGroup for CsrMatrix
// Matrix addition is element-wise, so it associates and commutes exactly when the element
// type does. These were previously unstatable: the only marker available promised
// `a * b == b * a`, and `CsrMatrix`'s `Mul` is matrix multiplication, which does not
// commute — so claiming any law meant claiming a false one.
impl<T> Associative<Additive> for CsrMatrix<T> where T: Associative<Additive> + Copy {}
impl<T> Commutative<Additive> for CsrMatrix<T> where T: Commutative<Additive> + Copy {}
// Matrix multiplication IS associative, and that is now sayable without also claiming
// that it commutes.
impl<T> Associative<Multiplicative> for CsrMatrix<T> where T: Associative<Multiplicative> + Copy {}

// Reached through the `AbelianGroup` blanket now that the additive markers are present.

impl<T> CsrMatrix<T>
where
    T: AbelianGroup + Copy + Default + PartialEq,
{
    /// Creates a zero matrix with the given shape.
    ///
    /// # Arguments
    /// * `rows` - Number of rows
    /// * `cols` - Number of columns
    ///
    /// # Returns
    /// A sparse matrix with all elements zero (empty CSR structure).
    pub fn zero(rows: usize, cols: usize) -> Self {
        Self {
            row_indices: vec![0; rows + 1],
            col_indices: Vec::new(),
            values: Vec::new(),
            shape: (rows, cols),
        }
    }

    /// Element-wise matrix addition (panics on shape mismatch).
    ///
    /// # Panics
    /// Panics if `self.shape != rhs.shape`.
    pub fn add(&self, rhs: &Self) -> Self {
        self.add_matrix_impl(rhs)
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> CsrMatrix<T>
where
    T: AbelianGroup + Copy + Sub<Output = T> + Default + PartialEq,
{
    /// Element-wise matrix subtraction (panics on shape mismatch).
    ///
    /// # Panics
    /// Panics if `self.shape != rhs.shape`.
    pub fn sub(&self, rhs: &Self) -> Self {
        self.sub_matrix_impl(rhs)
            .expect("CsrMatrix shape mismatch in sub")
    }
}

impl<T> CsrMatrix<T>
where
    T: AbelianGroup + Copy + core::ops::Neg<Output = T>,
{
    /// Element-wise negation.
    pub fn neg(&self) -> Self {
        Self {
            row_indices: self.row_indices.clone(),
            col_indices: self.col_indices.clone(),
            values: self.values.iter().map(|&v| -v).collect(),
            shape: self.shape,
        }
    }
}
