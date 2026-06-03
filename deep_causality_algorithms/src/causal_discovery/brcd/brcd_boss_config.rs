/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration for the BOSS structure-learning preprocessor.
//!
//! BOSS is not a standalone discovery algorithm here — it is the optional
//! preprocessing step that learns BRCD's required CPDAG from observational data
//! when none is supplied. Its files live under the `brcd` module with the
//! `brcd_boss_` prefix.

use deep_causality_num::FromPrimitive;

/// Default ridge `ε` added to the parent covariance block before the Schur
/// solve, matching the reference `local_score_BIC_from_cov` (`1e-6`).
pub const BOSS_RIDGE_DEFAULT: f64 = 1e-6;

/// Default BIC penalty discount `λ`, matching the reference (`lambda_value = 2`).
pub const BOSS_LAMBDA_DEFAULT: f64 = 2.0;

/// Configuration for a BOSS run.
///
/// Fields are public to mirror the sibling [`crate::brcd::BrcdConfig`]; the
/// constructors set the reference defaults. The two numeric knobs are pinned to
/// the reference (`ε = 1e-6`, `λ = 2`); `seed` drives the deterministic order
/// search (the per-sweep variable shuffle).
#[derive(Debug, Clone)]
pub struct BossConfig<T> {
    /// Ridge `ε` added to the diagonal of the parent covariance block so the
    /// Schur-complement conditional-variance solve stays finite when parents are
    /// collinear.
    pub ridge_eps: T,
    /// BIC penalty discount `λ` (per parent, plus the intercept term).
    pub bic_lambda: T,
    /// Seed for the deterministic order search.
    pub seed: u64,
}

impl<T: FromPrimitive> BossConfig<T> {
    /// A configuration with the reference defaults (`ε = 1e-6`, `λ = 2`) and the
    /// given search seed.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            ridge_eps: from_f64(BOSS_RIDGE_DEFAULT),
            bic_lambda: from_f64(BOSS_LAMBDA_DEFAULT),
            seed,
        }
    }
}

impl<T: FromPrimitive> Default for BossConfig<T> {
    fn default() -> Self {
        Self::with_seed(0)
    }
}

fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
