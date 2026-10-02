/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A lithium-ion cell charges from 20 % to 80 % while its coolant pump stops after ten minutes.
//! A Causal State Machine sets the charge current from the cell temperature each minute. In the
//! second session a causaloid detects the loss of cooling from the temperature trend, and the
//! controller replaces its charge states with a derated law while it runs.

mod constants;
mod model;
mod model_config;
mod model_types;
mod utils_print;

use crate::model::Cell;
use crate::model_config::STATES;
use crate::model_types::Summary;
use deep_causality::PropagatingEffect;
use std::error::Error;

/// The working precision of the whole example.
pub type FloatType = f64;

/// The version of the charge states after the controller replaces them.
const DERATED_VERSION: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    utils_print::print_intro();
    let fixed = charge(false)?;
    let adaptive = charge(true)?;
    utils_print::print_comparison(&fixed, &adaptive);
    Ok(())
}

/// One charge session. When `adaptive`, the controller replaces its charge states once the
/// detector finds the cooling lost.
fn charge(adaptive: bool) -> Result<Summary, Box<dyn Error>> {
    let controller = model_config::controller();
    let detector = model_config::cooling_loss_detector();
    let mut cell = Cell::plug_in();
    let mut derated = false;
    let mut last = None;

    utils_print::print_session(adaptive);
    while !cell.done() {
        let reading = cell.reading();

        // Reasoning: does the temperature trend show the cooling lost?
        if adaptive && !derated && model_config::cooling_lost_in(&detector, &reading)? {
            model_config::install_derated_law(&controller, DERATED_VERSION)?;
            derated = true;
            utils_print::print_swap(&cell, &reading, DERATED_VERSION);
        }

        // Action: every charge state evaluates the same reading; the active one sets the charger.
        let frame = PropagatingEffect::pure(reading);
        for id in STATES {
            controller.eval_single_state(id, &frame)?;
        }

        let amps = model::charger_a();
        if last != Some(amps) {
            utils_print::print_minute(&cell, amps);
            last = Some(amps);
        }
        cell.charge_one_minute(amps);
    }

    let summary = cell.summary();
    utils_print::print_summary(&summary);
    Ok(summary)
}
