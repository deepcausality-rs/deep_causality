/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, Velocity3, VelocityGradient,
    euler_momentum_rhs_kernel, incompressible_ns_rhs_kernel,
};

const TOL: f64 = 1e-12;

#[test]
fn test_euler_known_value() {
    // u = (1, 0, 0); grad_u diag(2, 3, 5) → conv = (2, 0, 0)
    // grad_p = (10, 0, 0); ρ = 2 → pressure_force = (-5, 0, 0)
    // body = (0, -9.81, 0)
    // RHS = (-2 + -5 + 0, 0 + 0 + -9.81, 0) = (-7, -9.81, 0)
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let gp = [10.0_f64, 0.0, 0.0];
    let rho = Density::<f64>::new(2.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, -9.81, 0.0]).unwrap();
    let out = euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b)
        .unwrap()
        .into_inner();
    assert!((out[0] - (-7.0)).abs() < TOL);
    assert!((out[1] - (-9.81)).abs() < TOL);
    assert!(out[2].abs() < TOL);
}

#[test]
fn test_euler_equals_incompressible_ns_at_zero_viscous_term() {
    // Spec scenario: euler ≡ incompressible_ns when ν=0 and ∇²u=0.
    let u = Velocity3::<f64>::new([2.0, 1.0, 0.5]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]]).unwrap();
    let gp = [1.0_f64, 2.0, 3.0];
    let rho = Density::<f64>::new(1.5).unwrap();
    let b = AccelerationVector::<f64>::new([0.0, 0.0, -9.81]).unwrap();

    let euler = euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b)
        .unwrap()
        .into_inner();
    let ns = incompressible_ns_rhs_kernel(
        &u,
        &g,
        &[0.0; 3],
        &gp,
        &rho,
        &KinematicViscosity::<f64>::new(0.0).unwrap(),
        &b,
    )
    .unwrap()
    .into_inner();
    for i in 0..3 {
        assert!((euler[i] - ns[i]).abs() < TOL);
    }
}

#[test]
fn test_euler_zero_density_errors() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0; 3]; 3]).unwrap();
    let gp = [1.0_f64, 0.0, 0.0];
    let rho = Density::<f64>::new(0.0).unwrap();
    let b = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    assert!(euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b).is_err());
}

#[test]
fn test_euler_body_force_linearity() {
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[0.1, 0.2, 0.0], [0.0, 0.3, 0.4], [0.5, 0.0, 0.6]]).unwrap();
    let gp = [10.0_f64, 20.0, 30.0];
    let rho = Density::<f64>::new(1.0).unwrap();
    let b0 = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let rhs0 = euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b0)
        .unwrap()
        .into_inner();
    let delta = [1.0_f64, -2.0, 3.0];
    let b = AccelerationVector::<f64>::new(delta).unwrap();
    let rhs = euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b)
        .unwrap()
        .into_inner();
    for i in 0..3 {
        assert!((rhs[i] - rhs0[i] - delta[i]).abs() < TOL);
    }
}

#[test]
fn test_euler_f32_sweep() {
    let u = Velocity3::<f32>::new([1.0, 2.0, 3.0]).unwrap();
    let g = VelocityGradient::<f32>::new([[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]])
        .unwrap();
    let gp = [1.0_f32, 1.0, 1.0];
    let rho = Density::<f32>::new(1.0).unwrap();
    let b = AccelerationVector::<f32>::new([0.0, -9.81, 0.0]).unwrap();
    let out = euler_momentum_rhs_kernel(&u, &g, &gp, &rho, &b)
        .unwrap()
        .into_inner();
    // x: -(1·0.1) + (-1) + 0 = -1.1
    // y: -(2·0.2) + (-1) + (-9.81) = -11.21
    // z: -(3·0.3) + (-1) + 0 = -1.9
    assert!((out[0] - (-1.1_f32)).abs() < 1e-5);
    assert!((out[1] - (-11.21_f32)).abs() < 1e-4);
    assert!((out[2] - (-1.9_f32)).abs() < 1e-5);
}
