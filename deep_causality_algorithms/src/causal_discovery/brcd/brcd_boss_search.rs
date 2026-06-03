/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Best-order search — the permutation optimization at the heart of BOSS.
//!
//! A port of the order loop in the reference `boss.py` (without background
//! knowledge, which the Petshop / no-CPDAG path does not use). Each variable has
//! a grow-shrink tree ([`Gst`]); the search repeatedly moves each variable to the
//! order position that maximizes the **total** score (the sum of every variable's
//! best parent-set score given its predecessors), sweeping until no move improves
//! the total.
//!
//! Determinism: the per-sweep variable visiting order is shuffled with a
//! [`Xoshiro256`] seeded from the config, so a fixed seed and dataset always
//! yield the same final order (and hence the same DAG). The shuffle is the only
//! randomness; everything else is exact.

use crate::brcd::BrcdError;
use crate::brcd::brcd_boss_gst::Gst;
use crate::brcd::brcd_boss_score::FamilyScorer;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_rand::{Rng, Xoshiro256};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Strict-improvement tolerance (reference `tol = 1e-8`).
const TOL: f64 = 1e-8;
/// Hard cap on sweeps (reference `max_rounds = 2000`).
const MAX_ROUNDS: usize = 2000;

/// The outcome of a best-order search: the final variable order and, for each
/// variable, its chosen parent set (defining the learned DAG).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderSearchResult {
    /// The variables in their final topological order.
    pub order: Vec<usize>,
    /// `parents[v]` is the (ascending) parent set chosen for variable `v`.
    pub parents: Vec<Vec<usize>>,
}

/// Runs the best-order search and returns the final order and per-variable
/// parent sets (the learned DAG).
///
/// # Errors
/// Propagates any scorer error encountered while building or tracing the
/// grow-shrink trees.
pub fn best_order_search<T, S>(scorer: &S, seed: u64) -> Result<OrderSearchResult, BrcdError>
where
    T: RealField + FromPrimitive,
    S: FamilyScorer<T>,
{
    let p = scorer.num_vars();
    let mut gsts: Vec<Gst<T>> = Vec::with_capacity(p);
    for v in 0..p {
        gsts.push(Gst::new(v, scorer)?);
    }

    let mut order: Vec<usize> = (0..p).collect();
    if p <= 1 {
        return finalize(order, &mut gsts, scorer);
    }

    let tol = from_f64::<T>(TOL);
    let mut rng = Xoshiro256::from_seed(seed);
    let mut visited: BTreeSet<Vec<usize>> = BTreeSet::new();

    for _ in 0..MAX_ROUNDS {
        // Break if this exact order recurs (defensive cycle guard).
        if !visited.insert(order.clone()) {
            break;
        }

        let current_total = total_score(&order, &mut gsts, scorer)?;

        // One sweep over the variables in a seed-shuffled order.
        let mut variables = order.clone();
        shuffle(&mut variables, &mut rng);
        let mut improved = false;
        for v in variables {
            if better_mutation(v, &mut order, &mut gsts, scorer, tol)? {
                improved = true;
            }
        }

        let new_total = total_score(&order, &mut gsts, scorer)?;
        if !improved || !new_total.is_finite() || new_total <= current_total + tol {
            break;
        }
    }

    finalize(order, &mut gsts, scorer)
}

/// Total order score: the sum over the order of each variable's best parent-set
/// score given its predecessors.
fn total_score<T, S>(order: &[usize], gsts: &mut [Gst<T>], scorer: &S) -> Result<T, BrcdError>
where
    T: RealField + FromPrimitive,
    S: FamilyScorer<T>,
{
    let mut total = T::zero();
    let mut prefix: Vec<usize> = Vec::with_capacity(order.len());
    for &w in order {
        let (_, val) = gsts[w].trace(&prefix, scorer)?;
        if !val.is_finite() {
            return Ok(neg_inf::<T>());
        }
        total += val;
        prefix.push(w);
    }
    Ok(total)
}

/// Moves `v` to the order position that maximizes the total score, if that
/// strictly beats its current position by more than `tol`. Returns whether a
/// move was made.
fn better_mutation<T, S>(
    v: usize,
    order: &mut Vec<usize>,
    gsts: &mut [Gst<T>],
    scorer: &S,
    tol: T,
) -> Result<bool, BrcdError>
where
    T: RealField + FromPrimitive,
    S: FamilyScorer<T>,
{
    let p = order.len();
    let i = order
        .iter()
        .position(|&x| x == v)
        .expect("v is in the order");

    // scores[j] = total score with v inserted just before position j.
    let mut scores = vec![neg_inf::<T>(); p + 1];
    let mut prefix: Vec<usize> = Vec::with_capacity(p);
    let mut accum = T::zero();

    for (j, &w) in order.iter().enumerate() {
        let (_, sv) = gsts[v].trace(&prefix, scorer)?;
        if sv.is_finite() && accum.is_finite() {
            scores[j] = sv + accum;
        }
        if v != w {
            let (_, sw) = gsts[w].trace(&prefix, scorer)?;
            accum = if sw.is_finite() && accum.is_finite() {
                accum + sw
            } else {
                neg_inf::<T>()
            };
            prefix.push(w);
        }
    }
    let (_, sv_end) = gsts[v].trace(&prefix, scorer)?;
    if sv_end.is_finite() && accum.is_finite() {
        scores[p] = sv_end + accum;
    }

    let best = argmax(&scores);
    // Accept the move only on a strict improvement (NaN-safe via partial_cmp).
    if scores[best].partial_cmp(&(scores[i] + tol)) != Some(Ordering::Greater) {
        return Ok(false);
    }

    // Re-insert v at `best`, accounting for its removal shifting later indices.
    order.remove(i);
    let insert_at = best - usize::from(best > i);
    order.insert(insert_at, v);
    Ok(true)
}

/// Builds the final per-variable parent sets from the converged order.
fn finalize<T, S>(
    order: Vec<usize>,
    gsts: &mut [Gst<T>],
    scorer: &S,
) -> Result<OrderSearchResult, BrcdError>
where
    T: RealField + FromPrimitive,
    S: FamilyScorer<T>,
{
    let p = order.len();
    let mut parents = vec![Vec::new(); p];
    let mut prefix: Vec<usize> = Vec::with_capacity(p);
    for &w in &order {
        let (pa, _) = gsts[w].trace(&prefix, scorer)?;
        parents[w] = pa;
        prefix.push(w);
    }
    Ok(OrderSearchResult { order, parents })
}

/// Index of the first maximal element.
fn argmax<T: RealField>(scores: &[T]) -> usize {
    let mut best = 0;
    for (k, s) in scores.iter().enumerate().skip(1) {
        if *s > scores[best] {
            best = k;
        }
    }
    best
}

/// In-place Fisher-Yates shuffle using the project RNG (deterministic per seed).
fn shuffle(items: &mut [usize], rng: &mut Xoshiro256) {
    for i in (1..items.len()).rev() {
        let j: usize = rng.random_range(0..(i + 1));
        items.swap(i, j);
    }
}

/// Negative infinity in `T` (`ln(0)`).
fn neg_inf<T: RealField>() -> T {
    T::zero().ln()
}

fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
