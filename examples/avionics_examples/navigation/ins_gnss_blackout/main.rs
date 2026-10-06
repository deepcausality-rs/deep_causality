/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # INS / GNSS-Blackout Clock Holdover — on real Galileo data
//!
//! A general **GPS-denial** scenario: a vehicle loses GNSS for a stretch — jamming, spoofing, an urban
//! canyon, a tunnel, or terrain shadowing (temporary GPS denial is a routine hazard today) — dead-reckons
//! on its INS, holds its clock, and reacquires on the far side. Run on the **real Galileo E14** GNSS
//! products (SP3 orbit + `.clk` clock), loaded through `deep_causality_file` (the haft IO monad). It
//! composes four native mechanisms:
//!
//! 1. **`deep_causality_file`** — real GNSS data ingestion (lazy `IoAction`, run at the edge).
//! 2. **`relativistic_clock_drift_rate_kernel`** predicts the clock rate from the
//!    real orbit geometry; this is the model **carried** across the outage.
//! 3. **`select_metric` regime detector** — a GNSS-denial indicator (interference / jamming /
//!    shadowing level) vs a critical threshold flips GNSS available ↔ denied: the two regime changes.
//! 4. **`alternate_value` / `branch_with`** — the GNSS fix corrects the INS error when available and is
//!    **withheld** during the blackout; the chain runs open-loop through the dark, then snaps back.
//!    `EffectLog` records every regime change and every intervention.
//!
//! The processed epoch series, the outage window, the accelerometer bias, the denial threshold and
//! the two fix gains form the navigation world, a `deep_causality_context` `Context` carried in the
//! `Context` channel; every stage reads it there.
//!
//! Two runs side by side (the airplane-INS insight: a recalibrated INS survives a short gap; a pure
//! INS drifts away): **open loop** (no GNSS coupling) vs **closed loop** (regime-gated `alternate_value`).
//! The data cadence is GNSS-native (~5 min epochs), so the modelled outage is an *extended* GNSS gap;
//! the same holdover mechanism scales down to a brief denial and up to a long one.
//!
//! ```bash
//! cargo run -p avionics_examples --example ins_gnss_blackout
//! ```

mod model;
mod utils_print;

use crate::model::{
    ACCEL_BIAS, BIAS_CAL_GAIN, BLACKOUT_THRESHOLD, GPS_GAIN, NavContext, NavProcess, OUTAGE_END,
    OUTAGE_START, advance, apply_fix, build_stream, detect_regime, epochs, gps_fix,
    initial_process, nav_world, record_metrics,
};
use deep_causality_core::CausalFlow;
use deep_causality_file::read_gnss_single_satellite;
use deep_causality_haft::IoAction;
use std::path::PathBuf;
use std::process::exit;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

const SAT_ID: &str = "E14"; // Galileo IOV, eccentric orbit → a clean relativistic clock signature.

fn main() {
    utils_print::print_intro(SAT_ID);

    // 1. Ingest the real Galileo orbit + clock through the IO monad (performed at the edge).
    let (clk, sp3) = (data_path("gbm18770.clk"), data_path("gbm18770.sp3"));
    if !clk.exists() || !sp3.exists() {
        eprintln!("Galileo data not found at {}", clk.display());
        exit(2);
    }
    let (clocks, orbits) = match read_gnss_single_satellite::<FloatType>(&clk, &sp3, SAT_ID).run() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("GNSS load failed: {e}");
            exit(2);
        }
    };
    utils_print::print_loaded(SAT_ID, orbits.len(), clocks.len());

    // 2. Build the processed epoch series (time, step, radius, speed, measured clock, relativistic
    //    rate) and the navigation world around them, each scenario scalar keyed by its contextoid
    //    id.
    let stream = match build_stream(orbits, clocks) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("epoch processing failed: {e}");
            exit(2);
        }
    };
    let scenario = [
        (OUTAGE_START, 0.45), // blackout window as a fraction of the day's epochs
        (OUTAGE_END, 0.55),
        (ACCEL_BIAS, 1.0e-4), // accelerometer bias, m/s² (~10 µg, navigation grade)
        (BLACKOUT_THRESHOLD, 0.5),
        (GPS_GAIN, 0.9),
        (BIAS_CAL_GAIN, 0.05),
    ];
    let world = match nav_world(stream, &scenario) {
        Ok(world) => world,
        Err(e) => {
            eprintln!("navigation world rejected a contextoid: {e}");
            exit(2);
        }
    };
    let n = match epochs(&world) {
        Ok(n) if n >= 10 => n,
        Ok(_) => {
            eprintln!("insufficient epochs after processing");
            exit(2);
        }
        Err(e) => {
            eprintln!("navigation world unreadable: {e}");
            exit(2);
        }
    };
    if let Err(e) = utils_print::print_stream_summary(&world) {
        eprintln!("navigation world unreadable: {e}");
        exit(2);
    }

    // 3. Run open loop (no GNSS coupling) and closed loop (regime-gated alternate_value); 4. report + gate.
    let open = run(world.clone(), n, false);
    let closed = run(world, n, true);
    for process in [&open, &closed] {
        if let Some(e) = process.error() {
            eprintln!("navigation run failed: {e}");
            exit(2);
        }
    }
    if !utils_print::report(SAT_ID, &open, &closed) {
        exit(1);
    }
}

/// Resolve the path to a bundled Galileo data file.
fn data_path(file: &str) -> PathBuf {
    avionics_examples::paths::manifest_dir()
        .join("../physics_examples/chronometric_gm_recovery/data/gnss")
        .join(file)
}

/// Run the full stream of `n` epochs once; `closed` wires the corrective `alternate_value` loop in
/// (closed loop) or leaves it out (open loop). Pipeline: `advance → detect_regime → [branch_with:
/// apply_fix + gps_fix | dead-reckon] → record_metrics`, iterated over every epoch. Every stage
/// reads the world from the `Context` channel.
fn run(world: NavContext, n: usize, closed: bool) -> NavProcess {
    CausalFlow::from(initial_process(world))
        .iterate_n(n, |tick| {
            let stepped = tick.bind(advance).bind(detect_regime);
            let routed = if closed {
                stepped.branch_with(
                    |_v, s, _c| s.gnss_denied,
                    |denied| denied, // GNSS denied: dead-reckon; the clock is carried, no fix.
                    |avail| avail.step_mut(apply_fix).bind(gps_fix),
                )
            } else {
                stepped
            };
            routed.step_mut(record_metrics)
        })
        .into_process()
}
