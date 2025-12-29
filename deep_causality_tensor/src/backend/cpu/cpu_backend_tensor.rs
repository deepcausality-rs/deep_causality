/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CpuBackend;
use crate::CausalTensorError;
use crate::backend::{Device, TensorBackend, TensorData};
use crate::traits::tensor::Tensor;
use crate::types::causal_tensor::{CpuTensor, EinSumAST};
use core::ops::Range;

impl TensorBackend for CpuBackend {
    type Tensor<T> = CpuTensor<T>;

    fn device() -> Device {
        Device::Cpu
    }

    // --- Creation ---

    fn create<T: Clone>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        CpuTensor::new(data.to_vec(), shape.to_vec()).expect("CpuBackend::create failed")
    }

    fn create_from_vec<T>(data: Vec<T>, shape: &[usize]) -> Self::Tensor<T> {
        CpuTensor::new(data, shape.to_vec()).expect("CpuBackend::create_from_vec failed")
    }

    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len = shape.iter().product();
        let data = vec![T::zero(); len];
        CpuTensor::new(data, shape.to_vec()).expect("CpuBackend::zeros failed")
    }

    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len = shape.iter().product();
        let data = vec![T::one(); len];
        CpuTensor::new(data, shape.to_vec()).expect("CpuBackend::ones failed")
    }

    fn from_shape_fn<T: Clone, F>(shape: &[usize], mut f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T,
    {
        // This is a naive implementation; optimize if needed
        // Iterate over all indices and apply f
        let len: usize = shape.iter().product();
        let mut data = Vec::with_capacity(len);

        // Cartesian product
        let mut index = vec![0; shape.len()];
        for _ in 0..len {
            data.push(f(&index));

            // Increment index
            for i in (0..shape.len()).rev() {
                index[i] += 1;
                if index[i] < shape[i] {
                    break;
                }
                index[i] = 0;
            }
        }

        CpuTensor::new(data, shape.to_vec()).expect("CpuBackend::from_shape_fn failed")
    }

    // --- Data Access ---

    fn to_vec<T: Clone>(tensor: &Self::Tensor<T>) -> Vec<T> {
        tensor.data.clone()
    }

    fn into_vec<T>(tensor: Self::Tensor<T>) -> Vec<T> {
        tensor.into_vec()
    }

    fn shape<T>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor.shape().to_vec()
    }

    fn get<T: Clone>(tensor: &Self::Tensor<T>, index: &[usize]) -> Option<T> {
        tensor.get(index).cloned()
    }

    // --- Shape Manipulation ---

    fn reshape<T: Clone>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T> {
        tensor.reshape(shape).expect("CpuBackend::reshape failed")
    }

    fn permute<T: Clone>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        tensor
            .permute_axes(axes)
            .expect("CpuBackend::permute failed")
    }

    fn slice<T: Clone>(_tensor: &Self::Tensor<T>, _ranges: &[Range<usize>]) -> Self::Tensor<T> {
        // CpuTensor slice takes (axis, index), not ranges.

        // Temporary: Panic "Not implemented for complex slicing"
        todo!("CpuBackend::slice for ranges not yet implemented")
    }

    // --- Element-wise Arithmetic ---

    fn add<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a + b
    }

    fn sub<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a - b
    }

    fn mul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a * b
    }

    fn div<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a / b
    }

    fn broadcast_op<T: Clone, F>(
        lhs: &Self::Tensor<T>,
        rhs: &Self::Tensor<T>,
        f: F,
    ) -> Result<Self::Tensor<T>, CausalTensorError>
    where
        F: Fn(T, T) -> Result<T, CausalTensorError>,
    {
        lhs.broadcast_op(rhs, f)
    }

    // --- Reduction ---

    fn sum<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        tensor.sum_axes(axes).expect("CpuBackend::sum failed")
    }

    fn max<T: TensorData>(_tensor: &Self::Tensor<T>, _axes: &[usize]) -> Self::Tensor<T> {
        // CpuTensor needs `max_axes`. Check if it exists.
        todo!("CpuBackend::max not implemented")
    }

    fn mean<T: TensorData + From<u32>>(
        tensor: &Self::Tensor<T>,
        axes: &[usize],
    ) -> Self::Tensor<T> {
        tensor.mean_axes(axes).expect("CpuBackend::mean failed")
    }

    // --- Advanced Shape ---

    fn ravel<T: Clone>(tensor: &Self::Tensor<T>) -> Self::Tensor<T> {
        tensor.clone().ravel()
    }

    fn arg_sort<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor.arg_sort().expect("CpuBackend::arg_sort failed")
    }

    fn shifted_view<T: Clone>(tensor: &Self::Tensor<T>, flat_index: usize) -> Self::Tensor<T> {
        tensor.shifted_view(flat_index)
    }

    // --- EinSum ---

    fn ein_sum<T: TensorData>(
        ast: &EinSumAST<Self::Tensor<T>>,
    ) -> Result<Self::Tensor<T>, CausalTensorError> {
        CpuTensor::execute_ein_sum(ast)
    }
}
