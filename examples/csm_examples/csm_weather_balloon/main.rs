/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A long-duration weather balloon keeps its lithium-ion pack inside one envelope from the
//! launch pad to 20 km and back. A Causal State Machine sets cooling, heating and charging from
//! what the sensors measure each minute, and swaps the cooling mechanism when the air thins.

mod constants;
mod model;
mod model_config;
mod utils_print;

use crate::model::{Actuators, Flight};
use crate::model_config::{Regime, STATES};
use deep_causality::PropagatingEffect;

/// The working precision of the whole example.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = model_config::controller();
    let detectors = model_config::regime_detectors();
    let mut regime = Regime::Fan;
    let mut cooling_version = 1;
    let mut flight = Flight::launch();
    let mut last: Option<Actuators> = None;

    utils_print::print_header();
    while !flight.landed() {
        let reading = flight.reading();

        // Reasoning: which cooling mechanism removes more heat in this air?
        let next = model_config::next_regime(regime, &reading, &detectors)?;
        if next != regime {
            cooling_version += 1;
            model_config::install_cooling(&controller, next, cooling_version)?;
            utils_print::print_regime_change(&flight, &reading, next, cooling_version);
            regime = next;
        }

        // Action: every state evaluates the same sensor frame and fires when its rule holds.
        let frame = PropagatingEffect::pure(reading);
        for id in STATES {
            controller.eval_single_state(id, &frame)?;
        }

        let now = Actuators::read().in_light(reading.sun_w_m2);
        if last != Some(now) {
            utils_print::print_event(&flight, &reading, &now);
            last = Some(now);
        }
        flight.step();
    }
    utils_print::print_summary(&flight);
    Ok(())
}
