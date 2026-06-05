/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Sensor Processing as a Stateful `CausalFlow` Pipeline
//!
//! Six daisy-chained bind stages over `PropagatingProcess<_, FleetState, FleetConfig>`,
//! driven through the `CausalFlow` facade:
//!
//! 1. `process_stage`      — robust per-sensor triage into `Uncertain<f64>`
//! 2. `validate_stage`     — fold per-sensor health counts and uncertainty into state
//! 3. `fusion_stage`       — inverse-variance fuse the temperature sensors
//! 4. `anomaly_stage`      — flag readings outside nominal bands
//! 5. `fallback_stage`     — historical-model fallback + temp/pressure physics check
//! 6. `reliability_stage`  — derive a final `RiskLevel` verdict from state
//!
//! The stages keep their `(value, state, ctx)` signatures and drop into the flow's
//! `bind` passthrough unchanged; `into_process` hands the raw process back for the
//! summary. Per-stage observability is routed through `EffectLog`; `main.rs` prints
//! the accumulated log once at the end. Magic-number plausibility bands and
//! calibration offsets live in `FleetConfig` and arrive through the process'
//! `Context` channel — the stages stay parameter-free.

mod model;
mod model_config;
mod model_types;
mod print_util;

use deep_causality_core::{CausalFlow, EffectLog, EffectValue, PropagatingProcess};
use model::{
    anomaly_stage, fallback_stage, fusion_stage, process_stage, reliability_stage, validate_stage,
};
use model_config::{nominal_fleet_config, seed_readings};
use model_types::{FleetProcess, FleetState, RawReadings};

fn main() {
    println!("Sensor Processing — Stateful Six-Stage `CausalFlow` Pipeline");
    println!("=======================================================================\n");

    let initial: FleetProcess<RawReadings> = PropagatingProcess {
        value: EffectValue::Value(seed_readings()),
        state: FleetState::default(),
        context: Some(nominal_fleet_config()),
        error: None,
        logs: EffectLog::new(),
    };

    let final_process = CausalFlow::from(initial)
        .bind(process_stage)
        .bind(validate_stage)
        .bind(fusion_stage)
        .bind(anomaly_stage)
        .bind(fallback_stage)
        .bind(reliability_stage)
        .into_process();

    print_util::print_summary(&final_process);
}
