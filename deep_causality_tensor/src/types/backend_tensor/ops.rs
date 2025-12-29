/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic and shape operations for BackendTensor.
//!
//! Note: Most shape operations (reshape, slice, permute, ravel, shifted_view)
//! are provided by the Tensor trait implementation in tensor_impl.rs.
//! This file provides operations not in the Tensor trait.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};
use core::ops::{Add, Div, Mul, Sub};

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
    /// Apply binary operation with broadcasting.
    pub fn broadcast_op<F>(&self, rhs: &Self, f: F) -> Result<Self, crate::CausalTensorError>
    where
        F: Fn(T, T) -> Result<T, crate::CausalTensorError>,
    {
        B::broadcast_op(&self.inner, &rhs.inner, f).map(Self::from_inner)
    }
}

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B>
where
    B::Tensor<T>: Clone,
{
    /// Sums elements along specified axes.
    pub fn sum(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::sum(&self.inner, axes))
    }

    /// Finds maximum along specified axes.
    pub fn max(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::max(&self.inner, axes))
    }

    /// Calculates the mean along specified axes.
    pub fn mean(&self, axes: &[usize]) -> Self
    where
        T: From<u32>,
    {
        Self::from_inner(B::mean(&self.inner, axes))
    }

    /// Returns indicies that would sort the tensor (1D only).
    pub fn arg_sort(&self) -> Vec<usize> {
        B::arg_sort(&self.inner)
    }

    /// Executes an Einstein summation AST.
    pub fn ein_sum(
        ast: &crate::types::cpu_tensor::EinSumAST<Self>,
    ) -> Result<Self, crate::CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
    {
        use crate::types::cpu_tensor::EinSumAST;

        // Recursive helper to map the AST.
        fn map_ast_recursive<T, B, F>(
            ast: &EinSumAST<BackendTensor<T, B>>,
            f: &F,
        ) -> EinSumAST<B::Tensor<T>>
        where
            T: TensorData,
            B: TensorBackend,
            B::Tensor<T>: Clone, // Required for EinSumOp::TensorSource
            F: Fn(BackendTensor<T, B>) -> B::Tensor<T>,
        {
            let current_op = ast.value();

            // Map the generic EinSumOp to the target tensor type.
            // We use map_tensor which takes a closure converting the tensor.
            // Note: EinSumOp::map_tensor consumes self, but we have reference.
            // We clone the op first (EinSumOp is Clone).
            let new_op = current_op.clone().map_tensor(f);

            let children = ast.children();
            if children.is_empty() {
                EinSumAST::new(new_op)
            } else {
                let new_children: Vec<_> =
                    children.iter().map(|c| map_ast_recursive(c, f)).collect();
                EinSumAST::with_children(new_op, new_children)
            }
        }

        // Convert wrapper function
        let converter = |t: Self| t.into_inner();

        let inner_ast = map_ast_recursive(ast, &converter);

        B::ein_sum(&inner_ast).map(Self::from_inner)
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

// --- Scalar Multiplication for f64 ---

impl<B: TensorBackend> Mul<f64> for BackendTensor<f64, B> {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner)
            .into_iter()
            .map(|x| x * rhs)
            .collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Mul<BackendTensor<f64, B>> for f64 {
    type Output = BackendTensor<f64, B>;

    fn mul(self, rhs: BackendTensor<f64, B>) -> Self::Output {
        rhs * self
    }
}

impl<B: TensorBackend> Mul<f64> for &BackendTensor<f64, B> {
    type Output = BackendTensor<f64, B>;

    fn mul(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner)
            .into_iter()
            .map(|x| x * rhs)
            .collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Multiplication for f32 ---

impl<B: TensorBackend> Mul<f32> for BackendTensor<f32, B> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner)
            .into_iter()
            .map(|x| x * rhs)
            .collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Mul<BackendTensor<f32, B>> for f32 {
    type Output = BackendTensor<f32, B>;

    fn mul(self, rhs: BackendTensor<f32, B>) -> Self::Output {
        rhs * self
    }
}

// --- Scalar Addition for f64 ---

impl<B: TensorBackend> Add<f64> for BackendTensor<f64, B> {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Add<BackendTensor<f64, B>> for f64 {
    type Output = BackendTensor<f64, B>;

    fn add(self, rhs: BackendTensor<f64, B>) -> Self::Output {
        rhs + self
    }
}

// --- Scalar Subtraction for f64 ---

impl<B: TensorBackend> Sub<f64> for BackendTensor<f64, B> {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Division for f64 ---

impl<B: TensorBackend> Div<f64> for BackendTensor<f64, B> {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Arithmetic for f32 ---

impl<B: TensorBackend> Add<f32> for BackendTensor<f32, B> {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Sub<f32> for BackendTensor<f32, B> {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Div<f32> for BackendTensor<f32, B> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
        Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Addition for f64 (Reference) ---

impl<B: TensorBackend> Add<f64> for &BackendTensor<f64, B> {
    type Output = BackendTensor<f64, B>;

    fn add(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Subtraction for f64 (Reference) ---

impl<B: TensorBackend> Sub<f64> for &BackendTensor<f64, B> {
    type Output = BackendTensor<f64, B>;

    fn sub(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Division for f64 (Reference) ---

impl<B: TensorBackend> Div<f64> for &BackendTensor<f64, B> {
    type Output = BackendTensor<f64, B>;

    fn div(self, rhs: f64) -> Self::Output {
        let data: Vec<f64> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

// --- Scalar Arithmetic for f32 (Reference) ---

impl<B: TensorBackend> Add<f32> for &BackendTensor<f32, B> {
    type Output = BackendTensor<f32, B>;

    fn add(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Sub<f32> for &BackendTensor<f32, B> {
    type Output = BackendTensor<f32, B>;

    fn sub(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}

impl<B: TensorBackend> Div<f32> for &BackendTensor<f32, B> {
    type Output = BackendTensor<f32, B>;

    fn div(self, rhs: f32) -> Self::Output {
        let data: Vec<f32> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}
