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
use crate::model_context::{GroundContext, GroundEthos, Review};
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

    let trace = trace_dir();
    let mut flight = TraceTable::new(
        "part_4_trace",
        &format!(
            "{FLIGHT_HEADER},gnss_degraded,fix_lost,link_lost,battery_critical,decision,below,maneuver,target_i,target_j,plan_status"
        ),
    );
    let mut patches = TraceTable::new("part_4_patches", "t,i,j,ground,slope_sigma_deg");
    let mut rounds = TraceTable::new(
        "part_4_rulings",
        "t,urgency,i,j,proposal,review,harm_cost,norms",
    );

    utils_print::print_intro();
    let mut last = None;
    let mut emergency = None;
    let mut rounds_tail = Vec::new();
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let frame = Frame {
            telemetry,
            readings: drone.scan(&terrain)?,
            position: drone.position(),
        };
        let in_frame: Vec<(i64, i64)> = frame.readings.iter().map(|r| r.patch()).collect();
        let logged = process.logs().messages().count();
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
        if trace.is_some() {
            let context = process
                .context()
                .as_ref()
                .ok_or("the process lost its context")?;
            let f = state.faults;
            let (ti, tj, status) = state
                .plan
                .and_then(|p| p.target.patch.map(|(i, j)| (i, j, p.status)))
                .map(|(i, j, status)| (i.to_string(), j.to_string(), variant_name(&status)))
                .unwrap_or_default();
            flight.push(format!(
                "{},{},{},{},{},{},{},{},{ti},{tj},{status}",
                flight_cells(&drone),
                f.gnss_degraded,
                f.gnss_lost,
                f.link_lost,
                f.battery_critical,
                variant_name(&state.urgency),
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
            let messages: Vec<String> = process
                .logs()
                .messages()
                .skip(logged)
                .map(str::to_string)
                .collect();
            for message in &messages {
                record_round(&mut rounds, &ethos, context, state, time_s, message)?;
            }
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

    let touchdown = Touchdown::of(&terrain, &drone);
    utils_print::print_touchdown(drone.time_s(), &touchdown);
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
        if trace.is_some() {
            let mut last_resort =
                TraceTable::new("part_4_last_resort", "t,kind,i,j,proposal,review,harm_cost");
            for ruling in &rulings {
                let (review, cost) = review_cells(&ruling.review);
                last_resort.push(format!(
                    "{time_s},{},{},{},{},{review},{cost}",
                    variant_name(&ruling.kind),
                    ruling.candidate.patch.0,
                    ruling.candidate.patch.1,
                    variant_name(&ruling.candidate.proposal),
                ));
            }
            rounds_tail.push(last_resort);
        }
    }
    utils_print::print_log(process.logs());
    if let Some(dir) = trace {
        for table in [
            flight,
            patches,
            rounds,
            touchdown_table("part_4_touchdown", drone.time_s(), &touchdown),
        ]
        .into_iter()
        .chain(rounds_tail)
        {
            table.write(&dir)?;
        }
    }
    Ok(())
}

/// Records one round of the Effect Ethos from its log line: every candidate the machine put to the
/// Ethos that round, reviewed again against the same context under the same urgency. The round's
/// log line states how many candidates it put; a different count is an error.
fn record_round(
    rounds: &mut TraceTable,
    ethos: &GroundEthos,
    context: &GroundContext,
    state: &FailsafeState,
    time_s: usize,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let Some(rest) = message
        .split(" proposals put to the Effect Ethos under the ")
        .nth(1)
    else {
        return Ok(());
    };
    let urgency = if rest.starts_with("contingency") {
        Urgency::Practicable
    } else if rest.starts_with("emergency") {
        Urgency::Possible
    } else {
        Urgency::LastResort
    };
    let put: usize = message
        .split_whitespace()
        .nth(2)
        .ok_or("a round's log line names no count")?
        .parse()?;
    let candidates = model::candidates(state, urgency);
    if candidates.len() != put {
        return Err(format!(
            "t={time_s} s: the trace rebuilt {} candidates, the round put {put}",
            candidates.len()
        )
        .into());
    }
    for candidate in candidates {
        let review = model::review(ethos, context, candidate, urgency)?;
        let (verdict, cost) = review_cells(&review);
        let norms = match &review {
            Review::Look(norms) | Review::Rejected(norms) => norms
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(";"),
            Review::Approved(_) => String::new(),
        };
        rounds.push(format!(
            "{time_s},{},{},{},{},{verdict},{cost},{norms}",
            variant_name(&urgency),
            candidate.patch.0,
            candidate.patch.1,
            variant_name(&candidate.proposal),
        ));
    }
    Ok(())
}

/// A review's verdict and its harm cost, empty when it forbids.
fn review_cells(review: &Review) -> (String, String) {
    match review {
        Review::Approved(cost) => ("Approved".to_string(), cost.to_string()),
        other => (variant_name(other), String::new()),
    }
}
