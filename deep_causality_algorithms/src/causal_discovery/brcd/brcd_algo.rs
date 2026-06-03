/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Posterior assembly and the BRCD driver.
//!
//! [`brcd_run`] is the public entry point — the supplied-CPDAG branch of the
//! authoritative `brcd_helper` (L1863) plus `brcd_update` (L1756). Given the
//! normal and anomalous datasets and a CPDAG it:
//!
//! 1. concatenates the two frames and appends the `FNODE` indicator column;
//! 2. forms the candidate root-cause sets (all `k`-subsets of the variables);
//! 3. for each candidate, enumerates valid cut configurations
//!    ([`get_configurations_multi`]), F-node-augments each, sizes its Markov
//!    equivalence class and samples one representative DAG;
//! 4. scores every unique `(node, parents)` family **once** (cached) — per-regime
//!    when `FNODE` is a parent, a single expert otherwise — as a per-row
//!    log-likelihood;
//! 5. per candidate, sums each DAG's family log-factors, adds the MEC log-weight
//!    `log(size/Σ)`, `logsumexp`-combines the DAGs per row, sums over rows, adds
//!    the log-prior, then normalizes and ranks the candidates by descending
//!    posterior.
//!
//! The no-CPDAG bootstrap path (BOSS) is out of scope (design D6).

use crate::brcd::brcd_augment::{augmented_graph, f_node_indicator, get_configurations_multi};
use crate::brcd::brcd_cache::{FamilyKey, family_key};
use crate::brcd::brcd_config::{BrcdConfig, FamilyKind};
use crate::brcd::brcd_dirichlet::dirichlet_logdensity;
use crate::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::brcd::brcd_gaussian::{GaussianFamilyConfig, gaussian_family_logdensity};
use crate::brcd::brcd_mec::{mec_sample_dag, mec_size};
use crate::brcd::brcd_result::BrcdResult;
use deep_causality_num::{FromPrimitive, RealField, ToPrimitive};
use deep_causality_rand::Xoshiro256;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::collections::BTreeMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Runs BRCD on the normal/anomalous datasets against a supplied CPDAG and
/// returns the candidate root-cause sets ranked by descending posterior.
///
/// `normal` and `anomalous` are `n × num_vars` row-major matrices over the same
/// variables; `cpdag` is the causal graph over those `num_vars` variables.
///
/// # Errors
/// * [`BrcdErrorEnum::DimensionMismatch`] if the datasets/graph disagree on
///   `num_vars`, the tensors are not 2-D, or `k` is not in `1..=num_vars`.
/// * [`BrcdErrorEnum::EmptyData`] if there are no rows.
/// * any error from configuration enumeration, MEC sizing, or family scoring.
pub fn brcd_run<T, N>(
    normal: &CausalTensor<T>,
    anomalous: &CausalTensor<T>,
    cpdag: &MixedGraph<N>,
    config: &BrcdConfig<T>,
) -> Result<BrcdResult<T>, BrcdError>
where
    T: RealField + FromPrimitive + ToPrimitive + Send + Sync,
    N: Clone,
{
    let (n_normal, num_vars) = shape_2d(normal)?;
    let (n_anom, num_vars2) = shape_2d(anomalous)?;
    if num_vars != num_vars2 || cpdag.num_vertices() != num_vars {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    let n_total = n_normal + n_anom;
    if n_total == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    let k = config.num_root_causes;
    if k == 0 || k > num_vars {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }

    let fnode_idx = num_vars;
    let columns = joint_columns(normal, anomalous, num_vars, n_normal, n_anom);
    let f_bool = f_node_indicator(n_normal, n_anom);
    let (int_columns, cardinalities) = if config.family == FamilyKind::Discrete {
        build_discrete(&columns)?
    } else {
        (Vec::new(), Vec::new())
    };

    let combos = combinations(num_vars, k);
    if combos.is_empty() {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    let log_prior = (T::one() / from_usize::<T>(combos.len())).ln();

    let ctx = ScoreCtx {
        family: config.family,
        gaussian_cfg: GaussianFamilyConfig {
            transform: config.node_transform,
            transform_parents: config.transform_parents,
            ridge: config.ridge,
            gate: config.gate,
        },
        alpha_star: config.alpha_star,
        columns: &columns,
        int_columns: &int_columns,
        cardinalities: &cardinalities,
        f_bool: &f_bool,
        fnode_idx,
        n_total,
    };

    // Phase 1 (sequential): structural enumeration. `mec_sample_dag` threads the
    // RNG candidate-by-candidate, so this stays sequential to keep the run
    // deterministic. Per candidate we retain its sampled DAGs and MEC log-weights;
    // `None` marks a candidate with no valid configuration (scored as −∞).
    let mut rng = Xoshiro256::from_seed(config.seed);
    type Plan<T> = (Vec<MixedGraph<()>>, Vec<T>);
    let mut plans: Vec<Option<Plan<T>>> = Vec::with_capacity(combos.len());
    for combo in &combos {
        let configs = get_configurations_multi(cpdag, combo)?;
        if configs.is_empty() {
            plans.push(None);
            continue;
        }
        let mut dags: Vec<MixedGraph<()>> = Vec::with_capacity(configs.len());
        let mut sizes: Vec<usize> = Vec::with_capacity(configs.len());
        for cfg in &configs {
            let aug = augmented_graph(cfg, combo)?;
            sizes.push(mec_size(&aug)?);
            dags.push(mec_sample_dag(&aug, &mut rng)?);
        }
        let total = sizes.iter().fold(T::zero(), |a, &s| a + from_usize::<T>(s));
        let tiny = from_f64::<T>(1e-300);
        let log_p_g: Vec<T> = sizes
            .iter()
            .map(|&s| (from_usize::<T>(s) / total + tiny).ln())
            .collect();
        plans.push(Some((dags, log_p_g)));
    }

    // Phase 2: collect every unique `(node, parents)` family across all sampled
    // DAGs (first-seen parent order, matching the sequential traversal), then
    // score each one. Scoring is the dominant cost and the families are
    // independent, so it runs in parallel under the `parallel` feature.
    let mut jobs: BTreeMap<FamilyKey, (usize, Vec<usize>)> = BTreeMap::new();
    for (dags, _) in plans.iter().flatten() {
        for dag in dags {
            for node in 0..dag.num_vertices() {
                let parents = dag.parents(node);
                jobs.entry(family_key(node, &parents))
                    .or_insert((node, parents));
            }
        }
    }
    let scored = score_families(&jobs, &ctx)?;

    // Phase 3 (sequential, cheap): assemble each candidate's posterior from the
    // scored families. Single configuration → sum per-family totals; multiple
    // configurations → the per-row mixture (`logsumexp` over DAGs, summed rows).
    let mut log_posterior = Vec::with_capacity(combos.len());
    for plan in &plans {
        let Some((dags, log_p_g)) = plan else {
            log_posterior.push(neg_inf::<T>());
            continue;
        };

        if dags.len() == 1 {
            let dag = &dags[0];
            let mut log_lik = from_usize::<T>(n_total) * log_p_g[0];
            for node in 0..dag.num_vertices() {
                let key = family_key(node, &dag.parents(node));
                log_lik += scored[&key].1;
            }
            log_posterior.push(log_lik + log_prior);
            continue;
        }

        let mut dag_cols: Vec<Vec<T>> = Vec::with_capacity(dags.len());
        for (i, dag) in dags.iter().enumerate() {
            let mut log_joint = vec![T::zero(); n_total];
            for node in 0..dag.num_vertices() {
                let key = family_key(node, &dag.parents(node));
                for (acc, &f) in log_joint.iter_mut().zip(scored[&key].0.iter()) {
                    *acc += f;
                }
            }
            let lg = log_p_g[i];
            for acc in log_joint.iter_mut() {
                *acc += lg;
            }
            dag_cols.push(log_joint);
        }

        let mut log_lik = T::zero();
        let mut row_vals = vec![T::zero(); dag_cols.len()];
        for r in 0..n_total {
            for (slot, col) in row_vals.iter_mut().zip(dag_cols.iter()) {
                *slot = col[r];
            }
            log_lik += logsumexp_slice(&row_vals);
        }
        log_posterior.push(log_lik + log_prior);
    }

    Ok(rank(combos, log_posterior))
}

/// Scores every unique family `(node, parents)` to its per-row log-likelihood
/// and the row-sum total. Each family is independent of the others, so under the
/// `parallel` feature the map runs across CPU cores via `rayon` (mirroring the
/// SURD decomposition loop); the result is identical to the sequential pass.
fn score_families<T>(
    jobs: &BTreeMap<FamilyKey, (usize, Vec<usize>)>,
    ctx: &ScoreCtx<'_, T>,
) -> Result<BTreeMap<FamilyKey, (Vec<T>, T)>, BrcdError>
where
    T: RealField + FromPrimitive + Send + Sync,
{
    #[cfg(feature = "parallel")]
    {
        jobs.par_iter()
            .map(|(key, (node, parents))| {
                let per_row = ctx.score(*node, parents)?;
                let total = per_row.iter().fold(T::zero(), |a, &x| a + x);
                Ok((key.clone(), (per_row, total)))
            })
            .collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        jobs.iter()
            .map(|(key, (node, parents))| {
                let per_row = ctx.score(*node, parents)?;
                let total = per_row.iter().fold(T::zero(), |a, &x| a + x);
                Ok((key.clone(), (per_row, total)))
            })
            .collect()
    }
}

// --- scoring context --------------------------------------------------------

/// Immutable scoring context shared across every family computation.
struct ScoreCtx<'a, T> {
    family: FamilyKind,
    gaussian_cfg: GaussianFamilyConfig<T>,
    alpha_star: T,
    columns: &'a [Vec<T>],
    int_columns: &'a [Vec<usize>],
    cardinalities: &'a [usize],
    f_bool: &'a [bool],
    fnode_idx: usize,
    n_total: usize,
}

impl<T: RealField + FromPrimitive> ScoreCtx<'_, T> {
    /// Per-row log-likelihood of the family `(node, parents)` in the augmented
    /// DAG. Continuous: per-regime when `FNODE` is a parent (separating it from
    /// the continuous parents), a single expert otherwise. Discrete: Dirichlet
    /// over all parents (FNODE included as an ordinary discrete parent).
    fn score(&self, node: usize, parents: &[usize]) -> Result<Vec<T>, BrcdError> {
        match self.family {
            FamilyKind::Continuous => {
                let has_fnode = parents.contains(&self.fnode_idx);
                let cont_parents: Vec<usize> = parents
                    .iter()
                    .copied()
                    .filter(|&p| p != self.fnode_idx)
                    .collect();
                let parent_rows = transpose(self.columns, &cont_parents, self.n_total);
                let f = if has_fnode { Some(self.f_bool) } else { None };
                gaussian_family_logdensity(
                    &self.columns[node],
                    &parent_rows,
                    f,
                    has_fnode,
                    &self.gaussian_cfg,
                )
            }
            FamilyKind::Discrete => {
                let parent_configs = transpose_int(self.int_columns, parents, self.n_total);
                dirichlet_logdensity(
                    &self.int_columns[node],
                    &parent_configs,
                    self.cardinalities[node],
                    self.alpha_star,
                )
            }
        }
    }
}

// --- helpers ----------------------------------------------------------------

/// Returns `(rows, cols)` for a 2-D tensor, else `DimensionMismatch`.
fn shape_2d<T>(t: &CausalTensor<T>) -> Result<(usize, usize), BrcdError> {
    match t.shape() {
        [rows, cols] => Ok((*rows, *cols)),
        _ => Err(BrcdError(BrcdErrorEnum::DimensionMismatch)),
    }
}

/// Builds the `num_vars + 1` joint columns: each variable's normal-then-anomalous
/// values, with the `FNODE` indicator column (`0.0` normal, `1.0` anomalous) last.
fn joint_columns<T: RealField + FromPrimitive>(
    normal: &CausalTensor<T>,
    anomalous: &CausalTensor<T>,
    num_vars: usize,
    n_normal: usize,
    n_anom: usize,
) -> Vec<Vec<T>> {
    let nd = normal.as_slice();
    let ad = anomalous.as_slice();
    let mut cols = Vec::with_capacity(num_vars + 1);
    for j in 0..num_vars {
        let mut col = Vec::with_capacity(n_normal + n_anom);
        for i in 0..n_normal {
            col.push(nd[i * num_vars + j]);
        }
        for i in 0..n_anom {
            col.push(ad[i * num_vars + j]);
        }
        cols.push(col);
    }
    let mut f = vec![T::zero(); n_normal];
    f.extend(std::iter::repeat_n(T::one(), n_anom));
    cols.push(f);
    cols
}

/// Rounds each column to non-negative integer states and infers each column's
/// cardinality `K = max_state + 1`.
fn build_discrete<T: RealField + FromPrimitive + ToPrimitive>(
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

/// Builds the `n_total` parent feature rows from the chosen continuous columns.
fn transpose<T: RealField>(columns: &[Vec<T>], idxs: &[usize], n: usize) -> Vec<Vec<T>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&p| columns[p][i]).collect())
        .collect()
}

/// Builds the `n_total` parent configuration rows from the chosen integer columns.
fn transpose_int(columns: &[Vec<usize>], idxs: &[usize], n: usize) -> Vec<Vec<usize>> {
    if idxs.is_empty() {
        return Vec::new();
    }
    (0..n)
        .map(|i| idxs.iter().map(|&p| columns[p][i]).collect())
        .collect()
}

/// All `k`-subsets of `0..n` in ascending lexicographic order (matching
/// `itertools.combinations`).
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    if k == 0 || k > n {
        return Vec::new();
    }
    let mut idx: Vec<usize> = (0..k).collect();
    let mut out = vec![idx.clone()];
    loop {
        // Rightmost position that can still be advanced.
        let mut i = k;
        let advanced = loop {
            if i == 0 {
                break false;
            }
            i -= 1;
            if idx[i] < n - k + i {
                break true;
            }
        };
        if !advanced {
            break;
        }
        idx[i] += 1;
        for j in (i + 1)..k {
            idx[j] = idx[j - 1] + 1;
        }
        out.push(idx.clone());
    }
    out
}

/// Stable `log(Σ eˣ)` over a slice, shifted by the max for numerical stability.
fn logsumexp_slice<T: RealField>(vals: &[T]) -> T {
    if vals.is_empty() {
        return neg_inf::<T>();
    }
    let max = vals.iter().fold(vals[0], |a, &b| if b > a { b } else { a });
    if !max.is_finite() {
        return max;
    }
    let sum = vals.iter().fold(T::zero(), |acc, &v| acc + (v - max).exp());
    max + sum.ln()
}

/// Ranks the candidates by descending log-posterior and reports the max-shifted
/// `exp` posterior alongside.
///
/// Ranking is done on the **log**-posterior, not on `exp(lp − max)`: when one
/// candidate dominates by more than ~`ln(f64::MAX)` (easily reached summing over
/// many rows), every other `exp(...)` underflows to `0.0` and ties, collapsing the
/// tail to index order. The log-posterior preserves the full ordering; the `exp`
/// value is kept only as an interpretable weight.
fn rank<T: RealField>(combos: Vec<Vec<usize>>, log_posterior: Vec<T>) -> BrcdResult<T> {
    let max = log_posterior
        .iter()
        .fold(neg_inf::<T>(), |a, &b| if b > a { b } else { a });
    let shift = if max.is_finite() { max } else { T::zero() };
    let posterior: Vec<T> = log_posterior.iter().map(|&lp| (lp - shift).exp()).collect();

    let mut order: Vec<usize> = (0..combos.len()).collect();
    order.sort_by(|&a, &b| {
        log_posterior[b]
            .partial_cmp(&log_posterior[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    BrcdResult::new(
        order.iter().map(|&i| combos[i].clone()).collect(),
        order.iter().map(|&i| posterior[i]).collect(),
    )
}

/// Negative infinity in `T` (`ln(0)`).
fn neg_inf<T: RealField>() -> T {
    T::zero().ln()
}

fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}

fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
