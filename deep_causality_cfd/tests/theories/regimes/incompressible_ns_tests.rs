/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::incompressible_ns_rhs_kernel;
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
};

const TOL: f64 = 1e-12;

fn vel(v: [f64; 3]) -> Velocity3<f64> {
    Velocity3::<f64>::new(v).unwrap()
}

fn grad(g: [[f64; 3]; 3]) -> VelocityGradient<f64> {
    VelocityGradient::<f64>::new(g).unwrap()
}

fn rho(r: f64) -> Density<f64> {
    Density::<f64>::new(r).unwrap()
}

fn nu(v: f64) -> KinematicViscosity<f64> {
    KinematicViscosity::<f64>::new(v).unwrap()
}

fn acc(a: [f64; 3]) -> AccelerationVector<f64> {
    AccelerationVector::<f64>::new(a).unwrap()
}

// =============================================================================
// Known-value sanity
// =============================================================================

#[test]
fn test_incompressible_ns_known_value() {
    // u = (1, 0, 0); grad_u = 0 → conv = 0
    // laplacian_u = (0, 0, 0); ν = 0 → viscous = 0
    // grad_p = (10, 0, 0); ρ = 2 → pressure = (-5, 0, 0)
    // body_force_per_mass = (0, -9.81, 0)
    // RHS = (-5, -9.81, 0)
    let u = vel([1.0, 0.0, 0.0]);
    let g = grad([[0.0; 3]; 3]);
    let lap = [0.0_f64; 3];
    let gp = [10.0_f64, 0.0, 0.0];
    let r = rho(2.0);
    let n = nu(0.0);
    let b = acc([0.0, -9.81, 0.0]);
    let out = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();
    assert!((out[0] - (-5.0)).abs() < TOL);
    assert!((out[1] - (-9.81)).abs() < TOL);
    assert!(out[2].abs() < TOL);
}

// =============================================================================
// Inviscid limit: ν = 0 ⇒ result matches Euler form (no viscous term)
// =============================================================================

#[test]
fn test_inviscid_limit_drops_viscous_term() {
    let u = vel([2.0, 1.0, 0.0]);
    let g = grad([[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]]);
    let lap = [5.0_f64, 7.0, 11.0]; // arbitrary, must not affect result
    let gp = [1.0_f64, 2.0, 3.0];
    let r = rho(1.0);
    let n_zero = nu(0.0);
    let n_big = nu(13.0);
    let b = acc([0.0, 0.0, 0.0]);

    let inviscid = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n_zero, &b)
        .unwrap()
        .into_inner();
    let viscous = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n_big, &b)
        .unwrap()
        .into_inner();
    // Difference equals ν · laplacian_u exactly.
    for i in 0..3 {
        assert!((viscous[i] - inviscid[i] - 13.0 * lap[i]).abs() < TOL);
    }
}

// =============================================================================
// Creeping-flow limit: u = 0, grad_u = 0 ⇒ Stokes form
// =============================================================================

#[test]
fn test_creeping_flow_limit_drops_convective_term() {
    let u_zero = vel([0.0, 0.0, 0.0]);
    let u_nonzero = vel([3.0, 4.0, 5.0]);
    let g_zero = grad([[0.0; 3]; 3]);
    let lap = [1.0_f64, 2.0, 3.0];
    let gp = [4.0_f64, 5.0, 6.0];
    let r = rho(2.0);
    let n = nu(0.5);
    let b = acc([7.0, 8.0, 9.0]);

    let stokes = incompressible_ns_rhs_kernel(&u_zero, &g_zero, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();
    // At u = 0 and grad_u = 0, swapping u to anything keeps stokes result
    // since conv = (u · grad_u) — but grad_u is zero so conv = 0 either way.
    let stokes_alt = incompressible_ns_rhs_kernel(&u_nonzero, &g_zero, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();
    for i in 0..3 {
        assert!((stokes[i] - stokes_alt[i]).abs() < TOL);
    }
    // Expected: -(1/ρ)·grad_p + ν·lap + g
    let expected = [
        -gp[0] / 2.0 + 0.5 * lap[0] + b.value()[0],
        -gp[1] / 2.0 + 0.5 * lap[1] + b.value()[1],
        -gp[2] / 2.0 + 0.5 * lap[2] + b.value()[2],
    ];
    for i in 0..3 {
        assert!((stokes[i] - expected[i]).abs() < TOL);
    }
}

// =============================================================================
// Body-force linearity (property)
// =============================================================================

#[test]
fn test_body_force_linearity_property() {
    let u = vel([1.0, 2.0, 3.0]);
    let g = grad([[0.1, 0.2, 0.0], [0.0, 0.3, 0.4], [0.5, 0.0, 0.6]]);
    let lap = [0.1_f64, 0.2, 0.3];
    let gp = [10.0_f64, 20.0, 30.0];
    let r = rho(1.5);
    let n = nu(0.01);

    let b0 = acc([0.0, 0.0, 0.0]);
    let rhs0 = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b0)
        .unwrap()
        .into_inner();
    for delta in [[1.0, 0.0, 0.0], [0.0, 5.0, 0.0], [-1.0, 2.0, -3.0]] {
        let b = acc(delta);
        let rhs = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b)
            .unwrap()
            .into_inner();
        for i in 0..3 {
            assert!((rhs[i] - rhs0[i] - delta[i]).abs() < TOL);
        }
    }
}

// =============================================================================
// Velocity-offset shift: shifting u → u + c shifts conv by grad_u·c
// (analog of the Group-3 linearity-in-velocity-offset property)
// =============================================================================

#[test]
fn test_velocity_offset_shifts_rhs_by_minus_grad_u_dot_c() {
    let u = vel([1.0, 0.5, 0.0]);
    let gmat = [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6], [0.7, 0.8, 0.9]];
    let g = grad(gmat);
    let lap = [0.0_f64; 3];
    let gp = [1.0_f64, 1.0, 1.0];
    let r = rho(1.0);
    let n = nu(0.0);
    let b = acc([0.0, 0.0, 0.0]);

    let rhs0 = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();

    let c = [2.0_f64, -1.0, 3.0];
    let u_shift = vel([
        u.value()[0] + c[0],
        u.value()[1] + c[1],
        u.value()[2] + c[2],
    ]);
    let rhs_shift = incompressible_ns_rhs_kernel(&u_shift, &g, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();

    // conv shifts by grad_u · c, so RHS shifts by −grad_u · c.
    let shift = [
        -(gmat[0][0] * c[0] + gmat[0][1] * c[1] + gmat[0][2] * c[2]),
        -(gmat[1][0] * c[0] + gmat[1][1] * c[1] + gmat[1][2] * c[2]),
        -(gmat[2][0] * c[0] + gmat[2][1] * c[1] + gmat[2][2] * c[2]),
    ];
    for i in 0..3 {
        assert!((rhs_shift[i] - rhs0[i] - shift[i]).abs() < TOL);
    }
}

// =============================================================================
// Error path: ρ = 0
// =============================================================================

#[test]
fn test_zero_density_errors() {
    let u = vel([1.0, 0.0, 0.0]);
    let g = grad([[0.0; 3]; 3]);
    let lap = [0.0_f64; 3];
    let gp = [1.0_f64, 0.0, 0.0];
    let r = rho(0.0);
    let n = nu(0.0);
    let b = acc([0.0; 3]);
    assert!(incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b).is_err());
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_incompressible_ns_f32_sweep() {
    let u = Velocity3::<f32>::new([1.0, 2.0, 3.0]).unwrap();
    let g =
        VelocityGradient::<f32>::new([[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]]).unwrap();
    let lap = [0.5_f32, 0.5, 0.5];
    let gp = [1.0_f32, 1.0, 1.0];
    let r = Density::<f32>::new(1.0).unwrap();
    let n = KinematicViscosity::<f32>::new(0.01).unwrap();
    let b = AccelerationVector::<f32>::new([0.0, -9.81, 0.0]).unwrap();
    let out = incompressible_ns_rhs_kernel(&u, &g, &lap, &gp, &r, &n, &b)
        .unwrap()
        .into_inner();
    // x: -(1·0.1) + (-1) + 0.005 + 0 = -1.095
    // y: -(2·0.2) + (-1) + 0.005 + (-9.81) = -11.205
    // z: -(3·0.3) + (-1) + 0.005 + 0 = -1.895
    assert!((out[0] - (-1.095_f32)).abs() < 1e-5);
    assert!((out[1] - (-11.205_f32)).abs() < 1e-4);
    assert!((out[2] - (-1.895_f32)).abs() < 1e-5);
}
