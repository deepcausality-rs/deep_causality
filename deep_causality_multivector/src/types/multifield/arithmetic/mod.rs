/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic operators for CausalMultiField.

use super::CausalMultiField;
use crate::types::multifield::ops::batched_matmul::BatchedMatMul;
use deep_causality_num::{Field, Ring, Zero};
use deep_causality_tensor::CausalTensor;
use std::ops::{Add, Mul, Neg, Sub};

// === Zero Implementation ===

impl<T> Zero for CausalMultiField<T>
where
    T: Field
        + Copy
        + Default
        + PartialOrd
        + Send
        + Sync
        + 'static
        + Zero
        + std::ops::Neg<Output = T>,
{
    fn zero() -> Self {
        panic!(
            "CausalMultiField::zero() requires context (Metric/Shape/Dx). Use zeros() factory method instead."
        )
    }

    fn is_zero(&self) -> bool {
        // Download and check all coefficients
        let mvs = self.to_coefficients();
        mvs.iter().all(|mv| mv.data.iter().all(|c| c.is_zero()))
    }
}

// === Add Implementation ===

impl<T> Add for CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + 'static,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metrics must match for addition");
        assert_eq!(self.shape, rhs.shape, "Shapes must match for addition");

        let result = &self.data + &rhs.data;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<T> Add for &CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + 'static,
{
    type Output = CausalMultiField<T>;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.metric, rhs.metric, "Metrics must match for addition");
        assert_eq!(self.shape, rhs.shape, "Shapes must match for addition");

        let result = &self.data + &rhs.data;

        CausalMultiField {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Sub Implementation ===

impl<T> Sub for CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + 'static,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metrics must match for subtraction"
        );
        assert_eq!(self.shape, rhs.shape, "Shapes must match for subtraction");

        let result = &self.data - &rhs.data;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<T> Sub for &CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + 'static,
{
    type Output = CausalMultiField<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metrics must match for subtraction"
        );
        assert_eq!(self.shape, rhs.shape, "Shapes must match for subtraction");

        let result = &self.data - &rhs.data;

        CausalMultiField {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Neg Implementation ===

impl<T> Neg for CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + 'static + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        let neg_one = -T::one();
        let neg_tensor = CausalTensor::<T>::from_shape_fn(&[1], |_| neg_one);
        let result = &self.data * &neg_tensor;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// === Mul Implementation (Geometric Product) ===

impl<T> Mul for CausalMultiField<T>
where
    T: Field + Ring + Copy + Default + PartialOrd + Send + Sync + 'static,
    CausalTensor<T>: BatchedMatMul<T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metrics must match for multiplication"
        );
        assert_eq!(
            self.shape, rhs.shape,
            "Shapes must match for multiplication"
        );

        let result = self.data.batched_matmul(&rhs.data);

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<T> Mul for &CausalMultiField<T>
where
    T: Field + Ring + Copy + Default + PartialOrd + Send + Sync + 'static,
    CausalTensor<T>: BatchedMatMul<T>,
{
    type Output = CausalMultiField<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metrics must match for multiplication"
        );
        assert_eq!(
            self.shape, rhs.shape,
            "Shapes must match for multiplication"
        );

        let result = self.data.batched_matmul(&rhs.data);

        CausalMultiField {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

impl<T> Mul<&CausalMultiField<T>> for CausalMultiField<T>
where
    T: Field + Ring + Copy + Default + PartialOrd + Send + Sync + 'static,
    CausalTensor<T>: BatchedMatMul<T>,
{
    type Output = CausalMultiField<T>;

    fn mul(self, rhs: &CausalMultiField<T>) -> Self::Output {
        assert_eq!(
            self.metric, rhs.metric,
            "Metrics must match for multiplication"
        );
        assert_eq!(
            self.shape, rhs.shape,
            "Shapes must match for multiplication"
        );

        let result = self.data.batched_matmul(&rhs.data);

        CausalMultiField {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}
