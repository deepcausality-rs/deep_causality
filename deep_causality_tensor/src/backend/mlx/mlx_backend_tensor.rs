/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! TensorBackend implementation for MlxBackend.

use super::{MlxBackend, MlxTensor};
use crate::CausalTensor; // Use alias to avoid confusion if needed, though implicit import works
use crate::backend::Device;
use crate::traits::{TensorBackend, TensorData};
use core::ops::Range;

// Implement internal helpers for MlxBackend
impl MlxBackend {
    fn create_impl<T>(data: &[T], shape: &[usize]) -> MlxTensor<T> {
        let shape_i32: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let name = std::any::type_name::<T>();

        if name == "f32" {
            let slice = unsafe { std::mem::transmute::<&[T], &[f32]>(data) };
            let array = mlx_rs::Array::from_slice(slice, &shape_i32);
            return MlxTensor::new(array);
        }
        if name == "i32" {
            let slice = unsafe { std::mem::transmute::<&[T], &[i32]>(data) };
            let array = mlx_rs::Array::from_slice(slice, &shape_i32);
            return MlxTensor::new(array);
        }
        if name == "f64" {
            let slice = unsafe { std::mem::transmute::<&[T], &[f64]>(data) };
            // Convert to f32
            let data_f32: Vec<f32> = slice.iter().map(|&x| x as f32).collect();
            let array = mlx_rs::Array::from_slice(&data_f32, &shape_i32);
            return MlxTensor::new(array);
        }

        panic!("MlxBackend::create: Unsupported type {}", name);
    }

    fn to_vec_impl<T: Clone>(tensor: &MlxTensor<T>) -> Vec<T> {
        let array = tensor.as_array();
        let name = std::any::type_name::<T>();

        if name == "f32" {
            let vals = array.as_slice::<f32>();
            let vals_t = unsafe { std::mem::transmute::<&[f32], &[T]>(vals) };
            return vals_t.to_vec();
        }

        if name == "f64" {
            let vals = array.as_slice::<f32>();
            let vals_f64: Vec<f64> = vals.iter().map(|&x| x as f64).collect();
            return unsafe { std::mem::transmute::<Vec<f64>, Vec<T>>(vals_f64) };
        }

        if name == "i32" {
            let vals = array.as_slice::<i32>();
            let vals_t = unsafe { std::mem::transmute::<&[i32], &[T]>(vals) };
            return vals_t.to_vec();
        }

        panic!("MlxBackend::to_vec: Unsupported type {}", name);
    }

    fn to_vec_impl_unrestricted<T>(tensor: &MlxTensor<T>) -> Vec<T> {
        let array = tensor.as_array();
        let name = std::any::type_name::<T>();

        if name == "f32" {
            let vals = array.as_slice::<f32>();
            let vec = vals.to_vec();
            let vec_t = unsafe { std::mem::transmute::<Vec<f32>, Vec<T>>(vec) };
            return vec_t;
        }

        if name == "f64" {
            let vals = array.as_slice::<f32>();
            let vec_f64: Vec<f64> = vals.iter().map(|&x| x as f64).collect();
            let vec_t = unsafe { std::mem::transmute::<Vec<f64>, Vec<T>>(vec_f64) };
            return vec_t;
        }

        if name == "i32" {
            let vals = array.as_slice::<i32>();
            let vec = vals.to_vec();
            let vec_t = unsafe { std::mem::transmute::<Vec<i32>, Vec<T>>(vec) };
            return vec_t;
        }

        panic!("MlxBackend::into_vec: Unsupported type {}", name);
    }
}

impl TensorBackend for MlxBackend {
    type Tensor<T> = MlxTensor<T>;

    fn device() -> Device {
        Device::Gpu(0)
    }

    fn create_from_vec<T>(data: Vec<T>, shape: &[usize]) -> Self::Tensor<T> {
        Self::create_impl(&data, shape)
    }

    fn create<T: Clone>(data: &[T], shape: &[usize]) -> Self::Tensor<T> {
        Self::create_impl(data, shape)
    }

    fn into_vec<T>(tensor: Self::Tensor<T>) -> Vec<T> {
        Self::to_vec_impl_unrestricted(&tensor)
    }

    fn to_vec<T: Clone>(tensor: &Self::Tensor<T>) -> Vec<T> {
        Self::to_vec_impl(tensor)
    }

    fn shape<T>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor
            .as_array()
            .shape()
            .iter()
            .map(|&s| s as usize)
            .collect()
    }

    fn strides<T>(tensor: &Self::Tensor<T>) -> Vec<usize> {
        tensor
            .as_array()
            .strides()
            .iter()
            .map(|&s| s as usize)
            .collect()
    }

    fn get<T: Clone>(tensor: &Self::Tensor<T>, index: &[usize]) -> Option<T> {
        // Fallback to CPU for single element access
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let shape = Self::shape(tensor);
        let cpu_tensor = crate::InternalCpuTensor::new(cpu_data, shape).ok()?;
        cpu_tensor.get(index).cloned()
    }

    fn broadcast_op<T: Clone, F>(
        lhs: &Self::Tensor<T>,
        rhs: &Self::Tensor<T>,
        f: F,
    ) -> Result<Self::Tensor<T>, crate::CausalTensorError>
    where
        F: Fn(T, T) -> Result<T, crate::CausalTensorError>,
    {
        // Fallback to CPU
        // Note: usage of to_vec requires T: Clone for safety or valid type support
        let cpu_lhs = crate::InternalCpuTensor::new(Self::to_vec(lhs), Self::shape(lhs)).unwrap();
        let cpu_rhs = crate::InternalCpuTensor::new(Self::to_vec(rhs), Self::shape(rhs)).unwrap();

        use crate::backend::CpuBackend;
        let result = CpuBackend::broadcast_op(&cpu_lhs, &cpu_rhs, f)?;
        Ok(Self::create(result.as_slice(), result.shape()))
    }

    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        // Ignoring dtype for now to fix compat
        let array = mlx_rs::ops::zeros::<f32>(&mlx_shape).expect("MlxBackend::zeros: failed");
        MlxTensor::new(array)
    }

    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        // Ignoring dtype
        let array = mlx_rs::ops::ones::<f32>(&mlx_shape).expect("MlxBackend::ones: failed");
        MlxTensor::new(array)
    }

    fn from_shape_fn<T: Clone, F>(shape: &[usize], mut f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T,
    {
        // Generate on CPU
        // Simple recursive fill
        fn fill_recursive<T, F>(
            current_dim: usize,
            indices: &mut Vec<usize>,
            shape: &[usize],
            f: &mut F,
            data: &mut Vec<T>,
        ) where
            T: Clone,
            F: FnMut(&[usize]) -> T,
        {
            if current_dim == shape.len() {
                data.push(f(indices));
                return;
            }

            for i in 0..shape[current_dim] {
                indices.push(i);
                fill_recursive(current_dim + 1, indices, shape, f, data);
                indices.pop();
            }
        }

        let size: usize = shape.iter().product();
        let mut data = Vec::with_capacity(size);
        let mut indices = Vec::with_capacity(shape.len());

        fill_recursive(0, &mut indices, shape, &mut f, &mut data);

        Self::create(&data, shape)
    }

    fn reshape<T: Clone>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = tensor
            .as_array()
            .reshape(&mlx_shape)
            .expect("MlxBackend::reshape: failed");
        MlxTensor::new(array)
    }

    fn permute<T: Clone>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        // Fallback to CPU due to binding mismatch
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor =
            CausalTensor::new(cpu_data, cpu_shape).expect("permute: cpu creation failed");
        use crate::backend::CpuBackend;
        let result = CpuBackend::permute(&cpu_tensor, axes);
        Self::create(result.as_slice(), result.shape())
    }

    fn slice<T: Clone>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T> {
        // Fallback to CPU
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = match CausalTensor::new(cpu_data, cpu_shape) {
            Ok(t) => t,
            Err(_) => panic!("MlxBackend::slice: cpu tensor creation failed"),
        };

        use crate::backend::CpuBackend;
        let result = CpuBackend::slice(&cpu_tensor, ranges);
        Self::create(result.as_slice(), result.shape())
    }

    fn stack<T: TensorData>(
        tensors: &[Self::Tensor<T>],
        axis: usize,
    ) -> Result<Self::Tensor<T>, crate::CausalTensorError> {
        // Fallback to CPU
        let mut cpu_tensors = Vec::with_capacity(tensors.len());
        for t in tensors {
            let data = Self::to_vec(t);
            let shape = Self::shape(t);
            cpu_tensors.push(crate::InternalCpuTensor::new(data, shape)?);
        }

        use crate::backend::CpuBackend;
        let result = CpuBackend::stack(&cpu_tensors, axis)?;
        Ok(Self::create(result.as_slice(), result.shape()))
    }

    fn ravel<T: Clone>(tensor: &Self::Tensor<T>) -> Self::Tensor<T> {
        let array = tensor
            .as_array()
            .reshape(&[-1])
            .expect("MlxBackend::ravel: reshape failed");
        MlxTensor::new(array)
    }

    fn arg_sort<T: TensorData>(
        tensor: &Self::Tensor<T>,
    ) -> Result<Vec<usize>, crate::CausalTensorError> {
        // Fallback to CPU for safety regarding indices return type
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = crate::CausalTensor::new(cpu_data, cpu_shape)?;
        cpu_tensor.arg_sort()
    }

    fn shifted_view<T: Clone>(tensor: &Self::Tensor<T>, flat_index: usize) -> Self::Tensor<T> {
        // Fallback to CPU
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = match CausalTensor::new(cpu_data, cpu_shape) {
            Ok(t) => t,
            Err(_) => panic!("MlxBackend::shifted_view: cpu tensor creation failed"),
        };

        use crate::backend::CpuBackend;
        let result = CpuBackend::shifted_view(&cpu_tensor, flat_index);
        Self::create(result.as_slice(), result.shape())
    }

    fn mean<T: TensorData + From<u32>>(
        tensor: &Self::Tensor<T>,
        axes: &[usize],
    ) -> Self::Tensor<T> {
        if axes.is_empty() {
            let array =
                mlx_rs::ops::mean(tensor.as_array(), false).expect("MlxBackend::mean: failed");
            return MlxTensor::new(array);
        }

        // Fallback to CPU
        let cpu_data: Vec<T> = Self::to_vec(tensor);
        let cpu_shape = Self::shape(tensor);
        let cpu_tensor = match CausalTensor::new(cpu_data, cpu_shape) {
            Ok(t) => t,
            Err(_) => panic!("MlxBackend::mean: cpu tensor creation failed"),
        };

        // CausalTensor mean uses different trait or inherent impl?
        // TensorBackend has mean.
        // CpuBackend::mean
        use crate::backend::CpuBackend;
        let result = CpuBackend::mean(&cpu_tensor, axes);
        Self::create(result.as_slice(), result.shape())
    }

    fn ein_sum<T: TensorData>(
        ast: &crate::types::cpu_tensor::EinSumAST<Self::Tensor<T>>,
    ) -> Result<Self::Tensor<T>, crate::CausalTensorError> {
        use crate::traits::LinearAlgebraBackend;
        use crate::types::cpu_tensor::{EinSumAST, EinSumOp}; // Required for MlxBackend::tensor_product

        // Helper to generate index characters for einsum
        fn get_char(i: usize) -> char {
            if i < 26 {
                (b'a' + i as u8) as char
            } else if i < 52 {
                (b'A' + (i - 26) as u8) as char
            } else {
                panic!("Rank too high for einsum generation");
            }
        }

        fn eval<T: TensorData>(
            ast: &EinSumAST<MlxTensor<T>>,
        ) -> Result<MlxTensor<T>, crate::CausalTensorError> {
            let op = ast.value();
            let children = ast.children();

            match op {
                EinSumOp::TensorSource { tensor } => Ok(tensor.clone()),

                EinSumOp::Contraction { lhs_axes, rhs_axes } => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;

                    let shape_l: Vec<usize> =
                        lhs.as_array().shape().iter().map(|&x| x as usize).collect();
                    let shape_r: Vec<usize> =
                        rhs.as_array().shape().iter().map(|&x| x as usize).collect();

                    // Assign chars
                    let chars_l: Vec<char> = (0..shape_l.len()).map(get_char).collect();
                    // Rhs starts after lhs to avoid collision initially
                    let offset = shape_l.len();
                    let mut chars_r: Vec<char> =
                        (0..shape_r.len()).map(|i| get_char(offset + i)).collect();

                    // Match up contracted axes
                    for (&l_idx, &r_idx) in lhs_axes.iter().zip(rhs_axes.iter()) {
                        chars_r[r_idx] = chars_l[l_idx];
                    }

                    // Build output chars
                    let mut chars_out = Vec::new();
                    for (i, &c) in chars_l.iter().enumerate() {
                        if !lhs_axes.contains(&i) {
                            chars_out.push(c);
                        }
                    }
                    for (i, &c) in chars_r.iter().enumerate() {
                        if !rhs_axes.contains(&i) {
                            chars_out.push(c);
                        }
                    }

                    let eq = format!(
                        "{},{}->{}",
                        chars_l.iter().collect::<String>(),
                        chars_r.iter().collect::<String>(),
                        chars_out.iter().collect::<String>()
                    );

                    let res = mlx_rs::ops::einsum(&eq, vec![lhs.as_array(), rhs.as_array()])
                        .expect("einsum contraction failed");
                    Ok(MlxTensor::new(res))
                }

                EinSumOp::Reduction { axes } => {
                    let operand = eval(&children[0])?;
                    let rank = operand.as_array().ndim();
                    let chars_in: Vec<char> = (0..rank).map(get_char).collect();
                    let mut chars_out = Vec::new();
                    for (i, &c) in chars_in.iter().enumerate() {
                        if !axes.contains(&i) {
                            chars_out.push(c);
                        }
                    }
                    let eq = format!(
                        "{}->{}",
                        chars_in.iter().collect::<String>(),
                        chars_out.iter().collect::<String>()
                    );
                    let res = mlx_rs::ops::einsum(&eq, vec![operand.as_array()])
                        .expect("einsum reduction failed");
                    Ok(MlxTensor::new(res))
                }

                EinSumOp::MatMul => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;
                    Ok(MlxTensor::new(
                        mlx_rs::ops::matmul(lhs.as_array(), rhs.as_array()).expect("matmul failed"),
                    ))
                }

                EinSumOp::DotProd => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;
                    Ok(MlxTensor::new(
                        mlx_rs::ops::matmul(lhs.as_array(), rhs.as_array())
                            .expect("dotprod failed"),
                    ))
                }

                EinSumOp::Trace { axes1, axes2 } => {
                    let operand = eval(&children[0])?;
                    let rank = operand.as_array().ndim();
                    let mut chars_in: Vec<char> = (0..rank).map(get_char).collect();

                    chars_in[*axes2] = chars_in[*axes1];

                    let mut chars_out = Vec::new();
                    for (i, &c) in chars_in.iter().enumerate() {
                        if i != *axes1 && i != *axes2 {
                            chars_out.push(c);
                        }
                    }

                    let eq = format!(
                        "{}->{}",
                        chars_in.iter().collect::<String>(),
                        chars_out.iter().collect::<String>()
                    );
                    let res = mlx_rs::ops::einsum(&eq, vec![operand.as_array()])
                        .expect("einsum trace failed");
                    Ok(MlxTensor::new(res))
                }

                EinSumOp::TensorProduct => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;
                    Ok(MlxBackend::tensor_product(&lhs, &rhs))
                }

                EinSumOp::ElementWiseProduct => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;
                    Ok(MlxBackend::mul(&lhs, &rhs))
                }

                EinSumOp::Transpose { new_order } => {
                    let operand = eval(&children[0])?;
                    Ok(MlxBackend::permute(&operand, new_order))
                }

                EinSumOp::DiagonalExtraction { axes1, axes2 } => {
                    let operand = eval(&children[0])?;
                    let rank = operand.as_array().ndim();
                    let mut chars_in: Vec<char> = (0..rank).map(get_char).collect();

                    chars_in[*axes2] = chars_in[*axes1];

                    let mut chars_out = Vec::new();
                    for (i, &c) in chars_in.iter().enumerate() {
                        if i != *axes2 {
                            chars_out.push(c);
                        }
                    }

                    let eq = format!(
                        "{}->{}",
                        chars_in.iter().collect::<String>(),
                        chars_out.iter().collect::<String>()
                    );
                    let res = mlx_rs::ops::einsum(&eq, vec![operand.as_array()])
                        .expect("einsum diagonal failed");
                    Ok(MlxTensor::new(res))
                }

                EinSumOp::BatchMatMul => {
                    let lhs = eval(&children[0])?;
                    let rhs = eval(&children[1])?;
                    Ok(MlxTensor::new(
                        mlx_rs::ops::matmul(lhs.as_array(), rhs.as_array())
                            .expect("batch matmul failed"),
                    ))
                }
            }
        }

        eval(ast)
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
            tensor
                .as_array()
                .sum(None)
                .expect("MlxBackend::sum: failed")
        } else {
            // Use fallback
            let cpu_data: Vec<T> = Self::to_vec(tensor);
            let cpu_shape = Self::shape(tensor);
            let cpu_tensor = match CausalTensor::new(cpu_data, cpu_shape) {
                Ok(t) => t,
                Err(_) => panic!("MlxBackend::sum: cpu tensor creation failed"),
            };

            let mut sorted_axes = axes.to_vec();
            sorted_axes.sort();

            use crate::Tensor as TensorTrait;
            let result = cpu_tensor
                .sum_axes(&sorted_axes)
                .expect("MlxBackend::sum: axis reduction failed");
            return Self::create(result.as_slice(), result.shape());
        };
        MlxTensor::new(array)
    }

    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T> {
        let array = if axes.is_empty() {
            tensor
                .as_array()
                .max(None)
                .expect("MlxBackend::max: failed")
        } else {
            let cpu_data: Vec<T> = Self::to_vec(tensor);
            let cpu_shape = Self::shape(tensor);
            let cpu_tensor = crate::InternalCpuTensor::new(cpu_data, cpu_shape)
                .expect("MlxBackend::max: cpu tensor creation failed");
            use crate::backend::CpuBackend;
            // use crate::traits::TensorBackend;
            let result_tensor = CpuBackend::max(&cpu_tensor, axes);
            return Self::create(result_tensor.as_slice(), result_tensor.shape());
        };
        MlxTensor::new(array)
    }
}
