/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, Metric};

/// Configuration for a plane wave in spacetime
#[derive(Clone, Debug, Default)]
pub struct PlaneWaveConfig {
    pub omega: f32,
    pub t: f32,
    pub z: f32,
}

/// State propagated through the causal chain
#[derive(Clone, Debug, Default)]
pub struct MaxwellState {
    pub phase: f32,
    pub potential_ax: f32,
    pub e_field: f32,
    pub b_field: f32,
    pub divergence: f32,
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
    let omega = 1.0_f32;
    let phase = input.phase;

    // Construct A (the 4-Vector Potential)
    let mut a_data: Vec<f32> = vec![0.0; 16];
    a_data[2] = input.potential_ax; // e_x component

    let potential_a = match CausalMultiVector::new(a_data, metric) {
        Ok(mv) => mv,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!("Failed to create potential: {:?}", e),
            )));
        }
    };

    // Analytically derived derivatives for A = cos(t-z) e_x
    let da_dt = -omega * phase.sin();
    let da_dz = omega * phase.sin();

    // Construct the Gradient Vector D
    let mut d_data = vec![0.0; 16];
    d_data[1] = da_dt; // e_t
    d_data[4] = da_dz; // e_z

    let gradient_d = match CausalMultiVector::new(d_data, metric) {
        Ok(mv) => mv,
        Err(e) => {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!("Failed to create gradient: {:?}", e),
            )));
        }
    };

    // F = D * A (Geometric Product)
    let field_f = gradient_d.geometric_product(&potential_a);

    // Extract components
    let divergence = *field_f.get(0).unwrap_or(&0.0);
    let e_field = *field_f.get(3).unwrap_or(&0.0); // e_tx bivector
    let b_field = *field_f.get(6).unwrap_or(&0.0); // e_zx bivector

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
