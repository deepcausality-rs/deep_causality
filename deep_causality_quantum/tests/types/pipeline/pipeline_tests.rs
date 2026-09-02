/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The pipeline: one origin, three subjects, `build()`'s refusals, the validate → Screened →
//! control hand-off, and the ledger invariants under the control stages.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_haft::Either;
use deep_causality_homology::ChainComplex;
use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CheckVerdict, CommutatorTolerance, Experiment, FactorSupports, Hypothesis,
    MinCostCover, Observable, ProcessFactors, QclBuilder, QuantumErrorEnum, QuantumPlant,
    QubitOperator, ScreenStatus, Spec,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;
type Count = u64;

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

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[c(a), c(b)], &[2])
}

/// The error of a result whose success type carries a graph and so has no `Debug`.
fn err<T, E>(r: Result<T, E>) -> E {
    match r {
        Ok(_) => panic!("expected an error"),
        Err(e) => e,
    }
}

fn graph(
    n: usize,
    edges: &[(usize, usize)],
    freeze: bool,
) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        nodes.push(
            g.add_causaloid(test_utils::get_test_causaloid_deterministic(i as u64))
                .unwrap(),
        );
    }
    for &(a, b) in edges {
        g.add_edge(nodes[a], nodes[b]).unwrap();
    }
    if freeze {
        g.freeze();
    }
    g
}

/// Commuting factors on legs {0} and {0}: one pair tested.
fn commuting() -> (ProcessFactors<f64>, FactorSupports) {
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    (pf, fs)
}

fn plant_ground() -> QuantumPlant<f64> {
    QuantumPlant::from_ket(&ket(1., 0.)).unwrap()
}

fn excited() -> Observable<f64, 2> {
    Observable::from_ket("excited", &ket(0., 1.)).unwrap()
}

fn mechanism(name: &str, u: QubitOperator<f64>) -> Hypothesis<f64> {
    Hypothesis::mechanism(name, Channel::unitary(&u).unwrap())
}

// ---------------------------------------------------------------------------
// The model subject maps onto the shipped freeze.
// ---------------------------------------------------------------------------

#[test]
fn test_the_model_subject_runs_the_two_level_checks_and_screens() {
    let g = graph(2, &[(0, 1)], true);
    let (pf, fs) = commuting();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_decomposable()
        .finalize()
        .unwrap();
    assert_eq!(screened.status(), ScreenStatus::Current);
    let names: Vec<&str> = screened.stages().iter().map(|(n, _)| *n).collect();
    assert_eq!(names, vec!["check_markov", "check_decomposable"]);
    let report = screened.report().unwrap();
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    // One pair tested by Markov; a 1 × 1 relation has no 3 × 3 block, so decomposability is vacuous.
    assert_eq!(screened.stages()[0].1.examined(), 1);
    assert_eq!(screened.stages()[1].1.verdict(), CheckVerdict::Vacuous);
}

#[test]
fn test_an_unfrozen_graph_is_rejected_before_any_check_runs() {
    let g = graph(2, &[(0, 1)], false);
    let (pf, fs) = commuting();
    match err(QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build())
    .0
    {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("frozen"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_cyclic_model_is_rejected_as_scope_not_as_c3() {
    let g = graph(3, &[(0, 1), (1, 2), (2, 0)], true);
    let mut pf = ProcessFactors::new();
    let mut fs = FactorSupports::new();
    for n in 0..3 {
        pf.insert(n, sigma_z());
        fs.declare(n, &[n]);
    }
    match err(QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build())
    .0
    {
        QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
            assert!(msg.contains("scope"), "{msg}");
            assert!(
                !msg.contains("C₃"),
                "names the limit, not an obstruction: {msg}"
            );
        }
        other => panic!("expected CyclicStructureUnsupported, got {other:?}"),
    }
}

#[test]
fn test_a_cyclic_structural_candidate_is_rejected_at_build_by_name() {
    // Supports encode parents: 0 ← 2, 1 ← 0, 2 ← 1 is a cycle, with no graph in sight.
    let mut pf = ProcessFactors::new();
    let mut fs = FactorSupports::new();
    let w = sigma_z().kronecker(&sigma_z()).unwrap();
    for (n, p) in [(0usize, 2usize), (1, 0), (2, 1)] {
        pf.insert(n, w.clone());
        fs.declare(n, &[n, p]);
    }
    let cyclic = Hypothesis::structural("cyclic", pf, fs).unwrap();
    let (pf2, fs2) = commuting();
    let fine = Hypothesis::structural("fine", pf2, fs2).unwrap();
    let err = err(QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .candidates(&[fine, cyclic])
        .build());
    match err.0 {
        QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
            assert!(msg.contains("'cyclic'"), "{msg}")
        }
        other => panic!("expected CyclicStructureUnsupported, got {other:?}"),
    }
}

#[test]
fn test_a_vacuous_or_unreachable_configuration_is_rejected() {
    // Empty candidate set.
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_plant(plant_ground(), &[excited()])
            .mechanisms(&[])
            .build()
            .is_err()
    );
    // An observable the plant does not expose: a 3-dimensional one on a qubit plant.
    let three = CausalTensor::from_slice(&[c(1.), c(0.), c(0.)], &[3]);
    let wide: Observable<f64, 3> = Observable::from_ket("wide", &three).unwrap();
    assert!(matches!(
        err(QclBuilder::config::<f64, Count>()
            .over_plant(plant_ground(), &[wide])
            .mechanisms(&[mechanism("x", QubitOperator::pauli_x())])
            .build())
        .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // A structural candidate offered to mechanisms().
    let (pf, fs) = commuting();
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_plant(plant_ground(), &[excited()])
            .mechanisms(&[Hypothesis::structural("s", pf, fs).unwrap()])
            .build()
            .is_err()
    );
}

#[cfg(feature = "qpu")]
#[test]
fn test_a_zero_shot_budget_is_a_construction_error() {
    use deep_causality_quantum::Evidence;
    let err = err(QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("x", QubitOperator::pauli_x())])
        .evidence(Evidence::shots(0))
        .build());
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}

// ---------------------------------------------------------------------------
// The hand-off.
// ---------------------------------------------------------------------------

#[test]
fn test_structural_candidates_reach_control_only_through_the_screen() {
    // Two structural candidates: one commuting, one not. validate admits one; control takes the
    // screen. (The config itself has no ControlSource impl, which is the compile-time half.)
    let (pf, fs) = commuting();
    let good = Hypothesis::structural("good", pf, fs).unwrap();
    let mut pf2 = ProcessFactors::new();
    pf2.insert(0, sigma_x());
    pf2.insert(1, sigma_z());
    let mut fs2 = FactorSupports::new();
    fs2.declare(0, &[0]);
    fs2.declare(1, &[0]);
    let bad = Hypothesis::structural("bad", pf2, fs2).unwrap();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .candidates(&[good, bad])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    assert_eq!(screened.admitted().len(), 1);
    assert_eq!(screened.admitted()[0].name(), "good");
    let control = QclBuilder::control::<f64, Count, 2, _>(&screened);
    let report = control.finalize().unwrap();
    assert_eq!(report.ledger.experiments(), 0);
}

#[test]
fn test_a_mechanism_config_reaches_control_directly() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("x", QubitOperator::pauli_x())])
        .build()
        .unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .finalize()
        .unwrap();
    assert!(report.worlds.is_empty());
}

#[test]
fn test_a_vacuous_pass_is_visible_in_the_screened_report() {
    let g = graph(2, &[(0, 1)], true);
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[1]);
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    let report = screened.report().unwrap();
    assert_eq!(report.examined(), 0);
    assert_eq!(report.verdict(), CheckVerdict::Vacuous);
}

#[test]
fn test_marginalisation_invalidates_the_screened_report() {
    let g = graph(1, &[], true);
    let w = mat(
        vec![c(1.), Complex::new(0., 2.), Complex::new(0., -2.), c(4.)],
        2,
    );
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_z().kronecker(&w).unwrap());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    assert!(screened.report().is_some());
    let (m, invalidated) = screened.marginalise(1, &sigma_z(), 1e-12).unwrap();
    assert!(m.warrant.holds);
    assert!(
        invalidated.report().is_none(),
        "pre-trace margins are unreadable as current"
    );
    assert!(matches!(
        invalidated.status(),
        ScreenStatus::Invalidated { .. }
    ));
    // The stale margin, degraded by √(d_B), is what remains readable.
    let degraded = invalidated.stale_report_degraded();
    let vacuous = screened.report().unwrap().is_vacuous();
    assert_eq!(degraded.is_some(), !vacuous);
}

// ---------------------------------------------------------------------------
// Transactional failure.
// ---------------------------------------------------------------------------

#[test]
fn test_a_rejected_pair_carries_the_structured_error_and_leaves_the_subject_as_it_was() {
    let g = graph(2, &[(0, 1)], true);
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();
    let e = err(QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_decomposable()
        .finalize());
    assert!(matches!(
        e.0,
        QuantumErrorEnum::CommutatorNonZero {
            node_j: 0,
            node_k: 1,
            ..
        }
    ));
    // Nothing was mutated: the graph is exactly as built.
    assert!(cfg.subject().graph().is_frozen());
}

#[test]
fn test_the_shipped_freeze_path_rolls_a_dynamic_graph_back() {
    let mut g = graph(2, &[(0, 1)], false);
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let err =
        QclBuilder::freeze_model(&mut g, &[], &pf, &fs, &CommutatorTolerance::default(), None)
            .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CommutatorNonZero { .. }));
    assert!(!g.is_frozen(), "rolled back to the dynamic state");
}

// ---------------------------------------------------------------------------
// The code subject.
// ---------------------------------------------------------------------------

#[test]
fn test_the_code_subject_offers_the_validate_stages_only() {
    let torus = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .unwrap()
        .0;
    assert_eq!(torus.num_cells(1), 27);
    let cfg = QclBuilder::config::<f64, Count>()
        .over_code(torus)
        .build()
        .unwrap();
    assert!(cfg.probes().is_empty() && cfg.baseline().is_none());
    let screened = QclBuilder::validate(&cfg)
        .derive_code()
        .check_ldpc_weights(6)
        .check_class_invariance()
        .check_clifford_action()
        .finalize()
        .unwrap();
    let names: Vec<&str> = screened.stages().iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        vec![
            "derive_code",
            "check_ldpc_weights",
            "check_class_invariance",
            "check_clifford_action"
        ]
    );
    assert_eq!(screened.report().unwrap().verdict(), CheckVerdict::Accepted);
    // Two classes × three diagonal gates, and two Hadamards.
    assert_eq!(screened.stages()[2].1.examined(), 6);
    assert_eq!(screened.stages()[3].1.examined(), 2);
    // A bound the X checks exceed fails the screen with the structured record.
    let err = QclBuilder::validate(&cfg).check_ldpc_weights(3);
    assert!(err.ldpc().unwrap().report.first_rejection().is_some());
}

// ---------------------------------------------------------------------------
// The control stages and the ledger invariants.
// ---------------------------------------------------------------------------

#[test]
fn test_only_the_device_boundary_increments_the_device_fields() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
            mechanism("phase", QubitOperator::pauli_z()),
        ])
        .seed(20260821)
        .build()
        .unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 256)
        .fork()
        .predict(0)
        .finalize()
        .unwrap();
    // One hardware experiment before the fork; three model evaluations after it.
    assert_eq!(report.ledger.experiments(), 1);
    assert_eq!(report.ledger.shots(), 256);
    assert_eq!(report.ledger.device_time(), 256.0);
    assert_eq!(report.ledger.predictions(), 0);
    assert_eq!(report.worlds.len(), 3);
    for w in &report.worlds {
        assert_eq!(w.ledger().experiments(), 1, "cloned, not re-observed");
        assert_eq!(w.ledger().shots(), 256);
        assert_eq!(
            w.ledger().predictions(),
            1,
            "predict counted once per world"
        );
    }
    let predictions: Vec<f64> = report
        .worlds
        .iter()
        .map(|w| w.prediction().unwrap())
        .collect();
    assert!((predictions[0] - 1.0).abs() < 1e-12, "X flips |0⟩ to |1⟩");
    assert!(predictions[1].abs() < 1e-12);
    assert!(predictions[2].abs() < 1e-12);
}

#[test]
fn test_fork_observe_gate_adjudicate_names_the_survivor_and_ledgers_are_read_side_by_side() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
            mechanism("phase", QubitOperator::pauli_z()),
        ])
        .probes(&[
            Experiment::new("e1", 1.0, 1024, vec![0.99, 0.01, 0.01]).unwrap(),
            Experiment::new("e2", 1.0, 1024, vec![0.01, 0.99, 0.01]).unwrap(),
        ])
        .seed(7)
        .build()
        .unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .fork()
        .observe(0, 1024)
        .gate(Spec::at_least(0.9))
        .design(MinCostCover::new(5.0))
        .adjudicate(5.0)
        .finalize()
        .unwrap();
    // Each world observed its own counterfactual plant: the ledgers are side by side, unjoined.
    for w in &report.worlds {
        assert_eq!(w.ledger().experiments(), 1);
        assert_eq!(w.ledger().shots(), 1024);
    }
    assert_eq!(report.ledger.experiments(), 0, "the root observed nothing");
    let a = report.adjudication.as_ref().unwrap();
    assert_eq!(a.worlds_folded, 3);
    assert_eq!(
        a.commutation_pairs_tested, 0,
        "read-out verdicts take no commutation test"
    );
    match &a.outcome {
        Either::Left(s) => assert_eq!(s.name, "flip"),
        other => panic!("expected the flipped world to survive, got {other:?}"),
    }
    assert!(report.ledger.bits() > 5.0, "the separation was credited");
    let plan = report.plan.as_ref().unwrap();
    assert_eq!(plan.total_cost(), 2.0);
    assert_eq!(report.ledger.cost(), 2.0);
}

#[test]
fn test_a_failing_stage_is_sticky_and_carries_its_cause() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("x", QubitOperator::pauli_x())])
        .build()
        .unwrap();
    // gate before observe: no read-out.
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .gate(Spec::at_least(0.5))
        .observe(0, 10)
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    // An observable index the plant does not expose.
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(3, 10)
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}
