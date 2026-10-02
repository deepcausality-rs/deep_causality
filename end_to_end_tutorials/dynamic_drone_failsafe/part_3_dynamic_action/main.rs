/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 3: dynamic action
//!
//! The drone of part 2 flies the same night with the same faults and the same ground context. A
//! causal state machine now turns the fail-safe ladder's command into a maneuver over the ground:
//! hold over the ground, fly home, or choose a landing patch and land on it. The drone flies these
//! by its camera and LiDAR, so it no longer drifts without a satellite fix. Each patch the machine
//! chooses replaces its landing state with a new version.

mod constants;
mod model;
mod model_config;
mod model_context;
mod utils_print;

use crate::model_context::{FailsafeProcess, FailsafeState, Frame, Maneuver};
use deep_causality::{CausalEffect, EffectLog};
use dynamic_drone_failsafe::{Drone, FLIGHT_LIMIT_S, Terrain, Touchdown};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let detectors = model_config::detectors();
    let machine = model_config::failsafe_machine();
    let terrain = Terrain::new();
    let mut drone = Drone::launch();
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Maneuver::None)),
        FailsafeState {
            landing_version: 1,
            ..FailsafeState::default()
        },
        Some(model_config::ground_context()?),
        EffectLog::new(),
    );

    utils_print::print_intro();
    let mut last = None;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let frame = Frame {
            telemetry,
            readings: drone.scan(&terrain)?,
            position: drone.position(),
        };
        let time_s = drone.time_s();
        process = process
            .bind(|_, state, ctx| model::sense(frame, state, ctx))
            .bind(model::perceive)
            .bind(model::judge)
            .bind(|value, state, ctx| model::detect(value, state, ctx, &detectors))
            .bind(|value, state, ctx| model::decide(value, state, ctx, time_s))
            .bind(|value, state, ctx| model::act(value, state, ctx, &machine, time_s));
        if let Some(err) = process.error() {
            return Err(Box::new(err.clone()));
        }
        let maneuver = *process
            .value()
            .ok_or("the fail-safe machine produced no maneuver")?;
        let state = process.state();
        let below = state.ground.get(&drone.patch_below()).copied();
        let now = (state.faults, state.failsafe, below, maneuver);
        if last != Some(now) {
            utils_print::print_second(&drone, &now.0, below, state.failsafe, maneuver);
            last = Some(now);
        }
        model::steer(&mut drone, &terrain, maneuver, state.failsafe);
    }

    utils_print::print_touchdown(drone.time_s(), &Touchdown::of(&terrain, &drone));
    let state = process.state();
    utils_print::print_map(state, drone.patch_below());
    utils_print::print_closing(
        state,
        model::nearest_person_seen_m(state, state.target.centre),
    );
    utils_print::print_log(process.logs());
    Ok(())
}
