/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Geometric TCAS
//!
//! **Scenario**: Two aircraft on a converging course with potential pilot incapacitation.
//! **System**: 'GeometricTCAS' monitors tracked entities, issues advisories, and executes **Autonomous Safety Interventions** via counterfactual value substitution if cues are ignored.
//!
//! **Key Concepts**:
//! *   **Geometric Algebra**: Uses Bivector magnitude for singularity-free collision detection.
//! *   **Causal Intervention**: Demonstrates counterfactual value substitution.
//!
//! Each tick of the safety loop is one `CausalFlow`: `assess -> intervene? -> output -> integrate`.
//! The auto-pilot takeover is a `branch` on the value, so the override runs only when the interlock
//! fires; the 30-tick encounter is a single `iterate_n`. The TCAS thresholds and the tick period
//! are a `deep_causality_context` `Context` in the flow's `Context` channel; the aircraft tracks
//! are the value.
mod model;

use crate::model::{
    assess, build_initial_engagement, build_tcas_world, integrate, intervene, output,
};
use deep_causality_core::CausalFlow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Airbus A3 System: Geometric Collision Avoidance Module ===");
    println!("[SYS] Initializing Safety Loop...");

    let world = build_tcas_world()?;
    let engagement = build_initial_engagement();
    println!(
        "[SYS] Traffic Detected: {}. Monitor Active.",
        engagement.intruder.callsign
    );
    println!("\nTime[s] | Range[m] | T_CPA[s] | D_CPA[m] | ALERT STATE      | ADVISORY");
    println!("-------------------------------------------------------------------------");

    // The 30-tick safety loop: assess -> (auto-intervene only if the interlock fires) -> output ->
    // integrate. The conditional takeover is the `branch`; the stages that read the encounter world
    // compose with `try_step_with`, which hands them the `Context` channel.
    CausalFlow::value(engagement)
        .context(world)
        .iterate_n(30, |tick| {
            tick.try_step_with(|e, _, ctx| assess(e, ctx))
                .branch(|e| e.will_intervene, |hot| hot.map(intervene), |cold| cold)
                .try_step_with(|e, _, ctx| output(e, ctx))
                .try_step_with(|e, _, ctx| integrate(e, ctx))
        })
        .finish()?;

    println!("\n[SYS] Encounter Complete. Log Saved.");
    Ok(())
}
