/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;

// Helper function to create a simple scalar tensor for testing
pub fn scalar_tensor(value: f64) -> CausalTensor<f64> {
    CausalTensor::from_slice(&[value], &[])
}

// Helper function to create a simple vector tensor for testing
pub fn vector_tensor(data: Vec<f64>) -> CausalTensor<f64> {
    let len = data.len();
    CausalTensor::from_slice(&data, &[len])
}

// Helper function to create a simple matrix tensor for testing
pub fn matrix_tensor(data: Vec<f64>, rows: usize, cols: usize) -> CausalTensor<f64> {
    CausalTensor::from_slice(&data, &[rows, cols])
}

pub fn tensor_3d(data: Vec<f64>, dim0: usize, dim1: usize, dim2: usize) -> CausalTensor<f64> {
    CausalTensor::from_slice(&data, &[dim0, dim1, dim2])
}
