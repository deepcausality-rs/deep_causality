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

/// A hand-built U(1) configuration whose charge density works out to `1/(16*pi^2)` on paper.
///
/// For U(1) a link variable is a single complex phase, so the whole chain is analytic. The
/// plaquette at the origin in the `(mu, nu)` plane is
/// `U_mu(0) U_nu(e_mu) conj(U_mu(e_nu)) conj(U_nu(0))`. On pure phases that is `exp(i*Theta)`
/// with `Theta_mu_nu = phi_mu(0) + phi_nu(e_mu) - phi_mu(e_nu) - phi_nu(0)`, the field strength
/// `(U - U^dag)/2` is the pure imaginary number `i*sin(Theta)`, and the trace of a product of two
/// field strengths is `-sin(Theta_mu_nu) * sin(Theta_rho_sigma)`.
///
/// The six perturbed links are chosen to move one plaquette angle each. A link at site `e_nu`
/// pointing in direction `mu < nu` lies on the far edge of the `(mu, nu)` plaquette at the origin
/// and appears in none of the other five plaquettes there, so it sets `Theta_mu_nu = -phi` and
/// leaves the rest at zero. With phases of `pi/2` and `pi/6`, whose sines are `1` and `1/2`:
///
/// ```text
/// Tr[F01 F23] = -sin(pi/2) sin(pi/6) = -1/2
/// Tr[F02 F13] = -sin(pi/2) sin(pi/2) = -1
/// Tr[F03 F12] = -sin(pi/6) sin(pi/6) = -1/4
/// ```
///
/// so `Tr[F01 F23] - Tr[F02 F13] + Tr[F03 F12] = -1/2 + 1 - 1/4 = 1/4`. The epsilon contraction
/// runs over 24 permutations of four distinct indices, eight per unordered pairing, and each of
/// the eight contributes the same term. The density is therefore
/// `q = (1/(32*pi^2)) * 8 * (1/4) = 1/(16*pi^2) ~ 0.006332573977646111`.
///
/// That number comes from the six phases and nothing else; no part of it is produced by
/// `try_field_strength`. The test therefore constrains the three pairing signs together with the
/// overall constant. Applying `1/(32*pi^2)` to the three-term sum returns an eighth of the
/// expected value. Flipping the sign on the `{02|13}` pairing turns the sum into `-7/4`, and
/// pairing `F01` with `F13` and `F02` with `F23` turns it into `-3/4`.
#[test]
fn test_topological_charge_density_uses_the_full_epsilon_normalization() {
    use deep_causality_topology::{LatticeCell, LinkVariable};
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_6, PI};

    type Link = LinkVariable<U1, Complex<f64>, f64>;

    let lattice = Arc::new(LatticeComplex::new([4, 4, 4, 4], [true; 4]));
    let mut field = LatticeGaugeField::<U1, 4, Complex<f64>, f64>::identity(lattice, 1.0);

    // (plane, phase). The phase goes on the link at site e_nu in direction mu, the far edge of
    // the (mu, nu) plaquette at the origin.
    let planes = [
        ((0, 1), FRAC_PI_2),
        ((0, 2), FRAC_PI_2),
        ((0, 3), FRAC_PI_6),
        ((1, 2), FRAC_PI_6),
        ((1, 3), FRAC_PI_2),
        ((2, 3), FRAC_PI_6),
    ];

    for ((mu, nu), phase) in planes {
        let mut far_site = [0usize; 4];
        far_site[nu] = 1;
        field.set_link(LatticeCell::edge(far_site, mu), Link::from_phase(phase));
    }

    let site = [0, 0, 0, 0];

    // The first step of the derivation, checked rather than assumed: each field strength is the
    // pure imaginary number i*sin(Theta_mu_nu) = -i*sin(phase).
    for ((mu, nu), phase) in planes {
        let f = field.try_field_strength(&site, mu, nu).unwrap();
        let entry = f.as_slice()[0];
        let analytic = Complex::new(0.0, -phase.sin());
        assert!(
            (entry - analytic).norm() < 1e-15,
            "F_{mu}{nu} should be {analytic}, got {entry}"
        );
    }

    let expected = 1.0 / (16.0 * PI * PI);
    let got = field.try_topological_charge_density(&site).unwrap();

    assert!(
        (got - expected).abs() <= expected * 1e-12,
        "expected {expected}, got {got}; the constant 1/(32*pi^2) on the three-term sum would \
         give {}",
        expected / 8.0
    );
}
