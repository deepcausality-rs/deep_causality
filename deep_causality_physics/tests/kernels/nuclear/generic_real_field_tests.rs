/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Generic real-field coverage for the nuclear kernels.
//!
//! The per-kernel suites (`qcd_tests`, `physics_tests`, …) pin `f64` and assert
//! exact numerics. This suite proves the same kernels are genuinely generic over
//! `R: RealField` by instantiating each at **both `f32` and `f64`**, and checks
//! that `LundParameters` and the `speed_of_light` accessor cast their `f64`
//! definitions into the target precision via `real_from_f64`. Tolerances are
//! loose so a single generic body is valid at both precisions.

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_physics::{
    AmountOfSubstance, Energy, HalfLife, LundParameters, Mass, SPEED_OF_LIGHT, Time,
    all_structure_constants, binding_energy_kernel, confinement_potential_kernel,
    covariant_derivative_kernel, gell_mann_matrices, radioactive_decay_kernel, real_from_f64,
    running_coupling_kernel, speed_of_light, structure_constant, wilson_loop_kernel,
};

// ---------------------------------------------------------------------------
// Constant accessor
// ---------------------------------------------------------------------------

fn check_speed_of_light<R: RealField + FromPrimitive + core::fmt::Debug>() {
    assert_eq!(speed_of_light::<R>(), real_from_f64::<R>(SPEED_OF_LIGHT));
    assert!(speed_of_light::<R>() > R::zero());
}

#[test]
fn speed_of_light_f32() {
    check_speed_of_light::<f32>();
}

#[test]
fn speed_of_light_f64() {
    check_speed_of_light::<f64>();
}

// ---------------------------------------------------------------------------
// QCD: SU(3) generators and structure constants
// ---------------------------------------------------------------------------

fn run_qcd_constants<R: RealField + FromPrimitive + core::fmt::Debug>() {
    let zero = R::zero();
    let one = R::one();
    let tol = real_from_f64::<R>(1.0e-4);

    // λ_3 = diag(1, -1, 0).
    let matrices = gell_mann_matrices::<R>();
    assert_eq!(matrices.len(), 8);
    let l3 = matrices[2];
    assert!((l3[0] - one).abs() < tol);
    assert!((l3[4] + one).abs() < tol);
    assert!((l3[8] - zero).abs() < tol);

    // f^{123} = 1 and antisymmetry f^{123} = -f^{213}.
    assert!((structure_constant::<R>(1, 2, 3) - one).abs() < tol);
    assert!((structure_constant::<R>(1, 2, 3) + structure_constant::<R>(2, 1, 3)).abs() < tol);
    assert_eq!(structure_constant::<R>(1, 1, 1), zero);

    assert_eq!(all_structure_constants::<R>().len(), 9);
}

#[test]
fn qcd_constants_f32() {
    run_qcd_constants::<f32>();
}

#[test]
fn qcd_constants_f64() {
    run_qcd_constants::<f64>();
}

// ---------------------------------------------------------------------------
// QCD: covariant derivative
// ---------------------------------------------------------------------------

fn run_covariant_derivative<R: RealField + FromPrimitive>() {
    let zero = R::zero();
    let mut psi = vec![zero; 6];
    psi[0] = R::one(); // color-0 real part
    let psi_gradient = vec![zero; 24];
    let mut gluon_field = vec![zero; 32];
    gluon_field[2] = real_from_f64::<R>(0.5); // some λ_3 contribution
    let coupling = real_from_f64::<R>(1.2);

    let res = covariant_derivative_kernel::<R>(&psi, &psi_gradient, &gluon_field, coupling);
    assert!(res.is_ok());
    let d = res.unwrap();
    assert_eq!(d.len(), 24);
    assert!(d.iter().all(|v| v.is_finite()));
}

#[test]
fn covariant_derivative_f32() {
    run_covariant_derivative::<f32>();
}

#[test]
fn covariant_derivative_f64() {
    run_covariant_derivative::<f64>();
}

// ---------------------------------------------------------------------------
// QCD: Wilson loop, confinement potential, running coupling
// ---------------------------------------------------------------------------

fn run_qcd_observables<R: RealField + FromPrimitive>() {
    let tol = real_from_f64::<R>(1.0e-3);

    // Wilson loop on a single segment.
    let gluon = vec![real_from_f64::<R>(0.1); 8];
    let lengths = vec![real_from_f64::<R>(0.2)];
    let coupling = real_from_f64::<R>(1.0);
    let w = wilson_loop_kernel::<R>(&gluon, &lengths, coupling).unwrap();
    assert!(w.is_finite() && w > R::zero() && w <= real_from_f64::<R>(3.0));

    // Confinement potential V = σ r (no Coulomb term).
    let sigma = real_from_f64::<R>(0.18);
    let r = real_from_f64::<R>(2.0);
    let v = confinement_potential_kernel::<R>(r, sigma, None).unwrap();
    assert!((v - sigma * r).abs() < tol);

    // Running coupling at Q² = 100 GeV², Λ = 0.2, n_f = 5.
    let alpha = running_coupling_kernel::<R>(real_from_f64::<R>(100.0), real_from_f64::<R>(0.2), 5)
        .unwrap();
    assert!(alpha.is_finite() && alpha > R::zero());
}

#[test]
fn qcd_observables_f32() {
    run_qcd_observables::<f32>();
}

#[test]
fn qcd_observables_f64() {
    run_qcd_observables::<f64>();
}

// ---------------------------------------------------------------------------
// LundParameters: generic config with f64 tune cast into R
// ---------------------------------------------------------------------------

fn run_lund_parameters<R: RealField + FromPrimitive>() {
    let p = LundParameters::<R>::default();
    let tol = real_from_f64::<R>(1.0e-4);
    // Monash-tune defaults, cast into R.
    assert!((p.kappa() - real_from_f64::<R>(1.0)).abs() < tol);
    assert!((p.lund_a() - real_from_f64::<R>(0.68)).abs() < tol);
    assert!((p.min_invariant_mass() - real_from_f64::<R>(0.5)).abs() < tol);
    assert!(p.strange_suppression() > R::zero());

    // Custom construction round-trips through R.
    let c = LundParameters::<R>::new(
        real_from_f64::<R>(2.0),
        real_from_f64::<R>(0.5),
        real_from_f64::<R>(0.8),
        real_from_f64::<R>(0.4),
        real_from_f64::<R>(0.2),
        real_from_f64::<R>(0.1),
        real_from_f64::<R>(0.6),
        real_from_f64::<R>(0.3),
    );
    assert!((c.lund_b() - real_from_f64::<R>(0.8)).abs() < tol);
}

#[test]
fn lund_parameters_f32() {
    run_lund_parameters::<f32>();
}

#[test]
fn lund_parameters_f64() {
    run_lund_parameters::<f64>();
}

// ---------------------------------------------------------------------------
// Nuclear physics kernels (radioactive decay, binding energy)
// ---------------------------------------------------------------------------

fn run_physics<R: RealField + FromPrimitive>() {
    // After one half-life, N(t) = N0 / 2.
    let n0 = AmountOfSubstance::new(real_from_f64::<R>(1000.0)).unwrap();
    let half_life = HalfLife::new(real_from_f64::<R>(5.0)).unwrap();
    let time = Time::new(real_from_f64::<R>(5.0)).unwrap();
    let n = radioactive_decay_kernel::<R>(&n0, &half_life, &time).unwrap();
    assert!((n.value() - real_from_f64::<R>(500.0)).abs() < real_from_f64::<R>(1.0));

    // Binding energy E = Δm c² must be positive for positive mass defect.
    let dm = Mass::new(real_from_f64::<R>(1.0e-3)).unwrap();
    let e: Energy<R> = binding_energy_kernel::<R>(&dm).unwrap();
    assert!(e.value() > R::zero());
}

#[test]
fn physics_f32() {
    run_physics::<f32>();
}

#[test]
fn physics_f64() {
    run_physics::<f64>();
}
