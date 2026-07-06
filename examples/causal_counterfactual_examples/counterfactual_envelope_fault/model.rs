/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions and chain constructors for the flight-envelope fault analysis chain.

use crate::model_types::{
    AircraftConfig, FlightProcess, FlightState, FloatType, SensorReading, Verdict,
};
use deep_causality_core::{CausalEffect, EffectLog};
use deep_causality_haft::LogAddEntry;

/// Build the initial chain: lift the sensor reading and aircraft config
/// into a `FlightProcess`, then bind the Stage 1 sensor-collection step.
pub fn build_chain(reading: SensorReading, cfg: AircraftConfig) -> FlightProcess<FloatType> {
    let initial: FlightProcess<SensorReading> = FlightProcess::<SensorReading>::new(
        Ok(CausalEffect::value(reading)),
        FlightState::default(),
        Some(cfg),
        EffectLog::new(),
    );
    initial.bind(collect_airspeed)
}

/// Stage 1. Sensor collection. Value channel projects from `SensorReading` to `FloatType` airspeed.
pub fn collect_airspeed(
    value: CausalEffect<SensorReading>,
    state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FloatType> {
    let reading = value.into_value().unwrap_or_default();
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

/// Stage 2. Fold the airspeed margin into `state.risk`.
pub fn airspeed_margin(
    value: CausalEffect<FloatType>,
    mut state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<FloatType> {
    let airspeed = value.into_value().unwrap_or(0.0);
    let cfg = ctx.as_ref().expect("AircraftConfig required");

    state.estimate_airspeed_kn = airspeed;
    let mut logs = EffectLog::new();

    if airspeed < cfg.stall_kn {
        let stall_severity = (cfg.stall_kn - airspeed) / cfg.stall_kn;
        state.risk += 0.8 * stall_severity;
        logs.add_entry(&format!(
            "stage2.airspeed: STALL margin {:.0} kn < stall {:.0} kn -> risk += {:.2}",
            airspeed,
            cfg.stall_kn,
            0.8 * stall_severity
        ));
    } else if airspeed > cfg.overspeed_kn {
        let overspeed_severity = (airspeed - cfg.overspeed_kn) / cfg.overspeed_kn;
        state.risk += 0.5 * overspeed_severity;
        logs.add_entry(&format!(
            "stage2.airspeed: OVERSPEED {:.0} kn > Vne {:.0} kn -> risk += {:.2}",
            airspeed,
            cfg.overspeed_kn,
            0.5 * overspeed_severity
        ));
    } else {
        logs.add_entry(&format!(
            "stage2.airspeed: nominal {:.0} kn within ({:.0}, {:.0}) kn",
            airspeed, cfg.stall_kn, cfg.overspeed_kn
        ));
    }

    FlightProcess::<FloatType>::new(Ok(CausalEffect::value(airspeed)), state, ctx, logs)
}

/// Stage 3. Envelope evaluation. Produces the final verdict from `state.risk`.
pub fn envelope_eval(
    _value: CausalEffect<FloatType>,
    state: FlightState,
    ctx: Option<AircraftConfig>,
) -> FlightProcess<Verdict> {
    let verdict = Verdict::from_risk(state.risk);
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "stage3.envelope: risk={:.3} -> verdict={:?}",
        state.risk, verdict
    ));
    FlightProcess::<Verdict>::new(Ok(CausalEffect::value(verdict)), state, ctx, logs)
}
