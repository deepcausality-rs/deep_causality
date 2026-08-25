/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::LatticeComplex;
use deep_causality_topology::LatticeGaugeField;
use std::sync::Arc;

// Define a test gauge group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct U1;

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "U1"
    }
}

#[test]
fn test_field_strength_diagonal() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    // F_mu_mu should be zero
    let f_00 = field.try_field_strength(&site, 0, 0).unwrap();

    // Check if it's zero
    let data = f_00.as_slice();
    assert_eq!(data[0], Complex::new(0.0, 0.0));
}

#[test]
fn test_field_strength_calculation() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let f_01 = field.try_field_strength(&site, 0, 1).unwrap();

    // For identity field, plaquette is identity.
    // F_01 ~ (U - U_dag)/2 = (1 - 1)/2 = 0
    let data = f_01.as_slice();
    assert!((data[0]).norm() < 1e-10);
}

#[test]
fn test_topological_charge_density_low_dim() {
    let shape = [4, 4]; // 2D
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let field = LatticeGaugeField::<U1, 2, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0];
    let q = field.try_topological_charge_density(&site).unwrap();

    // Should be exactly 0.0 for D < 4
    assert_eq!(q, 0.0);
}

#[test]
fn test_topological_charge_density_4d() {
    let shape = [4, 4, 4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true; 4]));
    let field = LatticeGaugeField::<U1, 4, Complex<f64>, f64>::identity(lattice, 1.0);

    let site = [0, 0, 0, 0];
    let q = field.try_topological_charge_density(&site).unwrap();

    // For identity field, F=0, so q=0
    assert!((q).abs() < 1e-10);

    let total_q = field.try_topological_charge().unwrap();
    assert!((total_q).abs() < 1e-10);
}

// ---- the epsilon normalization ----------------------------------------------------------------

/// The density must equal `(1/(4*pi^2)) * (Tr[F01 F23] - Tr[F02 F13] + Tr[F03 F12])`.
///
/// The docstring states the full contraction `q = (1/(32*pi^2)) eps_{mu nu rho sigma}
/// Tr[F_mu nu F_rho sigma]`, which runs over all 24 permutations of four distinct indices. Those
/// fall into three unordered pairings of eight permutations each, and every one of the eight
/// contributes the same term, so the contraction is 8 times the three-term sum the implementation
/// computes. The constant on that sum is therefore `8/(32*pi^2) = 1/(4*pi^2)`.
///
/// It was `1/(32*pi^2)`, leaving the density eight times too small and `Q` unquantized. The four
/// existing charge tests all use the identity field, where `F = 0` and every constant gives zero,
/// so none of them could see it. This one perturbs a link so the traces are non-zero.
#[test]
fn test_topological_charge_density_uses_the_full_epsilon_normalization() {
    use deep_causality_topology::LinkVariable;

    let lattice = Arc::new(LatticeComplex::new([4, 4, 4, 4], [true; 4]));
    // A hot start: every link is random, so every plaquette is non-trivial and the field strength
    // is non-zero. An identity field has F = 0, where any normalization gives zero.
    let mut rng = deep_causality_rand::rng();
    let field: LatticeGaugeField<U1, 4, Complex<f64>, f64> =
        LatticeGaugeField::random(lattice, 1.0, &mut rng);

    let site = [0, 0, 0, 0];
    let f = |mu, nu| field.try_field_strength(&site, mu, nu).unwrap();
    let tr = |a: &LinkVariable<U1, Complex<f64>, f64>, b: &LinkVariable<U1, Complex<f64>, f64>| {
        a.try_mul(b).unwrap().re_trace()
    };

    let three_term = tr(&f(0, 1), &f(2, 3)) - tr(&f(0, 2), &f(1, 3)) + tr(&f(0, 3), &f(1, 2));
    assert!(
        three_term.abs() > 1e-12,
        "the perturbation must leave a non-zero trace sum, or this test cannot discriminate; got {three_term}"
    );

    let expected = three_term / (4.0 * std::f64::consts::PI * std::f64::consts::PI);
    let got = field.try_topological_charge_density(&site).unwrap();

    assert!(
        (got - expected).abs() <= expected.abs() * 1e-12,
        "expected {expected}, got {got}; the old 1/(32*pi^2) constant would give {}",
        three_term / (32.0 * std::f64::consts::PI * std::f64::consts::PI)
    );
}
