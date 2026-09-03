/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The report-returning forms of the CPTP checks.
//!
//! The shipped `check_completely_positive` and `check_trace_preserving` compute a Hermiticity
//! defect, a spectrum and a trace defect, decide, and discard them. These siblings keep what was
//! computed, on the accepting path as well as the rejecting one, and the shipped forms keep their
//! signatures and their answers.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CheckItem, CheckVerdict, QuantumErrorEnum, check_completely_positive,
    check_completely_positive_report, check_trace_preserving, check_trace_preserving_report,
    choi_from_kraus, identity_matrix,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, rows: usize, cols: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

/// The qubit depolarizing channel with parameter p as a 4-element Kraus family.
fn depolarizing_kraus(p: f64) -> Vec<CausalTensor<C>> {
    let s0 = (1.0 - 3.0 * p / 4.0).sqrt();
    let s = (p / 4.0_f64).sqrt();
    vec![
        mat(vec![c(s0, 0.), c(0., 0.), c(0., 0.), c(s0, 0.)], 2, 2),
        mat(vec![c(0., 0.), c(s, 0.), c(s, 0.), c(0., 0.)], 2, 2),
        mat(vec![c(0., 0.), c(0., -s), c(0., s), c(0., 0.)], 2, 2),
        mat(vec![c(s, 0.), c(0., 0.), c(0., 0.), c(-s, 0.)], 2, 2),
    ]
}

/// The transpose map's Choi operator, the swap on C²⊗C², which has a −1 eigenvalue.
fn swap() -> CausalTensor<C> {
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
    CausalTensor::new(data, vec![4, 4]).unwrap()
}

const TOL: f64 = 1e-12;

#[test]
fn test_a_cp_channel_reports_its_spectrum_and_the_number_of_eigenvalues() {
    let j = choi_from_kraus(&depolarizing_kraus(0.3)).unwrap();
    let report = check_completely_positive_report(&j, TOL).unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    // Four eigenvalues examined; five records, the Hermiticity defect and one per eigenvalue.
    assert_eq!(report.examined(), 4);
    assert_eq!(report.checks().len(), 5);
    assert_eq!(report.checks()[0].item, CheckItem::Whole);
    assert!(report.checks()[0].measured <= TOL);
    // Measured as −λ, so a PSD spectrum reads as non-positive margins.
    for (i, check) in report.checks()[1..].iter().enumerate() {
        assert_eq!(check.item, CheckItem::Index(i));
        assert!(check.measured <= TOL, "eigenvalue {i} read as negative");
    }
    assert!(report.worst_margin().unwrap() <= 1.0);
    // And the shipped form agrees.
    check_completely_positive(&j, TOL).unwrap();
}

#[test]
fn test_a_non_cp_operator_reports_the_negative_eigenvalue_and_the_shipped_form_still_errs() {
    let s = swap();
    let report = check_completely_positive_report(&s, TOL).unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert_eq!(report.examined(), 4);
    let rejecting = report.first_rejection().expect("a −1 eigenvalue rejects");
    assert!(matches!(rejecting.item, CheckItem::Index(_)));
    // The swap has eigenvalue −1, so the measured −λ is 1 against 1e-12.
    assert!((rejecting.measured - 1.0).abs() < 1e-9);
    assert!(rejecting.margin > 1.0);
    // The worst record is the minimum eigenvalue.
    assert_eq!(report.worst().unwrap().item, rejecting.item);

    assert!(matches!(
        check_completely_positive(&s, TOL).unwrap_err().0,
        QuantumErrorEnum::NonCptpChannel(_)
    ));
}

#[test]
fn test_a_non_hermitian_input_records_the_defect_and_examines_no_spectrum() {
    // A spectrum through eigen_hermitian would certify the Hermitian part of the wrong operator,
    // so the report stops at the defect: one record, rejected, nothing examined, and the verdict
    // is a rejection rather than a vacuous pass because rejection is decided first.
    let mut data = vec![c(0., 0.); 16];
    data[1] = c(1., 0.);
    let non_hermitian = CausalTensor::new(data, vec![4, 4]).unwrap();
    let report = check_completely_positive_report(&non_hermitian, TOL).unwrap();
    assert_eq!(report.checks().len(), 1);
    assert_eq!(report.checks()[0].item, CheckItem::Whole);
    assert!(!report.checks()[0].accepted);
    assert_eq!(report.examined(), 0);
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert!(matches!(
        check_completely_positive(&non_hermitian, TOL).unwrap_err().0,
        QuantumErrorEnum::NonPositiveOperator(_)
    ));
}

#[test]
fn test_the_tp_defect_reaches_the_caller_on_the_accepting_path() {
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    let report = check_trace_preserving_report(&j, 2, 2, TOL).unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    // One record over the d_in × d_in residual's four entries.
    assert_eq!(report.checks().len(), 1);
    assert_eq!(report.examined(), 4);
    let record = &report.checks()[0];
    assert_eq!(record.item, CheckItem::Whole);
    assert!(record.measured <= TOL);
    assert_eq!(record.threshold, TOL);
    check_trace_preserving(&j, 2, 2, TOL).unwrap();
}

#[test]
fn test_a_non_tp_family_reports_its_defect_and_the_shipped_form_still_errs() {
    // 0.5·I is CP but not TP: Tr_out(J) = 0.25·I, so the defect is 0.75.
    let k = mat(vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)], 2, 2);
    let j = choi_from_kraus(&[k]).unwrap();
    let report = check_trace_preserving_report(&j, 2, 2, TOL).unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    let record = &report.checks()[0];
    assert!((record.measured - 0.75).abs() < 1e-12, "defect {}", record.measured);
    assert!(record.margin > 1.0);
    assert_eq!(report.examined(), 4);
    assert!(check_trace_preserving(&j, 2, 2, TOL).is_err());
}

#[test]
fn test_structural_failures_stay_on_the_error_path() {
    let j = choi_from_kraus(&[identity_matrix::<f64>(2)]).unwrap();
    // A dimension mismatch is not a rejection.
    assert!(matches!(
        check_trace_preserving_report(&j, 3, 2, TOL).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // Nor is an invalid tolerance.
    assert!(check_completely_positive_report(&j, -1.0).is_err());
    assert!(check_trace_preserving_report(&j, 2, 2, f64::NAN).is_err());
}
