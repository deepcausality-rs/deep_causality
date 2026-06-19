/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bootstrap CPDAG-uncertainty (paper §"Uncertainty of CPDAGs", Eq. 8–10).
//!
//! The single-CPDAG path ([`crate::brcd::brcd_run`] with `None`) commits to one
//! learned CPDAG. When the pre-failure structure is itself uncertain, the paper's
//! `BRCD-B10` / `BRCD-B100` variants instead **bootstrap** it: resample the
//! observational data `B` times, learn a CPDAG per resample, weight the distinct
//! CPDAGs by a frequency-corrected posterior, and **marginalize** the per-CPDAG
//! root-cause rankings.
//!
//! This ports the reference `bootstrap_cpdag_list` + `get_top_k_cpdags_with_ratio`
//! + `parallel_weighted_posterior`:
//!
//! 1. **Resample & learn.** For each of `B` resamples (rows drawn with
//!    replacement), [`crate::brcd::boss_learn`] learns a CPDAG; identical CPDAGs
//!    are deduplicated, counting occurrences.
//! 2. **Weight the top-`k`.** Keep the `k` most frequent CPDAGs. Each gets the
//!    log-weight `log p(D | C) + log(1/k) − log(count / Σ count)` — the marginal
//!    data likelihood under one sampled DAG, a uniform CPDAG prior, and the
//!    frequency correction `1/q(C)` that undoes the bootstrap's sampling bias.
//!    Normalizing across the top-`k` (log-sum-exp) gives `P(C | D)`.
//! 3. **Marginalize.** Run BRCD against each kept CPDAG to get `p(R | C, D)`, then
//!    combine `p(R | D) = Σ_C P(C | D) · p(R | C, D)`.
//!
//! When the bootstrap is not requested, callers use [`crate::brcd::brcd_run`]
//! directly — a single learned (or supplied) CPDAG, no resampling.

use crate::brcd::brcd_algo::brcd_run;
use crate::brcd::brcd_boss_config::BossConfig;
use crate::brcd::brcd_boss_learn::boss_learn;
use crate::brcd::brcd_config::{BrcdConfig, FamilyKind};
use crate::brcd::brcd_dirichlet::dirichlet_logdensity;
use crate::brcd::brcd_gaussian::{GaussianFamilyConfig, gaussian_family_logdensity};
use crate::brcd::brcd_result::BrcdResult;
use crate::brcd::{BrcdError, BrcdErrorEnum};
use crate::dag_sampling::sample_dag;
use deep_causality_num::{FromPrimitive, RealField, ToPrimitive};
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::collections::BTreeMap;

/// Configuration for the bootstrap CPDAG-uncertainty variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapConfig {
    /// Number of bootstrap resamples `B` (e.g. 10 for `BRCD-B10`, 100 for B100).
    pub samples: usize,
    /// Number of most-frequent distinct CPDAGs `k` to marginalize over.
    pub top_k: usize,
}

impl BootstrapConfig {
    /// A bootstrap configuration with `samples` resamples and `top_k` retained
    /// CPDAGs.
    pub fn new(samples: usize, top_k: usize) -> Self {
        Self { samples, top_k }
    }
}

/// A distinct learned CPDAG, its bootstrap occurrence count, and one resample
/// that produced it (used to score the structure's data likelihood).
struct CpdagEntry<T> {
    key: CpdagKey,
    cpdag: MixedGraph<()>,
    count: usize,
    sample: CausalTensor<T>,
}

/// Canonical structural identity of a CPDAG: sorted directed arcs and undirected
/// edges. Two CPDAGs are the same iff their keys are equal.
type CpdagKey = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// Runs the bootstrap CPDAG-uncertainty variant and returns the marginalized
/// candidate root-cause ranking.
///
/// `normal` / `anomalous` are the observational / failure datasets; the CPDAGs
/// are learned from `normal`. The result is ranked by descending marginal
/// posterior `p(R | D)`.
///
/// # Errors
/// * [`BrcdErrorEnum::DimensionMismatch`] if the data is not 2-D or `samples` /
///   `top_k` is zero.
/// * [`BrcdErrorEnum::EmptyData`] if `normal` has fewer than two rows.
/// * any error from BOSS learning, configuration enumeration, or scoring.
pub fn brcd_run_bootstrap<T>(
    normal: &CausalTensor<T>,
    anomalous: &CausalTensor<T>,
    config: &BrcdConfig<T>,
    bootstrap: &BootstrapConfig,
) -> Result<BrcdResult<T>, BrcdError>
where
    T: RealField + FromPrimitive + ToPrimitive + Send + Sync,
{
    let (n, p) = shape_2d(normal)?;
    if n < 2 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    if bootstrap.samples == 0 || bootstrap.top_k == 0 {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }

    // Phase 1: resample with replacement, learn a CPDAG each, deduplicate.
    let mut rng = Xoshiro256::from_seed(config.seed);
    let boss_cfg = BossConfig::<T>::with_seed(config.seed);
    let mut summary: Vec<CpdagEntry<T>> = Vec::new();
    for _ in 0..bootstrap.samples {
        let sample = resample_rows(normal, n, p, &mut rng)?;
        let cpdag = boss_learn(&sample, &boss_cfg)?;
        let key = cpdag_key(&cpdag);
        match summary.iter_mut().find(|e| e.key == key) {
            Some(e) => e.count += 1,
            None => summary.push(CpdagEntry {
                key,
                cpdag,
                count: 1,
                sample,
            }),
        }
    }

    // Phase 2: keep the top-k by frequency, then weight each CPDAG by
    // `log p(D|C) + log(1/k) − log(count/Σcount)` and normalize (log-sum-exp).
    summary.sort_by_key(|e| std::cmp::Reverse(e.count));
    summary.truncate(bootstrap.top_k);
    let k = summary.len();
    let total_top: usize = summary.iter().map(|e| e.count).sum();
    let log_pc = (T::one() / from_usize::<T>(k)).ln();

    let mut log_w = Vec::with_capacity(k);
    for e in &summary {
        let log_joint = joint_log_likelihood(&e.cpdag, &e.sample, config, &mut rng)?;
        let q = from_usize::<T>(e.count) / from_usize::<T>(total_top);
        log_w.push(log_joint + log_pc - q.ln());
    }
    let log_z = logsumexp(&log_w);
    let weights: Vec<T> = log_w.iter().map(|&lw| (lw - log_z).exp()).collect();

    // Phase 3: per-CPDAG BRCD posterior p(R|C,D), weighted-summed into p(R|D).
    let mut acc: BTreeMap<Vec<usize>, T> = BTreeMap::new();
    for (e, &w) in summary.iter().zip(weights.iter()) {
        let res = brcd_run::<T, ()>(normal, anomalous, Some(&e.cpdag), config)?;
        let post = res.posterior();
        let sum = post.iter().fold(T::zero(), |a, &x| a + x);
        for (cand, &pv) in res.ranks().iter().zip(post.iter()) {
            let p_rc = if sum > T::zero() { pv / sum } else { T::zero() };
            *acc.entry(cand.clone()).or_insert_with(T::zero) += w * p_rc;
        }
    }

    Ok(rank_normalized(acc))
}

/// Marginal data log-likelihood `log p(D | C)`: sample one DAG from the CPDAG and
/// sum the per-family conditional log-densities over the data, mirroring the
/// reference `get_top_k_cpdags_with_ratio`.
fn joint_log_likelihood<T>(
    cpdag: &MixedGraph<()>,
    sample: &CausalTensor<T>,
    config: &BrcdConfig<T>,
    rng: &mut Xoshiro256,
) -> Result<T, BrcdError>
where
    T: RealField + FromPrimitive + ToPrimitive,
{
    let dag = sample_dag::<T, (), _>(cpdag, rng)?;
    let (n, p) = shape_2d(sample)?;
    let columns = columns_of(sample, n, p);

    let mut total = T::zero();
    match config.family {
        FamilyKind::Continuous => {
            let gcfg = GaussianFamilyConfig {
                transform: config.node_transform,
                transform_parents: config.transform_parents,
                ridge: config.ridge,
                gate: config.gate,
            };
            for node in 0..p {
                let parents = dag.parents(node);
                let parent_rows = transpose(&columns, &parents, n);
                let rows =
                    gaussian_family_logdensity(&columns[node], &parent_rows, None, false, &gcfg)?;
                total += rows.iter().fold(T::zero(), |a, &x| a + x);
            }
        }
        FamilyKind::Discrete => {
            let (int_columns, cardinalities) = build_discrete(&columns)?;
            for node in 0..p {
                let parents = dag.parents(node);
                let parent_configs = transpose_int(&int_columns, &parents, n);
                let rows = dirichlet_logdensity(
                    &int_columns[node],
                    &parent_configs,
                    cardinalities[node],
                    config.alpha_star,
                )?;
                total += rows.iter().fold(T::zero(), |a, &x| a + x);
            }
        }
    }
    Ok(total)
}

// --- helpers ----------------------------------------------------------------

/// Ranks the accumulated `candidate → weighted posterior` map by descending
/// posterior, normalizing the weights to sum to one.
fn rank_normalized<T: RealField>(acc: BTreeMap<Vec<usize>, T>) -> BrcdResult<T> {
    let total = acc.values().fold(T::zero(), |a, &x| a + x);
    let mut items: Vec<(Vec<usize>, T)> = acc
        .into_iter()
        .map(|(c, v)| {
            let p = if total > T::zero() { v / total } else { v };
            (c, p)
        })
        .collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    BrcdResult::new(
        items.iter().map(|(c, _)| c.clone()).collect(),
        items.iter().map(|(_, v)| *v).collect(),
    )
}

/// Draws `n` rows of `normal` (each `p` wide) with replacement into a fresh
/// tensor.
fn resample_rows<T: RealField>(
    normal: &CausalTensor<T>,
    n: usize,
    p: usize,
    rng: &mut Xoshiro256,
) -> Result<CausalTensor<T>, BrcdError> {
    let data = normal.as_slice();
    let mut flat = Vec::with_capacity(n * p);
    for _ in 0..n {
        let r = rng.random_range(0..n);
        flat.extend_from_slice(&data[r * p..r * p + p]);
    }
    CausalTensor::new(flat, vec![n, p]).map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))
}

/// Canonical structural key of a CPDAG: sorted (arcs, undirected edges).
fn cpdag_key(g: &MixedGraph<()>) -> CpdagKey {
    let mut arcs = Vec::new();
    for v in 0..g.num_vertices() {
        for parent in g.parents(v) {
            arcs.push((parent, v));
        }
    }
    arcs.sort_unstable();
    let mut undirected = g.undirected_edges();
    undirected.sort_unstable();
    (arcs, undirected)
}

/// Extracts the `p` columns of a row-major `n × p` tensor.
fn columns_of<T: RealField>(t: &CausalTensor<T>, n: usize, p: usize) -> Vec<Vec<T>> {
    let data = t.as_slice();
    (0..p)
        .map(|j| (0..n).map(|i| data[i * p + j]).collect())
        .collect()
}

/// Builds the `n` parent feature rows from the chosen continuous columns.
fn transpose<T: RealField>(columns: &[Vec<T>], idxs: &[usize], n: usize) -> Vec<Vec<T>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&c| columns[c][i]).collect())
        .collect()
}

/// Builds the `n` parent configuration rows from the chosen integer columns.
fn transpose_int(columns: &[Vec<usize>], idxs: &[usize], n: usize) -> Vec<Vec<usize>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&c| columns[c][i]).collect())
        .collect()
}

/// Rounds each column to non-negative integer states and infers its cardinality
/// `K = max_state + 1` (mirrors the driver's discrete binning).
fn build_discrete<T: RealField + ToPrimitive>(
    columns: &[Vec<T>],
) -> Result<(Vec<Vec<usize>>, Vec<usize>), BrcdError> {
    let mut ints = Vec::with_capacity(columns.len());
    let mut cards = Vec::with_capacity(columns.len());
    for col in columns {
        let mut ic = Vec::with_capacity(col.len());
        let mut max_state = 0usize;
        for &v in col {
            let rounded = v.round();
            if rounded < T::zero() {
                return Err(BrcdError(BrcdErrorEnum::StateOutOfRange));
            }
            let s = rounded
                .to_usize()
                .ok_or(BrcdError(BrcdErrorEnum::StateOutOfRange))?;
            max_state = max_state.max(s);
            ic.push(s);
        }
        ints.push(ic);
        cards.push(max_state + 1);
    }
    Ok((ints, cards))
}

/// Returns `(rows, cols)` for a 2-D tensor, else `DimensionMismatch`.
fn shape_2d<T>(t: &CausalTensor<T>) -> Result<(usize, usize), BrcdError> {
    match t.shape() {
        [rows, cols] => Ok((*rows, *cols)),
        _ => Err(BrcdError(BrcdErrorEnum::DimensionMismatch)),
    }
}

/// Stable `log(Σ eˣ)` over a slice, shifted by the max.
fn logsumexp<T: RealField>(vals: &[T]) -> T {
    if vals.is_empty() {
        return T::zero().ln();
    }
    let max = vals.iter().fold(vals[0], |a, &b| if b > a { b } else { a });
    if !max.is_finite() {
        return max;
    }
    let sum = vals.iter().fold(T::zero(), |acc, &v| acc + (v - max).exp());
    max + sum.ln()
}

fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}
