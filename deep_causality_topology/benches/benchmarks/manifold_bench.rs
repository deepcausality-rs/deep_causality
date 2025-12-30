/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group};
use deep_causality_tensor::TensorBackend;
use deep_causality_topology::backend::ManifoldView;

#[cfg(feature = "mlx")]
use deep_causality_tensor::MlxBackend as TestBackend;

#[cfg(not(feature = "mlx"))]
use deep_causality_tensor::CpuBackend as TestBackend;

fn generate_random_metric(
    grid_size: usize,
    dim: usize,
) -> <TestBackend as TensorBackend>::Tensor<f32> {
    // Shape: [grid, grid, grid, dim, dim]
    let mut shape = vec![grid_size; 3];
    shape.push(dim);
    shape.push(dim);

    let total_elements: usize = shape.iter().product();
    let batch_size = grid_size.pow(3);
    let _matrix_size = dim * dim;

    let mut data = Vec::with_capacity(total_elements);

    for b in 0..batch_size {
        // Spatially varying factor to ensure non-zero derivatives
        // Use pseudo-random deterministic variation based on batch index
        let factor = 1.0 + 0.5 * ((b % 10) as f32 / 10.0);

        for r in 0..dim {
            for c in 0..dim {
                if r == c {
                    // Diagonal elements: factor + perturbation
                    // Large enough to ensure invertibility (diagonal dominance)
                    data.push(factor + 2.0);
                } else {
                    // Off-diagonal: small perturbation
                    let perturbation = ((b * r * c + r + c) % 100) as f32 / 1000.0;
                    data.push(perturbation);
                }
            }
        }
    }

    TestBackend::create(&data, &shape)
}

fn bench_christoffel(c: &mut Criterion) {
    let mut group = c.benchmark_group("Manifold Operations");

    // Test for different grid sizes
    let sizes: &[usize] = &[5, 10, 20];
    for &grid_size in sizes {
        let dim: usize = 4; // Spacetime (4x4 metric)
        let total_ops = (grid_size as u64).pow(3) * (dim as u64) * (dim as u64);

        group.throughput(Throughput::Elements(total_ops));

        group.bench_with_input(
            BenchmarkId::new(
                "compute_christoffel",
                format!("grid_{}^3_dim_{}", grid_size, dim),
            ),
            &grid_size,
            |b, &size| {
                // Setup: Create manifold outside the loop
                let metric = generate_random_metric(size, dim);
                let manifold = ManifoldView::<TestBackend, f32>::new(metric);

                b.iter(|| {
                    let res = manifold.compute_christoffel();
                    // Force MLX evaluation by accessing the underlying data
                    #[cfg(feature = "mlx")]
                    {
                        let _ = TestBackend::to_vec(&res);
                    }
                    std::hint::black_box(res)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(manifold_benches, bench_christoffel);
