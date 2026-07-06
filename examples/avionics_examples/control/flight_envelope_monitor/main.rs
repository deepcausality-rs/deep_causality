/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Flight Envelope Monitor — Demo Driver
//!
//! Runs two scenarios end-to-end through a single daisy-chained
//! `PropagatingProcess` composed with five `bind` calls:
//!
//! 1. Sensor collection                 (Stage 1)
//! 2. Health fold into `state.risk`     (Stage 2.1)
//! 3. Kalman covariance update          (Stage 2.2)
//! 4. Estimate-vector write             (Stage 2.3)
//! 5. Envelope hypergraph evaluation    (Stage 3)
//!
//! The verdict is derived in this file from `final_state.risk`. Both
//! scenarios print, both exit with status zero — the failing-sensor
//! scenario surfaces its error through stdout, demonstrating that the
//! bind-chain and envelope graph short-circuit.

mod model;
pub mod model_config;
mod model_types;

use crate::model_types::SafetyVerdict;
use deep_causality::*;
use deep_causality_core::CausalFlow;
use model::{estimate_step, health_fold, kalman_step, run_envelope_graph, run_sensor_collection};
use model_config::{nominal_aircraft_config, nominal_sensor_reading, seed_estimate_for};
use model_types::{AircraftConfig, FlightProcess, FlightState, FlightStateEstimate, SensorReading};

fn main() {
    println!("=== Flight Envelope Monitor — Stateful Three-Stage Pipeline ===\n");

    let aircraft_config = nominal_aircraft_config();
    let nominal_reading = nominal_sensor_reading();
    let seed_estimate = seed_estimate_for(&nominal_reading);

    let nominal_final = run_pipeline(
        nominal_reading.clone(),
        seed_estimate.clone(),
        aircraft_config.clone(),
        false,
    );
    print_section("Nominal", &nominal_final);

    let failing_final = run_pipeline(
        nominal_reading,
        seed_estimate,
        aircraft_config.clone(),
        true,
    );
    print_section("Failing sensor", &failing_final);

    println!("=== Done ===");
}

/// Daisy-chained pipeline: one initial `PropagatingProcess`, five `bind`s.
///
/// The chain uses `PropagatingProcess` (with non-trivial `State` and
/// `Context`) instead of the stateless `PropagatingEffect`.
fn run_pipeline(
    reading: SensorReading,
    seed_estimate: FlightStateEstimate,
    config: AircraftConfig,
    failing_airspeed: bool,
) -> FlightProcess<FlightStateEstimate> {
    let initial: FlightProcess<SensorReading> = PropagatingProcess::new(
        Ok(EffectValue::Value(reading)),
        FlightState::default(),
        Some(config),
        EffectLog::new(),
    );

    // Drive the stateful chain through CausalFlow. The existing
    // `(value, state, ctx) -> FlightProcess<_>` stages drop in unchanged via the `bind`
    // passthrough; `into_process()` hands the raw process back for `print_section`.
    CausalFlow::from(initial)
        .bind(|value, state, ctx| {
            println!("\n[Step 1] Sensor health collection (5 sensors, AggregateLogic::All)");
            run_sensor_collection(value, state, ctx, failing_airspeed)
        })
        .bind(|value, state, ctx| {
            println!("[Step 2] Health fold → risk += (1.0 − joint_health)");
            health_fold(value, state, ctx, seed_estimate.clone())
        })
        .bind(|value, state, ctx| {
            println!("[Step 3] Kalman covariance update (one-iteration scalar)");
            kalman_step(value, state, ctx)
        })
        .bind(|value, state, ctx| {
            println!("[Step 4] Write estimate vector into state.estimate");
            estimate_step(value, state, ctx)
        })
        .bind(|value, state, ctx| {
            println!("[Step 5] Envelope hypergraph (BFS over 6 risk nodes)");
            run_envelope_graph(value, state, ctx)
        })
        .into_process()
}

/// Print a labelled summary of a final pipeline process.
///
/// Shows verdict (or error), the final `FlightState` fields, and the
/// accumulated `EffectLog` split onto one entry per line.
fn print_section(label: &str, process: &FlightProcess<FlightStateEstimate>) {
    println!("\n--- {label} ---");
    match process.error() {
        Some(err) => {
            println!("  result: ERROR — {err:?}");
        }
        None => {
            let verdict = SafetyVerdict::from_risk(process.state().risk);
            println!(
                "  result: verdict={:?}  (risk={:.3})",
                verdict,
                process.state().risk
            );
        }
    }
    println!("  state.estimate:   {:?}", process.state().estimate);
    println!("  state.covariance: {:?}", process.state().covariance);
    println!("  state.risk:       {:.3}", process.state().risk);
    println!("  EffectLog:");
    let log_text = format!("{:?}", process.logs());
    for line in log_text.split(',').map(|s| s.trim()) {
        if !line.is_empty() {
            println!("    {line}");
        }
    }
    println!();
}
