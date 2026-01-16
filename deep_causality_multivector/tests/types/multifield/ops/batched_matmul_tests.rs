/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for BatchedMatMul trait implementations.

use deep_causality_multivector::BatchedMatMul;
use deep_causality_tensor::{CausalTensor};

#[test]
fn test_batched_matmul_rank_2_fallback() {
    // Rank 2 tensor (just a matrix) - should use standard matmul fallback
    let shape = [2, 2];

    // A = [[1, 2], [3, 4]]
    let data_a = vec![1.0, 2.0, 3.0, 4.0];
    let tensor_a = CausalTensor::<f32>::from_slice(&data_a, &shape);

    // B = [[1, 0], [0, 1]] (Identity)
    let data_b = vec![1.0, 0.0, 0.0, 1.0];
    let tensor_b = CausalTensor::<f32>::from_slice(&data_b, &shape);

    let result = tensor_a.batched_matmul(&tensor_b);

    assert_eq!(*result.shape(), shape);

    let res_data = CausalTensor::to_vec(result);
    // Should be equal to A
    assert_eq!(res_data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_batched_matmul_rank_3() {
    // Shape [2, 2, 2] -> Batch size 2, 2x2 matrices
    let shape = [2, 2, 2];

    // Batch 0: Identity, Batch 1: 2*Identity
    let data_a = vec![
        1.0, 0.0, 0.0, 1.0, // Batch 0
        2.0, 0.0, 0.0, 2.0, // Batch 1
    ];
    let tensor_a = CausalTensor::<f32>::from_slice(&data_a, &shape);

    // B: Scale factor matrix
    // Batch 0: [[2, 0], [0, 2]], Batch 1: [[3, 0], [0, 3]]
    let data_b = vec![
        2.0, 0.0, 0.0, 2.0, // Batch 0
        3.0, 0.0, 0.0, 3.0, // Batch 1
    ];
    let tensor_b = CausalTensor::<f32>::from_slice(&data_b, &shape);

    let result = tensor_a.batched_matmul(&tensor_b);

    assert_eq!(*result.shape(), shape);

    let res_data = CausalTensor::to_vec(result);

    // Expected:
    // Batch 0: I * 2I = 2I -> [2, 0, 0, 2]
    // Batch 1: 2I * 3I = 6I -> [6, 0, 0, 6]
    let expected = [2.0, 0.0, 0.0, 2.0, 6.0, 0.0, 0.0, 6.0];

    for (val, exp) in res_data.iter().zip(expected.iter()) {
        assert!((val - exp).abs() < 1e-5, "Expected {}, got {}", exp, val);
    }
}

#[test]
fn test_batched_matmul_rank_4() {
    // Shape [2, 2, 2, 2] -> Batch dimensions [2, 2], so 4 effective batches of 2x2 matrices
    let shape = [2, 2, 2, 2];

    // Create tensors filled with 1.0 everywhere
    // [ [1, 1], [1, 1] ] * [ [1, 1], [1, 1] ] = [ [2, 2], [2, 2] ]
    let size = 16; // 2*2*2*2
    let data = vec![1.0; size];

    let tensor_a = CausalTensor::<f32>::from_slice(&data, &shape);
    let tensor_b = CausalTensor::<f32>::from_slice(&data, &shape); // Same

    let result = tensor_a.batched_matmul(&tensor_b);

    assert_eq!(*result.shape(), shape);

    let res_data = CausalTensor::to_vec(result);

    // Every element should be 2.0
    for val in res_data {
        assert!((val - 2.0).abs() < 1e-5);
    }
}
