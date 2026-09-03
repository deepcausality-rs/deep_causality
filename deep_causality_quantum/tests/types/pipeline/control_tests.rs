/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The control stage: what `fork` hands a world, the refusal of an empty fork, `compare` on the
//! structural path, and the mechanism path on each world's own plant.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CommutatorTolerance, FactorSupports, Hypothesis, Observable, ProcessFactors,
    QclBuilder, QuantumErrorEnum, QuantumPlant, QubitOperator, Spec,
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

/// A qubit whose excited population is `p`.
fn plant_with_population(p: f64) -> QuantumPlant<f64> {
    QuantumPlant::from_ket(&ket((1.0 - p).sqrt(), p.sqrt())).unwrap()
}

fn plant_ground() -> QuantumPlant<f64> {
    plant_with_population(0.0)
}

fn excited() -> Observable<f64, 2> {
    Observable::from_ket("excited", &ket(0., 1.)).unwrap()
}

fn mechanism(name: &str, u: QubitOperator<f64>) -> Hypothesis<f64> {
    Hypothesis::mechanism(name, Channel::unitary(&u).unwrap())
}

/// A structural candidate of one factor on one leg whose excited population is `p`: its joint
/// operator is the factor, so its prediction for `excited` is `p`.
fn population(name: &str, p: f64) -> Hypothesis<f64> {
    let mut pf = ProcessFactors::new();
    pf.insert(0, diag(1.0 - p, p));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    Hypothesis::structural(name, pf, fs).unwrap()
}

/// A structural candidate the Markov check rejects: `σ_x` and `σ_z` on the same leg.
fn non_commuting() -> Hypothesis<f64> {
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    Hypothesis::structural("non_commuting", pf, fs).unwrap()
}

fn frozen_graph(n: usize) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    for i in 0..n {
        g.add_causaloid(test_utils::get_test_causaloid_deterministic(i as u64))
            .unwrap();
    }
    g.freeze();
    g
}

fn calculation_message(e: deep_causality_quantum::QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// fork
// ---------------------------------------------------------------------------

#[test]
fn test_fork_on_a_screen_that_admitted_nothing_fails_at_finalize() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .candidates(&[non_commuting()])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    assert!(screened.admitted().is_empty());
    let err = QclBuilder::control::<f64, Count, 2, _>(&screened)
        .fork()
        .finalize()
        .unwrap_err();
    let msg = calculation_message(err);
    assert!(msg.contains("admitted none"), "{msg}");
    assert!(msg.contains("declared none"), "{msg}");
}

#[test]
fn test_worlds_after_fork_carry_the_ledger_and_no_evidence() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .build()
        .unwrap();
    // The root was observed and gated before the fork; the worlds inherit neither.
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 64)
        .gate(Spec::at_most(0.5))
        .fork()
        .finalize()
        .unwrap();
    assert_eq!(report.worlds.len(), 2);
    for w in &report.worlds {
        assert_eq!(w.ledger(), &report.ledger, "the ledger is inherited");
        assert_eq!(w.ledger().experiments(), 1);
        assert!(w.read_out().is_none(), "no read-out is inherited");
        assert!(w.verdict().is_none(), "no verdict is inherited");
        assert!(w.prediction().is_none());
    }
    // gate after the fork, with no observe on the worlds, has nothing to judge.
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 64)
        .fork()
        .gate(Spec::at_most(0.5))
        .finalize()
        .unwrap_err();
    let msg = calculation_message(err);
    assert!(msg.contains("observe"), "{msg}");
}

// ---------------------------------------------------------------------------
// The structural path: observe → fork → predict → compare → adjudicate
// ---------------------------------------------------------------------------

#[test]
fn test_the_structural_path_selects_the_candidate_whose_prediction_matches_the_plant() {
    // The plant's excited population is 0.2, so the Born read-out of `excited` is 0.2: the
    // first candidate's prediction. The second predicts 0.8.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .candidates(&[
            population("two_tenths", 0.2),
            population("eight_tenths", 0.8),
        ])
        .seed(11)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    assert_eq!(screened.admitted().len(), 2);
    let shots = 4096u64;
    let report = QclBuilder::control::<f64, Count, 2, _>(&screened)
        .observe(0, shots)
        .fork()
        .predict(0)
        .compare(3.0)
        .adjudicate(5.0)
        .finalize()
        .unwrap();
    // One hardware experiment on the root, inherited; one model evaluation per world.
    assert_eq!(report.ledger.experiments(), 1);
    assert_eq!(report.ledger.shots(), shots);
    for w in &report.worlds {
        assert_eq!(w.ledger().experiments(), 1);
        assert_eq!(w.ledger().predictions(), 1);
    }
    // Each world's read-out is its prediction at the baseline's shots, and the predictions
    // differ, which is what a post-fork measurement of the unchanged plant could not give.
    let predictions: Vec<f64> = report
        .worlds
        .iter()
        .map(|w| w.prediction().unwrap())
        .collect();
    assert!((predictions[0] - 0.2).abs() < 1e-12);
    assert!((predictions[1] - 0.8).abs() < 1e-12);
    for (w, p) in report.worlds.iter().zip(&predictions) {
        let e = w.read_out().expect("compare set the read-out");
        assert_eq!(e.estimate(), *p);
        assert_eq!(e.shots(), shots);
        let expected_se = (p * (1.0 - p) / shots as f64).sqrt();
        assert!((e.standard_error() - expected_se).abs() < 1e-12);
        let v = w.verdict().expect("compare set the verdict");
        assert_eq!(v.examined(), shots as usize);
        assert_eq!(v.checks().len(), 1);
    }
    assert!(report.worlds[0].verdict().unwrap().accepted());
    assert!(!report.worlds[1].verdict().unwrap().accepted());
    // The survivor is the candidate whose prediction is the plant's read-out, separated from its
    // rival by the distance between the two predictions at the observed shots.
    let a = report.adjudication.as_ref().unwrap();
    assert_eq!(a.worlds_folded, 2);
    assert_eq!(a.commutation_pairs_tested, 0);
    match &a.outcome {
        Either::Left(s) => {
            assert_eq!(s.name, "two_tenths");
            assert!(s.separation_bits > 5.0);
        }
        other => panic!("expected two_tenths to survive, got {other:?}"),
    }
    assert!(report.ledger.bits() > 5.0);
}

#[test]
fn test_compare_names_the_missing_step() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .candidates(&[
            population("two_tenths", 0.2),
            population("eight_tenths", 0.8),
        ])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    let control = || QclBuilder::control::<f64, Count, 2, _>(&screened);

    // Before predict.
    let msg = calculation_message(
        control()
            .observe(0, 256)
            .fork()
            .compare(3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("predict"), "{msg}");

    // Before fork.
    let msg = calculation_message(
        control()
            .observe(0, 256)
            .compare(3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("fork"), "{msg}");

    // Without a root read-out.
    let msg = calculation_message(
        control()
            .fork()
            .predict(0)
            .compare(3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("observe"), "{msg}");

    // A NaN or negative sigmas.
    for sigmas in [f64::NAN, -1.0] {
        let msg = calculation_message(
            control()
                .observe(0, 256)
                .fork()
                .predict(0)
                .compare(sigmas)
                .finalize()
                .unwrap_err(),
        );
        assert!(msg.contains("sigmas"), "{msg}");
    }
}

#[test]
fn test_a_mechanism_world_may_be_compared() {
    // The plant sits at 0.3; the identity keeps it there and the flip takes it to 0.7. Against
    // the root's read-out the identity world's prediction agrees and the flip's does not.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.3), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .seed(5)
        .build()
        .unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 2048)
        .fork()
        .predict(0)
        .compare(3.0)
        .adjudicate(5.0)
        .finalize()
        .unwrap();
    assert!(!report.worlds[0].verdict().unwrap().accepted());
    assert!(report.worlds[1].verdict().unwrap().accepted());
    match &report.adjudication.as_ref().unwrap().outcome {
        Either::Left(s) => assert_eq!(s.name, "keep"),
        other => panic!("expected keep to survive, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The mechanism path: fork → observe → gate → adjudicate
// ---------------------------------------------------------------------------

#[test]
fn test_the_mechanism_path_adjudicates_each_world_on_its_own_plant() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .seed(3)
        .build()
        .unwrap();
    // The root is observed in the ground state before the fork; each world is then observed on
    // its own evolved plant, so the flipped world reads one where the root read zero.
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 512)
        .fork()
        .observe(0, 512)
        .gate(Spec::at_least(0.9))
        .adjudicate(5.0)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.experiments(), 1);
    assert_eq!(report.ledger.shots(), 512);
    let flip = &report.worlds[0];
    let keep = &report.worlds[1];
    assert_eq!(flip.ledger().experiments(), 2, "one inherited, one its own");
    assert_eq!(flip.ledger().shots(), 1024);
    assert_eq!(flip.read_out().unwrap().estimate(), 1.0);
    assert_eq!(keep.read_out().unwrap().estimate(), 0.0);
    assert!(flip.verdict().unwrap().accepted());
    assert!(!keep.verdict().unwrap().accepted());
    match &report.adjudication.as_ref().unwrap().outcome {
        Either::Left(s) => assert_eq!(s.name, "flip"),
        other => panic!("expected flip to survive, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// build(): the model subject
// ---------------------------------------------------------------------------

#[test]
fn test_a_model_subject_with_no_factors_fails_at_build() {
    let r = QclBuilder::config::<f64, Count>()
        .over_model(
            frozen_graph(1),
            ProcessFactors::new(),
            FactorSupports::new(),
        )
        .build();
    let err = match r {
        Ok(_) => panic!("expected an error"),
        Err(e) => e,
    };
    let msg = calculation_message(err);
    assert!(msg.contains("no factors"), "{msg}");
}
