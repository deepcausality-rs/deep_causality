/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
    incompressible_ns_rhs_kernel, stokes_momentum_rhs_kernel,
};

const TOL: f64 = 1e-12;

#[test]
fn test_stokes_known_value() {
    // grad_p = (10, 0, 0); ρ = 2 → pressure = (-5, 0, 0)
    // ν = 0.5; lap = (4, 0, 0) → visc = (2, 0, 0)
    // body = (0, -9.81, 0)
    // RHS = (-3, -9.81, 0)
    let lap = [4.0_f64, 0.0, 0.0];
    let gp = [10.0_f64, 0.0, 0.0];
    let rho = Density::<f64>::new(2.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let out = stokes_momentum_rhs_kernel(&lap, &gp, &rho, &nu, &b)
        .unwrap()
        .into_inner();
    assert!((out[0] - (-3.0)).abs() < TOL);
    assert!((out[1] - (-9.81)).abs() < TOL);
    assert!(out[2].abs() < TOL);
}

#[test]
fn test_stokes_equals_incompressible_ns_at_zero_convection() {
    // Spec scenario: stokes ≡ incompressible_ns when u=0 and grad_u=0.
    let lap = [1.0_f64, 2.0, 3.0];
    let gp = [4.0_f64, 5.0, 6.0];
    let rho = Density::<f64>::new(2.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let b = AccelerationVector::<f64>::new([7.0, 8.0, 9.0]).unwrap();

    let stokes = stokes_momentum_rhs_kernel(&lap, &gp, &rho, &nu, &b)
        .unwrap()
        .into_inner();
    let ns = incompressible_ns_rhs_kernel(
        &Velocity3::<f64>::new([0.0; 3]).unwrap(),
        &VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap(),
        &lap,
        &gp,
        &rho,
        &nu,
        &b,
    )
    .unwrap()
    .into_inner();
    for i in 0..3 {
        assert!((stokes[i] - ns[i]).abs() < TOL);
    }
}

#[test]
fn test_stokes_zero_density_errors() {
    let lap = [0.0_f64; 3];
    let gp = [1.0_f64, 0.0, 0.0];
    let rho = Density::<f64>::new(0.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    assert!(stokes_momentum_rhs_kernel(&lap, &gp, &rho, &nu, &b).is_err());
}

#[test]
fn test_stokes_linear_in_inputs() {
    // Stokes is fully linear in (lap, grad_p, body_force) at fixed (ρ, ν).
    let rho = Density::<f64>::new(1.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let lap_a = [1.0_f64, 0.0, 0.0];
    let lap_b = [0.0_f64, 2.0, 0.0];
    let gp_a = [0.0_f64, 0.0, 3.0];
    let gp_b = [4.0_f64, 0.0, 0.0];
    let ba = AccelerationVector::<f64>::new([1.0, 1.0, 1.0]).unwrap();
    let bb = AccelerationVector::<f64>::new([-1.0, 2.0, 0.5]).unwrap();

    let ra = stokes_momentum_rhs_kernel(&lap_a, &gp_a, &rho, &nu, &ba)
        .unwrap()
        .into_inner();
    let rb = stokes_momentum_rhs_kernel(&lap_b, &gp_b, &rho, &nu, &bb)
        .unwrap()
        .into_inner();
    let lap_sum = [
        lap_a[0] + lap_b[0],
        lap_a[1] + lap_b[1],
        lap_a[2] + lap_b[2],
    ];
    let gp_sum = [gp_a[0] + gp_b[0], gp_a[1] + gp_b[1], gp_a[2] + gp_b[2]];
    let b_sum = AccelerationVector::<f64>::new([
        ba.value()[0] + bb.value()[0],
        ba.value()[1] + bb.value()[1],
        ba.value()[2] + bb.value()[2],
    ])
    .unwrap();
    let rsum = stokes_momentum_rhs_kernel(&lap_sum, &gp_sum, &rho, &nu, &b_sum)
        .unwrap()
        .into_inner();
    for i in 0..3 {
        assert!((rsum[i] - (ra[i] + rb[i])).abs() < TOL);
    }
}

#[test]
fn test_stokes_f32_sweep() {
    let lap = [0.5_f32, 0.5, 0.5];
    let gp = [1.0_f32, 1.0, 1.0];
    let rho = Density::<f32>::new(1.0).unwrap();
    let nu = KinematicViscosity::<f32>::new(0.01).unwrap();
    let b = AccelerationVector::<f32>::new([0.0, -9.81, 0.0]).unwrap();
    let out = stokes_momentum_rhs_kernel(&lap, &gp, &rho, &nu, &b)
        .unwrap()
        .into_inner();
    // x: -1 + 0.005 + 0 = -0.995
    // y: -1 + 0.005 + -9.81 = -10.805
    // z: -1 + 0.005 + 0 = -0.995
    assert!((out[0] - (-0.995_f32)).abs() < 1e-5);
    assert!((out[1] - (-10.805_f32)).abs() < 1e-4);
    assert!((out[2] - (-0.995_f32)).abs() < 1e-5);
}
