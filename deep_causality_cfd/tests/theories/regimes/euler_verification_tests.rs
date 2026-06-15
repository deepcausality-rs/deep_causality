/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reference-solution verification for the Euler regime evaluator.
//!
//! These tests check `euler_momentum_rhs_kernel` against textbook analytical
//! solutions of the inviscid Euler equations. Each case has an *external*
//! reference; the kernel is correct iff it reproduces the textbook RHS at
//! the sample point.
//!
//! References:
//!   - Landau & Lifshitz, "Fluid Mechanics" (2nd ed., 1987), §3
//!     "The equation of motion of an ideal fluid".
//!   - Batchelor, "An Introduction to Fluid Dynamics" (1967), §1.4
//!     "Equations of motion".
//!   - Kundu, Cohen & Dowling, "Fluid Mechanics" (6th ed., 2016), Ch. 4.

use deep_causality_cfd::euler_momentum_rhs_kernel;
use deep_causality_physics::{AccelerationVector, Density, Velocity3, VelocityGradient};

const TOL: f64 = 1e-12;

// =============================================================================
// Verification 1: Hydrostatic equilibrium (Landau & Lifshitz §3)
//
// Static fluid in a gravitational field. Reference solution: at equilibrium
// the pressure gradient exactly balances gravity, ∇p = ρg, so the Eulerian
// acceleration vanishes pointwise: ∂u/∂t = 0.
//
// Setup: u = 0, grad_u = 0, g = (0, 0, −g₀) downward, ∇p = (0, 0, −ρ·g₀).
//   RHS = −(u·∇)u − (1/ρ) ∇p + g
//       =  0       − (0, 0, −g₀)       + (0, 0, −g₀)
//       =  0  ✓
// =============================================================================

#[test]
fn test_euler_hydrostatic_equilibrium() {
    let rho = Density::<f64>::new(1.225).unwrap(); // sea-level air, kg/m³
    let g0 = 9.80665_f64;
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let grad_u = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    // Hydrostatic pressure gradient: ∇p = ρ g, with g pointing in −z.
    let grad_p = [0.0_f64, 0.0, -rho.value() * g0];
    let body = AccelerationVector::<f64>::new([0.0, 0.0, -g0]).unwrap();

    let rhs = euler_momentum_rhs_kernel(&u, &grad_u, &grad_p, &rho, &body)
        .unwrap()
        .into_inner();
    assert!(
        rhs[0].abs() < TOL,
        "hydrostatic RHS_x must be 0, got {}",
        rhs[0]
    );
    assert!(
        rhs[1].abs() < TOL,
        "hydrostatic RHS_y must be 0, got {}",
        rhs[1]
    );
    assert!(
        rhs[2].abs() < TOL,
        "hydrostatic RHS_z must be 0, got {}",
        rhs[2]
    );
}

// =============================================================================
// Verification 2: Uniform inviscid flow with no pressure gradient
//                 (Batchelor §1.4, "trivial" exact solution)
//
// Constant-velocity inviscid flow in absence of external forcing is a steady
// solution. Reference: ∂u/∂t = 0 identically.
//
// Setup: u = (U, 0, 0), grad_u = 0, ∇p = 0, body = 0.
//   RHS = 0 ✓
// =============================================================================

#[test]
fn test_euler_uniform_flow_is_stationary() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Velocity3::<f64>::new([5.0, 0.0, 0.0]).unwrap();
    let grad_u = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let grad_p = [0.0_f64; 3];
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = euler_momentum_rhs_kernel(&u, &grad_u, &grad_p, &rho, &body)
        .unwrap()
        .into_inner();
    for &c in &rhs {
        assert!(c.abs() < TOL);
    }
}

// =============================================================================
// Verification 3: Steady Bernoulli flow along a stagnation streamline
//                 (Kundu, Cohen & Dowling §4.9; Batchelor §6.4)
//
// For steady inviscid flow along a streamline, p + 0.5 ρ |u|² + ρ g z = const.
// Differentiating along x, with a 1D acceleration u_x du/dx and a pressure
// gradient dp/dx = −ρ u_x du/dx, the Euler RHS along that streamline must
// vanish (the flow is steady).
//
// Setup at a point on a streamline: u = (U, 0, 0), du/dx = a (so the
// convective acceleration is U·a in x), grad_p = (−ρ·U·a, 0, 0), body = 0.
//   RHS_x = −U·a − (1/ρ)·(−ρ·U·a) + 0 = −U·a + U·a = 0 ✓
// =============================================================================

#[test]
fn test_euler_steady_bernoulli_streamline_is_stationary() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u_speed = 10.0_f64;
    let accel = 2.5_f64; // du/dx along the streamline (1/s)
    let u = Velocity3::<f64>::new([u_speed, 0.0, 0.0]).unwrap();
    let grad_u =
        VelocityGradient::<f64>::new([[accel, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])
            .unwrap();
    // Bernoulli: dp/dx = −ρ u du/dx along the streamline.
    let grad_p = [-rho.value() * u_speed * accel, 0.0, 0.0];
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = euler_momentum_rhs_kernel(&u, &grad_u, &grad_p, &rho, &body)
        .unwrap()
        .into_inner();
    for (i, &c) in rhs.iter().enumerate() {
        assert!(
            c.abs() < TOL,
            "steady Bernoulli RHS[{i}] must be 0, got {c}"
        );
    }
}
