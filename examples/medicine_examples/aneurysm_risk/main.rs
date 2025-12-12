/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Vascular Hemodynamics & Aneurysm Rupture Risk
//!
//! Demonstrates using Fluid Mechanics and Causal Fatigue Accumulation to predict
//! aneurysm rupture risk.
//!
//! ## Key Concepts
//! - **Digital Twin**: Simulates a vessel segment from CTA/MRA data.
//! - **Fluid Dynamics**: Calculates velocity gradients and shear stress.
//! - **Material Fatigue**: Causally accumulates damage when Wall Shear Stress (WSS) is high.

use deep_causality_core::{CausalityError, CausalityErrorEnum, EffectValue, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Constants (simplified for demo)
const BLOOD_VISCOSITY: f64 = 0.0035; // Pa.s
const RUPTURE_THRESHOLD: f64 = 0.75; // Arbitrary fatigue threshold
const CRITICAL_WSS: f64 = 15.0; // Pascal (high stress)

#[derive(Debug, Clone, Default)]
pub struct VesselState {
    pub positions: Vec<Vec<f64>>,
    pub velocities: Vec<Vec<f64>>,
    pub pressures: Vec<f64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Vascular Hemodynamics: Aneurysm Risk Analysis ===\n");

    // 1. Digital Twin Construction (Mock Data mimicking a vessel segment)
    // We create a simple cylinder with a bulge (aneurysm)
    let num_nodes = 50;
    let (vessel_manifold, initial_state) = build_mock_aneurysm(num_nodes)?;

    println!("Constructed Vessel Manifold with {} nodes.", num_nodes);
    println!("Simulating blood flow cycles...\n");

    // 2. Fatigue Accumulation
    // We simulate 10 cardiac cycles
    let initial_fatigue = 0.0;
    let mut current_fatigue = initial_fatigue;

    // We simulate time steps explicitly, feeding the state forward
    for t in 1..=10 {
        // A. Physics Update: Calculate Wall Shear Stress (WSS) from current flow
        let wss_map: Vec<f64> =
            calculate_wall_shear_stress(&vessel_manifold, &initial_state.velocities);
        let max_wss = wss_map.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        println!("[Cycle {:>2}] Max Wall Shear Stress: {:.2} Pa", t, max_wss);

        // B. Causal Risk Analysis Monad
        // We use PropagatingEffect to handle the logic and potential failure
        let effect = PropagatingEffect::pure(current_fatigue).bind(|val_ref, _, _| {
            let old_fatigue = match val_ref {
                EffectValue::Value(v) => v,
                _ => 0.0,
            };

            let mut new_fatigue = old_fatigue;

            if max_wss > CRITICAL_WSS {
                // Non-linear damage accumulation
                let damage = 0.1 * (max_wss / CRITICAL_WSS);
                new_fatigue += damage;

                // Check for rupture event -> maps to Error state
                if new_fatigue > RUPTURE_THRESHOLD {
                    return PropagatingEffect::from_error(CausalityError::new(
                        CausalityErrorEnum::Custom(format!(
                            "ANEURYSM RUPTURE DETECTED (Fatigue: {:.2})",
                            new_fatigue
                        )),
                    ));
                }
            } else {
                // Natural healing/repair (slow)
                new_fatigue = (new_fatigue - 0.01).max(0.0);
            }

            PropagatingEffect::pure(new_fatigue)
        });

        // Handle result
        match effect.value() {
            EffectValue::Value(f) => {
                current_fatigue = *f;
                // Clamp for display
                let disp = current_fatigue.clamp(0.0, 1.0);
                println!("           Accumulated Wall Fatigue: {:.1}%", disp * 100.0);
            }
            _ => {
                // If it's not Value, it might be failure or None.
                // Check error
                if let Some(err) = &effect.error {
                    println!("           [CRITICAL] {}", err);
                    println!("           !!! EMERGENCY INTERVENTION REQUIRED !!!");
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Physics Kernel: Approximates Wall Shear Stress using Manifold derivatives
fn calculate_wall_shear_stress(_manifold: &Manifold<f64>, velocities: &[Vec<f64>]) -> Vec<f64> {
    velocities
        .iter()
        .map(|v| {
            let speed = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
            // Fake relationship: Higher speed near wall ~ Higher Shear
            // We add some simulated "turbulence" noise
            let noise = (v[0] * 10.0).sin().abs();
            (speed * 5.0 * BLOOD_VISCOSITY * 1000.0) + noise // Scale to Pascals
        })
        .collect()
}

/// Helper to build a mock vessel
fn build_mock_aneurysm(
    n: usize,
) -> Result<(Manifold<f64>, VesselState), Box<dyn std::error::Error>> {
    let mut positions_flat = Vec::with_capacity(n * 3);
    let mut positions_vec = Vec::with_capacity(n);
    let mut velocities = Vec::with_capacity(n);
    let mut pressures = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f64 / n as f64;
        // Cylinder with a bulge in the middle (0.4 to 0.6)
        let radius = if t > 0.4 && t < 0.6 { 2.5 } else { 1.0 };

        let x = t * 10.0;
        let y = radius;
        let z = 0.0;

        positions_flat.push(x);
        positions_flat.push(y);
        positions_flat.push(z);

        positions_vec.push(vec![x, y, z]);

        // Fluid moves faster in constriction, slower in aneurysm (Bernoulli)
        let base_velocity = 5.0; // m/s
        let v = base_velocity / radius.powi(2);

        let stress_factor = if (t - 0.4).abs() < 0.05 || (t - 0.6).abs() < 0.05 {
            4.0
        } else {
            1.0
        };

        velocities.push(vec![v * stress_factor, 0.0, 0.0]);
        pressures.push(120.0);
    }

    // Create PontCloud and Triangulate
    let pos_tensor = CausalTensor::new(positions_flat, vec![n, 3])?;
    let data_tensor = CausalTensor::new(vec![0.0; n], vec![n])?; // Dummy scalar data on manifold

    let pc = PointCloud::new(pos_tensor, data_tensor, 0)?;

    // Triangulate to get SimplicialComplex
    // Use a small radius to ensure connectivity (neighbor dist ~0.2) but avoid explosion
    let complex = pc.triangulate(0.35)?;

    // Re-create manifold with this complex and corrected data size
    let total_simplices = complex.total_simplices();
    let manifold_data = CausalTensor::new(vec![0.0; total_simplices], vec![total_simplices])?;

    let manifold = Manifold::new(complex, manifold_data, 0)?;

    Ok((
        manifold,
        VesselState {
            positions: positions_vec,
            velocities,
            pressures,
        },
    ))
}
