/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fail-safe controller, as three stages of one causal process: read the telemetry, detect
//! the faults with a collection of causaloids, and decide the fail-safe on the standard ladder.

use crate::constants::*;
use crate::model_config::Detector;
use crate::model_types::{FailsafeProcess, FailsafeState, Faults};
use deep_causality::{
    AggregateLogic, CausalEffect, CausalityError, EffectLog, LogAddEntry, MonadicCausable,
    MonadicCausableCollection, PropagatingEffect,
};
use dynamic_drone_failsafe::{Command, Telemetry};

/// Stage 1: takes this second's telemetry into the process.
pub fn sense(
    telemetry: Telemetry,
    state: FailsafeState,
    ctx: Option<()>,
) -> FailsafeProcess<Telemetry> {
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={}s telemetry: {telemetry:?}",
        telemetry.time_s()
    ));
    FailsafeProcess::new(Ok(CausalEffect::value(telemetry)), state, ctx, logs)
}

/// Stage 2: the detector collection says whether anything is wrong; each detector says what. A lost
/// fix and a lost link are confirmed only once they have persisted.
pub fn detect(
    value: CausalEffect<Telemetry>,
    mut state: FailsafeState,
    ctx: Option<()>,
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
    for (slot, detector) in fired.iter_mut().zip(detectors) {
        match detector.evaluate(&input).into_parts().0 {
            Ok(effect) => *slot = effect.into_value() == Some(true),
            Err(err) => return failed(err, state, ctx),
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
    if faults != state.faults {
        logs.add_entry(&format!(
            "t={}s faults: anything wrong {anything_wrong}; {faults:?}",
            telemetry.time_s()
        ));
    }
    state.faults = faults;
    FailsafeProcess::new(Ok(CausalEffect::value(faults)), state, ctx, logs)
}

/// Stage 3: the fail-safe ladder. A critical battery, or a lost fix with a lost link, lands the
/// drone where it is; a lost fix alone holds it for the operator; a lost link alone flies it home.
/// Once an emergency has begun the fail-safe never steps back down in flight.
pub fn decide(
    value: CausalEffect<Faults>,
    mut state: FailsafeState,
    ctx: Option<()>,
    time_s: usize,
) -> FailsafeProcess<Command> {
    let Some(faults) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let (proposed, reason) = if faults.battery_critical {
        (Command::LandNow, "battery critical")
    } else if faults.gnss_lost && faults.link_lost {
        (
            Command::LandNow,
            "no fix and no link: the drone can neither navigate nor be told",
        )
    } else if faults.gnss_lost {
        (
            Command::Hold,
            "no fix, link up: hold for a fix or the operator",
        )
    } else if faults.link_lost {
        (Command::ReturnHome, "link lost, fix held: fly home")
    } else {
        (Command::Continue, "no fault")
    };
    let mut logs = EffectLog::new();
    if severity(proposed) > severity(state.failsafe) {
        logs.add_entry(&format!("t={time_s}s fail-safe: {proposed:?} ({reason})"));
        state.failsafe = proposed;
    }
    let failsafe = state.failsafe;
    FailsafeProcess::new(Ok(CausalEffect::value(failsafe)), state, ctx, logs)
}

fn severity(command: Command) -> u8 {
    match command {
        Command::Continue => 0,
        Command::Hold => 1,
        Command::ReturnHome => 2,
        Command::LandNow => 3,
    }
}

fn failed<V>(err: CausalityError, state: FailsafeState, ctx: Option<()>) -> FailsafeProcess<V> {
    FailsafeProcess::new(Err(err), state, ctx, EffectLog::new())
}
