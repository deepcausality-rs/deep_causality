/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The report-returning form of the Markov check, and the certificate it grants.
//!
//! Two obligations from the decision form: a factorization whose supports never overlap reaches
//! `Ok` having tested zero pairs and must read as vacuous, and a rejected candidate must still
//! report its margins and its count, which the legacy form drops with the error.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CheckItem, CheckVerdict, CommutatorTolerance, FactorSupports, Factorization, ProcessFactors,
    QuantumErrorEnum, markov_certificate, quantum_markov_check, quantum_markov_check_report,
    quantum_markov_check_report_as,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64) -> C {
    Complex::new(re, 0.0)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0.), c(1.), c(1.), c(0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1.), c(0.), c(0.), c(-1.)], 2)
}

fn diag(a: f64, b: f64) -> CausalTensor<C> {
    mat(vec![c(a), c(0.), c(0.), c(b)], 2)
}

#[test]
fn test_pairwise_disjoint_supports_certify_nothing() {
    // σx and σz would not commute, but on disjoint legs no commutator is formed. The report has
    // to say so rather than reading as a certified commutation.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[7]);

    let report = quantum_markov_check_report(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert_eq!(report.examined(), 0);
    assert_eq!(report.worst_margin(), None);
    assert_eq!(report.verdict(), CheckVerdict::Vacuous);
    assert_eq!(report.factorization(), Factorization::Rederived);
    // A vacuous report grants a certificate that certifies nothing; the count is what tells.
    assert!(markov_certificate(&report).is_ok());
}

#[test]
fn test_a_rejected_candidate_still_reports_its_margins_and_its_count() {
    // σx and σz on one leg. The legacy form drops the report with the error; the report form
    // keeps the rejecting pair's norm, threshold and margin, and the count.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let tol = CommutatorTolerance::default();

    assert!(quantum_markov_check(&pf, &fs, &tol).is_err());

    let report = quantum_markov_check_report(&pf, &fs, &tol).unwrap();
    assert_eq!(report.examined(), 1);
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    let rejecting = report.first_rejection().expect("the pair rejected");
    assert_eq!(rejecting.item, CheckItem::Pair(0, 1));
    assert!(rejecting.measured > 0.0);
    assert!(rejecting.threshold > 0.0);
    assert!(rejecting.margin > 1.0);
    assert_eq!(report.worst_margin(), Some(rejecting.margin));
}

#[test]
fn test_the_failure_path_keeps_every_pair_up_to_the_rejecting_one() {
    // Three factors on one leg: (0, 1) commute, (0, 2) do not. The loop stops there, and the
    // report carries both pairs, the rejecting one last.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    pf.insert(2, sigma_x());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    fs.declare(2, &[0]);

    let report = quantum_markov_check_report(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert_eq!(report.examined(), 2);
    assert_eq!(report.checks()[0].item, CheckItem::Pair(0, 1));
    assert!(report.checks()[0].accepted);
    assert_eq!(report.checks()[1].item, CheckItem::Pair(0, 2));
    assert!(!report.checks()[1].accepted);
}

#[test]
fn test_the_report_form_agrees_with_the_legacy_form_on_a_pass() {
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let tol = CommutatorTolerance::default();

    let legacy = quantum_markov_check(&pf, &fs, &tol).unwrap();
    let report = quantum_markov_check_report(&pf, &fs, &tol).unwrap();
    assert_eq!(report.examined(), legacy.tested_pairs());
    assert_eq!(report.worst_margin(), legacy.worst_margin());
    assert_eq!(report.checks()[0].measured, legacy.checks[0].norm);
    assert_eq!(report.checks()[0].threshold, legacy.checks[0].threshold);
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    assert!(markov_certificate(&report).is_ok());
}

#[test]
fn test_the_failure_variant_follows_the_provenance() {
    // The same non-commuting pair, on the model's own factors and on inherited ones. The first
    // is the model's defect; the second says only that the certificate did not transfer.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let tol = CommutatorTolerance::default();

    let own = quantum_markov_check_report_as(&pf, &fs, &tol, Factorization::Rederived).unwrap();
    match markov_certificate(&own).unwrap_err().0 {
        QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
            assert_eq!((node_j, node_k), (0, 1));
        }
        other => panic!("expected CommutatorNonZero, got {other:?}"),
    }

    let inherited =
        quantum_markov_check_report_as(&pf, &fs, &tol, Factorization::Inherited).unwrap();
    assert_eq!(inherited.factorization(), Factorization::Inherited);
    match markov_certificate(&inherited).unwrap_err().0 {
        QuantumErrorEnum::CertificateNotInherited {
            node_j,
            node_k,
            detail,
        } => {
            assert_eq!((node_j, node_k), (0, 1));
            assert!(
                detail.contains("different factor assignment"),
                "the message must say a Markov factorization may still exist: {detail}"
            );
        }
        other => panic!("expected CertificateNotInherited, got {other:?}"),
    }
}

#[test]
fn test_a_structural_failure_is_still_an_error() {
    // A factor whose dimension disagrees with its support is a shape error, not a rejection, and
    // stays on the `Err` path of both forms.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    assert!(quantum_markov_check_report(&pf, &fs, &CommutatorTolerance::default()).is_err());
}
