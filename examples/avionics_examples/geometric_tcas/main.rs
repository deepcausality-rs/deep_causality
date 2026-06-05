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

use crate::model::{AdvisoryLevel, AircraftState, GeometricTCAS, Resolution, vec3};
use deep_causality_calculus::{EndoArrow, Euler};
use deep_causality_core::CausalFlow;
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

        // B. Intervention Logic (Auto-Pilot) via the CausalFlow closed-loop interlock.
        // Determine whether a Resolution Advisory has persisted long enough to force a descent.
        let triggered = if report.advisory == AdvisoryLevel::RA {
            ra_duration += dt;
            ra_duration > 2.5 && report.resolution == Resolution::Descend
        } else {
            ra_duration = 0.0;
            false
        };

        // The auto-pilot only takes over while there is still descent authority left.
        let will_intervene = triggered && ownship.vel.data()[4] > -20.0;
        let sys_status = if will_intervene {
            " [\x1b[31mAUTO INTERVENE\x1b[0m]"
        } else if triggered {
            " [\x1b[32mAVOIDING\x1b[0m]"
        } else {
            ""
        };

        // `intervene_if` substitutes the velocity with the descent vector only when the interlock
        // fires (Pearl Layer 2), recording the override in the flow's audit log.
        ownship.vel = CausalFlow::value(ownship.vel.clone())
            .intervene_if(
                |_| will_intervene,
                |vel| {
                    let mut d = vel.data().clone();
                    d[4] = (d[4] - 5.0).max(-20.0);
                    CausalMultiVector::unchecked(d, Metric::Euclidean(3))
                },
            )
            .finish()
            .expect("velocity flow always carries a value");

        if will_intervene {
            println!("      > [BLACKBOX AUDIT]: Automatic Intervention Recorded.");
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

        // D. Update Dynamics: one Euler step of the constant-velocity kinematics (the Arrow
        // calculus integration operator; exact for constant velocity, so identical to pos += v·dt).
        let own_vel = ownship.vel.clone();
        ownship.pos = Euler::new(dt, move |_: &CausalMultiVector<f64>| own_vel.clone())
            .iterate_n(ownship.pos.clone(), 1);
        let intr_vel = intruder.vel.clone();
        intruder.pos = Euler::new(dt, move |_: &CausalMultiVector<f64>| intr_vel.clone())
            .iterate_n(intruder.pos.clone(), 1);
    }
    // Optional: mimic real-time pace
    // thread::sleep(Duration::from_millis(50));
    println!("\n[SYS] Encounter Complete. Log Saved.");
    Ok(())
}
