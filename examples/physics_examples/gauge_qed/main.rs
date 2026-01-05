/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QED Electromagnetic Wave Pipeline
//!
//! Demonstrates **modular causal composition** via `CausalEffectPropagationProcess`
//! for QED (Quantum Electrodynamics) electromagnetic wave propagation.
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
//! - **Real QED calculations** using deep_causality_physics

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_physics::{QED, QedOps};

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  QED Electromagnetic Wave Pipeline");
    println!("═══════════════════════════════════════════════════════════════\n");

    // =========================================================================
    // The Causal Monad Pipeline: Each stage is a decoupled function
    // =========================================================================

    // Stage 0: Create initial plane wave field
    let initial = stage_create_plane_wave();

    // Composed pipeline using bind_or_error
    let result = initial
        .bind_or_error(stage_compute_invariants, "Invariant computation failed")
        .bind_or_error(stage_energy_analysis, "Energy analysis failed")
        .bind_or_error(stage_poynting_radiation, "Radiation analysis failed")
        .bind_or_error(stage_field_classification, "Field classification failed");

    // Extract and display final result
    print_summary(&result);

    Ok(())
}

// =============================================================================
// QED State: Passed through pipeline stages
// =============================================================================

/// Accumulated results from pipeline stages
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct QEDState {
    /// The QED field configuration
    qed: Option<QED>,
    /// Field invariant F_μν F^μν
    field_invariant: f64,
    /// Dual invariant F_μν F̃^μν
    dual_invariant: f64,
    /// Energy density u = (E² + B²)/2
    energy_density: f64,
    /// Lagrangian density L = (E² - B²)/2
    lagrangian_density: f64,
    /// Intensity |S|
    intensity: f64,
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
fn stage_create_plane_wave() -> PropagatingEffect<Option<QED>> {
    println!("Stage 1: Create Plane Wave Field");
    println!("─────────────────────────────────");

    // Create a plane wave with E along x, B along y
    let amplitude = 1.0; // Natural units
    let polarization = 0; // x-polarization

    match QED::plane_wave(amplitude, polarization) {
        Ok(qed) => {
            println!("  Amplitude:     {} (natural units)", amplitude);
            println!("  Polarization:  x-polarized");
            println!("  E-field:       along x-axis");
            println!("  B-field:       along y-axis");
            println!("  Propagation:   z-direction\n");

            CausalEffectPropagationProcess::pure(Some(qed))
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
    qed_opt: Option<QED>,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(Option<QED>, f64, f64)> {
    println!("Stage 2: Compute Lorentz Invariants");
    println!("────────────────────────────────────");

    if let Some(qed) = &qed_opt {
        let field_inv = qed.field_invariant().unwrap_or(0.0);
        let dual_inv = qed.dual_invariant().unwrap_or(0.0);

        println!("  F_μν F^μν  = {:.6}  (field invariant)", field_inv);
        println!("  F_μν F̃^μν = {:.6}  (dual invariant)", dual_inv);

        // Physical interpretation
        if field_inv.abs() < 1e-10 {
            println!("\n  → |E| = |B| (null field / radiation)");
        } else if field_inv > 0.0 {
            println!("\n  → Magnetic-dominated field");
        } else {
            println!("\n  → Electric-dominated field");
        }

        if dual_inv.abs() < 1e-10 {
            println!("  → E ⟂ B (CP-conserving)");
        } else {
            println!("  → E·B ≠ 0 (CP-violating configuration)");
        }
        println!();

        CausalEffectPropagationProcess::pure((qed_opt, field_inv, dual_inv))
    } else {
        CausalEffectPropagationProcess::pure((None, 0.0, 0.0))
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
    (qed_opt, field_inv, dual_inv): (Option<QED>, f64, f64),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(Option<QED>, f64, f64, f64, f64)> {
    println!("Stage 3: Energy Analysis");
    println!("────────────────────────");

    if let Some(qed) = &qed_opt {
        let energy = qed.energy_density().unwrap_or(0.0);
        let lagrangian = qed.lagrangian_density().unwrap_or(0.0);

        println!("  Energy density:     u = {:.6} (natural units)", energy);
        println!(
            "  Lagrangian density: L = {:.6} (natural units)",
            lagrangian
        );

        // Convert to SI for context (assuming E ~ 1 V/m scale)
        let epsilon_0 = 8.854e-12; // F/m
        let energy_si = energy * epsilon_0; // J/m³
        println!("\n  In SI units (assuming E ~ 1 V/m scale):");
        println!("  u ≈ {:.3e} J/m³", energy_si);
        println!();

        CausalEffectPropagationProcess::pure((qed_opt, field_inv, dual_inv, energy, lagrangian))
    } else {
        CausalEffectPropagationProcess::pure((None, field_inv, dual_inv, 0.0, 0.0))
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
    (qed_opt, field_inv, dual_inv, energy, lagrangian): (Option<QED>, f64, f64, f64, f64),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<(Option<QED>, f64, f64, f64, f64, f64)> {
    println!("Stage 4: Radiation Analysis");
    println!("───────────────────────────");

    if let Some(qed) = &qed_opt {
        let intensity = qed.intensity().unwrap_or(0.0);

        println!("  Intensity: |S| = {:.6} (natural units)", intensity);

        // Poynting vector direction
        if let Ok(s) = qed.poynting_vector() {
            let s_data = s.data();
            // In 3D GA, bivector components are indices 4,5,6 (xy, xz, yz)
            // But data() returns &[f64].
            // If length is enough:
            if s_data.len() >= 7 {
                println!(
                    "  S_xy = {:.4}, S_xz = {:.4}, S_yz = {:.4}",
                    s_data[4], s_data[5], s_data[6]
                );
            }
        }

        // Radiation pressure
        let radiation_pressure = intensity; // P = I/c, but c=1 in natural units
        println!("\n  Radiation pressure: P = {:.6}", radiation_pressure);
        println!();

        CausalEffectPropagationProcess::pure((
            qed_opt, field_inv, dual_inv, energy, lagrangian, intensity,
        ))
    } else {
        CausalEffectPropagationProcess::pure((None, field_inv, dual_inv, energy, lagrangian, 0.0))
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
    (qed_opt, field_inv, dual_inv, energy, lagrangian, intensity): (
        Option<QED>,
        f64,
        f64,
        f64,
        f64,
        f64,
    ),
    _: (),
    _: Option<()>,
) -> PropagatingEffect<QEDState> {
    println!("Stage 5: Field Classification");
    println!("─────────────────────────────");

    let (is_radiation, is_null) = if let Some(qed) = &qed_opt {
        let is_rad = qed.is_radiation_field().unwrap_or(false);
        let is_nul = qed.is_null_field().unwrap_or(false);
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
    } else if field_inv > 0.0 {
        "Magnetic-Dominated Static Field"
    } else if field_inv < 0.0 {
        "Electric-Dominated Static Field"
    } else {
        "General EM Superposition"
    };

    println!("\n  Classification: {}", field_type);
    println!();

    let state = QEDState {
        qed: qed_opt,
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
fn print_summary(result: &PropagatingEffect<QEDState>) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");

    match result.value() {
        EffectValue::Value(state) => {
            println!("\n  ┌─────────────────────────────────────────────────────────┐");
            println!("  │  Lorentz Invariants                                     │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  F_μν F^μν  = {:>12.6}                              │",
                state.field_invariant
            );
            println!(
                "  │  F_μν F̃^μν = {:>12.6}                               │",
                state.dual_invariant
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  Physical Quantities                                    │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Energy density (u):     {:>12.6}                   │",
                state.energy_density
            );
            println!(
                "  │  Lagrangian density (L): {:>12.6}                   │",
                state.lagrangian_density
            );
            println!(
                "  │  Intensity (|S|):        {:>12.6}                   │",
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
            println!("\n[SUCCESS] QED Pipeline Completed.\n");
        }
        _ => {
            println!("  Pipeline returned unexpected result");
            println!("\n[WARN] Check individual stage outputs.\n");
        }
    }
}
