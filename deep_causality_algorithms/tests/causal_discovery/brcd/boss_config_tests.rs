/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::{BOSS_LAMBDA_DEFAULT, BOSS_RIDGE_DEFAULT, BossConfig};

#[test]
fn with_seed_sets_reference_defaults() {
    let c = BossConfig::<f64>::with_seed(7);
    assert_eq!(c.seed, 7);
    assert_eq!(c.ridge_eps, BOSS_RIDGE_DEFAULT);
    assert_eq!(c.bic_lambda, BOSS_LAMBDA_DEFAULT);
}

#[test]
fn default_is_seed_zero_with_reference_defaults() {
    let c = BossConfig::<f64>::default();
    assert_eq!(c.seed, 0);
    assert_eq!(c.ridge_eps, BOSS_RIDGE_DEFAULT);
    assert_eq!(c.bic_lambda, BOSS_LAMBDA_DEFAULT);
}

#[test]
fn reference_default_constants_match_the_paper() {
    assert_eq!(BOSS_RIDGE_DEFAULT, 1e-6);
    assert_eq!(BOSS_LAMBDA_DEFAULT, 2.0);
}

#[test]
fn config_is_clone_and_debug() {
    let c = BossConfig::<f64>::with_seed(3);
    let d = c.clone();
    assert_eq!(d.seed, 3);
    // Debug is derived; exercising it keeps the impl covered.
    assert!(format!("{c:?}").contains("BossConfig"));
}
