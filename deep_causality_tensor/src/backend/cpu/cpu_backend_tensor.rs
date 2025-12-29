/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! TensorBackend implementation for CpuBackend.
//!
//! When the `parallel` feature is enabled, operations like `from_shape_fn`,
//! `sum`, and `max` use rayon for multi-threaded execution.
use super::CpuBackend;
use crate::CausalTensor;
use crate::backend::Device;
use crate::traits::{TensorBackend, TensorData};
use core::ops::Range;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

impl TensorBackend for CpuBackend {
    type Tensor<T: TensorData> = CausalTensor<T>;

    fn device() -> Device {
        Device::Cpu
    }

    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        CausalTensor::new(data.to_vec(), shape.to_vec())
            .expect("CpuBackend::create: shape mismatch with data length")
    }

    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len: usize = shape.iter().product();
        let data = vec![T::zero(); len];
        CausalTensor::new(data, shape.to_vec()).expect("CpuBackend::zeros: invalid shape")
    }

    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let len: usize = shape.iter().product();
        let data = vec![T::one(); len];
        CausalTensor::new(data, shape.to_vec()).expect("CpuBackend::ones: invalid shape")
    }

    fn from_shape_fn<T: TensorData, F>(shape: &[usize], mut f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T,
    {
        let len: usize = shape.iter().product();
        let mut data = Vec::with_capacity(len);
        let mut indices = vec![0usize; shape.len()];

        for _ in 0..len {
            data.push(f(&indices));
            // Increment indices in row-major order
            for dim in (0..shape.len()).rev() {
                indices[dim] += 1;
                if indices[dim] < shape[dim] {
                    break;
                }
                indices[dim] = 0;
            }
        }

        CausalTensor::new(data, shape.to_vec()).expect("CpuBackend::from_shape_fn: invalid shape")
    }

    fn to_vec<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<T> {
        tensor.as_slice().to_vec()
    }

    fn shape<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor.shape().to_vec()
    }

    fn reshape<T: TensorData>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T> {
        use crate::Tensor as TensorTrait;
        tensor
            .reshape(shape)
            .expect("CpuBackend::reshape: incompatible shape")
    }

    fn permute<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        use crate::Tensor as TensorTrait;
        tensor
            .permute_axes(axes)
            .expect("CpuBackend::permute: invalid axes")
    }

    fn slice<T: TensorData>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T> {
        tensor
            .range_slice_impl(ranges)
            .expect("CpuBackend::slice: range slice failed")
    }

    fn add<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        // CausalTensor's Add already handles element-wise operations efficiently
        a.clone() + b.clone()
    }

    fn sub<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a.clone() - b.clone()
    }

    fn mul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a.clone() * b.clone()
    }

    fn div<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        a.clone() / b.clone()
    }

    fn sum<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        use crate::Tensor as TensorTrait;
        if axes.is_empty() {
            // Sum all elements - use parallel reduction when available
            #[cfg(feature = "parallel")]
            {
                let total: T = tensor
                    .as_slice()
                    .par_iter()
                    .copied()
                    .reduce(|| T::zero(), |a, b| a + b);
                CausalTensor::new(vec![total], vec![])
                    .expect("CpuBackend::sum: failed to create scalar")
            }
            #[cfg(not(feature = "parallel"))]
            {
                tensor.sum_axes(&[]).expect("CpuBackend::sum: failed")
            }
        } else {
            // Sum along specified axes - delegate to CausalTensor
            let mut sorted_axes = axes.to_vec();
            sorted_axes.sort();
            tensor
                .sum_axes(&sorted_axes)
                .expect("CpuBackend::sum: invalid axes")
        }
    }

    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        let data = tensor.as_slice();
        let shape = tensor.shape();

        if axes.is_empty() {
            // Max of all elements - use parallel reduction when available
            #[cfg(feature = "parallel")]
            let max_val = {
                data.par_iter()
                    .copied()
                    .reduce(|| T::zero(), |a, b| if a > b { a } else { b })
            };
            #[cfg(not(feature = "parallel"))]
            let max_val = {
                data.iter()
                    .copied()
                    .fold(T::zero(), |acc, x| if x > acc { x } else { acc })
            };

            CausalTensor::new(vec![max_val], vec![])
                .expect("CpuBackend::max: failed to create scalar")
        } else {
            // For single axis reduction
            assert_eq!(
                axes.len(),
                1,
                "CpuBackend::max: multi-axis not yet implemented"
            );
            let axis = axes[0];
            assert!(axis < shape.len(), "CpuBackend::max: axis out of bounds");

            // Calculate new shape (remove the axis)
            let mut new_shape: Vec<usize> = shape.to_vec();
            new_shape.remove(axis);
            if new_shape.is_empty() {
                new_shape.push(1);
            }

            let new_len: usize = new_shape.iter().product();

            // Parallel axis reduction
            #[cfg(feature = "parallel")]
            let result_data: Vec<T> = {
                use std::sync::Mutex;
                let results: Vec<Mutex<Option<T>>> =
                    (0..new_len).map(|_| Mutex::new(None)).collect();

                data.par_iter().enumerate().for_each(|(flat_idx, val)| {
                    // Convert flat index to multi-dimensional indices
                    let mut remaining = flat_idx;
                    let mut indices = vec![0usize; shape.len()];
                    for dim in (0..shape.len()).rev() {
                        indices[dim] = remaining % shape[dim];
                        remaining /= shape[dim];
                    }

                    // Remove the axis dimension to get result index
                    let mut result_indices = indices.clone();
                    result_indices.remove(axis);

                    // Convert result indices to flat index
                    let mut result_flat = 0;
                    let mut stride = 1;
                    for dim in (0..result_indices.len()).rev() {
                        result_flat += result_indices[dim] * stride;
                        stride *= new_shape[dim];
                    }

                    // Update max atomically
                    let mut guard = results[result_flat].lock().unwrap();
                    *guard = match *guard {
                        None => Some(*val),
                        Some(m) => Some(if *val > m { *val } else { m }),
                    };
                });

                results
                    .into_iter()
                    .map(|m| m.into_inner().unwrap().unwrap_or_else(T::zero))
                    .collect()
            };

            #[cfg(not(feature = "parallel"))]
            let result_data: Vec<T> = {
                let mut result = vec![None; new_len];

                for (flat_idx, val) in data.iter().enumerate() {
                    let mut remaining = flat_idx;
                    let mut indices = vec![0usize; shape.len()];
                    for dim in (0..shape.len()).rev() {
                        indices[dim] = remaining % shape[dim];
                        remaining /= shape[dim];
                    }

                    let mut result_indices = indices.clone();
                    result_indices.remove(axis);

                    let mut result_flat = 0;
                    let mut stride = 1;
                    for dim in (0..result_indices.len()).rev() {
                        result_flat += result_indices[dim] * stride;
                        stride *= new_shape[dim];
                    }

                    result[result_flat] = match result[result_flat] {
                        None => Some(*val),
                        Some(m) => Some(if *val > m { *val } else { m }),
                    };
                }

                result
                    .into_iter()
                    .map(|opt| opt.unwrap_or_else(T::zero))
                    .collect()
            };

            CausalTensor::new(result_data, new_shape)
                .expect("CpuBackend::max: failed to create result")
        }
    }
}
