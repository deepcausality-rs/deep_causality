/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    PhysicsErrorEnum, StrainRateTensor, Viscosity, newtonian_viscous_stress_kernel,
    newtonian_viscous_stress_with_bulk_kernel, power_law_apparent_viscosity_kernel,
};

const TOL_F64: f64 = 1e-12;
const TOL_F32: f32 = 1e-5;

// =============================================================================
// newtonian_viscous_stress_kernel
// =============================================================================

#[test]
fn test_newtonian_stress_vanishes_for_zero_strain_and_zero_div() {
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    let s = StrainRateTensor::<f64>::default();
    let tau = newtonian_viscous_stress_kernel(&mu, &s, 0.0).unwrap();
    for row in tau.value() {
        for v in row {
            assert_eq!(*v, 0.0);
        }
    }
}

#[test]
fn test_newtonian_stress_is_symmetric() {
    let mu = Viscosity::<f64>::new(0.01).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let tau = newtonian_viscous_stress_kernel(&mu, &s, 0.0).unwrap();
    let t = tau.value();
    assert!((t[0][1] - t[1][0]).abs() < TOL_F64);
    assert!((t[0][2] - t[2][0]).abs() < TOL_F64);
    assert!((t[1][2] - t[2][1]).abs() < TOL_F64);
}

#[test]
fn test_newtonian_stress_incompressible_is_2mu_s() {
    // For an incompressible fluid (∇·u = 0), τ = 2μ S exactly.
    let mu = Viscosity::<f64>::new(0.5).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[1.0, 2.0, 0.0], [2.0, -1.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let tau = newtonian_viscous_stress_kernel(&mu, &s, 0.0).unwrap();
    let t = tau.value();
    let sv = s.value();
    for i in 0..3 {
        for j in 0..3 {
            assert!((t[i][j] - 2.0 * 0.5 * sv[i][j]).abs() < TOL_F64);
        }
    }
}

#[test]
fn test_newtonian_stress_bulk_correction_on_diagonal_only() {
    // For an isotropic dilatation S = (div_u / 3) · I, the off-diagonal terms
    // of τ remain zero and the diagonal carries the bulk correction.
    let mu = Viscosity::<f64>::new(1.0).unwrap();
    let div_u = 3.0;
    let s_diag = 1.0; // S = diag(1, 1, 1) so trace(S) = 3 = div_u
    let s =
        StrainRateTensor::<f64>::new([[s_diag, 0.0, 0.0], [0.0, s_diag, 0.0], [0.0, 0.0, s_diag]])
            .unwrap();
    let tau = newtonian_viscous_stress_kernel(&mu, &s, div_u).unwrap();
    let t = tau.value();
    // Off-diagonals are zero
    assert_eq!(t[0][1], 0.0);
    assert_eq!(t[0][2], 0.0);
    assert_eq!(t[1][2], 0.0);
    // Diagonal: 2 * 1 * 1 - (2/3) * 1 * 3 = 2 - 2 = 0
    for (i, row) in t.iter().enumerate() {
        assert!(row[i].abs() < TOL_F64);
    }
}

#[test]
fn test_newtonian_stress_f32_precision() {
    let mu = Viscosity::<f32>::new(0.5).unwrap();
    let s =
        StrainRateTensor::<f32>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let tau = newtonian_viscous_stress_kernel(&mu, &s, 0.0).unwrap();
    // τ_00 = 2 * 0.5 * 1 = 1
    assert!((tau.value()[0][0] - 1.0).abs() < TOL_F32);
}

// =============================================================================
// newtonian_viscous_stress_with_bulk_kernel
// =============================================================================

#[test]
fn test_bulk_with_zero_zeta_matches_stokes_hypothesis() {
    // ζ = 0 should give bit-for-bit equal output to the bulk-free kernel.
    let mu = Viscosity::<f64>::new(0.01).unwrap();
    let zeta = Viscosity::<f64>::new(0.0).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[0.5, 1.0, 0.0], [1.0, -0.5, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let div_u = 0.2;
    let tau_with = newtonian_viscous_stress_with_bulk_kernel(&mu, &zeta, &s, div_u).unwrap();
    let tau_without = newtonian_viscous_stress_kernel(&mu, &s, div_u).unwrap();
    for (rw, ro) in tau_with.value().iter().zip(tau_without.value().iter()) {
        for (vw, vo) in rw.iter().zip(ro.iter()) {
            assert_eq!(vw, vo);
        }
    }
}

#[test]
fn test_bulk_term_adds_only_to_diagonal() {
    let mu = Viscosity::<f64>::new(0.0).unwrap();
    let zeta = Viscosity::<f64>::new(1.0).unwrap();
    let s = StrainRateTensor::<f64>::default();
    let div_u = 5.0;
    let tau = newtonian_viscous_stress_with_bulk_kernel(&mu, &zeta, &s, div_u).unwrap();
    let t = tau.value();
    // Off-diagonals zero
    assert_eq!(t[0][1], 0.0);
    assert_eq!(t[1][2], 0.0);
    // Diagonal: (-0 + 1) * 5 = 5
    for (i, row) in t.iter().enumerate() {
        assert!((row[i] - 5.0).abs() < TOL_F64);
    }
}

#[test]
fn test_bulk_kernel_symmetric() {
    let mu = Viscosity::<f64>::new(0.01).unwrap();
    let zeta = Viscosity::<f64>::new(0.005).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]).unwrap();
    let tau = newtonian_viscous_stress_with_bulk_kernel(&mu, &zeta, &s, 0.7).unwrap();
    let t = tau.value();
    assert!((t[0][1] - t[1][0]).abs() < TOL_F64);
    assert!((t[0][2] - t[2][0]).abs() < TOL_F64);
    assert!((t[1][2] - t[2][1]).abs() < TOL_F64);
}

// =============================================================================
// power_law_apparent_viscosity_kernel
// =============================================================================

#[test]
fn test_power_law_reduces_to_newtonian_at_n_eq_one() {
    let k: f64 = 0.5;
    let n: f64 = 1.0;
    for shear_rate in [0.0_f64, 1.0, 10.0, 1.0e5] {
        let mu_eff = power_law_apparent_viscosity_kernel(k, n, shear_rate).unwrap();
        assert!((mu_eff.value() - k).abs() < TOL_F64);
    }
}

#[test]
fn test_power_law_shear_thinning() {
    // n < 1: viscosity decreases with shear rate.
    let k: f64 = 1.0;
    let n: f64 = 0.5;
    let mu_low = power_law_apparent_viscosity_kernel(k, n, 1.0).unwrap();
    let mu_high = power_law_apparent_viscosity_kernel(k, n, 100.0).unwrap();
    assert!(mu_high.value() < mu_low.value());
}

#[test]
fn test_power_law_shear_thickening() {
    // n > 1: viscosity increases with shear rate.
    let k: f64 = 1.0;
    let n: f64 = 1.5;
    let mu_low = power_law_apparent_viscosity_kernel(k, n, 1.0).unwrap();
    let mu_high = power_law_apparent_viscosity_kernel(k, n, 100.0).unwrap();
    assert!(mu_high.value() > mu_low.value());
}

#[test]
fn test_power_law_known_value() {
    // K = 2, n = 0.5, γ̇ = 4: μ_eff = 2 * 4^(-0.5) = 2 / 2 = 1.
    let mu_eff = power_law_apparent_viscosity_kernel::<f64>(2.0, 0.5, 4.0).unwrap();
    assert!((mu_eff.value() - 1.0).abs() < TOL_F64);
}

#[test]
fn test_power_law_errors_on_negative_shear_rate() {
    let r = power_law_apparent_viscosity_kernel::<f64>(1.0, 0.5, -0.1);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("shear_rate")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_power_law_zero_shear_rate_with_shear_thinning_errors() {
    // n < 1, γ̇ = 0 => γ̇^(n-1) = 0^(negative) = +∞ => Viscosity::new rejects.
    let r = power_law_apparent_viscosity_kernel::<f64>(1.0, 0.5, 0.0);
    assert!(r.is_err());
}

#[test]
fn test_power_law_f32_precision() {
    let mu_eff = power_law_apparent_viscosity_kernel::<f32>(2.0, 0.5, 4.0).unwrap();
    assert!((mu_eff.value() - 1.0).abs() < TOL_F32);
}
