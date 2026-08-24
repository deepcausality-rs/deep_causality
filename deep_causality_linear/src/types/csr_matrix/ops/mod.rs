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
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use deep_causality_algebra::{CommutativeRing, CommutativeSemiring, Ring};
use deep_causality_num::{One, Zero};

impl<T> MatrixView for CsrMatrix<T>
where
    T: Zero + Clone,
{
    type Scalar = T;

    fn rows(&self) -> usize {
        self.shape().0
    }
    fn cols(&self) -> usize {
        self.shape().1
    }

    /// A position inside the shape but outside the stored pattern returns the scalar zero. It is
    /// genuinely zero, and a caller asking for it has done nothing wrong.
    fn get(&self, row: usize, col: usize) -> Result<T, LinearError> {
        let (r, c) = self.shape();
        if row >= r || col >= c {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, col),
                shape: (r, c),
            });
        }
        // A position inside the shape but outside the stored pattern is a zero, not an error.
        let (start, end) = (self.row_indices()[row], self.row_indices()[row + 1]);
        for k in start..end {
            if self.col_indices()[k] == col {
                return Ok(self.values()[k].clone());
            }
        }
        Ok(T::zero())
    }
}

impl<T> MatrixBuild for CsrMatrix<T>
where
    T: Zero + Clone,
{
    /// Stores nothing, so the cost is the row-pointer array rather than `rows * cols`.
    fn zeros(rows: usize, cols: usize) -> Self {
        CsrMatrix::with_capacity(rows, cols, 0)
    }

    fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), LinearError> {
        let (r, c) = self.shape();
        if row >= r || col >= c {
            return Err(LinearError::IndexOutOfBounds {
                index: (row, col),
                shape: (r, c),
            });
        }
        // Rebuilding is O(nnz) per write and this is not the construction path a caller should
        // reach for in a loop; `from_triplets` builds in one pass. `set` exists because
        // `MatrixBuild` needs it, and correctness matters more here than the constant.
        let mut triplets = alloc::vec::Vec::with_capacity(self.values().len() + 1);
        let mut replaced = false;
        for i in 0..r {
            for k in self.row_indices()[i]..self.row_indices()[i + 1] {
                let j = self.col_indices()[k];
                if i == row && j == col {
                    triplets.push((i, j, value.clone()));
                    replaced = true;
                } else {
                    triplets.push((i, j, self.values()[k].clone()));
                }
            }
        }
        if !replaced {
            triplets.push((row, col, value));
        }
        triplets.sort_by_key(|&(i, j, _)| (i, j));

        let mut row_indices = alloc::vec![0usize; r + 1];
        let mut col_indices = alloc::vec::Vec::new();
        let mut values = alloc::vec::Vec::new();
        for (i, j, v) in triplets {
            if v.is_zero() {
                continue;
            }
            col_indices.push(j);
            values.push(v);
            row_indices[i + 1] += 1;
        }
        for i in 0..r {
            row_indices[i + 1] += row_indices[i];
        }
        *self = CsrMatrix::from_raw_parts(row_indices, col_indices, values, (r, c));
        Ok(())
    }
}

impl<T> Zero for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn zero() -> Self {
        CsrMatrix::new()
    }
    fn is_zero(&self) -> bool {
        self.values().iter().all(|v| v.is_zero())
    }
}

impl<T> One for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn one() -> Self {
        // The 1x1 identity: `One` has no size to work from, so it takes the smallest.
        CsrMatrix::from_triplets(1, 1, &[(0, 0, T::one())])
            .expect("a 1x1 triplet is inside a 1x1 shape")
    }
    fn is_one(&self) -> bool {
        let (r, c) = self.shape();
        if r != c {
            return false;
        }
        (0..r).all(|i| {
            (0..c).all(|j| {
                let v = self.get_value_at(i, j);
                if i == j { v == T::one() } else { v.is_zero() }
            })
        })
    }
}

impl<T> Add for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add_matrix(&rhs)
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> Sub for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let neg = rhs.scalar_mult(T::zero() - T::one());
        self.add_matrix(&neg)
            .expect("CsrMatrix shape mismatch in sub")
    }
}

impl<T> Neg for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = Self;
    fn neg(self) -> Self {
        let minus_one = T::zero() - T::one();
        self.scalar_mult(minus_one)
    }
}

impl<T> Mul for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mat_mult(&rhs)
            .expect("CsrMatrix dimension mismatch in mul")
    }
}

impl<T, S> Mul<S> for CsrMatrix<T>
where
    T: Copy + Mul<S, Output = T>,
    S: Ring + Copy,
{
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        let (ri, ci, vals, shape) = self.into_parts();
        CsrMatrix::from_raw_parts(
            ri,
            ci,
            vals.into_iter().map(|v| v * scalar).collect(),
            shape,
        )
    }
}

impl<T, S> MulAssign<S> for CsrMatrix<T>
where
    T: Copy + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        let taken = core::mem::take(self);
        let (ri, ci, vals, shape) = taken.into_parts();
        let mut scaled = vals;
        for v in &mut scaled {
            *v *= scalar;
        }
        *self = CsrMatrix::from_raw_parts(ri, ci, scaled, shape);
    }
}

// ---- the borrowing and assigning forms ----------------------------------------------------------
//
// The crate this moves from implements every combination of owned and borrowed operand, plus the
// assigning forms. Porting its suite found sixteen tests that could not compile without them.
//
// They are not conveniences. Phase 5 repoints 102 import sites, and a call site written as
// `&a + &b` — which is the common shape, since neither operand is being consumed — would stop
// compiling against a type offering only the owned form.

impl<T> Add for &CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: Self) -> CsrMatrix<T> {
        self.add_matrix(rhs)
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> Add<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: &CsrMatrix<T>) -> CsrMatrix<T> {
        self.add_matrix(rhs)
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> Add<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        self.add_matrix(&rhs)
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> AddAssign for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = self
            .add_matrix(&rhs)
            .expect("CsrMatrix shape mismatch in add_assign");
    }
}

impl<T> AddAssign<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    fn add_assign(&mut self, rhs: &CsrMatrix<T>) {
        *self = self
            .add_matrix(rhs)
            .expect("CsrMatrix shape mismatch in add_assign");
    }
}

impl<T> Sub for &CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn sub(self, rhs: Self) -> CsrMatrix<T> {
        let negated = rhs.scalar_mult(T::zero() - T::one());
        self.add_matrix(&negated)
            .expect("CsrMatrix shape mismatch in sub")
    }
}

impl<T> Sub<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn sub(self, rhs: &CsrMatrix<T>) -> CsrMatrix<T> {
        let negated = rhs.scalar_mult(T::zero() - T::one());
        self.add_matrix(&negated)
            .expect("CsrMatrix shape mismatch in sub")
    }
}

impl<T> Sub<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn sub(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        let negated = rhs.scalar_mult(T::zero() - T::one());
        self.add_matrix(&negated)
            .expect("CsrMatrix shape mismatch in sub")
    }
}

impl<T> SubAssign for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    fn sub_assign(&mut self, rhs: Self) {
        let negated = rhs.scalar_mult(T::zero() - T::one());
        *self = self
            .add_matrix(&negated)
            .expect("CsrMatrix shape mismatch in sub_assign");
    }
}

impl<T> SubAssign<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    fn sub_assign(&mut self, rhs: &CsrMatrix<T>) {
        let negated = rhs.scalar_mult(T::zero() - T::one());
        *self = self
            .add_matrix(&negated)
            .expect("CsrMatrix shape mismatch in sub_assign");
    }
}

impl<T> Neg for &CsrMatrix<T>
where
    T: CommutativeRing + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn neg(self) -> CsrMatrix<T> {
        self.scalar_mult(T::zero() - T::one())
    }
}

impl<T> Mul for &CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    type Output = CsrMatrix<T>;
    fn mul(self, rhs: Self) -> CsrMatrix<T> {
        self.mat_mult(rhs)
            .expect("CsrMatrix dimension mismatch in mul")
    }
}
