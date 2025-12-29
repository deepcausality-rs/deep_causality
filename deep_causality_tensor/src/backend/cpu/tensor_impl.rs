/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Implementation of Tensor trait for BackendTensor<T, CpuBackend>.
//!
//! This delegates all operations to the inner InternalCpuTensor and wraps results
//! to maintain the BackendTensor wrapper type.

use crate::backend::CpuBackend;
use crate::types::backend_tensor::BackendTensor;
use crate::types::cpu_tensor::EinSumAST;
use crate::{CausalTensorError, Tensor};
use core::iter::Sum;
use core::ops::{Add, Div, Mul, Neg};
use deep_causality_num::{RealField, Zero};

impl<T: Clone> Tensor<T> for BackendTensor<T, CpuBackend> {
    fn ein_sum(ast: &EinSumAST<Self>) -> Result<Self, CausalTensorError>
    where
        T: crate::backend::TensorData,
    {
        // Convert AST from BackendTensor to InternalCpuTensor
        fn convert_ast<T: crate::backend::TensorData + Clone>(
            ast: &EinSumAST<BackendTensor<T, CpuBackend>>,
        ) -> EinSumAST<crate::InternalCpuTensor<T>> {
            let op = ast.value().clone().map_tensor(|t| t.into_inner());
            let children: Vec<_> = ast.children().iter().map(convert_ast).collect();
            if children.is_empty() {
                EinSumAST::new(op)
            } else {
                EinSumAST::with_children(op, children)
            }
        }

        let inner_ast = convert_ast(ast);
        crate::InternalCpuTensor::ein_sum(&inner_ast).map(Self::from_inner)
    }

    fn matmul(&self, rhs: &Self) -> Result<Self, CausalTensorError>
    where
        T: crate::backend::TensorData,
    {
        Tensor::matmul(&self.inner, &rhs.inner).map(Self::from_inner)
    }

    fn tensor_product(&self, rhs: &Self) -> Result<Self, CausalTensorError>
    where
        T: Clone + Mul<Output = T>,
    {
        Tensor::tensor_product(&self.inner, &rhs.inner).map(Self::from_inner)
    }

    fn norm_l2(&self) -> T
    where
        T: RealField + Default + Zero + Sum + Copy,
    {
        Tensor::norm_l2(&self.inner)
    }

    fn norm_sq(&self) -> T
    where
        T: RealField + Default + Zero + Sum + Copy + Mul,
    {
        Tensor::norm_sq(&self.inner)
    }

    fn sum_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<T, Output = T>,
    {
        Tensor::sum_axes(&self.inner, axes).map(Self::from_inner)
    }

    fn mean_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Div<T, Output = T> + From<u32> + Add<T, Output = T>,
    {
        Tensor::mean_axes(&self.inner, axes).map(Self::from_inner)
    }

    fn arg_sort(&self) -> Result<Vec<usize>, CausalTensorError>
    where
        T: Clone + Default + PartialOrd,
    {
        Tensor::arg_sort(&self.inner)
    }

    fn reshape(&self, new_shape: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        Tensor::reshape(&self.inner, new_shape).map(Self::from_inner)
    }

    fn ravel(self) -> Self
    where
        T: Clone,
    {
        Self::from_inner(Tensor::ravel(self.inner))
    }

    fn slice(&self, axis: usize, index: usize) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        Tensor::slice(&self.inner, axis, index).map(Self::from_inner)
    }

    fn permute_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        Tensor::permute_axes(&self.inner, axes).map(Self::from_inner)
    }

    fn shifted_view(&self, flat_index: usize) -> Self
    where
        T: Clone,
    {
        Self::from_inner(Tensor::shifted_view(&self.inner, flat_index))
    }

    fn inverse(&self) -> Result<Self, CausalTensorError>
    where
        T: crate::backend::TensorData + RealField,
    {
        Tensor::inverse(&self.inner).map(Self::from_inner)
    }

    fn qr(&self) -> Result<(Self, Self), CausalTensorError>
    where
        T: crate::backend::TensorData + Sum + RealField + Neg<Output = T>,
    {
        Tensor::qr(&self.inner).map(|(q, r)| (Self::from_inner(q), Self::from_inner(r)))
    }

    fn svd(&self) -> Result<(Self, Self, Self), CausalTensorError>
    where
        T: crate::backend::TensorData + Sum + RealField,
    {
        Tensor::svd(&self.inner).map(|(u, s, v)| {
            (
                Self::from_inner(u),
                Self::from_inner(s),
                Self::from_inner(v),
            )
        })
    }

    fn cholesky_decomposition(&self) -> Result<Self, CausalTensorError>
    where
        T: crate::backend::TensorData + RealField,
    {
        Tensor::cholesky_decomposition(&self.inner).map(Self::from_inner)
    }

    fn solve_least_squares_cholsky(a: &Self, b: &Self) -> Result<Self, CausalTensorError>
    where
        T: crate::backend::TensorData + RealField,
    {
        crate::InternalCpuTensor::solve_least_squares_cholsky(&a.inner, &b.inner)
            .map(Self::from_inner)
    }
}
