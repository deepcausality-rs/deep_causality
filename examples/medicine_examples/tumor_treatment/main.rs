/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Aiming Tumour Treating Fields at a glioblastoma
//!
//! Tumour Treating Fields are an approved glioblastoma therapy: transducer arrays on the scalp
//! put a low-intensity alternating electric field through the tumour, and the field disrupts cell
//! division by pulling on the charged machinery that separates chromosomes. The disruption is
//! strongest when the field runs **along** a cell's division axis, so the clinical question is
//! which direction to aim the arrays.
//!
//! The objective is the mean alignment `⟨|E(θ,φ)·a|⟩` between the field direction and the tumour's
//! division axes. Maximising it is a two-parameter optimisation, and three abstractions carry it.
//!
//! - **The tangent functor.** The objective is written once over the `Scalar` bound, so evaluating
//!   it over `Dual` returns its exact gradient. The optimiser ascends that gradient directly.
//! - **Precision as a parameter.** One alias re-runs the objective and its gradient at another
//!   scalar.
//! - **The causal monad.** `CausalFlow` sequences the ascent, and a gradient that leaves the
//!   finite range ends the run through the error channel.
//!
//! The tumour is sampled with a seeded generator, so every run reports the same anatomy.

mod model;
mod utils_print;

use deep_causality_core::CausalFlow;
use deep_causality_num::{Float106, const_scalar_from_float};
use model::{Efficacy, ascend, build_tumor, failed};
use utils_print::{print_header, print_outcome, print_trace, print_tumor};

/// Voxels sampled from the tumour volume.
const VOXELS: usize = 100;

/// Where the ascent starts, in radians: near the equator, with the field roughly perpendicular to
/// the tumour's invasion axis. That is the orientation a clinician would improve on.
const START_THETA: FloatType = const_scalar_from_float!(FloatType, 1.4);
const START_PHI: FloatType = const_scalar_from_float!(FloatType, 0.8);

/// Step size and step count for the ascent.
const LEARNING_RATE: FloatType = const_scalar_from_float!(FloatType, 0.6);
const ASCENT_STEPS: usize = 12;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the objective,
/// its autodiff gradient and the whole ascent re-run at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let tumor = build_tumor(VOXELS);
    print_tumor(&tumor);

    let efficacy = Efficacy {
        cell_axes: tumor.cell_axes,
    };
    let start = [START_THETA, START_PHI];

    // The ascent as a flow: the starting orientation enters the chain, and a value that leaves the
    // finite range short-circuits to the error channel. The ascent's own error is what the flow
    // carries out, so the run names the value that failed.
    let outcome = CausalFlow::value(start)
        .try_step(move |orientation| {
            let (outcome, _, _, _) =
                ascend(&efficacy, orientation, LEARNING_RATE, ASCENT_STEPS).into_parts();
            outcome?
                .into_value()
                .ok_or_else(|| failed("the ascent returned no orientation"))
        })
        .finish()?;

    print_trace(&outcome);
    print_outcome(&outcome);
    Ok(())
}
