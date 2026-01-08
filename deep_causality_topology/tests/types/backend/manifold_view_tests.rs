/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CpuBackend, TensorBackend};
use deep_causality_topology::backend::ManifoldView;

#[test]
fn test_manifold_christoffel_constant() {
    // 2x2 Metric tensor (Identity)
    // Shape [2, 2] -> Rank 2 -> Constant
    let data = vec![1.0, 0.0, 0.0, 1.0];
    let metric = CpuBackend::create(&data, &[2, 2]);
    let view = ManifoldView::<CpuBackend, f32>::new(metric);

    let christoffel = view.compute_christoffel();
    let shape = christoffel.shape();

    // Should be [2, 2, 2] rank 3
    assert_eq!(shape, vec![2, 2, 2]);

    // Should be all zeros
    let vec_out = CpuBackend::to_vec(&christoffel);
    assert!(vec_out.iter().all(|&x| x == 0.0));
}

#[test]
fn test_manifold_christoffel_field() {
    // 1D Grid of 3 points. Metric is 2x2.
    // Shape [3, 2, 2].
    // We vary g_00 across the grid: 1.0, 2.0, 1.0.
    // g_11 = 1.0 constant.
    // Off-diagonals 0.

    let mut data = Vec::with_capacity(3 * 2 * 2);
    // Point 0: diag(1, 1)
    data.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);
    // Point 1: diag(2, 1)
    data.extend_from_slice(&[2.0, 0.0, 0.0, 1.0]);
    // Point 2: diag(1, 1)
    data.extend_from_slice(&[1.0, 0.0, 0.0, 1.0]);

    let metric = CpuBackend::create(&data, &[3, 2, 2]);
    let view = ManifoldView::<CpuBackend, f32>::new(metric);

    let christoffel = view.compute_christoffel();
    // Expected shape: [3, 2, 2, 2]
    // The last dimension (2) corresponds to the 'k' index of \Gamma^k_{ij}.
    // But wait, compute_christoffel appends 'n' to shape.
    // Input shape [3, 2, 2]. N=2.
    // Output shape [3, 2, 2, 2].
    assert_eq!(christoffel.shape(), vec![3, 2, 2, 2]);

    let out = CpuBackend::to_vec(&christoffel);

    // Let's check \Gamma^0_{00} at index 0.
    // Index mapping for [3, 2, 2, 2]:
    // (batch, i, j, m) or (batch, m, i, j)?
    // My implementation:
    // Matmul I^{mk} * Lower_{k, ij}.
    // I is [3, 2, 2] (indices m, k).
    // Lower is [3, 2, 4] (indices k, ij).
    // Result [3, 2, 4] -> [3, 2, 2, 2] (m, i, j).
    // So indices are (batch, m, i, j).

    // Check \Gamma^0_{00} at batch 0.
    // Indices: (0, 0, 0, 0). Flat index 0.
    // \partial_0 g_{00} at batch 0:
    // Central diff: (g(1) - g(2)) / 2 = (2 - 1) / 2 = 0.5.
    // \Gamma^0_{00} = 0.5 * g^{00} * \partial_0 g_{00}
    // g^{00} at batch 0 is 1/1 = 1.
    // Result = 0.5 * 1 * 0.5 = 0.25.

    let gamma_0_00_0 = out[0]; // Offset 0
    assert!(
        (gamma_0_00_0 - 0.25).abs() < 1e-5,
        "Gamma^0_00 at x=0 should be 0.25, got {}",
        gamma_0_00_0
    );

    // Check at batch 1.
    // Indices (1, 0, 0, 0).
    // Offset = 1 * (2*2*2) + 0 = 8.
    // \partial_0 g_{00} at batch 1:
    // (g(2) - g(0)) / 2 = (1 - 1) / 2 = 0.
    // Result should be 0.
    let gamma_0_00_1 = out[8];
    assert!(
        gamma_0_00_1.abs() < 1e-5,
        "Gamma^0_00 at x=1 should be 0, got {}",
        gamma_0_00_1
    );

    // Check at batch 2.
    // Indices (2, 0, 0, 0). Offset 16.
    // \partial_0 g_{00}: (g(0) - g(1)) / 2 = (1 - 2) / 2 = -0.5.
    // g^{00} at batch 2 is 1/1 = 1.
    // Result = 0.5 * 1 * -0.5 = -0.25.
    let gamma_0_00_2 = out[16];
    assert!(
        (gamma_0_00_2 - (-0.25)).abs() < 1e-5,
        "Gamma^0_00 at x=2 should be -0.25, got {}",
        gamma_0_00_2
    );
}
