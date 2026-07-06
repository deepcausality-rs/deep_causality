/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Glioblastoma TTFields Treatment Optimization
//!
//! Optimizing Tumor Treating Fields (TTFields) electrode orientation, with the three DeepCausality
//! pillars on one problem:
//!
//! - **The tangent functor.** The treatment objective `⟨|E(θ,φ)·axis|⟩` is a `DifferentiableField`,
//!   so its gradient comes from autodiff. The optimizer ascends the *exact* gradient rather than
//!   sampling random perturbations the way the previous simulated-annealing version did.
//! - **Precision as a parameter.** One `FloatType` alias re-runs the objective and its gradient at
//!   `f32`, `f64`, or `Float106`.
//! - **The causal monad.** `PropagatingEffect` sequences the ascent and short-circuits through the
//!   error channel if the gradient ever leaves the finite range.

use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt};
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_num::FromPrimitive;

mod model;

use model::Efficacy;

/// Switch this alias to `f32`, `f64`, or `Float106` (the latter also needs
/// `use deep_causality_num::Float106;`). The objective and its autodiff gradient re-run at it.
pub type FloatType = f64;

fn main() {
    println!("=== Glioblastoma TTFields Optimization (autodiff gradient ascent) ===\n");

    // 1. Clinical Data Ingestion (Mock): a tumour with non-uniform cell-division directions.
    let tumor = model::build_mock_tumor(100);
    println!("Loaded Tumor Volume: {} voxels", tumor.voxels.len());
    let efficacy = Efficacy {
        cell_axes: tumor.cell_axes,
    };

    // Start just off the degenerate θ = 0 pole, then ascend the exact gradient.
    let start = [ft(0.1), ft(0.1)];
    let learning_rate = ft(0.6);

    // The workflow is a CausalFlow chain: from a starting orientation, ascend; a non-finite
    // gradient short-circuits the error channel.
    let pipeline = CausalFlow::value(start)
        .bind(move |p, _, _| match p.into_value() {
            Some(s) => ascend(&efficacy, s, learning_rate, ASCENT_STEPS),
            None => fail("no starting orientation"),
        })
        .into_effect();

    match pipeline.value_cloned() {
        Some(r) => {
            println!("\n=== Optimization Complete ===");
            println!(
                "Optimal Transducer Orientation: Theta={:.3}, Phi={:.3}",
                r.theta, r.phi
            );
            println!(
                "Disruption Index: {:.4} -> {:.4} over {} gradient steps",
                r.initial, r.final_score, r.steps
            );
        }
        None => eprintln!("Optimization failed: {:?}", pipeline.error()),
    }
}

/// Gradient ascent on the efficacy field: each step moves the orientation along `∇efficacy`, the
/// gradient supplied by the tangent functor (`field.gradient`).
fn ascend(
    efficacy: &Efficacy,
    start: [FloatType; 2],
    learning_rate: FloatType,
    steps: usize,
) -> PropagatingEffect<Report> {
    let initial = efficacy.run(&start);
    let mut params = start;
    let mut score = initial;

    for step in 1..=steps {
        let grad = efficacy.gradient(&params);
        if !model::finite(grad[0]) || !model::finite(grad[1]) {
            return fail("gradient left the finite range");
        }
        params[0] += learning_rate * grad[0];
        params[1] += learning_rate * grad[1];
        score = efficacy.run(&params);
        println!(
            "[Step {:>2}] efficacy {:.4}   (θ={:.3}, φ={:.3})   ∇=({:+.3}, {:+.3})",
            step, score, params[0], params[1], grad[0], grad[1]
        );
    }

    PropagatingEffect::pure(Report {
        initial,
        final_score: score,
        theta: params[0],
        phi: params[1],
        steps,
    })
}

fn fail<T: Default + Clone + std::fmt::Debug>(msg: &str) -> PropagatingEffect<T> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(msg.into())))
}

const TUMOR_RADIUS: f64 = 2.0; // cm
const ASCENT_STEPS: usize = 12;

/// Lift an exact `f64` constant into the working precision (fully qualified to dodge the inherent
/// `from_f64` some scalars carry).
fn ft(x: f64) -> FloatType {
    <FloatType as FromPrimitive>::from_f64(x).expect("constant lifts into FloatType")
}

// Represents the 3D grid of the tumor
struct TumorVolume {
    // Voxel positions
    voxels: Vec<[f64; 3]>,
    // The dominant axis of cell division at each voxel (unit vector)
    cell_axes: Vec<[f64; 3]>,
}

/// The optimization outcome carried out of the monadic ascent.
#[derive(Default, Clone, Debug)]
struct Report {
    initial: FloatType,
    final_score: FloatType,
    theta: FloatType,
    phi: FloatType,
    steps: usize,
}
