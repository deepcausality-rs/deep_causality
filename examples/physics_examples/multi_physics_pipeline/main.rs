/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
use deep_causality_algebra::Real;
use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, CausalityError, PropagatingEffect,
};
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::{lift, lift_usize, lower};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    FourMomentum, Hadron, LundParameters, heat_diffusion, klein_gordon,
    lund_string_fragmentation_kernel,
};
use deep_causality_quantum::born_probability;
use deep_causality_tensor::CausalTensor;
mod model;

/// Scale from the Klein-Gordon field energy to a centre-of-mass energy, and the range it is
/// held to, in GeV.
const FIELD_ENERGY_SCALE: f64 = 0.01;
const MIN_CMS_ENERGY_GEV: f64 = 10.0;
const MAX_CMS_ENERGY_GEV: f64 = 500.0;
/// Fraction of the hadron energy that goes into the thermal bath, and the range it is held to.
const TEMPERATURE_SHARE: f64 = 0.5;
const MIN_TEMPERATURE_MEV: f64 = 100.0;
const MAX_TEMPERATURE_MEV: f64 = 500.0;
/// Cells in the 1D temperature field, and the fractional drop per cell.
const TEMPERATURE_CELLS: usize = 10;
const TEMPERATURE_GRADIENT: f64 = 0.02;
/// Thermal diffusivity, and the cooling applied when the diffusion step yields nothing.
const DIFFUSIVITY: f64 = 0.1;
const COOLING_FACTOR: f64 = 0.9;
/// Initial Klein-Gordon field profile across the cells.
const PHI_PROFILE: [f64; TEMPERATURE_CELLS] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];
/// Scalar mass driving the Klein-Gordon evolution, in GeV.
const HIGGS_MASS_GEV: f64 = 125.0;
/// QGP transition temperature, in MeV.
const CRITICAL_TEMPERATURE_MEV: f64 = 170.0;
/// The detection amplitude is held inside this range so neither basis state is exactly empty.
const MIN_AMPLITUDE: f64 = 0.01;
const MAX_AMPLITUDE: f64 = 0.99;

/// `clamp` at the working scalar. `Ord::clamp` is unavailable for a partially ordered float, so
/// this spells out the two comparisons rather than pinning the type to a primitive.
fn clamp(value: FloatType, low: FloatType, high: FloatType) -> FloatType {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Multi-Physics Pipeline: QFT → QCD → Thermal → Detection");
    println!("  (Modular Stages Composed via Causal Monad)");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Initial field profile: a linear ramp across the cells, lifted into the working type.
    let phi_data: Vec<FloatType> = PHI_PROFILE.iter().map(|&v| lift::<FloatType>(v)).collect();
    let phi_manifold = model::make_1d_manifold(phi_data)?;
    let mass = lift::<FloatType>(HIGGS_MASS_GEV);

    // =========================================================================
    // The Causal Monad Pipeline: Each stage is a decoupled function
    // =========================================================================
    // The same decoupled stages, now driven by `CausalFlow`. The existing
    // `(value, (), Option<()>) -> PropagatingEffect` stages drop in unchanged
    // through the `bind_or_error` passthrough; only the seed (`From`) and the
    // terminal (`run`) change.
    CausalFlow::from(klein_gordon(&phi_manifold, mass))
        .bind_or_error(stage_field_to_partons, "Field → Partons failed")
        .bind_or_error(stage_lund_fragmentation, "Lund fragmentation failed")
        .bind_or_error(stage_thermalization, "Thermalization failed")
        .bind_or_error(stage_quantum_detection, "Detection failed")
        .run(print_summary_ok, |err| print_summary_err(&err));

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
    evolved_tensor: CausalTensor<FloatType>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<Vec<(FourMomentum<FloatType>, FourMomentum<FloatType>)>> {
    println!("Stage 1: Klein-Gordon Scalar Field");
    println!("───────────────────────────────────");

    // Compute field energy
    let field_energy: FloatType = evolved_tensor
        .data()
        .iter()
        .map(|&v| Real::abs(v) * Real::abs(v))
        .sum::<FloatType>()
        * lift::<FloatType>(FIELD_ENERGY_SCALE);

    let cms_energy = clamp(
        field_energy,
        lift(MIN_CMS_ENERGY_GEV),
        lift(MAX_CMS_ENERGY_GEV),
    );
    println!("  Field energy: E_cms = {:.2} GeV\n", cms_energy);

    // Create virtual q-q̄ pair (back-to-back in CM frame)
    let half_e = cms_energy / lift::<FloatType>(2.0);
    let zero = lift::<FloatType>(0.0);
    let quark = FourMomentum::<FloatType>::new(half_e, zero, zero, half_e);
    let antiquark = FourMomentum::<FloatType>::new(half_e, zero, zero, -half_e);

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
    endpoints: Vec<(FourMomentum<FloatType>, FourMomentum<FloatType>)>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, FloatType)> {
    println!("\nStage 3: Lund String Fragmentation");
    println!("───────────────────────────────────");

    let params = LundParameters::default();
    let mut rng = deep_causality_rand::rng();

    match lund_string_fragmentation_kernel(&endpoints, &params, &mut rng) {
        Ok(hadrons) => {
            let valid: Vec<&Hadron<FloatType>> =
                hadrons.iter().filter(|h| h.energy() > 0.0).collect();

            println!(
                "  Produced {} hadrons ({} physical)",
                hadrons.len(),
                valid.len()
            );
            print_hadron_sample(&valid);

            let total_e: FloatType = valid.iter().map(|h| h.energy()).sum();
            CausalEffectPropagationProcess::pure((valid.len(), total_e))
        }
        Err(_) => CausalEffectPropagationProcess::pure((0, lift::<FloatType>(0.0))),
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
    (hadron_count, total_energy): (usize, FloatType),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, FloatType)> {
    println!("\nStage 4: Thermalization");
    println!("───────────────────────");

    // Scale to MeV (typical QGP temperature ~ 150-400 MeV)
    let temp_scale = clamp(
        total_energy * lift::<FloatType>(TEMPERATURE_SHARE),
        lift(MIN_TEMPERATURE_MEV),
        lift(MAX_TEMPERATURE_MEV),
    );
    let one = lift::<FloatType>(1.0);
    let initial_temp: Vec<FloatType> = (0..TEMPERATURE_CELLS)
        .map(|i| {
            temp_scale
                * (one - lift_usize::<FloatType>(i) * lift::<FloatType>(TEMPERATURE_GRADIENT))
        })
        .collect();
    let temp_manifold = match model::make_1d_manifold(initial_temp.clone()) {
        Ok(m) => m,
        Err(e) => {
            println!("  [ERROR] mesh construction failed: {e:?}");
            return CausalEffectPropagationProcess::pure((hadron_count, temp_scale));
        }
    };

    let diffusivity = lift::<FloatType>(DIFFUSIVITY);
    let heat_result = heat_diffusion(&temp_manifold, diffusivity);

    // Use diffused result if valid, otherwise use initial average
    let avg_temp = match heat_result.value() {
        Some(final_temp) => {
            // Ten cells, so neither mean can refuse; dispatched rather than divided by a
            // literal, which silently decouples from the cell count if the grid changes.
            let zero = lift::<FloatType>(0.0);
            let avg = deep_causality_stats::mean(final_temp.data().as_slice()).unwrap_or(zero);
            if Real::abs(avg) > one {
                Real::abs(avg)
            } else {
                // Fallback: use initial temperature average
                deep_causality_stats::mean(&initial_temp).unwrap_or(zero)
            }
        }
        _ => temp_scale * lift::<FloatType>(COOLING_FACTOR),
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
    (hadron_count, avg_temp): (usize, FloatType),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(usize, FloatType, FloatType)> {
    println!("\nStage 5: Quantum Detection");
    println!("──────────────────────────");

    // Detection probability scales with temperature
    // At T_c ~ 170 MeV (QGP transition), detection is 50%
    // Higher temp → higher detection probability
    let t_critical = lift::<FloatType>(CRITICAL_TEMPERATURE_MEV);
    let zero = lift::<FloatType>(0.0);
    let one = lift::<FloatType>(1.0);
    let psi_val = clamp(
        avg_temp / (avg_temp + t_critical),
        lift(MIN_AMPLITUDE),
        lift(MAX_AMPLITUDE),
    );
    let psi = Complex::new(Real::sqrt(psi_val), zero);
    let psi_orth = Complex::new(Real::sqrt(one - psi_val), zero);

    let metric = Metric::Euclidean(1);
    let state = match HilbertState::<FloatType>::new(vec![psi, psi_orth], metric) {
        Ok(s) => s,
        Err(e) => {
            println!("  [ERROR] state construction failed: {e:?}");
            return CausalEffectPropagationProcess::pure((hadron_count, avg_temp, zero));
        }
    };
    let basis = match HilbertState::<FloatType>::new(
        vec![Complex::new(one, zero), Complex::new(zero, zero)],
        metric,
    ) {
        Ok(b) => b,
        Err(e) => {
            println!("  [ERROR] basis construction failed: {e:?}");
            return CausalEffectPropagationProcess::pure((hadron_count, avg_temp, zero));
        }
    };

    let detection = born_probability(&state, &basis);
    let prob = match detection.value() {
        Some(p) => *p,
        _ => zero,
    };

    println!("  Critical temp: T_c = {CRITICAL_TEMPERATURE_MEV} MeV");
    println!(
        "  |ψ⟩ = {:.3}|QGP⟩ + {:.3}|hadron⟩",
        lower(psi.re()),
        lower(psi_orth.re())
    );
    println!("  P(QGP detection) = {:.4}", lower(prob));

    CausalEffectPropagationProcess::pure((hadron_count, avg_temp, prob))
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints a sample of produced hadrons.
fn print_hadron_sample(hadrons: &[&Hadron<FloatType>]) {
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

/// Prints the summary banner shared by the success and failure paths.
fn print_summary_header() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");
}

/// Prints the final pipeline summary on success.
fn print_summary_ok((hadron_count, avg_temp, prob): (usize, FloatType, FloatType)) {
    print_summary_header();
    println!("  Hadron multiplicity:    {} particles", hadron_count);
    println!("  Thermal equilibrium:    {:.2} MeV", lower(avg_temp));
    println!("  Detection probability:  {:.4}", lower(prob));
    println!("\n[SUCCESS] Modular Pipeline Completed.\n");
}

/// Prints the final pipeline summary when a stage short-circuits the chain.
fn print_summary_err(err: &CausalityError) {
    print_summary_header();
    println!("  Pipeline failed: {err:?}");
    println!("\n[WARN] Check individual stage outputs.\n");
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
