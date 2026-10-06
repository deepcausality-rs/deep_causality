/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Closed-Loop Insulin Pump as a Corrective `intervene` Loop
//!
//! Blood glucose in a type-1 diabetic patient drifts under baseline
//! hepatic production and three scheduled meals across a six-hour
//! window. Without a pump, glucose climbs into hyperglycemic range and
//! eventually crosses the ketoacidosis threshold. The monitor watches
//! the post-tick glucose reading. When it climbs above 180 mg/dL, the
//! pump computes the bolus units needed to reach target and applies
//! `.alternate_value(target)` on the chain. The next tick advances from the
//! corrected glucose level.
//!
//! Two trajectories run side by side.
//!
//! * Open loop. No pump activity. Glucose climbs monotonically across
//!   the meal events and crosses the ketoacidosis threshold within the
//!   six-hour window.
//! * Closed loop. Each meal spike triggers a corrective bolus.
//!   Glucose stays within the physiological range. The `EffectLog`
//!   records every bolus event with its timestamp and dose.
//!
//! Same chain, same perturbation schedule, same patient parameters.
//! The only difference is whether the corrective `.alternate_value` fires.
//! The catastrophic outcome of the open-loop run is the failure the
//! corrective interventions prevent.

mod model;
pub mod model_types;
mod model_utils;

use crate::model_types::{
    FloatType, HYPERGLYCEMIC_THRESHOLD, N_TICKS, PumpContext, PumpProcess, nominal_pump_context,
    read,
};
use causal_correction_examples::print_utils;
use deep_causality_core::{CausalFlow, CausalityError};

fn main() -> Result<(), CausalityError> {
    println!("=== Closed-Loop Insulin Pump as a Corrective `intervene` Loop ===\n");

    let pump = nominal_pump_context().map_err(|err| CausalityError::GraphError(err.to_string()))?;
    let open = run_open_loop(pump.clone());
    let closed = run_closed_loop(pump);

    model_utils::print_section("Open loop (no pump)", &open)?;
    model_utils::print_section("Closed loop (monitor + corrective bolus)", &closed)?;

    println!("=== Summary ===");
    model_utils::summary_line("Open loop  ", &open);
    model_utils::summary_line("Closed loop", &closed);

    println!(
        "\nThe open-loop trajectory crosses the ketoacidosis threshold\n\
         (300 mg/dL) within the six-hour window. The closed-loop run\n\
         uses the same meal schedule, but each hyperglycemic excursion\n\
         triggers an `.alternate_value(target)` call. Glucose stays bounded\n\
         and the patient finishes the window in normoglycemia."
    );

    println!("\n--- Closed-loop EffectLog (per-tick readings and bolus events) ---");
    print_utils::print_effect_log(closed.logs());
    Ok(())
}

/// Open loop: each tick is just `simulate_step`, run `N_TICKS` times. No monitor, no bolus.
fn run_open_loop(pump: PumpContext) -> PumpProcess<FloatType> {
    CausalFlow::from(model::initial_process(pump))
        .iterate_n(N_TICKS as usize, |tick| tick.bind(model::simulate_step))
        .into_process()
}

/// Closed loop: the same tick, but each one `branch`es on the monitor — on a hyperglycemic excursion
/// the `bolus` step records the bolus, accumulates the delivered units, and `alternate_value`s the
/// corrected glucose. The only difference from the open loop is the `branch`. The trigger and the
/// bolus both read the pump context the process carries.
fn run_closed_loop(pump: PumpContext) -> PumpProcess<FloatType> {
    CausalFlow::from(model::initial_process(pump))
        .iterate_n(N_TICKS as usize, |tick| {
            tick.bind(model::simulate_step).branch_with(
                // A predicate has no error channel. It runs only on a value, so `simulate_step`
                // succeeded and read the same threshold from the same context.
                |glucose, _state, ctx| {
                    let pump = ctx.expect("simulate_step returns the pump context it read");
                    let threshold = read(pump, HYPERGLYCEMIC_THRESHOLD)
                        .expect("simulate_step read the hyperglycemic threshold from this context");
                    *glucose > threshold
                },
                |hot| hot.bind(model::bolus),
                |cold| cold,
            )
        })
        .into_process()
}
