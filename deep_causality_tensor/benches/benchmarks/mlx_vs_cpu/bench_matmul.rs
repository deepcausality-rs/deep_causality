/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};
use deep_causality_tensor::{CausalTensor, CpuBackend, LinearAlgebraBackend};
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::{MlxBackend, MlxCausalTensor};
use std::hint::black_box;

// Matrices sizes to benchmark
const SIZES: [usize; 4] = [128, 512, 1024, 2048];

fn bench_matmul_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("matmul_cpu");
    group.sample_size(10); // Matmul is slow on CPU for large N

    for &size in &SIZES {
        // Skip 2048 for CPU as it takes too long (~10s per iter)
        if size > 1024 {
            continue;
        }
        let len = size * size;
        let data: Vec<f64> = (0..len).map(|i| (i % 100) as f64).collect();
        let shape = vec![size, size];

        // We use CpuBackend explicitly to avoid any ambiguity
        let tensor_a = CausalTensor::new(data.clone(), shape.clone()).unwrap();
        let tensor_b = CausalTensor::new(data, shape).unwrap();

        group.bench_function(format!("cpu_{}x{}", size, size), |b| {
            b.iter(|| {
                // CpuBackend uses f64
                let _ =
                    CpuBackend::matmul(black_box(tensor_a.inner()), black_box(tensor_b.inner()));
            })
        });
    }
    group.finish();
}

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
fn bench_matmul_mlx(c: &mut Criterion) {
    let mut group = c.benchmark_group("matmul_mlx");

    for &size in &SIZES {
        let len = size * size;
        let data: Vec<f32> = (0..len).map(|i| (i % 100) as f32).collect();
        let shape = vec![size, size];

        // Create MLX tensors directly (f32)
        let tensor_a = MlxCausalTensor::new(data.clone(), shape.clone()).unwrap();
        let tensor_b = MlxCausalTensor::new(data, shape).unwrap();

        group.bench_function(format!("mlx_{}x{}", size, size), |b| {
            b.iter(|| {
                let res =
                    MlxBackend::matmul(black_box(tensor_a.inner()), black_box(tensor_b.inner()));
                let _ = res.as_array().eval().expect("eval failed");
            })
        });
    }
    group.finish();
}

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
fn bench_matmul_mlx(_c: &mut Criterion) {
    // No-op if MLX not enabled
    println!("Skipping MLX benchmarks (feature disabled or wrong OS)");
}

criterion_group!(
    name = mlx_vs_cpu_benches;
    config = Criterion::default();
    targets = bench_matmul_cpu, bench_matmul_mlx
);
