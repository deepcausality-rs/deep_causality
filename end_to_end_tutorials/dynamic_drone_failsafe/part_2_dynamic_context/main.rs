/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 2: dynamic context
//!
//! The drone of part 1 flies the same night with the same faults. Its thermal camera and LiDAR now
//! look down every second, and the controller fuses what they see into a context: one node per
//! measured quantity of every ground patch, each an uncertain value that sharpens with every
//! frame, and a clock node that says whether it is night. From that context the controller judges
//! every patch: safe, too steep, water, a person, or unsure. The fail-safe ladder is still the one
//! of part 1, and so the drone lands where part 1 landed it.

mod constants;
mod model;
mod model_config;
mod model_context;
mod utils_print;

use crate::model_context::{FailsafeProcess, FailsafeState, Frame, LandNow};
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
        Some(model_config::ground_context()?),
        EffectLog::new(),
    );

    utils_print::print_intro();
    let mut last = None;
    let mut land_now = None;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let frame = Frame {
            telemetry,
            readings: drone.scan(&terrain)?,
        };
        let time_s = drone.time_s();
        process = process
            .bind(|_, state, ctx| model::sense(frame, state, ctx))
            .bind(model::perceive)
            .bind(model::judge)
            .bind(|value, state, ctx| model::detect(value, state, ctx, &detectors))
            .bind(|value, state, ctx| model::decide(value, state, ctx, time_s));
        if let Some(err) = process.error() {
            return Err(Box::new(err.clone()));
        }
        let command = *process
            .value()
            .ok_or("the fail-safe stage produced no command")?;
        let below = process.state().ground.get(&drone.patch_below()).copied();
        let now = (process.state().faults, command, below);
        if last != Some(now) {
            utils_print::print_second(&drone, &telemetry, &now.0, below, command);
            last = Some(now);
        }
        if command == Command::LandNow && land_now.is_none() {
            land_now = Some(LandNow {
                time_s,
                position: drone.position(),
                patch: drone.patch_below(),
                below,
            });
        }
        drone.step(command, &terrain);
    }

    utils_print::print_touchdown(drone.time_s(), &Touchdown::of(&terrain, &drone));
    let context = process
        .context()
        .as_ref()
        .ok_or("the process lost its context")?;
    utils_print::print_map(process.state(), land_now.as_ref(), drone.patch_below());
    let (water, still_water) = model::daytime_reading(context, process.state())?;
    utils_print::print_daytime(water, still_water);
    utils_print::print_closing(land_now.as_ref(), drone.time_s(), drone.position());
    utils_print::print_log(process.logs());
    Ok(())
}
