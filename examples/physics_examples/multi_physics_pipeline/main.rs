/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Multi-Physics Pipeline: Complete QFT → QCD → Thermal → Detection
//!
//! Demonstrates **modular composition** via `CausalEffectPropagationProcess`
//! (the Causal Monad) for a complete high-energy physics simulation.
//!
//! ## Key Design Pattern
//!
//! Each physics stage is a **standalone function** that can be:
//! - Tested independently
//! - Replaced without affecting the pipeline
//! - Composed in different orders
//! - Reused across different simulations
//!
//! ```ignore
//! klein_gordon()
//!     .bind_or_error(stage_field_to_partons, ...)    // Modular: Field → q-q̄
//!     .bind_or_error(stage_lund_fragmentation, ...)  // Modular: q-q̄ → hadrons
//!     .bind_or_error(stage_thermalization, ...)      // Modular: hadrons → thermal
//!     .bind_or_error(stage_quantum_detection, ...)   // Modular: thermal → detection
//! ```
//!
//! This is the power of the Causal Monad: **decoupled physics modules**
//! that compose seamlessly with automatic error propagation.
use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    FourMomentum, Hadron, LundParameters, born_probability, heat_diffusion, klein_gordon,
    lund_string_fragmentation_kernel,
};
use deep_causality_tensor::CausalTensor;
mod model;

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Multi-Physics Pipeline: QFT → QCD → Thermal → Detection");
    println!("  (Modular Stages Composed via Causal Monad)");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Initial conditions
    let phi_data = vec![1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];
    let phi_manifold = model::make_1d_manifold(phi_data);
    let mass = 125.0;

    // =========================================================================
    // The Causal Monad Pipeline: Each stage is a decoupled function
    // =========================================================================
    let result = klein_gordon(&phi_manifold, mass)
        .bind_or_error(stage_field_to_partons, "Field → Partons failed")
        .bind_or_error(stage_lund_fragmentation, "Lund fragmentation failed")
        .bind_or_error(stage_thermalization, "Thermalization failed")
        .bind_or_error(stage_quantum_detection, "Detection failed");

    // Extract final result
    print_summary(&result);

    Ok(())
}

// =============================================================================
// STAGE 1: Field Energy → Virtual Quark-Antiquark Creation
// =============================================================================

/// Converts Klein-Gordon field evolution into virtual quark-antiquark endpoints.
///
/// # Physics
/// - Computes total field energy: E = Σ|φ|²
/// - Creates back-to-back q-q̄ pair with combined energy = E_cms
///
/// # Maintenance
/// - Modify quark masses here
/// - Change energy scaling independently
/// - Add multiple q-q̄ pairs for multi-jet events
fn stage_field_to_partons(
    evolved_tensor: CausalTensor<f64>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<Vec<(FourMomentum, FourMomentum)>> {
    println!("Stage 1: Klein-Gordon Scalar Field");
    println!("───────────────────────────────────");

    // Compute field energy
    let field_energy: f64 = evolved_tensor
        .data()
        .iter()
        .map(|&v| v.abs().powi(2))
        .sum::<f64>()
        * 0.01;

    let cms_energy = field_energy.clamp(10.0, 500.0);
    println!("  Field energy: E_cms = {:.2} GeV\n", cms_energy);

    // Create virtual q-q̄ pair (back-to-back in CM frame)
    let half_e = cms_energy / 2.0;
    let quark = FourMomentum::new(half_e, 0.0, 0.0, half_e);
    let antiquark = FourMomentum::new(half_e, 0.0, 0.0, -half_e);

    println!("Stage 2: QCD String Creation");
    println!("────────────────────────────");
    println!("  q:  (E={:.1}, pz=+{:.1}) GeV", half_e, half_e);
    println!("  q̄:  (E={:.1}, pz=-{:.1}) GeV", half_e, half_e);

    CausalEffectPropagationProcess::pure(vec![(quark, antiquark)])
}

// =============================================================================
// STAGE 2: Lund String Fragmentation → Hadrons
// =============================================================================

/// Fragments QCD strings into hadrons using the Lund model.
///
/// # Physics
/// - Iterative string breaking with q-q̄ pair creation
/// - Produces π, K, ρ, ω, η, etc.
///
/// # Maintenance
/// - Tune Lund parameters (a, b, σ_pt) independently
/// - Replace with different fragmentation model
/// - Add particle filtering or cuts
fn stage_lund_fragmentation(
    endpoints: Vec<(FourMomentum, FourMomentum)>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, f64)> {
    println!("\nStage 3: Lund String Fragmentation");
    println!("───────────────────────────────────");

    let params = LundParameters::default();
    let mut rng = deep_causality_rand::rng();

    match lund_string_fragmentation_kernel(&endpoints, &params, &mut rng) {
        Ok(hadrons) => {
            let valid: Vec<&Hadron> = hadrons.iter().filter(|h| h.energy() > 0.0).collect();

            println!(
                "  Produced {} hadrons ({} physical)",
                hadrons.len(),
                valid.len()
            );
            print_hadron_sample(&valid);

            let total_e: f64 = valid.iter().map(|h| h.energy()).sum();
            CausalEffectPropagationProcess::pure((valid.len(), total_e))
        }
        Err(_) => CausalEffectPropagationProcess::pure((0, 0.0)),
    }
}

// =============================================================================
// STAGE 3: Thermalization via Heat Diffusion
// =============================================================================

/// Thermalizes hadron gas using heat diffusion.
///
/// # Physics
/// - Creates temperature field from hadron energies
/// - Evolves via diffusion equation: ∂T/∂t = κ∇²T
///
/// # Maintenance
/// - Adjust diffusivity independently
/// - Replace with hydrodynamic evolution
/// - Add viscosity corrections
fn stage_thermalization(
    (hadron_count, total_energy): (usize, f64),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, f64)> {
    println!("\nStage 4: Thermalization");
    println!("───────────────────────");

    // Scale to MeV (typical QGP temperature ~ 150-400 MeV)
    let temp_scale = (total_energy * 0.5).clamp(100.0, 500.0);
    let initial_temp: Vec<f64> = (0..10)
        .map(|i| temp_scale * (1.0 - i as f64 * 0.02))
        .collect();
    let temp_manifold = model::make_1d_manifold(initial_temp.clone());

    let diffusivity = 0.1;
    let heat_result = heat_diffusion(&temp_manifold, diffusivity);

    // Use diffused result if valid, otherwise use initial average
    let avg_temp = match heat_result.value() {
        EffectValue::Value(final_temp) => {
            let avg = final_temp.data().iter().sum::<f64>() / 10.0;
            if avg.abs() > 1.0 {
                avg.abs()
            } else {
                // Fallback: use initial temperature average
                initial_temp.iter().sum::<f64>() / 10.0
            }
        }
        _ => temp_scale * 0.9, // Slight cooling
    };

    println!("  Initial temp: {:.1} MeV", temp_scale);
    println!("  Equilibrium:  {:.1} MeV", avg_temp);

    CausalEffectPropagationProcess::pure((hadron_count, avg_temp))
}

// =============================================================================
// STAGE 4: Quantum Detection via Born Rule
// =============================================================================

/// Computes detection probability using the Born rule.
///
/// # Physics
/// - Creates detector wavefunction from thermal signal
/// - P(detection) = |⟨basis|ψ⟩|²
///
/// # Maintenance
/// - Change basis states independently
/// - Add multiple detector channels
/// - Implement more complex observables
fn stage_quantum_detection(
    (hadron_count, avg_temp): (usize, f64),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, f64, f64)> {
    println!("\nStage 5: Quantum Detection");
    println!("──────────────────────────");

    // Detection probability scales with temperature
    // At T_c ~ 170 MeV (QGP transition), detection is 50%
    // Higher temp → higher detection probability
    let t_critical = 170.0; // MeV
    let psi_val = (avg_temp / (avg_temp + t_critical)).clamp(0.01, 0.99);
    let psi = Complex::new(psi_val.sqrt(), 0.0);
    let psi_orth = Complex::new((1.0 - psi_val).sqrt(), 0.0);

    let metric = Metric::Euclidean(1);
    let state = HilbertState::new(vec![psi, psi_orth], metric).unwrap();
    let basis =
        HilbertState::new(vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)], metric).unwrap();

    let detection = born_probability(&state, &basis);
    let prob = match detection.value() {
        EffectValue::Value(p) => p.value(),
        _ => 0.0,
    };

    println!("  Critical temp: T_c = {} MeV", t_critical);
    println!(
        "  |ψ⟩ = {:.3}|QGP⟩ + {:.3}|hadron⟩",
        psi.re(),
        psi_orth.re()
    );
    println!("  P(QGP detection) = {:.4}", prob);

    CausalEffectPropagationProcess::pure((hadron_count, avg_temp, prob))
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints a sample of produced hadrons.
fn print_hadron_sample(hadrons: &[&Hadron]) {
    println!("\n  Sample hadrons:");
    for (i, h) in hadrons.iter().take(5).enumerate() {
        println!(
            "    [{:2}] {} (PDG {}): E = {:.2} GeV",
            i + 1,
            pdg_name(h.pdg_id()),
            h.pdg_id(),
            h.energy()
        );
    }
    if hadrons.len() > 5 {
        println!("    ... and {} more", hadrons.len() - 5);
    }
}

/// Prints the final pipeline summary.
fn print_summary(result: &PropagatingEffect<(usize, f64, f64)>) {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");

    match result.value() {
        EffectValue::Value((hadron_count, avg_temp, prob)) => {
            println!("  Hadron multiplicity:    {} particles", hadron_count);
            println!("  Thermal equilibrium:    {:.2} MeV", avg_temp);
            println!("  Detection probability:  {:.4}", prob);
            println!("\n[SUCCESS] Modular Pipeline Completed.\n");
        }
        _ => {
            println!("  Pipeline returned unexpected result");
            println!("\n[WARN] Check individual stage outputs.\n");
        }
    }
}

/// Gets particle name from PDG ID.
fn pdg_name(pdg_id: i32) -> &'static str {
    match pdg_id.abs() {
        111 => "π⁰",
        211 => {
            if pdg_id > 0 {
                "π⁺"
            } else {
                "π⁻"
            }
        }
        221 => "η",
        311 => "K⁰",
        321 => {
            if pdg_id > 0 {
                "K⁺"
            } else {
                "K⁻"
            }
        }
        113 => "ρ⁰",
        213 => {
            if pdg_id > 0 {
                "ρ⁺"
            } else {
                "ρ⁻"
            }
        }
        223 => "ω",
        331 => "η'",
        333 => "φ",
        _ => "hadron",
    }
}
