/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::brcd::brcd_gate::{GateConfig, LogisticGate, fit_logistic_gate};

fn fit(rows: &[Vec<f64>], y: &[bool]) -> LogisticGate<f64> {
    fit_logistic_gate(rows, y, &GateConfig::default()).unwrap()
}

#[test]
fn symmetric_two_point_fit_matches_the_closed_form() {
    // Rows x = [-1, 1], labels [0, 1], ridge = 1.0. By symmetry the optimum has
    // bias 0 and weight solving w = 2(1 − σ(w)), i.e. w ≈ 0.6749. (Hand-derived
    // from the penalized logistic objective.)
    let rows = vec![vec![-1.0], vec![1.0]];
    let y = [false, true];
    let g = fit(&rows, &y);

    assert!(g.bias().abs() < 1e-3, "bias should be ~0, got {}", g.bias());
    assert!(
        (g.weights()[0] - 0.6749).abs() < 1e-2,
        "weight should be ~0.6749, got {}",
        g.weights()[0]
    );
    // At x = 0 the gate is exactly the base rate 0.5.
    assert!((g.predict_proba(&[0.0]) - 0.5).abs() < 1e-3);
}

#[test]
fn separable_data_orders_probabilities_by_feature() {
    // A cleanly separable 1-D set: negatives on the left, positives on the right.
    let rows = vec![
        vec![-3.0],
        vec![-2.0],
        vec![-1.0],
        vec![1.0],
        vec![2.0],
        vec![3.0],
    ];
    let y = [false, false, false, true, true, true];
    let g = fit(&rows, &y);

    // Positive weight, probabilities monotone in the feature, all in (0, 1).
    assert!(g.weights()[0] > 0.0);
    let mut prev = 0.0;
    for x in [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0] {
        let p = g.predict_proba(&[x]);
        assert!(p > 0.0 && p < 1.0);
        assert!(p > prev, "probability should increase with x");
        prev = p;
    }
    // The far-left point is classified negative, the far-right positive.
    assert!(g.predict_proba(&[-3.0]) < 0.5);
    assert!(g.predict_proba(&[3.0]) > 0.5);
}

#[test]
fn two_feature_fit_is_finite_and_calibrated() {
    // y depends on the sum of two features; the gate should separate the classes.
    let rows = vec![
        vec![-2.0, -1.0],
        vec![-1.0, -1.0],
        vec![-1.0, 0.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
        vec![2.0, 1.0],
    ];
    let y = [false, false, false, true, true, true];
    let g = fit(&rows, &y);

    assert!(g.bias().is_finite());
    assert!(g.weights().iter().all(|w| w.is_finite()));
    assert!(g.predict_proba(&[-2.0, -1.0]) < 0.5);
    assert!(g.predict_proba(&[2.0, 1.0]) > 0.5);
}

#[test]
fn all_true_labels_give_a_constant_high_gate() {
    let rows = vec![vec![-1.0], vec![0.0], vec![1.0]];
    let y = [true, true, true];
    let g = fit(&rows, &y);
    assert!(g.weights().iter().all(|&w| w == 0.0));
    for x in [-5.0, 0.0, 5.0] {
        assert!(g.predict_proba(&[x]) > 0.99);
    }
}

#[test]
fn all_false_labels_give_a_constant_low_gate() {
    let rows = vec![vec![-1.0], vec![0.0], vec![1.0]];
    let y = [false, false, false];
    let g = fit(&rows, &y);
    assert!(g.weights().iter().all(|&w| w == 0.0));
    for x in [-5.0, 0.0, 5.0] {
        assert!(g.predict_proba(&[x]) < 0.01);
    }
}

#[test]
fn fit_is_deterministic() {
    let rows = vec![vec![-1.0], vec![0.5], vec![1.0], vec![2.0]];
    let y = [false, false, true, true];
    let a = fit(&rows, &y);
    let b = fit(&rows, &y);
    assert_eq!(a, b);
}

#[test]
fn empty_data_is_rejected() {
    let rows: Vec<Vec<f64>> = vec![];
    let y: [bool; 0] = [];
    assert_eq!(
        fit_logistic_gate(&rows, &y, &GateConfig::default()).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
}

#[test]
fn mismatched_label_count_is_rejected() {
    let rows = vec![vec![1.0], vec![2.0]];
    let y = [true]; // one short
    assert_eq!(
        fit_logistic_gate(&rows, &y, &GateConfig::default()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn ragged_rows_are_rejected() {
    let rows = vec![vec![1.0, 2.0], vec![3.0]]; // different widths
    let y = [true, false];
    assert_eq!(
        fit_logistic_gate(&rows, &y, &GateConfig::default()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn fits_at_f32_and_f64_agree() {
    let rows64 = vec![vec![-1.0_f64], vec![1.0]];
    let y = [false, true];
    let g64 = fit_logistic_gate(&rows64, &y, &GateConfig::default()).unwrap();

    let rows32 = vec![vec![-1.0_f32], vec![1.0]];
    let g32 = fit_logistic_gate(&rows32, &y, &GateConfig::<f32>::default()).unwrap();

    // Same closed-form target at both precisions (looser tolerance for f32).
    assert!((g64.weights()[0] - 0.6749).abs() < 1e-2);
    assert!((g32.weights()[0] - 0.6749).abs() < 1e-2);
    assert!(g32.bias().abs() < 1e-3);
}
