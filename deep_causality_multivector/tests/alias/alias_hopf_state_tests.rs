/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{HilbertState, HopfState, Metric};
use deep_causality_num::Complex;
use std::f64::consts::PI;

const F64_EPSILON: f64 = 1.0e-12; // Custom epsilon for floating-point comparisons

#[test]
fn test_new_hopf_state_success() {
    // A simple rotor for a 90-degree rotation around Z-axis
    // R = cos(PI/4) + e12 * sin(PI/4) = 1/sqrt(2) + e12 * 1/sqrt(2)
    let s = 1.0 / (2.0f64).sqrt();
    let data = vec![s, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data).unwrap();
    // Normalization should keep the values same for an already normalized input
    assert!((hopf.as_inner().data()[0] - s).abs() < F64_EPSILON);
    assert!((hopf.as_inner().data()[3] - s).abs() < F64_EPSILON);
    assert_eq!(hopf.as_inner().metric(), Metric::Euclidean(3));
}

#[test]
fn test_new_hopf_state_normalization() {
    let s = 1.0; // Not normalized
    let data = vec![s, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data).unwrap();
    let expected_s = 1.0 / (2.0f64).sqrt();
    assert!((hopf.as_inner().data()[0] - expected_s).abs() < F64_EPSILON);
    assert!((hopf.as_inner().data()[3] - expected_s).abs() < F64_EPSILON);
}

#[test]
fn test_new_hopf_state_error_data_length_mismatch() {
    let data = vec![1.0, 0.0, 0.0, 0.0]; // Incorrect length
    let err = HopfState::new(data).unwrap_err();
    assert_eq!(err.to_string(), "Data length mismatch: expected 8, found 4");
}

#[test]
fn test_from_spinor() {
    // |psi> = 1/sqrt(2) |0> + 1/sqrt(2) |1>
    let s = Complex::new(1.0 / (2.0f64).sqrt(), 0.0);
    let alpha = s;
    let beta = s;

    let hopf = HopfState::from_spinor(alpha, beta);

    // Expected data based on the mapping in `from_spinor`:
    // data[0] = alpha.re = s
    // data[3] = alpha.im = 0.0
    // data[5] = beta.im = 0.0
    // data[6] = beta.re = s
    // All other components should be 0.0.
    // The resulting MV is (s + s*e23), which should be normalized.
    // A = s + s e_23 -> A A_ = s^2 + s^2 = 2s^2
    // Normalization = A / sqrt(2s^2) = A / (s*sqrt(2))
    // So, scalar and e23 components should be (s / (s*sqrt(2))) = 1/sqrt(2)
    let expected_s_norm = 1.0 / (2.0f64).sqrt();

    assert!((hopf.as_inner().data()[0] - expected_s_norm).abs() < F64_EPSILON);
    assert!((hopf.as_inner().data()[6] - expected_s_norm).abs() < F64_EPSILON);
    assert!((hopf.as_inner().data()[3] - 0.0).abs() < F64_EPSILON); // alpha.im
    assert!((hopf.as_inner().data()[5] - 0.0).abs() < F64_EPSILON); // beta.im
    assert_eq!(hopf.as_inner().metric(), Metric::Euclidean(3));
}

#[test]
fn test_project_north_pole() {
    // Identity rotor (scalar 1)
    let data = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data).unwrap();

    let projection = hopf.project();
    // For identity rotor, R * e3 * ~R should give e3
    // e3 is index 4
    let expected_data = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    assert_eq!(projection.data(), &expected_data);
    assert_eq!(projection.metric(), Metric::Euclidean(3));
}

#[test]
fn test_project_rotated_state() {
    // Rotor for 90-degree rotation around Y-axis (e31)
    // R = cos(PI/4) + e31 * sin(PI/4)
    let s = 1.0 / (2.0f64).sqrt();
    let data = vec![s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0]; // e31 is index 5
    let hopf = HopfState::new(data).unwrap();

    let projection = hopf.project();
    // Applying R * e3 * ~R (R rotates e3 to ex)
    // R = exp(e31 * PI/4). R e3 R~ = e1 (rotated by PI/2 around Y)
    // So, it should project to e1 (index 1)
    assert!((projection.data()[1] - 1.0).abs() < F64_EPSILON); // Expect e1 = 1.0
    assert_eq!(projection.data()[0], 0.0);
    assert_eq!(projection.data()[4], 0.0); // No e3 component
    assert_eq!(projection.metric(), Metric::Euclidean(3));
}

#[test]
fn test_fiber_shift() {
    // Initial state: identity rotor (scalar 1)
    let data = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data).unwrap();

    // Shift by PI (180 degrees)
    let shifted_hopf = hopf.fiber_shift(PI);

    // R' = R * e^(-I * PI/2) = (1) * (cos(PI/2) - e12*sin(PI/2)) = -e12
    // So, the scalar part should be 0, and e12 (index 3) should be -1.0
    assert!((shifted_hopf.as_inner().data()[0] - 0.0).abs() < F64_EPSILON);
    assert!((shifted_hopf.as_inner().data()[3] - (-1.0)).abs() < F64_EPSILON);
    assert_eq!(shifted_hopf.as_inner().metric(), Metric::Euclidean(3));

    // Verify projection is unchanged (from e3 to -e3 or similar, but the vector part should be e3)
    let original_projection = hopf.project();
    let shifted_projection = shifted_hopf.project();
    assert_eq!(original_projection.data(), shifted_projection.data());
}

#[test]
fn test_as_inner() {
    let data = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data.clone()).unwrap();
    let inner_mv = hopf.as_inner();
    assert_eq!(inner_mv.data()[0], 1.0);
    assert_eq!(inner_mv.metric(), Metric::Euclidean(3));
}

#[test]
fn test_try_from_hilbert_state_success() {
    // |psi> = 1/sqrt(2) |0> + 1/sqrt(2) |1>
    let s = Complex::new(1.0 / (2.0f64).sqrt(), 0.0);
    let mut hilbert_data = vec![Complex::new(0.0, 0.0); 1024];
    hilbert_data[0] = s; // alpha
    hilbert_data[1] = s; // beta
    let hilbert_state = HilbertState::new_spin10(hilbert_data).unwrap();

    let hopf_state = HopfState::try_from(&hilbert_state).unwrap();

    // Expected data based on from_spinor:
    // Scalar (data[0]) = alpha.re = s
    // e12 (data[3]) = alpha.im = 0
    // e13 (data[5]) = beta.im = 0
    // e23 (data[6]) = beta.re = s
    let expected_s_norm = 1.0 / (2.0f64).sqrt();
    assert!((hopf_state.as_inner().data()[0] - expected_s_norm).abs() < F64_EPSILON);
    assert!((hopf_state.as_inner().data()[6] - expected_s_norm).abs() < F64_EPSILON);
    assert!((hopf_state.as_inner().data()[3] - 0.0).abs() < F64_EPSILON);
    assert!((hopf_state.as_inner().data()[5] - 0.0).abs() < F64_EPSILON);
}

#[test]
fn test_new_hilbert_state_error_dimension_mismatch() {
    // HilbertState with insufficient data for HopfState (needs at least 2 for alpha/beta extraction)
    let hilbert_data = vec![Complex::new(1.0, 0.0); 1]; // Only 1 element
    let res = HilbertState::new(hilbert_data, Metric::NonEuclidean(1)); // Create a valid HilbertState
    assert!(res.is_err());
}

#[test]
fn test_from_hopf_state_to_hilbert_state() {
    // Create a HopfState corresponding to a known Spinor
    // R = 1/sqrt(2) + 1/sqrt(2) e23
    let s = 1.0 / (2.0f64).sqrt();
    let data = vec![s, 0.0, 0.0, 0.0, 0.0, 0.0, s, 0.0];
    let hopf_state = HopfState::new(data).unwrap();

    let hilbert_state = HilbertState::from(hopf_state);

    // Expected HilbertState: alpha = Complex(s, 0), beta = Complex(s, 0)
    let expected_s = Complex::new(1.0 / (2.0f64).sqrt(), 0.0);

    let hilbert_data = hilbert_state.as_inner().data();
    assert!((hilbert_data[0].re - expected_s.re).abs() < F64_EPSILON); // alpha re
    assert!((hilbert_data[0].im - expected_s.im).abs() < F64_EPSILON); // alpha im
    assert!((hilbert_data[1].re - expected_s.re).abs() < F64_EPSILON); // beta re
    assert!((hilbert_data[1].im - expected_s.im).abs() < F64_EPSILON); // beta im
    assert_eq!(hilbert_state.as_inner().metric(), Metric::NonEuclidean(10));
}

#[test]
fn test_hopf_state_display() {
    let data = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let hopf = HopfState::new(data).unwrap();
    assert_eq!(
        format!("{}", hopf),
        "HopfState(CausalMultiVector { data: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], metric: Euclidean(3) })"
    );
}
