/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Implementation of Tensor trait for BackendTensor<T, MlxBackend>.
//!
//! This delegates operations to MlxBackend (implementing TensorBackend)
//! and wraps results in BackendTensor.

use crate::LinearAlgebraBackend;
use crate::MlxBackend;
use crate::types::backend_tensor::BackendTensor;
use crate::types::cpu_tensor::EinSumAST;
use crate::{CausalTensorError, Tensor, TensorBackend, TensorData};
use core::iter::Sum;
use core::ops::{Add, Div, Mul, Neg};
use deep_causality_num::{RealField, Zero};

impl<T: TensorData + std::iter::Sum> Tensor<T> for BackendTensor<T, MlxBackend> {
    fn ein_sum(ast: &EinSumAST<Self>) -> Result<Self, CausalTensorError>
    where
        T: TensorData,
    {
        // MlxBackend:ein_sum expects EinSumAST<MlxTensor<T>>
        // We need to convert EinSumAST<BackendTensor<T, MlxBackend>> to EinSumAST<MlxTensor<T>>
        fn convert_ast<T: TensorData>(
            ast: &EinSumAST<BackendTensor<T, MlxBackend>>,
        ) -> EinSumAST<crate::types::backend::mlx::MlxTensor<T>> {
            let op = ast
                .value()
                .clone()
                .map_tensor(|t: BackendTensor<T, MlxBackend>| t.into_inner());
            let children: Vec<_> = ast.children().iter().map(convert_ast).collect();
            if children.is_empty() {
                EinSumAST::new(op)
            } else {
                EinSumAST::with_children(op, children)
            }
        }

        let inner_ast = convert_ast(ast);
        MlxBackend::ein_sum(&inner_ast).map(Self::from_inner)
    }

    fn matmul(&self, rhs: &Self) -> Result<Self, CausalTensorError>
    where
        T: TensorData,
    {
        use crate::LinearAlgebraBackend;
        Ok(Self::from_inner(MlxBackend::matmul(
            &self.inner,
            &rhs.inner,
        )))
    }

    fn tensor_product(&self, _rhs: &Self) -> Result<Self, CausalTensorError>
    where
        T: Clone + Mul<Output = T>,
    {
        Err(CausalTensorError::NotImplemented(
            "MlxBackend::tensor_product".into(),
        ))
    }

    fn norm_l2(&self) -> T
    where
        T: RealField + Default + Zero + Sum + Copy,
    {
        let sq = MlxBackend::mul(&self.inner, &self.inner);
        let s = MlxBackend::sum(&sq, &[]);
        let vec = MlxBackend::to_vec(&s);
        if vec.is_empty() {
            T::zero()
        } else {
            vec[0].sqrt()
        }
    }

    fn norm_sq(&self) -> T
    where
        T: RealField + Default + Zero + Sum + Copy + Mul,
    {
        let sq = MlxBackend::mul(&self.inner, &self.inner);
        let s = MlxBackend::sum(&sq, &[]);
        let vec = MlxBackend::to_vec(&s);
        if vec.is_empty() { T::zero() } else { vec[0] }
    }

    fn sum_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<T, Output = T>,
    {
        Ok(Self::from_inner(MlxBackend::sum(&self.inner, axes)))
    }

    fn mean_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Div<T, Output = T> + From<u32> + Add<T, Output = T>,
    {
        Ok(Self::from_inner(MlxBackend::mean(&self.inner, axes)))
    }

    fn arg_sort(&self) -> Result<Vec<usize>, CausalTensorError>
    where
        T: Clone + Default + PartialOrd,
    {
        MlxBackend::arg_sort(&self.inner)
    }

    fn reshape(&self, new_shape: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        Ok(Self::from_inner(MlxBackend::reshape(
            &self.inner,
            new_shape,
        )))
    }

    fn ravel(self) -> Self
    where
        T: Clone,
    {
        Self::from_inner(MlxBackend::ravel(&self.inner))
    }

    fn slice(&self, axis: usize, index: usize) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        // TensorBackend::slice takes ranges.
        // Tensor::slice takes axis, index (returns slice at index along axis).
        // Convert axis/index to Range.
        // ranges[axis] = index..index+1, others full.
        // But we need shape to know full ranges?
        let shape = MlxBackend::shape(&self.inner);
        if axis >= shape.len() {
            return Err(CausalTensorError::AxisOutOfBounds);
        }
        if index >= shape[axis] {
            return Err(CausalTensorError::IndexOutOfBounds);
        }

        // This slice logic is usually reduction of dimension?
        // Using range_slice_impl logic typically keeps dimension 1?
        // TensorBackend::slice typically returns tensor of same rank?
        // Or depends on backend.
        // If Tensor::slice implies extracting sub-tensor, it usually returns same rank or 1 less?
        // InternalCpuTensor::slice: "Returns a view...". Rank preserved?
        // range_slice_impl takes ranges.
        // If TensorBackend::slice mimics numpy slice, it keeps rank if range is used.

        let mut ranges = Vec::new();
        for (i, &dim) in shape.iter().enumerate() {
            if i == axis {
                ranges.push(index..index + 1);
            } else {
                ranges.push(0..dim);
            }
        }

        // This keeps rank. If Tensor::slice expects rank reduction, we might need reshape/squeeze.
        // Let's assume keeping rank is fine for now, or match CpuBackend behavior in slice impl.
        Ok(Self::from_inner(MlxBackend::slice(&self.inner, &ranges)))
    }

    fn permute_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
    {
        Ok(Self::from_inner(MlxBackend::permute(&self.inner, axes)))
    }

    fn shifted_view(&self, flat_index: usize) -> Self
    where
        T: Clone,
    {
        Self::from_inner(MlxBackend::shifted_view(&self.inner, flat_index))
    }

    fn inverse(&self) -> Result<Self, CausalTensorError>
    where
        T: TensorData + RealField,
    {
        Ok(Self::from_inner(MlxBackend::inverse(&self.inner)))
    }

    fn qr(&self) -> Result<(Self, Self), CausalTensorError>
    where
        T: TensorData + Sum + RealField + Neg<Output = T>,
    {
        let (q, r) = MlxBackend::qr(&self.inner);
        Ok((Self::from_inner(q), Self::from_inner(r)))
    }

    fn svd(&self) -> Result<(Self, Self, Self), CausalTensorError>
    where
        T: TensorData + Sum + RealField,
    {
        let (u, s, v) = MlxBackend::svd(&self.inner);
        Ok((
            Self::from_inner(u),
            Self::from_inner(s),
            Self::from_inner(v),
        ))
    }

    fn cholesky_decomposition(&self) -> Result<Self, CausalTensorError>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        Ok(Self::from_inner(MlxBackend::cholesky_decomposition(
            &self.inner,
        )))
    }

    fn solve_least_squares_cholsky(a: &Self, b: &Self) -> Result<Self, CausalTensorError>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        Ok(Self::from_inner(MlxBackend::solve_least_squares_cholsky(
            &a.inner, &b.inner,
        )))
    }

    fn stack(tensors: &[Self], axis: usize) -> Result<Self, CausalTensorError>
    where
        T: TensorData,
        Self: Sized,
    {
        let inner_tensors: Vec<_> = tensors.iter().map(|t| t.inner.clone()).collect();
        MlxBackend::stack(&inner_tensors, axis).map(Self::from_inner)
    }
}
