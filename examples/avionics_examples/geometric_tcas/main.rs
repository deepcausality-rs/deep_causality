/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Geometric TCAS
//!
//! **Scenario**: Two aircraft on a converging course with potential pilot incapacitation.
//! **System**: 'GeometricTCAS' monitors tracked entities, issues advisories, and executes **Autonomous Safety Interventions** via the `Intervenable` trait if cues are ignored.
//!
//! **Key Concepts**:
//! *   **Geometric Algebra**: Uses Bivector magnitude for singularity-free collision detection.
//! *   **Causal Intervention**: Demonstrates the `Intervenable` trait.
//!
//! The `Intervenable` trait is used to computationally override the causal chain (velocity vector)
//! effectively simulating an auto-pilot takeover in absence of a human reaction to the detected
//! collision.
mod model;

use crate::model::{
    AdvisoryLevel, AircraftState, GeometricTCAS, Resolution, add_vec, scale_vec, vec3,
};
use deep_causality_core::Intervenable;
use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Airbus A3 System: Geometric Collision Avoidance Module ===");
    println!("[SYS] Initializing Safety Loop...");

    let tcas = GeometricTCAS::new();

    // 1. Define Scenario: Converging head-on but slightly offset in Z
    let mut ownship = AircraftState {
        callsign: "AIRBUS_01".into(),
        pos: vec3(0.0, 0.0, 10000.0), // 10km alt
        vel: vec3(0.0, 200.0, 0.0),   // ~400 kts North
    };

    let mut intruder = AircraftState {
        callsign: "UNK_TRAFFIC".into(),
        pos: vec3(0.0, 8000.0, 10050.0), // 8km North, 50m higher
        vel: vec3(0.0, -200.0, 0.0),     // ~400 kts South (Closing speed 400m/s)
    };

    println!(
        "[SYS] Traffic Detected: {}. Monitor Active.",
        intruder.callsign
    );
    println!("\nTime[s] | Range[m] | T_CPA[s] | D_CPA[m] | ALERT STATE      | ADVISORY");
    println!("-------------------------------------------------------------------------");

    // Simulation Loop (0.5s steps)
    let dt = 0.5;
    let mut ra_duration = 0.0;

    for t in 0..30 {
        let time = t as f64 * dt;

        // A. Run Safety Logic
        let report = tcas.assess_threat(&ownship, &intruder);

        // B. Intervention Logic (Auto-Pilot) with Causal Intervention Trait
        // This demonstrates "Computational Intervention" to model the override.
        let mut sys_status = "";

        // 1. Wrap current state in Causal Effect (Option wrapper to satisfy Default)
        let vel_effect: PropagatingEffect<Option<CausalMultiVector<f64>>> =
            PropagatingEffect::pure(Some(ownship.vel.clone()));

        // 2. Determine if Intervention is needed
        let triggered = if report.advisory == AdvisoryLevel::RA {
            ra_duration += dt;
            ra_duration > 2.5 && report.resolution == Resolution::Descend
        } else {
            ra_duration = 0.0;
            false
        };

        // 3. Apply Intervention (if triggered)
        let final_effect = if triggered {
            // Calculate target velocity (Descent)
            let mut d = ownship.vel.data().clone();
            if d[4] > -20.0 {
                d[4] -= 5.0;
                if d[4] < -20.0 {
                    d[4] = -20.0;
                }

                // Construct the "Intervention Value"
                let target_vel = CausalMultiVector::unchecked(d, Metric::Euclidean(3));

                sys_status = " [\x1b[31mAUTO INTERVENE\x1b[0m]";

                // USE TRAIT: intervene()
                vel_effect.intervene(Some(target_vel))
            } else {
                sys_status = " [\x1b[32mAVOIDING\x1b[0m]";
                vel_effect // Already avoiding
            }
        } else {
            vel_effect
        };

        // 4. Unwrap & Log
        if triggered && sys_status.contains("INTERVENE") {
            // Print a custom message citing the log happened
            println!("      > [BLACKBOX AUDIT]: Automatic Intervention Recorded.");
        }

        if let EffectValue::Value(Some(v)) = final_effect.value() {
            ownship.vel = v.clone();
        }

        // C. Output
        let alert_str = match report.advisory {
            AdvisoryLevel::None => "CLEAR",
            AdvisoryLevel::TA => "\x1b[33mTRAFFIC ADVISORY\x1b[0m", // Yellow
            AdvisoryLevel::RA => "\x1b[31mRES ADVISORY\x1b[0m",     // Red
        };

        let res_str = match report.resolution {
            Resolution::Maintain => "-".to_string(),
            _ => format!("{:?}", report.resolution).to_uppercase(),
        };

        println!(
            "{:>6.1}  | {:>8.0} | {:>7.1}  | {:>7.1}  | {:<16} | {}{}",
            time,
            report.t_cpa * 400.0, // approx closing
            report.t_cpa,
            report.cpa_dist,
            alert_str,
            res_str,
            sys_status
        );

        // D. Update Dynamics (Full Vector Integration)
        ownship.pos = add_vec(&ownship.pos, &scale_vec(&ownship.vel, dt));
        intruder.pos = add_vec(&intruder.pos, &scale_vec(&intruder.vel, dt));
    }
    // Optional: mimic real-time pace
    // thread::sleep(Duration::from_millis(50));
    println!("\n[SYS] Encounter Complete. Log Saved.");
    Ok(())
}
