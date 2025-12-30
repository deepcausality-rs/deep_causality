/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ring and Module trait implementations for CausalMultiField.
//!
// These implementations dispatch to the backend for all operations,
// using the Matrix Isomorphism representation.

use super::CausalMultiField;
use crate::types::multifield::ops::batched_matmul::BatchedMatMul;
use deep_causality_num::{Ring, Zero};
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};
use std::ops::{Add, Mul, Neg, Sub};

// === Zero Implementation ===

impl<B, T> Zero for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData + Zero,
{
    fn zero() -> Self {
        panic!(
            "CausalMultiField::zero() requires shape and metric - use CausalMultiField::zeros() instead"
        );
    }

    fn is_zero(&self) -> bool {
        // Check if all elements are zero by summing and comparing
        let data_vec = B::to_vec(&self.data);
        data_vec.iter().all(|x| x.is_zero())
    }
}

// === Add Implementation ===

impl<B, T> Add for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in add");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in add");

        let result = B::add(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<B, T> Add<&Self> for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in add");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in add");

        let result = B::add(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Sub Implementation ===

impl<B, T> Sub for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in sub");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in sub");

        let result = B::sub(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<B, T> Sub<&Self> for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch in sub");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in sub");

        let result = B::sub(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Neg Implementation ===

impl<B, T> Neg for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        // Negate by multiplying with -1 tensor
        let neg_one = B::from_shape_fn(&[1], |_| -T::one());
        let result = B::mul(&self.data, &neg_one);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Mul (Geometric Product) Implementation ===

impl<B, T> Mul for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T>,
    T: TensorData + Ring + Default + PartialOrd,
{
    type Output = Self;

    /// Geometric product of two multifields.
    ///
    /// Since data is stored in Matrix Representation, this is a direct matmul.
    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metric mismatch in geometric product"
        );
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in geometric product");

        let result = B::batched_matmul(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<B, T> Mul<&Self> for CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T>,
    T: TensorData + Ring + Default + PartialOrd,
{
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metric mismatch in geometric product"
        );
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in geometric product");

        let result = B::batched_matmul(&self.data, &rhs.data);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<B, T> Mul for &CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T>,
    T: TensorData + Ring + Default + PartialOrd,
{
    type Output = CausalMultiField<B, T>;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metric mismatch in geometric product"
        );
        assert_eq!(self.shape, rhs.shape, "Shape mismatch in geometric product");

        let result = B::batched_matmul(&self.data, &rhs.data);
        CausalMultiField {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}
