/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_metric::MetricError;
use deep_causality_quantum::{QuantumError, QuantumErrorEnum};

#[test]
fn test_display_covers_every_variant() {
    let cases: Vec<(QuantumError, &str)> = vec![
        (
            QuantumError::DimensionMismatch("d".into()),
            "Dimension Mismatch: d",
        ),
        (
            QuantumError::MetricMismatch("m".into()),
            "Metric Mismatch: m",
        ),
        (
            QuantumError::UnsupportedMetric("u".into()),
            "Unsupported Metric: u",
        ),
        (
            QuantumError::NonFiniteValue("n".into()),
            "Non-Finite Value: n",
        ),
        (
            QuantumError::NormalizationError("p".into()),
            "Normalization Error: p",
        ),
        (
            QuantumError::NonPositiveOperator("o".into()),
            "Non-Positive Operator: o",
        ),
        (QuantumError::NonUnitTrace("t".into()), "Non-Unit Trace: t"),
        (
            QuantumError::NonCptpChannel("c".into()),
            "Non-CPTP Channel: c",
        ),
        (
            QuantumError::PartialTraceShape("s".into()),
            "Partial Trace Shape Error: s",
        ),
        (
            QuantumError::CommutatorNonZero(3, 7, "norm 0.5".into()),
            "Non-Zero Commutator: factors at nodes 3 and 7 do not commute: norm 0.5",
        ),
        (
            QuantumError::NotFaithfullyRepresentable("g".into()),
            "Not Faithfully Representable (C3 obstruction): g",
        ),
        (
            QuantumError::CertificateNotInherited(2, 5, "see D9".into()),
            "Certificate Not Inherited: the parts' factors at nodes 2 and 5 do not certify the composite: see D9",
        ),
        (
            QuantumError::NotInNormalizer(4, "Z-stabilizer".into()),
            "Not In Normalizer: anticommutes with stabilizer generator 4 (Z-stabilizer)",
        ),
        (
            QuantumError::NonCliffordGate("T(2) at position 5".into()),
            "Non-Clifford Gate: T(2) at position 5",
        ),
        (
            QuantumError::CalculationError("x".into()),
            "Calculation Error: x",
        ),
    ];

    for (err, expected) in cases {
        assert_eq!(format!("{}", err), expected);
    }
}

#[test]
fn test_commutator_variant_names_the_offending_pair() {
    let err = QuantumError::CommutatorNonZero(1, 2, "detail".into());
    match &err.0 {
        QuantumErrorEnum::CommutatorNonZero {
            node_j,
            node_k,
            detail,
        } => {
            assert_eq!(*node_j, 1);
            assert_eq!(*node_k, 2);
            assert_eq!(detail, "detail");
        }
        other => panic!("expected CommutatorNonZero, got {:?}", other),
    }
}

#[test]
fn test_from_quantum_error_for_causality_error() {
    let err = QuantumError::MetricMismatch("Euclidean(3) vs Euclidean(1)".into());
    let cause: CausalityError = err.into();
    let msg = format!("{}", cause);
    assert!(
        msg.contains("Metric Mismatch: Euclidean(3) vs Euclidean(1)"),
        "unexpected message: {}",
        msg
    );
}

#[test]
fn test_from_metric_error() {
    let err: QuantumError = MetricError::InvalidDimension("dim 0".into()).into();
    match &err.0 {
        QuantumErrorEnum::UnsupportedMetric(msg) => assert!(msg.contains("dim 0")),
        other => panic!("expected UnsupportedMetric, got {:?}", other),
    }
}

#[test]
fn test_error_trait_object() {
    let err = QuantumError::NonUnitTrace("trace = 0.9".into());
    let dyn_err: &dyn core::error::Error = &err;
    assert!(format!("{}", dyn_err).contains("Non-Unit Trace"));
}

#[test]
fn test_eq_and_clone() {
    let a = QuantumError::NonFiniteValue("nan".into());
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, QuantumError::NonFiniteValue("inf".into()));
}
