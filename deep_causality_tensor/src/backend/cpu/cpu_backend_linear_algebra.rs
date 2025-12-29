/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! LinearAlgebraBackend implementation for CpuBackend.

use super::CpuBackend;
use crate::Tensor as TensorTrait;
use crate::traits::{LinearAlgebraBackend, TensorData};
use core::iter::Sum;
use deep_causality_num::{RealField, Ring};
impl LinearAlgebraBackend for CpuBackend {
    fn matmul<T>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + Ring + Default + PartialOrd,
    {
        a.matmul(b).expect("CpuBackend::matmul: failed")
    }

    fn qr<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input.qr().expect("CpuBackend::qr: decomposition failed")
    }

    fn svd<T>(input: &Self::Tensor<T>) -> (Self::Tensor<T>, Self::Tensor<T>, Self::Tensor<T>)
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input.svd().expect("CpuBackend::svd: decomposition failed")
    }

    fn inverse<T>(input: &Self::Tensor<T>) -> Self::Tensor<T>
    where
        T: TensorData + RealField + Sum + PartialEq,
    {
        input
            .inverse()
            .expect("CpuBackend::inverse: matrix singular or not square")
    }
}
