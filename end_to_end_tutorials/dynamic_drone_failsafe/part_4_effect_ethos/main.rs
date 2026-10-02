/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Dynamic drone fail-safe, part 4: Effect Ethos
//!
//! The drone of part 3 flies the same night with the same faults and the same ground context, now
//! under a safety protocol: the contingency and emergency procedures published for drone pilots,
//! encoded as norms of the Effect Ethos. A lost fix or link with a healthy battery is a
//! contingency: hold, wait for recovery, then land as soon as practicable. A critical battery is
//! an emergency: land as soon as possible, and ditch the drone if no landing is permitted. Each
//! round, the fail-safe machine puts every candidate to the Ethos and flies the cheapest permitted
//! one. People first, the drone last: only as a last resort do the person bans yield, to costs.

mod constants;
mod model;
mod model_config;
mod model_context;
mod utils_print;

use crate::model_context::{
    Candidate, FailsafeProcess, FailsafeState, Frame, Ground, Maneuver, Proposal, Urgency,
};
use deep_causality::{CausalEffect, EffectLog};
use dynamic_drone_failsafe::{Drone, FLIGHT_LIMIT_S, Terrain, Touchdown};
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
    let mut emergency = None;
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
            .bind(|value, state, ctx| model::assess(value, state, ctx, time_s))
            .bind(|value, state, ctx| model::act(value, state, ctx, &machine, &ethos, time_s));
        if let Some(err) = process.error() {
            return Err(Box::new(err.clone()));
        }
        let maneuver = *process
            .value()
            .ok_or("the fail-safe machine produced no maneuver")?;
        let state = process.state();
        let below = state.ground.get(&drone.patch_below()).copied();
        let now = (state.faults, state.urgency, below, maneuver);
        if last != Some(now) {
            utils_print::print_second(&drone, &now.0, below, state.urgency, maneuver);
            last = Some(now);
        }
        if emergency.is_none() && state.urgency >= Urgency::Possible {
            let context = process
                .context()
                .clone()
                .ok_or("the process lost its context")?;
            emergency = Some((time_s, drone.position(), context, state.clone()));
        }
        model::steer(&mut drone, &terrain, maneuver, state.urgency);
    }

    utils_print::print_touchdown(drone.time_s(), &Touchdown::of(&terrain, &drone));
    let state = process.state();
    utils_print::print_map(state, drone.patch_below());
    let target = state.plan.map(|p| p.target.centre);
    utils_print::print_closing(
        state,
        target.and_then(|centre| model::nearest_person_seen_m(state, centre)),
    );
    if let Some((time_s, at, context, state)) = &emergency {
        let person = model::nearest_judged(state, Ground::Person, *at).map(|p| p.1);
        let mut rulings = Vec::new();
        for (kind, near) in [
            (Ground::Safe, person.unwrap_or(*at)),
            (Ground::Steep, *at),
            (Ground::Water, *at),
        ] {
            if let Some((patch, centre)) = model::nearest_judged(state, kind, near) {
                let candidate = Candidate {
                    proposal: Proposal::Ditch,
                    patch,
                    centre,
                };
                rulings.push(model::ruling(
                    &ethos,
                    context,
                    kind,
                    candidate,
                    Urgency::LastResort,
                )?);
            }
        }
        utils_print::print_rulings(*time_s, &rulings);
    }
    utils_print::print_log(process.logs());
    Ok(())
}
