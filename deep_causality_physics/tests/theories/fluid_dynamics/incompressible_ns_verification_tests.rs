/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reference-solution verification for the incompressible Navier-Stokes
//! regime evaluator.
//!
//! Each test cites an external reference for the setup and the expected RHS.
//!
//! References:
//!   - G. I. Taylor, "On the decay of vortices in a viscous fluid",
//!     Philosophical Magazine, Series 6, Vol. 46, No. 274 (1923), pp. 671–674.
//!     [The original Taylor-Green vortex paper.]
//!   - G. I. Taylor & A. E. Green, "Mechanism of the production of small
//!     eddies from large ones", Proc. Roy. Soc. London A, Vol. 158
//!     (1937), pp. 499–521.
//!   - S. B. Pope, "Turbulent Flows" (Cambridge UP, 2000), §6.1.5.
//!   - Brachet et al., "Small-scale structure of the Taylor–Green vortex",
//!     J. Fluid Mech. 130 (1983), pp. 411–452. [Standard reference for the
//!     2D form of the velocity / pressure fields used below.]

use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
    incompressible_ns_rhs_kernel,
};

// =============================================================================
// Verification: 2D Taylor-Green vortex at t = 0
//
// Source: Taylor (1923), Phil. Mag. 46(274); Taylor & Green (1937),
//         Proc. Roy. Soc. A 158, pp. 499–521.
//         Closed-form fields as quoted in Pope (2000) §6.1.5 and
//         Brachet et al. (1983):
//
//   u(x,y,t) =   cos(x) sin(y) · exp(−2 ν t)
//   v(x,y,t) = − sin(x) cos(y) · exp(−2 ν t)
//   w        = 0
//   p(x,y,t) = − (ρ/4) [cos(2x) + cos(2y)] · exp(−4 ν t)
//
// This is an *exact* solution of the 2D incompressible NS equations. Its
// defining property: each velocity component decays as a single exponential
//
//   ∂u/∂t |_{t=0} = −2 ν · u(x,y,0)
//   ∂v/∂t |_{t=0} = −2 ν · v(x,y,0)
//
// At the sample point (x, y) = (π/4, π/4):
//   u =  cos(π/4) sin(π/4) =  0.5
//   v = −sin(π/4) cos(π/4) = −0.5
//   ∂u/∂x = −sin(π/4) sin(π/4) = −0.5
//   ∂u/∂y =  cos(π/4) cos(π/4) =  0.5
//   ∂v/∂x = −cos(π/4) cos(π/4) = −0.5
//   ∂v/∂y =  sin(π/4) sin(π/4) =  0.5
//   ∇²u   = −2 u = −1.0    (Laplacian eigenfunction)
//   ∇²v   = −2 v =  1.0
//   ∂p/∂x = ρ/4 · 2 sin(2·π/4) = 0.5 ρ
//   ∂p/∂y = ρ/4 · 2 sin(2·π/4) = 0.5 ρ
//
// Reference values (ν = 0.1, ρ = 1):
//   ∂u/∂t = −2 ν u = −0.1
//   ∂v/∂t = −2 ν v =  0.1
// =============================================================================

#[test]
fn test_incompressible_ns_taylor_green_vortex_decay() {
    let nu_val = 0.1_f64;
    let rho_val = 1.0_f64;

    // u, v at (π/4, π/4):
    let half = 0.5_f64;
    let u = Velocity3::<f64>::new([half, -half, 0.0]).unwrap();
    // grad_u: rows = i, cols = j, [i][j] = ∂u_i/∂x_j
    let grad_u = VelocityGradient::<f64>::new([
        [-half, half, 0.0], // ∂u/∂x, ∂u/∂y, ∂u/∂z
        [-half, half, 0.0], // ∂v/∂x, ∂v/∂y, ∂v/∂z
        [0.0, 0.0, 0.0],
    ])
    .unwrap();
    // ∇²u = (−2u, −2v, 0) = (−1, 1, 0)
    let laplacian_u = [-1.0_f64, 1.0, 0.0];
    // ∇p = (ρ/4 · 2 sin(π/2), ρ/4 · 2 sin(π/2), 0) = (ρ/2, ρ/2, 0)
    let grad_p = [rho_val * 0.5, rho_val * 0.5, 0.0];

    let rho = Density::<f64>::new(rho_val).unwrap();
    let nu = KinematicViscosity::<f64>::new(nu_val).unwrap();
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = incompressible_ns_rhs_kernel(&u, &grad_u, &laplacian_u, &grad_p, &rho, &nu, &body)
        .unwrap()
        .into_inner();

    // Reference (Taylor–Green decay): ∂u/∂t = −2ν·u.
    let expected_dudt = -2.0 * nu_val * half; // = −0.1
    let expected_dvdt = -2.0 * nu_val * (-half); // =  0.1
    let expected_dwdt = 0.0_f64;

    let tol = 1e-12_f64;
    assert!(
        (rhs[0] - expected_dudt).abs() < tol,
        "Taylor–Green ∂u/∂t mismatch: got {}, expected {}",
        rhs[0],
        expected_dudt
    );
    assert!(
        (rhs[1] - expected_dvdt).abs() < tol,
        "Taylor–Green ∂v/∂t mismatch: got {}, expected {}",
        rhs[1],
        expected_dvdt
    );
    assert!((rhs[2] - expected_dwdt).abs() < tol);
}

// =============================================================================
// Verification: Plane Poiseuille flow is also a steady solution of the
//               *full* incompressible NS equations (not just Stokes).
//
// Source: Batchelor (1967), §4.2. The convective term `(u·∇)u` vanishes
// identically for unidirectional fully-developed flow because u depends
// only on the cross-stream coordinate, so the velocity is constant along
// streamlines. Thus the full NS RHS reduces to the Stokes RHS, which is
// zero (see Batchelor Eq. 4.2.5–4.2.6). This is a stronger check than
// the Stokes-regime test because it also exercises the convective kernel.
// =============================================================================

#[test]
fn test_incompressible_ns_plane_poiseuille_steady_state() {
    let rho_val = 1000.0_f64;
    let mu = 1.0e-3_f64;
    let nu_val = mu / rho_val;
    let g_press = 100.0_f64; // G = −dp/dx

    // At the centerline: u_x = G h²/(2μ) (irrelevant for RHS — convective
    // term vanishes because ∂u_x/∂x = ∂u_x/∂z = 0 and u_y = u_z = 0; the
    // only nonzero entry ∂u_x/∂y multiplies u_y = 0). Choose u_x = 1 for
    // concreteness.
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let grad_u =
        VelocityGradient::<f64>::new([[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let lap = [-g_press / mu, 0.0_f64, 0.0]; // Batchelor 4.2.5
    let grad_p = [-g_press, 0.0_f64, 0.0];

    let rho = Density::<f64>::new(rho_val).unwrap();
    let nu = KinematicViscosity::<f64>::new(nu_val).unwrap();
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = incompressible_ns_rhs_kernel(&u, &grad_u, &lap, &grad_p, &rho, &nu, &body)
        .unwrap()
        .into_inner();
    // Reference: steady-state ⇒ RHS = 0. Loosened tolerance to absorb the
    // cancellation between viscous and pressure terms (~1e5 in magnitude).
    for &c in &rhs {
        assert!(c.abs() < 1e-9);
    }
}
