/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions and chain constructors for the flight-envelope fault analysis chain.

use crate::model_types::{
    EnvelopeContext, FlightProcess, FlightState, FloatType, OVERSPEED_KN, OVERSPEED_RISK_WEIGHT,
    STALL_KN, STALL_RISK_WEIGHT, SensorReading, Verdict, read,
};
use deep_causality_core::{CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;

/// Build the initial chain: lift the sensor reading and the envelope context
/// into a `FlightProcess`, then bind the Stage 1 sensor-collection step.
pub fn build_chain(reading: SensorReading, envelope: EnvelopeContext) -> FlightProcess<FloatType> {
    let initial: FlightProcess<SensorReading> = FlightProcess::<SensorReading>::new(
        Ok(CausalEffect::value(reading)),
        FlightState::default(),
        Some(envelope),
        EffectLog::new(),
    );
    initial.bind(collect_airspeed)
}

/// A process in the error channel that keeps the state and context it was handed.
fn stage_failure<T>(
    error: CausalityError,
    state: FlightState,
    ctx: Option<EnvelopeContext>,
) -> FlightProcess<T> {
    FlightProcess::<T>::new(Err(error), state, ctx, EffectLog::new())
}

/// Stall speed, never-exceed speed, stall risk weight and overspeed risk weight, in that order.
fn airspeed_limits(
    ctx: Option<&EnvelopeContext>,
) -> Result<(FloatType, FloatType, FloatType, FloatType), CausalityError> {
    let envelope = ctx.ok_or(CausalityError::MissingContext())?;
    Ok((
        read(envelope, STALL_KN)?,
        read(envelope, OVERSPEED_KN)?,
        read(envelope, STALL_RISK_WEIGHT)?,
        read(envelope, OVERSPEED_RISK_WEIGHT)?,
    ))
}

/// Stage 1. Sensor collection. Records the altitude estimate in `state`; the value channel
/// projects from `SensorReading` to `FloatType` airspeed.
pub fn collect_airspeed(
    value: CausalEffect<SensorReading>,
    mut state: FlightState,
    ctx: Option<EnvelopeContext>,
) -> FlightProcess<FloatType> {
    let Some(reading) = value.into_value() else {
        return stage_failure(CausalityError::ValueNotAvailable(), state, ctx);
    };
    state.estimate_altitude_ft = reading.altitude_ft;
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage1.collect: airspeed_kn={:.0} altitude_ft={:.0}",
        reading.airspeed_kn, reading.altitude_ft
    ));
    FlightProcess::<FloatType>::new(
        Ok(CausalEffect::value(reading.airspeed_kn)),
        state,
        ctx,
        logs,
    )
}

/// Stage 2. Fold the airspeed margin into `state.risk`. The stall and
/// never-exceed speeds and their risk weights are read from the envelope
/// context the process carries.
pub fn airspeed_margin(
    value: CausalEffect<FloatType>,
    mut state: FlightState,
    ctx: Option<EnvelopeContext>,
) -> FlightProcess<FloatType> {
    let Some(airspeed) = value.into_value() else {
        return stage_failure(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let (stall_kn, overspeed_kn, stall_weight, overspeed_weight) =
        match airspeed_limits(ctx.as_ref()) {
            Ok(limits) => limits,
            Err(error) => return stage_failure(error, state, ctx),
        };

    state.estimate_airspeed_kn = airspeed;
    let mut logs = EffectLog::new();

    if airspeed < stall_kn {
        let stall_severity = (stall_kn - airspeed) / stall_kn;
        state.risk += stall_weight * stall_severity;
        logs.add_entry(&format!(
            "stage2.airspeed: STALL margin {:.0} kn < stall {:.0} kn -> risk += {:.2}",
            airspeed,
            stall_kn,
            stall_weight * stall_severity
        ));
    } else if airspeed > overspeed_kn {
        let overspeed_severity = (airspeed - overspeed_kn) / overspeed_kn;
        state.risk += overspeed_weight * overspeed_severity;
        logs.add_entry(&format!(
            "stage2.airspeed: OVERSPEED {:.0} kn > Vne {:.0} kn -> risk += {:.2}",
            airspeed,
            overspeed_kn,
            overspeed_weight * overspeed_severity
        ));
    } else {
        logs.add_entry(&format!(
            "stage2.airspeed: nominal {:.0} kn within ({:.0}, {:.0}) kn",
            airspeed, stall_kn, overspeed_kn
        ));
    }

    FlightProcess::<FloatType>::new(Ok(CausalEffect::value(airspeed)), state, ctx, logs)
}

/// Stage 3. Envelope evaluation. Produces the final verdict from `state.risk`
/// against the verdict thresholds of the envelope context.
pub fn envelope_eval(
    _value: CausalEffect<FloatType>,
    state: FlightState,
    ctx: Option<EnvelopeContext>,
) -> FlightProcess<Verdict> {
    let verdict = match ctx.as_ref() {
        Some(envelope) => Verdict::from_risk(state.risk, envelope),
        None => Err(CausalityError::MissingContext()),
    };
    let verdict = match verdict {
        Ok(verdict) => verdict,
        Err(error) => return stage_failure(error, state, ctx),
    };
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage3.envelope: risk={:.3} -> verdict={:?}",
        state.risk, verdict
    ));
    FlightProcess::<Verdict>::new(Ok(CausalEffect::value(verdict)), state, ctx, logs)
}
