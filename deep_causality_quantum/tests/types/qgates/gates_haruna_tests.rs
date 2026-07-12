/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    QuantumErrorEnum, logical_cz, logical_hadamard, logical_s, logical_t, logical_x, logical_z,
};
use std::f64::consts::FRAC_1_SQRT_2;

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

/// Asserts a gate output is the pure scalar `re + im·i` (all non-scalar blades ≈ 0),
/// pinning the actual mathematical value rather than merely a non-empty result.
fn assert_scalar(result: &CausalMultiVector<Complex<f64>>, re: f64, im: f64, name: &str) {
    let d = result.data();
    assert!(
        (d[0].re - re).abs() < 1e-10 && (d[0].im - im).abs() < 1e-10,
        "{name}: scalar component {:?} != expected ({re}, {im})",
        d[0]
    );
    for (k, c) in d[1..].iter().enumerate() {
        assert!(
            c.re.abs() < 1e-10 && c.im.abs() < 1e-10,
            "{name}: non-scalar blade {} leaked: {:?}",
            k + 1,
            c
        );
    }
}

#[test]
fn test_logical_s_gate() {
    // S(1) = exp(i·π/2·1²) = i.
    assert_scalar(
        &logical_s(&create_projector_field()).unwrap(),
        0.0,
        1.0,
        "logical_s(1)",
    );
}

#[test]
fn test_logical_z_gate() {
    // Z(1) = exp(i·π) = -1.
    assert_scalar(
        &logical_z(&create_projector_field()).unwrap(),
        -1.0,
        0.0,
        "logical_z(1)",
    );
}

#[test]
fn test_logical_x_gate() {
    // X(1) = exp(i·π) = -1.
    assert_scalar(
        &logical_x(&create_projector_field()).unwrap(),
        -1.0,
        0.0,
        "logical_x(1)",
    );
}

#[test]
fn test_logical_hadamard_gate() {
    // H(1,1) = phase·S(1)·exp(i·π/2)·S(1) = exp(-i·3π/4) = -1/√2 - i/√2.
    let a = create_projector_field();
    let b = create_projector_field();
    assert_scalar(
        &logical_hadamard(&a, &b).unwrap(),
        -FRAC_1_SQRT_2,
        -FRAC_1_SQRT_2,
        "logical_hadamard(1,1)",
    );
}

#[test]
fn test_logical_cz_gate() {
    // CZ(1,1) = exp(i·π) = -1.
    let a1 = create_projector_field();
    let a2 = create_projector_field();
    assert_scalar(&logical_cz(&a1, &a2).unwrap(), -1.0, 0.0, "logical_cz(1,1)");
}

#[test]
fn test_logical_t_gate() {
    // T(1) = exp(i·π·(½·1³ − ¾·1² + ½·1)) = exp(i·π/4) = 1/√2 + i/√2.
    assert_scalar(
        &logical_t(&create_projector_field()).unwrap(),
        FRAC_1_SQRT_2,
        FRAC_1_SQRT_2,
        "logical_t(1)",
    );
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

#[test]
fn test_non_convergent_exponent_errors() {
    // A scalar field 7 gives logical_z the exponent i·π·7 ≈ i·22. The norm ~22 is
    // in the (16, 1e6) band: it clears the overflow guard but the 64-term Taylor
    // series cannot converge to 1e-12, so the truncated sum would be silently
    // inaccurate. exp must report non-convergence (CalculationError), not Ok.
    let mut data = vec![Complex::new(0.0, 0.0); 8];
    data[0] = Complex::new(7.0, 0.0);
    let a = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    assert!(matches!(
        logical_z(&a).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
}
