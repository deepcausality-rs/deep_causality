/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TUMOR_RADIUS, TumorVolume};
use deep_causality_calculus::{DifferentiableField, Scalar};

pub(crate) fn build_mock_tumor(n: usize) -> TumorVolume {
    let mut voxels = Vec::with_capacity(n);
    let mut cell_axes = Vec::with_capacity(n);

    for _i in 0..n {
        let u = rand_f64();
        let v = rand_f64();
        let w = rand_f64();

        // Random position in box
        voxels.push([u * TUMOR_RADIUS, v * TUMOR_RADIUS, w * TUMOR_RADIUS]);

        // Random axis of division with a dominant invasion direction (biased toward +z), so the
        // tumour has a real preferred orientation for the field to align with.
        let dx = rand_f64() - 0.5;
        let dy = rand_f64() - 0.5;
        let dz = rand_f64() - 0.5 + 0.8;
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        cell_axes.push([dx / len, dy / len, dz / len]);
    }

    TumorVolume { voxels, cell_axes }
}

/// Simple pseudo-random (LCG) to avoid linking `rand` crate
pub(crate) fn rand_f64() -> f64 {
    // Just a placeholder. In real app use `rand` crate.
    // Atomic seed keeps the demo dependency-free and free of `unsafe`.
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(12345);
    let next = (SEED
        .load(Ordering::Relaxed)
        .wrapping_mul(1664525)
        .wrapping_add(1013904223))
        % 4294967296;
    SEED.store(next, Ordering::Relaxed);
    (next as f64) / 4294967296.0
}

/// The treatment objective as a differentiable field of the transducer orientation `(θ, φ)`: the
/// mean alignment `⟨|E(θ,φ)·a|⟩` of a uniform field `E` with the tumour's cell-division axes `a`.
/// TTFields disrupt mitosis best when the field is parallel to the division axis, so maximizing
/// this score is the treatment goal.
///
/// Written once over the `Scalar` bound, so the tangent functor differentiates it: the optimizer
/// ascends the *exact* gradient `∇⟨|E·a|⟩` instead of sampling random perturbations. The clinical
/// data (the axes) stay `f64`; only the computation path is scalar-generic.
pub(crate) struct Efficacy {
    pub cell_axes: Vec<[f64; 3]>,
}

impl DifferentiableField<2> for Efficacy {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let (theta, phi) = (p[0], p[1]);
        // Uniform field E in the spherical direction (θ, φ).
        let ex = theta.sin() * phi.cos();
        let ey = theta.sin() * phi.sin();
        let ez = theta.cos();

        let mut total = S::from_f64(0.0).expect("zero lifts into the working scalar");
        for axis in &self.cell_axes {
            let ax = S::from_f64(axis[0]).expect("axis lifts into the working scalar");
            let ay = S::from_f64(axis[1]).expect("axis lifts into the working scalar");
            let az = S::from_f64(axis[2]).expect("axis lifts into the working scalar");
            total += (ex * ax + ey * ay + ez * az).abs();
        }
        total / S::from_f64(self.cell_axes.len() as f64).expect("count lifts")
    }
}

/// Finiteness check at the working precision (used to guard the ascent through the monad's error
/// channel). The `Scalar` bound exposes `is_finite`, so this stays precision-generic.
pub(crate) fn finite<S: Scalar>(x: S) -> bool {
    x.is_finite()
}
