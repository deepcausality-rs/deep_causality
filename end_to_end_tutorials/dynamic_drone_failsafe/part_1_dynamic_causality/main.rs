/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 1: dynamic causality
//!
//! An inspection drone flies a high-voltage line at night along a mountain slope. Its satellite
//! fix degrades and drops out, its command link goes behind the ridge, and a cell fails. A causal
//! process reasons over the telemetry each second: a collection of causaloids detects the faults,
//! the process state confirms the ones that persist, and the standard fail-safe ladder decides
//! what the drone does. The reasoning is right at every step, and the drone still comes down where
//! it cannot stay, because the controller knows nothing about the ground below it.

mod constants;
mod model;
mod model_config;
mod model_types;
mod utils_print;

use crate::model_types::{FailsafeProcess, FailsafeState};
use deep_causality::{CausalEffect, EffectLog};
use dynamic_drone_failsafe::{Command, Drone, FLIGHT_LIMIT_S, Terrain, Touchdown};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let detectors = model_config::detectors();
    let terrain = Terrain::new();
    let mut drone = Drone::launch();
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Command::Continue)),
        FailsafeState::default(),
        None,
        EffectLog::new(),
    );

    utils_print::print_intro();
    let mut last = None;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let time_s = drone.time_s();
        process = process
            .bind(|_, state, ctx| model::sense(telemetry, state, ctx))
            .bind(|value, state, ctx| model::detect(value, state, ctx, &detectors))
            .bind(|value, state, ctx| model::decide(value, state, ctx, time_s));
        if let Some(err) = process.error() {
            return Err(Box::new(err.clone()));
        }
        let command = *process
            .value()
            .ok_or("the fail-safe stage produced no command")?;
        let now = (process.state().faults, command);
        if last != Some(now) {
            utils_print::print_second(&drone, &telemetry, &now.0, command);
            last = Some(now);
        }
        drone.step(command, &terrain);
    }

    utils_print::print_touchdown(drone.time_s(), &Touchdown::of(&terrain, &drone));
    utils_print::print_log(process.logs());
    Ok(())
}
