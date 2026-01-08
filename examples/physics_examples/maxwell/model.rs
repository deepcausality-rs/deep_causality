/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_physics::MaxwellSolver;

/// Configuration for a plane wave in spacetime
#[derive(Clone, Debug, Default)]
pub struct PlaneWaveConfig {
    pub omega: f64,
    pub t: f64,
    pub z: f64,
}

/// State propagated through the causal chain
#[derive(Clone, Debug, Default)]
pub struct MaxwellState {
    pub phase: f64,
    pub potential_ax: f64,
    pub e_field: f64,
    pub b_field: f64,
    pub divergence: f64,
    pub poynting_flux: f64,
    pub gauge_satisfied: bool,
}

impl MaxwellState {
    pub fn from_config(config: &PlaneWaveConfig) -> Self {
        let phase = config.omega * (config.t - config.z);
        Self {
            phase,
            potential_ax: phase.cos(),
            ..Default::default()
        }
    }
}

/// Causaloid 1: Compute the Vector Potential A
///
/// A plane wave moving in Z-direction with linear polarization:
/// A = (0, A_x, 0, 0) * cos(ω(t - z))
pub fn compute_potential(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let phase = input.phase;
    let ax = phase.cos();

    PropagatingEffect::pure(MaxwellState {
        potential_ax: ax,
        ..input
    })
}

/// Causaloid 2: Derive the Electromagnetic Field F = ∇A
///
/// Uses Geometric Algebra: F = D * A (geometric product)
/// - Divergence (scalar) → Lorenz Gauge check
/// - Bivector components → E and B fields
pub fn compute_em_field(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let metric = Metric::Minkowski(4);
    let omega = 1.0_f64;
    let phase = input.phase;
    let potential_ax = input.potential_ax;

    // Construct A (the 4-Vector Potential) - Promoted to f64 for Solver
    let mut a_data: Vec<f64> = vec![0.0; 16];
    a_data[2] = potential_ax; // e_x component
    let potential_a = CausalMultiVector::new(a_data, metric).unwrap();

    // Analytically derived derivatives for A = cos(t-z) e_x
    let da_dt = -omega * phase.sin();
    let da_dz = omega * phase.sin();

    // Construct the Gradient Vector D
    let mut d_data = vec![0.0; 16];
    d_data[1] = da_dt; // e_t
    d_data[4] = da_dz; // e_z
    let gradient_d = CausalMultiVector::new(d_data, metric).unwrap();

    // Field Calculation via MaxwellSolver
    // 1. Calculate Divergence (Scalar L)
    let divergence = match MaxwellSolver::calculate_potential_divergence(&gradient_d, &potential_a)
    {
        Ok(d) => d,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!("Divergence calc failed: {:?}", e),
            )));
        }
    };

    // 2. Calculate Field Tensor (Bivector F)
    let f_result = match MaxwellSolver::calculate_field_tensor(&gradient_d, &potential_a) {
        Ok(f) => f,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!("Field calc failed: {:?}", e),
            )));
        }
    };

    // Extract components from Field Tensor
    let e_field = *f_result.get(3).unwrap_or(&0.0); // e_tx bivector
    let b_field = *f_result.get(6).unwrap_or(&0.0); // e_zx bivector

    PropagatingEffect::pure(MaxwellState {
        e_field,
        b_field,
        divergence,
        ..input
    })
}
/// Causaloid 3: Check Lorenz Gauge Condition
///
/// The Lorenz Gauge requires divergence ≈ 0 for gauge invariance.
pub fn check_lorenz_gauge(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let gauge_satisfied = input.divergence.abs() < 1e-9;

    PropagatingEffect::pure(MaxwellState {
        gauge_satisfied,
        ..input
    })
}

/// Causaloid 4: Compute Poynting Vector (Energy Flux)
///
/// S = E x B (using physics wrapper)
pub fn compute_poynting_flux(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let metric = Metric::Minkowski(4);

    // Reconstruct E and B vectors (simplified for example)
    // E is tx bivector (index 3), B is zx bivector (index 6) in previous step context
    // In dense GA, Electric field E is usually vector part of F (relative to an observer).
    // Here we treat E and B as independent vectors for the Poynting calculation example.

    // E vector along X (Promote to f64 for physics kernel)
    let mut e_data = vec![0.0; 16];
    e_data[2] = input.e_field; // x component
    let e_vec = CausalMultiVector::new(e_data, metric).unwrap();

    // B vector along Y (plane wave orthogonal)
    let mut b_data = vec![0.0; 16];
    b_data[3] = input.b_field; // y component
    let b_vec = CausalMultiVector::new(b_data, metric).unwrap();

    // Calculate poynting_vector using Solver
    match MaxwellSolver::calculate_poynting_flux(&e_vec, &b_vec) {
        Ok(s_field) => {
            let flux = s_field.squared_magnitude().sqrt();
            PropagatingEffect::pure(MaxwellState {
                poynting_flux: flux,
                ..input
            })
        }
        Err(e) => PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
            format!("Poynting Flux calc failed: {:?}", e),
        ))),
    }
}
