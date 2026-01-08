/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for BatchedMatMul trait implementations.

use deep_causality_multivector::BatchedMatMul;
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// CpuBackend BatchedMatMul tests
// =============================================================================

#[test]
fn test_batched_matmul_2d_fallback() {
    // 2D tensors should fall back to regular matmul
    let a = CpuBackend::from_shape_fn(&[2, 2], |idx| (idx[0] * 2 + idx[1] + 1) as f32);
    let b = CpuBackend::from_shape_fn(
        &[2, 2],
        |idx| {
            if idx[0] == idx[1] { 1.0f32 } else { 0.0f32 }
        },
    ); // Identity

    let c = CpuBackend::batched_matmul(&a, &b);
    let result = CpuBackend::to_vec(&c);
    let original = CpuBackend::to_vec(&a);

    // A * I = A
    for (r, o) in result.iter().zip(original.iter()) {
        assert!((r - o).abs() < 1e-5, "A * I = A failed: {} != {}", r, o);
    }
}

#[test]
fn test_batched_matmul_3d_single_batch() {
    // [1, 2, 2] tensors - single batch
    let a = CpuBackend::from_shape_fn(&[1, 2, 2], |idx| (idx[1] * 2 + idx[2] + 1) as f32);
    let b = CpuBackend::from_shape_fn(
        &[1, 2, 2],
        |idx| {
            if idx[1] == idx[2] { 1.0f32 } else { 0.0f32 }
        },
    );

    let c = CpuBackend::batched_matmul(&a, &b);
    let shape = CpuBackend::shape(&c);

    assert_eq!(shape, vec![1, 2, 2]);
}

#[test]
fn test_batched_matmul_3d_multiple_batches() {
    // [3, 2, 2] tensors - three batches
    let a = CpuBackend::from_shape_fn(&[3, 2, 2], |idx| (idx[0] * 4 + idx[1] * 2 + idx[2]) as f32);
    let b = CpuBackend::from_shape_fn(
        &[3, 2, 2],
        |idx| {
            if idx[1] == idx[2] { 1.0f32 } else { 0.0f32 }
        },
    ); // Identity in each batch

    let c = CpuBackend::batched_matmul(&a, &b);
    let shape = CpuBackend::shape(&c);
    let result = CpuBackend::to_vec(&c);
    let original = CpuBackend::to_vec(&a);

    assert_eq!(shape, vec![3, 2, 2]);

    // Since B is identity in each batch, C should equal A
    for (r, o) in result.iter().zip(original.iter()) {
        assert!((r - o).abs() < 1e-5);
    }
}

#[test]
fn test_batched_matmul_5d_field_shape() {
    // [2, 2, 2, 2, 2] - typical CausalMultiField shape
    let a = CpuBackend::from_shape_fn(&[2, 2, 2, 2, 2], |idx| {
        if idx[3] == idx[4] { 1.0f32 } else { 0.0f32 }
    }); // Identity matrices
    let b = CpuBackend::from_shape_fn(&[2, 2, 2, 2, 2], |idx| {
        if idx[3] == idx[4] { 1.0f32 } else { 0.0f32 }
    });

    let c = CpuBackend::batched_matmul(&a, &b);
    let shape = CpuBackend::shape(&c);
    let result = CpuBackend::to_vec(&c);

    assert_eq!(shape, vec![2, 2, 2, 2, 2]);

    // I * I = I, so check diagonal elements are 1
    // Total elements: 2*2*2*2*2 = 32
    // Identity matrices mean [i,j,k,r,c] has 1 when r==c
    for idx0 in 0..2 {
        for idx1 in 0..2 {
            for idx2 in 0..2 {
                for r in 0..2 {
                    for c in 0..2 {
                        let linear = idx0 * 16 + idx1 * 8 + idx2 * 4 + r * 2 + c;
                        if r == c {
                            assert!(
                                (result[linear] - 1.0).abs() < 1e-5,
                                "Diagonal should be 1 at ({},{},{},{},{})",
                                idx0,
                                idx1,
                                idx2,
                                r,
                                c
                            );
                        } else {
                            assert!(
                                result[linear].abs() < 1e-5,
                                "Off-diagonal should be 0 at ({},{},{},{},{})",
                                idx0,
                                idx1,
                                idx2,
                                r,
                                c
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_batched_matmul_associativity() {
    // (A * B) * C = A * (B * C)
    let a = CpuBackend::from_shape_fn(&[2, 2, 2], |idx| {
        (idx[0] * 4 + idx[1] * 2 + idx[2] + 1) as f32
    });
    let b = CpuBackend::from_shape_fn(&[2, 2, 2], |idx| {
        ((idx[0] * 4 + idx[1] * 2 + idx[2]) % 3 + 1) as f32
    });
    let c = CpuBackend::from_shape_fn(&[2, 2, 2], |idx| {
        ((idx[0] * 4 + idx[1] * 2 + idx[2]) % 5 + 1) as f32
    });

    let ab = CpuBackend::batched_matmul(&a, &b);
    let ab_c = CpuBackend::batched_matmul(&ab, &c);

    let bc = CpuBackend::batched_matmul(&b, &c);
    let a_bc = CpuBackend::batched_matmul(&a, &bc);

    let lhs = CpuBackend::to_vec(&ab_c);
    let rhs = CpuBackend::to_vec(&a_bc);

    for (l, r) in lhs.iter().zip(rhs.iter()) {
        assert!((l - r).abs() < 1e-3, "Associativity failed: {} != {}", l, r);
    }
}

// =============================================================================
// MlxBackend BatchedMatMul tests
// =============================================================================

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx_batched_matmul_tests {
    use super::*;
    use deep_causality_tensor::MlxBackend;

    #[test]
    fn test_mlx_batched_matmul_delegates_to_matmul() {
        // MLX should just call regular matmul (which supports broadcasting)
        let a = MlxBackend::from_shape_fn(&[2, 2, 2], |idx| {
            (idx[0] * 4 + idx[1] * 2 + idx[2] + 1) as f32
        });
        let b =
            MlxBackend::from_shape_fn(
                &[2, 2, 2],
                |idx| {
                    if idx[1] == idx[2] { 1.0f32 } else { 0.0f32 }
                },
            );

        let c = MlxBackend::batched_matmul(&a, &b);
        let shape = MlxBackend::shape(&c);

        assert_eq!(shape, vec![2, 2, 2]);
    }

    #[test]
    fn test_mlx_batched_matmul_identity() {
        let a = MlxBackend::from_shape_fn(&[2, 2, 2], |idx| {
            (idx[0] * 4 + idx[1] * 2 + idx[2] + 1) as f32
        });
        let identity =
            MlxBackend::from_shape_fn(
                &[2, 2, 2],
                |idx| {
                    if idx[1] == idx[2] { 1.0f32 } else { 0.0f32 }
                },
            );

        let c = MlxBackend::batched_matmul(&a, &identity);
        let result = MlxBackend::to_vec(&c);
        let original = MlxBackend::to_vec(&a);

        for (r, o) in result.iter().zip(original.iter()) {
            assert!((r - o).abs() < 1e-5);
        }
    }
}
