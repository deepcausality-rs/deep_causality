/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Event Horizon Probe
//!
//! A simulation of a space probe approaching a black hole, demonstrating regime-switching
//! between Newtonian and Relativistic physics using Causal Monads.

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{Length, Mass, PhysicsError};
use deep_causality_physics::{escape_velocity, schwarzschild_radius, time_dilation_angle};

#[derive(Debug, Clone, Default)]
struct ProbeState {
    distance: f64, // Meters from singularity
    velocity: f64, // m/s (radial)
    mass: f64,     // kg
    status: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Event Horizon Probe Simulation ===\n");

    // 1. Setup: Supermassive Black Hole (Sagittarius A* approx)
    let black_hole_mass = Mass::new(4.0e6 * 1.989e30).map_err(|e: PhysicsError| e.to_string())?; // 4 million solar masses
    let rs_effect = schwarzschild_radius(&black_hole_mass);
    let r_s = rs_effect.value().clone().into_value().unwrap().value();

    println!("Target: Supermassive Black Hole");
    println!("Mass: {:.2e} kg", black_hole_mass.value());
    println!("Schwarzschild Radius (Rs): {:.2e} m\n", r_s);

    // 2. Initial State: Probe far away
    let initial_state = ProbeState {
        distance: r_s * 100.0, // 100x Rs
        velocity: 0.0,         // Starting from rest (freefall)
        mass: 1000.0,          // 1000 kg probe
        status: "Approaching".to_string(),
    };

    // 3. Causal Chain: Fall and Evaluate
    let steps = 5;
    let mut current_state = initial_state;

    for t in 0..steps {
        println!("---\nStep {} ---", t);
        let dist_ratio = current_state.distance / r_s;
        println!(
            "Distance: {:.2e} m ({:.1} Rs)",
            current_state.distance, dist_ratio
        );

        // Define the physics context based on state
        let regime_check = if dist_ratio > 10.0 {
            "Newtonian"
        } else {
            "Relativistic"
        };
        println!("Physics Regime: {}", regime_check);

        // Run Causal Logic
        let next_state_effect = CausalEffectPropagationProcess::with_state(
            CausalEffectPropagationProcess::pure(()),
            current_state.clone(),
            Some(black_hole_mass),
        )
        .bind(|_, state, ctx: Option<Mass>| {
            let bh_mass = ctx.unwrap(); // Context has BH mass
            let r = Length::new(state.distance).unwrap();

            // A. Calculate expected orbital/escape velocities (Context assessment)
            let v_esc_effect = escape_velocity(&bh_mass, &r);
            let v_esc = v_esc_effect.value().clone().into_value().unwrap().value();

            println!("  Escape Velocity required: {:.2e} m/s", v_esc);

            // B. Regime-Specific Logic
            if state.distance / r_s > 10.0 {
                // --- Newtonian Regime ---
                // Simple freefall approximation v = sqrt(2GM/r) (which is v_esc)
                // We just update velocity to match freefall speed at this radius
                let new_vel = v_esc;
                let new_dist = state.distance * 0.5; // Simulate falling

                CausalEffectPropagationProcess::pure(ProbeState {
                    distance: new_dist,
                    velocity: new_vel,
                    status: "Freefall (Newtonian)".to_string(),
                    mass: state.mass,
                })
            } else {
                // --- Relativistic Regime ---
                // Calculate Time Dilation effects
                // Metric (+---)
                let metric = Metric::Minkowski(4);

                // Probe 4-velocity (approx)
                let t_static = CausalMultiVector::new(
                    vec![
                        0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                        0.0,
                    ],
                    metric,
                )
                .unwrap();

                // Falling probe vector (gamma, gamma*v, 0, 0)
                let v_rel = 0.9;
                let gamma = 1.0f64 / (1.0f64 - v_rel * v_rel).sqrt();
                let t_probe = CausalMultiVector::new(
                    vec![
                        0.0,
                        gamma,
                        gamma * v_rel,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                        0.0,
                    ],
                    metric,
                )
                .unwrap();

                let dilation_effect = time_dilation_angle(&t_static, &t_probe);
                let rapidity = dilation_effect
                    .value()
                    .clone()
                    .into_value()
                    .unwrap()
                    .value();

                println!("  [GR] Relativistic Rapidity: {:.4}", rapidity);
                println!("  [GR] Time Dilation Factor: {:.2}", rapidity.cosh());

                // Check Horizon crossing
                if state.distance <= r_s * 1.1 {
                    CausalEffectPropagationProcess::pure(ProbeState {
                        distance: state.distance * 0.1,
                        velocity: 2.99e8, // c
                        status: "EVENT HORIZON CROSSED".to_string(),
                        mass: state.mass,
                    })
                } else {
                    CausalEffectPropagationProcess::pure(ProbeState {
                        distance: state.distance * 0.5,
                        velocity: v_esc,
                        status: "Relativistic Plunge".to_string(),
                        mass: state.mass,
                    })
                }
            }
        });

        // Update State
        if let EffectValue::Value(s) = next_state_effect.value() {
            current_state = s.clone();
            println!("  -> New Status: {}", current_state.status);
            println!("  -> Current Velocity: {:.2e} m/s", current_state.velocity);

            if current_state.status == "EVENT HORIZON CROSSED" {
                println!("\n!!! SIGNAL LOST !!! Probe has crossed the event horizon.");
                break;
            }
        }
        println!();
    }

    Ok(())
}
