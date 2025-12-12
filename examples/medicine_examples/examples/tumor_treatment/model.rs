/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TUMOR_RADIUS, TumorVolume};

pub(crate) fn build_mock_tumor(n: usize) -> TumorVolume {
    let mut voxels = Vec::with_capacity(n);
    let mut cell_axes = Vec::with_capacity(n);

    for _i in 0..n {
        let u = rand_f64();
        let v = rand_f64();
        let w = rand_f64();

        // Random position in box
        voxels.push([u * TUMOR_RADIUS, v * TUMOR_RADIUS, w * TUMOR_RADIUS]);

        // Random axis of division (normalized)
        let dx = rand_f64() - 0.5;
        let dy = rand_f64() - 0.5;
        let dz = rand_f64() - 0.5;
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        cell_axes.push([dx / len, dy / len, dz / len]);
    }

    TumorVolume { voxels, cell_axes }
}

/// Simple pseudo-random (LCG) to avoid linking `rand` crate
pub(crate) fn rand_f64() -> f64 {
    // Just a placeholder. In real app use `rand` crate.
    // We use a static mutable seed hack for demo simplicity
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = (SEED.wrapping_mul(1664525).wrapping_add(1013904223)) % 4294967296;
        (SEED as f64) / 4294967296.0
    }
}

/// Simulation Kernel: Geometric Algebra Alignment
/// Returns total disruption score
pub(crate) fn evaluate_efficacy(
    tumor: &TumorVolume,
    (theta, phi): (f64, f64),
) -> Result<f64, Box<dyn std::error::Error>> {
    // 1. Calculate Electric Field Vector E based on transducer orientation
    // Assume uniform field E pointing in direction (theta, phi) for simplicity
    let ex = theta.sin() * phi.cos();
    let ey = theta.sin() * phi.sin();
    let ez = theta.cos();
    let e_vec = [ex, ey, ez];

    // Alignment = |E . a| where a is cell axis. (Inner product of vectors)
    let mut total_score = 0.0;

    for axis in &tumor.cell_axes {
        let dot = (e_vec[0] * axis[0]) + (e_vec[1] * axis[1]) + (e_vec[2] * axis[2]);
        // TTFields work best when E is PARALLEL to axis of division
        total_score += dot.abs(); // Magnitude of alignment
    }

    Ok(total_score / tumor.voxels.len() as f64)
}
