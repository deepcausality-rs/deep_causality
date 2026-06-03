/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_dirichlet::{ALPHA_STAR_DEFAULT, dirichlet_logdensity};
use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};

#[test]
fn parentless_prequential_matches_closed_form() {
    // K=2, α*=5 → α₀=2.5. Sequence [0,1,0]:
    //   row0: (0+2.5)/(0+5)   = 0.5
    //   row1: (0+2.5)/(1+5)   = 2.5/6
    //   row2: (1+2.5)/(2+5)   = 3.5/7 = 0.5
    let node = vec![0, 1, 0];
    let out = dirichlet_logdensity(&node, &[], 2, ALPHA_STAR_DEFAULT).unwrap();
    assert!((out[0] - 0.5_f64.ln()).abs() < 1e-12);
    assert!((out[1] - (2.5_f64 / 6.0).ln()).abs() < 1e-12);
    assert!((out[2] - 0.5_f64.ln()).abs() < 1e-12);
}

#[test]
fn parent_configurations_are_independent_streams() {
    // K=2, α*=5, α₀=2.5. Configs [0],[1],[0]:
    //   row0: config 0, x=0 → (0+2.5)/(0+5)   = 0.5
    //   row1: config 1, x=0 → (0+2.5)/(0+5)   = 0.5   (fresh stream)
    //   row2: config 0, x=1 → (0+2.5)/(1+5)   = 2.5/6 (config-0 has seen one row)
    let node = vec![0, 0, 1];
    let parents = vec![vec![0], vec![1], vec![0]];
    let out = dirichlet_logdensity(&node, &parents, 2, ALPHA_STAR_DEFAULT).unwrap();
    assert!((out[0] - 0.5_f64.ln()).abs() < 1e-12);
    assert!((out[1] - 0.5_f64.ln()).abs() < 1e-12);
    assert!((out[2] - (2.5_f64 / 6.0).ln()).abs() < 1e-12);
}

#[test]
fn product_over_rows_is_the_marginal_likelihood() {
    // The product of per-row predictives is the integrated marginal likelihood;
    // each factor is in (0, 1], so every log is ≤ 0.
    let node = vec![0, 1, 1, 0, 1];
    let out = dirichlet_logdensity(&node, &[], 2, ALPHA_STAR_DEFAULT).unwrap();
    assert!(out.iter().all(|&v| v <= 0.0));
}

#[test]
fn rejects_bad_inputs() {
    assert_eq!(
        dirichlet_logdensity::<f64>(&[], &[], 2, ALPHA_STAR_DEFAULT).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
    assert_eq!(
        dirichlet_logdensity::<f64>(&[0, 1], &[], 0, ALPHA_STAR_DEFAULT).err(),
        Some(BrcdError(BrcdErrorEnum::ZeroCardinality))
    );
    assert_eq!(
        dirichlet_logdensity::<f64>(&[0, 2], &[], 2, ALPHA_STAR_DEFAULT).err(),
        Some(BrcdError(BrcdErrorEnum::StateOutOfRange))
    );
    assert_eq!(
        dirichlet_logdensity::<f64>(&[0, 1], &[vec![0]], 2, ALPHA_STAR_DEFAULT).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
    // A non-positive concentration is rejected rather than producing NaN/inf logs.
    assert_eq!(
        dirichlet_logdensity::<f64>(&[0, 1], &[], 2, 0.0).err(),
        Some(BrcdError(BrcdErrorEnum::NonPositiveConcentration))
    );
    assert_eq!(
        dirichlet_logdensity::<f64>(&[0, 1], &[], 2, -1.0).err(),
        Some(BrcdError(BrcdErrorEnum::NonPositiveConcentration))
    );
}

#[test]
fn agrees_at_f32_and_f64() {
    let node = vec![0, 1, 0];
    let o64 = dirichlet_logdensity(&node, &[], 2, ALPHA_STAR_DEFAULT).unwrap();
    let o32 = dirichlet_logdensity(&node, &[], 2, ALPHA_STAR_DEFAULT as f32).unwrap();
    assert!((o64[1] - (2.5_f64 / 6.0).ln()).abs() < 1e-12);
    assert!((o32[1] - (2.5_f32 / 6.0).ln()).abs() < 1e-5);
}
