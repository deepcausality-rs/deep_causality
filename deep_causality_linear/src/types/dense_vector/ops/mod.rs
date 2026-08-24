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
        todo!("DenseVector::zero")
    }
    fn is_zero(&self) -> bool {
        todo!("DenseVector::is_zero")
    }
}

impl<T> Add for DenseVector<T>
where
    T: CommutativeSemiring + Clone,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("DenseVector::add_op")
    }
}

impl<T> Sub for DenseVector<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let _ = rhs;
        todo!("DenseVector::sub_op")
    }
}

impl<T> Neg for DenseVector<T>
where
    T: CommutativeRing + Clone,
{
    type Output = Self;
    fn neg(self) -> Self {
        todo!("DenseVector::neg_op")
    }
}

impl<T, S> Mul<S> for DenseVector<T>
where
    T: Clone + Mul<S, Output = T>,
    S: Ring + Copy,
{
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        let _ = scalar;
        todo!("DenseVector::mul_scalar")
    }
}

impl<T, S> MulAssign<S> for DenseVector<T>
where
    T: Clone + MulAssign<S>,
    S: Ring + Copy,
{
    fn mul_assign(&mut self, scalar: S) {
        let _ = scalar;
        todo!("DenseVector::mul_assign_scalar")
    }
}
