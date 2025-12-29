/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic and shape operations for BackendTensor.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};
use core::ops::{Add, Div, Mul, Range, Sub};

// --- Shape Operations ---

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
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

    /// Flattens the tensor into a 1D vector.
    pub fn ravel(&self) -> Self {
        Self::from_inner(B::ravel(&self.inner))
    }

    /// Creates a cyclically shifted view.
    pub fn shifted_view(&self, flat_index: usize) -> Self {
        Self::from_inner(B::shifted_view(&self.inner, flat_index))
    }

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
        ast: &crate::types::causal_tensor::EinSumAST<Self>,
    ) -> Result<Self, crate::CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
    {
        use crate::types::causal_tensor::EinSumAST;

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
            let new_op = current_op.clone().map_tensor(|t| f(t));

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
