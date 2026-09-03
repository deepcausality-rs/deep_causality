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
    CheckItem, CheckVerdict, DEFAULT_MAX_HYPOTHESES, Experiment, MinCostCover, QuantumErrorEnum,
    design, separation_bits,
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
