/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `adjudicate`: the verdict-law fold, and the survivor against the residual ambiguity;
//! `adjudicate_campaign`: the same fold over worlds that read several experiments.

use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Ambiguity, CampaignWorld, CheckItem, CheckVerdict, CountHistogram, Projection,
    QuantumErrorEnum, ShotEstimate, World, adjudicate, adjudicate_campaign,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[Complex::new(a, 0.), Complex::new(b, 0.)], &[2])
}

fn estimate(ones: u64, total: u64) -> ShotEstimate<f64> {
    let mut h = CountHistogram::new(1).unwrap();
    h.record_n(1, ones).unwrap();
    h.record_n(0, total - ones).unwrap();
    ShotEstimate::of_outcome(&h, 1).unwrap()
}

/// A read-out world: accepted when the estimate reaches `spec`.
fn read_out_world(name: &str, ones: u64, total: u64, spec: f64) -> World<f64, 2> {
    let e = estimate(ones, total);
    World::read_out(name, e.at_least(spec), e)
}

// ---------------------------------------------------------------------------
// The projection path: commutation first.
// ---------------------------------------------------------------------------

#[test]
fn test_non_commuting_verdicts_fold_to_ambiguous_and_declare_no_survivor() {
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let z0 = Projection::<f64, 2>::from_ket(&ket(1., 0.)).unwrap();
    let plus = Projection::<f64, 2>::from_ket(&ket(r, r)).unwrap();
    let worlds = [
        World::projection("h0", z0, estimate(900, 1024)),
        World::projection("h+", plus, estimate(100, 1024)),
    ];
    let a = adjudicate(&worlds, 5.0).unwrap();
    assert_eq!(a.worlds_folded, 2);
    assert_eq!(a.commutation_pairs_tested, 1);
    assert!(a.fold.is_none());
    match a.outcome {
        Either::Right(Ambiguity::NonCommuting { pair, pairs_tested }) => {
            assert_eq!(pair, (0, 1));
            assert_eq!(pairs_tested, 1);
        }
        other => panic!("expected NonCommuting, got {other:?}"),
    }
    // The separation report is measured before the commutation test, so it is the real one.
    assert_eq!(a.report.examined(), 1);
    assert_eq!(a.report.verdict(), CheckVerdict::Accepted);
    let record = &a.report.checks()[0];
    assert_eq!(record.item, CheckItem::Pair(0, 1));
    let sep = estimate(900, 1024).separation_bits(&estimate(100, 1024));
    assert_eq!(record.measured, sep);
    assert_eq!(record.threshold, 5.0);
}

#[test]
fn test_a_floor_that_is_not_a_finite_non_negative_number_is_refused_first() {
    let worlds = [
        read_out_world("a", 1023, 1024, 0.999),
        read_out_world("b", 500, 1024, 0.999),
    ];
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        match adjudicate(&worlds, bad).unwrap_err().0 {
            QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("floor"), "{msg}"),
            other => panic!("expected CalculationError, got {other:?}"),
        }
    }
    // Before anything else: an empty fork at a bad floor names the floor, not the emptiness.
    let empty: [World<f64, 2>; 0] = [];
    match adjudicate(&empty, f64::NAN).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("floor"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
    // A zero floor is a finite, non-negative number and runs.
    assert!(adjudicate(&worlds, 0.0).is_ok());
}

#[test]
fn test_a_commuting_fold_answers_through_meet_and_join() {
    // |0⟩⟨0| and 0 commute; only the first holds. The fold's meet is bottom and its join is |0⟩⟨0|.
    let z0 = Projection::<f64, 2>::from_ket(&ket(1., 0.)).unwrap();
    let worlds = [
        World::projection("h0", z0.clone(), estimate(900, 1024)),
        World::projection("none", Projection::zero(), estimate(100, 1024)),
    ];
    let a = adjudicate(&worlds, 5.0).unwrap();
    assert_eq!(a.commutation_pairs_tested, 1);
    let fold = a.fold.as_ref().expect("a commuting family folds");
    assert_eq!(fold.meet.rank(), 0);
    assert!(fold.join.leq(&z0) && z0.leq(&fold.join));
    match &a.outcome {
        Either::Left(s) => {
            assert_eq!(s.name, "h0");
            assert!(s.separation_bits > 5.0);
        }
        other => panic!("expected a survivor, got {other:?}"),
    }
    assert_eq!(a.report.examined(), 1);
    assert_eq!(a.report.verdict(), CheckVerdict::Accepted);
}

// ---------------------------------------------------------------------------
// The read-out path: no commutation test.
// ---------------------------------------------------------------------------

#[test]
fn test_a_real_valued_spec_fold_runs_no_commutation_test() {
    // Three worlds gated on at_least(0.999): one reaches it, two do not, and the survivor is well
    // separated from both. No commutation test runs and the guard produces no Ambiguous.
    let worlds = [
        read_out_world("amplitude", 1023, 1024, 0.999),
        read_out_world("detuning", 900, 1024, 0.999),
        read_out_world("decoherence", 700, 1024, 0.999),
    ];
    let a = adjudicate(&worlds, 5.0).unwrap();
    assert_eq!(a.worlds_folded, 3);
    assert_eq!(a.commutation_pairs_tested, 0);
    assert!(a.fold.is_none());
    assert_eq!(a.report.examined(), 3);
    match &a.outcome {
        Either::Left(s) => assert_eq!(s.name, "amplitude"),
        other => panic!("expected a survivor, got {other:?}"),
    }
}

#[test]
fn test_candidates_that_overlap_within_shot_noise_stay_unseparated() {
    // The survivor's nearest rival differs by a few shots in a thousand: below the floor.
    let worlds = [
        read_out_world("a", 1023, 1024, 0.999),
        read_out_world("b", 1020, 1024, 0.999),
        read_out_world("c", 500, 1024, 0.999),
    ];
    let a = adjudicate(&worlds, 5.0).unwrap();
    match &a.outcome {
        Either::Right(Ambiguity::Unseparated {
            survivor,
            tightest,
            separation_bits,
            floor_bits,
        }) => {
            assert_eq!(survivor, "a");
            assert_eq!(*tightest, (0, 1));
            assert!(*separation_bits < *floor_bits);
            assert_eq!(*floor_bits, 5.0);
        }
        other => panic!("expected Unseparated, got {other:?}"),
    }
    // The largest point estimate is not named as the survivor by default.
    assert!(matches!(a.outcome, Either::Right(_)));
}

#[test]
fn test_no_survivor_and_several_survivors_are_named() {
    let none = [
        read_out_world("a", 900, 1024, 0.999),
        read_out_world("b", 800, 1024, 0.999),
    ];
    assert!(matches!(
        adjudicate(&none, 5.0).unwrap().outcome,
        Either::Right(Ambiguity::NoSurvivor { worlds: 2 })
    ));
    let several = [
        read_out_world("a", 1024, 1024, 0.5),
        read_out_world("b", 1000, 1024, 0.5),
    ];
    match adjudicate(&several, 5.0).unwrap().outcome {
        Either::Right(Ambiguity::SeveralSurvive { survivors }) => {
            assert_eq!(survivors, vec!["a".to_string(), "b".to_string()])
        }
        other => panic!("expected SeveralSurvive, got {other:?}"),
    }
}

#[test]
fn test_a_fold_over_one_world_is_visible_as_vacuous() {
    let one = [read_out_world("only", 1023, 1024, 0.999)];
    let a = adjudicate(&one, 5.0).unwrap();
    assert_eq!(a.worlds_folded, 1);
    assert_eq!(a.report.examined(), 0);
    assert_eq!(a.report.verdict(), CheckVerdict::Vacuous);
    assert!(matches!(
        a.outcome,
        Either::Right(Ambiguity::Vacuous { worlds: 1 })
    ));
}

#[test]
fn test_mixed_kinds_and_no_worlds_are_refused() {
    let z0 = Projection::<f64, 2>::from_ket(&ket(1., 0.)).unwrap();
    let mixed = [
        World::projection("p", z0, estimate(900, 1024)),
        read_out_world("r", 900, 1024, 0.5),
    ];
    assert!(adjudicate(&mixed, 5.0).is_err());
    let empty: [World<f64, 2>; 0] = [];
    assert!(adjudicate(&empty, 5.0).is_err());
}

#[test]
fn test_verdicts_reach_adjudicate_only_from_the_measurement_boundary() {
    // The two constructors are the only way in: one takes a Projection, the other a CheckReport.
    // Neither takes an operator, so a world carrying one has nothing to fold.
    let z0 = Projection::<f64, 2>::from_ket(&ket(1., 0.)).unwrap();
    let w = World::projection("p", z0, estimate(1, 2));
    assert!(matches!(
        w.verdict(),
        deep_causality_quantum::WorldVerdict::Projection(_)
    ));
    let r = read_out_world("r", 1, 2, 0.5);
    assert!(matches!(
        r.verdict(),
        deep_causality_quantum::WorldVerdict::ReadOut(_)
    ));
    assert_eq!(w.estimate().shots(), 2);
}

// ---------------------------------------------------------------------------
// Campaigns: one reading per observed experiment.
// ---------------------------------------------------------------------------

/// A campaign world with one `(ones, total, spec)` reading per experiment.
fn campaign_world(name: &str, readings: &[(u64, u64, f64)]) -> CampaignWorld<f64> {
    CampaignWorld::new(
        name,
        readings
            .iter()
            .map(|&(ones, total, spec)| {
                let e = estimate(ones, total);
                (e.at_least(spec), e)
            })
            .collect(),
    )
}

fn measured(report: &deep_causality_quantum::CheckReport<f64>, pair: (usize, usize)) -> f64 {
    report
        .checks()
        .iter()
        .find(|c| c.item == CheckItem::Pair(pair.0, pair.1))
        .unwrap()
        .measured
}

#[test]
fn test_a_campaign_pair_separates_by_the_sum_of_its_experiments() {
    // Three worlds, three experiments; each world's k-th reading is the k-th experiment's.
    let readings: [[(u64, u64, f64); 3]; 3] = [
        [(300, 1024, 0.0), (90, 512, 0.0), (700, 2048, 0.0)],
        [(250, 1024, 0.0), (140, 512, 0.0), (650, 2048, 0.0)],
        [(320, 1024, 0.0), (60, 512, 0.0), (900, 2048, 0.0)],
    ];
    let worlds: Vec<CampaignWorld<f64>> = readings
        .iter()
        .enumerate()
        .map(|(i, r)| campaign_world(&format!("h{i}"), r))
        .collect();
    let campaign = adjudicate_campaign::<f64, 2>(&worlds, 5.0).unwrap();
    for pair in [(0, 1), (0, 2), (1, 2)] {
        // Each experiment adjudicated alone, then summed in campaign order.
        let summed = (0..3).fold(0.0, |bits, k| {
            let alone: Vec<World<f64, 2>> = readings
                .iter()
                .enumerate()
                .map(|(i, r)| read_out_world(&format!("h{i}"), r[k].0, r[k].1, r[k].2))
                .collect();
            bits + measured(&adjudicate(&alone, 5.0).unwrap().report, pair)
        });
        assert_eq!(measured(&campaign.report, pair), summed, "{pair:?}");
    }
    assert_eq!(campaign.worlds_folded, 3);
    assert_eq!(campaign.commutation_pairs_tested, 0);
    assert!(campaign.fold.is_none());
}

#[test]
fn test_a_one_experiment_campaign_adjudicates_as_adjudicate() {
    // A survivor, an unseparated survivor, several survivors, none, and a lone world.
    let cases: [&[(u64, u64, f64)]; 5] = [
        &[(950, 1024, 0.9), (100, 1024, 0.9), (500, 1024, 0.9)],
        &[(950, 1024, 0.9), (930, 1024, 0.95)],
        &[(950, 1024, 0.9), (960, 1024, 0.9)],
        &[(100, 1024, 0.9), (200, 1024, 0.9)],
        &[(950, 1024, 0.9)],
    ];
    for case in cases {
        let worlds: Vec<World<f64, 2>> = case
            .iter()
            .enumerate()
            .map(|(i, &(o, t, s))| read_out_world(&format!("h{i}"), o, t, s))
            .collect();
        let campaign: Vec<CampaignWorld<f64>> = case
            .iter()
            .enumerate()
            .map(|(i, &r)| campaign_world(&format!("h{i}"), &[r]))
            .collect();
        assert_eq!(
            adjudicate_campaign::<f64, 2>(&campaign, 5.0).unwrap(),
            adjudicate(&worlds, 5.0).unwrap(),
            "{case:?}"
        );
    }
}

#[test]
fn test_bits_from_two_experiments_separate_what_neither_does_alone() {
    // 0.10 against 0.05 at 512 shots is about 3.4 bits per experiment; two make about 6.9.
    let one = [(51, 512, 0.09), (26, 512, 0.09)];
    let alone: Vec<World<f64, 2>> = one
        .iter()
        .enumerate()
        .map(|(i, &(o, t, s))| read_out_world(&format!("h{i}"), o, t, s))
        .collect();
    assert!(matches!(
        adjudicate(&alone, 5.0).unwrap().outcome,
        Either::Right(Ambiguity::Unseparated { .. })
    ));
    let twice = [
        campaign_world("h0", &[one[0], one[0]]),
        campaign_world("h1", &[one[1], one[1]]),
    ];
    match adjudicate_campaign::<f64, 2>(&twice, 5.0).unwrap().outcome {
        Either::Left(s) => {
            assert_eq!(s.name, "h0");
            assert!(
                s.separation_bits > 5.0 && s.separation_bits < 8.0,
                "{}",
                s.separation_bits
            );
        }
        other => panic!("expected h0 to survive, got {other:?}"),
    }
}

#[test]
fn test_a_world_holds_only_when_every_reading_agrees() {
    // h0 meets its spec in the first experiment and misses it in the second.
    let worlds = [
        campaign_world("h0", &[(950, 1024, 0.9), (100, 1024, 0.9)]),
        campaign_world("h1", &[(100, 1024, 0.9), (100, 1024, 0.9)]),
    ];
    assert!(matches!(
        adjudicate_campaign::<f64, 2>(&worlds, 5.0).unwrap().outcome,
        Either::Right(Ambiguity::NoSurvivor { worlds: 2 })
    ));
}

#[test]
fn test_a_campaign_without_shared_readings_is_refused() {
    let calc = |r: Result<deep_causality_quantum::Adjudication<f64, 2>, _>| match r {
        Err(deep_causality_quantum::QuantumError(QuantumErrorEnum::CalculationError(msg))) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    };
    let msg = calc(adjudicate_campaign(&[], 5.0));
    assert!(msg.contains("at least one world"), "{msg}");
    let uneven = [
        campaign_world("h0", &[(950, 1024, 0.9), (950, 1024, 0.9)]),
        campaign_world("h1", &[(100, 1024, 0.9)]),
    ];
    let msg = calc(adjudicate_campaign(&uneven, 5.0));
    assert!(
        msg.contains("'h1' has 1 readings where 'h0' has 2"),
        "{msg}"
    );
    let empty = [campaign_world("h0", &[]), campaign_world("h1", &[])];
    assert!(calc(adjudicate_campaign(&empty, 5.0)).contains("'h0' has 0 readings"));
    let worlds = [campaign_world("h0", &[(950, 1024, 0.9)])];
    for floor in [f64::NAN, -1.0] {
        assert!(calc(adjudicate_campaign(&worlds, floor)).contains("floor"));
    }
    assert_eq!(worlds[0].name(), "h0");
    assert_eq!(worlds[0].readings().len(), 1);
}

#[test]
fn test_a_vacuous_read_out_holds_in_neither_fold() {
    // A report that examined nothing accepts vacuously; neither fold counts it as holding. With
    // one such world beside a holding one, both folds name the holding one as the survivor.
    let (held, far) = (estimate(950, 1024), estimate(100, 1024));
    let worlds = [
        World::<f64, 2>::read_out("held", held.at_least(0.9), held),
        World::read_out(
            "vacuous",
            deep_causality_quantum::CheckReport::vacuous(),
            far,
        ),
    ];
    let campaign = [
        CampaignWorld::new("held", vec![(held.at_least(0.9), held)]),
        CampaignWorld::new(
            "vacuous",
            vec![(deep_causality_quantum::CheckReport::vacuous(), far)],
        ),
    ];
    let single = adjudicate(&worlds, 5.0).unwrap().outcome;
    let folded = adjudicate_campaign::<f64, 2>(&campaign, 5.0)
        .unwrap()
        .outcome;
    for outcome in [single, folded] {
        match outcome {
            Either::Left(s) => assert_eq!(s.name, "held"),
            other => panic!("expected the holding world to survive, got {other:?}"),
        }
    }
}
