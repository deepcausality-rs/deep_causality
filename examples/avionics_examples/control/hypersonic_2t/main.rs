/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Hypersonic 2T Tracking
//!
//! **Scenario**: Tracking a Hypersonic Glide Vehicle (HGV) during terminal phase.
//! **System**: 'ConformalTracker' uses 6D Phase Space to predict non-linear motion linearly.
//!
//! **Approach**: one linear propagation in 6D phase space follows the manoeuvre without mode
//! switching. The example has no radar measurement update, so it reports the propagated track only.
//!
//! The 100 Hz tracking loop is expressed with the `CausalFlow` DSL: the per-tick state is one value,
//! each tick is the composed pipeline `predict -> observe -> derive`, and the 20-tick run is a single
//! `iterate_n`. The radar world (initial fix, initial velocity, update period) is a
//! `deep_causality_context` `Context`: the tracker is built from it, and it rides in the flow's
//! `Context` channel for the stages that read the update period.
mod model;

use crate::model::{build_initial_track, build_radar_world, derive, observe, predict};
use deep_causality_core::CausalFlow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Defense Sys: 2T-Physics Tracker Initialization ===");
    println!("[RADAR] Target Acquired. ID: HGV-09. Vel: Mach 10.");
    println!("[TRACK] 2T Metric (4,2) Engaged.");

    println!("\nTime[ms] |   X [m]   |   Y [m]    |   Z [m]   | Vel [m/s] | G-Load");
    println!("---------------------------------------------------------------------");

    // The 100 Hz tracking loop: each tick is the pipeline predict -> observe -> derive, run 20 times.
    let world = build_radar_world()?;
    CausalFlow::value(build_initial_track(&world)?)
        .context(world)
        .iterate_n(20, |tick| {
            tick.try_step_with(|t, _, ctx| predict(t, ctx))
                .map(observe)
                .try_step_with(|t, _, ctx| derive(t, ctx))
        })
        .finish()?;

    println!("\n[SYS] Track propagated over 20 ticks.");
    Ok(())
}
