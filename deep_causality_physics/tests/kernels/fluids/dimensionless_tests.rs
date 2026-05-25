/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, KinematicViscosity, Length, Speed, Viscosity, bond_number_kernel,
    capillary_number_kernel, eckert_number_kernel, froude_number_kernel, grashof_number_kernel,
    knudsen_number_kernel, lewis_number_kernel, mach_number_kernel, nusselt_number_kernel,
    particle_stokes_number_kernel, peclet_number_kernel, prandtl_number_kernel,
    rayleigh_number_kernel, reynolds_number_kernel, richardson_number_kernel,
    schmidt_number_kernel, strouhal_number_kernel, weber_number_kernel,
};

const TOL: f64 = 1e-10;

// =============================================================================
// Reynolds
// =============================================================================

#[test]
fn test_reynolds_known_value() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    // Re = 2 * 0.1 / 1e-3 = 200
    let re = reynolds_number_kernel(&u, &l, &nu).unwrap();
    assert!((re - 200.0).abs() < TOL);
}

#[test]
fn test_reynolds_errors_on_zero_nu() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let l = Length::<f64>::new(1.0).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(reynolds_number_kernel(&u, &l, &nu).is_err());
}

// =============================================================================
// Mach
// =============================================================================

#[test]
fn test_mach_known_value() {
    let u = Speed::<f64>::new(170.0).unwrap();
    let a = Speed::<f64>::new(340.0).unwrap();
    let m = mach_number_kernel(&u, &a).unwrap();
    assert!((m - 0.5).abs() < TOL);
}

#[test]
fn test_mach_errors_on_zero_sound_speed() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let a = Speed::<f64>::new(0.0).unwrap();
    assert!(mach_number_kernel(&u, &a).is_err());
}

// =============================================================================
// Froude
// =============================================================================

#[test]
fn test_froude_known_value() {
    let u = Speed::<f64>::new(10.0).unwrap();
    let l = Length::<f64>::new(2.5).unwrap();
    // Fr = 10 / sqrt(9.8 * 2.5) = 10 / sqrt(24.5) ≈ 2.020...
    let fr = froude_number_kernel(&u, 9.8, &l).unwrap();
    let expected = 10.0 / 24.5_f64.sqrt();
    assert!((fr - expected).abs() < TOL);
}

#[test]
fn test_froude_errors_on_zero_gravity_length() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let l = Length::<f64>::new(0.0).unwrap();
    assert!(froude_number_kernel(&u, 9.8, &l).is_err());
}

// =============================================================================
// Weber
// =============================================================================

#[test]
fn test_weber_known_value() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.001).unwrap();
    let sigma = 0.072_f64;
    // We = 1000 * 4 * 0.001 / 0.072 ≈ 55.555...
    let we = weber_number_kernel(&rho, &u, &l, sigma).unwrap();
    assert!((we - (1000.0 * 4.0 * 0.001 / 0.072)).abs() < TOL);
}

#[test]
fn test_weber_errors_on_zero_surface_tension() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    let l = Length::<f64>::new(0.001).unwrap();
    assert!(weber_number_kernel(&rho, &u, &l, 0.0).is_err());
}

// =============================================================================
// Prandtl
// =============================================================================

#[test]
fn test_prandtl_known_value() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let alpha = 2.1e-5_f64;
    // Pr = 1.5e-5 / 2.1e-5
    let pr = prandtl_number_kernel(&nu, alpha).unwrap();
    assert!((pr - (1.5 / 2.1)).abs() < TOL);
}

#[test]
fn test_prandtl_errors_on_zero_alpha() {
    let nu = KinematicViscosity::<f64>::new(1.0).unwrap();
    assert!(prandtl_number_kernel(&nu, 0.0).is_err());
}

// =============================================================================
// Peclet — identity Pe = Re * Pr
// =============================================================================

#[test]
fn test_peclet_known_value() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    let alpha = 2.0e-5_f64;
    // Pe = 2 * 0.1 / 2e-5 = 10_000
    let pe = peclet_number_kernel(&u, &l, alpha).unwrap();
    assert!((pe - 10_000.0).abs() < TOL * 1e3);
}

#[test]
fn test_peclet_equals_reynolds_times_prandtl_identity() {
    let u = Speed::<f64>::new(1.5).unwrap();
    let l = Length::<f64>::new(0.05).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    let alpha = 1.5e-3_f64;

    let re = reynolds_number_kernel(&u, &l, &nu).unwrap();
    let pr = prandtl_number_kernel(&nu, alpha).unwrap();
    let pe = peclet_number_kernel(&u, &l, alpha).unwrap();
    assert!((pe - re * pr).abs() < TOL * pe.abs());
}

// =============================================================================
// Strouhal
// =============================================================================

#[test]
fn test_strouhal_known_value() {
    let u = Speed::<f64>::new(5.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    // Sr = 10 * 0.1 / 5 = 0.2
    let sr = strouhal_number_kernel(10.0_f64, &l, &u).unwrap();
    assert!((sr - 0.2).abs() < TOL);
}

#[test]
fn test_strouhal_errors_on_zero_velocity() {
    let u = Speed::<f64>::new(0.0).unwrap();
    let l = Length::<f64>::new(0.1).unwrap();
    assert!(strouhal_number_kernel(10.0_f64, &l, &u).is_err());
}

// =============================================================================
// Knudsen
// =============================================================================

#[test]
fn test_knudsen_known_value() {
    let l = Length::<f64>::new(1.0e-6).unwrap();
    // Kn = 1e-7 / 1e-6 = 0.1
    let kn = knudsen_number_kernel(1.0e-7_f64, &l).unwrap();
    assert!((kn - 0.1).abs() < TOL);
}

#[test]
fn test_knudsen_errors_on_zero_length() {
    let l = Length::<f64>::new(0.0).unwrap();
    assert!(knudsen_number_kernel(1.0_f64, &l).is_err());
}

// =============================================================================
// Richardson
// =============================================================================

#[test]
fn test_richardson_known_value() {
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(1.0).unwrap();
    // Ri = 9.8 * 3e-3 * 10 * 1 / 4 = 0.0735
    let ri = richardson_number_kernel(9.8_f64, 3.0e-3, 10.0, &l, &u).unwrap();
    assert!((ri - 0.0735).abs() < TOL);
}

#[test]
fn test_richardson_errors_on_zero_velocity() {
    let u = Speed::<f64>::new(0.0).unwrap();
    let l = Length::<f64>::new(1.0).unwrap();
    assert!(richardson_number_kernel(9.8_f64, 1.0e-3, 1.0, &l, &u).is_err());
}

// =============================================================================
// Rayleigh, Grashof — identity Ra = Gr * Pr
// =============================================================================

#[test]
fn test_rayleigh_known_value() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    // Ra = 9.8 * 3e-3 * 10 * 0.001 / (1.5e-5 * 2.1e-5)
    let ra = rayleigh_number_kernel(9.8_f64, 3.0e-3, 10.0, &l, &nu, 2.1e-5).unwrap();
    let expected = 9.8 * 3.0e-3 * 10.0 * 0.001 / (1.5e-5 * 2.1e-5);
    assert!((ra - expected).abs() < TOL * expected.abs());
}

#[test]
fn test_rayleigh_equals_grashof_times_prandtl_identity() {
    let l = Length::<f64>::new(0.05).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    let alpha = 2.0e-5_f64;
    let beta = 3.0e-3_f64;
    let delta_t = 5.0_f64;

    let ra = rayleigh_number_kernel(9.8, beta, delta_t, &l, &nu, alpha).unwrap();
    let gr = grashof_number_kernel(9.8, beta, delta_t, &l, &nu).unwrap();
    let pr = prandtl_number_kernel(&nu, alpha).unwrap();
    assert!((ra - gr * pr).abs() < TOL * ra.abs());
}

#[test]
fn test_rayleigh_errors_on_zero_nu() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(rayleigh_number_kernel(9.8_f64, 1.0e-3, 10.0, &l, &nu, 1.0e-5).is_err());
}

#[test]
fn test_rayleigh_errors_on_zero_alpha() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(rayleigh_number_kernel(9.8_f64, 1.0e-3, 10.0, &l, &nu, 0.0).is_err());
}

#[test]
fn test_grashof_errors_on_zero_nu() {
    let l = Length::<f64>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(grashof_number_kernel(9.8_f64, 1.0e-3, 10.0, &l, &nu).is_err());
}

// =============================================================================
// Eckert
// =============================================================================

#[test]
fn test_eckert_known_value() {
    let u = Speed::<f64>::new(10.0).unwrap();
    // Ec = 100 / (1000 * 5) = 0.02
    let ec = eckert_number_kernel(&u, 1000.0_f64, 5.0).unwrap();
    assert!((ec - 0.02).abs() < TOL);
}

#[test]
fn test_eckert_errors_on_zero_denominator() {
    let u = Speed::<f64>::new(1.0).unwrap();
    assert!(eckert_number_kernel(&u, 0.0_f64, 1.0).is_err());
    assert!(eckert_number_kernel(&u, 1.0_f64, 0.0).is_err());
}

// =============================================================================
// Schmidt, Lewis — identity Le = Sc / Pr
// =============================================================================

#[test]
fn test_schmidt_known_value() {
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    let sc = schmidt_number_kernel(&nu, 2.0e-9_f64).unwrap();
    assert!((sc - (1.0e-6 / 2.0e-9)).abs() < TOL * sc.abs());
}

#[test]
fn test_lewis_known_value() {
    let le = lewis_number_kernel(2.0e-5_f64, 5.0e-9_f64).unwrap();
    assert!((le - (2.0e-5 / 5.0e-9)).abs() < TOL * le.abs());
}

#[test]
fn test_lewis_equals_schmidt_over_prandtl_identity() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let alpha = 2.1e-5_f64;
    let d = 4.0e-9_f64;

    let sc = schmidt_number_kernel(&nu, d).unwrap();
    let pr = prandtl_number_kernel(&nu, alpha).unwrap();
    let le = lewis_number_kernel(alpha, d).unwrap();
    assert!((le - sc / pr).abs() < TOL * le.abs());
}

#[test]
fn test_schmidt_errors_on_zero_diffusivity() {
    let nu = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    assert!(schmidt_number_kernel(&nu, 0.0_f64).is_err());
}

#[test]
fn test_lewis_errors_on_zero_diffusivity() {
    assert!(lewis_number_kernel(1.0e-5_f64, 0.0_f64).is_err());
}

// =============================================================================
// Particle Stokes, Capillary, Bond, Nusselt
// =============================================================================

#[test]
fn test_particle_stokes_known_value() {
    let u = Speed::<f64>::new(10.0).unwrap();
    let l = Length::<f64>::new(0.01).unwrap();
    // St = 1e-3 * 10 / 0.01 = 1
    let st = particle_stokes_number_kernel(1.0e-3_f64, &u, &l).unwrap();
    assert!((st - 1.0).abs() < TOL);
}

#[test]
fn test_particle_stokes_errors_on_zero_length() {
    let u = Speed::<f64>::new(1.0).unwrap();
    let l = Length::<f64>::new(0.0).unwrap();
    assert!(particle_stokes_number_kernel(1.0_f64, &u, &l).is_err());
}

#[test]
fn test_capillary_known_value() {
    let mu = Viscosity::<f64>::new(0.001).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    // Ca = 0.001 * 1 / 0.072 ≈ 0.01389
    let ca = capillary_number_kernel(&mu, &u, 0.072_f64).unwrap();
    assert!((ca - (0.001 / 0.072)).abs() < TOL);
}

#[test]
fn test_capillary_errors_on_zero_surface_tension() {
    let mu = Viscosity::<f64>::new(0.001).unwrap();
    let u = Speed::<f64>::new(1.0).unwrap();
    assert!(capillary_number_kernel(&mu, &u, 0.0_f64).is_err());
}

#[test]
fn test_bond_known_value() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let l = Length::<f64>::new(0.01).unwrap();
    // Bo = 1000 * 9.8 * 1e-4 / 0.072 ≈ 13.61
    let bo = bond_number_kernel(&rho, 9.8_f64, &l, 0.072).unwrap();
    let expected = 1000.0 * 9.8 * 1.0e-4 / 0.072;
    assert!((bo - expected).abs() < TOL);
}

#[test]
fn test_bond_errors_on_zero_surface_tension() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let l = Length::<f64>::new(0.01).unwrap();
    assert!(bond_number_kernel(&rho, 9.8_f64, &l, 0.0).is_err());
}

#[test]
fn test_nusselt_known_value() {
    let l = Length::<f64>::new(0.1).unwrap();
    // Nu = 100 * 0.1 / 0.5 = 20
    let nu = nusselt_number_kernel(100.0_f64, &l, 0.5).unwrap();
    assert!((nu - 20.0).abs() < TOL);
}

#[test]
fn test_nusselt_errors_on_zero_conductivity() {
    let l = Length::<f64>::new(0.1).unwrap();
    assert!(nusselt_number_kernel(100.0_f64, &l, 0.0).is_err());
}

// =============================================================================
// f32 precision sweep on representative numbers
// =============================================================================

#[test]
fn test_reynolds_f32_precision() {
    let u = Speed::<f32>::new(2.0).unwrap();
    let l = Length::<f32>::new(0.1).unwrap();
    let nu = KinematicViscosity::<f32>::new(1.0e-3).unwrap();
    let re = reynolds_number_kernel(&u, &l, &nu).unwrap();
    assert!((re - 200.0_f32).abs() < 1.0e-3);
}

#[test]
fn test_peclet_eq_re_times_pr_identity_f32() {
    let u = Speed::<f32>::new(1.5).unwrap();
    let l = Length::<f32>::new(0.05).unwrap();
    let nu = KinematicViscosity::<f32>::new(1.0e-3).unwrap();
    let alpha = 1.5e-3_f32;

    let re = reynolds_number_kernel(&u, &l, &nu).unwrap();
    let pr = prandtl_number_kernel(&nu, alpha).unwrap();
    let pe = peclet_number_kernel(&u, &l, alpha).unwrap();
    assert!((pe - re * pr).abs() < 1.0e-3 * pe.abs());
}
