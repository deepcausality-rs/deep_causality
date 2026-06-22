/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::brcd::brcd_gaussian::{
    RIDGE_DEFAULT, Transform, effective_transform, fit_ridge, gaussian_single_expert_logdensity,
    transform_and_jacobian,
};

const LN_2PI: f64 = 1.837_877_066_409_345_6;

fn ridge() -> f64 {
    RIDGE_DEFAULT
}

// --- fit_ridge --------------------------------------------------------------

#[test]
fn fit_ridge_recovers_a_clean_line() {
    // y = 2 + 3x exactly; with λ = 1e-4 the fit is ~[2, 3] and σ² is floored.
    let x = vec![
        vec![1.0, 0.0],
        vec![1.0, 1.0],
        vec![1.0, 2.0],
        vec![1.0, 3.0],
    ];
    let y = vec![2.0, 5.0, 8.0, 11.0];
    let fit = fit_ridge(&x, &y, ridge()).unwrap();
    assert!(
        (fit.beta[0] - 2.0).abs() < 1e-2,
        "intercept {}",
        fit.beta[0]
    );
    assert!((fit.beta[1] - 3.0).abs() < 1e-2, "slope {}", fit.beta[1]);
    assert!(fit.sigma2 > 0.0 && fit.sigma2 < 1e-4);
    // predict matches the line.
    assert!((fit.predict(&[1.0, 4.0]) - 14.0).abs() < 1e-1);
}

#[test]
fn fit_ridge_variance_is_floored_on_a_perfect_fit() {
    // Two points, two params, no ridge → exact fit, zero residual → σ² hits the
    // 1e-12 floor. (With λ > 0 the shrunk β leaves a small non-zero residual, so
    // the floor is exercised here with λ = 0 on a consistent system.)
    let x = vec![vec![1.0_f64, 0.0], vec![1.0, 1.0]];
    let y = vec![1.0_f64, 2.0];
    let fit = fit_ridge(&x, &y, 0.0).unwrap();
    assert!((fit.sigma2 - 1e-12).abs() < 1e-15);
}

#[test]
fn fit_ridge_rejects_bad_shapes() {
    assert_eq!(
        fit_ridge::<f64>(&[], &[], ridge()).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
    assert_eq!(
        fit_ridge(&[vec![1.0, 0.0]], &[1.0, 2.0], ridge()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
    assert_eq!(
        fit_ridge(&[vec![1.0, 0.0], vec![1.0]], &[1.0, 2.0], ridge()).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

// --- transforms -------------------------------------------------------------

#[test]
fn transform_none_is_identity() {
    let (z, j) = transform_and_jacobian(3.5_f64, Transform::None).unwrap();
    assert_eq!(z, 3.5);
    assert_eq!(j, 0.0);
}

#[test]
fn transform_log_and_log1p_jacobians() {
    let (z, j) = transform_and_jacobian(2.0_f64, Transform::Log).unwrap();
    assert!((z - 2.0_f64.ln()).abs() < 1e-12);
    assert!((j + 2.0_f64.ln()).abs() < 1e-12); // log|dz/dx| = -ln x

    let (z1, j1) = transform_and_jacobian(1.0_f64, Transform::Log1p).unwrap();
    assert!((z1 - 2.0_f64.ln()).abs() < 1e-12); // ln(1+1) = ln 2
    assert!((j1 + 2.0_f64.ln()).abs() < 1e-12);
}

#[test]
fn transform_domain_and_unsupported_errors() {
    assert_eq!(
        transform_and_jacobian(0.0_f64, Transform::Log).err(),
        Some(BrcdError(BrcdErrorEnum::InvalidTransformDomain))
    );
    assert_eq!(
        transform_and_jacobian(-2.0_f64, Transform::Log1p).err(),
        Some(BrcdError(BrcdErrorEnum::InvalidTransformDomain))
    );
    assert_eq!(
        transform_and_jacobian(1.0_f64, Transform::Yeojohnson).err(),
        Some(BrcdError(BrcdErrorEnum::YeojohnsonUnsupported))
    );
}

#[test]
fn effective_transform_downgrade_ladder() {
    // log stays log when all values are positive.
    assert_eq!(
        effective_transform(&[1.0_f64, 2.0, 3.0], Transform::Log),
        Transform::Log
    );
    // log → log1p when a zero (or negative ≥ -1) is present.
    assert_eq!(
        effective_transform(&[0.0_f64, 2.0], Transform::Log),
        Transform::Log1p
    );
    // log → yeojohnson when a value < -1 is present.
    assert_eq!(
        effective_transform(&[-2.0_f64, 2.0], Transform::Log),
        Transform::Yeojohnson
    );
    // log1p → yeojohnson on values < -1, else stays.
    assert_eq!(
        effective_transform(&[-2.0_f64], Transform::Log1p),
        Transform::Yeojohnson
    );
    assert_eq!(
        effective_transform(&[-0.5_f64, 3.0], Transform::Log1p),
        Transform::Log1p
    );
    // none is unchanged.
    assert_eq!(
        effective_transform(&[-5.0_f64], Transform::None),
        Transform::None
    );
}

// --- single-expert log-density ---------------------------------------------

#[test]
fn parentless_density_matches_closed_form() {
    // y = [1, 2, 3]: mean 2, variance (ddof=1) = 1, transform none.
    let y = vec![1.0, 2.0, 3.0];
    let out = gaussian_single_expert_logdensity(&y, &[], Transform::None, ridge()).unwrap();

    // logpdf(z; μ=2, σ²=1) = -0.5(ln(2π) + (z-2)²).
    let expect = |z: f64| -0.5 * (LN_2PI + (z - 2.0).powi(2));
    assert!((out[0] - expect(1.0)).abs() < 1e-9);
    assert!((out[1] - expect(2.0)).abs() < 1e-9);
    assert!((out[2] - expect(3.0)).abs() < 1e-9);
}

#[test]
fn parented_density_recovers_a_sharp_line() {
    // y = 2 + 3x exactly → near-zero residuals, tiny σ², very high density,
    // ~equal across rows.
    let y = vec![2.0, 5.0, 8.0, 11.0];
    let parents = vec![vec![0.0], vec![1.0], vec![2.0], vec![3.0]];
    let out = gaussian_single_expert_logdensity(&y, &parents, Transform::None, ridge()).unwrap();
    assert!(out.iter().all(|v| v.is_finite()));
    assert!(
        out.iter().all(|&v| v > 5.0),
        "density should be sharp: {out:?}"
    );
    let spread =
        out.iter().cloned().fold(f64::MIN, f64::max) - out.iter().cloned().fold(f64::MAX, f64::min);
    assert!(spread < 1.0, "rows should be ~equal, spread {spread}");
}

#[test]
fn log_transform_applies_the_jacobian() {
    // Parentless with a log transform: out_i = logpdf(ln y_i; mean, var) - ln y_i.
    let y = vec![1.0, 2.0, 4.0];
    let out = gaussian_single_expert_logdensity(&y, &[], Transform::Log, ridge()).unwrap();
    let z: Vec<f64> = y.iter().map(|v| v.ln()).collect();
    let mean = z.iter().sum::<f64>() / 3.0;
    let var = z.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / 2.0;
    for i in 0..3 {
        let expect = -0.5 * ((2.0 * std::f64::consts::PI * var).ln() + (z[i] - mean).powi(2) / var)
            - y[i].ln();
        assert!(
            (out[i] - expect).abs() < 1e-9,
            "row {i}: {} vs {expect}",
            out[i]
        );
    }
}

#[test]
fn auto_downgrade_keeps_density_finite() {
    // log requested but a zero is present → downgrades to log1p, no domain error.
    let y = vec![0.0, 1.0, 3.0];
    let out = gaussian_single_expert_logdensity(&y, &[], Transform::Log, ridge()).unwrap();
    assert!(out.iter().all(|v| v.is_finite()));
}

#[test]
fn yeojohnson_selection_surfaces_unsupported() {
    // log with a value < -1 → downgrades to yeojohnson → unsupported (deferred).
    let y = vec![-2.0, 1.0, 3.0];
    assert_eq!(
        gaussian_single_expert_logdensity(&y, &[], Transform::Log, ridge()).err(),
        Some(BrcdError(BrcdErrorEnum::YeojohnsonUnsupported))
    );
}

#[test]
fn single_expert_rejects_bad_shapes() {
    assert_eq!(
        gaussian_single_expert_logdensity::<f64>(&[], &[], Transform::None, ridge()).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
    // Parents present but count mismatched.
    assert_eq!(
        gaussian_single_expert_logdensity(&[1.0, 2.0], &[vec![0.0]], Transform::None, ridge())
            .err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn parented_density_falls_back_when_no_row_is_finite() {
    // Every parent row is non-finite, so the finite-row filter empties `x_fit`
    // and the fit falls back to the node's sample mean/variance (the parentless
    // closed form) instead of a ridge fit.
    let y = vec![1.0, 2.0, 3.0];
    let parents = vec![vec![f64::NAN], vec![f64::INFINITY], vec![f64::NAN]];
    let out = gaussian_single_expert_logdensity(&y, &parents, Transform::None, ridge()).unwrap();

    // Fallback is mean(y)=2, var(ddof1)=1: identical to the parentless density.
    let parentless = gaussian_single_expert_logdensity(&y, &[], Transform::None, ridge()).unwrap();
    for (a, b) in out.iter().zip(parentless.iter()) {
        assert!((a - b).abs() < 1e-12, "{a} vs {b}");
    }
}

#[test]
fn density_agrees_at_f32_and_f64() {
    let y64 = vec![1.0_f64, 2.0, 3.0];
    let o64 = gaussian_single_expert_logdensity(&y64, &[], Transform::None, RIDGE_DEFAULT).unwrap();
    let y32 = vec![1.0_f32, 2.0, 3.0];
    let o32 = gaussian_single_expert_logdensity(&y32, &[], Transform::None, RIDGE_DEFAULT as f32)
        .unwrap();
    assert!((o64[1] - (-0.5 * LN_2PI)).abs() < 1e-9);
    assert!((o32[1] - (-0.5 * LN_2PI as f32)).abs() < 1e-4);
}
