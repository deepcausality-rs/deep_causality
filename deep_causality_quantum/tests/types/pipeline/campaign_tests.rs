/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sequential campaign over the crosstalk candidates, on `CausalFlow::iterate_until`.
//!
//! The experiments read, under H₁ / H₂ / H₃: E0 passive 0.04 / 0.04 / 0.04, E1 (hold Q1, read Q2)
//! 0.40 / 0.10 / 0.10, E2 (hold Q2, read Q1) 0.10 / 0.40 / 0.10, E3 echo 0.01 / 0.01 / 0.04. The
//! static plan is E1 and E2 at cost 2.

use deep_causality_core::CausalFlow;
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::{
    CROSSTALK_Q1, CROSSTALK_Q2, CrosstalkModel, CrosstalkSetting, crosstalk_candidates,
    crosstalk_plant,
};
use deep_causality_quantum::{
    Ambiguity, Campaign, CampaignRules, CampaignStop, CommutatorTolerance, ConfiguredExperiment,
    Control, EvidenceSource, FactorSupports, Hypothesis, MinCostCover, ProcessFactors, QclBuilder,
    QuantumErrorEnum, design,
};
use deep_causality_tensor::CausalTensor;

type Count = u64;
type Crosstalk = ConfiguredExperiment<f64, CrosstalkSetting>;
type Flow = CausalFlow<Campaign<f64, Count, 4>, (), Vec<Crosstalk>>;

fn experiments() -> Vec<Crosstalk> {
    let e = |name: &str, cost: f64, setting| {
        ConfiguredExperiment::new(name, cost, 1024, setting, 0).unwrap()
    };
    let hold = |node, read| CrosstalkSetting::Hold { node, read };
    vec![
        e("E0 passive", 1.0, CrosstalkSetting::Passive),
        e("E1 hold Q1", 1.0, hold(CROSSTALK_Q1, CROSSTALK_Q2)),
        e("E2 hold Q2", 1.0, hold(CROSSTALK_Q2, CROSSTALK_Q1)),
        e("E3 echo", 2.0, CrosstalkSetting::Echo),
    ]
}

/// The control stage over the three candidates, screened and forked.
fn forked() -> Control<f64, Count, 4> {
    let (plant, e2) = crosstalk_plant().unwrap();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[e2])
        .candidates(&crosstalk_candidates().unwrap())
        .seed(20_260_821)
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_normalization()
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    QclBuilder::control::<f64, Count, 4, _>(&screened).fork()
}

fn rules() -> CampaignRules<f64> {
    CampaignRules::new(MinCostCover::new(5.0), 3.0, 0.01).unwrap()
}

/// The campaign over `exps` from `truth`, stepped to its stop.
fn run(truth: &Hypothesis<f64>, exps: Vec<Crosstalk>) -> Campaign<f64, Count, 4> {
    let source = EvidenceSource::Simulated(truth.clone());
    let steps = exps.len() + 1;
    CausalFlow::value(Campaign::new(forked()))
        .context(exps)
        .iterate_until(Campaign::is_stopped, steps, |f| {
            Campaign::step(f, &CrosstalkModel, &source, &rules())
        })
        .finish()
        .unwrap()
}

fn survivor(campaign: &Campaign<f64, Count, 4>) -> &str {
    match campaign.stop() {
        Some(CampaignStop::Survivor(s)) => &s.name,
        other => panic!("expected a survivor, got {other:?}"),
    }
}

#[test]
fn test_each_cause_survives_within_the_static_plans_cost() {
    let candidates = crosstalk_candidates().unwrap();
    let plant = crosstalk_plant().unwrap().0;
    let probes: Vec<_> = experiments()
        .iter()
        .map(|e| {
            let p: Vec<f64> = candidates
                .iter()
                .map(|h| e.predict::<_, 4>(&CrosstalkModel, h, &plant, &[]).unwrap())
                .collect();
            deep_causality_quantum::Experiment::new(e.name(), e.cost(), e.shots(), p).unwrap()
        })
        .collect();
    let static_cost = design(3, &probes, MinCostCover::new(5.0))
        .unwrap()
        .total_cost();
    assert_eq!(static_cost, 2.0);
    // H₁ stops after E1 alone; H₂ and H₃ need E2 after it.
    for (truth, run_order) in candidates.iter().zip([vec![1], vec![1, 2], vec![1, 2]]) {
        let c = run(truth, experiments());
        assert_eq!(survivor(&c), truth.name());
        assert_eq!(
            c.experiments_run(),
            run_order.as_slice(),
            "{}",
            truth.name()
        );
        assert!(
            c.spent() <= static_cost,
            "{} spent {}",
            truth.name(),
            c.spent()
        );
        assert_eq!(c.replans(), 0);
        let report = c.finalize().unwrap();
        assert_eq!(report.ledger.cost(), run_order.len() as f64);
        assert_eq!(report.observations.len(), run_order.len());
    }
}

#[test]
fn test_observations_no_candidate_explains_stop_outside_the_model() {
    // A truth that excites Q2 nine times in ten whatever Q1 does.
    let mut pf = ProcessFactors::new();
    let diag = |a: f64, b: f64| {
        CausalTensor::from_slice(&[a, 0.0, 0.0, b].map(|v| Complex::new(v, 0.0)), &[2, 2])
    };
    pf.insert(CROSSTALK_Q1, diag(0.9, 0.1));
    pf.insert(CROSSTALK_Q2, diag(0.1, 0.9));
    let mut fs = FactorSupports::new();
    fs.declare(CROSSTALK_Q1, &[CROSSTALK_Q1]);
    fs.declare(CROSSTALK_Q2, &[CROSSTALK_Q2]);
    let stranger = Hypothesis::structural("stranger", pf, fs).unwrap();
    let c = run(&stranger, experiments());
    assert_eq!(c.stop(), Some(&CampaignStop::OutsideTheModel));
    assert_eq!(c.experiments_run(), &[1]);
}

#[test]
fn test_no_experiment_that_separates_stops_unresolvable_before_any_run() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let passive_only = experiments().into_iter().take(1).collect();
    let c = run(&h1, passive_only);
    assert!(c.experiments_run().is_empty());
    match c.stop() {
        Some(CampaignStop::Unresolvable(Ambiguity::SeveralSurvive { survivors })) => {
            assert_eq!(survivors.len(), 3)
        }
        other => panic!("expected an unresolvable ambiguity, got {other:?}"),
    }
}

#[test]
fn test_running_out_of_experiments_stops_unresolvable() {
    // Only E1: under H₂ it rejects H₁ and leaves H₂ and H₃, which it cannot tell apart.
    let [_, h2, _] = crosstalk_candidates().unwrap();
    let e1_only = experiments().into_iter().skip(1).take(1).collect();
    let c = run(&h2, e1_only);
    assert_eq!(c.experiments_run(), &[0]);
    match c.stop() {
        Some(CampaignStop::Unresolvable(Ambiguity::SeveralSurvive { survivors })) => {
            assert_eq!(survivors, &["H2 Q2->Q1", "H3 Q1<-B->Q2"])
        }
        other => panic!("expected an unresolvable ambiguity, got {other:?}"),
    }
}

/// One step, then `context`, then the rest of the campaign.
fn with_context_change(truth: &Hypothesis<f64>, context: Vec<Crosstalk>) -> Flow {
    let source = EvidenceSource::Simulated(truth.clone());
    let step = |f: Flow| Campaign::step(f, &CrosstalkModel, &source, &rules());
    step(CausalFlow::value(Campaign::new(forked())).context(experiments()))
        .alternate_context(context)
        .iterate_until(Campaign::is_stopped, 5, step)
}

#[test]
fn test_a_context_change_beyond_the_drift_replans() {
    // Under H₂ the campaign plans E2 after E1. The new context reads Q2 instead of Q1 under E2,
    // which moves its predictions; the campaign re-plans and separates with the echo instead.
    let [_, h2, _] = crosstalk_candidates().unwrap();
    let mut drifted = experiments();
    drifted[2] = ConfiguredExperiment::new(
        "E2 hold Q2, read Q2",
        1.0,
        1024,
        CrosstalkSetting::Hold {
            node: CROSSTALK_Q2,
            read: CROSSTALK_Q2,
        },
        0,
    )
    .unwrap();
    let process = with_context_change(&h2, drifted).into_process();
    assert!(format!("{:?}", process.logs()).contains("!!ContextAlternation!!"));
    let c = CausalFlow::from(process).finish().unwrap();
    assert_eq!(c.replans(), 1);
    assert_eq!(c.experiments_run(), &[1, 3]);
    assert_eq!(survivor(&c), "H2 Q2->Q1");
}

#[test]
fn test_a_context_change_within_the_drift_keeps_the_plan() {
    let [_, h2, _] = crosstalk_candidates().unwrap();
    let c = with_context_change(&h2, experiments()).finish().unwrap();
    assert_eq!(c.replans(), 0);
    assert_eq!(c.experiments_run(), &[1, 2]);
    // The plan dropped from the context altogether also re-plans.
    let c = with_context_change(&h2, experiments().into_iter().take(2).collect())
        .finish()
        .unwrap();
    assert_eq!(c.replans(), 1);
}

#[test]
fn test_a_stopped_campaign_steps_to_itself() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let c = run(&h1, experiments());
    let source = EvidenceSource::Simulated(h1);
    let again = c
        .advance(&experiments(), &CrosstalkModel, &source, &rules())
        .unwrap();
    assert_eq!(again.experiments_run(), &[1]);
    assert!(again.is_stopped());
}

#[test]
fn test_a_campaign_needs_forked_worlds_and_a_context() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let source = EvidenceSource::Simulated(h1);
    let (plant, e2) = crosstalk_plant().unwrap();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[e2])
        .candidates(&crosstalk_candidates().unwrap())
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();
    let unforked = Campaign::new(QclBuilder::control::<f64, Count, 4, _>(&screened));
    match unforked
        .advance(&experiments(), &CrosstalkModel, &source, &rules())
        .map(|_| ())
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("fork"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
    let flow: Flow = CausalFlow::value(Campaign::new(forked()))
        .context(experiments())
        .update_context(|_, _| None);
    let err = Campaign::step(flow, &CrosstalkModel, &source, &rules())
        .finish()
        .map(|_| ())
        .unwrap_err();
    assert!(format!("{err:?}").contains("MissingContext"), "{err:?}");
}

#[test]
fn test_the_rules_refuse_a_bad_sigmas_or_drift() {
    let r = rules();
    assert_eq!(
        (r.sigmas(), r.drift(), r.objective().floor_bits),
        (3.0, 0.01, 5.0)
    );
    for (sigmas, drift) in [
        (f64::NAN, 0.0),
        (-1.0, 0.0),
        (3.0, -0.1),
        (3.0, f64::INFINITY),
    ] {
        assert!(matches!(
            CampaignRules::new(MinCostCover::new(5.0), sigmas, drift)
                .unwrap_err()
                .0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
    assert!(CampaignRules::new(MinCostCover::new(5.0), 0.0, 0.0).is_ok());
}

#[test]
fn test_a_campaign_survivor_matches_the_control_report() {
    let [h1, ..] = crosstalk_candidates().unwrap();
    let c = run(&h1, experiments());
    let name = survivor(&c).to_string();
    let report = c.finalize().unwrap();
    match report.adjudication.unwrap().outcome {
        Either::Left(s) => assert_eq!(s.name, name),
        other => panic!("expected a survivor, got {other:?}"),
    }
}
