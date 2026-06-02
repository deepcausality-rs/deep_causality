/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::brcd::brcd_gaussian::{
    GaussianFamilyConfig, RIDGE_DEFAULT, Transform, gaussian_family_logdensity,
    gaussian_single_expert_logdensity,
};

const LN_4PI: f64 = 2.531_024_246_969_291;

fn cfg() -> GaussianFamilyConfig<f64> {
    GaussianFamilyConfig::default()
}

#[test]
fn f_absent_equals_the_single_expert() {
    // With no F, the family reduces to the stage-3 single expert.
    let node = vec![1.0, 2.0, 4.0, 3.0];
    let parents = vec![vec![0.5], vec![1.0], vec![2.0], vec![1.5]];
    let family = gaussian_family_logdensity(&node, &parents, None, false, &cfg()).unwrap();
    let single =
        gaussian_single_expert_logdensity(&node, &parents, Transform::None, RIDGE_DEFAULT).unwrap();
    for (a, b) in family.iter().zip(single.iter()) {
        assert!((a - b).abs() < 1e-12);
    }
}

#[test]
fn f_in_parents_scores_each_row_in_its_regime() {
    // F is the only parent. Regime 0 rows = {1,3} (mean 2, var 2); regime 1 rows
    // = {11,13} (mean 12, var 2). Each row is scored within its own regime, so
    // every residual is ±1 and every density is logpdf(·; μ, 2) = -½(ln4π + ½).
    let node = vec![1.0, 3.0, 11.0, 13.0];
    let f = [false, false, true, true];
    let out = gaussian_family_logdensity(&node, &[], Some(&f), true, &cfg()).unwrap();
    let expect = -0.5 * (LN_4PI + 0.5);
    for v in out {
        assert!((v - expect).abs() < 1e-6, "{v} vs {expect}");
    }
}

#[test]
fn f_in_parents_with_a_continuous_parent_is_finite_and_separated() {
    // Each regime has a linear node–parent relationship with a different offset.
    // Three rows per regime (n_g > p_g = 2) so the per-regime ridge fit runs
    // rather than the small-sample fallback; the near-perfect fit → sharp density.
    let node = vec![0.0, 2.0, 4.0, 10.0, 12.0, 14.0];
    let parents = vec![
        vec![0.0],
        vec![1.0],
        vec![2.0],
        vec![0.0],
        vec![1.0],
        vec![2.0],
    ];
    let f = [false, false, false, true, true, true];
    let out = gaussian_family_logdensity(&node, &parents, Some(&f), true, &cfg()).unwrap();
    assert!(out.iter().all(|v| v.is_finite()));
    assert!(out.iter().all(|&v| v > 0.0), "sharp linear fit: {out:?}");
}

#[test]
fn mixture_with_all_anomalous_collapses_to_the_present_expert() {
    // F not a parent, but every row is F=1: expert-0 is empty, the gate predicts
    // ≈1, so the mixture ≈ the single expert fit on all rows (F-absent path).
    let node = vec![1.0, 2.0, 4.0, 3.0, 5.0];
    let parents = vec![vec![0.5], vec![1.0], vec![2.0], vec![1.5], vec![2.5]];
    let f = [true, true, true, true, true];
    let mixture = gaussian_family_logdensity(&node, &parents, Some(&f), false, &cfg()).unwrap();
    let absent = gaussian_family_logdensity(&node, &parents, None, false, &cfg()).unwrap();
    for (a, b) in mixture.iter().zip(absent.iter()) {
        assert!((a - b).abs() < 1e-5, "{a} vs {b}");
    }
}

#[test]
fn mixture_is_finite_and_deterministic() {
    let node = vec![0.0, 1.0, 2.0, 8.0, 9.0, 10.0];
    let parents = vec![
        vec![0.0],
        vec![1.0],
        vec![2.0],
        vec![0.0],
        vec![1.0],
        vec![2.0],
    ];
    let f = [false, false, false, true, true, true];
    let a = gaussian_family_logdensity(&node, &parents, Some(&f), false, &cfg()).unwrap();
    let b = gaussian_family_logdensity(&node, &parents, Some(&f), false, &cfg()).unwrap();
    assert!(a.iter().all(|v| v.is_finite()));
    assert_eq!(a, b);
}

#[test]
fn transform_parents_changes_the_fit_when_a_transform_is_active() {
    // With node_transform=log and transform_parents, the parents are log-scaled
    // before the fit, so the densities differ from the untransformed-parent fit.
    let node = vec![1.0, 2.0, 4.0, 8.0];
    let parents = vec![vec![1.0], vec![2.0], vec![4.0], vec![8.0]];

    let mut with = cfg();
    with.transform = Transform::Log;
    with.transform_parents = true;
    let mut without = cfg();
    without.transform = Transform::Log;
    without.transform_parents = false;

    let a = gaussian_family_logdensity(&node, &parents, None, false, &with).unwrap();
    let b = gaussian_family_logdensity(&node, &parents, None, false, &without).unwrap();
    assert!(a.iter().all(|v| v.is_finite()));
    assert!(
        a.iter().zip(b.iter()).any(|(x, y)| (x - y).abs() > 1e-9),
        "transform_parents should change the result"
    );
}

#[test]
fn family_rejects_bad_shapes() {
    assert_eq!(
        gaussian_family_logdensity::<f64>(&[], &[], None, false, &cfg()).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
    // F length mismatch.
    let f = [true];
    assert_eq!(
        gaussian_family_logdensity(&[1.0, 2.0], &[], Some(&f), false, &cfg()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}
