/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic and shape operations for BackendTensor.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};
use core::ops::{Add, Div, Mul, Range, Sub};

// --- Shape Operations ---

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B> {
    /// Reshapes the tensor to the given dimensions.
    pub fn reshape(&self, shape: &[usize]) -> Self {
        Self::from_inner(B::reshape(&self.inner, shape))
    }

    /// Permutes the axes of the tensor.
    pub fn permute(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::permute(&self.inner, axes))
    }

    /// Extracts a slice of the tensor.
    pub fn slice(&self, ranges: &[Range<usize>]) -> Self {
        Self::from_inner(B::slice(&self.inner, ranges))
    }

    /// Sums elements along specified axes.
    pub fn sum(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::sum(&self.inner, axes))
    }

    /// Finds maximum along specified axes.
    pub fn max(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::max(&self.inner, axes))
    }
}

// --- Arithmetic Trait Implementations ---

impl<T: TensorData, B: TensorBackend> Add for BackendTensor<T, B> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::add(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Add for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn add(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::add(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Sub for BackendTensor<T, B> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Sub for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn sub(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Mul for BackendTensor<T, B> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Mul for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn mul(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Div for BackendTensor<T, B> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::div(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Div for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn div(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::div(&self.inner, &rhs.inner))
    }
}
