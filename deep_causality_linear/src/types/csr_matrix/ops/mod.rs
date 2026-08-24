/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The access-trait and operator impls for the sparse matrix.
//!
//! # `RowOps` is absent, deliberately
//!
//! There is no `impl RowOps for CsrMatrix<T>` in this file or anywhere else, and there must not be.
//! `swap_rows` on CSR would be fine; `axpy_rows` is not, because adding a multiple of one sparse row
//! to another changes that row's non-zero pattern, which means reallocating every row after it.
//!
//! The consequence a caller sees is a missing-impl error rather than a slow correct answer, which is
//! the intent: eliminating on a sparse matrix means converting to a dense layout, and that
//! conversion belongs at the call site where its cost is visible.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_build::MatrixBuild;
use crate::traits::matrix_view::MatrixView;
use crate::types::csr_matrix::CsrMatrix;
use core::ops::{Add, Mul, MulAssign, Neg, Sub};
use deep_causality_algebra::{CommutativeRing, CommutativeSemiring, Ring};
use deep_causality_num::{One, Zero};

impl<T> MatrixView for CsrMatrix<T>
where
    T: Zero + Clone,
{
    type Scalar = T;

    fn rows(&self) -> usize {
        todo!("CsrMatrix::rows")
    }
    fn cols(&self) -> usize {
        todo!("CsrMatrix::cols")
    }

    /// A position inside the shape but outside the stored pattern returns the scalar zero. It is
    /// genuinely zero, and a caller asking for it has done nothing wrong.
    fn get(&self, row: usize, col: usize) -> Result<T, LinearError> {
        let _ = (row, col);
        todo!("CsrMatrix::get_view")
    }
}

impl<T> MatrixBuild for CsrMatrix<T>
where
    T: Zero + Clone,
{
    /// Stores nothing, so the cost is the row-pointer array rather than `rows * cols`.
    fn zeros(rows: usize, cols: usize) -> Self {
        let _ = (rows, cols);
        todo!("CsrMatrix::zeros_build")
    }

    fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), LinearError> {
        let _ = (row, col, value);
        todo!("CsrMatrix::set_build")
    }
}

impl<T> Zero for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn zero() -> Self {
        todo!("CsrMatrix::zero_op")
    }
    fn is_zero(&self) -> bool {
        todo!("CsrMatrix::is_zero_op")
    }
}

impl<T> One for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn one() -> Self {
        todo!("CsrMatrix::one_op")
    }
    fn is_one(&self) -> bool {
        todo!("CsrMatrix::is_one_op")
    }
}

impl<T> Add for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("CsrMatrix::add_op")
    }
}

impl<T> Sub for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("CsrMatrix::sub_op")
    }
}

impl<T> Neg for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = Self;
    fn neg(self) -> Self {
        todo!("CsrMatrix::neg_op")
    }
}

impl<T> Mul for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("CsrMatrix::mul_op")
    }
}

impl<T, S> Mul<S> for CsrMatrix<T>
where
    T: Copy + Mul<S, Output = T>,
    S: Ring + Copy,
{
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        let _ = scalar;
        todo!("CsrMatrix::mul_scalar_op")
    }
}

impl<T, S> MulAssign<S> for CsrMatrix<T>
where
    T: Copy + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        let _ = scalar;
        todo!("CsrMatrix::mul_assign_scalar_op")
    }
}
