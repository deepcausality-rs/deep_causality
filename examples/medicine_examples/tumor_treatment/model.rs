/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the TTFields optimisation: the tumour, the treatment objective, and the
//! gradient ascent that maximises it.
//!
//! The objective is a [`DifferentiableField`], written once over the `Scalar` bound. Evaluated at
//! [`FloatType`] it returns the alignment score; evaluated at `Dual` it returns that score's exact
//! gradient. The optimiser therefore ascends the true gradient, and the objective is stated once.

use crate::FloatType;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_core::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_num::{Lift, lift};
use deep_causality_rand::{Rng, Xoshiro256};

/// Tumour radius, in centimetres.
///
/// This and [`INVASION_BIAS`] stay `f64` because they bound a `random_range` draw, which takes a
/// primitive range. They are imaging inputs rather than model scalars: the tumour arrives as `f64`
/// and only the computation path is scalar-generic.
pub const TUMOR_RADIUS_CM: f64 = 2.0;

/// The seed for the voxel sampler, so every run reports the same tumour.
pub const SEED: u64 = 0x5DEECE66D;

/// How strongly cell division favours the invasion axis. Zero would leave the axes uniform on the
/// sphere, and the objective would have no preferred orientation to find.
pub const INVASION_BIAS: f64 = 0.8;

/// One tumour: the voxel centres, and the dominant axis of cell division at each.
pub struct TumorVolume {
    pub voxels: Vec<[f64; 3]>,
    pub cell_axes: Vec<[f64; 3]>,
}

/// The treatment objective: the mean alignment `⟨|E(θ,φ)·a|⟩` between a uniform field `E` pointing
/// in the direction `(θ, φ)` and the tumour's cell-division axes `a`.
///
/// Tumour Treating Fields disrupt mitosis by pulling on the charged machinery that separates
/// chromosomes, and they do it best when the field runs along the division axis. Maximising this
/// score is therefore the treatment goal, and the transducer orientation is what the clinician
/// controls.
pub struct Efficacy {
    pub cell_axes: Vec<[f64; 3]>,
}

impl DifferentiableField<2> for Efficacy {
    fn run<S: Scalar>(&self, orientation: &[S; 2]) -> S {
        let (theta, phi) = (orientation[0], orientation[1]);

        // The field direction in spherical coordinates.
        let ex = theta.sin() * phi.cos();
        let ey = theta.sin() * phi.sin();
        let ez = theta.cos();

        let total = self.cell_axes.iter().fold(lift::<S>(0.0), |sum, axis| {
            let dot = ex * lift::<S>(axis[0]) + ey * lift::<S>(axis[1]) + ez * lift::<S>(axis[2]);
            sum + dot.abs()
        });
        total / self.cell_axes.len().lift::<S>()
    }
}

/// One step of the ascent, kept so the run can be shown after it finishes.
#[derive(Debug, Clone, Copy)]
pub struct AscentStep {
    pub step: usize,
    pub score: FloatType,
    pub theta: FloatType,
    pub phi: FloatType,
    pub gradient: [FloatType; 2],
}

/// What the optimisation found.
#[derive(Default, Clone, Debug)]
pub struct Report {
    pub initial_score: FloatType,
    pub final_score: FloatType,
    pub theta: FloatType,
    pub phi: FloatType,
    pub trace: Vec<AscentStep>,
}

/// Samples a tumour: voxels in a box, each with a division axis biased toward the invasion
/// direction.
///
/// The clinical data stay `f64`, which is how they arrive from imaging. Only the computation path
/// is scalar-generic, so the objective above runs at the working precision and at `Dual`.
pub fn build_tumor(voxels: usize) -> TumorVolume {
    let mut rng = Xoshiro256::from_seed(SEED);
    let mut positions = Vec::with_capacity(voxels);
    let mut axes = Vec::with_capacity(voxels);

    for _ in 0..voxels {
        positions.push([
            rng.random_range(0.0..TUMOR_RADIUS_CM),
            rng.random_range(0.0..TUMOR_RADIUS_CM),
            rng.random_range(0.0..TUMOR_RADIUS_CM),
        ]);

        // A direction drawn around the +z invasion axis, then normalised onto the unit sphere.
        let dx = rng.random_range(-0.5..0.5);
        let dy = rng.random_range(-0.5..0.5);
        let dz = rng.random_range(-0.5..0.5) + INVASION_BIAS;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        axes.push([dx / length, dy / length, dz / length]);
    }

    TumorVolume {
        voxels: positions,
        cell_axes: axes,
    }
}

/// Gradient ascent on the efficacy field.
///
/// Each step reads the exact gradient `∇⟨|E·a|⟩` from the tangent functor and moves the
/// orientation along it. The run ends through the monad's error channel, which is what
/// `PropagatingEffect` carries, when the learning rate or the starting orientation is outside the
/// finite range, or when a gradient, an updated orientation or a score leaves it.
pub fn ascend(
    efficacy: &Efficacy,
    start: [FloatType; 2],
    learning_rate: FloatType,
    steps: usize,
) -> PropagatingEffect<Report> {
    if !is_finite(learning_rate) {
        return PropagatingEffect::from_error(failed(
            "the learning rate is outside the finite range",
        ));
    }
    if !is_finite(start[0]) || !is_finite(start[1]) {
        return PropagatingEffect::from_error(failed(
            "the starting orientation is outside the finite range",
        ));
    }

    let initial_score = efficacy.run(&start);
    let mut orientation = start;
    let mut score = initial_score;
    let mut trace = Vec::with_capacity(steps);

    for step in 1..=steps {
        let gradient = efficacy.gradient(&orientation);
        if !is_finite(gradient[0]) || !is_finite(gradient[1]) {
            return PropagatingEffect::from_error(failed("the gradient left the finite range"));
        }

        orientation[0] += learning_rate * gradient[0];
        orientation[1] += learning_rate * gradient[1];
        if !is_finite(orientation[0]) || !is_finite(orientation[1]) {
            return PropagatingEffect::from_error(failed(
                "the orientation update left the finite range",
            ));
        }
        score = efficacy.run(&orientation);
        if !is_finite(score) {
            return PropagatingEffect::from_error(failed(
                "the efficacy score left the finite range",
            ));
        }

        trace.push(AscentStep {
            step,
            score,
            theta: orientation[0],
            phi: orientation[1],
            gradient,
        });
    }

    PropagatingEffect::pure(Report {
        initial_score,
        final_score: score,
        theta: orientation[0],
        phi: orientation[1],
        trace,
    })
}

/// A finiteness check at the working precision. The `Scalar` bound carries `is_finite`, so this
/// stays precision-generic.
pub fn is_finite<S: Scalar>(value: S) -> bool {
    value.is_finite()
}

/// Names a failure for the monad's error channel.
pub fn failed(reason: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(reason.into()))
}
