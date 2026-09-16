/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{HilbertState, HopfState, Metric, MultiVector};
use deep_causality_num_complex::Complex;
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

    // A rotor built from a normalized spinor is already a unit rotor, so `from_spinor` returns it
    // with its magnitude untouched.
    assert!((hopf.as_inner().squared_magnitude() - 1.0).abs() < F64_EPSILON);
    assert_eq!(hopf.as_inner().metric(), Metric::Euclidean(3));
}

/// The projection carries the Bloch vector of the spinor it was built from.
///
/// For `|psi> = alpha|0> + beta|1>` the Bloch vector is
///
/// ```text
/// n_x = 2 Re(conj(alpha) beta)   n_y = 2 Im(conj(alpha) beta)   n_z = |alpha|^2 - |beta|^2
/// ```
///
/// so the six cardinal states land on the six semi-axes. This is the property that fixes where
/// `beta.re` and `beta.im` belong among the bivector coefficients: exchanging the two leaves every
/// state normalized and every round trip intact, and silently transposes the x and y axes of every
/// projection.
#[test]
fn test_project_bloch_vector_of_the_six_cardinal_states() {
    // e1 is index 1, e2 is index 2, e3 is index 4.
    const E1: usize = 1;
    const E2: usize = 2;
    const E3: usize = 4;

    let s = 1.0 / (2.0f64).sqrt();
    let zero = Complex::new(0.0, 0.0);

    // (name, alpha, beta, expected Bloch vector)
    let cardinal = [
        ("|0>", Complex::new(1.0, 0.0), zero, [0.0, 0.0, 1.0]),
        ("|1>", zero, Complex::new(1.0, 0.0), [0.0, 0.0, -1.0]),
        (
            "|+>",
            Complex::new(s, 0.0),
            Complex::new(s, 0.0),
            [1.0, 0.0, 0.0],
        ),
        (
            "|->",
            Complex::new(s, 0.0),
            Complex::new(-s, 0.0),
            [-1.0, 0.0, 0.0],
        ),
        (
            "|+i>",
            Complex::new(s, 0.0),
            Complex::new(0.0, s),
            [0.0, 1.0, 0.0],
        ),
        (
            "|-i>",
            Complex::new(s, 0.0),
            Complex::new(0.0, -s),
            [0.0, -1.0, 0.0],
        ),
    ];

    for (name, alpha, beta, expected) in cardinal {
        let bloch = HopfState::from_spinor(alpha, beta).project();
        let got = [
            bloch.data()[E1],
            bloch.data()[E2],
            bloch.data()[E3],
        ];

        for (axis, (got, want)) in got.iter().zip(expected.iter()).enumerate() {
            assert!(
                (got - want).abs() < F64_EPSILON,
                "{name}: axis {axis} projected to {got}, expected {want}"
            );
        }
    }
}

/// Every state on the Bloch sphere, not only the cardinal six.
#[test]
fn test_project_bloch_vector_off_axis() {
    const E1: usize = 1;
    const E2: usize = 2;
    const E3: usize = 4;

    // |psi> = cos(theta/2)|0> + e^{i phi} sin(theta/2)|1> sits at (theta, phi) in spherical
    // coordinates, so its Bloch vector is the point itself.
    let theta: f64 = 0.7;
    let phi: f64 = 1.3;

    let alpha = Complex::new((theta / 2.0).cos(), 0.0);
    let beta = Complex::new(
        phi.cos() * (theta / 2.0).sin(),
        phi.sin() * (theta / 2.0).sin(),
    );

    let bloch = HopfState::from_spinor(alpha, beta).project();

    let expected = [
        theta.sin() * phi.cos(),
        theta.sin() * phi.sin(),
        theta.cos(),
    ];
    let got = [bloch.data()[E1], bloch.data()[E2], bloch.data()[E3]];

    for (axis, (got, want)) in got.iter().zip(expected.iter()).enumerate() {
        assert!(
            (got - want).abs() < F64_EPSILON,
            "axis {axis} projected to {got}, expected {want}"
        );
    }
}

/// A fiber shift moves the state along `S^1` and leaves the projection where it was. That is the
/// defining property of the fibration, and the global phase of quantum mechanics.
#[test]
fn test_fiber_shift_moves_the_state_and_fixes_the_projection() {
    let s = 1.0 / (2.0f64).sqrt();
    let state = HopfState::from_spinor(Complex::new(s, 0.0), Complex::new(s, 0.0));
    let before = state.project();

    for turns in 1..8 {
        let shifted = state.fiber_shift(PI / 4.0 * f64::from(turns));
        let after = shifted.project();

        assert!(
            (before.clone() - after).squared_magnitude() < F64_EPSILON,
            "projection moved after a fiber shift of {turns} eighth-turns"
        );
        assert!(
            (state.as_inner() - shifted.as_inner()).squared_magnitude() > F64_EPSILON,
            "state stood still under a fiber shift of {turns} eighth-turns"
        );
    }
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

    // Verify projection is unchanged (from e3 to -e3 or similar, but the vector part should be e3).
    // Per-element tolerance comparison: the shifted projection runs through trig + multiplies,
    // so under soft-float emulation (Miri) it can drift by ~1 ULP from the original.
    let original_projection = hopf.project();
    let shifted_projection = shifted_hopf.project();
    let original_data = original_projection.data();
    let shifted_data = shifted_projection.data();
    assert_eq!(original_data.len(), shifted_data.len());
    for (o, s) in original_data.iter().zip(shifted_data.iter()) {
        assert!(
            (o - s).abs() < F64_EPSILON,
            "projection drift: original {} vs shifted {}",
            o,
            s
        );
    }
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
    let hilbert_state = HilbertState::<f64>::new_spin10(hilbert_data).unwrap();

    let hopf_state = HopfState::try_from(&hilbert_state).unwrap();

    // The conversion reads the first two amplitudes as the spinor, so the rotor it returns is the
    // one `from_spinor` builds, and it projects onto the same Bloch vector: |+> sits on +x.
    let bloch = hopf_state.project();
    assert!((bloch.data()[1] - 1.0).abs() < F64_EPSILON); // e1
    assert!((bloch.data()[2] - 0.0).abs() < F64_EPSILON); // e2
    assert!((bloch.data()[4] - 0.0).abs() < F64_EPSILON); // e3
}

#[test]
fn test_new_hilbert_state_error_dimension_mismatch() {
    // HilbertState with insufficient data for HopfState (needs at least 2 for alpha/beta extraction)
    let hilbert_data = vec![Complex::new(1.0, 0.0); 1]; // Only 1 element
    let res = HilbertState::<f64>::new(hilbert_data, Metric::NonEuclidean(1)); // Create a valid HilbertState
    assert!(res.is_err());
}

/// The two conversions invert each other: a spinor that goes out to the rotor and back arrives
/// unchanged. This holds for either placement of `beta.re` and `beta.im`, which is why it is
/// checked alongside the projection rather than instead of it.
#[test]
fn test_hilbert_hopf_round_trip() {
    let s = 1.0 / (2.0f64).sqrt();
    let spinors = [
        (Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)),
        (Complex::new(s, 0.0), Complex::new(s, 0.0)),
        (Complex::new(s, 0.0), Complex::new(0.0, s)),
        (Complex::new(0.6, 0.0), Complex::new(0.48, 0.64)),
    ];

    for (alpha, beta) in spinors {
        let hopf = HopfState::from_spinor(alpha, beta);
        let hilbert = HilbertState::try_from(hopf).unwrap();
        let back = hilbert.as_inner().data();

        assert!((back[0].re - alpha.re).abs() < F64_EPSILON);
        assert!((back[0].im - alpha.im).abs() < F64_EPSILON);
        assert!((back[1].re - beta.re).abs() < F64_EPSILON);
        assert!((back[1].im - beta.im).abs() < F64_EPSILON);
        assert_eq!(hilbert.as_inner().metric(), Metric::NonEuclidean(10));
    }
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
