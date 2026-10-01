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
use deep_causality_num::lower;
use dynamic_drone_failsafe::{Command, Telemetry};

/// Stage 1: takes this second's telemetry into the process and records it in the log.
pub fn sense(
    telemetry: Telemetry,
    state: FailsafeState,
    ctx: Option<()>,
) -> FailsafeProcess<Telemetry> {
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={} s: Telemetry: {:.0} satellites, HDOP {:.1}, {:.0} % packet loss, lowest cell {:.2} V, {:.0} m above ground.",
        telemetry.time_s(),
        lower(telemetry.satellites()),
        lower(telemetry.hdop()),
        lower(telemetry.link_loss_pct()),
        lower(telemetry.min_cell_v()),
        lower(telemetry.altitude_agl_m()),
    ));
    FailsafeProcess::new(Ok(CausalEffect::value(telemetry)), state, ctx, logs)
}

/// Stage 2: the detector collection says whether anything is wrong; only then does each detector
/// say what. A lost fix and a lost link are confirmed only once they have persisted. Each fault
/// that is confirmed or clears is recorded in the log.
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

/// Stage 3: the fail-safe ladder. A critical battery, or a lost fix with a lost link, lands the
/// drone where it is; a lost fix alone holds it for the operator; a lost link alone flies it home.
/// Once an emergency has begun the fail-safe never steps back down in flight. Each step up the
/// ladder is recorded in the log with its reason.
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
