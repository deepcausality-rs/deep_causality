/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator impls that carry `DenseMatrix` into the tower, and the access-trait impls.
//!
//! The law markers in `algebra/` state which laws hold. They reach nothing on their own: `Ring`
//! also needs `Add`, `Sub`, `Neg`, `Mul`, `Zero` and `One` to be present, and `Module<S>` needs
//! `Mul<S>` and `MulAssign<S>`. These are those.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_build::MatrixBuild;
use crate::traits::matrix_view::MatrixView;
use crate::traits::row_ops::RowOps;
use crate::types::dense_matrix::DenseMatrix;
use core::ops::{Add, Mul, MulAssign, Neg, Sub};
use deep_causality_algebra::{CommutativeRing, CommutativeSemiring, Field, Ring};
use deep_causality_num::{One, Zero};

impl<T> MatrixView for DenseMatrix<T>
where
    T: Zero + Clone,
{
    type Scalar = T;

    fn rows(&self) -> usize {
        todo!("DenseMatrix::rows")
    }

    fn cols(&self) -> usize {
        todo!("DenseMatrix::cols")
    }

    fn get(&self, row: usize, col: usize) -> Result<T, LinearError> {
        let _ = (row, col);
        todo!("DenseMatrix::get")
    }
}

impl<T> MatrixBuild for DenseMatrix<T>
where
    T: Zero + Clone,
{
    fn zeros(rows: usize, cols: usize) -> Self {
        let _ = (rows, cols);
        todo!("DenseMatrix::zeros")
    }

    fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), LinearError> {
        let _ = (row, col, value);
        todo!("DenseMatrix::set")
    }
}

/// The dense layout is what makes the row operations cheap: each row's entries are contiguous, so
/// `axpy_rows` walks one slice and `swap_rows` exchanges two.
impl<T> RowOps for DenseMatrix<T>
where
    T: Field + Clone,
{
    fn swap_rows(&mut self, a: usize, b: usize) -> Result<(), LinearError> {
        let _ = (a, b);
        todo!("DenseMatrix::swap_rows")
    }

    fn scale_row(&mut self, row: usize, factor: &T, from_col: usize) -> Result<(), LinearError> {
        let _ = (row, factor, from_col);
        todo!("DenseMatrix::scale_row")
    }

    fn axpy_rows(
        &mut self,
        dst: usize,
        src: usize,
        factor: &T,
        from_col: usize,
    ) -> Result<(), LinearError> {
        let _ = (dst, src, factor, from_col);
        todo!("DenseMatrix::axpy_rows")
    }
}

// ---- the structural impls the tower reads ------------------------------------------------------

impl<T> Zero for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    fn zero() -> Self {
        todo!("DenseMatrix::zero")
    }

    fn is_zero(&self) -> bool {
        todo!("DenseMatrix::is_zero")
    }
}

impl<T> One for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    fn one() -> Self {
        todo!("DenseMatrix::one")
    }

    fn is_one(&self) -> bool {
        todo!("DenseMatrix::is_one")
    }
}

impl<T> Add for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("DenseMatrix::add")
    }
}

impl<T> Sub for DenseMatrix<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("DenseMatrix::sub")
    }
}

impl<T> Neg for DenseMatrix<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn neg(self) -> Self {
        todo!("DenseMatrix::neg")
    }
}

/// Matrix multiplication, which is what makes this a `Ring` and not a `CommutativeRing`.
impl<T> Mul for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("DenseMatrix::mul")
    }
}

/// Scaling by a ring element, which is what `Module<S>` reads.
impl<T, S> Mul<S> for DenseMatrix<T>
where
    T: Clone + Mul<S, Output = T>,
    S: Ring + Copy,
{
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        let _ = scalar;
        todo!("DenseMatrix::mul_scalar")
    }
}

impl<T, S> MulAssign<S> for DenseMatrix<T>
where
    T: Clone + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        let _ = scalar;
        todo!("DenseMatrix::mul_assign_scalar")
    }
}
