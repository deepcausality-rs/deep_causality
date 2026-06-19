/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Keyed "learn-once, rank-many" CPDAG cache for the BRCD sub-pipeline.
//!
//! When no CPDAG is supplied, learning the structure with BOSS is the expensive
//! step and depends only on the observational (normal) data plus the BOSS seed.
//! This module persists the learned CPDAG keyed to that input, so a later run on
//! identical data/seed loads the cached graph and skips structure learning.
//!
//! ## Correctness, not convenience
//! A saved CPDAG is valid only for the same normal dataset and the same BOSS
//! configuration. Because the learn step always uses
//! [`BossConfig::with_seed(seed)`], fixing the seed implicitly fixes the entire
//! BOSS config (the two numeric knobs `ε`/`λ` are pinned to the reference). The
//! cache key therefore hashes the normal tensor values, its shape, and the seed.
//!
//! The key is stored out-of-band in a sidecar file `"<cache_path>.key"` so the
//! CSV stays round-trippable through the unchanged
//! [`load_cpdag_csv`]/[`save_cpdag_csv`]. On load the key is recomputed from the
//! current data + seed and compared to the sidecar; only an exact match is a hit.
//! Any mismatch, missing sidecar, or unreadable cache degrades to re-learning and
//! overwrites both the CSV and the sidecar — a stale cache never silently yields a
//! wrong graph.

use crate::types::data_loader::cpdag_csv::{load_cpdag_csv, save_cpdag_csv};
use crate::{BrcdLoadError, Precision};
use deep_causality_algorithms::brcd::{BossConfig, boss_learn};
use deep_causality_num::ToPrimitive;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// FNV-1a offset basis (64-bit).
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime (64-bit).
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Computes a deterministic, dependency-free FNV-1a hash over the normal tensor
/// values, its shape, and the BOSS seed.
///
/// Each value is normalized through `to_f64().to_bits()` so the key is stable
/// across precisions that represent the same numbers, and the shape dims and seed
/// are folded in so a reshape or a seed change invalidates the cache.
pub(crate) fn cache_key<T: ToPrimitive>(normal: &CausalTensor<T>, seed: u64) -> u64 {
    let mut hash = FNV_OFFSET;
    let mut fold = |bytes: [u8; 8]| {
        for b in bytes {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };

    // Shape dims first (length, then each dim) so two tensors with the same flat
    // values but different shapes hash differently.
    fold((normal.shape().len() as u64).to_le_bytes());
    for &dim in normal.shape() {
        fold((dim as u64).to_le_bytes());
    }

    // Values, normalized to f64 bit patterns. A non-finite or non-convertible
    // value folds in a fixed sentinel so the hash stays total and deterministic.
    for value in normal.as_slice() {
        let bits = value.to_f64().map(f64::to_bits).unwrap_or(u64::MAX);
        fold(bits.to_le_bytes());
    }

    // Seed last; this also implicitly fixes the BOSS config (see module docs).
    fold(seed.to_le_bytes());

    hash
}

/// The sidecar path holding the cache key for a given CPDAG cache CSV.
fn sidecar_path(cache_path: &str) -> String {
    format!("{cache_path}.key")
}

/// Reads the stored key from the sidecar, returning `None` if it is missing or
/// unparseable (treated as a cache miss, never a hard failure).
fn read_sidecar(cache_path: &str) -> Option<u64> {
    let raw = std::fs::read_to_string(sidecar_path(cache_path)).ok()?;
    u64::from_str_radix(raw.trim(), 16).ok()
}

/// Writes the key to the sidecar as lowercase hex.
fn write_sidecar(cache_path: &str, key: u64) -> Result<(), BrcdLoadError> {
    std::fs::write(sidecar_path(cache_path), format!("{key:016x}"))
        .map_err(|e| BrcdLoadError::Learning(format!("failed to write cache key sidecar: {e}")))
}

/// Resolves the CPDAG from a keyed cache, learning + persisting on a miss.
///
/// Order of operations:
/// 1. compute `key` from `normal` + `seed`;
/// 2. if the cache CSV exists, its sidecar exists, and the stored key equals the
///    recomputed key, load and return the cached graph (hit, skips BOSS);
/// 3. otherwise run `boss_learn` on `normal` with `BossConfig::with_seed(seed)`
///    (matching `brcd_run(None)` exactly so warm == cold), persist the learned
///    graph to the cache CSV and write the key sidecar, and return it.
///
/// A read/parse failure of either the cache CSV or the sidecar degrades to
/// re-learning rather than failing.
///
/// # Errors
/// * [`BrcdLoadError::Learning`] if BOSS learning or writing the cache/sidecar
///   fails.
/// * [`BrcdLoadError::Cpdag`] if saving the learned graph as CSV fails.
pub(crate) fn resolve_cached_cpdag<T>(
    normal: &CausalTensor<T>,
    cache_path: &str,
    seed: u64,
) -> Result<MixedGraph<()>, BrcdLoadError>
where
    T: Precision + ToPrimitive,
{
    let key = cache_key(normal, seed);

    // Cache hit: CSV present, sidecar present, the stored key matches, and the CSV
    // parses. An unreadable / corrupt CSV simply fails the `let Ok(..)` and falls
    // through to re-learning below.
    if std::path::Path::new(cache_path).exists()
        && read_sidecar(cache_path) == Some(key)
        && let Ok(graph) = load_cpdag_csv(cache_path)
    {
        return Ok(graph);
    }

    // Cache miss or stale: re-learn, then overwrite the cache CSV and sidecar.
    let config = BossConfig::<T>::with_seed(seed);
    let learned = boss_learn(normal, &config)
        .map_err(|e| BrcdLoadError::Learning(format!("BOSS structure learning failed: {e}")))?;

    save_cpdag_csv(&learned, cache_path)?;
    write_sidecar(cache_path, key)?;

    Ok(learned)
}
