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
use deep_causality_num::lower;
use dynamic_drone_failsafe::{
    Drone, FLIGHT_HEADER, FLIGHT_LIMIT_S, Quantity, Terrain, Touchdown, TraceTable, flight_cells,
    touchdown_table, trace_dir, variant_name,
};
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

    let trace = trace_dir();
    let mut flight = TraceTable::new(
        "part_3_trace",
        &format!(
            "{FLIGHT_HEADER},gnss_degraded,fix_lost,link_lost,battery_critical,decision,below,maneuver,target_i,target_j"
        ),
    );
    let mut patches = TraceTable::new("part_3_patches", "t,i,j,ground,slope_sigma_deg");

    utils_print::print_intro();
    let mut last = None;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let frame = Frame {
            telemetry,
            readings: drone.scan(&terrain)?,
            position: drone.position(),
        };
        let in_frame: Vec<(i64, i64)> = frame.readings.iter().map(|r| r.patch()).collect();
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
        if trace.is_some() {
            let f = state.faults;
            let (ti, tj) = state
                .target
                .patch
                .map(|(i, j)| (i.to_string(), j.to_string()))
                .unwrap_or_default();
            flight.push(format!(
                "{},{},{},{},{},{},{},{},{ti},{tj}",
                flight_cells(&drone),
                f.gnss_degraded,
                f.gnss_lost,
                f.link_lost,
                f.battery_critical,
                variant_name(&state.failsafe),
                below.map(|g| variant_name(&g)).unwrap_or_default(),
                variant_name(&maneuver),
            ));
            for patch in &in_frame {
                if let (Some(ground), Some(fusion)) =
                    (state.ground.get(patch), state.fusion.get(patch))
                {
                    let precision = lower(fusion.precision[Quantity::Slope as usize]);
                    patches.push(format!(
                        "{time_s},{},{},{},{:.3}",
                        patch.0,
                        patch.1,
                        variant_name(ground),
                        1.0 / precision.sqrt(),
                    ));
                }
            }
        }
        model::steer(&mut drone, &terrain, maneuver, state.failsafe);
    }

    let touchdown = Touchdown::of(&terrain, &drone);
    utils_print::print_touchdown(drone.time_s(), &touchdown);
    let state = process.state();
    utils_print::print_map(state, drone.patch_below());
    utils_print::print_closing(
        state,
        model::nearest_person_seen_m(state, state.target.centre),
    );
    utils_print::print_log(process.logs());
    if let Some(dir) = trace {
        for table in [
            flight,
            patches,
            touchdown_table("part_3_touchdown", drone.time_s(), &touchdown),
        ] {
            table.write(&dir)?;
        }
    }
    Ok(())
}
