/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `QubitOperator`: built by name, unitary by construction, sealed.
//!
//! The matrices are Nielsen & Chuang §4.2: the Paulis, `H = (X + Z)/√2`, `R_n(θ) = cos(θ/2) I −
//! i sin(θ/2) σ_n`, and `P(θ) = diag(1, e^{iθ})`.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Axis, QuantumErrorEnum, QubitOperator};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn entry(op: &QubitOperator<f64>, i: usize, j: usize) -> C {
    op.matrix().as_slice()[i * 2 + j]
}

fn close(a: C, re: f64, im: f64) -> bool {
    (a.re - re).abs() < 1e-12 && (a.im - im).abs() < 1e-12
}

#[test]
fn test_a_named_constructor_replaces_a_packed_slice() {
    let x = QubitOperator::<f64>::pauli_x();
    assert_eq!(x.dim(), 2);
    assert_eq!(x.matrix().shape(), &[2, 2]);
    assert!(close(entry(&x, 0, 1), 1.0, 0.0));
    assert!(close(entry(&x, 1, 0), 1.0, 0.0));
    assert!(close(entry(&x, 0, 0), 0.0, 0.0));
    let y = QubitOperator::<f64>::pauli_y();
    assert!(close(entry(&y, 0, 1), 0.0, -1.0));
    assert!(close(entry(&y, 1, 0), 0.0, 1.0));
    let z = QubitOperator::<f64>::pauli_z();
    assert!(close(entry(&z, 1, 1), -1.0, 0.0));
    let h = QubitOperator::<f64>::hadamard();
    let r = std::f64::consts::FRAC_1_SQRT_2;
    assert!(close(entry(&h, 0, 0), r, 0.0));
    assert!(close(entry(&h, 1, 1), -r, 0.0));
}

#[test]
fn test_the_named_constructors_are_unitary_within_sqrt_epsilon() {
    let tol = f64::EPSILON.sqrt();
    let ops = [
        QubitOperator::<f64>::identity(),
        QubitOperator::pauli_x(),
        QubitOperator::pauli_y(),
        QubitOperator::pauli_z(),
        QubitOperator::hadamard(),
        QubitOperator::rotation(Axis::X, 0.7).unwrap(),
        QubitOperator::rotation(Axis::Y, 2.3).unwrap(),
        QubitOperator::rotation(Axis::Z, -1.1).unwrap(),
        QubitOperator::phase(0.4).unwrap(),
    ];
    for op in &ops {
        assert!(
            op.unitarity_defect() <= tol,
            "defect {}",
            op.unitarity_defect()
        );
    }
}

#[test]
fn test_rotations_and_phase_match_the_textbook() {
    // R_x(π) = −iX, R_z(θ) = diag(e^{−iθ/2}, e^{iθ/2}), P(π/2) = S.
    let rx = QubitOperator::<f64>::rotation(Axis::X, std::f64::consts::PI).unwrap();
    assert!(close(entry(&rx, 0, 1), 0.0, -1.0));
    assert!(close(entry(&rx, 0, 0), 0.0, 0.0));
    let rz = QubitOperator::<f64>::rotation(Axis::Z, 1.0).unwrap();
    assert!(close(entry(&rz, 0, 0), 0.5f64.cos(), -(0.5f64.sin())));
    assert!(close(entry(&rz, 1, 1), 0.5f64.cos(), 0.5f64.sin()));
    let s = QubitOperator::<f64>::phase(std::f64::consts::FRAC_PI_2).unwrap();
    assert!(close(entry(&s, 1, 1), 0.0, 1.0));
}

#[test]
fn test_a_non_finite_parameter_is_rejected_at_construction() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            QubitOperator::<f64>::rotation(Axis::Y, bad).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
        assert!(matches!(
            QubitOperator::<f64>::phase(bad).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
}

#[test]
fn test_from_matrix_admits_a_unitary_and_refuses_the_rest() {
    let c = |re: f64, im: f64| Complex::new(re, im);
    // A unitary not in the named alphabet: e^{iπ/4}·X.
    let phase = std::f64::consts::FRAC_PI_4;
    let u = CausalTensor::from_slice(
        &[
            c(0.0, 0.0),
            c(phase.cos(), phase.sin()),
            c(phase.cos(), phase.sin()),
            c(0.0, 0.0),
        ],
        &[2, 2],
    );
    assert!(QubitOperator::from_matrix(u).is_ok());
    // Not unitary: 2·I.
    let two = CausalTensor::from_slice(
        &[c(2.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(2.0, 0.0)],
        &[2, 2],
    );
    assert!(matches!(
        QubitOperator::from_matrix(two).unwrap_err().0,
        QuantumErrorEnum::NonPositiveOperator(_)
    ));
    // Wrong shape.
    let three = CausalTensor::from_slice(&[c(1.0, 0.0); 9], &[3, 3]);
    assert!(matches!(
        QubitOperator::from_matrix(three).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // Non-finite entry.
    let nan = CausalTensor::from_slice(
        &[c(f64::NAN, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)],
        &[2, 2],
    );
    assert!(matches!(
        QubitOperator::from_matrix(nan).unwrap_err().0,
        QuantumErrorEnum::NonFiniteValue(_)
    ));
}

#[test]
fn test_the_precision_lift_composes_two_functors_and_re_validates_at_the_target() {
    // Down, f64 → f32: the outer functor maps cells, the inner maps re and im, and the target
    // re-validates unitarity at its own √ε, which the source's rounding is far inside.
    let r = QubitOperator::<f64>::rotation(Axis::Y, 0.9).unwrap();
    let narrow: QubitOperator<f32> = r.lift(|x| x as f32).unwrap();
    assert!(narrow.unitarity_defect() <= f32::EPSILON.sqrt());
    for (a, b) in r.matrix().as_slice().iter().zip(narrow.matrix().as_slice()) {
        assert!((a.re - b.re as f64).abs() < 1e-6 && (a.im - b.im as f64).abs() < 1e-6);
    }

    // Up, f32 → f64: the f32 rounding of about 6e-8 per entry is visible at f64's √ε ≈ 1.5e-8,
    // and the target refuses. The check tightened with the scalar; the data did not.
    let err = narrow.lift(|x| x as f64).unwrap_err();
    match err.0 {
        QuantumErrorEnum::NonPositiveOperator(msg) => assert!(msg.contains("not unitary"), "{msg}"),
        other => panic!("expected NonPositiveOperator, got {other:?}"),
    }

    // Up with exactly representable entries: X is 0s and 1s at every scalar, so it lifts cleanly.
    let x32 = QubitOperator::<f32>::pauli_x();
    let x64: QubitOperator<f64> = x32.lift(|x| x as f64).unwrap();
    assert_eq!(x64, QubitOperator::pauli_x());
}

#[test]
fn test_the_unitarity_check_conjugates_and_names_the_defect() {
    let c = |re: f64, im: f64| Complex::new(re, im);
    // i·X is unitary only because U† conjugates: without the conjugate the (0, 0) entry of
    // UU† would be i·i = −1 and the defect 2.
    let ix = CausalTensor::from_slice(
        &[c(0.0, 0.0), c(0.0, 1.0), c(0.0, 1.0), c(0.0, 0.0)],
        &[2, 2],
    );
    let op = QubitOperator::from_matrix(ix).unwrap();
    assert!(op.unitarity_defect() <= f64::EPSILON);
    // 2i·X has UU† = 4I: the defect is |4 − 1| = 3 on the diagonal, and the refusal names it.
    let two_ix = CausalTensor::from_slice(
        &[c(0.0, 0.0), c(0.0, 2.0), c(0.0, 2.0), c(0.0, 0.0)],
        &[2, 2],
    );
    match QubitOperator::from_matrix(two_ix).unwrap_err().0 {
        QuantumErrorEnum::NonPositiveOperator(msg) => {
            assert!(msg.contains("= 3.0 >"), "names the defect: {msg}")
        }
        other => panic!("expected NonPositiveOperator, got {other:?}"),
    }
}

#[test]
fn test_the_default_is_the_identity() {
    assert_eq!(QubitOperator::<f64>::default(), QubitOperator::identity());
}
