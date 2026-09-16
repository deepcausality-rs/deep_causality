/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCUBA decompression planner
//!
//! A dive plan answers one question: how does a diver surface while the dissolved nitrogen stays
//! in solution? The Bühlmann ZH-L16C algorithm answers it by tracking sixteen tissue compartments,
//! each absorbing and releasing nitrogen at its own rate, and holding the ascent whenever the
//! fastest-loaded compartment reaches its tolerance.
//!
//! Three DeepCausality abstractions carry the program.
//!
//! - **The lax monoidal structure.** Every compartment computation pairs a tension with the
//!   constant that belongs to it, and `ZipTensorWitness::zip_with` walks the two tensors slot by
//!   slot. The loading law and the ceiling law are each written once for one compartment, and the
//!   witness applies them to all sixteen.
//! - **The causal monad.** The dive is a chain of phases, each taking the diver state and
//!   returning the next one. `CausalFlow::try_step` sequences them and routes a failure straight
//!   to the error channel, so the phases below hold physiology alone.
//! - **The tangent functor.** The gas-loading rate `dp/dt` is what a dive computer watches, and it
//!   comes from evaluating the loading curve over `Dual`. The model states the curve once; the
//!   derivative follows from the type.
//!
//! The dive is simulated once. [`DiveProfile`] carries the result, and `utils_print` renders it.

mod model;
mod utils_print;

use deep_causality_calculus::DifferentiateExt;
use deep_causality_num::lift;
use model::{SchreinerLoading, dive_table_rows, plan_dive};
use utils_print::{print_dive_table, print_gas_loading_rate, print_header, print_simulation};

/// The dive this run plans: maximum depth in metres, time held at that depth in minutes.
const MAX_DEPTH_M: f64 = 30.0;
const BOTTOM_MINUTES: f64 = 20.0;

/// Where the gas-loading rate is sampled for the autodiff demonstration: compartment index, and
/// the elapsed time in minutes.
const RATE_SAMPLE_COMPARTMENT: usize = 0;
const RATE_SAMPLE_MINUTES: f64 = 10.0;

/// The working scalar. Switch it to `f32`, `deep_causality_num::BFloat16` or
/// `deep_causality_num::Float106`; the constants, the tissue tensions, the ceilings, the CNS clock
/// and the autodiff rate all recompute at that precision.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // Every row of the table is its own dive, planned by the same chain of phases.
    print_dive_table(&dive_table_rows()?);

    // The headline dive: descend, hold, ascend, surface, sequenced through the causal monad.
    let profile = plan_dive(
        lift::<FloatType>(MAX_DEPTH_M),
        lift::<FloatType>(BOTTOM_MINUTES),
    )?;
    print_simulation(&profile);

    // The tangent functor: one evaluation over `Dual` returns the tension and the rate at which it
    // is loading. The analytic rate k·(p_inspired − p) is printed beside it as a check.
    let loading = SchreinerLoading::at_depth(MAX_DEPTH_M, RATE_SAMPLE_COMPARTMENT);
    let (tension, rate) = loading.value_and_derivative(lift::<FloatType>(RATE_SAMPLE_MINUTES));
    print_gas_loading_rate(&loading, tension, rate);

    Ok(())
}
