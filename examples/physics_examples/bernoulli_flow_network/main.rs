/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Bernoulli Flow Network
//!
//! Simulates a fluid pipe network with varying elevation and diameters.
//! Each pipe segment is a named stage; the segments are composed into one
//! `CausalFlow` pipeline that threads the fluid state through the value channel.

use deep_causality_core::{CausalFlow, EffectValue, PropagatingEffect, PropagatingProcess};
use deep_causality_physics::{Density, Length, PhysicsError, Pressure, Speed, bernoulli_pressure};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

/// Volumetric flow rate held constant across the network (continuity: `A1 v1 = A2 v2 = Q`).
const FLOW_RATE_VOLUMETRIC: f64 = 0.1;

#[derive(Debug, Clone, Default)]
struct FluidState {
    pressure: Pressure<FloatType>,
    velocity: Speed<FloatType>,
    height: Length<FloatType>,
    description: String,
}

/// Water at 1000 kg/m^3, the working fluid for every segment.
fn water_density() -> Density<FloatType> {
    Density::<FloatType>::new(1000.0).unwrap()
}

fn main() -> Result<(), PhysicsError> {
    println!("=== Bernoulli Flow Network Simulation ===\n");

    println!("Fluid: Water (1000 kg/m^3)");
    println!("Flow Rate: {:.2} m^3/s\n", FLOW_RATE_VOLUMETRIC);

    // 1. Initial State (Reservoir)
    let initial_state = FluidState {
        pressure: Pressure::<FloatType>::new(200_000.0)?,
        velocity: Speed::<FloatType>::new(0.0)?,
        height: Length::<FloatType>::new(10.0)?,
        description: "Reservoir".to_string(),
    };

    println!(
        "[0] {:<15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m",
        initial_state.description,
        initial_state.pressure.value(),
        initial_state.velocity.value(),
        initial_state.height.value()
    );

    // 2. The network as one CausalFlow pipeline: the reservoir state seeds the
    // value channel, then each segment binds the next fluid state.
    let process = CausalFlow::value(initial_state)
        .bind(segment_main_pipe)
        .bind(segment_venturi)
        .bind(segment_vertical_drop)
        .into_process();

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

/// Compute the Bernoulli pressure for a segment of the given diameter and outlet
/// height, given the upstream fluid state.
fn flow_segment(
    prev: &FluidState,
    new_diam: FloatType,
    new_height: Length<FloatType>,
    description: &str,
) -> FluidState {
    let new_diam = Length::<FloatType>::new(new_diam).unwrap();
    let density = water_density();

    // Velocity from continuity: A1 v1 = A2 v2 = Q.
    let area = std::f64::consts::PI * (new_diam.value() / 2.0).powi(2);
    let new_vel = Speed::<FloatType>::new(FLOW_RATE_VOLUMETRIC / area).unwrap();

    // Pressure via Bernoulli.
    let p_effect = bernoulli_pressure(
        &prev.pressure,
        &prev.velocity,
        &prev.height,
        &new_vel,
        &new_height,
        &density,
    );
    let new_pressure = p_effect.value().clone().into_value().unwrap();

    FluidState {
        pressure: new_pressure,
        velocity: new_vel,
        height: new_height,
        description: description.to_string(),
    }
}

/// Segment 1: main pipe (diameter 0.2 m, height 10 m).
fn segment_main_pipe(
    value: EffectValue<FluidState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<FluidState, (), ()> {
    let prev = value.into_value().unwrap();
    let next = flow_segment(&prev, 0.2, prev.height, "Main Pipe");
    println!(
        "[1] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m",
        "Main Pipe",
        next.pressure.value(),
        next.velocity.value(),
        next.height.value()
    );
    PropagatingEffect::pure(next)
}

/// Segment 2: Venturi constriction (diameter 0.1 m, same height).
fn segment_venturi(
    value: EffectValue<FluidState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<FluidState, (), ()> {
    let prev = value.into_value().unwrap();
    let next = flow_segment(&prev, 0.1, prev.height, "Venturi");
    println!(
        "[2] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m  (Venturi: pressure drops as velocity increases)",
        "Venturi",
        next.pressure.value(),
        next.velocity.value(),
        next.height.value()
    );
    PropagatingEffect::pure(next)
}

/// Segment 3: vertical drop to the ground outlet (diameter 0.2 m, height 0 m).
fn segment_vertical_drop(
    value: EffectValue<FluidState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<FluidState, (), ()> {
    let prev = value.into_value().unwrap();
    let new_height = Length::<FloatType>::new(0.0).unwrap();
    let next = flow_segment(&prev, 0.2, new_height, "Ground Outlet");
    println!(
        "[3] {:>15} | P={:.1} Pa | v={:.2} m/s | h={:.1} m  (Drop: potential energy -> pressure)",
        "Ground Outlet",
        next.pressure.value(),
        next.velocity.value(),
        next.height.value()
    );
    PropagatingEffect::pure(next)
}
