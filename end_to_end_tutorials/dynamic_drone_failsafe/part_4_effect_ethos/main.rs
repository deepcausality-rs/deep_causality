/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 4: Effect Ethos
//!
//! The drone of part 3 flies the same night with the same faults, the same ground context and the
//! same fail-safe machine. The machine now flies only what the Effect Ethos approves. Each landing
//! it proposes goes to the Ethos, whose norms read the same context: no person within the
//! clearance, no unseen ground within it, enough battery, and a touchdown the drone survives.
//! Losing the drone is acceptable; harming a person is not, so nothing defeats the person norm.

mod constants;
mod model;
mod model_config;
mod model_types;
mod utils_print;

use crate::model_types::{
    Candidate, FailsafeProcess, FailsafeState, Frame, Ground, Maneuver, Proposal,
};
use deep_causality::{CausalEffect, EffectLog};
use dynamic_drone_failsafe::{Command, Drone, FLIGHT_LIMIT_S, Guidance, Terrain, Touchdown};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let detectors = model_config::detectors();
    let machine = model_config::failsafe_machine();
    let ethos = model_config::effect_ethos()?;
    let terrain = Terrain::new();
    let mut drone = Drone::launch();
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Maneuver::None)),
        FailsafeState {
            plan_version: 1,
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
            .bind(|value, state, ctx| model::act(value, state, ctx, &machine, &ethos, time_s));
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
        match maneuver {
            Maneuver::HoldOver { x, y } => drone.guide(Guidance::HoldOver { x, y }),
            Maneuver::LandOn { x, y } => drone.guide(Guidance::LandOn { x, y }),
            Maneuver::LookOver { x, y } => drone.guide(Guidance::HoldOver { x, y }),
            Maneuver::ReturnHome => drone.step(Command::ReturnHome),
            // Nothing approved: on land now the drone holds rather than descend where it is.
            Maneuver::None | Maneuver::ChooseTarget if state.failsafe == Command::LandNow => {
                let (x, y) = drone.position();
                drone.guide(Guidance::HoldOver { x, y })
            }
            Maneuver::None | Maneuver::ChooseTarget => drone.step(state.failsafe),
        }
    }

    let (x, y) = drone.position();
    utils_print::print_touchdown(drone.time_s(), &Touchdown::assess(&terrain, x, y));
    let state = process.state();
    utils_print::print_map(state, drone.patch_below());
    let target = state.plan.map(|p| p.target.centre);
    utils_print::print_closing(
        state,
        target.and_then(|centre| model::nearest_person_seen_m(state, centre)),
    );
    let context = process
        .context()
        .as_ref()
        .ok_or("the process lost its context")?;
    let landed_at = drone.position();
    let person = model::nearest_judged(state, Ground::Person, landed_at).map(|p| p.1);
    let mut rulings = Vec::new();
    for (proposal, kind, near) in [
        (Proposal::Land, Ground::Safe, person.unwrap_or(landed_at)),
        (Proposal::Land, Ground::Steep, landed_at),
        (Proposal::Ditch, Ground::Water, landed_at),
    ] {
        if let Some((patch, centre)) = model::nearest_judged(state, kind, near) {
            let candidate = Candidate {
                proposal,
                patch,
                centre,
            };
            rulings.push(model::ruling(&ethos, context, kind, candidate)?);
        }
    }
    utils_print::print_rulings(&rulings);
    utils_print::print_log(process.logs());
    Ok(())
}
