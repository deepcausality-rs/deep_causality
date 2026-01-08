/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Bernoulli Flow Network
//!
//! Simulates a fluid pipe network with varying elevation and diameters.
//! Calculates pressure distribution using Causal Monads to chain fluid states.

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_physics::{Density, Length, PhysicsError, Pressure, Speed, bernoulli_pressure};

#[derive(Debug, Clone, Default)]
struct FluidState {
    pressure: Pressure,
    velocity: Speed,
    height: Length,
    description: String,
}

fn main() -> Result<(), PhysicsError> {
    println!("=== Bernoulli Flow Network Simulation ===\n");

    let density = Density::new(1000.0)?;
    let flow_rate_volumetric = 0.1;

    println!("Fluid: Water (1000 kg/m^3)");
    println!("Flow Rate: {:.2} m^3/s\n", flow_rate_volumetric);

    // 1. Initial State (Reservoir)
    let initial_state = FluidState {
        pressure: Pressure::new(200_000.0)?,
        velocity: Speed::new(0.0)?,
        height: Length::new(10.0)?,
        description: "Reservoir".to_string(),
    };

    println!(
        "[0] {:<15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m",
        initial_state.description,
        initial_state.pressure.value(),
        initial_state.velocity.value(),
        initial_state.height.value()
    );

    // 2. Define Network Segments
    // Step 1: Main Pipe (Diameter 0.2m, Height 10m)
    // Step 2: Venturi Constriction (Diameter 0.1m, Height 10m)
    // Step 3: Vertical Drop (Diameter 0.2m, Height 0m)

    // Build Causal Chain
    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(()),
        initial_state.clone(),
        Some(density),
    )
    .bind(|_, state, _| {
        // --- Segment 1: Main Pipe ---
        // First bind uses `state` (initial state) since value is ()
        let new_diam = Length::new(0.2).unwrap();
        let new_height = state.height;

        // Calculate velocity from continuity: A1v1 = A2v2 = Q
        let area = std::f64::consts::PI * (new_diam.value() / 2.0).powi(2);
        let new_vel_val = flow_rate_volumetric / area;
        let new_vel = Speed::new(new_vel_val).unwrap();

        // Calculate Pressure via Bernoulli
        let p_effect = bernoulli_pressure(
            &state.pressure,
            &state.velocity,
            &state.height,
            &new_vel,
            &new_height,
            &density,
        );

        let new_pressure = p_effect.value().clone().into_value().unwrap();

        println!(
            "[1] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m",
            "Main Pipe",
            new_pressure.value(),
            new_vel.value(),
            new_height.value()
        );

        let next_state = FluidState {
            pressure: new_pressure,
            velocity: new_vel,
            height: new_height,
            // diameter: new_diam,
            description: "Main Pipe".to_string(),
        };

        CausalEffectPropagationProcess::pure(next_state)
    })
    .bind(|prev_state, _, _| {
        // --- Segment 2: Venturi Constriction ---
        // prev_state is FluidState from Segment 1
        let prev = prev_state.into_value().unwrap();
        let new_diam = Length::new(0.1).unwrap(); // Constriction
        let new_height = prev.height;

        let area = std::f64::consts::PI * (new_diam.value() / 2.0).powi(2);
        let new_vel_val = flow_rate_volumetric / area;
        let new_vel = Speed::new(new_vel_val).unwrap();

        let p_effect = bernoulli_pressure(
            &prev.pressure,
            &prev.velocity,
            &prev.height,
            &new_vel,
            &new_height,
            &density,
        );
        let new_pressure = p_effect.value().clone().into_value().unwrap();

        println!(
            "[2] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m  (Venturi: pressure drops as velocity increases)",
            "Venturi",
            new_pressure.value(),
            new_vel.value(),
            new_height.value()
        );

        let next_state = FluidState {
            pressure: new_pressure,
            velocity: new_vel,
            height: new_height,
            description: "Venturi".to_string(),
        };

        CausalEffectPropagationProcess::pure(next_state)
    })
    .bind(|prev_state, _, _| {
        // --- Segment 3: Vertical Drop ---
        // prev_state is FluidState from Segment 2
        let prev = prev_state.into_value().unwrap();
        let new_diam = Length::new(0.2).unwrap(); // Back to main size
        let new_height = Length::new(0.0).unwrap(); // Drop to ground

        let area = std::f64::consts::PI * (new_diam.value() / 2.0).powi(2);
        let new_vel_val = flow_rate_volumetric / area;
        let new_vel = Speed::new(new_vel_val).unwrap();

        // Use Bernoulli for the drop as well (captures potential energy conversion)
        let p_effect = bernoulli_pressure(
            &prev.pressure,
            &prev.velocity,
            &prev.height,
            &new_vel,
            &new_height,
            &density,
        );
        let new_pressure = p_effect.value().clone().into_value().unwrap();

        println!(
            "[3] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m  (Drop: potential energy -> pressure)",
            "Ground Outlet",
            new_pressure.value(),
            new_vel.value(),
            new_height.value()
        );

        let next_state = FluidState {
            pressure: new_pressure,
            velocity: new_vel,
            height: new_height,
            description: "Ground Outlet".to_string(),
        };

        CausalEffectPropagationProcess::pure(next_state)
    });

    // Print summary
    println!("\n=== Simulation Complete ===");
    if let EffectValue::Value(final_state) = process.value() {
        println!(
            "Final State: {} at P={:.1} Pa, v={:.2} m/s, h={:.1} m",
            final_state.description,
            final_state.pressure.value(),
            final_state.velocity.value(),
            final_state.height.value()
        );
    }

    Ok(())
}
