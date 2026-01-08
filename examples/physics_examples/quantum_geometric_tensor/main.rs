/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Quantum Geometric Tensor Example
//!
//! Demonstrates the Quantum Geometric Tensor (QGT) and its physical observables
//! in the context of twisted bilayer graphene (TBG) and flat-band systems.
//!
//! The QGT unifies:
//! - **Quantum Metric** (real part): "distance" between quantum states
//! - **Berry Curvature** (imaginary part): "magnetic field" in momentum space

use deep_causality_core::EffectValue;
use deep_causality_num::Complex;
use deep_causality_physics::{
    Energy, Length, PhysicsError, QuantumEigenvector, QuantumMetric, QuantumVelocity,
    effective_band_drude_weight, quantum_geometric_tensor,
};
use deep_causality_tensor::CausalTensor;

fn main() -> Result<(), PhysicsError> {
    println!("=== Quantum Geometric Tensor Analysis ===");
    println!("Application: Twisted Bilayer Graphene (TBG) Flat Bands\n");

    // =========================================================================
    // Setup: A minimal 2-band model (simplified for demonstration)
    // =========================================================================
    // In real applications, these would be computed from a tight-binding model
    // or DFT calculation.

    let num_bands = 2;
    let basis_size = 2; // Minimal basis

    // Energy eigenvalues at a k-point (in meV)
    // Band 0: flat band near zero, Band 1: remote band
    let eigenvalues_data = vec![1.0, 10.0]; // meV
    let eigenvalues = CausalTensor::new(eigenvalues_data, vec![num_bands])?;

    println!("Band Energies:");
    println!("  Band 0 (flat): {:.1} meV", eigenvalues.as_slice()[0]);
    println!("  Band 1 (remote): {:.1} meV\n", eigenvalues.as_slice()[1]);

    // Eigenvectors: Orthonormal basis (columns are eigenstates)
    // Complex values: [real, imag] pairs
    // |u_0> = (1, 0), |u_1> = (0, 1) (trivial case for demo)
    let eigenvector_data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0), // Column 0: |u_0>
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0), // Column 1: |u_1>
    ];
    let eigenvectors = QuantumEigenvector::new(CausalTensor::new(
        eigenvector_data,
        vec![basis_size, num_bands],
    )?);

    // Velocity matrices: v_x and v_y (momentum derivatives of Hamiltonian)
    // For TBG, these encode the Dirac cone structure
    // v_x|u_0> and v_x|u_1> stored as columns
    let vx_data = vec![
        Complex::new(0.0, 0.0),
        Complex::new(0.5, 0.3), // v_x|u_0>
        Complex::new(0.5, -0.3),
        Complex::new(0.0, 0.0), // v_x|u_1>
    ];
    let velocity_x = QuantumVelocity::new(CausalTensor::new(vx_data, vec![basis_size, num_bands])?);

    let vy_data = vec![
        Complex::new(0.0, 0.0),
        Complex::new(-0.3, 0.5), // v_y|u_0>
        Complex::new(-0.3, -0.5),
        Complex::new(0.0, 0.0), // v_y|u_1>
    ];
    let velocity_y = QuantumVelocity::new(CausalTensor::new(vy_data, vec![basis_size, num_bands])?);

    // =========================================================================
    // Calculate QGT for the flat band (Band 0)
    // =========================================================================
    println!("--- Quantum Geometric Tensor Q_ij for Band 0 ---");

    let regularization = 1e-6; // Small epsilon to avoid divergence at degeneracies

    // Q_xx component
    let qxx_effect = quantum_geometric_tensor(
        &eigenvalues,
        &eigenvectors,
        &velocity_x,
        &velocity_x,
        0, // Band 0
        regularization,
    );

    // Q_xy component (off-diagonal)
    let qxy_effect = quantum_geometric_tensor(
        &eigenvalues,
        &eigenvectors,
        &velocity_x,
        &velocity_y,
        0,
        regularization,
    );

    if let EffectValue::Value(qxx) = qxx_effect.value() {
        // Extract quantum metric (real part) and Berry curvature (imaginary part)
        println!("\nQ_xx = {:.6} + {:.6}i", qxx.re, qxx.im);
        println!("  → Quantum Metric g_xx = Re(Q_xx) = {:.6}", qxx.re);
        println!(
            "  → Berry Curvature Ω_xx = -2·Im(Q_xx) = {:.6}",
            -2.0 * qxx.im
        );
    }

    if let EffectValue::Value(qxy) = qxy_effect.value() {
        println!("\nQ_xy = {:.6} + {:.6}i", qxy.re, qxy.im);
        println!("  → Quantum Metric g_xy = Re(Q_xy) = {:.6}", qxy.re);
        println!(
            "  → Berry Curvature Ω_xy = -2·Im(Q_xy) = {:.6}",
            -2.0 * qxy.im
        );
    }

    // =========================================================================
    // Effective Band Drude Weight (Transport in Flat Bands)
    // =========================================================================
    println!("\n--- Effective Band Drude Weight ---");
    println!("Key insight: In flat bands, conventional transport vanishes but");
    println!("GEOMETRIC transport persists via the quantum metric!\n");

    // Parameters for TBG-like system
    let energy_flat = Energy::new(1e-3)?; // 1 meV (flat band)
    let energy_remote = Energy::new(10e-3)?; // 10 meV (remote band)
    let band_curvature = 0.0; // Flat band → zero curvature (conventional transport = 0)
    let quantum_metric = QuantumMetric::new(0.5)?; // Significant geometric contribution
    let lattice_const = Length::new(0.246e-9)?; // Graphene lattice constant (nm)

    let drude_effect = effective_band_drude_weight(
        energy_flat,
        energy_remote,
        band_curvature,
        quantum_metric,
        lattice_const,
    );

    if let EffectValue::Value(drude_weight) = drude_effect.value() {
        println!(
            "Band Drude Weight D = {:.4e} eV·nm²",
            drude_weight.value() * 1e18
        );
        println!("\nPhysical interpretation:");
        println!("  • D > 0 means coherent transport is possible");
        println!("  • Even with zero curvature (flat band), the quantum metric");
        println!("    provides a 'geometric lower bound' on conductivity");
        println!("  • This explains metallic behavior in magic-angle TBG!");
    }

    // =========================================================================
    // Summary of QGT-derived observables
    // =========================================================================
    println!("\n=== QGT-Derived Physical Observables ===");
    println!("┌─────────────────────────────┬────────────────────────────────────┐");
    println!("│ Observable                   │ QGT Relation                       │");
    println!("├─────────────────────────────┼────────────────────────────────────┤");
    println!("│ Quantum Metric g_ij          │ Re(Q_ij) - state distance          │");
    println!("│ Berry Curvature Ω_ij         │ -2·Im(Q_ij) - anomalous velocity   │");
    println!("│ Band Drude Weight D          │ D_conv + g·ΔE - geometric transport│");
    println!("│ Orbital Magnetization        │ ∫ Ω·f(E) dk - magnetic moment      │");
    println!("└─────────────────────────────┴────────────────────────────────────┘");

    println!("\n=== Simulation Complete ===");

    Ok(())
}
