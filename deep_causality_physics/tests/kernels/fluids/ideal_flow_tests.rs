/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, Length, Pressure, Speed, Velocity3, bernoulli_total_head_kernel, circulation_kernel,
    dynamic_pressure_kernel, kutta_joukowski_lift_kernel, stream_function_2d_kernel,
    velocity_potential_2d_kernel,
};

const TOL: f64 = 1e-10;

// =============================================================================
// dynamic_pressure_kernel
// =============================================================================

#[test]
fn test_dynamic_pressure_known_value() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u = Speed::<f64>::new(20.0).unwrap();
    // q = 0.5 * 1.225 * 400 = 245
    let q = dynamic_pressure_kernel(&rho, &u).unwrap();
    assert!((q.value() - 245.0).abs() < TOL);
}

#[test]
fn test_dynamic_pressure_scales_quadratically_with_speed() {
    // Spec scenario: q(ρ, k·u) = k²·q(ρ, u).
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_ref = Speed::<f64>::new(2.0).unwrap();
    let q_ref = dynamic_pressure_kernel(&rho, &u_ref).unwrap();
    for k in [0.5_f64, 1.0, 2.0, 5.0] {
        let u_k = Speed::<f64>::new(2.0 * k).unwrap();
        let q_k = dynamic_pressure_kernel(&rho, &u_k).unwrap();
        assert!((q_k.value() - k * k * q_ref.value()).abs() < TOL);
    }
}

#[test]
fn test_dynamic_pressure_zero_for_zero_velocity() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u = Speed::<f64>::new(0.0).unwrap();
    assert_eq!(dynamic_pressure_kernel(&rho, &u).unwrap().value(), 0.0);
}

// =============================================================================
// bernoulli_total_head_kernel
// =============================================================================

#[test]
fn test_bernoulli_head_known_value() {
    // p = 0, ρ = 1000 kg/m³, u = √(2g·5) = sqrt(98.0665) ≈ 9.903, h = 0:
    //   H = 0 + u²/(2g) + 0 = 98.0665/(2·9.80665) = 5.0
    let p = Pressure::<f64>::new(0.0).unwrap();
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u_val = (2.0 * 9.80665 * 5.0_f64).sqrt();
    let u = Speed::<f64>::new(u_val).unwrap();
    let h = Length::<f64>::new(0.0).unwrap();
    let head = bernoulli_total_head_kernel(&p, &rho, &u, &h).unwrap();
    assert!((head.value() - 5.0).abs() < 1e-6);
}

#[test]
fn test_bernoulli_head_errors_on_zero_density() {
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    let rho = Density::<f64>::new(0.0).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    let h = Length::<f64>::new(0.0).unwrap();
    assert!(bernoulli_total_head_kernel(&p, &rho, &u, &h).is_err());
}

#[test]
fn test_bernoulli_head_static_term_only() {
    // u = 0, h = 0: H = p / (ρ·g).
    let p = Pressure::<f64>::new(98066.5).unwrap();
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(0.0).unwrap();
    let h = Length::<f64>::new(0.0).unwrap();
    // H = 98066.5 / (1000 · 9.80665) = 10.0
    let head = bernoulli_total_head_kernel(&p, &rho, &u, &h).unwrap();
    assert!((head.value() - 10.0).abs() < 1e-6);
}

// =============================================================================
// stream_function_2d_kernel + velocity_potential_2d_kernel
// =============================================================================

#[test]
fn test_stream_function_known_value() {
    // u = 1, v = 0 (uniform flow in x). dψ = u·dy − v·dx = dy.
    assert_eq!(stream_function_2d_kernel(1.0_f64, 0.0, 0.5, 0.3), 0.3);
    assert_eq!(stream_function_2d_kernel(0.0_f64, 2.0, 0.5, 0.0), -1.0);
}

#[test]
fn test_velocity_potential_known_value() {
    // dφ = u·dx + v·dy.
    assert_eq!(velocity_potential_2d_kernel(1.0_f64, 0.0, 0.5, 0.3), 0.5);
    assert_eq!(velocity_potential_2d_kernel(2.0_f64, 3.0, 1.0, 1.0), 5.0);
}

// =============================================================================
// circulation_kernel
// =============================================================================

#[test]
fn test_circulation_uniform_flow_closed_loop_is_zero() {
    // Uniform horizontal flow u = (1, 0, 0). A closed loop returns Γ = 0 by
    // Stokes' theorem (no vorticity). Use a square loop: 4 edges that sum to 0.
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let velocities = vec![u; 4];
    let tangents: Vec<[f64; 3]> = vec![
        [1.0, 0.0, 0.0],  // right
        [0.0, 1.0, 0.0],  // up
        [-1.0, 0.0, 0.0], // left
        [0.0, -1.0, 0.0], // down
    ];
    let gamma = circulation_kernel(&velocities, &tangents).unwrap();
    assert!(gamma.abs() < TOL);
}

#[test]
fn test_circulation_pure_rotation_picks_up_vorticity() {
    // Rigid-body rotation u = Ω × r. On a unit-radius circle approximated by
    // four points the discrete circulation ≈ ω·perimeter.
    // u at (1, 0): (0, ω, 0); at (0, 1): (-ω, 0, 0); at (-1, 0): (0, -ω, 0);
    // at (0, -1): (ω, 0, 0). With tangent vectors along the loop.
    let omega = 0.5_f64;
    let velocities = vec![
        Velocity3::<f64>::new([0.0, omega, 0.0]).unwrap(),
        Velocity3::<f64>::new([-omega, 0.0, 0.0]).unwrap(),
        Velocity3::<f64>::new([0.0, -omega, 0.0]).unwrap(),
        Velocity3::<f64>::new([omega, 0.0, 0.0]).unwrap(),
    ];
    let tangents: Vec<[f64; 3]> = vec![
        [0.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
    ];
    let gamma = circulation_kernel(&velocities, &tangents).unwrap();
    // Each segment contributes ω · 1 = ω. 4 segments => 4ω = 2.0.
    assert!((gamma - 2.0).abs() < TOL);
}

#[test]
fn test_circulation_errors_on_length_mismatch() {
    let velocities = vec![Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap(); 3];
    let tangents: Vec<[f64; 3]> = vec![[1.0, 0.0, 0.0]; 2];
    assert!(circulation_kernel(&velocities, &tangents).is_err());
}

#[test]
fn test_circulation_empty_loop_is_zero() {
    let velocities: Vec<Velocity3<f64>> = vec![];
    let tangents: Vec<[f64; 3]> = vec![];
    let gamma = circulation_kernel(&velocities, &tangents).unwrap();
    assert_eq!(gamma, 0.0);
}

// =============================================================================
// kutta_joukowski_lift_kernel  (spec scenario)
// =============================================================================

#[test]
fn test_kutta_joukowski_zero_circulation_zero_lift() {
    // Spec scenario: lift = 0 when Γ = 0 (algebraic, not tolerance-bound).
    let rho = Density::<f64>::new(1.225).unwrap();
    let u_inf = Speed::<f64>::new(50.0).unwrap();
    assert_eq!(kutta_joukowski_lift_kernel(&rho, &u_inf, 0.0), 0.0);
}

#[test]
fn test_kutta_joukowski_known_value() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u_inf = Speed::<f64>::new(50.0).unwrap();
    // Γ = 10 ⇒ L' = 1.225 · 50 · 10 = 612.5
    let lift = kutta_joukowski_lift_kernel(&rho, &u_inf, 10.0);
    assert!((lift - 612.5).abs() < TOL);
}

#[test]
fn test_kutta_joukowski_sign_follows_circulation() {
    let rho = Density::<f64>::new(1.225).unwrap();
    let u_inf = Speed::<f64>::new(50.0).unwrap();
    let lift_pos = kutta_joukowski_lift_kernel(&rho, &u_inf, 10.0);
    let lift_neg = kutta_joukowski_lift_kernel(&rho, &u_inf, -10.0);
    assert!(lift_pos > 0.0);
    assert!(lift_neg < 0.0);
    assert!((lift_pos + lift_neg).abs() < TOL);
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_dynamic_pressure_quadratic_scaling_f32() {
    let rho = Density::<f32>::new(1.225).unwrap();
    let u_ref = Speed::<f32>::new(2.0).unwrap();
    let q_ref = dynamic_pressure_kernel(&rho, &u_ref).unwrap();
    let u_2 = Speed::<f32>::new(4.0).unwrap();
    let q_2 = dynamic_pressure_kernel(&rho, &u_2).unwrap();
    assert!((q_2.value() - 4.0 * q_ref.value()).abs() < 1e-4 * q_2.value());
}
