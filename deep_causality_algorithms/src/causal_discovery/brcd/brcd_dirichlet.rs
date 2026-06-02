/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Discrete Dirichlet posterior-predictive (prequential) family estimator.
//!
//! For a discrete family `(node, parents)` the authoritative `brcd.py`
//! (`dirichlet_postpred_rowwise`, L596) scores each row under a symmetric
//! Dirichlet prior `Dir(α₀, …, α₀)` with `α₀ = α* / K` (`K` = the node's
//! cardinality, `α*` default `5.0`). Within each parent configuration the rows
//! are scored **prequentially** — row `i`'s posterior-predictive probability
//! uses only the prior plus the counts of the earlier rows in the same group:
//!
//! ```text
//! p(xᵢ | past) = (count[xᵢ] + α₀) / (total + α*)
//! ```
//!
//! and the product over a group's rows equals that group's integrated marginal
//! likelihood. This module returns the per-row **log** probabilities, the
//! discrete counterpart of [`super::brcd_gaussian::gaussian_family_logdensity`].

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_num::{FromPrimitive, RealField};
use std::collections::BTreeMap;

/// Default Dirichlet concentration `α*` (matches `brcd.py`).
pub const ALPHA_STAR_DEFAULT: f64 = 5.0;

/// Per-row log posterior-predictive probability of the discrete family
/// `p(node | parents)` under a symmetric `Dir(α*/K)` prior, scored
/// prequentially within each parent configuration.
///
/// `node` holds each row's state in `0..cardinality`; `parents` holds each row's
/// discrete parent configuration (empty for a parentless family). `alpha_star`
/// is `α*`.
///
/// # Errors
/// [`BrcdErrorEnum::EmptyData`], [`BrcdErrorEnum::DimensionMismatch`],
/// [`BrcdErrorEnum::ZeroCardinality`], or [`BrcdErrorEnum::StateOutOfRange`].
pub fn dirichlet_logdensity<T: RealField + FromPrimitive>(
    node: &[usize],
    parents: &[Vec<usize>],
    cardinality: usize,
    alpha_star: T,
) -> Result<Vec<T>, BrcdError> {
    let n = node.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    if cardinality == 0 {
        return Err(BrcdError(BrcdErrorEnum::ZeroCardinality));
    }
    let p = parents.first().map_or(0, Vec::len);
    if !parents.is_empty() && (parents.len() != n || parents.iter().any(|r| r.len() != p)) {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    if node.iter().any(|&x| x >= cardinality) {
        return Err(BrcdError(BrcdErrorEnum::StateOutOfRange));
    }

    let alpha0 = alpha_star / from_usize::<T>(cardinality);

    // One running stream per parent configuration: (per-state counts, total).
    let mut streams: BTreeMap<Vec<usize>, (Vec<usize>, usize)> = BTreeMap::new();
    let mut out = Vec::with_capacity(n);

    for (i, &x) in node.iter().enumerate() {
        let key = parents.get(i).cloned().unwrap_or_default();
        let (counts, total) = streams
            .entry(key)
            .or_insert_with(|| (vec![0usize; cardinality], 0usize));

        let num = from_usize::<T>(counts[x]) + alpha0;
        let den = from_usize::<T>(*total) + alpha_star;
        out.push((num / den).ln());

        counts[x] += 1;
        *total += 1;
    }

    Ok(out)
}

/// `T` from a `usize`.
fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}
