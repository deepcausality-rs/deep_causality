/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Chronometric GM Recovery — Public Demonstration
//!
//! Recovers Earth's geocentric gravitational constant $GM_\oplus$ from one
//! full GPS week (7 days, days 0..6) of Galileo broadcast clock and SP3
//! orbit data via the J2-corrected weak-field 1PN inversion (Bjerhammar
//! 1975, Vermeer 1983). Earth's mass is then derived as
//! $M_\oplus = GM_\oplus / G$ — *the planet weighed by clock time-dilation
//! alone.*
//!
//! The structural showcase is the `CausalMonad` bind chain visible in
//! `main()` below: five typed stages composed end-to-end through
//! [`PropagatingEffect::bind`].
//!
//! ```bash
//! cargo run -p chronometric_examples --example gm_recovery
//! ```

use chronometric_examples::data_manager::get_gnss_data_input_path;
use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_num::Float106;

use crate::display::print_gm_report;
use crate::pipeline::{
    DatasetInputs, GmReport, stage_aggregate, stage_align, stage_load, stage_pair, stage_solve_gm,
};

pub mod display;
pub mod pipeline;

/// Galileo satellite for this run. E14 is the IOV satellite with eccentric
/// orbit, providing the radial range required to invert GM.
const SAT_ID: &str = "E14";

/// One full GPS week of GBM (GFZ MGEX) datasets bundled with this example
/// (week 1877, days 0..6 — a seven-day span from 2016).
const DATASETS: &[&str] = &[
    "gbm18770", "gbm18771", "gbm18772", "gbm18773", "gbm18774", "gbm18775", "gbm18776",
];

/// Change this to `f32` for low precision  `f64` for standard precision or use `Float106` for high precision.
pub type FloatType = Float106;

fn main() {
    println!("=== Chronometric GM Recovery ===");
    println!(
        "Datasets:  {} files ({} .. {}) — one full GPS week",
        DATASETS.len(),
        DATASETS.first().unwrap_or(&""),
        DATASETS.last().unwrap_or(&"")
    );
    println!("Satellite: {SAT_ID}");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());

    let inputs = DatasetInputs {
        data_dir: get_gnss_data_input_path().to_string_lossy().into_owned(),
        datasets: DATASETS.iter().map(|s| s.to_string()).collect(),
        sat_id: SAT_ID.to_string(),
    };

    // ── The CausalMonad bind chain ────────────────────────────────────────
    //   load   ─►   align   ─►   pair   ─►   solve_gm   ─►   aggregate
    // ──────────────────────────────────────────────────────────────────────
    let result: PropagatingEffect<GmReport<FloatType>> = PropagatingEffect::pure(inputs)
        .bind(stage_load::<FloatType>)
        .bind(stage_align)
        .bind(stage_pair)
        .bind(stage_solve_gm)
        .bind(stage_aggregate);

    match result.value {
        EffectValue::Value(report) => print_gm_report(&report),
        _ => {
            eprintln!("Pipeline failed:");
            if let Some(err) = result.error {
                eprintln!("  {:?}", err.0);
            }
            std::process::exit(1);
        }
    }
}
