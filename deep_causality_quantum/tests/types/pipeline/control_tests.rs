/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The control stage: what `fork` hands a world, the refusal of an empty fork, `compare` on the
//! structural path, the mechanism path on each world's own plant, the baseline that refuses
//! candidates before the plan, the shot budget every observation draws from, and configured
//! experiments planned, predicted and observed against their evidence.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_context::TimeScale;
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::{
    CROSSTALK_Q1, CROSSTALK_Q2, CrosstalkModel, CrosstalkSetting, crosstalk_candidates,
};
use deep_causality_quantum::{
    Channel, CheckItem, CommutatorTolerance, ConfiguredExperiment, CountHistogram,
    EnvironmentReading, EvidenceSource, Experiment, FactorSupports, Fringe, Hypothesis,
    InterferometerConfiguration, InterferometerContext, InterferometerModel, Mechanisms,
    MinCostCover, Observable, PlantSubject, ProcessFactors, Projection, QclBuilder, QuantumError,
    QuantumErrorEnum, QuantumPlant, QubitOperator, Response, ResponseModel, ShotHistogram, Spec,
    WaveVector, instrument_configuration, interferometer_context, record_environment,
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

// ---------------------------------------------------------------------------
// The observable index, and the accessors a world exposes
// ---------------------------------------------------------------------------

fn dimension_message(e: deep_causality_quantum::QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => msg,
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
}

fn two_mechanism_config()
-> deep_causality_quantum::Config<f64, Count, PlantSubject<f64, 2, Mechanisms>> {
    QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .build()
        .unwrap()
}

#[test]
fn test_observe_refuses_an_observable_the_plant_does_not_expose() {
    let cfg = two_mechanism_config();
    // Exactly one observable is declared, so index 0 is the only valid one; 1 is the
    // first past the end and a large index is well past it.
    for bad in [1usize, 7, usize::MAX] {
        let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .observe(bad, 16)
            .finalize()
            .unwrap_err();
        let msg = dimension_message(err);
        assert!(msg.contains(&bad.to_string()), "{msg}");
        assert!(
            msg.contains('1'),
            "the count of declared observables: {msg}"
        );
    }
}

#[test]
fn test_observe_accepts_the_last_valid_observable_index() {
    // The boundary the refusal above sits next to: index 0 of one observable works.
    let cfg = two_mechanism_config();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 16)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.shots(), 16);
}

#[test]
fn test_predict_before_fork_is_refused() {
    let cfg = two_mechanism_config();
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 16)
        .predict(0)
        .finalize()
        .unwrap_err();
    let msg = calculation_message(err);
    assert!(msg.contains("fork"), "{msg}");
}

#[test]
fn test_predict_refuses_an_observable_the_plant_does_not_expose() {
    let cfg = two_mechanism_config();
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 16)
        .fork()
        .predict(4)
        .finalize()
        .unwrap_err();
    let msg = dimension_message(err);
    assert!(msg.contains('4'), "{msg}");
}

#[test]
fn test_a_world_names_its_candidate_and_carries_its_own_plant() {
    let cfg = two_mechanism_config();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .fork()
        .finalize()
        .unwrap();
    let names: Vec<&str> = report.worlds.iter().map(|w| w.name()).collect();
    assert_eq!(names, vec!["flip", "keep"]);

    // `flip` evolves the ground plant by σ_x and `keep` by the identity, so the two
    // worlds hold different plants and only `keep` still holds the root's.
    let flip = &report.worlds[0];
    let keep = &report.worlds[1];
    assert_ne!(flip.plant(), keep.plant());
    assert_eq!(keep.plant(), &plant_ground());
    assert_eq!(flip.plant(), &plant_with_population(1.0));
}

#[test]
fn test_zero_shots_is_refused_because_an_empty_histogram_carries_no_estimate() {
    // The zero end of the shot range. An observation of nothing has no frequency to
    // bridge to a probability, so the stage refuses rather than recording a vacuous
    // read-out.
    let cfg = two_mechanism_config();
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 0)
        .finalize()
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert!(msg.contains("empty"), "{msg}")
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
}

#[test]
fn test_one_shot_is_the_smallest_observation_that_is_accepted() {
    // The first accepted value above the refused zero.
    let cfg = two_mechanism_config();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 1)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.shots(), 1);
    assert_eq!(report.ledger.experiments(), 1);
}

#[test]
fn test_a_shot_count_the_width_cannot_hold_is_refused_by_the_ledger() {
    // The ledger counts shots on `N`. At `u8` the counter saturates at 255, so two
    // observations of 200 shots overflow it, and the guard on `observed` reports the
    // width rather than wrapping the total to 145. The same program at `u64` runs.
    let cfg = QclBuilder::config::<f64, u8>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .build()
        .unwrap();
    let err = QclBuilder::control::<f64, u8, 2, _>(&cfg)
        .observe(0, 200)
        .observe(0, 200)
        .finalize()
        .unwrap_err();
    let msg = calculation_message(err);
    assert!(msg.contains("overflow"), "{msg}");

    // One observation of 200 fits, so the refusal above is the sum and not the value.
    let ok = QclBuilder::control::<f64, u8, 2, _>(&cfg)
        .observe(0, 200)
        .finalize()
        .unwrap();
    assert_eq!(ok.ledger.shots(), 200u8);
}

// ---------------------------------------------------------------------------
// Prediction slots: probes are positional over the config's candidates
// ---------------------------------------------------------------------------

/// Two probes over three candidates `a, b, c`: `ac` separates `a` from `c` and leaves `b` with
/// `a`; `ab` separates `a` from `b` and leaves `c` with `a`. With `b` gone only `ac` covers the
/// remaining pair, and a plan reading the predictions of `a` and `b` would pick `ab`.
fn probes_over_a_b_c() -> Vec<Experiment<f64>> {
    vec![
        Experiment::new("ac", 1.0, 256, vec![0.1, 0.1, 0.9]).unwrap(),
        Experiment::new("ab", 0.5, 256, vec![0.1, 0.9, 0.1]).unwrap(),
    ]
}

fn plan_names(report: &deep_causality_quantum::ControlReport<f64, Count, 2>) -> Vec<String> {
    let plan = report.plan.as_ref().expect("design ran");
    assert!(plan.is_complete(), "every live pair is covered");
    plan.entries().iter().map(|e| e.name.clone()).collect()
}

#[test]
fn test_a_candidate_the_screen_drops_takes_its_prediction_with_it() {
    // The middle candidate fails the Markov check. The probes still predict for all three, and
    // the plan reads them at the admitted candidates' own positions, 0 and 2.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .candidates(&[population("a", 0.2), non_commuting(), population("c", 0.8)])
        .probes(&probes_over_a_b_c())
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    assert_eq!(screened.admitted_slots(), &[0, 2]);
    let report = QclBuilder::control::<f64, Count, 2, _>(&screened)
        .fork()
        .design(MinCostCover::new(5.0))
        .finalize()
        .unwrap();
    assert_eq!(plan_names(&report), vec!["ac"]);
}

#[test]
fn test_a_probe_predicting_for_another_candidate_count_is_refused_by_design() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .probes(&probes_over_a_b_c())
        .build()
        .unwrap();
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .design(MinCostCover::new(5.0))
        .finalize()
        .unwrap_err();
    let msg = dimension_message(err);
    assert!(msg.contains("'ac' predicts for 3"), "{msg}");
    assert!(msg.contains("declares 2"), "{msg}");
}

// ---------------------------------------------------------------------------
// The baseline: observed first, refusing candidates before the plan
// ---------------------------------------------------------------------------

/// The plant reads 0.2; `a` and `c` predict that for the baseline and `b` predicts 0.8.
fn baseline_config(
    baseline: Experiment<f64>,
) -> deep_causality_quantum::Config<f64, Count, PlantSubject<f64, 2, Mechanisms>> {
    QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .mechanisms(&[
            mechanism("a", QubitOperator::identity()),
            mechanism("b", QubitOperator::pauli_x()),
            mechanism("c", QubitOperator::identity()),
        ])
        .probes(&probes_over_a_b_c())
        .baseline(baseline)
        .seed(17)
        .build()
        .unwrap()
}

fn e0() -> Experiment<f64> {
    Experiment::new("E0", 2.0, 4096, vec![0.2, 0.8, 0.2]).unwrap()
}

#[test]
fn test_the_baseline_refuses_a_candidate_before_design() {
    let cfg = baseline_config(e0());
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .baseline(0, 3.0)
        .fork()
        .design(MinCostCover::new(5.0))
        .finalize()
        .unwrap();

    // The baseline is one hardware observation at its own shots.
    assert_eq!(report.ledger.experiments(), 1);
    assert_eq!(report.ledger.shots(), 4096);

    // `b` is refused with what decided it; `a` and `c` go on.
    assert_eq!(report.refused.len(), 1);
    let r = &report.refused[0];
    assert_eq!(r.name(), "b");
    assert_eq!(r.prediction(), 0.8);
    assert_eq!(r.observed().shots(), 4096);
    let check = r.check();
    assert_eq!(check.item, CheckItem::Index(1));
    assert!(!check.accepted);
    assert_eq!(check.measured, (0.8 - r.observed().estimate()).abs());
    assert_eq!(check.threshold, 3.0 * r.observed().standard_error());
    let names: Vec<&str> = report.worlds.iter().map(|w| w.name()).collect();
    assert_eq!(names, vec!["a", "c"]);

    // The plan covers the survivors' pair, read at their own positions.
    assert_eq!(plan_names(&report), vec!["ac"]);
}

#[test]
fn test_the_baseline_is_the_read_out_compare_judges_against() {
    // `a` keeps the plant at 0.2 and `b` flips it to 0.8. The baseline already refused `b`, and
    // `compare` judges the survivors' predictions against the baseline's read-out.
    let cfg = baseline_config(e0());
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .baseline(0, 3.0)
        .fork()
        .predict(0)
        .compare(3.0)
        .finalize()
        .unwrap();
    for w in &report.worlds {
        assert!(w.verdict().unwrap().accepted(), "{}", w.name());
        assert_eq!(w.read_out().unwrap().shots(), 4096);
    }
}

#[test]
fn test_a_named_baseline_is_observed_before_anything_else() {
    let cfg = baseline_config(e0());
    let control = || QclBuilder::control::<f64, Count, 2, _>(&cfg);
    for (stage, run) in [
        ("observe", control().observe(0, 16)),
        ("fork", control().fork()),
        ("design", control().design(MinCostCover::new(5.0))),
    ] {
        let msg = calculation_message(run.finalize().unwrap_err());
        assert!(msg.contains("baseline 'E0'"), "{msg}");
        assert!(msg.contains(&format!("before {stage}")), "{msg}");
    }

    // After the baseline the root may be observed again.
    let report = control()
        .baseline(0, 3.0)
        .observe(0, 16)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.experiments(), 2);
    assert_eq!(report.ledger.shots(), 4096 + 16);
}

#[test]
fn test_the_baseline_stage_needs_a_baseline_that_has_not_run() {
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&two_mechanism_config())
            .baseline(0, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("names none"), "{msg}");

    let cfg = baseline_config(e0());
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .baseline(0, 3.0)
            .baseline(0, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("already ran"), "{msg}");
}

#[test]
fn test_the_baseline_refuses_bad_arguments() {
    let cfg = baseline_config(e0());
    let control = || QclBuilder::control::<f64, Count, 2, _>(&cfg);
    for sigmas in [f64::NAN, f64::INFINITY, -1.0] {
        let msg = calculation_message(control().baseline(0, sigmas).finalize().unwrap_err());
        assert!(msg.contains("sigmas"), "{msg}");
    }
    // Zero sigmas is the boundary and is accepted.
    assert!(control().baseline(0, 0.0).finalize().is_ok());

    let msg = dimension_message(control().baseline(3, 3.0).finalize().unwrap_err());
    assert!(msg.contains("observable 3"), "{msg}");

    let short = Experiment::new("short", 1.0, 64, vec![0.2, 0.8]).unwrap();
    let msg = dimension_message(
        QclBuilder::control::<f64, Count, 2, _>(&baseline_config(short))
            .baseline(0, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("'short' predicts for 2"), "{msg}");
    assert!(msg.contains("declares 3"), "{msg}");
}

#[test]
fn test_baseline_shots_the_count_width_cannot_hold_are_refused() {
    let cfg = QclBuilder::config::<f64, u8>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .baseline(Experiment::new("wide", 1.0, 256, vec![0.0]).unwrap())
        .build()
        .unwrap();
    let msg = calculation_message(
        QclBuilder::control::<f64, u8, 2, _>(&cfg)
            .baseline(0, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("256 shots"), "{msg}");
}

#[test]
fn test_a_baseline_that_refuses_every_candidate_leaves_nothing_to_fork() {
    // The ground plant reads 0 exactly, so the allowance is zero and both predictions of 0.9 are
    // refused; a prediction of exactly 0 would be kept.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .baseline(Experiment::new("dark", 1.0, 64, vec![0.9, 0.9]).unwrap())
        .build()
        .unwrap();
    let control = QclBuilder::control::<f64, Count, 2, _>(&cfg).baseline(0, 3.0);
    let msg = calculation_message(control.fork().finalize().unwrap_err());
    assert!(msg.contains("refused every one"), "{msg}");

    let kept = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .baseline(Experiment::new("dark", 1.0, 64, vec![0.0]).unwrap())
        .build()
        .unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&kept)
        .baseline(0, 3.0)
        .fork()
        .finalize()
        .unwrap();
    assert!(report.refused.is_empty());
    assert_eq!(report.worlds.len(), 1);
}

// ---------------------------------------------------------------------------
// The evidence budget (qpu): every observation draws from it
// ---------------------------------------------------------------------------

#[cfg(feature = "qpu")]
fn budgeted_config(
    shots: Count,
    seed: u64,
    evidence_seed: u64,
) -> deep_causality_quantum::Config<f64, Count, PlantSubject<f64, 2, Mechanisms>> {
    QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.5), &[excited()])
        .mechanisms(&[
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("keep", QubitOperator::identity()),
        ])
        .seed(seed)
        .evidence(deep_causality_quantum::Evidence::shots(shots).seed(evidence_seed))
        .build()
        .unwrap()
}

#[cfg(feature = "qpu")]
#[test]
fn test_an_observation_beyond_the_budget_fails_in_observe() {
    let cfg = budgeted_config(100, 0, 0);
    let control = || QclBuilder::control::<f64, Count, 2, _>(&cfg);

    // The budget is spent exactly, and the remainder is on the ledger.
    let report = control().observe(0, 64).observe(0, 36).finalize().unwrap();
    assert_eq!(report.ledger.remaining(), Some(0));
    assert_eq!(report.ledger.shots(), 100);

    // Past the budget is an overdraw, reported with its shortfall.
    let msg = calculation_message(
        control()
            .observe(0, 64)
            .observe(0, 64)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("overdrawn"), "{msg}");
    assert!(msg.contains("shortfall 28"), "{msg}");
}

#[cfg(feature = "qpu")]
#[test]
fn test_each_world_draws_on_its_own_copy_of_the_budget() {
    let cfg = budgeted_config(1024, 0, 0);
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 256)
        .fork()
        .observe(0, 256)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.remaining(), Some(768));
    for w in &report.worlds {
        assert_eq!(w.ledger().remaining(), Some(512));
    }

    // Each world's copy runs out on its own: 512 remain, 600 are asked for.
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 256)
        .fork()
        .observe(0, 256)
        .observe(0, 600)
        .finalize()
        .unwrap_err();
    assert!(calculation_message(err).contains("overdrawn"));
}

#[cfg(feature = "qpu")]
#[test]
fn test_the_baseline_draws_from_the_budget() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .baseline(Experiment::new("E0", 1.0, 4096, vec![0.0]).unwrap())
        .evidence(deep_causality_quantum::Evidence::shots(1000))
        .build()
        .unwrap();
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .baseline(0, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("overdrawn"), "{msg}");
}

#[cfg(feature = "qpu")]
#[test]
fn test_the_evidence_seed_selects_the_draws() {
    let read = |seed: u64, evidence_seed: u64| -> f64 {
        let report =
            QclBuilder::control::<f64, Count, 2, _>(&budgeted_config(1 << 16, seed, evidence_seed))
                .fork()
                .observe(0, 4096)
                .finalize()
                .unwrap();
        report.worlds[1].read_out().unwrap().estimate()
    };
    assert_eq!(read(1, 7), read(2, 7), "the run seed is not the draw seed");
    assert_ne!(read(1, 7), read(1, 8), "the evidence seed is");
}

// ---------------------------------------------------------------------------
// Configured experiments: design_with → observe_experiment → fork → predict_with
// ---------------------------------------------------------------------------

type Crosstalk = ConfiguredExperiment<f64, CrosstalkSetting>;

/// The crosstalk family: passive, hold Q1 and read Q2, hold Q2 and read Q1, 1024 shots each.
fn crosstalk_experiments() -> [Crosstalk; 3] {
    let e = |name: &str, setting| ConfiguredExperiment::new(name, 1.0, 1024, setting, 0).unwrap();
    [
        e("E0 passive", CrosstalkSetting::Passive),
        e(
            "E1 hold Q1",
            CrosstalkSetting::Hold {
                node: CROSSTALK_Q1,
                read: CROSSTALK_Q2,
            },
        ),
        e(
            "E2 hold Q2",
            CrosstalkSetting::Hold {
                node: CROSSTALK_Q2,
                read: CROSSTALK_Q1,
            },
        ),
    ]
}

/// The two-qubit plant in `|00⟩` with "qubit 2 excited", screened over the three candidates.
fn crosstalk_screen() -> deep_causality_quantum::Screened<
    f64,
    Count,
    PlantSubject<f64, 4, deep_causality_quantum::Structural>,
> {
    let zero = c(0.0);
    let plant =
        QuantumPlant::from_ket(&CausalTensor::from_slice(&[c(1.0), zero, zero, zero], &[4]))
            .unwrap();
    let e2 = Observable::new(
        "e2",
        Projection::<f64, 4>::new(mat(
            vec![
                zero,
                zero,
                zero,
                zero,
                zero,
                c(1.0),
                zero,
                zero,
                zero,
                zero,
                zero,
                zero,
                zero,
                zero,
                zero,
                c(1.0),
            ],
            4,
        ))
        .unwrap(),
    );
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[e2])
        .candidates(&crosstalk_candidates().unwrap())
        .seed(20_260_821)
        .build()
        .unwrap();
    QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap()
}

/// Plan, observe E1 from `source`, then predict, compare and adjudicate.
fn run_crosstalk(
    source: &EvidenceSource<f64>,
) -> deep_causality_quantum::ControlReport<f64, Count, 4> {
    let screened = crosstalk_screen();
    let exps = crosstalk_experiments();
    QclBuilder::control::<f64, Count, 4, _>(&screened)
        .design_with(&CrosstalkModel, &exps, MinCostCover::new(5.0))
        .observe_experiment(&CrosstalkModel, &exps[1], source)
        .fork()
        .predict_with(&CrosstalkModel, &exps[1])
        .compare(3.0)
        .adjudicate(5.0)
        .finalize()
        .unwrap()
}

#[test]
fn test_a_configured_plan_runs_against_simulated_evidence_and_names_the_truth() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let report = run_crosstalk(&EvidenceSource::Simulated(h1));

    // The plan: E1 separates H1 from both rivals and E2 separates H2 from H3; passive reads
    // 0.04 in every candidate and separates nothing.
    let plan = report.plan.as_ref().unwrap();
    assert!(plan.is_complete());
    assert_eq!(plan.total_cost(), 2.0);
    let names: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["E1 hold Q1", "E2 hold Q2"]);
    assert_eq!(report.ledger.cost(), 2.0);

    // One observation, recorded with its experiment, shots and counts; a setting has no context.
    assert_eq!(report.observations.len(), 1);
    let o = &report.observations[0];
    assert_eq!(o.experiment(), "E1 hold Q1");
    assert_eq!(o.shots(), 1024);
    assert_eq!(o.counts().map(|h| h.total()), Some(1024));
    assert_eq!(o.context(), None);
    assert_eq!(
        (report.ledger.experiments(), report.ledger.shots()),
        (1, 1024)
    );

    // Each world's prediction is its model's, counted once; the truth survives.
    let predictions: Vec<f64> = report
        .worlds
        .iter()
        .map(|w| w.prediction().unwrap())
        .collect();
    for (p, want) in predictions.iter().zip([0.4, 0.1, 0.1]) {
        assert!((p - want).abs() < 1e-12, "{predictions:?}");
    }
    for w in &report.worlds {
        assert_eq!((w.ledger().predictions(), w.ledger().experiments()), (1, 1));
    }
    match &report.adjudication.as_ref().unwrap().outcome {
        Either::Left(s) => assert_eq!(s.name, "H1 Q1->Q2"),
        other => panic!("expected H1 to survive, got {other:?}"),
    }
}

#[test]
fn test_recorded_counts_reproduce_the_simulated_verdict() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let simulated = run_crosstalk(&EvidenceSource::Simulated(h1));
    let counts = simulated.observations[0].counts().unwrap().clone();
    let recorded = run_crosstalk(&EvidenceSource::Recorded(counts));
    assert_eq!(recorded.ledger, simulated.ledger);
    for (r, s) in recorded.worlds.iter().zip(&simulated.worlds) {
        assert_eq!(r.verdict(), s.verdict(), "{}", r.name());
        assert_eq!(r.read_out(), s.read_out());
    }
    assert_eq!(
        recorded.adjudication.unwrap().outcome,
        simulated.adjudication.unwrap().outcome
    );
    assert_eq!(
        recorded.observations[0].read_out(),
        simulated.observations[0].read_out()
    );
}

/// The qubit's response to an interferometer context: the wave vector pointing down flips the
/// read-out, up leaves it.
struct KReversal;

impl ResponseModel<f64, InterferometerContext<f64>> for KReversal {
    fn respond(
        &self,
        _: &Hypothesis<f64>,
        context: &InterferometerContext<f64>,
    ) -> Result<Response<f64>, QuantumError> {
        let u = match instrument_configuration(context)?.wave_vector() {
            WaveVector::Up => QubitOperator::identity(),
            WaveVector::Down => QubitOperator::pauli_x(),
        };
        Ok(Response::Channel(Channel::unitary(&u)?))
    }
}

fn gravimeter(wave_vector: WaveVector) -> InterferometerContext<f64> {
    let model = InterferometerModel::new(1.6106e7, 0.1, 0.5, 2.4e-7, 0.5, 1000.0, 30.0).unwrap();
    let configuration = InterferometerConfiguration::new(1.0e-5, 2.0e-6, 1.5708e5)
        .unwrap()
        .with_wave_vector(wave_vector);
    let mut context = interferometer_context(1, "gravimeter", model, configuration).unwrap();
    record_environment(
        &mut context,
        0,
        TimeScale::Second,
        EnvironmentReading::new(1.2e-6, 296.15, 4.8e-5).unwrap(),
    )
    .unwrap();
    context
}

fn kreversal_config(
    p: f64,
) -> deep_causality_quantum::Config<f64, Count, PlantSubject<f64, 2, Mechanisms>> {
    QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(p), &[excited()])
        .mechanisms(&[
            mechanism("keep", QubitOperator::identity()),
            mechanism("flip", QubitOperator::pauli_x()),
        ])
        .seed(9)
        .build()
        .unwrap()
}

#[test]
fn test_an_observation_records_the_context_it_ran_in() {
    let context = gravimeter(WaveVector::Down);
    let e = ConfiguredExperiment::new("E1 k-down", 1.0, 1024, context.clone(), 0).unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&kreversal_config(0.2))
        .observe_experiment(
            &KReversal,
            &e,
            &EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity())),
        )
        .finalize()
        .unwrap();
    let o = &report.observations[0];
    assert_eq!(o.context(), Some(&context.snapshot().unwrap()));
    // k reversed flips the plant's 0.2 to 0.8; 1024 shots put it within 0.05.
    assert!(
        (o.read_out().estimate() - 0.8).abs() < 0.05,
        "{}",
        o.read_out().estimate()
    );
}

/// The passive configuration observed first from `keep`: the plant's 0.2 under k up.
fn kreversal_baseline(
    cfg: &deep_causality_quantum::Config<f64, Count, PlantSubject<f64, 2, Mechanisms>>,
) -> deep_causality_quantum::Control<f64, Count, 2> {
    let e0 =
        ConfiguredExperiment::new("E0 k-up", 1.0, 1024, gravimeter(WaveVector::Up), 0).unwrap();
    QclBuilder::control::<f64, Count, 2, _>(cfg).baseline_with(
        &KReversal,
        &e0,
        &EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity())),
        3.0,
    )
}

#[test]
fn test_baseline_with_refuses_what_its_evidence_contradicts_before_the_fork() {
    // `keep` predicts the plant's 0.2 and `flip` 0.8; the evidence, drawn from `keep`, refuses
    // `flip` with what decided it.
    let report = kreversal_baseline(&kreversal_config(0.2))
        .fork()
        .finalize()
        .unwrap();
    assert_eq!(report.refused.len(), 1);
    let r = &report.refused[0];
    assert_eq!(r.name(), "flip");
    assert!((r.prediction() - 0.8).abs() < 1e-12);
    assert_eq!(r.observed().shots(), 1024);
    let check = r.check();
    assert_eq!(check.item, CheckItem::Index(1));
    assert!(!check.accepted);
    assert_eq!(
        check.measured,
        (r.prediction() - r.observed().estimate()).abs()
    );
    assert_eq!(check.threshold, 3.0 * r.observed().standard_error());
    let names: Vec<&str> = report.worlds.iter().map(|w| w.name()).collect();
    assert_eq!(names, vec!["keep"]);
    // One observation, charged to the root, recorded with the context it ran in.
    assert_eq!(
        (report.ledger.experiments(), report.ledger.shots()),
        (1, 1024)
    );
    assert_eq!(report.observations.len(), 1);
    let o = &report.observations[0];
    assert_eq!(o.experiment(), "E0 k-up");
    assert_eq!(
        o.context(),
        Some(&gravimeter(WaveVector::Up).snapshot().unwrap())
    );
}

#[test]
fn test_baseline_with_is_the_read_out_compare_judges_against() {
    let report = kreversal_baseline(&kreversal_config(0.2))
        .fork()
        .predict(0)
        .compare(3.0)
        .finalize()
        .unwrap();
    let w = &report.worlds[0];
    assert!(w.verdict().unwrap().accepted());
    assert_eq!(w.read_out().unwrap().shots(), 1024);
}

#[test]
fn test_baseline_with_is_the_first_observation_and_checks_its_arguments() {
    let cfg = kreversal_config(0.2);
    let passive =
        ConfiguredExperiment::new("E0 k-up", 1.0, 64, gravimeter(WaveVector::Up), 0).unwrap();
    let keep = EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity()));
    let control = || QclBuilder::control::<f64, Count, 2, _>(&cfg);
    for (after, run) in [
        ("an observation", control().observe(0, 16)),
        ("the fork", control().fork()),
    ] {
        let msg = calculation_message(
            run.baseline_with(&KReversal, &passive, &keep, 3.0)
                .finalize()
                .unwrap_err(),
        );
        assert!(msg.contains("first observation"), "after {after}: {msg}");
    }
    for sigmas in [-1.0, f64::NAN] {
        let msg = calculation_message(
            control()
                .baseline_with(&KReversal, &passive, &keep, sigmas)
                .finalize()
                .unwrap_err(),
        );
        assert!(msg.contains("sigmas"), "{msg}");
    }
    assert!(
        control()
            .baseline_with(&KReversal, &passive, &keep, 0.0)
            .finalize()
            .is_ok()
    );
    // A typed baseline the config names is observed first.
    let named = baseline_config(e0());
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&named)
            .baseline_with(&KReversal, &passive, &keep, 3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("call baseline before baseline_with"), "{msg}");
}

#[test]
fn test_baseline_with_keeps_the_first_failure_and_fails_on_empty_evidence() {
    let cfg = kreversal_config(0.2);
    let passive =
        ConfiguredExperiment::new("E0 k-up", 1.0, 64, gravimeter(WaveVector::Up), 0).unwrap();
    let keep = EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity()));
    // An earlier stage's failure stands.
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(5, 16)
        .baseline_with(&KReversal, &passive, &keep, 3.0)
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    // Evidence that carries no estimate fails the stage, which refuses no candidate.
    let empty = EvidenceSource::Recorded(CountHistogram::new(1).unwrap());
    let err = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .baseline_with(&KReversal, &passive, &empty, 3.0)
        .finalize()
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::NormalizationError(_)),
        "{err:?}"
    );
}

#[test]
fn test_a_mechanism_truth_draws_what_observe_draws_on_its_plant() {
    // The bare plant as the truth under k up: the same state `observe` samples, at the same seed.
    let cfg = kreversal_config(0.3);
    let up = ConfiguredExperiment::new("E0 k-up", 1.0, 512, gravimeter(WaveVector::Up), 0).unwrap();
    let bare = EvidenceSource::Simulated(mechanism("bare", QubitOperator::identity()));
    let observed = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 512)
        .fork()
        .predict(0)
        .compare(3.0)
        .finalize()
        .unwrap();
    let configured = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe_experiment(&KReversal, &up, &bare)
        .fork()
        .predict(0)
        .compare(3.0)
        .finalize()
        .unwrap();
    for (a, b) in observed.worlds.iter().zip(&configured.worlds) {
        assert_eq!(a.verdict(), b.verdict(), "{}", a.name());
    }
    assert_eq!(observed.ledger, configured.ledger);
}

#[test]
fn test_a_published_value_enters_as_effective_draws() {
    let fringe = Fringe::new(0.5, 40_265.0).unwrap();
    let draws = fringe.effective_draws(-5.0e-8, 2.4e-7).unwrap();
    let e = ConfiguredExperiment::new("published", 0.0, 1, gravimeter(WaveVector::Up), 0).unwrap();
    let report = QclBuilder::control::<f64, Count, 2, _>(&kreversal_config(0.5))
        .observe_experiment(&KReversal, &e, &EvidenceSource::Published(draws))
        .finalize()
        .unwrap();
    let o = &report.observations[0];
    assert_eq!(o.read_out().estimate(), draws.probability());
    assert_eq!(o.read_out().standard_error(), draws.standard_error());
    assert_eq!(o.shots(), draws.draws().round() as u64);
    assert_eq!(o.counts(), None);
    assert_eq!(report.ledger.shots(), o.shots());
}

#[test]
fn test_after_the_fork_an_observed_experiment_is_charged_to_every_world() {
    let exps = crosstalk_experiments();
    let [h1, ..] = crosstalk_candidates().unwrap();
    let report = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .fork()
        .observe_experiment(&CrosstalkModel, &exps[2], &EvidenceSource::Simulated(h1))
        .finalize()
        .unwrap();
    // The root ledger stops at the fork; the device's evidence is in every world's history.
    assert_eq!(report.ledger.experiments(), 0);
    for w in &report.worlds {
        assert_eq!((w.ledger().experiments(), w.ledger().shots()), (1, 1024));
    }
    assert_eq!(report.observations.len(), 1);
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
            .predict_with(&CrosstalkModel, &exps[2])
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("fork first"), "{msg}");
}

// ---------------------------------------------------------------------------
// Campaigns: one reading per observed experiment, adjudicated together
// ---------------------------------------------------------------------------

/// Fork, then observe `experiment` from H1 `times` times, predicting and comparing each time.
fn crosstalk_campaign(
    experiment: &Crosstalk,
    times: usize,
) -> deep_causality_quantum::ControlReport<f64, Count, 4> {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let source = EvidenceSource::Simulated(h1);
    let mut control = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen()).fork();
    for _ in 0..times {
        control = control
            .observe_experiment(&CrosstalkModel, experiment, &source)
            .predict_with(&CrosstalkModel, experiment)
            .compare(3.0);
    }
    control.adjudicate(5.0).finalize().unwrap()
}

#[test]
fn test_a_campaign_separates_what_one_experiment_cannot() {
    // E1 at 40 shots separates H1 (0.40) from its rivals (0.10) by about 3.9 bits; twice, by
    // about 7.8.
    let half = crosstalk_experiments()[1].clone().with_shots(40).unwrap();
    let once = crosstalk_campaign(&half, 1);
    assert!(matches!(
        once.adjudication.unwrap().outcome,
        Either::Right(deep_causality_quantum::Ambiguity::Unseparated { .. })
    ));
    let twice = crosstalk_campaign(&half, 2);
    for w in &twice.worlds {
        assert_eq!(w.readings().len(), 2);
        assert!(w.readings().iter().all(|r| r.experiment() == "E1 hold Q1"));
        assert_eq!((w.ledger().experiments(), w.ledger().predictions()), (2, 2));
    }
    match &twice.adjudication.as_ref().unwrap().outcome {
        Either::Left(s) => {
            assert_eq!(s.name, "H1 Q1->Q2");
            assert!(s.separation_bits > 5.0, "{}", s.separation_bits);
        }
        other => panic!("expected H1 to survive, got {other:?}"),
    }
    assert_eq!(twice.observations.len(), 2);
}

#[test]
fn test_a_second_compare_of_one_observation_replaces_its_reading() {
    let cfg = kreversal_config(0.3);
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .observe(0, 256)
        .fork()
        .predict(0)
        .compare(3.0)
        .predict(0)
        .compare(2.0)
        .finalize()
        .unwrap();
    for w in &report.worlds {
        assert_eq!(w.readings().len(), 1, "{}", w.name());
        let r = &w.readings()[0];
        assert_eq!(r.experiment(), "excited");
        assert_eq!(r.prediction(), w.prediction());
        assert_eq!(r.read_out(), w.read_out().unwrap());
    }
    // The prediction is consumed: compare again without one names predict.
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .observe(0, 256)
            .fork()
            .predict(0)
            .compare(3.0)
            .compare(3.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("call predict before compare"), "{msg}");
}

#[test]
fn test_gate_judges_every_unjudged_reading_and_needs_one() {
    let cfg = two_mechanism_config();
    let report = QclBuilder::control::<f64, Count, 2, _>(&cfg)
        .fork()
        .observe(0, 256)
        .observe(0, 256)
        .gate(Spec::at_least(0.9))
        .adjudicate(5.0)
        .finalize()
        .unwrap();
    for w in &report.worlds {
        assert_eq!(w.readings().len(), 2);
        assert!(w.readings().iter().all(|r| r.verdict().is_some()));
    }
    // flip reads 1 twice, keep reads 0 twice: 512 shots of certainty separate them.
    match &report.adjudication.as_ref().unwrap().outcome {
        Either::Left(s) => assert_eq!(s.name, "flip"),
        other => panic!("expected flip to survive, got {other:?}"),
    }
    // Once every reading is judged there is nothing left to gate.
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .fork()
            .observe(0, 16)
            .gate(Spec::at_least(0.9))
            .gate(Spec::at_least(0.9))
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("unjudged"), "{msg}");
}

#[test]
fn test_a_reading_no_stage_judged_stops_adjudicate() {
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&two_mechanism_config())
            .fork()
            .observe(0, 16)
            .adjudicate(5.0)
            .finalize()
            .unwrap_err(),
    );
    assert!(
        msg.contains("without an observed and gated read-out"),
        "{msg}"
    );
}

// ---------------------------------------------------------------------------
// Instrument time: device time in seconds, and a plan sized in time
// ---------------------------------------------------------------------------

#[test]
fn test_device_time_is_seconds_when_the_config_names_an_instrument_time() {
    let timed = QclBuilder::config::<f64, Count>()
        .over_plant(plant_ground(), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .instrument_time(deep_causality_quantum::InstrumentTime::new(0.25, 1000.0).unwrap())
        .build()
        .unwrap();
    assert_eq!(timed.instrument_time().map(|t| t.shot_time()), Some(0.25));
    let report = QclBuilder::control::<f64, Count, 2, _>(&timed)
        .observe(0, 100)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.device_time(), 25.0);
    // Without it, a shot is one unit.
    let report = QclBuilder::control::<f64, Count, 2, _>(&two_mechanism_config())
        .observe(0, 100)
        .finalize()
        .unwrap();
    assert_eq!(report.ledger.device_time(), 100.0);
}

#[test]
fn test_a_plan_sized_in_time_runs_at_its_planned_shots() {
    // E2 separates H1 from both rivals; sized in time, it takes the fewest shots that do.
    let time = deep_causality_quantum::InstrumentTime::new(0.01, 100.0).unwrap();
    let exps = crosstalk_experiments();
    let report = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .design_with(&CrosstalkModel, &exps, MinCostCover::new(5.0).timed(time))
        .finalize()
        .unwrap();
    let plan = report.plan.unwrap();
    assert!(plan.is_complete());
    let entry = plan
        .entries()
        .iter()
        .find(|e| e.name == "E2 hold Q2")
        .unwrap();
    assert!(entry.shots < 1024, "{}", entry.shots);
    assert_eq!(entry.cost, 1.0 + entry.shots as f64 * 0.01);
    let sized = exps[2].clone().with_shots(entry.shots).unwrap();
    assert_eq!(sized.shots(), entry.shots);
    assert!(matches!(
        exps[2].clone().with_shots(0).unwrap_err().0,
        QuantumErrorEnum::NormalizationError(_)
    ));
}

#[test]
fn test_a_pending_baseline_comes_before_a_configured_experiment() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .mechanisms(&[
            mechanism("keep", QubitOperator::identity()),
            mechanism("flip", QubitOperator::pauli_x()),
        ])
        .baseline(Experiment::new("E0", 1.0, 64, vec![0.2, 0.8]).unwrap())
        .build()
        .unwrap();
    let e = ConfiguredExperiment::new("k", 1.0, 64, gravimeter(WaveVector::Up), 0).unwrap();
    let source = EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity()));
    let control = || QclBuilder::control::<f64, Count, 2, _>(&cfg);
    let msg = calculation_message(
        control()
            .observe_experiment(&KReversal, &e, &source)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("before observe_experiment"), "{msg}");
    let msg = calculation_message(
        control()
            .design_with(&KReversal, std::slice::from_ref(&e), MinCostCover::new(5.0))
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("before design"), "{msg}");
}

/// The crosstalk model with its instrument doubled, so every prediction is twice the model's.
struct Doubled;

impl ResponseModel<f64, CrosstalkSetting> for Doubled {
    fn respond(
        &self,
        candidate: &Hypothesis<f64>,
        setting: &CrosstalkSetting,
    ) -> Result<Response<f64>, QuantumError> {
        match CrosstalkModel.respond(candidate, setting)? {
            Response::Intervention {
                factors,
                instrument,
            } => Ok(Response::Intervention {
                factors,
                instrument: instrument * Complex::new(20.0, 0.0),
            }),
            channel => Ok(channel),
        }
    }
}

#[test]
fn test_a_prediction_that_is_not_a_probability_is_refused() {
    let exps = crosstalk_experiments();
    // H1 holding Q2 reads 0.1 under the model, 2.0 doubled twenty-fold.
    let [h1, ..] = crosstalk_candidates().unwrap();
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .observe_experiment(&Doubled, &exps[2], &EvidenceSource::Simulated(h1))
        .finalize()
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert!(msg.contains("truth 'H1 Q1->Q2'"), "{msg}")
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .design_with(&Doubled, &exps, MinCostCover::new(5.0))
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NormalizationError(_)));
}

#[test]
fn test_an_empty_recording_is_refused_and_records_nothing() {
    let exps = crosstalk_experiments();
    let empty = CountHistogram::new(1).unwrap();
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .observe_experiment(&CrosstalkModel, &exps[0], &EvidenceSource::Recorded(empty))
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NormalizationError(_)));
}

#[cfg(feature = "qpu")]
#[test]
fn test_a_configured_experiment_draws_from_the_budget() {
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .mechanisms(&[mechanism("keep", QubitOperator::identity())])
        .evidence(deep_causality_quantum::Evidence::shots(1000))
        .build()
        .unwrap();
    let e = ConfiguredExperiment::new("k", 1.0, 1024, gravimeter(WaveVector::Up), 0).unwrap();
    let source = EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity()));
    let msg = calculation_message(
        QclBuilder::control::<f64, Count, 2, _>(&cfg)
            .observe_experiment(&KReversal, &e, &source)
            .finalize()
            .unwrap_err(),
    );
    assert!(msg.contains("overdrawn"), "{msg}");
}
