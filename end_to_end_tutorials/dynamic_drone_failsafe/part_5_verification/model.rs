/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The campaign: every scenario flown by every controller, and the bound on each rate it counts.
//! Each flight wires the controller's stages the way that part's own `main` does.

use crate::constants::*;
use crate::model_config::scenario;
use crate::model_types::{Controller, Ending, Flight, Flown, Scenario};
use crate::{part_1, part_3, part_4};
use deep_causality::{CausalEffect, EffectLog};
use deep_causality_algebra::Real;
use deep_causality_num::lift_usize;
use dynamic_drone_failsafe::{Command, Drone, FLIGHT_LIMIT_S, FloatType, Touchdown};

/// Flies scenarios `0..count` with every controller, spread over the machine's cores, in scenario
/// order. Each scenario is a pure function of its number, so the result does not depend on how the
/// work is spread.
pub fn campaign(count: usize) -> Result<Vec<Flown>, String> {
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut flown = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|worker| {
                scope.spawn(move || {
                    (worker..count)
                        .step_by(workers)
                        .map(fly_scenario)
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "a campaign worker panicked".to_string())?
            })
            .collect::<Result<Vec<_>, String>>()
    })?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    flown.sort_by_key(|(s, _)| s.id);
    Ok(flown)
}

/// Scenario `id` flown by every controller.
pub fn fly_scenario(id: usize) -> Result<Flown, String> {
    let scenario = scenario(id).map_err(|e| e.to_string())?;
    let flights = Controller::ALL
        .iter()
        .map(|c| fly(*c, &scenario))
        .collect::<Result<Vec<_>, String>>()?;
    Ok((scenario, flights))
}

/// One scenario flown by one controller.
pub fn fly(controller: Controller, scenario: &Scenario) -> Result<Flight, String> {
    match controller {
        Controller::Textbook => fly_part_1(scenario),
        Controller::ContextAndAction => fly_part_3(scenario),
        Controller::EffectEthos => fly_part_4(scenario),
    }
}

/// The one-sided upper confidence bound, at [`CONFIDENCE`], on a rate seen `k` times in `n`
/// trials: the rate at which `k` or fewer occurrences would be no likelier than one minus the
/// confidence (Clopper and Pearson, 1934). Zero in a thousand bounds the rate at 0.3 %.
pub fn upper_bound(k: usize, n: usize) -> FloatType {
    if k >= n {
        return ONE;
    }
    let alpha = ONE - CONFIDENCE;
    let (mut low, mut high) = (lift_usize::<FloatType>(k) / lift_usize::<FloatType>(n), ONE);
    for _ in 0..60 {
        let mid = (low + high) / (ONE + ONE);
        if at_most(k, n, mid) > alpha {
            low = mid;
        } else {
            high = mid;
        }
    }
    high
}

/// The probability of `k` or fewer occurrences in `n` trials at rate `p`, summed in logarithms.
fn at_most(k: usize, n: usize, p: FloatType) -> FloatType {
    let (ln_p, ln_q) = (Real::ln(p), Real::ln(ONE - p));
    let mut ln_choose = ZERO;
    let mut total = ZERO;
    for i in 0..=k {
        if i > 0 {
            ln_choose +=
                Real::ln(lift_usize::<FloatType>(n - i + 1)) - Real::ln(lift_usize::<FloatType>(i));
        }
        total += Real::exp(
            ln_choose + lift_usize::<FloatType>(i) * ln_p + lift_usize::<FloatType>(n - i) * ln_q,
        );
    }
    total
}

/// The textbook fail-safe of part 1.
fn fly_part_1(scenario: &Scenario) -> Result<Flight, String> {
    use part_1::model;
    use part_1::model_types::{FailsafeProcess, FailsafeState};
    let detectors = part_1::model_config::detectors();
    let mut drone = Drone::launch_on(scenario.mission);
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Command::Continue)),
        FailsafeState::default(),
        None,
        EffectLog::new(),
    );
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let telemetry = drone.telemetry();
        let time_s = drone.time_s();
        process = process
            .bind(|_, state, ctx| model::sense(telemetry, state, ctx))
            .bind(|value, state, ctx| model::detect(value, state, ctx, &detectors))
            .bind(|value, state, ctx| model::decide(value, state, ctx, time_s));
        if let Some(err) = process.error() {
            return Err(err.to_string());
        }
        let command = *process.value().ok_or("part 1 produced no command")?;
        drone.step(command, &scenario.terrain);
    }
    Ok(flight(scenario, &drone, process.logs()))
}

/// Part 3: the ground as context and a fail-safe machine that acts on it.
fn fly_part_3(scenario: &Scenario) -> Result<Flight, String> {
    use part_3::model;
    use part_3::model_context::{FailsafeProcess, FailsafeState, Frame, Maneuver};
    let detectors = part_3::model_config::detectors();
    let machine = part_3::model_config::failsafe_machine();
    let context = part_3::model_config::ground_context().map_err(|e| e.to_string())?;
    let mut drone = Drone::launch_on(scenario.mission);
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Maneuver::None)),
        FailsafeState {
            landing_version: 1,
            ..FailsafeState::default()
        },
        Some(context),
        EffectLog::new(),
    );
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let frame = Frame {
            telemetry: drone.telemetry(),
            readings: drone.scan(&scenario.terrain).map_err(|e| e.to_string())?,
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
            return Err(err.to_string());
        }
        let maneuver = *process.value().ok_or("part 3 produced no maneuver")?;
        model::steer(
            &mut drone,
            &scenario.terrain,
            maneuver,
            process.state().failsafe,
        );
    }
    Ok(flight(scenario, &drone, process.logs()))
}

/// Part 4: part 3 with the Effect Ethos approving every landing.
fn fly_part_4(scenario: &Scenario) -> Result<Flight, String> {
    use part_4::model;
    use part_4::model_context::{FailsafeProcess, FailsafeState, Frame, Maneuver, Urgency};
    let detectors = part_4::model_config::detectors();
    let machine = part_4::model_config::failsafe_machine();
    let ethos = part_4::model_config::effect_ethos().map_err(|e| e.to_string())?;
    let context = part_4::model_config::ground_context().map_err(|e| e.to_string())?;
    let mut drone = Drone::launch_on(scenario.mission);
    let mut process = FailsafeProcess::new(
        Ok(CausalEffect::value(Maneuver::None)),
        FailsafeState {
            plan_version: 1,
            ..FailsafeState::default()
        },
        Some(context),
        EffectLog::new(),
    );
    let mut contingency = false;
    while !drone.landed() && drone.time_s() < FLIGHT_LIMIT_S {
        let frame = Frame {
            telemetry: drone.telemetry(),
            readings: drone.scan(&scenario.terrain).map_err(|e| e.to_string())?,
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
            return Err(err.to_string());
        }
        let maneuver = *process.value().ok_or("part 4 produced no maneuver")?;
        let urgency = process.state().urgency;
        contingency |= urgency != Urgency::Routine;
        // Back to routine after a contingency, with no landing under way and no fault still to
        // come: the mission resumes.
        let fault_ahead = scenario
            .mission
            .faults()
            .battery_fault_at_s()
            .is_some_and(|at| at > time_s);
        if contingency && urgency == Urgency::Routine && maneuver == Maneuver::None && !fault_ahead
        {
            return Ok(resumed(&drone, process.logs()));
        }
        model::steer(&mut drone, &scenario.terrain, maneuver, urgency);
    }
    Ok(flight(scenario, &drone, process.logs()))
}

/// How the flight ended, judged on the scenario's ground, with the controller's decisions from its
/// log.
fn flight(scenario: &Scenario, drone: &Drone, log: &EffectLog) -> Flight {
    let touchdown = Touchdown::of(&scenario.terrain, drone);
    Flight {
        ending: Ending::of(touchdown.outcome()),
        time_s: drone.time_s(),
        nearest_person_m: touchdown.nearest_person_m(),
        log: decisions(log),
    }
}

/// A flight that resumed its mission after the fix and the link came back, with no one at risk.
fn resumed(drone: &Drone, log: &EffectLog) -> Flight {
    Flight {
        ending: Ending::Resumed,
        time_s: drone.time_s(),
        nearest_person_m: NO_ONE_NEARBY_M,
        log: decisions(log),
    }
}

/// The controller's decisions from its log: faults, fail-safes, proposals and maneuvers, without
/// the per-second telemetry and ground judgements.
fn decisions(log: &EffectLog) -> Vec<String> {
    log.messages()
        .filter(|m| m.starts_with("t=") && !m.contains(": Telemetry:") && !m.contains(": Ground:"))
        .map(str::to_string)
        .collect()
}
