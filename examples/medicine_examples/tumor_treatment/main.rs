/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Glioblastoma TTFields Treatment Optimization
//!
//! Demonstrates using Electromagnetism and Geometric Algebra (GA) to optimize
//! Tumor Treating Fields (TTFields) electrode placement.
//!
//! ## Key Concepts
//! - **Geometric Algebra**: Calculating alignment between Field vectors and Cell Axis bivectors.
//! - **Causal Optimization**: Iterative process to maximize "Disruption Efficacy".
//! - **Simulated Annealing**: Simple optimizer finding best field orientation.
use deep_causality_core::{CausalityError, CausalityErrorEnum, EffectValue, PropagatingEffect};

mod model;

const TUMOR_RADIUS: f64 = 2.0; // cm
const OPTIMIZATION_STEPS: usize = 20;

// Represents the 3D grid of the tumor
struct TumorVolume {
    // Voxel positions
    voxels: Vec<[f64; 3]>,
    // The dominant axis of cell division at each voxel (unit vector)
    cell_axes: Vec<[f64; 3]>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Glioblastoma TTFields Optimization ===\n");

    // 1. Clinical Data Ingestion (Mock)
    // Create a tumor volume with non-uniform cell division directions
    let tumor = model::build_mock_tumor(100);
    println!("Loaded Tumor Volume: {} voxels", tumor.voxels.len());

    // 2. Optimization Loop
    // We want to find the best electrode angle (theta, phi) to maximize disruption.
    // Disruption ~ |E . Axis|^2

    // Initial guess: Transducer at (0, 0)
    let initial_params = (0.0, 0.0); // (Theta, Phi)
    let initial_score = model::evaluate_efficacy(&tumor, initial_params)?;

    println!("Initial Configuration Efficacy: {:.4}", initial_score);
    println!("Starting Causal Optimization Loop...\n");

    let mut current_params = initial_params;
    let mut current_score = initial_score;
    let mut temperature = 1.0;

    for step in 1..=OPTIMIZATION_STEPS {
        // A. Propose new parameters (random perturbation)
        let (theta, phi) = current_params;
        let d_theta = (model::rand_f64() - 0.5) * temperature;
        let d_phi = (model::rand_f64() - 0.5) * temperature;

        let new_params = (theta + d_theta, phi + d_phi);

        // B. Causal Impact Analysis using PropagatingEffect
        // We evaluate "What if we change to new_params?"
        let effect = PropagatingEffect::pure(new_params).bind(|params, _, _| {
            let p = match params {
                EffectValue::Value(v) => v,
                _ => (0.0, 0.0),
            };

            // Simulate Physics
            let score = match model::evaluate_efficacy(&tumor, p) {
                Ok(s) => s,
                Err(e) => {
                    return PropagatingEffect::from_error(CausalityError::new(
                        CausalityErrorEnum::Custom(e.to_string()),
                    ));
                }
            };

            PropagatingEffect::pure(score)
        });

        // C. Decision Logic (Metropolis-like accept/reject)
        if let EffectValue::Value(new_score) = effect.value() {
            let delta = *new_score - current_score;
            if delta > 0.0 {
                // Improvement: Accept
                current_params = new_params;
                current_score = *new_score;
                println!(
                    "[Step {:>2}] IMPROVED: Score {:.4} (Angle: {:.2}, {:.2})",
                    step, current_score, current_params.0, current_params.1
                );
            } else {
                // Worse: Accept with probability related to temperature
                let prob = (delta / (0.1 * temperature)).exp();
                if model::rand_f64() < prob {
                    current_params = new_params;
                    current_score = *new_score;
                }
            }
        }

        temperature *= 0.9; // Cool down
    }

    println!("\n=== Optimization Complete ===");
    println!(
        "Optimal Transducer Orientation: Theta={:.2}, Phi={:.2}",
        current_params.0, current_params.1
    );
    println!("Final Disruption Index: {:.4}", current_score);

    Ok(())
}
