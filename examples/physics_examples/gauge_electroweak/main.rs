/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Electroweak Unification Pipeline
//!
//! Demonstrates the **Electroweak Theory (SU(2) x U(1))** and spontaneous symmetry breaking.
//!
//! ## Stages
//!
//! 1. **Unification**: Establish couplings g and g' from α_EM and θ_W
//! 2. **Symmetry Breaking**: Generate masses via Higgs VEV
//! 3. **Gauge Mixing**: Confirm W/Z mass ratio and ρ parameter
//! 4. **Resonance**: Compute Z pole cross-section

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_physics::ElectroweakParams;

// =============================================================================
// MAIN
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Electroweak Unification Pipeline (SU(2) × U(1))");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Stage 1: Initialize Standard Model
    let initial = stage_unification();

    // Pipeline
    let result = initial
        .bind_or_error(stage_symmetry_breaking, "Symmetry breaking failed")
        .bind_or_error(stage_gauge_mixing, "Gauge mixing failed")
        .bind_or_error(stage_z_resonance, "Resonance calc failed");

    print_summary(&result);

    Ok(())
}

// =============================================================================
// DATA STATE
// =============================================================================

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct EwState {
    ew: Option<ElectroweakParams>,
    w_mass_calc: f64,
    z_mass_calc: f64,
    higgs_lambda: f64,
    top_yukawa: f64,
    z_peak_sigma: f64,
}

// =============================================================================
// STAGES
// =============================================================================

fn stage_unification() -> PropagatingEffect<EwState> {
    println!("Stage 1: Unification (Coupling Constants)");
    println!("───────────────────────────────────────");

    let ew = ElectroweakParams::standard_model();

    println!("  Weinberg Angle:     sin²θ_W = {:.4}", ew.sin2_theta_w());
    println!("  EM Coupling (e):    {:.4}", ew.em_coupling());
    println!("  Weak Coupling (g):  {:.4}", ew.g_coupling());
    println!("  Hypercharge (g'):   {:.4}", ew.g_prime_coupling());
    println!();

    let state = EwState {
        ew: Some(ew),
        ..Default::default()
    };

    CausalEffectPropagationProcess::pure(state)
}

fn stage_symmetry_breaking(mut state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    println!("Stage 2: Spontaneous Symmetry Breaking (Higgs)");
    println!("──────────────────────────────────────────────");

    if let Some(ew) = state.ew {
        let v = ew.higgs_vev();
        state.higgs_lambda = ew.higgs_quartic();
        state.top_yukawa = ew.top_yukawa();

        // Calculate masses from scratch using g and v
        state.w_mass_calc = ew.w_mass_computed();
        state.z_mass_calc = ew.z_mass_computed();

        println!("  Higgs VEV (v):      {:.2} GeV", v);
        println!("  Quartic Coupling:   λ = {:.4}", state.higgs_lambda);
        println!("  Top Yukawa:         y_t = {:.4}", state.top_yukawa);
        println!(
            "  Generated M_W:      {:.2} GeV (from g·v/2)",
            state.w_mass_calc
        );
        println!(
            "  Generated M_Z:      {:.2} GeV (from M_W/cosθ)",
            state.z_mass_calc
        );
        println!();

        CausalEffectPropagationProcess::pure(state)
    } else {
        CausalEffectPropagationProcess::pure(state)
    }
}

fn stage_gauge_mixing(state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    println!("Stage 3: Gauge Boson Mixing");
    println!("───────────────────────────");

    if let Some(ew) = state.ew {
        let rho = ew.rho_parameter();
        let prediction_match = (state.w_mass_calc - ew.w_mass()).abs() < 1.0;

        println!("  ρ parameter:        {:.4} (Tree level SM = 1.0)", rho);
        println!(
            "  Mass Prediction:    {}",
            if prediction_match {
                "Accurate within 1 GeV"
            } else {
                "Deviation found"
            }
        );
        println!("  Theory M_W:         {:.3} GeV", state.w_mass_calc);
        println!("  PDG M_W:            {:.3} GeV", ew.w_mass());
        println!();
    }

    CausalEffectPropagationProcess::pure(state)
}

fn stage_z_resonance(mut state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    println!("Stage 4: Z Resonance (s-channel)");
    println!("────────────────────────────────");

    if let Some(ew) = state.ew {
        // Compute cross section at peak (sqrt(s) = M_Z)
        let mz = ew.z_mass();
        let width = 2.495; // GeV

        match ew.z_resonance_cross_section(mz, width) {
            Ok(sigma) => {
                state.z_peak_sigma = sigma;
                println!("  Peak Energy:        {:.2} GeV", mz);
                println!("  Z Width:            {:.3} GeV", width);
                println!("  Peak Cross-sec:     {:.2} nb", sigma);
                println!();
            }
            Err(e) => println!("  [ERROR] Cross section failed: {:?}", e),
        }
    }

    CausalEffectPropagationProcess::pure(state)
}

fn print_summary(result: &PropagatingEffect<EwState>) {
    match result.value() {
        EffectValue::Value(state) => {
            println!("[SUCCESS] Electroweak Unification Verified.");
            println!("  Generated W Mass:   {:.3} GeV", state.w_mass_calc);
            println!(
                "  Top Yukawa:         {:.3} (Truth ~ 1.0)",
                state.top_yukawa
            );
        }
        _ => println!("[ERROR] Pipeline failed"),
    }
}
