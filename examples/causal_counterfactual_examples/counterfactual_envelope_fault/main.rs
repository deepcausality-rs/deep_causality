/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Counterfactual Flight-Envelope Fault Analysis
//!
//! A mid-chain intervention on a stateful `CausalFlow`. The factual chain
//! runs against a nominal sensor reading. The counterfactual takes that
//! same reading through Stage 1 (sensor collection) and then, between
//! Stage 1 and Stage 2, replaces the value channel with a stall-region
//! airspeed.
//!
//! ## Why intervene, not bind
//!
//! A `bind` chain can "inject a fault" in two ways, both of which conflate
//! changes:
//!
//! * Rewrite a stage. The model itself is now different. Any verdict
//!   difference is no longer attributable to the fault alone.
//! * Feed a different upstream input. The state accumulated by earlier
//!   stages now reflects the alternate sensor world; `FlightState` and
//!   any covariance carry residue from the other history.
//!
//! `.alternate_value(value)` swaps only the value passed to the next bind.
//! `FlightState` and the envelope context are untouched. The envelope graph
//! runs the factual aircraft against the counterfactual airspeed. That
//! is the question an operational what-if analysis actually asks.
//!
//! The envelope context holds the aircraft's stall and never-exceed speeds,
//! the risk weight of each, and the risk thresholds of the verdict.

mod model;
pub mod model_config;
pub mod model_types;
mod model_utils;

use crate::model_types::{
    EnvelopeContext, FlightProcess, FloatType, OVERSPEED_KN, STALL_KN, SensorReading, Verdict, read,
};
use deep_causality_core::CausalFlow;
use model::{airspeed_margin, build_chain, envelope_eval};
use model_config::{SERVICE_CEILING_FT, nominal_envelope_context, nominal_sensor_reading};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Counterfactual Flight-Envelope Fault Analysis ===\n");

    let reading = nominal_sensor_reading();
    let envelope = nominal_envelope_context()?;
    let stall_kn = read(&envelope, STALL_KN)?;

    println!(
        "Sensor reading: airspeed={:.0} kn, altitude={:.0} ft, attitude={:.1} deg",
        reading.airspeed_kn, reading.altitude_ft, reading.attitude_deg
    );
    println!(
        "Aircraft limits: stall={:.0} kn, Vne={:.0} kn, ceiling={:.0} ft\n",
        stall_kn,
        read(&envelope, OVERSPEED_KN)?,
        SERVICE_CEILING_FT
    );

    let factual = run_factual(reading.clone(), envelope.clone());
    let counterfactual = run_counterfactual(reading, envelope, stall_kn - 25.0);

    model_utils::print_section("Factual world", &factual);
    model_utils::print_section(
        "Counterfactual: do(airspeed = stall - 25 kn)",
        &counterfactual,
    );
    // Both worlds are printed, the failed one with its error; the run then fails with it.
    match factual.error().or(counterfactual.error()) {
        Some(err) => Err(err.clone().into()),
        None => Ok(()),
    }
}

fn run_factual(reading: SensorReading, envelope: EnvelopeContext) -> FlightProcess<Verdict> {
    CausalFlow::from(build_chain(reading, envelope))
        .bind(airspeed_margin)
        .bind(envelope_eval)
        .into_process()
}

fn run_counterfactual(
    reading: SensorReading,
    envelope: EnvelopeContext,
    airspeed_kn: FloatType,
) -> FlightProcess<Verdict> {
    CausalFlow::from(build_chain(reading, envelope))
        .alternate_value(airspeed_kn)
        .bind(airspeed_margin)
        .bind(envelope_eval)
        .into_process()
}
