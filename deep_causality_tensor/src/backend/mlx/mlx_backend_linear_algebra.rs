/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! LinearAlgebraBackend implementation for MlxBackend.

use super::{MlxBackend, MlxTensor};
use crate::traits::{LinearAlgebraBackend, TensorData};
use core::iter::Sum;
use deep_causality_num::{RealField, Ring};

impl LinearAlgebraBackend for MlxBackend {
    fn matmul<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        let array =
            mlx_rs::ops::matmul(a.as_array(), b.as_array()).expect("MlxBackend::matmul: failed");
        MlxTensor::new(array)
    }

    fn qr<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::qr returns (Q, R)
        let (q, r) =
            mlx_rs::linalg::qr(input.as_array()).expect("MlxBackend::qr: decomposition failed");
        (MlxTensor::new(q), MlxTensor::new(r))
    }

    fn svd<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::svd returns (U, S, Vt)
        let (u, s, vt) =
            mlx_rs::linalg::svd(input.as_array()).expect("MlxBackend::svd: decomposition failed");
        (MlxTensor::new(u), MlxTensor::new(s), MlxTensor::new(vt))
    }

    fn inverse<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        // MLX linalg::inv for matrix inversion
        let array = mlx_rs::linalg::inv(input.as_array())
            .expect("MlxBackend::inverse: matrix singular or not square");
        MlxTensor::new(array)
    }

    fn cholesky_decomposition<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        let array = mlx_rs::linalg::cholesky(input.as_array(), Some(false))
            .expect("MlxBackend::cholesky: decomposition failed");
        MlxTensor::new(array)
    }

    fn solve_least_squares_cholsky<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        let array = mlx_rs::linalg::solve(a.as_array(), b.as_array())
            .expect("MlxBackend::solve_least_squares_cholsky: solve failed");
        MlxTensor::new(array)
    }

    fn tensor_product<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        // Generalized outer product
        let shape_a = a.as_array().shape();
        let shape_b = b.as_array().shape();

        let size_a: i32 = shape_a.iter().product();
        let size_b: i32 = shape_b.iter().product();

        // MLX reshape to column/row vectors
        let a_flat = a
            .as_array()
            .reshape(&[size_a, 1])
            .expect("MlxBackend::tensor_product: reshape a failed");
        let b_flat = b
            .as_array()
            .reshape(&[1, size_b])
            .expect("MlxBackend::tensor_product: reshape b failed");

        let res_flat = mlx_rs::ops::matmul(&a_flat, &b_flat)
            .expect("MlxBackend::tensor_product: matmul failed");

        // Construct new shape
        let mut new_shape = Vec::with_capacity(shape_a.len() + shape_b.len());
        new_shape.extend_from_slice(shape_a);
        new_shape.extend_from_slice(shape_b);

        let res = res_flat
            .reshape(&new_shape)
            .expect("MlxBackend::tensor_product: final reshape failed");

        MlxTensor::new(res)
    }
}
