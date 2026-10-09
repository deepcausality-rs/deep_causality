/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `design`: the exact minimum-cost cover over hypothesis pairs.
//!
//! Separation is the shot-scaled Bhattacharyya distance. At 1024 shots the pair (0.10, 0.04)
//! separates by about 10.7 bits, (0.10, 0.20) by about 15 and (0.04, 0.20) by about 51, all above
//! a floor of 5, while equal predictions separate by zero; those three levels are the alphabet
//! every instance below is written in.

use deep_causality_quantum::{
    CheckItem, CheckVerdict, CoverMode, DEFAULT_MAX_HYPOTHESES, Experiment, InstrumentTime,
    MAX_COMBINED_EXPERIMENTS, MinCostCover, QuantumError, QuantumErrorEnum, design,
    separation_bits,
};

const A: f64 = 0.10;
const B: f64 = 0.04;
const C: f64 = 0.20;
const SHOTS: u64 = 1024;

fn exp(name: &str, cost: f64, predictions: &[f64]) -> Experiment<f64> {
    Experiment::new(name, cost, SHOTS, predictions.to_vec()).unwrap()
}

/// The crosstalk instance: three survivors, two cheap interventions and a tomography.
fn crosstalk() -> Vec<Experiment<f64>> {
    vec![
        exp("do_q1", 1.0, &[A, B, B]),
        exp("do_q2", 1.0, &[B, A, B]),
        exp("echo_both", 1.0, &[A, A, A]),
        exp("process_tomography", 200.0, &[A, B, C]),
    ]
}

#[test]
fn test_the_alphabet_separates_as_the_module_doc_says() {
    assert!(separation_bits(A, B, SHOTS) > 5.0);
    assert!(separation_bits(A, C, SHOTS) > 5.0);
    assert!(separation_bits(B, C, SHOTS) > 5.0);
    assert_eq!(separation_bits(A, A, SHOTS), 0.0);
}

#[test]
fn test_the_crosstalk_case_selects_the_two_interventions_over_tomography() {
    let plan = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    let names: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["do_q1", "do_q2"], "in declared order");
    assert_eq!(plan.total_cost(), 2.0);
    assert!(plan.is_complete());
    assert_eq!(plan.entries()[0].resolves, vec![(0, 1), (0, 2)]);
    assert_eq!(plan.entries()[1].resolves, vec![(0, 1), (1, 2)]);
    assert_eq!(plan.pairs_examined(), 3);
    assert_eq!(plan.hypotheses(), 3);
    assert_eq!(plan.report().verdict(), CheckVerdict::Accepted);
}

#[test]
fn test_the_same_instance_yields_the_same_plan() {
    let a = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    let b = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    assert_eq!(a, b);
    // Two equal-cost covers: the declared order breaks the tie the same way every time.
    let tied = vec![
        exp("first", 1.0, &[A, B, C]),
        exp("second", 1.0, &[A, B, C]),
    ];
    let plan = design(3, &tied, MinCostCover::new(5.0)).unwrap();
    assert_eq!(plan.entries().len(), 1);
    assert_eq!(plan.entries()[0].name, "first");
}

#[test]
fn test_the_cover_is_optimal_rather_than_greedy() {
    // Four hypotheses, six pairs. A most-pairs-first greedy takes the five-pair experiment at
    // cost 2 and then needs (2, 3) from one of the bipartite covers: cost 3. The exact solve
    // takes the two bipartite covers at cost 1 each, whose union is all six pairs: cost 2.
    let trap = vec![
        exp("five_pairs", 2.0, &[A, B, C, C]),
        exp("k22_a", 1.0, &[A, A, B, B]),
        exp("k22_b", 1.0, &[A, B, A, B]),
        exp("process_tomography", 200.0, &[A, B, C, 0.5]),
    ];
    let plan = design(4, &trap, MinCostCover::new(5.0)).unwrap();
    let names: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["k22_a", "k22_b"]);
    assert_eq!(plan.total_cost(), 2.0);
    assert!(plan.is_complete());
    assert_eq!(plan.pairs_examined(), 6);
}

#[test]
fn test_offering_more_experiments_keeps_the_optimum() {
    // n stays at 3 while k grows from 4 to 40: the table keeps its 2^3 entries and the plan is
    // the same exact optimum.
    let mut many = crosstalk();
    for i in 0..36 {
        many.push(exp(&format!("noise_{i}"), 0.5, &[A, A, A]));
    }
    assert_eq!(many.len(), 40);
    let plan = design(3, &many, MinCostCover::new(5.0)).unwrap();
    let names: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["do_q1", "do_q2"]);
    assert_eq!(plan.total_cost(), 2.0);
}

#[test]
fn test_too_many_hypotheses_fails_loudly_before_the_table() {
    let ten = vec![exp("e", 1.0, &[A; 10])];
    let err = design(10, &ten, MinCostCover::new(5.0)).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::HypothesisCountExceeded { n: 10, pairs: 45 }
    ));
    assert_eq!(DEFAULT_MAX_HYPOTHESES, 7);
}

#[test]
fn test_the_pair_count_is_bounded_before_the_pairs_are_allocated() {
    // `C(usize::MAX, 2)` overflows: the count is checked first and reads as above any cap, so
    // no pair list is built.
    let none: Vec<Experiment<f64>> = vec![];
    match design(usize::MAX, &none, MinCostCover::new(5.0))
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::HypothesisCountExceeded { n, pairs } => {
            assert_eq!(n, usize::MAX);
            assert_eq!(pairs, usize::MAX);
        }
        other => panic!("expected HypothesisCountExceeded, got {other:?}"),
    }
    // A count that fits a usize but not the pair mask is refused with its exact pair count,
    // under a cap raised above it, and still before the pairs are allocated: at 2^20
    // hypotheses that list would hold about 5.5e11 entries.
    let big = 1usize << 20;
    let raised = MinCostCover::new(5.0).with_max_hypotheses(usize::MAX);
    match design(big, &none, raised).unwrap_err().0 {
        QuantumErrorEnum::HypothesisCountExceeded { n, pairs } => {
            assert_eq!(n, big);
            assert_eq!(
                pairs,
                big.checked_mul(big - 1)
                    .map(|twice| twice / 2)
                    .unwrap_or(usize::MAX)
            );
        }
        other => panic!("expected HypothesisCountExceeded, got {other:?}"),
    }
}

#[test]
fn test_a_floor_that_is_not_a_finite_non_negative_number_is_refused() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        match design(3, &crosstalk(), MinCostCover::new(bad))
            .unwrap_err()
            .0
        {
            QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("floor"), "{msg}"),
            other => panic!("expected CalculationError, got {other:?}"),
        }
    }
    // A zero floor is a finite, non-negative number: every pair with any separation is covered.
    let plan = design(3, &crosstalk(), MinCostCover::new(0.0)).unwrap();
    assert!(plan.is_complete());
}

#[test]
fn test_the_cap_is_a_parameter() {
    // Seven hypotheses run at the default cap and are refused at a cap of six.
    let seven: Vec<f64> = vec![A, B, C, A, B, C, 0.5];
    let e = vec![exp("e", 1.0, &seven)];
    assert!(design(7, &e, MinCostCover::new(5.0)).is_ok());
    let err = design(7, &e, MinCostCover::new(5.0).with_max_hypotheses(6)).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::HypothesisCountExceeded { n: 7, pairs: 21 }
    ));
}

#[test]
fn test_a_plan_that_discriminates_nothing_says_so() {
    let useless = vec![exp("echo", 1.0, &[A, A, A]), exp("idle", 3.0, &[B, B, B])];
    let plan = design(3, &useless, MinCostCover::new(5.0)).unwrap();
    assert!(plan.entries().is_empty());
    assert_eq!(plan.total_cost(), 0.0);
    assert_eq!(plan.uncovered(), &[(0, 1), (0, 2), (1, 2)]);
    assert_eq!(plan.pairs_examined(), 3);
    assert_eq!(plan.report().verdict(), CheckVerdict::Rejected);
}

#[test]
fn test_a_partial_cover_is_returned_with_its_gap() {
    // Four hypotheses; nothing offered separates (2, 3). The other five pairs cover at cost 4.
    let partial = vec![
        exp("k22_a", 2.0, &[A, A, B, B]),
        exp("star_0", 2.0, &[A, B, B, B]),
    ];
    let plan = design(4, &partial, MinCostCover::new(5.0)).unwrap();
    assert_eq!(plan.total_cost(), 4.0);
    assert_eq!(plan.uncovered(), &[(2, 3)]);
    assert!(!plan.is_complete());
    let rejected = plan.report().first_rejection().unwrap();
    assert_eq!(rejected.item, CheckItem::Pair(2, 3));
}

#[test]
fn test_design_reports_the_pair_closest_to_the_floor() {
    // Three pairs covered; the tightest separates by ~10.7 bits against a floor of 5.
    let plan = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    let worst = plan.report().worst().unwrap();
    assert_eq!(plan.report().examined(), 3);
    assert!((worst.measured - separation_bits(A, B, SHOTS)).abs() < 1e-9);
    assert_eq!(worst.threshold, 5.0);
    assert!(worst.margin < 1.0 && worst.margin > 0.0);
    assert!(worst.accepted);
}

#[test]
fn test_the_experiment_budget_is_drawn_in_checked_arithmetic() {
    let plan = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    assert_eq!(plan.experiment_count(), 2);
    assert_eq!(plan.draw_experiments(5u64).unwrap(), 3);
    assert_eq!(plan.draw_experiments(2u64).unwrap(), 0);
    match plan.draw_experiments(1u64).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("shortfall 1"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_malformed_inputs_are_refused() {
    assert!(Experiment::<f64>::new("neg", -1.0, 10, vec![0.5]).is_err());
    assert!(Experiment::<f64>::new("nan", f64::NAN, 10, vec![0.5]).is_err());
    assert!(Experiment::<f64>::new("zero_shots", 1.0, 0, vec![0.5]).is_err());
    assert!(Experiment::<f64>::new("not_prob", 1.0, 10, vec![1.5]).is_err());
    assert!(design(1, &crosstalk(), MinCostCover::new(5.0)).is_err());
    let wrong_width = vec![exp("w", 1.0, &[A, B])];
    assert!(matches!(
        design(3, &wrong_width, MinCostCover::new(5.0))
            .unwrap_err()
            .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

// ---------------------------------------------------------------------------
// The shots an entry takes, and combining bits across experiments.
// ---------------------------------------------------------------------------

#[test]
fn test_a_fixed_plan_takes_each_experiment_at_its_own_shots() {
    let plan = design(3, &crosstalk(), MinCostCover::new(5.0)).unwrap();
    assert!(plan.entries().iter().all(|e| e.shots == SHOTS));
}

/// Two experiments that each separate the pair (0.10, 0.05) by about 3.4 bits at 512 shots, and
/// a dearer one that reaches the floor alone.
fn halves() -> Vec<Experiment<f64>> {
    let at = |name: &str, cost: f64, shots: u64, p: [f64; 2]| {
        Experiment::new(name, cost, shots, p.to_vec()).unwrap()
    };
    vec![
        at("first half", 1.0, 512, [0.10, 0.05]),
        at("second half", 1.0, 512, [0.10, 0.05]),
        at("whole", 3.0, 1024, [0.10, 0.05]),
    ]
}

#[test]
fn test_combining_adds_a_pairs_bits_across_experiments() {
    let half = separation_bits(0.10, 0.05, 512);
    assert!(half < 5.0 && 2.0 * half > 5.0, "{half}");
    // Alone, only the dear experiment covers the pair.
    let alone = design(2, &halves(), MinCostCover::new(5.0)).unwrap();
    let names: Vec<&str> = alone.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!((names, alone.total_cost()), (vec!["whole"], 3.0));
    // Combined, the two halves cover it for less, and neither covers it alone.
    let combined = design(2, &halves(), MinCostCover::new(5.0).combining()).unwrap();
    let names: Vec<&str> = combined.entries().iter().map(|e| e.name.as_str()).collect();
    assert_eq!(
        (names, combined.total_cost()),
        (vec!["first half", "second half"], 2.0)
    );
    assert!(combined.entries().iter().all(|e| e.resolves.is_empty()));
    assert!(combined.is_complete());
    assert_eq!(combined.report().checks()[0].measured, 0.0 + half + half);
}

#[test]
fn test_a_combining_plan_reports_a_pair_all_experiments_cannot_reach() {
    let short = |name: &str| Experiment::new(name, 1.0, 64, vec![0.10, 0.05]).unwrap();
    let plan = design(
        2,
        &[short("a"), short("b")],
        MinCostCover::new(5.0).combining(),
    )
    .unwrap();
    assert_eq!(plan.uncovered(), &[(0, 1)]);
    assert!(plan.entries().is_empty());
    assert_eq!(plan.total_cost(), 0.0);
}

fn calculation(e: QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_combining_refuses_too_many_experiments() {
    let many: Vec<Experiment<f64>> = (0..=MAX_COMBINED_EXPERIMENTS)
        .map(|i| Experiment::new(format!("e{i}"), 1.0, 64, vec![0.1, 0.2]).unwrap())
        .collect();
    let msg = calculation(design(2, &many, MinCostCover::new(5.0).combining()).unwrap_err());
    assert!(msg.contains("17 exceed the cap of 16"), "{msg}");
    // At the cap it runs.
    assert!(design(2, &many[1..], MinCostCover::new(5.0).combining()).is_ok());
}

#[test]
fn test_an_objective_has_exactly_one_mode_and_the_last_builder_sets_it() {
    let time = InstrumentTime::new(0.01, 100.0).unwrap();
    assert_eq!(MinCostCover::new(5.0).mode, CoverMode::Fixed);
    assert_eq!(
        MinCostCover::new(5.0).combining().mode,
        CoverMode::Combining
    );
    assert_eq!(
        MinCostCover::new(5.0).combining().timed(time).mode,
        CoverMode::Timed(time)
    );
    assert_eq!(
        MinCostCover::new(5.0).timed(time).combining().mode,
        CoverMode::Combining
    );
    // Whichever mode the chain ends in is the one `design` solves.
    let timed_last = design(2, &halves(), MinCostCover::new(5.0).combining().timed(time)).unwrap();
    let timed = design(2, &halves(), MinCostCover::new(5.0).timed(time)).unwrap();
    assert_eq!(timed_last, timed);
    let combined_last =
        design(2, &halves(), MinCostCover::new(5.0).timed(time).combining()).unwrap();
    let combined = design(2, &halves(), MinCostCover::new(5.0).combining()).unwrap();
    assert_eq!(combined_last, combined);
    assert_ne!(timed, combined);
}

// ---------------------------------------------------------------------------
// Priced in time: each experiment sized to the floor, within the white-noise range.
// ---------------------------------------------------------------------------

/// Setup time `setup` and predictions `p`; the shots are what timing sizes.
fn timed_exp(name: &str, setup: f64, p: &[f64]) -> Experiment<f64> {
    Experiment::new(name, setup, 1, p.to_vec()).unwrap()
}

#[test]
fn test_a_timed_entry_takes_the_fewest_shots_that_reach_the_floor() {
    let time = InstrumentTime::new(0.01, 1000.0).unwrap();
    for (p, q) in [(0.10, 0.20), (0.5, 0.498), (0.04, 0.10), (0.3, 0.7)] {
        let plan = design(
            2,
            &[timed_exp("e", 30.0, &[p, q])],
            MinCostCover::new(5.0).timed(time),
        )
        .unwrap();
        if separation_bits(p, q, time.max_shots()) < 5.0 {
            assert_eq!(plan.uncovered(), &[(0, 1)], "{p} {q}");
            continue;
        }
        let entry = &plan.entries()[0];
        let n = entry.shots;
        // The sizing law: n reaches the floor and n − 1 does not.
        assert!(
            separation_bits(p, q, n) >= 5.0 * (1.0 - 1e-12),
            "{p} {q} {n}"
        );
        assert!(separation_bits(p, q, n - 1) < 5.0, "{p} {q} {n}");
        assert_eq!(entry.cost, 30.0 + n as f64 * 0.01);
        assert_eq!(plan.total_cost(), entry.cost);
        assert_eq!(entry.resolves, vec![(0, 1)]);
    }
}

#[test]
fn test_a_pair_that_needs_more_than_the_white_noise_range_is_uncovered() {
    // 0.5 against 0.498 needs about 1.7 million shots; at 0.01 s a shot, a range of 100 s holds
    // 10 000. The report states how far the most the range allows gets.
    let time = InstrumentTime::new(0.01, 100.0).unwrap();
    let plan = design(
        3,
        &[timed_exp("e", 0.0, &[0.5, 0.498, 0.2])],
        MinCostCover::new(5.0).timed(time),
    )
    .unwrap();
    assert_eq!(plan.uncovered(), &[(0, 1)]);
    let entry = &plan.entries()[0];
    assert_eq!(entry.resolves, vec![(0, 2), (1, 2)]);
    let reported = plan.report().checks()[0];
    assert_eq!(reported.measured, separation_bits(0.5, 0.498, 10_000));
    assert!(!reported.accepted);
}

#[test]
fn test_a_timed_experiment_is_sized_to_the_hardest_pair_it_covers() {
    // One experiment, three pairs: it takes the shots its hardest pair needs.
    let time = InstrumentTime::new(0.001, 1000.0).unwrap();
    let p = [0.10, 0.20, 0.50];
    let plan = design(
        3,
        &[timed_exp("e", 5.0, &p)],
        MinCostCover::new(5.0).timed(time),
    )
    .unwrap();
    assert!(plan.is_complete());
    let n = plan.entries()[0].shots;
    let hardest = separation_bits(p[0], p[1], n);
    assert!(hardest >= 5.0 * (1.0 - 1e-12) && separation_bits(p[0], p[1], n - 1) < 5.0);
    assert_eq!(plan.experiment_count(), 1);
}

#[test]
fn test_a_timed_plan_weighs_setup_against_integration() {
    // A quick setup on a weak contrast against a slow setup on a strong one.
    let time = InstrumentTime::new(0.01, 10_000.0).unwrap();
    let weak = timed_exp("weak, no setup", 0.0, &[0.10, 0.12]);
    let strong = timed_exp("strong, 60 s setup", 60.0, &[0.10, 0.30]);
    let plan = design(
        2,
        &[weak.clone(), strong.clone()],
        MinCostCover::new(5.0).timed(time),
    )
    .unwrap();
    let (n_weak, n_strong) = (
        design(2, &[weak], MinCostCover::new(5.0).timed(time))
            .unwrap()
            .entries()[0]
            .shots,
        design(2, &[strong], MinCostCover::new(5.0).timed(time))
            .unwrap()
            .entries()[0]
            .shots,
    );
    let (cost_weak, cost_strong) = (n_weak as f64 * 0.01, 60.0 + n_strong as f64 * 0.01);
    let want = if cost_weak < cost_strong {
        "weak, no setup"
    } else {
        "strong, 60 s setup"
    };
    assert_eq!(plan.entries()[0].name, want);
    assert_eq!(plan.total_cost(), cost_weak.min(cost_strong));
}

#[test]
fn test_a_timed_pair_one_shot_resolves_takes_one_shot() {
    // Read-outs at 0 and 1 separate by infinitely many bits per shot, and a floor within the
    // slack is reached by any pair: both take one shot, as the fixed plan at one shot covers them.
    let time = InstrumentTime::new(0.01, 100.0).unwrap();
    for (predictions, floor) in [
        ([0.0, 1.0], 5.0),
        ([1.0, 0.0], 5.0),
        ([0.0, 1.0], 0.0),
        ([0.5, 0.500_000_1], 0.0),
        ([0.3, 0.3], 0.0),
    ] {
        let one_shot = Experiment::new("e", 2.0, 1, predictions.to_vec()).unwrap();
        let fixed = design(2, std::slice::from_ref(&one_shot), MinCostCover::new(floor)).unwrap();
        assert!(fixed.is_complete(), "{predictions:?} at {floor}");
        let timed = design(2, &[one_shot], MinCostCover::new(floor).timed(time)).unwrap();
        assert!(timed.is_complete(), "{predictions:?} at {floor}");
        let entry = &timed.entries()[0];
        assert_eq!(entry.shots, 1, "{predictions:?} at {floor}");
        assert_eq!(entry.cost, 2.0 + 0.01);
        assert_eq!(entry.resolves, vec![(0, 1)]);
    }
}
