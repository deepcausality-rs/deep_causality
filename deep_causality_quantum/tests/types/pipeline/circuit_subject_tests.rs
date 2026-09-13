/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fourth subject: a circuit on the builder, screened through its dilation.

use deep_causality_quantum::{
    Abstraction, Axis, Channel, CircuitBox, CircuitModel, CommutatorTolerance, Factorization,
    NumericCaps, QcMorphism, QclBuilder, QuantumErrorEnum, QubitOperator, Query, ScreenOrigin,
    TypeAlignment, WireType,
};

type FloatType = f64;
type NumberType = u64;

fn ry(theta: f64) -> CircuitBox<FloatType> {
    CircuitBox::Channel {
        wires: vec![0],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

fn chain() -> CircuitModel<FloatType> {
    CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0.7), ry(0.9)],
        vec![],
        vec![0],
    )
    .unwrap()
}

#[test]
fn test_a_circuit_builds_and_its_dilation_is_screened() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(chain())
        .build()
        .expect("an acyclic quantum circuit builds");
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::new())
        .check_decomposable(&[0], &[1])
        .finalize()
        .expect("the dilation is Markov and C₃-free");
    assert_eq!(screened.origin(), ScreenOrigin::Circuit);
    assert!(screened.require_compositional().is_ok());
    let stages = screened.stages();
    assert_eq!(stages[0].0, "check_markov");
    assert_eq!(stages[0].1.examined(), 1);
    assert_eq!(stages[0].1.factorization(), Factorization::Rederived);
    assert_eq!(stages[1].0, "check_decomposable");
    assert!(screened.report().unwrap().accepted());
    assert_eq!(cfg.subject().model().boxes().len(), 2);
}

#[test]
fn test_a_cyclic_grouping_is_refused_at_build() {
    let cyclic = CircuitModel::new(
        vec![WireType::qubit()],
        vec![ry(0.1), ry(0.2), ry(0.3)],
        vec![vec![0, 2], vec![1]],
        vec![],
        vec![0],
    )
    .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(cyclic)
        .build()
        .err()
        .expect("a cyclic grouping is refused");
    assert!(matches!(
        err.0,
        QuantumErrorEnum::CyclicStructureUnsupported(_)
    ));
    let empty =
        CircuitModel::<FloatType>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0])
            .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(empty)
        .build()
        .err()
        .expect("refused");
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    let classical = CircuitModel::<FloatType>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let err = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(classical)
        .build()
        .err()
        .expect("refused");
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}

#[test]
fn test_a_model_subject_cannot_enter_an_abstraction() {
    use deep_causality::utils_test::test_utils;
    use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
    use deep_causality_num_complex::Complex;
    use deep_causality_quantum::{FactorSupports, ProcessFactors};
    use deep_causality_tensor::CausalTensor;
    let mut g: CausaloidGraph<BaseCausaloid<f64, bool>> = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    g.add_edge(n0, n1).unwrap();
    g.freeze();
    let diag = |a: f64, b: f64| {
        CausalTensor::from_slice(
            &[
                Complex::new(a, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(0.0, 0.0),
                Complex::new(b, 0.0),
            ],
            &[2, 2],
        )
    };
    let mut factors = ProcessFactors::new();
    factors.insert(0, diag(0.9, 0.1));
    let mut supports = FactorSupports::new();
    supports.declare(0, &[0]);
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_model(g, factors, supports)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::new())
        .finalize()
        .unwrap();
    assert_eq!(screened.origin(), ScreenOrigin::Marginal);
    let err = screened.require_compositional().unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::NoCompositionalModel(ref m) if m.contains("Marginal"))
    );
}

/// A two-node chain with its wire as input and output, and the identity abstraction onto a copy
/// of itself: the partition `[[0], [1]]` is simple, `[[1], [0]]` puts the second gate's block
/// upstream of the first's and is not.
fn io_chain() -> CircuitModel<FloatType> {
    CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0.7), ry(0.9)],
        vec![0],
        vec![0],
    )
    .unwrap()
}

fn identity_abstraction(
    low: CircuitModel<FloatType>,
) -> Abstraction<FloatType, CircuitModel<FloatType>, CircuitModel<FloatType>> {
    let identity = QcMorphism::<FloatType>::identity(2).unwrap();
    Abstraction::new(
        low,
        io_chain(),
        TypeAlignment::new(vec![(vec![0], vec![0], identity.clone(), identity)]).unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap()
}

#[test]
fn test_both_abstraction_stages_record_in_order() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(io_chain())
        .build()
        .unwrap();
    let abstraction = identity_abstraction(io_chain());
    let screened = QclBuilder::validate(&cfg)
        .check_alignment_structure(&abstraction, &[vec![0], vec![1]])
        .check_naturality(&abstraction, &NumericCaps::default())
        .finalize()
        .expect("a simple partition and a commuting square");
    let stages = screened.stages();
    assert_eq!(stages.len(), 2);
    assert_eq!(stages[0].0, "check_alignment_structure");
    assert_eq!(
        stages[0].1.examined(),
        2,
        "two ordered pairs of high-level vertices"
    );
    assert_eq!(stages[1].0, "check_naturality");
    assert_eq!(stages[1].1.examined(), 1, "one query");
    let report = screened.report().unwrap();
    assert!(report.accepted());
    assert_eq!(report.examined(), 3);
}

#[test]
fn test_a_rejecting_precheck_stops_the_naturality_check() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(io_chain())
        .build()
        .unwrap();
    let abstraction = identity_abstraction(io_chain());
    // A zero cap: had the naturality stage run, it would have failed with the cap, not with the
    // partition.
    let no_matrix = NumericCaps {
        max_entries: 0,
        max_operators: 0,
    };
    let err = QclBuilder::validate(&cfg)
        .check_alignment_structure(&abstraction, &[vec![1], vec![0]])
        .check_naturality(&abstraction, &no_matrix)
        .finalize()
        .err()
        .expect("the partition is not simple");
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("not simple"), "{msg}");
            assert!(msg.contains("α(0) meets π(1)"), "{msg}");
            assert!(msg.contains("Necessary"), "{msg}");
        }
        other => panic!("{other:?}"),
    }
    // The same zero cap with a simple partition reaches the naturality stage and fails there.
    let err = QclBuilder::validate(&cfg)
        .check_alignment_structure(&abstraction, &[vec![0], vec![1]])
        .check_naturality(&abstraction, &no_matrix)
        .finalize()
        .err()
        .expect("the zero cap refuses the square");
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded { .. }
            | QuantumErrorEnum::KrausFamilyExceeded { .. }
    ));
}

#[test]
fn test_an_abstraction_over_another_circuit_is_refused_by_both_stages() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(io_chain())
        .build()
        .unwrap();
    let other = CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0.1), ry(0.9)],
        vec![0],
        vec![0],
    )
    .unwrap();
    let abstraction = identity_abstraction(other);
    for stage in 0..2 {
        let v = QclBuilder::validate(&cfg);
        let v = if stage == 0 {
            v.check_alignment_structure(&abstraction, &[vec![0], vec![1]])
        } else {
            v.check_naturality(&abstraction, &NumericCaps::default())
        };
        let err = v.finalize().err().expect("refused");
        assert!(
            matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not the screened circuit")),
            "stage {stage}: {err:?}"
        );
    }
}

/// The decomposability stage refuses a boundary that names no system or a node the dilation does
/// not have, rather than deriving an empty structure and accepting it.
#[test]
fn test_an_empty_or_out_of_range_boundary_is_a_stage_failure() {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(chain())
        .build()
        .unwrap();
    for (inputs, outputs, needle) in [
        (vec![], vec![1], "input"),
        (vec![0], vec![], "output"),
        (vec![0], vec![7], "node 7"),
        (vec![9], vec![1], "node 9"),
    ] {
        let err = QclBuilder::validate(&cfg)
            .check_decomposable(&inputs, &outputs)
            .finalize()
            .err()
            .unwrap_or_else(|| panic!("{inputs:?} → {outputs:?} was accepted"));
        assert!(
            matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains(needle)),
            "{inputs:?} → {outputs:?}: {err:?}"
        );
    }
    assert!(
        QclBuilder::validate(&cfg)
            .check_decomposable(&[0], &[1])
            .finalize()
            .is_ok()
    );
}
