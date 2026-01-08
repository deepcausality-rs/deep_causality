/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Weak Force Pipeline
//!
//! Demonstrates **Weak Force (SU(2))** interactions using the Causal Monad.
//!
//! ## Stages
//!
//! 1. **Initialize Particles**: Create left-handed fermions
//! 2. **Charged Current**: Compute W-mediated decay (Muon decay)
//! 3. **Neutral Current**: Compute Z-mediated scattering (Neutrino scattering)
//! 4. **Analysis**: Lifetime and width calculations

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_num::DoubleFloat;
use deep_causality_physics::{WeakField, WeakFieldOps, WeakIsospin};

// =============================================================================
// FLOAT TYPE CONFIGURATION
// =============================================================================

// Change this to f32 or DoubleFloat to use different precision
type FloatType = DoubleFloat;
type WeakTheory = WeakField<FloatType>;

/// Macro to convert f64 literals to FloatType
macro_rules! flt {
    ($x:expr) => {
        <FloatType as From<f64>>::from($x)
    };
}

// =============================================================================
// MAIN
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Weak Force Analysis Pipeline (SU(2))");
    println!("  (Float Type: {})", std::any::type_name::<FloatType>());
    println!("═══════════════════════════════════════════════════════════════\n");

    // Stage 1: Initialize
    let initial = stage_initialize();

    // Pipeline
    let result = initial
        .bind_or_error(stage_charged_current, "Charged current failed")
        .bind_or_error(stage_neutral_current, "Neutral current failed")
        .bind_or_error(stage_decay_properties, "Decay calculation failed");

    print_summary(&result);

    Ok(())
}

// =============================================================================
// DATA STATE
// =============================================================================

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct WeakState {
    muon: Option<WeakIsospin>,
    neutrino: Option<WeakIsospin>,
    cc_propagator: FloatType,
    nc_propagator: FloatType,
    muon_width: FloatType,
    muon_lifetime: FloatType,
    w_width: FloatType,
}

// =============================================================================
// STAGES
// =============================================================================

fn stage_initialize() -> PropagatingEffect<WeakState> {
    println!("Stage 1: Particle Initialization (Left-Handed Doublets)");
    println!("─────────────────────────────────────────────────────");

    let muon = WeakIsospin::lepton_doublet(); // I3 = -1/2
    let neutrino = WeakIsospin::neutrino(); // I3 = +1/2

    println!(
        "  Muon (L):      I = {}, I3 = {}, Q = {}",
        muon.isospin,
        muon.i3,
        muon.electric_charge()
    );
    println!(
        "  Neutrino (L):  I = {}, I3 = {}, Q = {}",
        neutrino.isospin,
        neutrino.i3,
        neutrino.electric_charge()
    );
    println!();

    let state = WeakState {
        muon: Some(muon),
        neutrino: Some(neutrino),
        ..Default::default()
    };

    CausalEffectPropagationProcess::pure(state)
}

fn stage_charged_current(
    mut state: WeakState,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<WeakState> {
    println!("Stage 2: Charged Current Interaction (W Exchange)");
    println!("─────────────────────────────────────────────────");

    // Simulate W exchange at low energy (q² << M_W²)
    let q2 = flt!(0.01); // GeV²
    match WeakTheory::charged_current_propagator(q2) {
        Ok(prop) => {
            state.cc_propagator = prop;
            println!("  q²:            {} GeV²", q2);
            println!("  W Propagator:  {:.4e} GeV⁻²", prop);
            println!("  Interaction:   μ⁻ → e⁻ + ν_μ + ν̄_e");
            println!();
            CausalEffectPropagationProcess::pure(state)
        }
        Err(e) => {
            println!("  [ERROR] CC Calc failed: {:?}", e);
            CausalEffectPropagationProcess::pure(state)
        }
    }
}

fn stage_neutral_current(
    mut state: WeakState,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<WeakState> {
    println!("Stage 3: Neutral Current Interaction (Z Exchange)");
    println!("─────────────────────────────────────────────────");

    if let Some(nu) = state.neutrino {
        // Scattering at higher energy
        let q2 = flt!(100.0); // GeV²
        match WeakTheory::neutral_current_propagator(q2, &nu) {
            Ok(prop) => {
                state.nc_propagator = prop;
                println!("  q²:            {} GeV²", q2);
                println!("  Z Propagator:  {} GeV⁻²", prop);
                println!("  Coupling g_V:  {}", nu.vector_coupling());
                println!("  Coupling g_A:  {}", nu.axial_coupling());
                println!();
                CausalEffectPropagationProcess::pure(state)
            }
            Err(e) => {
                println!("  [ERROR] NC Calc failed: {:?}", e);
                CausalEffectPropagationProcess::pure(state)
            }
        }
    } else {
        CausalEffectPropagationProcess::pure(state)
    }
}

fn stage_decay_properties(
    mut state: WeakState,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<WeakState> {
    println!("Stage 4: Decay Properties");
    println!("─────────────────────────");

    // Muon decay
    let m_mu = flt!(0.10566); // GeV
    if let Ok(width) = WeakTheory::weak_decay_width(m_mu) {
        state.muon_width = width;
        println!("  Muon decay width: {} GeV", width);
    }

    state.muon_lifetime = WeakTheory::muon_lifetime();
    println!("  Muon lifetime:    {} s", state.muon_lifetime);

    state.w_width = WeakTheory::w_boson_width();
    println!("  W Boson width:    {} GeV", state.w_width);
    println!();

    CausalEffectPropagationProcess::pure(state)
}

fn print_summary(result: &PropagatingEffect<WeakState>) {
    match result.value() {
        EffectValue::Value(state) => {
            println!("[SUCCESS] Weak Force Analysis Complete.");
            println!("  W Width: {} GeV (PDG: ~2.085 GeV)", state.w_width);
            println!(
                "  Muon Lifetime: {} s (PDG: ~2.2e-6 s)",
                state.muon_lifetime
            );
        }
        _ => println!("[ERROR] Pipeline failed"),
    }
}
