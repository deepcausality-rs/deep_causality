/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `adjudicate`: the verdict-law fold, and the survivor against the residual ambiguity.

use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Ambiguity, CheckItem, CheckVerdict, CountHistogram, Projection, QuantumErrorEnum, ShotEstimate,
    World, adjudicate,
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
