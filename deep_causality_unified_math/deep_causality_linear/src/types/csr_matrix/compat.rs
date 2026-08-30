/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inherent surface `deep_causality_sparse` published, kept so that repointing a caller is a
//! change of `use` line and nothing else.
//!
//! # Why these are here rather than left to the operators
//!
//! `CsrMatrix` already implements `Add`, `Sub`, `Neg` and `Mul` in every combination of owned and
//! borrowed operands, so `&a + &b` works and these look redundant. They are not, for a mechanical
//! reason: Rust's method probe tries the by-value receiver first, so `self.weights.add(&rhs.weights)`
//! resolves to `Add<&CsrMatrix<T>>::add`, which takes `self` by value, and fails E0507 against a
//! borrowed field. It never falls through to a `&self` method that does not exist.
//!
//! `deep_causality_topology`'s `Chain` is written that way at four sites. Without these it does not
//! compile, and "add a `.clone()`" is not a migration — it is a behaviour-preserving change that
//! allocates a matrix per operation in the crate's hottest algebra.
//!
//! # The contextual zero
//!
//! [`from_triplets_with_zero`](CsrMatrix::from_triplets_with_zero) and
//! [`add_with_zero`](CsrMatrix::add_with_zero) take the value to treat as absent rather than
//! assuming `T::zero()`. That is not a convenience: a chain over a semiring where the additive
//! identity is `1` — which `chain_algebra_tests.rs:140` builds — has no `T::zero()` worth dropping,
//! and using one would store the wrong pattern. The plain constructors keep `T::zero()`.

use crate::errors::linear_error::LinearError;
use crate::types::csr_matrix::CsrMatrix;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Neg;
use deep_causality_algebra::{CommutativeSemiring, Module, Ring};
use deep_causality_num::{One, Zero};

impl<T> CsrMatrix<T> {
    /// The zero matrix of the given shape.
    ///
    /// Distinct from [`Zero::zero`], which takes no shape and gives the `0x0` matrix. The row
    /// pointer is `rows + 1` entries of zero, so the structure is empty but the shape is not.
    pub fn zero(rows: usize, cols: usize) -> Self {
        Self::from_parts(vec![0; rows + 1], Vec::new(), Vec::new(), (rows, cols))
    }
}

impl<T> CsrMatrix<T>
where
    T: Clone + PartialEq + core::ops::Add<Output = T>,
{
    /// Entrywise addition, treating `absent` as the value a position outside the stored pattern
    /// holds and the value a result is dropped for.
    ///
    /// Bounded on `Add` and `PartialEq` alone — not on a semiring — because adding two matrices
    /// needs neither multiplication nor its laws. The narrower bound is what lets
    /// `deep_causality_topology`'s `Chain<T, S>` reach this at its own `AbelianGroup` scalar.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the shapes differ.
    pub fn add_with_zero(&self, other: &Self, absent: T) -> Result<Self, LinearError> {
        if self.shape() != other.shape() {
            return Err(LinearError::ShapeMismatch(self.shape(), other.shape()));
        }
        let (rows, cols) = self.shape();

        // Both operands keep each row's columns sorted, so the sum is a merge of two sorted runs.
        // Reading every position instead would cost `rows * cols` against a stored count that is
        // the whole reason for the representation.
        let max_nnz = self.values().len() + other.values().len();
        let mut row_indices = Vec::with_capacity(rows + 1);
        let mut col_indices = Vec::with_capacity(max_nnz);
        let mut values: Vec<T> = Vec::with_capacity(max_nnz);
        row_indices.push(0usize);

        for i in 0..rows {
            let (mut pa, ea) = (self.row_indices()[i], self.row_indices()[i + 1]);
            let (mut pb, eb) = (other.row_indices()[i], other.row_indices()[i + 1]);

            while pa < ea && pb < eb {
                let (ca, cb) = (self.col_indices()[pa], other.col_indices()[pb]);
                let (col, val) = if ca < cb {
                    pa += 1;
                    (ca, self.values()[pa - 1].clone())
                } else if cb < ca {
                    pb += 1;
                    (cb, other.values()[pb - 1].clone())
                } else {
                    pa += 1;
                    pb += 1;
                    (
                        ca,
                        self.values()[pa - 1].clone() + other.values()[pb - 1].clone(),
                    )
                };
                if val != absent {
                    col_indices.push(col);
                    values.push(val);
                }
            }
            // Only one of these runs: the loop above exhausted the other side.
            while pa < ea {
                let val = self.values()[pa].clone();
                if val != absent {
                    col_indices.push(self.col_indices()[pa]);
                    values.push(val);
                }
                pa += 1;
            }
            while pb < eb {
                let val = other.values()[pb].clone();
                if val != absent {
                    col_indices.push(other.col_indices()[pb]);
                    values.push(val);
                }
                pb += 1;
            }
            row_indices.push(values.len());
        }

        Ok(Self::from_parts(
            row_indices,
            col_indices,
            values,
            (rows, cols),
        ))
    }
}

impl<T> CsrMatrix<T>
where
    T: Clone + PartialEq + Zero + core::ops::Add<Output = T>,
{
    /// Entrywise addition.
    ///
    /// # Panics
    ///
    /// If the shapes differ. [`add_matrix`](CsrMatrix::add_matrix) is the same operation returning
    /// a [`Result`].
    pub fn add(&self, rhs: &Self) -> Self {
        self.add_with_zero(rhs, T::zero())
            .expect("CsrMatrix shape mismatch in add")
    }
}

impl<T> CsrMatrix<T>
where
    T: Copy + PartialEq + Zero + core::ops::Add<Output = T> + Neg<Output = T>,
{
    /// Entrywise subtraction.
    ///
    /// Built as `self + (-rhs)`, so it inherits the merge above rather than walking every position.
    ///
    /// # Panics
    ///
    /// If the shapes differ. [`sub_matrix`](CsrMatrix::sub_matrix) is the same operation returning
    /// a [`Result`].
    pub fn sub(&self, rhs: &Self) -> Self {
        self.sub_matrix(rhs)
            .expect("CsrMatrix shape mismatch in sub")
    }

    /// Entrywise subtraction, reporting a shape mismatch rather than panicking.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the shapes differ.
    pub fn sub_matrix(&self, other: &Self) -> Result<Self, LinearError> {
        if self.shape() != other.shape() {
            return Err(LinearError::ShapeMismatch(self.shape(), other.shape()));
        }
        self.add_with_zero(&other.neg(), T::zero())
    }
}

impl<T> CsrMatrix<T>
where
    T: Copy + Neg<Output = T>,
{
    /// Entrywise negation.
    ///
    /// The stored pattern is unchanged: negating a non-zero cannot produce a zero in a ring
    /// without zero divisors, and the structural zeros stay structural.
    pub fn neg(&self) -> Self {
        let (ri, ci, vals, shape) = self.parts_cloned();
        Self::from_parts(ri, ci, vals.into_iter().map(|v| -v).collect(), shape)
    }
}

impl<T> CsrMatrix<T> {
    /// Scalar multiplication, generic in the scalar's own type.
    ///
    /// `scale::<S>` is what carries `Module<S>` for a `T` whose scalar ring is not `T` itself —
    /// which is the case `deep_causality_topology`'s `Chain<T, S>` is built on.
    /// [`scalar_mult`](CsrMatrix::scalar_mult) is the narrower `S = T` form.
    pub fn scale<S>(&self, scalar: S) -> Self
    where
        T: Module<S> + Copy,
        S: Ring + Copy,
    {
        let (ri, ci, vals, shape) = self.parts_cloned();
        Self::from_parts(
            ri,
            ci,
            vals.into_iter().map(|v| v * scalar).collect(),
            shape,
        )
    }
}

impl<T> CsrMatrix<T>
where
    T: CommutativeSemiring + Ring + One + Copy + Default + PartialEq,
{
    /// The `size x size` identity.
    ///
    /// Distinct from [`One::one`], which takes no size.
    pub fn one(size: usize) -> Self {
        let mut row_indices = vec![0usize; size + 1];
        let mut col_indices = Vec::with_capacity(size);
        let mut values = Vec::with_capacity(size);
        for i in 0..size {
            col_indices.push(i);
            values.push(T::one());
            row_indices[i + 1] = i + 1;
        }
        Self::from_parts(row_indices, col_indices, values, (size, size))
    }

    /// Matrix multiplication.
    ///
    /// # Panics
    ///
    /// If the inner dimensions do not meet. [`mat_mult`](CsrMatrix::mat_mult) is the same
    /// operation returning a [`Result`].
    pub fn mul(&self, rhs: &Self) -> Self {
        self.mat_mult(rhs)
            .expect("CsrMatrix dimension mismatch in mul")
    }
}

impl<T> CsrMatrix<T>
where
    T: CommutativeSemiring + Copy + PartialEq,
{
    /// Builds from triplets, treating `zero` as the value to leave unstored.
    ///
    /// [`from_triplets`](CsrMatrix::from_triplets) drops `T::zero()`; this drops whatever the
    /// caller names. Duplicate positions are **summed**, as they are there.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if a triplet names a position outside the shape.
    pub fn from_triplets_with_zero(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, T)],
        zero: T,
    ) -> Result<Self, LinearError> {
        for &(r, c, _) in triplets {
            if r >= rows || c >= cols {
                return Err(LinearError::IndexOutOfBounds((r, c), (rows, cols)));
            }
        }
        let mut sorted: Vec<(usize, usize, T)> = triplets.to_vec();
        sorted.sort_by_key(|&(r, c, _)| (r, c));

        let mut row_indices = vec![0usize; rows + 1];
        let mut col_indices = Vec::with_capacity(sorted.len());
        let mut values: Vec<T> = Vec::with_capacity(sorted.len());

        let mut i = 0usize;
        while i < sorted.len() {
            let (r, c, mut acc) = sorted[i];
            let mut j = i + 1;
            while j < sorted.len() && sorted[j].0 == r && sorted[j].1 == c {
                acc = acc + sorted[j].2;
                j += 1;
            }
            i = j;
            if acc == zero {
                continue;
            }
            col_indices.push(c);
            values.push(acc);
            row_indices[r + 1] += 1;
        }
        for i in 0..rows {
            row_indices[i + 1] += row_indices[i];
        }
        Ok(Self::from_parts(
            row_indices,
            col_indices,
            values,
            (rows, cols),
        ))
    }
}

impl<T> CsrMatrix<T> {
    /// The entry at `(row, col)`, or `absent` when the position is outside the stored pattern.
    ///
    /// The contextual-zero counterpart of [`get_value_at`](CsrMatrix::get_value_at), which uses
    /// `T::zero()`.
    pub fn value_at_or(&self, row: usize, col: usize, absent: T) -> T
    where
        T: Copy,
    {
        let (r, _) = self.shape();
        if row >= r {
            return absent;
        }
        let start = self.row_indices()[row];
        let end = self.row_indices()[row + 1];
        for k in start..end {
            if self.col_indices()[k] == col {
                return self.values()[k];
            }
        }
        absent
    }

    /// The four CSR arrays, cloned.
    fn parts_cloned(&self) -> (Vec<usize>, Vec<usize>, Vec<T>, (usize, usize))
    where
        T: Clone,
    {
        (
            self.row_indices().clone(),
            self.col_indices().clone(),
            self.values().clone(),
            self.shape(),
        )
    }
}
