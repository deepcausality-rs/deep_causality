/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Gauge EM Electromagnetic Wave Pipeline
//!
//! Demonstrates **modular causal composition** via `CausalEffectPropagationProcess`
//! for gauge-theoretic electromagnetic wave analysis.
//!
//! ## Key Design Pattern
//!
//! Each physics stage is a **standalone function** composed via `bind_or_error`:
//!
//! ```ignore
//! create_plane_wave()
//!     .bind_or_error(stage_compute_invariants, ...)     // Modular: Compute Lorentz invariants
//!     .bind_or_error(stage_energy_analysis, ...)        // Modular: Energy & momentum
//!     .bind_or_error(stage_poynting_radiation, ...)     // Modular: Radiation properties
//!     .bind_or_error(stage_field_classification, ...)   // Modular: Field type
//! ```
//!
//! This demonstrates:
//! - **Type-safe error propagation** through physics stages
//! - **Decoupled physics modules** that compose seamlessly
//! - **Classical EM via gauge field formalism** using deep_causality_physics

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_num::{DoubleFloat, Float, Zero};
use deep_causality_physics::{EM, GaugeEmOps};

// =============================================================================
// FLOAT TYPE CONFIGURATION
// =============================================================================

// Change this to f32 or DoubleFloat to use different precision
type FloatType = DoubleFloat;
type EmTheory = EM<FloatType>;

/// Macro to convert f64 literals to FloatType
macro_rules! flt {
    ($x:expr) => {
        <FloatType as From<f64>>::from($x)
    };
}

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("       Gauge EM: Relativistic Electrodynamics Pipeline");
    println!("  (Float Type: {})", std::any::type_name::<FloatType>());
    println!("═══════════════════════════════════════════════════════════════\n");

    // Composed pipeline: Each stage is a decoupled function
    let result = stage_create_plane_wave()
        .bind_or_error(stage_compute_invariants, "Invariant computation failed")
        .bind_or_error(stage_energy_analysis, "Energy analysis failed")
        .bind_or_error(stage_poynting_radiation, "Radiation analysis failed")
        .bind_or_error(stage_field_classification, "Field classification failed");

    // Extract and display final result
    print_summary(&result);

    Ok(())
}

// =============================================================================
// GaugeEM State: Passed through pipeline stages
// =============================================================================

/// Accumulated results from pipeline stages
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct GaugeEMState {
    /// The GaugeEM field configuration
    field: Option<EmTheory>,
    /// Field invariant F_μν F^μν
    field_invariant: FloatType,
    /// Dual invariant F_μν F̃^μν
    dual_invariant: FloatType,
    /// Energy density u = (E² + B²)/2
    energy_density: FloatType,
    /// Lagrangian density L = (E² - B²)/2
    lagrangian_density: FloatType,
    /// Intensity |S|
    intensity: FloatType,
    /// Is radiation field (E ⟂ B)
    is_radiation: bool,
    /// Is null field (|E| = |B|)
    is_null: bool,
}

// =============================================================================
// STAGE 0: Create Plane Wave
// =============================================================================

/// Creates an initial plane wave electromagnetic field.
///
/// # Physics
/// - Plane wave: E and B perpendicular, equal magnitude
/// - Propagating in z-direction
fn stage_create_plane_wave() -> PropagatingEffect<Option<EmTheory>> {
    println!("Stage 1: Create Plane Wave Field");
    println!("─────────────────────────────────");

    // Create a plane wave with E along x, B along y
    let amplitude = flt!(1.0); // Natural units
    let polarization = 0; // x-polarization

    match EmTheory::plane_wave(amplitude, polarization) {
        Ok(em) => {
            println!("  Amplitude:     {} (natural units)", amplitude);
            println!("  Polarization:  x-polarized");
            println!("  E-field:       along x-axis");
            println!("  B-field:       along y-axis");
            println!("  Propagation:   z-direction\n");

            // GaugeEM Gauge Theory Properties
            println!("  ┌─ U(1) Gauge Field Structure ─────────────────────────┐");
            println!(
                "  │  Gauge Group:      {} (Electromagnetism)",
                em.gauge_group_name()
            );
            println!(
                "  │  Lie Algebra:      u(1), dim = {}",
                em.lie_algebra_dim()
            );
            println!(
                "  │  Abelian:          {} (F = dA, no self-interaction)",
                em.is_abelian()
            );
            println!(
                "  │  Metric:           {} (+--- West Coast / Particle Physics)",
                if em.is_west_coast() {
                    "Minkowski 4D"
                } else {
                    "Custom"
                }
            );
            println!("  │  Spacetime:        {} dimensions", em.spacetime_dim());

            // Field strength tensor info
            if let Ok(f) = em.computed_field_strength() {
                let shape = f.shape();
                println!("  │  F_μν shape:       {:?}", shape);
                println!("  │  Connection:       A_μ (4-potential)");
                println!("  │  Curvature:        F_μν = ∂_μA_ν - ∂_νA_μ");
            }
            println!("  └────────────────────────────────────────────────────────┘\n");

            CausalEffectPropagationProcess::pure(Some(em))
        }
        Err(e) => {
            println!("  [ERROR] Failed to create plane wave: {:?}", e);
            // Return default zero field
            CausalEffectPropagationProcess::pure(None)
        }
    }
}

// =============================================================================
// STAGE 1: Compute Lorentz Invariants
// =============================================================================

/// Computes the Lorentz-invariant scalars of the electromagnetic field.
///
/// # Physics
/// - F_μν F^μν = 2(B² - E²) — same in all reference frames
/// - F_μν F̃^μν = -4 E·B — measures CP violation
///
/// # Maintenance
/// - These invariants characterize the field type
/// - Add more invariants here (e.g., stress-energy trace)
fn stage_compute_invariants(
    em_opt: Option<EmTheory>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(Option<EmTheory>, FloatType, FloatType)> {
    println!("Stage 2: Compute Lorentz Invariants");
    println!("────────────────────────────────────");

    if let Some(em) = &em_opt {
        let field_inv = em.field_invariant().unwrap_or(FloatType::zero());
        let dual_inv = em.dual_invariant().unwrap_or(FloatType::zero());

        println!("  F_μν F^μν  = {}  (field invariant)", field_inv);
        println!("  F_μν F̃^μν = {}  (dual invariant)", dual_inv);

        // Physical interpretation
        if s_abs(field_inv) < flt!(1e-10) {
            println!("\n  → |E| = |B| (null field / radiation)");
        } else if field_inv > flt!(0.0) {
            println!("\n  → Magnetic-dominated field");
        } else {
            println!("\n  → Electric-dominated field");
        }

        if s_abs(dual_inv) < flt!(1e-10) {
            println!("  → E ⟂ B (CP-conserving)");
        } else {
            println!("  → E·B ≠ 0 (CP-violating configuration)");
        }
        println!();

        CausalEffectPropagationProcess::pure((em_opt, field_inv, dual_inv))
    } else {
        CausalEffectPropagationProcess::pure((None, flt!(0.0), flt!(0.0)))
    }
}

// =============================================================================
// STAGE 2: Energy Analysis
// =============================================================================

/// Analyzes the energy and Lagrangian density of the field.
///
/// # Physics
/// - Energy density: u = (E² + B²)/2 = T^{00}
/// - Lagrangian: L = (E² - B²)/2 = -¼F_μν F^μν
///
/// # Maintenance
/// - Modify energy scale conversions here
/// - Add momentum density computation
fn stage_energy_analysis(
    (em_opt, field_inv, dual_inv): (Option<EmTheory>, FloatType, FloatType),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(Option<EmTheory>, FloatType, FloatType, FloatType, FloatType)> {
    println!("Stage 3: Energy Analysis");
    println!("────────────────────────");

    if let Some(em) = &em_opt {
        let energy = em.energy_density().unwrap_or(FloatType::zero());
        let lagrangian = em.lagrangian_density().unwrap_or(FloatType::zero());

        println!("  Energy density:     u = {} (natural units)", energy);
        println!("  Lagrangian density: L = {} (natural units)", lagrangian);

        // Convert to SI for context (assuming E ~ 1 V/m scale)
        let epsilon_0 = flt!(8.854e-12); // F/m
        let energy_val = energy;
        let energy_si = energy_val * epsilon_0; // J/m³
        println!("\n  In SI units (assuming E ~ 1 V/m scale):");
        println!("  u ≈ {:.3e} J/m³", energy_si);
        println!();

        CausalEffectPropagationProcess::pure((em_opt, field_inv, dual_inv, energy, lagrangian))
    } else {
        CausalEffectPropagationProcess::pure((None, field_inv, dual_inv, flt!(0.0), flt!(0.0)))
    }
}

// =============================================================================
// STAGE 3: Poynting/Radiation Analysis
// =============================================================================

/// Analyzes the radiation properties of the electromagnetic field.
///
/// # Physics
/// - Poynting vector: S = E × B (energy flux)
/// - Intensity: |S| (power per unit area)
///
/// # Maintenance
/// - Add radiation pressure computation
/// - Add momentum flux analysis
fn stage_poynting_radiation(
    (em_opt, field_inv, dual_inv, energy, lagrangian): (
        Option<EmTheory>,
        FloatType,
        FloatType,
        FloatType,
        FloatType,
    ),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(
    Option<EmTheory>,
    FloatType,
    FloatType,
    FloatType,
    FloatType,
    FloatType,
)> {
    println!("Stage 4: Radiation Analysis");
    println!("───────────────────────────");

    if let Some(em) = &em_opt {
        let intensity = em.intensity().unwrap_or(FloatType::zero());

        println!("  Intensity: |S| = {} (natural units)", intensity);

        // Poynting vector direction
        if let Ok(s) = em.poynting_vector() {
            let s_data = s.data();
            // Poynting vector S = E × B is a 3D vector stored at indices 2, 3, 4 (x, y, z)
            if s_data.len() >= 5 {
                let sx = s_data.get(2).copied().unwrap_or(FloatType::zero());
                let sy = s_data.get(3).copied().unwrap_or(FloatType::zero());
                let sz = s_data.get(4).copied().unwrap_or(FloatType::zero());
                println!("  S_x = {:.4}, S_y = {:.4}, S_z = {:.4}", sx, sy, sz);
            }
        }

        // Radiation pressure
        let radiation_pressure = intensity; // P = I/c, but c=1 in natural units
        println!("\n  Radiation pressure: P = {}", radiation_pressure);
        println!();

        CausalEffectPropagationProcess::pure((
            em_opt, field_inv, dual_inv, energy, lagrangian, intensity,
        ))
    } else {
        CausalEffectPropagationProcess::pure((
            None,
            field_inv,
            dual_inv,
            energy,
            lagrangian,
            flt!(0.0),
        ))
    }
}

// =============================================================================
// STAGE 4: Field Classification
// =============================================================================

/// Classifies the electromagnetic field type.
///
/// # Physics
/// - Radiation: E ⟂ B and |E| = |B|
/// - Electric-dominated: |E| > |B|
/// - Magnetic-dominated: |B| > |E|
///
/// # Maintenance
/// - Add more field classifications (near-field, etc.)
/// - Add wave type detection
fn stage_field_classification(
    (em_opt, field_inv, dual_inv, energy, lagrangian, intensity): (
        Option<EmTheory>,
        FloatType,
        FloatType,
        FloatType,
        FloatType,
        FloatType,
    ),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<GaugeEMState> {
    println!("Stage 5: Field Classification");
    println!("─────────────────────────────");

    let (is_radiation, is_null) = if let Some(em) = &em_opt {
        let is_rad = em.is_radiation_field().unwrap_or(false);
        let is_nul = em.is_null_field().unwrap_or(false);
        (is_rad, is_nul)
    } else {
        (false, false)
    };

    println!("  Is radiation field (E ⟂ B):  {}", is_radiation);
    println!("  Is null field (|E| = |B|):   {}", is_null);

    // Classification
    let field_type = if is_radiation && is_null {
        "Transverse Electromagnetic (TEM) Wave"
    } else if is_radiation {
        "Elliptically Polarized Wave"
    } else if is_null {
        "Null Field (non-radiative)"
    } else if field_inv > flt!(0.0) {
        "Magnetic-Dominated Static Field"
    } else if field_inv < flt!(0.0) {
        "Electric-Dominated Static Field"
    } else {
        "General EM Superposition"
    };

    println!("\n  Classification: {}", field_type);
    println!();

    let state = GaugeEMState {
        field: em_opt,
        field_invariant: field_inv,
        dual_invariant: dual_inv,
        energy_density: energy,
        lagrangian_density: lagrangian,
        intensity,
        is_radiation,
        is_null,
    };

    CausalEffectPropagationProcess::pure(state)
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints the final pipeline summary.
fn print_summary(result: &PropagatingEffect<GaugeEMState>) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");

    match result.value() {
        EffectValue::Value(state) => {
            println!("\n  ┌─────────────────────────────────────────────────────────┐");
            println!("  │  Lorentz Invariants                                     │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  F_μν F^μν  = {}                                         │",
                state.field_invariant
            );
            println!(
                "  │  F_μν F̃^μν = {}                                         │",
                state.dual_invariant
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  Physical Quantities                                    │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Energy density (u):     {}                              │",
                state.energy_density
            );
            println!(
                "  │  Lagrangian density (L): {}                              │",
                state.lagrangian_density
            );
            println!(
                "  │  Intensity (|S|):        {}                              │",
                state.intensity
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  Classification                                         │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Radiation field: {}                                   │",
                if state.is_radiation { "Yes" } else { "No " }
            );
            println!(
                "  │  Null field:      {}                                   │",
                if state.is_null { "Yes" } else { "No " }
            );
            println!("  └─────────────────────────────────────────────────────────┘");
            println!("\n[SUCCESS] GaugeEM Pipeline Completed.\n");
        }
        _ => {
            println!("  Pipeline returned unexpected result");
            println!("\n[WARN] Check individual stage outputs.\n");
        }
    }
}

// Helper for abs
fn s_abs(x: FloatType) -> FloatType {
    <FloatType as Float>::abs(x)
}
