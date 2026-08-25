/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator impls that carry `DenseVector` into the tower.
//!
//! Additive only, plus scaling. There is no `Mul` on this type: the dot product returns a scalar and
//! the outer product returns a matrix, so neither is a product of two vectors giving a vector.
//! That places the type at `AbelianGroup` and `Module<R>`, which is what a vector space is.

use crate::types::dense_vector::DenseVector;
use core::ops::{Add, Mul, MulAssign, Neg, Sub};
use deep_causality_algebra::{CommutativeRing, CommutativeSemiring, Ring};
use deep_causality_num::Zero;

impl<T> Zero for DenseVector<T>
where
    T: CommutativeSemiring + Clone,
{
    fn zero() -> Self {
        DenseVector::from_vec(alloc::vec::Vec::new())
    }
    fn is_zero(&self) -> bool {
        self.as_slice().iter().all(|v| v.is_zero())
    }
}

impl<T> Add for DenseVector<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.len(), rhs.len(), "length mismatch in add");
        DenseVector::from_vec(
            self.into_data()
                .into_iter()
                .zip(rhs.into_data())
                .map(|(a, b)| a + b)
                .collect(),
        )
    }
}

impl<T> Sub for DenseVector<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.len(), rhs.len(), "length mismatch in sub");
        DenseVector::from_vec(
            self.into_data()
                .into_iter()
                .zip(rhs.into_data())
                .map(|(a, b)| a - b)
                .collect(),
        )
    }
}

impl<T> Neg for DenseVector<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn neg(self) -> Self {
        DenseVector::from_vec(
            self.into_data()
                .into_iter()
                .map(|v| T::zero() - v)
                .collect(),
        )
    }
}

impl<T, S> Mul<S> for DenseVector<T>
where
    T: Clone + Mul<S, Output = T>,
    S: Ring + Copy,
{
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        DenseVector::from_vec(self.into_data().into_iter().map(|v| v * scalar).collect())
    }
}

impl<T, S> MulAssign<S> for DenseVector<T>
where
    T: Clone + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        for v in self.as_mut_data() {
            *v *= scalar;
        }
    }
}
