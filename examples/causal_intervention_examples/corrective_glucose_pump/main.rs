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
//! `.intervene(target)` on the chain. The next tick advances from the
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
//! The only difference is whether the corrective `.intervene` fires.
//! The catastrophic outcome of the open-loop run is the failure the
//! corrective interventions prevent.

mod model;
pub mod model_types;
mod model_utils;

use crate::model_types::{FloatType, N_TICKS, PumpProcess};
use causal_intervention_examples::print_utils;
use deep_causality_core::{EffectValue, Intervenable};

fn main() {
    println!("=== Closed-Loop Insulin Pump as a Corrective `intervene` Loop ===\n");

    let open = run_open_loop();
    let closed = run_closed_loop();

    model_utils::print_section("Open loop (no pump)", &open);
    model_utils::print_section("Closed loop (monitor + corrective bolus)", &closed);

    println!("=== Summary ===");
    model_utils::summary_line("Open loop  ", &open);
    model_utils::summary_line("Closed loop", &closed);

    println!(
        "\nThe open-loop trajectory crosses the ketoacidosis threshold\n\
         (300 mg/dL) within the six-hour window. The closed-loop run\n\
         uses the same meal schedule, but each hyperglycemic excursion\n\
         triggers an `.intervene(target)` call. Glucose stays bounded\n\
         and the patient finishes the window in normoglycemia."
    );

    println!("\n--- Closed-loop EffectLog (per-tick readings and bolus events) ---");
    print_utils::print_effect_log(&closed.logs);
}

fn run_open_loop() -> PumpProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);
    }
    process
}

fn run_closed_loop() -> PumpProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);

        let current = match &process.value {
            EffectValue::Value(v) => *v,
            _ => continue,
        };
        let cfg = process.context.clone().expect("PumpConfig present");
        if current > cfg.hyperglycemic_threshold {
            let (corrected, units) = model::corrective_bolus(current, &cfg);
            process.state.bolus_count += 1;
            process.state.total_insulin_units += units;
            process = process.intervene(corrected);
        }
    }
    process
}
