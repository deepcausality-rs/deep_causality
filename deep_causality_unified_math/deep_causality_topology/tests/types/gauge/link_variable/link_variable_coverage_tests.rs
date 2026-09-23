/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Additional coverage for LinkVariable phase construction, the determinant and SU(N)
//! projection at matrix dimensions other than 1, 2 or 3.

use deep_causality_num_complex::Complex;
use deep_causality_stats::Xoshiro256;
use deep_causality_topology::{GaugeGroup, LinkVariable, RandomField, SE3, SO3_1, SU3, SU3_SU2_U1};

// ============================================================================
// try_from_phase: general SU(n) arm for n >= 4 (matrix_dim() == 4 here).
// ============================================================================

#[test]
fn test_try_from_phase_general_arm_dim_four() {
    // SO3_1::matrix_dim() == 4, so this drives the `_ =>` general SU(n) arm:
    //   diag(exp(iφ), exp(-iφ/(n-1)), exp(-iφ/(n-1)), ...).
    let phase = 0.5_f64;
    let link: LinkVariable<SO3_1, Complex<f64>, f64> =
        LinkVariable::try_from_phase(phase).expect("phase link for 4x4 group");

    let s = link.as_slice();
    assert_eq!(s.len(), 16);

    // First diagonal entry is exp(iφ).
    let expected0 = Complex::new(phase.cos(), phase.sin());
    assert!((s[0].re - expected0.re).abs() < 1e-12);
    assert!((s[0].im - expected0.im).abs() < 1e-12);

    // Remaining diagonal entries are exp(-iφ/(n-1)) with n = 4.
    let comp_angle = -phase / 3.0;
    let comp = Complex::new(comp_angle.cos(), comp_angle.sin());
    for i in 1..4 {
        let d = s[i * 4 + i];
        assert!((d.re - comp.re).abs() < 1e-12);
        assert!((d.im - comp.im).abs() < 1e-12);
    }
}

#[test]
fn test_try_from_phase_general_arm_se3() {
    // SE3::matrix_dim() == 4 as well: independent confirmation of the general arm.
    let link: LinkVariable<SE3, Complex<f64>, f64> =
        LinkVariable::try_from_phase(0.25).expect("phase link for SE3");
    assert_eq!(link.as_slice().len(), 16);
}

// ============================================================================
// determinant: closed forms for N <= 3, pivoted LU above.
// ============================================================================

/// A 5x5 gauge group, used only to reach the determinant at an odd order above four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct FiveDimGroup;
impl GaugeGroup for FiveDimGroup {
    const LIE_ALGEBRA_DIM: usize = 24;
    const IS_ABELIAN: bool = false;
    fn matrix_dim() -> usize {
        5
    }
    fn name() -> &'static str {
        "FiveDim"
    }
}

fn c(re: f64, im: f64) -> Complex<f64> {
    Complex::new(re, im)
}

fn assert_close(got: Complex<f64>, want: Complex<f64>) {
    assert!((got - want).norm() < 1e-10, "got {got:?}, want {want:?}");
}

#[test]
fn test_determinant_3x3_closed_form() {
    // [[1+i, 2, 0], [-i, 3, 1-2i], [4, i, 2]]; det = 13 - 9i by cofactor expansion.
    let m: LinkVariable<SU3, Complex<f64>, f64> = LinkVariable::try_from_matrix(vec![
        c(1.0, 1.0),
        c(2.0, 0.0),
        c(0.0, 0.0),
        c(0.0, -1.0),
        c(3.0, 0.0),
        c(1.0, -2.0),
        c(4.0, 0.0),
        c(0.0, 1.0),
        c(2.0, 0.0),
    ])
    .unwrap();
    assert_close(m.determinant(), c(13.0, -9.0));
}

/// [[0, 2+i, 1, -i], [3-i, 1, 0, 2], [i, -2, 4+2i, 1], [1, 1+i, -1, 3]].
/// The zero in the corner forces a row swap at the first column.
fn matrix_4x4() -> Vec<Complex<f64>> {
    vec![
        c(0.0, 0.0),
        c(2.0, 1.0),
        c(1.0, 0.0),
        c(0.0, -1.0),
        c(3.0, -1.0),
        c(1.0, 0.0),
        c(0.0, 0.0),
        c(2.0, 0.0),
        c(0.0, 1.0),
        c(-2.0, 0.0),
        c(4.0, 2.0),
        c(1.0, 0.0),
        c(1.0, 0.0),
        c(1.0, 1.0),
        c(-1.0, 0.0),
        c(3.0, 0.0),
    ]
}

#[test]
fn test_determinant_4x4_pivoted_lu() {
    // det = -79 - 34i by cofactor expansion in exact Gaussian integers.
    let m: LinkVariable<SO3_1, Complex<f64>, f64> =
        LinkVariable::try_from_matrix(matrix_4x4()).unwrap();
    assert_close(m.determinant(), c(-79.0, -34.0));
}

#[test]
fn test_determinant_5x5_pivoted_lu() {
    // [[0, 1, 2-i, 0, i], [2, 0, 1, 1+i, -1], [1-2i, 3, 0, 1, 2], [0, i, 1, -2, 1],
    //  [1, 0, -i, 2, 3+i]]; det = -86 + 84i by cofactor expansion.
    let m: LinkVariable<FiveDimGroup, Complex<f64>, f64> = LinkVariable::try_from_matrix(vec![
        c(0.0, 0.0),
        c(1.0, 0.0),
        c(2.0, -1.0),
        c(0.0, 0.0),
        c(0.0, 1.0),
        c(2.0, 0.0),
        c(0.0, 0.0),
        c(1.0, 0.0),
        c(1.0, 1.0),
        c(-1.0, 0.0),
        c(1.0, -2.0),
        c(3.0, 0.0),
        c(0.0, 0.0),
        c(1.0, 0.0),
        c(2.0, 0.0),
        c(0.0, 0.0),
        c(0.0, 1.0),
        c(1.0, 0.0),
        c(-2.0, 0.0),
        c(1.0, 0.0),
        c(1.0, 0.0),
        c(0.0, 0.0),
        c(0.0, -1.0),
        c(2.0, 0.0),
        c(3.0, 1.0),
    ])
    .unwrap();
    assert_close(m.determinant(), c(-86.0, 84.0));
}

#[test]
fn test_determinant_4x4_singular() {
    // Last row = row 0 + i * row 1: complex linear dependence, det = 0.
    let mut data = matrix_4x4();
    for k in 0..4 {
        data[12 + k] = data[k] + c(0.0, 1.0) * data[4 + k];
    }
    let m: LinkVariable<SO3_1, Complex<f64>, f64> = LinkVariable::try_from_matrix(data).unwrap();
    assert!(m.determinant().norm() < 1e-10);

    // A zero column leaves no pivot at all, and the determinant is exactly zero.
    let mut data = matrix_4x4();
    for r in 0..4 {
        data[r * 4 + 2] = c(0.0, 0.0);
    }
    let m: LinkVariable<SO3_1, Complex<f64>, f64> = LinkVariable::try_from_matrix(data).unwrap();
    assert_eq!(m.determinant(), c(0.0, 0.0));
}

// ============================================================================
// project_sun for N >= 4.
// ============================================================================

/// U†U = I and det U = 1 within `tol`.
fn assert_special_unitary<G: GaugeGroup>(u: &LinkVariable<G, Complex<f64>, f64>, tol: f64) {
    let n = G::matrix_dim();
    let udu = u.dagger().mul(u);
    for i in 0..n {
        for j in 0..n {
            let want = if i == j { c(1.0, 0.0) } else { c(0.0, 0.0) };
            let got = udu.as_slice()[i * n + j];
            assert!((got - want).norm() < tol, "U†U[{i},{j}] = {got:?}");
        }
    }
    let det = u.determinant();
    assert!((det - c(1.0, 0.0)).norm() < tol, "det U = {det:?}");
}

#[test]
fn test_project_sun_random_4x4_is_special_unitary() {
    let mut rng = Xoshiro256::from_seed(11);
    let m: LinkVariable<SO3_1, Complex<f64>, f64> = LinkVariable::try_from_matrix(
        (0..16)
            .map(|_| Complex::<f64>::generate_uniform(&mut rng))
            .collect(),
    )
    .unwrap();
    // The input is neither unitary nor of unit determinant.
    assert!((m.determinant() - c(1.0, 0.0)).norm() > 1e-3);

    let u = m.project_sun().expect("4x4 projection");
    assert_special_unitary(&u, 1e-9);
}

#[test]
fn test_try_random_for_groups_above_three() {
    let mut rng = Xoshiro256::from_seed(5);
    let u: LinkVariable<SE3, Complex<f64>, f64> = LinkVariable::try_random(&mut rng).unwrap();
    assert_special_unitary(&u, 1e-9);
    let u: LinkVariable<SU3_SU2_U1, Complex<f64>, f64> =
        LinkVariable::try_random(&mut rng).unwrap();
    assert_special_unitary(&u, 1e-9);
}
