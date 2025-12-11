/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Multi-Physics Pipeline: Grand Unification
//!
//! Demonstrates a complete simulation chain: Quantum Field → Hadronization → Hydro → Detection.
//!
//! ## Key Concepts
//! - **Klein-Gordon**: Relativistic scalar field evolution
//! - **Hadronization**: Particle production from energy density
//! - **Heat Diffusion**: Thermal equilibration of particle cloud
//! - **Born Rule**: Quantum measurement probability
//!
//! ## APIs Demonstrated
//! - `klein_gordon()` - Scalar field dynamics
//! - `hadronization()` - Jet production
//! - `heat_diffusion()` - Thermal physics
//! - `born_probability()` - Quantum detection
//! - `bind_or_error()` - Error-handling monadic composition

use deep_causality_core::CausalEffectPropagationProcess;
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    EnergyDensity, born_probability, hadronization, heat_diffusion, klein_gordon,
};
mod model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Pipeline: QF (Klein-Gordon) -> Hadronization -> Hydro (Heat Diffusion) -> Detection\n"
    );

    let phi_data = vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];
    let phi_manifold = model::make_1d_manifold(phi_data);
    let mass = 125.0;

    let result = klein_gordon(&phi_manifold, mass)
        .bind_or_error(
            |evolved_tensor, _, _| {
                let data = evolved_tensor.data();
                let densities: Vec<EnergyDensity> = data
                    .iter()
                    .map(|&v| EnergyDensity::new(v * v * 100.0).unwrap_or_default())
                    .collect();

                CausalEffectPropagationProcess::pure(densities)
            },
            "Failed to compute energy density",
        )
        .bind_or_error(
            |densities, _, _| {
                let threshold = 5.0;
                let dim = 3;
                hadronization(&densities, threshold, dim)
            },
            "Hadronization failed",
        )
        .bind_or_error(
            |jets, _, _| {
                println!("  -> Generated {} Particle Jets/Hadrons", jets.len());
                let total_energy: f64 = jets.iter().map(|v| v.0.data()[1]).sum();
                let temp_val = total_energy * 1000.0;
                let initial_temp_grid = vec![temp_val.max(0.1); 10];
                let temp_manifold = model::make_1d_manifold(initial_temp_grid);

                // Wrap in Option for Default
                CausalEffectPropagationProcess::pure(Some(temp_manifold))
            },
            "Thermalization failed",
        )
        .bind_or_error(
            |temp_manifold_opt, _, _| {
                let temp_manifold = temp_manifold_opt.unwrap();
                let diffusivity = 0.5;
                heat_diffusion(&temp_manifold, diffusivity)
            },
            "Heat diffusion failed",
        )
        .bind_or_error(
            |final_temp_tensor, _, _| {
                let avg_temp = final_temp_tensor.data().iter().sum::<f64>() / 10.0;
                println!("  -> Final Quark-Gluon Plasma Temp: {:.2} K", avg_temp);

                let psi_val = avg_temp.clamp(0.0, 1.0);
                let psi = Complex::new(psi_val, 0.0);

                let metric = Metric::Euclidean(1);
                let state = HilbertState::new(vec![psi, Complex::new(0.0, 0.0)], metric).unwrap();
                let basis =
                    HilbertState::new(vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)], metric)
                        .unwrap();

                born_probability(&state, &basis)
            },
            "Observation failed",
        );

    match result.value() {
        deep_causality_core::EffectValue::Value(prob) => {
            println!("  -> Detection Probabilities: {:.4}", prob.value());
            println!("\n[SUCCESS] Pipeline Completed.");
        }
        _ => {
            eprintln!("\n[FAILURE] Pipeline Failed");
        }
    }

    Ok(())
}
