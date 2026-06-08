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
//! `.intervene(value)` swaps only the value passed to the next bind.
//! `FlightState` and `AircraftConfig` are untouched. The envelope graph
//! runs the factual aircraft against the counterfactual airspeed. That
//! is the question an operational what-if analysis actually asks.

mod model;
pub mod model_config;
pub mod model_types;
mod model_utils;

use crate::model_types::{AircraftConfig, FlightProcess, SensorReading, Verdict};
use deep_causality_core::CausalFlow;
use model::{airspeed_margin, build_chain, envelope_eval};
use model_config::{nominal_aircraft_config, nominal_sensor_reading};

fn main() {
    println!("=== Counterfactual Flight-Envelope Fault Analysis ===\n");

    let reading = nominal_sensor_reading();
    let cfg = nominal_aircraft_config();

    println!(
        "Sensor reading: airspeed={:.0} kn, altitude={:.0} ft, attitude={:.1} deg",
        reading.airspeed_kn, reading.altitude_ft, reading.attitude_deg
    );
    println!(
        "AircraftConfig: stall={:.0} kn, Vne={:.0} kn, ceiling={:.0} ft\n",
        cfg.stall_kn, cfg.overspeed_kn, cfg.service_ceiling_ft
    );

    let factual = run_factual(reading.clone(), cfg.clone());
    let counterfactual = run_counterfactual(reading, cfg);

    model_utils::print_section("Factual world", &factual);
    model_utils::print_section(
        "Counterfactual: do(airspeed = stall - 25 kn)",
        &counterfactual,
    );
}

fn run_factual(reading: SensorReading, cfg: AircraftConfig) -> FlightProcess<Verdict> {
    CausalFlow::from(build_chain(reading, cfg))
        .bind(airspeed_margin)
        .bind(envelope_eval)
        .into_process()
}

fn run_counterfactual(reading: SensorReading, cfg: AircraftConfig) -> FlightProcess<Verdict> {
    let stall_kn = cfg.stall_kn;
    CausalFlow::from(build_chain(reading, cfg))
        .intervene(stall_kn - 25.0)
        .bind(airspeed_margin)
        .bind(envelope_eval)
        .into_process()
}
