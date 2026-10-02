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
use deep_causality_num::lower;
use dynamic_drone_failsafe::{
    Command, Drone, FLIGHT_HEADER, FLIGHT_LIMIT_S, Quantity, Terrain, Touchdown, TraceTable,
    flight_cells, touchdown_table, trace_dir, variant_name,
};
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

    let trace = trace_dir();
    let mut flight = TraceTable::new(
        "part_2_trace",
        &format!(
            "{FLIGHT_HEADER},gnss_degraded,fix_lost,link_lost,battery_critical,decision,below"
        ),
    );
    let mut patches = TraceTable::new("part_2_patches", "t,i,j,ground,slope_sigma_deg");

    utils_print::print_intro();
    let mut last = None;
    let mut land_now = None;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let frame = Frame {
            telemetry,
            readings: drone.scan(&terrain)?,
        };
        let in_frame: Vec<(i64, i64)> = frame.readings.iter().map(|r| r.patch()).collect();
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
        if trace.is_some() {
            let state = process.state();
            let f = state.faults;
            flight.push(format!(
                "{},{},{},{},{},{},{}",
                flight_cells(&drone),
                f.gnss_degraded,
                f.gnss_lost,
                f.link_lost,
                f.battery_critical,
                variant_name(&command),
                below.map(|g| variant_name(&g)).unwrap_or_default(),
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

    let touchdown = Touchdown::of(&terrain, &drone);
    utils_print::print_touchdown(drone.time_s(), &touchdown);
    let context = process
        .context()
        .as_ref()
        .ok_or("the process lost its context")?;
    utils_print::print_map(process.state(), land_now.as_ref(), drone.patch_below());
    let (water, still_water) = model::daytime_reading(context, process.state())?;
    utils_print::print_daytime(water, still_water);
    utils_print::print_closing(land_now.as_ref(), drone.time_s(), drone.position());
    utils_print::print_log(process.logs());
    if let Some(dir) = trace {
        let mut reading = TraceTable::new("part_2_daytime", "water_patches,still_water_by_day");
        reading.push(format!("{water},{still_water}"));
        for table in [
            flight,
            patches,
            reading,
            touchdown_table("part_2_touchdown", drone.time_s(), &touchdown),
        ] {
            table.write(&dir)?;
        }
    }
    Ok(())
}
