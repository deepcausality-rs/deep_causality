/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Counterfactual Fast Charge: value, context and state
//!
//! A lithium-ion cell fast-charges for 30 minutes. Each minute is one causal step: the cell
//! temperature is the value handed to the next minute, the charge is the state carried forward,
//! and the cooling environment is the context every minute reads. The charger derates its current
//! with temperature and tapers it above 80 % charge.
//!
//! The run is observed for ten minutes. Three counterfactuals then each replace one channel and
//! let the remaining twenty minutes run on the replacement:
//!
//! * **Value.** What if the cell had been at 52 °C? `alternate_value` replaces the temperature.
//! * **Context.** What if the coolant pump had dropped to a quarter of its flow?
//!   `alternate_context` replaces the cooling environment.
//! * **State.** What if the cell had held 75 % charge? `alternate_state` replaces the charge.
//!
//! The same `one_minute` step runs in all four runs. Each substitution leaves the other two
//! channels as observed and appends its own marker to the audit log.

mod model;
pub mod model_config;
pub mod model_types;
mod model_utils;

use model::{one_minute, plug_in};
use model_config::{
    BRANCH_MINUTE, HIGH_CHARGE_PCT, HOT_CELL_C, RUN_MINUTES, full_flow_cooling,
    quarter_flow_cooling,
};
use model_types::Charge;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Counterfactual Fast Charge: value, context and state ===\n");

    // The first ten minutes, as observed. Every run continues from this flow for the rest.
    let at_branch = plug_in(full_flow_cooling()?)?.iterate_n(BRANCH_MINUTE, one_minute);
    let finish = |run: Charge| {
        run.iterate_n(RUN_MINUTES - BRANCH_MINUTE, one_minute)
            .into_process()
    };

    let observed = finish(at_branch.clone());
    // What if the cell had been at 52 °C after ten minutes?
    let hot_cell = finish(at_branch.clone().alternate_value(HOT_CELL_C));
    // What if the coolant pump had dropped to a quarter of its flow?
    let weak_pump = finish(at_branch.clone().alternate_context(quarter_flow_cooling()?));
    // What if the cell had held 75 % charge after ten minutes?
    let full_cell = finish(at_branch.alternate_state(HIGH_CHARGE_PCT));

    let runs = [
        ("Observed", observed),
        ("Value: the cell at 52 °C after ten minutes", hot_cell),
        (
            "Context: the coolant pump at a quarter of its flow",
            weak_pump,
        ),
        (
            "State: the cell at 75 % charge after ten minutes",
            full_cell,
        ),
    ];

    for (label, run) in &runs {
        model_utils::print_run(label, run);
    }

    // Every run is printed, a failed one with its error; the program then fails with it.
    match runs.iter().find_map(|(_, run)| run.error()) {
        Some(err) => Err(err.clone().into()),
        None => Ok(()),
    }
}
