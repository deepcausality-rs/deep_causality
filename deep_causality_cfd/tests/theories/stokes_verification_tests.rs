/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reference-solution verification for the Stokes regime evaluator.
//!
//! Each test cites an external reference for the setup and the expected RHS.
//!
//! References:
//!   - Batchelor, "An Introduction to Fluid Dynamics" (1967), §4.2
//!     "Steady unidirectional flow", equations (4.2.5)–(4.2.6) for plane
//!     Poiseuille flow.
//!   - Kundu, Cohen & Dowling, "Fluid Mechanics" (6th ed., 2016), §9.3
//!     "Steady flow between parallel plates".
//!   - Landau & Lifshitz, "Fluid Mechanics" (2nd ed., 1987), §17 "Flow in a
//!     pipe", Eq. (17.4) for Hagen–Poiseuille pipe flow.

use deep_causality_cfd::stokes_momentum_rhs;
use deep_causality_physics::{AccelerationVector, Density, KinematicViscosity};

const TOL: f64 = 1e-12;

// =============================================================================
// Verification 1: Plane Poiseuille flow
//   Source: Batchelor (1967), §4.2 Eq. (4.2.6)
//           Kundu, Cohen & Dowling (2016), §9.3 Eq. (9.10)
//
// Pressure-driven steady flow between two stationary parallel plates at
// y = ±h. Closed-form solution:
//     u_x(y) = (G / (2 μ)) (h² − y²),     G ≡ −dp/dx > 0
//     u_y = u_z = 0
// At steady state, the Stokes momentum RHS must vanish.
//
// At the centerline y = 0, the textbook gives:
//     ∂²u_x/∂y² = −G/μ        (Batchelor 4.2.5 differentiated twice)
//     u_x(0)    =  G h² / (2 μ)   (max velocity, Batchelor 4.2.6)
// Reference RHS:
//   RHS_x = −(1/ρ)·(−G) + ν·(−G/μ) = G/ρ − (μ/ρ)·G/μ = G/ρ − G/ρ = 0 ✓
// =============================================================================

#[test]
fn test_stokes_plane_poiseuille_steady_state() {
    // Pick representative values for water: ρ = 1000 kg/m³, μ = 1e-3 Pa·s
    // ⇒ ν = μ/ρ = 1e-6 m²/s. Channel half-height h = 0.01 m, G = 100 Pa/m.
    let rho_val = 1000.0_f64;
    let mu = 1.0e-3_f64;
    let nu_val = mu / rho_val;
    let pressure_drop_per_length = 100.0_f64; // G = −dp/dx
    // ∇²u_x at the centerline (Batchelor Eq. 4.2.5):
    let lap_u_x = -pressure_drop_per_length / mu;
    let lap = [lap_u_x, 0.0_f64, 0.0];
    // ∇p = (−G, 0, 0):
    let grad_p = [-pressure_drop_per_length, 0.0_f64, 0.0];

    let rho = Density::<f64>::new(rho_val).unwrap();
    let nu = KinematicViscosity::<f64>::new(nu_val).unwrap();
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = stokes_momentum_rhs(&lap, &grad_p, &rho, &nu, &body)
        .unwrap()
        .into_inner();
    // Reference: ∂u/∂t = 0 (steady state). Tolerance is loosened to absorb
    // the cancellation of two ~1e5 terms in f64.
    for (i, &c) in rhs.iter().enumerate() {
        assert!(
            c.abs() < 1e-9,
            "plane Poiseuille steady-state RHS[{i}] must be 0, got {c}"
        );
    }
}

// =============================================================================
// Verification 2: Plane Couette flow
//   Source: Batchelor (1967), §4.2 (preceding the Poiseuille derivation);
//           Kundu, Cohen & Dowling (2016), §9.2.
//
// Steady linear shear between parallel plates at y = 0 and y = h with the
// top plate moving at U. Closed-form: u_x(y) = U·y/h. Both ∇²u and ∇p are
// identically zero, so the Stokes RHS must vanish.
// =============================================================================

#[test]
fn test_stokes_plane_couette_steady_state() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    // ∇²u = 0 everywhere (u_x is linear in y), ∇p = 0.
    let lap = [0.0_f64; 3];
    let grad_p = [0.0_f64; 3];
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = stokes_momentum_rhs(&lap, &grad_p, &rho, &nu, &body)
        .unwrap()
        .into_inner();
    for &c in &rhs {
        assert!(c.abs() < TOL);
    }
}

// =============================================================================
// Verification 3: Hagen–Poiseuille pipe flow
//   Source: Landau & Lifshitz, "Fluid Mechanics" (1987), §17 Eq. (17.4).
//
// Steady axisymmetric flow through a circular pipe of radius R driven by
// pressure gradient G ≡ −dp/dz > 0. Closed-form:
//     u_z(r) = (G / (4 μ)) (R² − r²)
// On the axis (r = 0) the Laplacian in cylindrical coords reduces to
//     ∇²u_z = (1/r) d/dr (r du_z/dr) |_{r→0} = −G/μ
// (the same magnitude as the planar case but derived in cylindrical
// coordinates; see L&L Eq. (17.3)). Steady state ⇒ RHS = 0.
// =============================================================================

#[test]
fn test_stokes_hagen_poiseuille_pipe_centerline_steady_state() {
    let rho_val = 1000.0_f64;
    let mu = 1.0e-3_f64;
    let nu_val = mu / rho_val;
    let pressure_drop_per_length = 50.0_f64; // G = −dp/dz

    let lap = [0.0_f64, 0.0, -pressure_drop_per_length / mu];
    let grad_p = [0.0_f64, 0.0, -pressure_drop_per_length];
    let rho = Density::<f64>::new(rho_val).unwrap();
    let nu = KinematicViscosity::<f64>::new(nu_val).unwrap();
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = stokes_momentum_rhs(&lap, &grad_p, &rho, &nu, &body)
        .unwrap()
        .into_inner();
    for &c in &rhs {
        assert!(c.abs() < 1e-9);
    }
}
