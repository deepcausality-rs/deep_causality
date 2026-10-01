/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fail-safe controller, as five stages of one causal process: read the telemetry and the
//! sensor frame, fuse the frame into the ground context, judge each patch from the context, detect
//! the faults with a collection of causaloids, and decide the fail-safe on the standard ladder.

use crate::constants::*;
use crate::model_config::Detector;
use crate::model_context::{
    FailsafeProcess, FailsafeState, Faults, Frame, Fusion, Ground, GroundContext, GroundNode,
};
use deep_causality::{
    AggregateLogic, CausalEffect, CausalityError, EffectLog, LogAddEntry, MonadicCausable,
    MonadicCausableCollection, PropagatingEffect,
};
use deep_causality_algebra::Real;
use deep_causality_context::{
    ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Datable,
    DiscreteTime, EuclideanSpace, RelationKind, Temporal, TimeScale, UncertainData,
};
use deep_causality_num::lower;
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool, UncertainError};
use dynamic_drone_failsafe::{Command, FLIGHT_START_HOUR, FloatType, Quantity, Telemetry};

/// Stage 1: takes this second's telemetry and sensor frame into the process and records the
/// telemetry in the log.
pub fn sense(
    frame: Frame,
    state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Frame> {
    let t = frame.telemetry;
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={} s: Telemetry: {:.0} satellites, HDOP {:.1}, {:.0} % packet loss, lowest cell {:.2} V, {:.0} m above ground.",
        t.time_s(),
        lower(t.satellites()),
        lower(t.hdop()),
        lower(t.link_loss_pct()),
        lower(t.min_cell_v()),
        lower(t.altitude_agl_m()),
    ));
    FailsafeProcess::new(Ok(CausalEffect::value(frame)), state, ctx, logs)
}

/// Stage 2: sets the context's clock and fuses the frame into it. Each patch's five quantities are
/// fused over every frame that has seen the patch, each reading weighted by its inverse variance,
/// and the context node of each quantity holds the fused value as a normal distribution.
pub fn perceive(
    value: CausalEffect<Frame>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Frame> {
    let Some(frame) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(mut context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    match fuse(&mut context, &mut state, &frame) {
        Ok(()) => FailsafeProcess::new(
            Ok(CausalEffect::value(frame)),
            state,
            Some(context),
            EffectLog::new(),
        ),
        Err(err) => failed(graph_error(err), state, Some(context)),
    }
}

/// Stage 3: judges every patch in the frame from what the context holds about it. The clock node
/// says whether it is night, which decides what a thermal contrast means: water is warmer than the
/// land at night and cooler by day. Records how much of the ground below is safe.
pub fn judge(
    value: CausalEffect<Frame>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Telemetry> {
    let Some(frame) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    let night = match is_night(&context) {
        Ok(night) => night,
        Err(err) => return failed(err, state, Some(context)),
    };
    if let Some(land_c) = land_temperature_c(&state, &frame) {
        state.land_temperature_c = Some(land_c);
    }
    let Some(land_c) = state.land_temperature_c else {
        return failed(CausalityError::ValueNotAvailable(), state, Some(context));
    };
    for reading in &frame.readings {
        match judge_patch(&context, reading.patch(), land_c, night) {
            Ok(ground) => {
                state.ground.insert(reading.patch(), ground);
            }
            Err(err) => return failed(err, state, Some(context)),
        }
    }
    let count = |g: Ground| {
        frame
            .readings
            .iter()
            .filter(|r| state.ground.get(&r.patch()) == Some(&g))
            .count()
    };
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={} s: Ground: of {} patches below, {} safe, {} too steep, {} water, {} with a person, {} unsure.",
        frame.telemetry.time_s(),
        frame.readings.len(),
        count(Ground::Safe),
        count(Ground::Steep),
        count(Ground::Water),
        count(Ground::Person),
        count(Ground::Unsure),
    ));
    FailsafeProcess::new(
        Ok(CausalEffect::value(frame.telemetry)),
        state,
        Some(context),
        logs,
    )
}

/// Stage 4: the detector collection says whether anything is wrong; only then does each detector
/// say what. A lost fix and a lost link are confirmed only once they have persisted. Each fault
/// that is confirmed or clears is recorded in the log.
pub fn detect(
    value: CausalEffect<Telemetry>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    detectors: &[Detector],
) -> FailsafeProcess<Faults> {
    let Some(telemetry) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let input = PropagatingEffect::pure(telemetry);
    let (outcome, _, _, mut logs) = detectors
        .evaluate_collection(&input, &AggregateLogic::Any, None)
        .into_parts();
    let anything_wrong = match outcome {
        Ok(effect) => effect.into_value() == Some(true),
        Err(err) => return failed(err, state, ctx),
    };

    let mut fired = [false; 4];
    if anything_wrong {
        for (slot, detector) in fired.iter_mut().zip(detectors) {
            match detector.evaluate(&input).into_parts().0 {
                Ok(effect) => *slot = effect.into_value() == Some(true),
                Err(err) => return failed(err, state, ctx),
            }
        }
    }
    let [degraded, no_fix, link, battery] = fired;
    state.no_fix_for_s = if no_fix { state.no_fix_for_s + 1 } else { 0 };
    state.link_down_for_s = if link { state.link_down_for_s + 1 } else { 0 };
    let faults = Faults {
        gnss_degraded: degraded,
        gnss_lost: state.no_fix_for_s >= GNSS_CONFIRM_S,
        link_lost: state.link_down_for_s >= LINK_CONFIRM_S,
        battery_critical: battery,
    };

    let before = state.faults;
    let changes = [
        (
            before.gnss_degraded,
            faults.gnss_degraded,
            format!(
                "Satellite fix degraded: {:.0} satellites, HDOP {:.1}.",
                lower(telemetry.satellites()),
                lower(telemetry.hdop())
            ),
            "Satellite fix no longer degraded.".to_string(),
        ),
        (
            before.gnss_lost,
            faults.gnss_lost,
            format!("Satellite fix lost: none for {GNSS_CONFIRM_S} s."),
            "Satellite fix restored.".to_string(),
        ),
        (
            before.link_lost,
            faults.link_lost,
            format!(
                "Command link lost: {:.0} % of packets dropped for {LINK_CONFIRM_S} s.",
                lower(telemetry.link_loss_pct())
            ),
            "Command link restored.".to_string(),
        ),
        (
            before.battery_critical,
            faults.battery_critical,
            format!(
                "Battery critical: one cell at {:.2} V.",
                lower(telemetry.min_cell_v())
            ),
            "Battery no longer critical.".to_string(),
        ),
    ];
    for (was, is, raised, cleared) in changes {
        if is && !was {
            logs.add_entry(&format!("t={} s: {raised}", telemetry.time_s()));
        } else if was && !is {
            logs.add_entry(&format!("t={} s: {cleared}", telemetry.time_s()));
        }
    }
    state.faults = faults;
    FailsafeProcess::new(Ok(CausalEffect::value(faults)), state, ctx, logs)
}

/// Stage 5: the fail-safe ladder of part 1, unchanged. Each step up the ladder is recorded in the
/// log with its reason.
pub fn decide(
    value: CausalEffect<Faults>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    time_s: usize,
) -> FailsafeProcess<Command> {
    let Some(faults) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let (proposed, reason) = if faults.battery_critical {
        (Command::LandNow, "land now: the battery is critical.")
    } else if faults.gnss_lost && faults.link_lost {
        (
            Command::LandNow,
            "land now: with no satellite fix and no link, the drone can neither navigate nor receive orders.",
        )
    } else if faults.gnss_lost {
        (
            Command::Hold,
            "hold: no satellite fix, but the link is up, so wait for the fix or the operator.",
        )
    } else if faults.link_lost {
        (
            Command::ReturnHome,
            "return home: the link is lost, but the fix holds.",
        )
    } else {
        (Command::Continue, "continue: no fault.")
    };
    let mut logs = EffectLog::new();
    if severity(proposed) > severity(state.failsafe) {
        logs.add_entry(&format!("t={time_s} s: Fail-safe set to {reason}"));
        state.failsafe = proposed;
    }
    let failsafe = state.failsafe;
    FailsafeProcess::new(Ok(CausalEffect::value(failsafe)), state, ctx, logs)
}

/// The patches the controller judged to be water, and how many of them a daytime reading of the
/// same context would still call water. The daytime reading takes a cool patch for water, which at
/// night is the wrong way round.
pub fn daytime_reading(
    context: &GroundContext,
    state: &FailsafeState,
) -> Result<(usize, usize), CausalityError> {
    let land_c = state
        .land_temperature_c
        .ok_or(CausalityError::ValueNotAvailable())?;
    let water: Vec<(i64, i64)> = state
        .ground
        .iter()
        .filter(|(_, g)| **g == Ground::Water)
        .map(|(patch, _)| *patch)
        .collect();
    let mut still_water = 0;
    for patch in &water {
        if judge_patch(context, *patch, land_c, false)? == Ground::Water {
            still_water += 1;
        }
    }
    Ok((water.len(), still_water))
}

/// Fuses one frame into the context and the fusion sums. A patch seen for the first time gets a
/// space node at its centre and one data node per quantity, each linked to the space node.
fn fuse(
    context: &mut GroundContext,
    state: &mut FailsafeState,
    frame: &Frame,
) -> Result<(), ContextIndexError> {
    let clock = DiscreteTime::new(CLOCK_ID, TimeScale::Second, clock_s(&frame.telemetry));
    context.update_node(
        CLOCK_ID,
        Contextoid::new(CLOCK_ID, ContextoidType::Tempoid(clock)),
    )?;
    for reading in &frame.readings {
        let patch = reading.patch();
        let first_sight = !state.fusion.contains_key(&patch);
        let fusion = state.fusion.entry(patch).or_default();
        for q in Quantity::ALL {
            let k = q as usize;
            let weight = ONE / (reading.sigma(q) * reading.sigma(q));
            fusion.precision[k] += weight;
            fusion.weighted[k] += weight * reading.value(q);
        }
        let fused = *fusion;
        if first_sight {
            let (x, y) = reading.centre();
            let space_id = node_id(patch, 0);
            let space = context.add_node(Contextoid::new(
                space_id,
                ContextoidType::Spaceoid(EuclideanSpace::new(space_id, x, y, ZERO)),
            ))?;
            for q in Quantity::ALL {
                let id = node_id(patch, q as u64 + 1);
                let data = context.add_node(data_node(id, &fused, q))?;
                context.add_edge(space, data, RelationKind::Spatial)?;
            }
        } else {
            for q in Quantity::ALL {
                let id = node_id(patch, q as u64 + 1);
                context.update_node(id, data_node(id, &fused, q))?;
            }
        }
    }
    Ok(())
}

/// A data node holding the fused value of one quantity as a normal distribution.
fn data_node(id: ContextoidId, fusion: &Fusion, q: Quantity) -> GroundNode {
    let k = q as usize;
    let mean = fusion.weighted[k] / fusion.precision[k];
    let sigma = Real::sqrt(ONE / fusion.precision[k]);
    Contextoid::new(
        id,
        ContextoidType::Datoid(UncertainData::new(id, Uncertain::normal(mean, sigma))),
    )
}

/// What the context says about one patch. A patch is safe when, with the stated confidence, it is
/// neither steep, nor a person, and the LiDAR found a surface on it. Otherwise the first hazard
/// that holds with that confidence names it, people first; a patch with none is unsure.
fn judge_patch(
    context: &GroundContext,
    patch: (i64, i64),
    land_c: FloatType,
    night: bool,
) -> Result<Ground, CausalityError> {
    let read = |q: Quantity| belief(context, patch, q);
    let steep = read(Quantity::Slope)?.greater_than(TIP_OVER_DEG);
    let no_return = read(Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    let contrast = if night {
        read(Quantity::MeanTemperature)?.greater_than(land_c + WATER_CONTRAST_C)
    } else {
        read(Quantity::MeanTemperature)?.less_than(land_c - WATER_CONTRAST_C)
    };
    let water = no_return.clone() & contrast;
    // The LiDAR measures a bump only where it has a return.
    let person = read(Quantity::HotSpot)?.greater_than(land_c + PERSON_EXCESS_C)
        | (!no_return.clone() & read(Quantity::Protrusion)?.greater_than(PERSON_BUMP_M));

    if holds(&!(steep.clone() | no_return | person.clone()))? {
        return Ok(Ground::Safe);
    }
    for (ground, hazard) in [
        (Ground::Person, person),
        (Ground::Water, water),
        (Ground::Steep, steep),
    ] {
        if holds(&hazard)? {
            return Ok(ground);
        }
    }
    Ok(Ground::Unsure)
}

/// Whether an uncertain condition holds with probability above [`CONFIDENCE`], decided by a
/// sequential test that stops drawing once the evidence settles it. The draws are seeded, so every
/// run judges alike.
fn holds(condition: &UncertainBool<FloatType>) -> Result<bool, CausalityError> {
    condition
        .probability_exceeds(
            &SampleSession::seeded(JUDGE_SEED),
            CONFIDENCE,
            CONFIDENCE,
            TEST_EPSILON,
            MAX_SAMPLES,
        )
        .map_err(uncertain_error)
}

/// The fused value of one quantity of one patch, as the context holds it.
fn belief(
    context: &GroundContext,
    patch: (i64, i64),
    q: Quantity,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let node = context
        .get_node_index_by_id(node_id(patch, q as u64 + 1))
        .and_then(|index| context.get_node(index))
        .ok_or(CausalityError::MissingContext())?;
    match node.vertex_type() {
        ContextoidType::Datoid(data) => Ok(data.get_data()),
        _ => Err(CausalityError::MissingContext()),
    }
}

/// Whether the context's clock reads night.
fn is_night(context: &GroundContext) -> Result<bool, CausalityError> {
    let node = context
        .get_node_index_by_id(CLOCK_ID)
        .and_then(|index| context.get_node(index))
        .ok_or(CausalityError::MissingContext())?;
    let ContextoidType::Tempoid(clock) = node.vertex_type() else {
        return Err(CausalityError::MissingContext());
    };
    let hour = clock.time_unit() / SECONDS_PER_HOUR % HOURS_PER_DAY;
    Ok(!(DAWN_HOUR..DUSK_HOUR).contains(&hour))
}

/// The temperature of the land in this frame: the median fused temperature of the patches the
/// LiDAR found a surface on. `None` when the frame holds no land.
fn land_temperature_c(state: &FailsafeState, frame: &Frame) -> Option<FloatType> {
    let returns = Quantity::Returns as usize;
    let temperature = Quantity::MeanTemperature as usize;
    let mut land: Vec<FloatType> = frame
        .readings
        .iter()
        .filter_map(|r| state.fusion.get(&r.patch()))
        .filter(|f| f.weighted[returns] / f.precision[returns] >= NO_RETURN_BELOW)
        .map(|f| f.weighted[temperature] / f.precision[temperature])
        .collect();
    land.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    land.get(land.len() / 2).copied()
}

/// The local time of day of a telemetry sample, in seconds since midnight.
fn clock_s(telemetry: &Telemetry) -> u64 {
    lower(FLIGHT_START_HOUR) as u64 * SECONDS_PER_HOUR + telemetry.time_s() as u64
}

/// The context identifier of a patch's node: slot 0 is its space node, slots 1 to 5 its data
/// nodes. Patch indices stay within a thousand of zero, so the identifiers never meet the clock's.
fn node_id(patch: (i64, i64), slot: u64) -> ContextoidId {
    let (i, j) = patch;
    (((i + 1000) as u64) * 10_000 + (j + 1000) as u64) * 8 + slot
}

fn severity(command: Command) -> u8 {
    match command {
        Command::Continue => 0,
        Command::Hold => 1,
        Command::ReturnHome => 2,
        Command::LandNow => 3,
    }
}

fn graph_error(err: ContextIndexError) -> CausalityError {
    CausalityError::GraphError(err.to_string())
}

fn uncertain_error(err: UncertainError) -> CausalityError {
    CausalityError::UncertainError(err.to_string())
}

fn failed<V>(
    err: CausalityError,
    state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<V> {
    FailsafeProcess::new(Err(err), state, ctx, EffectLog::new())
}
