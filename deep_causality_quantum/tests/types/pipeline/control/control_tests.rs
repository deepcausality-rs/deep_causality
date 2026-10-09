/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The control stage: what `fork` hands a world, the refusal of an empty fork, `compare` on the
//! structural path, the mechanism path on each world's own plant, the baseline that refuses
//! candidates before the plan, and the shot budget every observation draws from. Configured
//! experiments are in `control_experiments_tests.rs`.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CheckItem, CommutatorTolerance, Experiment, FactorSupports, Hypothesis, Mechanisms,
    MinCostCover, Observable, PlantSubject, ProcessFactors, QclBuilder, QuantumErrorEnum,
    QuantumPlant, QubitOperator, Spec,
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
