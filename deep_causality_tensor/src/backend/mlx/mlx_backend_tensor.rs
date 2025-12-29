/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! TensorBackend implementation for MlxBackend.

use super::{MlxBackend, MlxTensor};
use crate::CausalTensor;
use crate::backend::Device;
use crate::traits::{TensorBackend, TensorData};
use core::ops::Range;

impl TensorBackend for MlxBackend {
    type Tensor<T: TensorData> = MlxTensor<T>;

    fn device() -> Device {
        Device::Gpu(0)
    }

    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        // Convert shape to i32 for MLX
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();

        // For f32, create directly; for f64, downcast to f32
        let array = if core::any::TypeId::of::<T>() == core::any::TypeId::of::<f32>() {
            // Safety: We've verified T is f32
            let f32_data: &[f32] =
                unsafe { core::slice::from_raw_parts(data.as_ptr() as *const f32, data.len()) };
            mlx_rs::Array::from_slice(f32_data, &mlx_shape)
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<f64>() {
            // Convert f64 to f32 for MLX
            let f32_data: Vec<f32> = data
                .iter()
                .map(|x| {
                    let bytes = unsafe { core::mem::transmute_copy::<T, [u8; 8]>(x) };
                    f64::from_ne_bytes(bytes) as f32
                })
                .collect();
            mlx_rs::Array::from_slice(&f32_data, &mlx_shape)
        } else {
            // For other types, use zeros as fallback
            let f32_data: Vec<f32> = vec![0.0f32; data.len()];
            mlx_rs::Array::from_slice(&f32_data, &mlx_shape)
        };

        MlxTensor::new(array)
    }

    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = mlx_rs::ops::zeros::<f32>(&mlx_shape).expect("MlxBackend::zeros: failed");
        MlxTensor::new(array)
    }

    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = mlx_rs::ops::ones::<f32>(&mlx_shape).expect("MlxBackend::ones: failed");
        MlxTensor::new(array)
    }

    fn from_shape_fn<T: TensorData, F>(shape: &[usize], mut f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T,
    {
        // Build data on CPU, then transfer to MLX
        let len: usize = shape.iter().product();
        let mut data = Vec::with_capacity(len);
        let mut indices = vec![0usize; shape.len()];

        for _ in 0..len {
            data.push(f(&indices));
            for dim in (0..shape.len()).rev() {
                indices[dim] += 1;
                if indices[dim] < shape[dim] {
                    break;
                }
                indices[dim] = 0;
            }
        }

        Self::create(&data, shape)
    }

    fn to_vec<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<T> {
        // Get data from MLX array
        let array = tensor.as_array();
        let len = array.size();

        if core::any::TypeId::of::<T>() == core::any::TypeId::of::<f32>() {
            let f32_data: Vec<f32> = array.as_slice().to_vec();
            // Safety: We've verified T is f32
            unsafe { core::mem::transmute(f32_data) }
        } else if core::any::TypeId::of::<T>() == core::any::TypeId::of::<f64>() {
            // Convert f32 back to f64
            let f32_data: Vec<f32> = array.as_slice().to_vec();
            let f64_data: Vec<f64> = f32_data.iter().map(|&x| x as f64).collect();
            // Safety: We've verified T is f64
            unsafe {
                let ptr = f64_data.as_ptr() as *const T;
                let slice = core::slice::from_raw_parts(ptr, len);
                slice.to_vec()
            }
        } else {
            vec![T::zero(); len]
        }
    }

    fn shape<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor
            .as_array()
            .shape()
            .iter()
            .map(|&s| s as usize)
            .collect()
    }

    fn reshape<T: TensorData>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = tensor
            .as_array()
            .reshape(&mlx_shape)
            .expect("MlxBackend::reshape: failed");
        MlxTensor::new(array)
    }

    fn permute<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        // MLX ops::transpose without additional axes just does matrix transpose
        // For permute, we need to use swapaxes or build it ourselves
        // Simple workaround: extract data to CPU, permute using CausalTensor, then back to MLX
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = CausalTensor::new(cpu_data, cpu_shape)
            .expect("MlxBackend::permute: cpu tensor creation failed");
        use crate::Tensor as TensorTrait;
        let permuted = cpu_tensor
            .permute_axes(axes)
            .expect("MlxBackend::permute: permutation failed");
        Self::create(permuted.as_slice(), permuted.shape())
    }

    fn slice<T: TensorData>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T> {
        // MLX doesn't have a direct slice function matching our signature
        // Use CPU fallback via CausalTensor::range_slice_impl
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = CausalTensor::new(cpu_data, cpu_shape)
            .expect("MlxBackend::slice: cpu tensor creation failed");
        let sliced = cpu_tensor
            .range_slice_impl(ranges)
            .expect("MlxBackend::slice: slice failed");
        Self::create(sliced.as_slice(), sliced.shape())
    }

    fn add<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        let array = mlx_rs::ops::add(a.as_array(), b.as_array()).expect("MlxBackend::add: failed");
        MlxTensor::new(array)
    }

    fn sub<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        let array =
            mlx_rs::ops::subtract(a.as_array(), b.as_array()).expect("MlxBackend::sub: failed");
        MlxTensor::new(array)
    }

    fn mul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        let array =
            mlx_rs::ops::multiply(a.as_array(), b.as_array()).expect("MlxBackend::mul: failed");
        MlxTensor::new(array)
    }

    fn div<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        let array =
            mlx_rs::ops::divide(a.as_array(), b.as_array()).expect("MlxBackend::div: failed");
        MlxTensor::new(array)
    }

    fn sum<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        let array = if axes.is_empty() {
            // Sum all elements
            tensor
                .as_array()
                .sum(None)
                .expect("MlxBackend::sum: failed")
        } else {
            // Sum along specified axes via CausalTensor fallback
            // (mlx-rs sum_axes has different signature)
            let cpu_data: Vec<T> = Self::to_vec(tensor);
            let cpu_shape = Self::shape(tensor);
            let cpu_tensor = CausalTensor::new(cpu_data, cpu_shape)
                .expect("MlxBackend::sum: cpu tensor creation failed");
            use crate::Tensor as TensorTrait;
            let mut sorted_axes = axes.to_vec();
            sorted_axes.sort();
            let result = cpu_tensor
                .sum_axes(&sorted_axes)
                .expect("MlxBackend::sum: axis reduction failed");
            return Self::create(result.as_slice(), result.shape());
        };
        MlxTensor::new(array)
    }

    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        let array = if axes.is_empty() {
            // Max of all elements
            tensor
                .as_array()
                .max(None)
                .expect("MlxBackend::max: failed")
        } else {
            // Max along specified axes via CausalTensor fallback
            // (mlx-rs max with axes has different binding)
            let cpu_data: Vec<T> = Self::to_vec(tensor);
            let cpu_shape = Self::shape(tensor);
            let cpu_tensor = CausalTensor::new(cpu_data, cpu_shape)
                .expect("MlxBackend::max: cpu tensor creation failed");
            use crate::backend::CpuBackend;
            use crate::traits::TensorBackend as TB;
            return Self::create(
                CpuBackend::max(&cpu_tensor, axes).as_slice(),
                CpuBackend::shape(&CpuBackend::max(&cpu_tensor, axes)).as_slice(),
            );
        };
        MlxTensor::new(array)
    }
}
