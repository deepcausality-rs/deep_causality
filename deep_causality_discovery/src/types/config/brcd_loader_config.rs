/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsvConfig;
use deep_causality_algorithms::brcd::BrcdConfig;
use std::fmt;

/// The fully-specified configuration for a BRCD CDL run.
///
/// Constructed only through [`crate::CdlConfigBuilder::build_brcd_config`] (the
/// single source of truth). The two dataset paths and the algorithm `BrcdConfig`
/// are required and explicit; `cpdag_path` is optional, and `None` makes the
/// driver learn the CPDAG from the normal data via BOSS.
#[derive(Debug, Clone)]
pub struct BrcdLoaderConfig<T> {
    normal_path: String,
    anomalous_path: String,
    cpdag_path: Option<String>,
    cpdag_cache_path: Option<String>,
    csv: CsvConfig,
    brcd_config: BrcdConfig<T>,
}

impl<T> BrcdLoaderConfig<T> {
    /// Builder-only constructor.
    pub(crate) fn new(
        normal_path: String,
        anomalous_path: String,
        cpdag_path: Option<String>,
        cpdag_cache_path: Option<String>,
        csv: CsvConfig,
        brcd_config: BrcdConfig<T>,
    ) -> Self {
        Self {
            normal_path,
            anomalous_path,
            cpdag_path,
            cpdag_cache_path,
            csv,
            brcd_config,
        }
    }
}

// Getters
impl<T> BrcdLoaderConfig<T> {
    /// The observational ("normal") dataset path.
    pub fn normal_path(&self) -> &str {
        &self.normal_path
    }

    /// The failure ("anomalous") dataset path.
    pub fn anomalous_path(&self) -> &str {
        &self.anomalous_path
    }

    /// The optional CPDAG file path. `None` defers structure learning to BOSS.
    pub fn cpdag_path(&self) -> Option<&String> {
        self.cpdag_path.as_ref()
    }

    /// The optional CPDAG cache path (distinct from the supplied [`Self::cpdag_path`]).
    ///
    /// When set and no `cpdag_path` is supplied, the loader uses a keyed
    /// "learn-once, rank-many" cache at this path: a cache hit (same normal data +
    /// seed) loads the stored graph and skips BOSS; a miss/stale entry re-learns
    /// and overwrites the cache. `None` leaves structure learning to `brcd_run`.
    pub fn cpdag_cache_path(&self) -> Option<&String> {
        self.cpdag_cache_path.as_ref()
    }

    /// The shared CSV parse options.
    pub fn csv(&self) -> &CsvConfig {
        &self.csv
    }

    /// The reused algorithm configuration.
    pub fn brcd_config(&self) -> &BrcdConfig<T> {
        &self.brcd_config
    }
}

impl<T> fmt::Display for BrcdLoaderConfig<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BrcdLoaderConfig(normal: {}, anomalous: {}, cpdag: {}, cpdag_cache: {})",
            self.normal_path,
            self.anomalous_path,
            self.cpdag_path.as_deref().unwrap_or("None (BOSS)"),
            self.cpdag_cache_path.as_deref().unwrap_or("None")
        )
    }
}
