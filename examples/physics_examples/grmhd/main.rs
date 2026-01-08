/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GRMHD: General Relativistic Magnetohydrodynamics
//!
//! This example demonstrates a "Multi-Physics Monad" approach for simulating extreme
//! environments (Black Holes, Neutron Stars) by coupling:
//!
//! 1. **GR Solver (Tensor)**: Computes spacetime curvature
//! 2. **Coupling Layer**: Dynamically selects appropriate Metric based on curvature
//! 3. **MHD Solver (MultiVector)**: Computes plasma forces using the selected metric
//!
//! ## Causal Chain
//!
//! ```text
//! Spacetime Metric (g_uv) → Einstein Tensor (G_uv) → Metric Selection
//!                                                          ↓
//!                                              Lorentz Force (F = J·B)
//!                                                          ↓
//!                                                   Stability Analysis
//! ```

mod model;

use deep_causality::PropagatingEffect;
use model::{GrmhdState, SimulationConfig};

fn main() {
    println!("============================================================");
    println!("   GRMHD: General Relativistic Magnetohydrodynamics");
    println!("============================================================");
    println!("   (Causaloid-based Multi-Physics Simulation)\n");

    // =========================================================================
    // Configuration
    // =========================================================================
    let config = SimulationConfig {
        current_density: 10.0, // Plasma current J
        magnetic_field: 2.0,   // Magnetic field B
        curvature_threshold: 0.05,
    };

    println!("Configuration:");
    println!("  Current Density J: {:.2}", config.current_density);
    println!("  Magnetic Field B:  {:.2}", config.magnetic_field);
    println!("  Curvature Threshold: {:.4}", config.curvature_threshold);

    // =========================================================================
    // Execute Causal Chain via Monadic Composition
    // =========================================================================
    // The chain: GR Solver → Coupling → MHD Solver → Analysis
    let result: PropagatingEffect<GrmhdState> = PropagatingEffect::pure(GrmhdState::new(&config))
        .bind(|state, _, _| {
            println!("\n[Step 1] GR Solver: Calculating Spacetime Curvature...");
            model::calculate_curvature(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            println!("\n[Step 2] Causal Coupling: Configuring MHD Solver...");
            model::select_metric(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            println!("\n[Step 3] MHD Solver: Calculating Plasma Confinement...");
            model::calculate_lorentz_force(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            println!("\n[Step 4] GRMHD Coupling: Calculating EM Stress-Energy...");
            model::calculate_energy_momentum(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            println!("\n[Step 5] Stability Analysis...");
            model::analyze_stability(state.into_value().unwrap_or_default())
        });

    // =========================================================================
    // Display Results
    // =========================================================================
    if result.is_err() {
        eprintln!("Simulation failed: {:?}", result.error);
        return;
    }

    let final_state = result.value.into_value().unwrap_or_default();

    println!("\n============================================================");
    println!("CONCLUSION:");
    println!("============================================================");
    println!(
        "  Curvature Intensity:  {:.4}",
        final_state.curvature_intensity
    );
    println!("  Selected Metric:      {}", final_state.metric_label);
    println!("  Lorentz Force F:      {:.4}", final_state.lorentz_force);
    println!(
        "  EM Energy Density:    {:.4}",
        final_state.em_energy_density
    );
    println!("  Stability Status:     {}", final_state.stability_status);
    println!();
    println!("Data Flow: Spacetime Geometry → Coupling → Plasma Physics → Gravity Feedback");
    println!("============================================================");
}
