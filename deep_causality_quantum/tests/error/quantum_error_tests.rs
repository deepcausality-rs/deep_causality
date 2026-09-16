/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_metric::MetricError;
use deep_causality_multivector::HilbertState;
use deep_causality_num_complex::Complex;
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
            QuantumError::CyclicStructureUnsupported("candidate 'cyclic'".into()),
            "Cyclic Structure Unsupported: candidate 'cyclic'",
        ),
        (
            QuantumError::HypothesisCountExceeded(10, 45),
            "Hypothesis Count Exceeded: 10 hypotheses give 45 pairs, above the design cap",
        ),
        (
            QuantumError::BoundaryNotHeld("residual 0.3 > 1e-9".into()),
            "Boundary Not Held: residual 0.3 > 1e-9",
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
            QuantumError::NoCompositionalModel("Marginal".into()),
            "No Compositional Model: a Marginal subject is the marginal of a compositional model and not one itself; only a circuit subject, whose dilation carries the model, can enter an abstraction",
        ),
        (
            QuantumError::NaturalityDimensionExceeded(18, 2, 1 << 40, 1 << 24),
            "Naturality Dimension Exceeded: a channel from 18 to 2 qubits has a composite Choi of 1099511627776 entries, above the cap of 16777216",
        ),
        (
            QuantumError::KrausFamilyExceeded(8192, 4096),
            "Kraus Family Exceeded: 8192 operators, above the cap of 4096",
        ),
        (
            QuantumError::NoPropagationNormalForm(1, 0),
            "No Propagation Normal Form: layer 1 is a non-diagonal Clifford layer following the non-Clifford remainder left by layer 0; the propagated error is neither a Pauli nor diagonal and has no normal form of polynomial size",
        ),
        (
            QuantumError::NotParallelisable(0, 0, 1),
            "Not Parallelisable: interchange set 0 holds nodes 0 and 1, joined by the directed path 0 → 1",
        ),
        (
            QuantumError::SectionNotInverse(2, "residual 0.2 above 0.001".into()),
            "Section Not Inverse: entry 2: residual 0.2 above 0.001",
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

/// `?` joins a multivector construction to a quantum kernel.
///
/// The kernels take `HilbertState` operands, so a caller builds one, calls a kernel, and builds
/// another from the result. Without this conversion the two halves report through different error
/// types and the sequence cannot be written with `?` at all.
#[test]
fn test_from_causal_multivector_error() {
    // `Cl(2)` needs four coefficients, so three is a length mismatch.
    let source = HilbertState::<f64>::new(
        vec![Complex::new(1.0, 0.0); 3],
        deep_causality_multivector::Metric::Euclidean(2),
    )
    .unwrap_err();
    let text = format!("{source}");

    let err: QuantumError = source.into();
    match &err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => assert_eq!(msg, &text),
        other => panic!("expected DimensionMismatch, got {:?}", other),
    }
}

/// The conversion is what makes a build-then-commute sequence expressible with `?`.
#[test]
fn test_question_mark_joins_construction_and_kernel() {
    fn commute_two(len: usize) -> Result<usize, QuantumError> {
        let metric = deep_causality_multivector::Metric::Euclidean(2);
        let a = HilbertState::<f64>::new(vec![Complex::new(1.0, 0.0); len], metric)?;
        let b = HilbertState::<f64>::new(vec![Complex::new(0.0, 1.0); len], metric)?;
        let c = deep_causality_quantum::commutator_kernel(&a, &b)?;
        Ok(c.as_inner().data().len())
    }

    assert_eq!(commute_two(4).unwrap(), 4);
    assert!(matches!(
        commute_two(3).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
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

#[test]
fn test_dimension_variant_carries_its_counts() {
    let err = QuantumError::NaturalityDimensionExceeded(8, 2, 1 << 20, 1 << 24);
    match &err.0 {
        QuantumErrorEnum::NaturalityDimensionExceeded { n, k, entries, cap } => {
            assert_eq!((*n, *k), (8, 2));
            assert_eq!(*entries, 1 << 20);
            assert_eq!(*cap, 1 << 24);
        }
        other => panic!("expected NaturalityDimensionExceeded, got {:?}", other),
    }
    let err = QuantumError::KrausFamilyExceeded(3, 2);
    match &err.0 {
        QuantumErrorEnum::KrausFamilyExceeded { operators, cap } => {
            assert_eq!((*operators, *cap), (3, 2));
        }
        other => panic!("expected KrausFamilyExceeded, got {:?}", other),
    }
}
