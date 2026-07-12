/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    logical_cz, logical_hadamard, logical_s, logical_t, logical_x, logical_z,
};

// A simple projector field a (scalar 1, so a² = a) — the exp series converges.
fn create_projector_field() -> CausalMultiVector<Complex<f64>> {
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_logical_s_gate() {
    // S(a) = exp(i pi/2 a^2); for a = 1 the series converges to a valid operator.
    let result = logical_s(&create_projector_field()).unwrap();
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_z_gate() {
    let result = logical_z(&create_projector_field()).unwrap();
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_x_gate() {
    let result = logical_x(&create_projector_field()).unwrap();
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_hadamard_gate() {
    let a = create_projector_field();
    let b = create_projector_field();
    let result = logical_hadamard(&a, &b).unwrap();
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_cz_gate() {
    let a1 = create_projector_field();
    let a2 = create_projector_field();
    let result = logical_cz(&a1, &a2).unwrap();
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_t_gate() {
    let result = logical_t(&create_projector_field()).unwrap();
    assert!(!result.data().is_empty());
}

// A zero field.
fn create_zero_field() -> CausalMultiVector<Complex<f64>> {
    CausalMultiVector::new(vec![Complex::new(0.0, 0.0); 8], Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_exp_zero_fast_path_is_identity() {
    // logical_z(0) => exp(0) = I — a genuine identity result (Ok). This is the one
    // case where an identity operator is the correct answer, not a masked failure.
    let result = logical_z(&create_zero_field()).unwrap();
    assert!((result.data()[0].re - 1.0).abs() < 1e-12);
    assert!(result.data()[0].im.abs() < 1e-12);
    for c in &result.data()[1..] {
        assert!(c.re.abs() < 1e-12 && c.im.abs() < 1e-12);
    }
}

#[test]
fn test_overflowing_field_errors_instead_of_masking_as_identity() {
    // A field whose exponent norm exceeds the 1e6 overflow bound has no finite
    // exp. It must surface a QuantumError, NOT silently return the scalar identity
    // (which a caller could not distinguish from a real identity gate). This is
    // the whole point of the logical gates returning `Result`.
    let mut data: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); 8];
    data[1] = Complex::new(1e8, 0.0); // |exponent| ~ 1e8 * pi >> 1e6
    let a = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    assert!(logical_z(&a).is_err());
    // The same overflowing field errors through every single-field gate.
    assert!(logical_x(&a).is_err());
    assert!(logical_s(&a).is_err());
    assert!(logical_t(&a).is_err());
}
