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
        self.row_count()
    }

    fn cols(&self) -> usize {
        self.col_count()
    }

    fn get(&self, row: usize, col: usize) -> Result<T, LinearError> {
        if row >= self.row_count() || col >= self.col_count() {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, col),
                shape: (self.row_count(), self.col_count()),
            });
        }
        Ok(self.as_slice()[row * self.col_count() + col].clone())
    }
}

impl<T> MatrixBuild for DenseMatrix<T>
where
    T: Zero + Clone,
{
    fn zeros(rows: usize, cols: usize) -> Self {
        DenseMatrix::from_vec(alloc::vec![T::zero(); rows * cols], rows, cols)
            .expect("the buffer is built from the shape, so it matches by construction")
    }

    fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), LinearError> {
        let (r, c) = (self.row_count(), self.col_count());
        if row >= r || col >= c {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, col),
                shape: (r, c),
            });
        }
        self.as_mut_slice()[row * c + col] = value;
        Ok(())
    }
}

/// The dense layout is what makes the row operations cheap: each row's entries are contiguous, so
/// `axpy_rows` walks one slice and `swap_rows` exchanges two.
impl<T> RowOps for DenseMatrix<T>
where
    T: Field + Clone,
{
    fn swap_rows(&mut self, a: usize, b: usize) -> Result<(), LinearError> {
        let (r, c) = (self.row_count(), self.col_count());
        if a >= r || b >= r {
            return Err(LinearError::IndexOutOfBounds {
                index: (a.max(b), 0),
                shape: (r, c),
            });
        }
        if a == b {
            return Ok(());
        }
        for j in 0..c {
            self.as_mut_slice().swap(a * c + j, b * c + j);
        }
        Ok(())
    }

    fn scale_row(&mut self, row: usize, factor: &T, from_col: usize) -> Result<(), LinearError> {
        let (r, c) = (self.row_count(), self.col_count());
        if row >= r {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, 0),
                shape: (r, c),
            });
        }
        for j in from_col..c {
            let v = self.as_slice()[row * c + j].clone();
            self.as_mut_slice()[row * c + j] = v * factor.clone();
        }
        Ok(())
    }

    fn axpy_rows(
        &mut self,
        dst: usize,
        src: usize,
        factor: &T,
        from_col: usize,
    ) -> Result<(), LinearError> {
        let (r, c) = (self.row_count(), self.col_count());
        if dst >= r || src >= r {
            return Err(LinearError::IndexOutOfBounds {
                index: (dst.max(src), 0),
                shape: (r, c),
            });
        }
        for j in from_col..c {
            let s = self.as_slice()[src * c + j].clone();
            let d = self.as_slice()[dst * c + j].clone();
            self.as_mut_slice()[dst * c + j] = d + s * factor.clone();
        }
        Ok(())
    }
}

// ---- the structural impls the tower reads ------------------------------------------------------

impl<T> Zero for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    fn zero() -> Self {
        DenseMatrix::from_vec(alloc::vec::Vec::new(), 0, 0).expect("0x0 holds nothing")
    }

    fn is_zero(&self) -> bool {
        self.as_slice().iter().all(|v| v.is_zero())
    }
}

impl<T> One for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    fn one() -> Self {
        // `One` has no order to work from, so it takes the smallest identity.
        DenseMatrix::from_vec(alloc::vec![T::one()], 1, 1).expect("1x1 holds one entry")
    }

    fn is_one(&self) -> bool {
        let (r, c) = (self.row_count(), self.col_count());
        if r != c {
            return false;
        }
        (0..r).all(|i| {
            (0..c).all(|j| {
                let v = &self.as_slice()[i * c + j];
                if i == j { v.is_one() } else { v.is_zero() }
            })
        })
    }
}

impl<T> Add for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (r, c) = (self.row_count(), self.col_count());
        assert_eq!(
            (r, c),
            (rhs.row_count(), rhs.col_count()),
            "shape mismatch in add"
        );
        let out: alloc::vec::Vec<T> = self
            .into_data()
            .into_iter()
            .zip(rhs.into_data())
            .map(|(a, b)| a + b)
            .collect();
        DenseMatrix::from_vec(out, r, c).expect("addition preserves the element count")
    }
}

impl<T> Sub for DenseMatrix<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let (r, c) = (self.row_count(), self.col_count());
        assert_eq!(
            (r, c),
            (rhs.row_count(), rhs.col_count()),
            "shape mismatch in sub"
        );
        let out: alloc::vec::Vec<T> = self
            .into_data()
            .into_iter()
            .zip(rhs.into_data())
            .map(|(a, b)| a - b)
            .collect();
        DenseMatrix::from_vec(out, r, c).expect("subtraction preserves the element count")
    }
}

impl<T> Neg for DenseMatrix<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn neg(self) -> Self {
        let (r, c) = (self.row_count(), self.col_count());
        let out: alloc::vec::Vec<T> = self
            .into_data()
            .into_iter()
            .map(|v| T::zero() - v)
            .collect();
        DenseMatrix::from_vec(out, r, c).expect("negation preserves the element count")
    }
}

/// Matrix multiplication, which is what makes this a `Ring` and not a `CommutativeRing`.
impl<T> Mul for DenseMatrix<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    /// Matrix multiplication, which is what makes this a `Ring` and not a `CommutativeRing`.
    fn mul(self, rhs: Self) -> Self {
        let (m, k) = (self.row_count(), self.col_count());
        let (k2, n) = (rhs.row_count(), rhs.col_count());
        assert_eq!(k, k2, "inner dimension mismatch in mul");
        let mut out = alloc::vec::Vec::with_capacity(m * n);
        for i in 0..m {
            for j in 0..n {
                let mut acc = T::zero();
                for t in 0..k {
                    acc = acc
                        + self.as_slice()[i * k + t].clone() * rhs.as_slice()[t * n + j].clone();
                }
                out.push(acc);
            }
        }
        DenseMatrix::from_vec(out, m, n).expect("the product has m*n entries")
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
        let (r, c) = (self.row_count(), self.col_count());
        let out: alloc::vec::Vec<T> = self.into_data().into_iter().map(|v| v * scalar).collect();
        DenseMatrix::from_vec(out, r, c).expect("scaling preserves the element count")
    }
}

impl<T, S> MulAssign<S> for DenseMatrix<T>
where
    T: Clone + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        for v in self.as_mut_slice() {
            *v *= scalar;
        }
    }
}
