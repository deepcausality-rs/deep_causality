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
}
