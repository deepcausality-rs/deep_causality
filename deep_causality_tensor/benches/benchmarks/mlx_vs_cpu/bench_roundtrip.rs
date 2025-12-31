/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{Criterion, criterion_group};
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::{CausalTensor, MlxBackend, MlxCausalTensor, TensorBackend};
use std::hint::black_box;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
const SIZES: [usize; 3] = [128, 512, 1024];

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
fn bench_cast_and_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_transfer");

    for &size in &SIZES {
        let len = size * size;
        let data: Vec<f64> = (0..len).map(|i| (i % 100) as f64).collect();
        let shape = vec![size, size];
        let cpu_tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

        // Measure full roundtrip: CPU(f64) -> GPU(f32) -> CPU(f64)
        // This includes allocation, casting, and memory copying.
        group.bench_function(format!("roundtrip_{}x{}", size, size), |b| {
            b.iter(|| {
                // 1. To GPU (includes f64->f32 cast)
                // Manual impl since helper doesn't exist
                let data_f64 = black_box(&cpu_tensor).as_slice();
                let data_f32: Vec<f32> = data_f64.iter().map(|&x| x as f32).collect();
                let mlx_tensor =
                    MlxCausalTensor::from_slice(&data_f32, black_box(cpu_tensor.shape()));

                // 2. Force eval to ensure upload finishes
                mlx_tensor.inner().as_array().eval().expect("eval");

                // 3. Back to CPU (includes f32->f64 cast)
                let res_f32 = mlx_tensor.into_inner();
                let _res_f64: Vec<f64> = MlxBackend::to_vec(&res_f32)
                    .iter()
                    .map(|&x| x as f64)
                    .collect();
            })
        });

        // Measure direct f32 upload (no cast overhead)
        group.bench_function(format!("direct_f32_{}x{}", size, size), |b| {
            // Pre-convert to f32 (simulate data already being in f32)
            let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();
            let shape = vec![size, size];

            b.iter(|| {
                // 1. Direct upload (no cast)
                let mlx_tensor = MlxCausalTensor::from_slice(black_box(&data_f32), &shape);

                // 2. Force eval
                mlx_tensor.inner().as_array().eval().expect("eval");

                // 3. Direct download (no cast)
                let _res = mlx_tensor.into_inner();
            })
        });
    }
    group.finish();
}

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
fn bench_cast_and_copy(_c: &mut Criterion) {
    println!("Skipping MLX benchmarks");
}

criterion_group!(
    name = roundtrip_benches;
    config = Criterion::default();
    targets = bench_cast_and_copy
);
