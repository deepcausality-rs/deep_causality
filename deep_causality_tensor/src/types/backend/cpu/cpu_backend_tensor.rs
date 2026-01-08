/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CpuBackend;
use crate::types::cpu_tensor::{EinSumAST, InternalCpuTensor};
use crate::{CausalTensorError, Device, Tensor, TensorBackend, TensorData};
use core::ops::Range;

impl TensorBackend for CpuBackend {
    type Tensor<T> = InternalCpuTensor<T>;

    fn device() -> Device {
        Device::Cpu
    }

    // --- Creation ---

    fn create<T: Clone>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        InternalCpuTensor::new(data.to_vec(), shape.to_vec()).expect("CpuBackend::create failed")
    }

    fn create_from_vec<T>(data: Vec<T>, shape: &[usize]) -> Self::Tensor<T> {
        InternalCpuTensor::new(data, shape.to_vec()).expect("CpuBackend::create_from_vec failed")
    }

    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len = shape.iter().product();
        let data = vec![T::zero(); len];
        InternalCpuTensor::new(data, shape.to_vec()).expect("CpuBackend::zeros failed")
    }

    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len = shape.iter().product();
        let data = vec![T::one(); len];
        InternalCpuTensor::new(data, shape.to_vec()).expect("CpuBackend::ones failed")
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

        InternalCpuTensor::new(data, shape.to_vec()).expect("CpuBackend::from_shape_fn failed")
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

    fn strides<T>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor.strides().to_vec()
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

    fn slice<T: Clone>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T> {
        tensor
            .range_slice_impl(ranges)
            .expect("CpuBackend::slice failed")
    }

    fn stack<T: TensorData>(
        tensors: &[Self::Tensor<T>],
        axis: usize,
    ) -> Result<Self::Tensor<T>, CausalTensorError> {
        InternalCpuTensor::stack_impl(tensors, axis)
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

    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        tensor.max_axes_impl(axes).expect("CpuBackend::max failed")
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

    fn arg_sort<T: TensorData>(tensor: &Self::Tensor<T>) -> Result<Vec<usize>, CausalTensorError> {
        tensor.arg_sort()
    }

    fn shifted_view<T: Clone>(tensor: &Self::Tensor<T>, flat_index: usize) -> Self::Tensor<T> {
        tensor.shifted_view(flat_index)
    }

    // --- EinSum ---

    fn ein_sum<T: TensorData>(
        ast: &EinSumAST<Self::Tensor<T>>,
    ) -> Result<Self::Tensor<T>, CausalTensorError> {
        InternalCpuTensor::execute_ein_sum(ast)
    }
}
