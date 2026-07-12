/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    QuantumErrorEnum, apply_choi, apply_kraus, check_completely_positive, check_trace_preserving,
    choi_from_kraus, frobenius_norm, identity_matrix, kraus_from_choi, matrix_trace,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, rows: usize, cols: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

fn max_abs_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    assert_eq!(
        a.shape(),
        b.shape(),
        "max_abs_diff shape mismatch: {:?} vs {:?}",
        a.shape(),
        b.shape()
    );
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

/// The qubit depolarizing channel with parameter p as a 4-element Kraus family.
fn depolarizing_kraus(p: f64) -> Vec<CausalTensor<C>> {
    let s0 = (1.0 - 3.0 * p / 4.0).sqrt();
    let s = (p / 4.0_f64).sqrt();
    vec![
        mat(vec![c(s0, 0.), c(0., 0.), c(0., 0.), c(s0, 0.)], 2, 2), // √(1−3p/4)·I
        mat(vec![c(0., 0.), c(s, 0.), c(s, 0.), c(0., 0.)], 2, 2),   // √(p/4)·σx
        mat(vec![c(0., 0.), c(0., -s), c(0., s), c(0., 0.)], 2, 2),  // √(p/4)·σy
        mat(vec![c(s, 0.), c(0., 0.), c(0., 0.), c(-s, 0.)], 2, 2),  // √(p/4)·σz
    ]
}

#[test]
fn test_identity_channel_choi_is_maximally_entangled() {
    // J(id) = Σ_{ik} |i⟩⟨k| ⊗ |i⟩⟨k| — the unnormalized maximally entangled
    // projector with Tr J = d_in and J² = d·J (rank 1 · d).
    let id = identity_matrix::<f64>(2);
    let j = choi_from_kraus(&[id]).unwrap();
    let tr = matrix_trace(&j).unwrap();
    assert!((tr.re - 2.0).abs() < 1e-12);
    check_completely_positive(&j, 1e-12).unwrap();
    check_trace_preserving(&j, 2, 2, 1e-12).unwrap();
}

#[test]
fn test_cptp_checks_on_depolarizing_channel() {
    let j = choi_from_kraus(&depolarizing_kraus(0.3)).unwrap();
    check_completely_positive(&j, 1e-12).unwrap();
    check_trace_preserving(&j, 2, 2, 1e-12).unwrap();
}

#[test]
fn test_non_tp_family_rejected() {
    // A single non-isometric Kraus operator (0.5·I) is CP but not TP.
    let k = mat(vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], 2, 2);
    let j = choi_from_kraus(&[k]).unwrap();
    check_completely_positive(&j, 1e-12).unwrap();
    assert!(check_trace_preserving(&j, 2, 2, 1e-12).is_err());
}

#[test]
fn test_non_cp_operator_rejected() {
    // The transpose map's Choi (the swap operator) has a −1 eigenvalue.
    // Swap on C²⊗C²: J[(i,j),(k,l)] = δ_il·δ_jk.
    let mut data = vec![c(0., 0.); 16];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                for l in 0..2 {
                    if i == l && j == k {
                        data[(i * 2 + j) * 4 + (k * 2 + l)] = c(1., 0.);
                    }
                }
            }
        }
    }
    let swap = CausalTensor::new(data, vec![4, 4]).unwrap();
    assert!(check_completely_positive(&swap, 1e-12).is_err());
}

#[test]
fn test_choi_kraus_choi_round_trip() {
    // Choi → Kraus → Choi is the identity up to numerical tolerance.
    let j = choi_from_kraus(&depolarizing_kraus(0.37)).unwrap();
    let kraus = kraus_from_choi(&j, 2, 2, 1e-12).unwrap();
    let j2 = choi_from_kraus(&kraus).unwrap();
    assert!(
        max_abs_diff(&j, &j2) < 1e-10,
        "round trip drifted: {}",
        max_abs_diff(&j, &j2)
    );
}

// The Kraus↔Choi application-agreement and the ℂ-linearity of apply_choi are
// the THEOREM_MAP witnesses for Quantum/Choi.lean; they live in
// tests/formalization_lean/choi_tests.rs.

#[test]
fn test_depolarizing_contracts_toward_maximally_mixed() {
    // Full depolarizing (p = 1) sends every state to I/2.
    let kraus = depolarizing_kraus(1.0);
    let rho = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2, 2); // |0⟩⟨0|
    let out = apply_kraus(&kraus, &rho).unwrap();
    let half_id = mat(vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], 2, 2);
    assert!(max_abs_diff(&out, &half_id) < 1e-12);
}

#[test]
fn test_kraus_rejections() {
    assert!(choi_from_kraus::<f64>(&[]).is_err());
    let k2 = identity_matrix::<f64>(2);
    let k3 = identity_matrix::<f64>(3);
    assert!(choi_from_kraus(&[k2.clone(), k3]).is_err());

    let rho3 = identity_matrix::<f64>(3);
    assert!(apply_kraus(&[k2], &rho3).is_err());

    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    assert!(kraus_from_choi(&j, 3, 2, 1e-12).is_err()); // wrong (d_in, d_out)
    let zero = CausalTensor::new(vec![c(0., 0.); 16], vec![4, 4]).unwrap();
    assert!(kraus_from_choi(&zero, 2, 2, 1e-12).is_err()); // zero channel
    assert!(frobenius_norm(&j) > 0.0);
}

// =============================================================================
// Error-path coverage (llvm-cov gap closure).
// =============================================================================

#[test]
fn test_choi_from_kraus_rejects_non_matrix_operator() {
    // A rank-1 tensor is not a Kraus matrix.
    let bad = CausalTensor::new(vec![c(1., 0.), c(0., 0.)], vec![2]).unwrap();
    assert!(matches!(
        choi_from_kraus(&[bad]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_kraus_from_choi_rejects_negative_eigenvalue() {
    // The swap operator (the transpose map's Choi) has a −1 eigenvalue, so it is
    // not a CP channel and no Kraus family can be recovered from it.
    let mut data = vec![c(0., 0.); 16];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                for l in 0..2 {
                    if i == l && j == k {
                        data[(i * 2 + j) * 4 + (k * 2 + l)] = c(1., 0.);
                    }
                }
            }
        }
    }
    let swap = CausalTensor::new(data, vec![4, 4]).unwrap();
    assert!(matches!(
        kraus_from_choi(&swap, 2, 2, 1e-12).unwrap_err().0,
        QuantumErrorEnum::NonCptpChannel(_)
    ));
}

#[test]
fn test_apply_kraus_rejects_empty_family() {
    let rho = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2, 2);
    assert!(matches!(
        apply_kraus(&[], &rho).unwrap_err().0,
        QuantumErrorEnum::NonCptpChannel(_)
    ));
}

#[test]
fn test_apply_choi_rejects_dimension_mismatch() {
    // A (2,2) channel's Choi is 4×4; asking it to act as a (3,2) channel fails.
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    let rho = mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0., 0.)], 2, 2);
    assert!(matches!(
        apply_choi(&j, &rho, 3, 2).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_check_trace_preserving_rejects_dimension_mismatch() {
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap(); // 4×4
    assert!(matches!(
        check_trace_preserving(&j, 3, 2, 1e-12).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_tolerance_validation_rejects_nonfinite_and_negative() {
    // A NaN/∞/negative tolerance makes every `defect > tol` / `λ < -tol` check
    // vacuously false, silently certifying an invalid operator; reject it.
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    for bad in [f64::NAN, f64::INFINITY, -1e-9] {
        assert!(matches!(
            kraus_from_choi(&j, 2, 2, bad).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
        assert!(matches!(
            check_completely_positive(&j, bad).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
        assert!(matches!(
            check_trace_preserving(&j, 2, 2, bad).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
}
