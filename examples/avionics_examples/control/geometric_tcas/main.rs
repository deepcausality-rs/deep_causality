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
//! fires; the 30-tick encounter is a single `iterate_n`.
mod model;

use crate::model::{assess, build_initial_engagement, integrate, intervene, output};
use deep_causality_core::CausalFlow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Airbus A3 System: Geometric Collision Avoidance Module ===");
    println!("[SYS] Initializing Safety Loop...");

    let engagement = build_initial_engagement();
    println!(
        "[SYS] Traffic Detected: {}. Monitor Active.",
        engagement.intruder.callsign
    );
    println!("\nTime[s] | Range[m] | T_CPA[s] | D_CPA[m] | ALERT STATE      | ADVISORY");
    println!("-------------------------------------------------------------------------");

    // The 30-tick safety loop: assess -> (auto-intervene only if the interlock fires) -> output ->
    // integrate. The conditional takeover is the `branch`; everything else composes with `next`.
    CausalFlow::value(engagement).iterate_n(30, |tick| {
        tick.next(assess)
            .branch(|e| e.will_intervene, |hot| hot.next(intervene), |cold| cold)
            .next(output)
            .next(integrate)
    });

    println!("\n[SYS] Encounter Complete. Log Saved.");
    Ok(())
}
