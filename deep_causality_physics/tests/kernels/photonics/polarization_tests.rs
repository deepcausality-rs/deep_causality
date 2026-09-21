/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    JonesVector, PhysicsErrorEnum, RayAngle, StokesVector, degree_of_polarization_kernel,
    jones_rotation_kernel, stokes_from_jones_kernel,
};
use deep_causality_tensor::CausalTensor;
use std::f64::consts::PI;

/// A linear polarizer transmitting along x: `diag(1, 0)`.
fn horizontal_polarizer() -> CausalTensor<Complex<f64>> {
    CausalTensor::new(
        vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap()
}

fn assert_matrix_close(actual: &CausalTensor<Complex<f64>>, expected: [[f64; 2]; 2]) {
    let d = actual.data();
    for row in 0..2 {
        for col in 0..2 {
            let got = d[row * 2 + col];
            assert!(
                (got.re - expected[row][col]).abs() < 1e-10 && got.im.abs() < 1e-10,
                "[{row}][{col}] = {got:?}, expected {}",
                expected[row][col]
            );
        }
    }
}

#[test]
fn test_jones_rotation_turns_a_horizontal_polarizer_into_a_vertical_one() {
    // The identity is invariant under any similarity transform, so rotating it cannot tell a
    // correct rotation matrix from a wrong one. A polarizer can: with R(phi) = [[c, s], [-s, c]]
    // and M' = R(-phi) M R(phi), a quarter turn takes diag(1, 0) to diag(0, 1).
    let angle = RayAngle::<f64>::new(PI / 2.0).unwrap();
    let rotated = jones_rotation_kernel(&horizontal_polarizer(), angle).unwrap();
    assert_matrix_close(&rotated, [[0.0, 0.0], [0.0, 1.0]]);
}

#[test]
fn test_jones_rotation_at_forty_five_degrees_gives_the_diagonal_projector() {
    // At phi = pi/4, c = s = 1/sqrt(2) and R(-phi) diag(1, 0) R(phi) = [[c^2, cs], [sc, s^2]],
    // which is the projector onto the 45-degree axis: every entry 1/2.
    let angle = RayAngle::<f64>::new(PI / 4.0).unwrap();
    let rotated = jones_rotation_kernel(&horizontal_polarizer(), angle).unwrap();
    assert_matrix_close(&rotated, [[0.5, 0.5], [0.5, 0.5]]);
}

#[test]
fn test_jones_rotation_preserves_trace_and_determinant() {
    // M' = R(-phi) M R(phi) is a similarity transform, so both invariants survive it for any
    // matrix and any angle. Neither needs an oracle.
    let m = CausalTensor::new(
        vec![
            Complex::new(1.0, 0.5),
            Complex::new(-2.0, 0.0),
            Complex::new(0.25, -1.0),
            Complex::new(3.0, 2.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let d = m.data();
    let trace = d[0] + d[3];
    let det = d[0] * d[3] - d[1] * d[2];

    for phi in [0.3_f64, PI / 6.0, PI / 2.0, 2.0] {
        let angle = RayAngle::<f64>::new(phi).unwrap();
        let r = jones_rotation_kernel(&m, angle).unwrap();
        let rd = r.data();
        let r_trace = rd[0] + rd[3];
        let r_det = rd[0] * rd[3] - rd[1] * rd[2];
        assert!(
            (r_trace.re - trace.re).abs() < 1e-10 && (r_trace.im - trace.im).abs() < 1e-10,
            "phi = {phi}: trace moved from {trace:?} to {r_trace:?}"
        );
        assert!(
            (r_det.re - det.re).abs() < 1e-10 && (r_det.im - det.im).abs() < 1e-10,
            "phi = {phi}: determinant moved from {det:?} to {r_det:?}"
        );
    }
}

#[test]
fn test_jones_rotation_by_zero_is_the_identity_transform() {
    let angle = RayAngle::<f64>::new(0.0).unwrap();
    let rotated = jones_rotation_kernel(&horizontal_polarizer(), angle).unwrap();
    assert_matrix_close(&rotated, [[1.0, 0.0], [0.0, 0.0]]);
}

#[test]
fn test_jones_rotation_error() {
    let m = CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1]).unwrap();
    let angle = RayAngle::<f64>::new(0.0).unwrap();
    assert!(
        matches!(
            jones_rotation_kernel(&m, angle).unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_stokes_from_jones() {
    // H = [1, 0]. Stokes = [1, 1, 0, 0]
    let j_data = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let j_tensor = CausalTensor::new(j_data, vec![2]).unwrap();
    let jones = JonesVector::<f64>::new(j_tensor);

    let res = stokes_from_jones_kernel(&jones);
    assert!(res.is_ok());
    let s = res.unwrap();
    let d = s.inner().data();

    assert!((d[0] - 1.0).abs() < 1e-10); // S0
    assert!((d[1] - 1.0).abs() < 1e-10); // S1
    assert!((d[2] - 0.0).abs() < 1e-10);
    assert!((d[3] - 0.0).abs() < 1e-10);
}

#[test]
fn test_stokes_from_jones_error() {
    let j =
        JonesVector::<f64>::new(CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1]).unwrap());
    assert!(
        matches!(
            stokes_from_jones_kernel(&j).unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_dop() {
    // Fully polarized [1, 1, 0, 0]
    let s_data = vec![1.0, 1.0, 0.0, 0.0];
    let s_tensor = CausalTensor::new(s_data, vec![4]).unwrap();
    let stokes = StokesVector::<f64>::new(s_tensor).unwrap();

    let res = degree_of_polarization_kernel(&stokes);
    assert!(res.is_ok());
    assert!((res.unwrap().value() - 1.0).abs() < 1e-10);

    // Unpolarized [1, 0, 0, 0]
    let s_unpol =
        StokesVector::<f64>::new(CausalTensor::new(vec![1.0, 0.0, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();
    let res2 = degree_of_polarization_kernel(&s_unpol);
    assert!((res2.unwrap().value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_dop_errors() {
    // S0 < 0
    // StokesVector::new enforces S0^2 >= S1^2 + S2^2 + S3^2.
    // If S0 < 0, S0^2 is positive. So we can have S0 = -1, S1=0,0,0.
    // However, degree_of_polarization_kernel checks S0 <= 0.
    let s_neg =
        StokesVector::<f64>::new(CausalTensor::new(vec![-1.0, 0.0, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();
    assert!(
        matches!(
            degree_of_polarization_kernel(&s_neg).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );

    // The kernel's DOP > 1 branch is unreachable from this constructor: `StokesVector::new`
    // enforces the same invariant, so any vector it accepts already has DOP <= 1. The shape
    // guard is reachable and is covered by `test_dop_wrong_length_error` below.
}

#[test]
fn test_dop_wrong_length_error() {
    // A default StokesVector wraps an empty tensor whose shape is not [4],
    // tripping the DimensionMismatch guard in degree_of_polarization_kernel
    // (polarization.rs:135-138).
    let stokes = StokesVector::<f64>::default();
    let res = degree_of_polarization_kernel(&stokes);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_dop_zero_intensity_returns_zero() {
    // S = [0, 0, 0, 0] passes StokesVector::new (0 >= 0) and exercises the
    // zero-intensity early return that yields DOP = 0 (polarization.rs:147-150).
    let stokes =
        StokesVector::<f64>::new(CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();
    let res = degree_of_polarization_kernel(&stokes);
    assert!(res.is_ok());
    assert!((res.unwrap().value() - 0.0).abs() < 1e-12);
}

#[test]
fn test_stokes_vector_new_error() {
    // Shape error
    let t_wrong = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    assert!(
        matches!(
            StokesVector::<f64>::new(t_wrong).unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );

    // Invariant error: S0^2 < S1^2 + S2^2 + S3^2
    let t_inv = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![4]).unwrap();
    assert!(
        matches!(
            StokesVector::<f64>::new(t_inv).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

// NOTE on polarization.rs:163-165 — the "DOP > 1, unphysical Stokes vector"
// guard in `degree_of_polarization_kernel`. The only constructor for a
// non-default `StokesVector` is `StokesVector::new`, which enforces the
// physical invariant `S0² >= S1² + S2² + S3²`. That invariant implies
// `sqrt(S1²+S2²+S3²) / S0 <= 1` whenever `S0 > 0`, so the computed DOP can never
// exceed the `1.000001` tolerance. There is no `new_unchecked` escape hatch for
// `StokesVector`, so this guard is unreachable for any constructible input.
