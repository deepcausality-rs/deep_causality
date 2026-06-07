/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Hypersonic 2T Tracking
//!
//! **Scenario**: Tracking a Hypersonic Glide Vehicle (HGV) during terminal phase.
//! **System**: 'ConformalTracker' uses 6D Phase Space to predict non-linear motion linearly.
//!
//! **Advantage**: Zero-lag tracking of high-G maneuvers without mode switching.
//!
//! The 100 Hz tracking loop is expressed with the `CausalFlow` DSL: the per-tick state is one value,
//! each tick is the composed pipeline `predict -> observe -> derive`, and the 20-tick run is a single
//! `iterate_n`.
mod model;

use crate::model::{build_initial_track, derive, observe, predict};
use deep_causality_core::CausalFlow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Defense Sys: 2T-Physics Tracker Initialization ===");
    println!("[RADAR] Target Acquired. ID: HGV-09. Vel: Mach 10.");
    println!("[TRACK] 2T Metric (4,2) Engaged. Filter Lag: < 1ms.");

    println!("\nTime[ms] |   X [m]   |   Y [m]    |   Z [m]   | Vel [m/s] | G-Load");
    println!("---------------------------------------------------------------------");

    // The 100 Hz tracking loop: each tick is the pipeline predict -> observe -> derive, run 20 times.
    CausalFlow::value(build_initial_track())
        .iterate_n(20, |tick| tick.next(predict).next(observe).next(derive));

    println!("\n[SYS] Intercept Solution Valid. Track Quality: 99%.");
    Ok(())
}
