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
#[cfg_attr(miri, ignore)]
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

#[test]
fn family_rejects_parent_count_mismatch() {
    // parents present but `parents.len() != n` → DimensionMismatch (the parent
    // shape guard inside `gaussian_family_logdensity`).
    let node = vec![1.0, 2.0, 3.0];
    let parents = vec![vec![0.5]]; // one row, three node values
    assert_eq!(
        gaussian_family_logdensity(&node, &parents, None, false, &cfg()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn family_rejects_ragged_parent_rows() {
    // parents.len() == n but a row is the wrong width → DimensionMismatch.
    let node = vec![1.0, 2.0];
    let parents = vec![vec![0.5, 0.5], vec![1.0]];
    assert_eq!(
        gaussian_family_logdensity(&node, &parents, None, false, &cfg()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn f_in_parents_skips_an_empty_regime() {
    // F is the only parent and every row is regime 1: the regime-0 index set is
    // empty, so its per-regime fit is skipped (`continue`). The result is finite
    // and every row is scored within regime 1.
    let node = vec![10.0, 11.0, 12.0, 13.0];
    let f = [true, true, true, true];
    let out = gaussian_family_logdensity(&node, &[], Some(&f), true, &cfg()).unwrap();
    assert_eq!(out.len(), 4);
    assert!(out.iter().all(|v| v.is_finite()), "{out:?}");
}

#[test]
fn f_in_parents_single_row_regime_uses_unit_variance() {
    // Regime 0 has a single row (variance with <2 values falls back to 1.0); the
    // node has no continuous parents, so the constant-mean expert is used.
    let node = vec![5.0, 0.0, 1.0, 2.0];
    let f = [false, true, true, true];
    let out = gaussian_family_logdensity(&node, &[], Some(&f), true, &cfg()).unwrap();
    assert_eq!(out.len(), 4);
    assert!(out.iter().all(|v| v.is_finite()), "{out:?}");
    // The lone regime-0 row (index 0) sits exactly at its own mean (residual 0),
    // scored with the unit-variance fallback: logpdf(0; 0, 1) = -0.5 ln(2π).
    let expect = -0.5 * (2.0 * std::f64::consts::PI).ln();
    assert!((out[0] - expect).abs() < 1e-9, "{} vs {expect}", out[0]);
}

#[test]
fn f_in_parents_constant_regime_hits_the_variance_floor() {
    // Regime 1's node values are all identical → its sample variance is 0, so the
    // density variance falls back to the 1e-12 floor (the `density_variance` else
    // branch). The constant rows have residual 0 under their own mean → a large
    // finite log-density (≈ -0.5 ln(2π·1e-12)).
    let node = vec![1.0, 3.0, 7.0, 7.0, 7.0];
    let f = [false, false, true, true, true];
    let out = gaussian_family_logdensity(&node, &[], Some(&f), true, &cfg()).unwrap();
    assert!(out.iter().all(|v| v.is_finite()), "{out:?}");
    // Floored-variance density at a zero residual is large and positive.
    assert!(
        out[2] > 10.0,
        "constant regime should score sharply: {}",
        out[2]
    );
}

#[test]
fn f_absent_with_all_nonfinite_parents_falls_back_to_const_expert() {
    // F absent, parents present but every parent row is non-finite: the streaming
    // ridge fit sees no finite row and returns None, so `fit_expert` falls back to
    // the constant sample-mean expert. The result equals a parentless single
    // expert over the node.
    let node = vec![1.0, 2.0, 4.0, 3.0];
    let parents = vec![
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![f64::NAN],
        vec![f64::NEG_INFINITY],
    ];
    let out = gaussian_family_logdensity(&node, &parents, None, false, &cfg()).unwrap();
    let no_parents = gaussian_family_logdensity(&node, &[], None, false, &cfg()).unwrap();
    for (a, b) in out.iter().zip(no_parents.iter()) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }
}

#[test]
fn f_in_parents_too_few_finite_rows_uses_the_guarded_fallback() {
    // F is a parent with a continuous parent (p = 1, so the guard is n ≤ p+1 = 2).
    // Regime 1 has exactly two finite rows → too few for the ridge design, so
    // `fit_expert_guarded` falls back to the regime's sample mean/variance instead
    // of a ridge fit. Regime 0 has enough rows to fit normally.
    let node = vec![0.0, 1.0, 2.0, 3.0, 20.0, 21.0];
    let parents = vec![
        vec![0.0],
        vec![1.0],
        vec![2.0],
        vec![3.0],
        vec![0.0],
        vec![1.0],
    ];
    let f = [false, false, false, false, true, true];
    let out = gaussian_family_logdensity(&node, &parents, Some(&f), true, &cfg()).unwrap();
    assert_eq!(out.len(), 6);
    assert!(out.iter().all(|v| v.is_finite()), "{out:?}");
}

#[test]
fn mixture_with_nonfinite_parent_falls_back_to_the_prior_gate() {
    // F is not a parent (mixture branch) and a parent feature is non-finite, so the
    // logistic-gate Newton fit diverges to a non-finite parameter (SingularSystem).
    // `gate_probabilities` then falls back to the empirical base rate, and the
    // mixture is still produced for the finite rows.
    let node = vec![0.0, 1.0, 2.0, 8.0, 9.0, 10.0];
    let parents = vec![
        vec![0.0],
        vec![1.0],
        vec![f64::NAN],
        vec![0.0],
        vec![1.0],
        vec![2.0],
    ];
    let f = [false, false, false, true, true, true];
    let out = gaussian_family_logdensity(&node, &parents, Some(&f), false, &cfg()).unwrap();
    assert_eq!(out.len(), 6);
    // The prior-gate fallback must keep the gate mixture defined (no panic); the
    // NaN parent row may be non-finite, but the all-finite rows must score finite.
    assert!(out[0].is_finite() && out[5].is_finite(), "{out:?}");
}
