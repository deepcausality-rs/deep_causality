/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `boss_learn` — the BOSS entry point: observational data → learned CPDAG.
//!
//! Composes the four stages once: compute the sample covariance, build the
//! [`BicScorer`], run the [`best_order_search`], and convert the learned DAG to a
//! CPDAG ([`dag_to_cpdag`]). The returned [`MixedGraph`] has one vertex per data
//! column (same indexing as the input), so it plugs straight into
//! [`crate::brcd::brcd_run`] as the `Some(cpdag)` argument.
//!
//! Zero-variance columns are **not** dropped (unlike the reference, which removes
//! them to dodge BOSS's singular-matrix error). The score's variance floor and
//! ridge keep such a column finite; it simply gains no edges and stays an
//! isolated CPDAG node. Keeping it preserves the column indexing `brcd_run`
//! relies on, and the reference confirms a zero-variance metric is never a true
//! root cause, so an inert isolated node is harmless.

use crate::brcd::brcd_boss_config::BossConfig;
use crate::brcd::brcd_boss_cpdag::dag_to_cpdag;
use crate::brcd::brcd_boss_score::BicScorer;
use crate::brcd::brcd_boss_search::best_order_search;
use crate::brcd::{BrcdError, BrcdErrorEnum};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};
use deep_causality_topology::MixedGraph;

/// Learns a CPDAG from an observational `n × p` data matrix via BOSS.
///
/// The returned graph has one vertex per column, in input order, ready to feed
/// [`crate::brcd::brcd_run`].
///
/// # Errors
/// * [`BrcdErrorEnum::DimensionMismatch`] if `data` is not a 2-D matrix or has no
///   columns.
/// * [`BrcdErrorEnum::EmptyData`] if there are fewer than two rows (a sample
///   covariance needs at least two observations).
/// * any error propagated from scoring, the order search, or the CPDAG build.
pub fn boss_learn<T>(
    data: &CausalTensor<T>,
    config: &BossConfig<T>,
) -> Result<MixedGraph<()>, BrcdError>
where
    T: RealField + FromPrimitive,
{
    let (n, p) = match data.shape() {
        [rows, cols] => (*rows, *cols),
        _ => return Err(BrcdError(BrcdErrorEnum::DimensionMismatch)),
    };
    if p == 0 {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    if n < 2 {
        // A sample covariance (ddof = 1) needs at least two observations.
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }

    let cov = data
        .sample_covariance()
        .map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))?;
    let scorer = BicScorer::new(&cov, n, config)?;
    let result = best_order_search(&scorer, config.seed)?;

    dag_to_cpdag(&result.parents)
}
