/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Sensor Processing as a Stateful `CausalFlow` Pipeline
//!
//! Six daisy-chained bind stages over `PropagatingProcess<_, FleetState, FleetContext>`,
//! driven through the `CausalFlow` facade:
//!
//! 1. `process_stage`      — robust per-sensor triage into `Uncertain<FloatType>`
//! 2. `validate_stage`     — fold per-sensor health counts and uncertainty into state
//! 3. `fusion_stage`       — inverse-variance fuse the temperature sensors
//! 4. `anomaly_stage`      — flag readings outside nominal bands
//! 5. `fallback_stage`     — historical-model fallback + temp/pressure physics check
//! 6. `reliability_stage`  — derive a final `RiskLevel` verdict from state
//!
//! The stages keep their `(value, state, ctx)` signatures and drop into the flow's
//! `bind` passthrough unchanged; `into_process` hands the raw process back for the
//! summary. Per-stage observability is routed through `EffectLog`; `main.rs` prints
//! the accumulated log once at the end. The plausibility and nominal bands,
//! calibration offsets, triage uncertainty factors, historical temperature
//! model, temperature–pressure correlation and thresholds are `Data<FloatType>`
//! contextoids of the fleet context and arrive through the process' `Context`
//! channel — the stages stay parameter-free.

mod model;
mod model_config;
mod model_types;
mod print_util;

use deep_causality_core::{CausalEffect, CausalFlow, EffectLog, PropagatingProcess};
use model::{
    anomaly_stage, fallback_stage, fusion_stage, process_stage, reliability_stage, validate_stage,
};
use model_config::{nominal_fleet_context, seed_readings};
use model_types::{FleetProcess, FleetState, RawReadings};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Sensor Processing — Stateful Six-Stage `CausalFlow` Pipeline");
    println!("=======================================================================\n");

    let initial: FleetProcess<RawReadings> = PropagatingProcess::new(
        Ok(CausalEffect::value(seed_readings())),
        FleetState::default(),
        Some(nominal_fleet_context()?),
        EffectLog::new(),
    );

    let final_process = CausalFlow::from(initial)
        .bind(process_stage)
        .bind(validate_stage)
        .bind(fusion_stage)
        .bind(anomaly_stage)
        .bind(fallback_stage)
        .bind(reliability_stage)
        .into_process();

    print_util::print_summary(&final_process);
    // The summary shows a failed pipeline with its error; the run then fails with it.
    match final_process.error() {
        Some(err) => Err(err.clone().into()),
        None => Ok(()),
    }
}
