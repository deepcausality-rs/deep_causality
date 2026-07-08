/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    JonesVector, RayAngle, StokesVector, degree_of_polarization_kernel, jones_rotation_kernel,
    stokes_from_jones_kernel,
};
use deep_causality_tensor::CausalTensor;
use std::f64::consts::PI;

#[test]
fn test_jones_rotation() {
    // Horizontal H = [1, 0]
    // Rotate 90 deg -> Vertical V = [0, 1]
    // But kernel rotates the MATRIX operator, not vector.
    // Let's test rotating an Identity operator? R(-t) I R(t) = I.

    let id_data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
    ];
    let id = CausalTensor::new(id_data, vec![2, 2]).unwrap();

    let angle = RayAngle::<f64>::new(PI / 2.0).unwrap();
    let res = jones_rotation_kernel(&id, angle);
    assert!(res.is_ok());
    let rot = res.unwrap();
    // Should still be Identity
    let d = rot.data();
    assert!((d[0].re - 1.0).abs() < 1e-10);
    assert!((d[3].re - 1.0).abs() < 1e-10);
}

#[test]
fn test_jones_rotation_error() {
    let m = CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1]).unwrap();
    let angle = RayAngle::<f64>::new(0.0).unwrap();
    assert!(jones_rotation_kernel(&m, angle).is_err());
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
    assert!(stokes_from_jones_kernel(&j).is_err());
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
    assert!(degree_of_polarization_kernel(&s_neg).is_err());

    // DOP > 1
    // This is hard because StokesVector::new also validates the invariant.
    // But we can test the error hit in the kernel if we could bypass StokesVector::new (we can't easily).
    // Let's assume StokesVector is valid, then DOP <= 1.
    // However, if we use a different tensor shape, the kernel hits DimensionMismatch.
}

#[test]
fn test_dop_wrong_length_error() {
    // A default StokesVector wraps an empty tensor whose shape is not [4],
    // tripping the DimensionMismatch guard in degree_of_polarization_kernel
    // (polarization.rs:135-138).
    let stokes = StokesVector::<f64>::default();
    let res = degree_of_polarization_kernel(&stokes);
    assert!(res.is_err());
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
    assert!(StokesVector::<f64>::new(t_wrong).is_err());

    // Invariant error: S0^2 < S1^2 + S2^2 + S3^2
    let t_inv = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![4]).unwrap();
    assert!(StokesVector::<f64>::new(t_inv).is_err());
}

// NOTE on polarization.rs:163-165 — the "DOP > 1, unphysical Stokes vector"
// guard in `degree_of_polarization_kernel`. The only constructor for a
// non-default `StokesVector` is `StokesVector::new`, which enforces the
// physical invariant `S0² >= S1² + S2² + S3²`. That invariant implies
// `sqrt(S1²+S2²+S3²) / S0 <= 1` whenever `S0 > 0`, so the computed DOP can never
// exceed the `1.000001` tolerance. There is no `new_unchecked` escape hatch for
// `StokesVector`, so this guard is unreachable for any constructible input.
