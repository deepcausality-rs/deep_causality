/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! EinSum operator benchmarks comparing CPU vs MLX performance.
use criterion::{Criterion, criterion_group};
use std::hint::black_box;

// Common tensor sizes to benchmark
const SIZES: [usize; 4] = [128, 512, 1024, 2048];

// ============================================================================
// CPU Benchmarks
// ============================================================================

use deep_causality_tensor::{CausalTensor, GenericEinSumOp};

fn bench_einsum_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("einsum_cpu");
    group.sample_size(10);

    for &size in &SIZES {
        // Skip 2048 for CPU - too slow
        if size >= 2048 {
            continue;
        }

        let n = size * size;
        let data: Vec<f64> = (0..n).map(|i| (i as f64) * 0.001).collect();
        let mat = CausalTensor::new(data.clone(), vec![size, size]).expect("create matrix");

        // MatMul (O(N³))
        group.bench_function(format!("matmul_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::mat_mul(black_box(mat.clone()), black_box(mat.clone()));
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });

        // Transpose (O(N²))
        group.bench_function(format!("transpose_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::transpose(black_box(mat.clone()), vec![1, 0]);
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });

        // Trace (O(N))
        group.bench_function(format!("trace_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::trace(black_box(mat.clone()), 0, 1);
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });

        // ElementWiseProduct (O(N²))
        group.bench_function(format!("hadamard_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::element_wise_product(
                    black_box(mat.clone()),
                    black_box(mat.clone()),
                );
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });

        // DiagonalExtraction (O(N))
        group.bench_function(format!("diag_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::diagonal_extraction(black_box(mat.clone()), 0, 1);
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });

        // Reduction (sum all elements, O(N²))
        group.bench_function(format!("reduction_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::reduction(black_box(mat.clone()), vec![0, 1]);
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });
    }

    // Vector operations
    for &size in &[128, 512, 1024] {
        let vec_data: Vec<f64> = (0..size).map(|i| (i as f64) * 0.001).collect();
        let vec_tensor = CausalTensor::new(vec_data, vec![size]).expect("create vector");

        // DotProd (O(N))
        group.bench_function(format!("dotprod_{}", size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::dot_prod(
                    black_box(vec_tensor.clone()),
                    black_box(vec_tensor.clone()),
                );
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });
    }

    // BatchMatMul (3D tensors: batch x M x N)
    for &size in &[32, 64, 128] {
        let batch = 8;
        let n = batch * size * size;
        let data: Vec<f64> = (0..n).map(|i| (i as f64) * 0.001).collect();
        let batch_mat =
            CausalTensor::new(data, vec![batch, size, size]).expect("create batch matrix");

        group.bench_function(format!("batch_matmul_{}x{}x{}", batch, size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::batch_mat_mul(
                    black_box(batch_mat.clone()),
                    black_box(batch_mat.clone()),
                );
                let _ = CausalTensor::ein_sum(&ast).expect("einsum");
            })
        });
    }

    group.finish();
}

// ============================================================================
// MLX GPU Benchmarks
// ============================================================================

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
fn bench_einsum_mlx(c: &mut Criterion) {
    use deep_causality_tensor::{MlxBackend, MlxCausalTensor, TensorBackend};

    let mut group = c.benchmark_group("einsum_mlx");
    group.sample_size(100);

    for &size in &SIZES {
        let n = size * size;
        let data: Vec<f32> = (0..n).map(|i| (i as f32) * 0.001).collect();
        let mat = MlxCausalTensor::from_slice(&data, &[size, size]);
        // Extract inner MlxTensor for AST construction
        let inner_mat = mat.inner().clone();

        // MatMul (O(N³))
        group.bench_function(format!("matmul_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::mat_mul(
                    black_box(inner_mat.clone()),
                    black_box(inner_mat.clone()),
                );
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });

        // Transpose (O(N²))
        group.bench_function(format!("transpose_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::transpose(black_box(inner_mat.clone()), vec![1, 0]);
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });

        // Trace (O(N))
        group.bench_function(format!("trace_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::trace(black_box(inner_mat.clone()), 0, 1);
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });

        // ElementWiseProduct / Hadamard (O(N²))
        group.bench_function(format!("hadamard_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::element_wise_product(
                    black_box(inner_mat.clone()),
                    black_box(inner_mat.clone()),
                );
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });

        // DiagonalExtraction (O(N))
        group.bench_function(format!("diag_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::diagonal_extraction(black_box(inner_mat.clone()), 0, 1);
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });

        // Reduction (sum all elements, O(N²))
        group.bench_function(format!("reduction_{}x{}", size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::reduction(black_box(inner_mat.clone()), vec![0, 1]);
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });
    }

    // Vector operations
    for &size in &SIZES {
        let vec_data: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();
        let vec_tensor = MlxCausalTensor::from_slice(&vec_data, &[size]);
        let inner_vec = vec_tensor.inner().clone();

        // DotProd (O(N))
        group.bench_function(format!("dotprod_{}", size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::dot_prod(
                    black_box(inner_vec.clone()),
                    black_box(inner_vec.clone()),
                );
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });
    }

    // BatchMatMul (3D tensors: batch x M x N)
    for &size in &[32, 64, 128, 256] {
        let batch = 8;
        let n = batch * size * size;
        let data: Vec<f32> = (0..n).map(|i| (i as f32) * 0.001).collect();
        let batch_mat = MlxCausalTensor::from_slice(&data, &[batch, size, size]);
        let inner_batch = batch_mat.inner().clone();

        group.bench_function(format!("batch_matmul_{}x{}x{}", batch, size, size), |b| {
            b.iter(|| {
                let ast = GenericEinSumOp::batch_mat_mul(
                    black_box(inner_batch.clone()),
                    black_box(inner_batch.clone()),
                );
                let res = MlxBackend::ein_sum(&ast).expect("einsum");
                res.as_array().eval().expect("eval");
            })
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(einsum_cpu_benches, bench_einsum_cpu);

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
criterion_group!(einsum_mlx_benches, bench_einsum_mlx);

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
criterion_group!(einsum_mlx_benches, bench_einsum_cpu);
