/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The control stage's configured experiments: planned, predicted and observed against their
//! evidence, read as a campaign, and timed on the instrument.

use deep_causality_context::TimeScale;
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::{
    CROSSTALK_Q1, CROSSTALK_Q2, CrosstalkModel, CrosstalkSetting, crosstalk_candidates,
};
use deep_causality_quantum::{
    Channel, CheckItem, CommutatorTolerance, ConfiguredExperiment, CountHistogram,
    EnvironmentReading, EvidenceSource, Experiment, Fringe, Hypothesis,
    InterferometerConfiguration, InterferometerContext, InterferometerModel, Mechanisms,
    MinCostCover, Observable, PlantSubject, Projection, QclBuilder, QuantumError, QuantumErrorEnum,
    QuantumPlant, QubitOperator, Response, ResponseModel, ShotHistogram, Spec, WaveVector,
    instrument_configuration, interferometer_context, record_environment,
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

fn calculation_message(e: deep_causality_quantum::QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
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

/// Two probes over three candidates `a, b, c`: `ac` separates `a` from `c` and leaves `b` with
/// `a`; `ab` separates `a` from `b` and leaves `c` with `a`. With `b` gone only `ac` covers the
/// remaining pair, and a plan reading the predictions of `a` and `b` would pick `ab`.
fn probes_over_a_b_c() -> Vec<Experiment<f64>> {
    vec![
        Experiment::new("ac", 1.0, 256, vec![0.1, 0.1, 0.9]).unwrap(),
        Experiment::new("ab", 0.5, 256, vec![0.1, 0.9, 0.1]).unwrap(),
    ]
}

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
fn test_baseline_with_counts_one_prediction_per_candidate_it_judged() {
    // Three candidates judged, `flip` refused: three model evaluations on the root, which the
    // surviving worlds inherit at the fork, and predict_with counts one more in each.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant_with_population(0.2), &[excited()])
        .mechanisms(&[
            mechanism("keep", QubitOperator::identity()),
            mechanism("flip", QubitOperator::pauli_x()),
            mechanism("also keep", QubitOperator::identity()),
        ])
        .seed(9)
        .build()
        .unwrap();
    let e0 =
        ConfiguredExperiment::new("E0 k-up", 1.0, 1024, gravimeter(WaveVector::Up), 0).unwrap();
    let report = kreversal_baseline(&cfg)
        .fork()
        .predict_with(&KReversal, &e0)
        .finalize()
        .unwrap();
    assert_eq!(report.refused.len(), 1);
    assert_eq!(report.ledger.predictions(), 3);
    assert_eq!(report.worlds.len(), 2);
    for w in &report.worlds {
        assert_eq!(w.ledger().predictions(), 4, "{}", w.name());
    }
    // Planning evaluates the model without counting.
    let planned = kreversal_baseline(&cfg)
        .design_with(&KReversal, &[e0], MinCostCover::new(5.0))
        .finalize()
        .unwrap();
    assert_eq!(planned.ledger.predictions(), 3);
}

#[test]
fn test_baseline_with_fails_when_its_predictions_overflow_the_count_width() {
    // A u8 ledger counts 255 evaluations; the 256th candidate's overflows it.
    let passive =
        ConfiguredExperiment::new("E0 k-up", 1.0, 64, gravimeter(WaveVector::Up), 0).unwrap();
    let keep = EvidenceSource::Simulated(mechanism("keep", QubitOperator::identity()));
    let run = |candidates: usize| {
        let mechanisms: Vec<Hypothesis<f64>> = (0..candidates)
            .map(|i| mechanism(&format!("keep {i}"), QubitOperator::identity()))
            .collect();
        let cfg = QclBuilder::config::<f64, u8>()
            .over_plant(plant_with_population(0.2), &[excited()])
            .mechanisms(&mechanisms)
            .seed(9)
            .build()
            .unwrap();
        QclBuilder::control::<f64, u8, 2, _>(&cfg)
            .baseline_with(&KReversal, &passive, &keep, 3.0)
            .finalize()
    };
    assert_eq!(run(255).unwrap().ledger.predictions(), 255);
    let msg = calculation_message(run(256).unwrap_err());
    assert!(msg.contains("prediction count overflows"), "{msg}");
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
    // The simulated truth's prediction is refused naming the truth and the experiment.
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert!(
                msg.contains("'H1 Q1->Q2'") && msg.contains("'E2 hold Q2'"),
                "{msg}"
            )
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .design_with(&Doubled, &exps, MinCostCover::new(5.0))
        .finalize()
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NormalizationError(_)));
    // After the fork, each world's prediction is refused before `compare` would judge it.
    let mut recorded = CountHistogram::new(1).unwrap();
    recorded.record_n(1, 100).unwrap();
    recorded.record_n(0, 924).unwrap();
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .observe_experiment(
            &CrosstalkModel,
            &exps[2],
            &EvidenceSource::Recorded(recorded.clone()),
        )
        .fork()
        .predict_with(&Doubled, &exps[2])
        .finalize()
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert!(msg.contains("predicts 2.0"), "{msg}")
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
}

#[test]
fn test_baseline_with_refuses_a_prediction_that_is_not_a_probability() {
    // Recorded counts never ask the model for the truth's prediction, so the candidates'
    // predictions of 2.0, 8.0 and 2.0 reach the baseline's gap test unless `predict` refuses
    // them. They are an invalid response, not three contradicted candidates.
    let exps = crosstalk_experiments();
    let mut recorded = CountHistogram::new(1).unwrap();
    recorded.record_n(1, 100).unwrap();
    recorded.record_n(0, 924).unwrap();
    let err = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .baseline_with(
            &Doubled,
            &exps[2],
            &EvidenceSource::Recorded(recorded.clone()),
            3.0,
        )
        .finalize()
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert!(
                msg.contains("'H1 Q1->Q2'") && msg.contains("not a probability"),
                "{msg}"
            )
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
    // The same evidence under the model as it is refuses by the gap test and keeps the rest.
    let report = QclBuilder::control::<f64, Count, 4, _>(&crosstalk_screen())
        .baseline_with(
            &CrosstalkModel,
            &exps[2],
            &EvidenceSource::Recorded(recorded),
            3.0,
        )
        .finalize()
        .unwrap();
    let refused: Vec<&str> = report.refused.iter().map(|r| r.name()).collect();
    assert_eq!(refused, vec!["H2 Q2->Q1"]);
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
