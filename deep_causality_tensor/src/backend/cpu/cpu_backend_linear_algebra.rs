/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CpuBackend;
use crate::backend::{LinearAlgebraBackend, TensorData};
use crate::traits::tensor::Tensor;
use deep_causality_num::{RealField, Ring};
use std::iter::Sum;

impl LinearAlgebraBackend for CpuBackend {
    fn matmul<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        a.matmul(b).expect("CpuBackend::matmul failed")
    }

    fn qr<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input.qr().expect("CpuBackend::qr failed")
    }

    fn svd<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input.svd().expect("CpuBackend::svd failed")
    }

    fn inverse<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input.inverse().expect("CpuBackend::inverse failed")
    }

    fn cholesky_decomposition<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input
            .cholesky_decomposition()
            .expect("CpuBackend::cholesky_decomposition failed")
    }

    fn solve_least_squares_cholsky<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        Tensor::solve_least_squares_cholsky(a, b)
            .expect("CpuBackend::solve_least_squares_cholsky failed")
    }

    fn tensor_product<T>(lhs: &Self::Tensor<T>, rhs: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        lhs.tensor_product(rhs)
            .expect("CpuBackend::tensor_product failed")
    }
}
