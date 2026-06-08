/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Staged builder for [`BrcdLoaderConfig`]. The three required fields are
//! sequential stages; the optional `cpdag_path` / `csv` and `build()` live on the
//! final ready stage.

use crate::types::cdl_config_builder::check_file_exists;
use crate::{BrcdLoaderConfig, CdlError, CsvConfig};
use deep_causality_algorithms::brcd::BrcdConfig;

/// Stage 0: needs the normal-dataset path.
pub struct BrcdConfigNeedsNormal;

impl Default for BrcdConfigNeedsNormal {
    fn default() -> Self {
        Self::new()
    }
}

impl BrcdConfigNeedsNormal {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Sets the observational ("normal") dataset path (required).
    pub fn with_normal_path(self, path: impl Into<String>) -> BrcdConfigNeedsAnomalous {
        BrcdConfigNeedsAnomalous {
            normal: path.into(),
        }
    }
}

/// Stage 1: needs the anomalous-dataset path.
pub struct BrcdConfigNeedsAnomalous {
    normal: String,
}

impl BrcdConfigNeedsAnomalous {
    /// Sets the failure ("anomalous") dataset path (required).
    pub fn with_anomalous_path(self, path: impl Into<String>) -> BrcdConfigNeedsConfig {
        BrcdConfigNeedsConfig {
            normal: self.normal,
            anomalous: path.into(),
        }
    }
}

/// Stage 2: needs the algorithm config.
pub struct BrcdConfigNeedsConfig {
    normal: String,
    anomalous: String,
}

impl BrcdConfigNeedsConfig {
    /// Sets the reused algorithm configuration (required; explicit, no hidden default).
    pub fn with_brcd_config<T>(self, brcd_config: BrcdConfig<T>) -> BrcdConfigReady<T> {
        BrcdConfigReady {
            normal: self.normal,
            anomalous: self.anomalous,
            cpdag: None,
            csv: CsvConfig::default(),
            brcd_config,
        }
    }
}

/// Final stage: all required fields set; optional setters and `build()` available.
pub struct BrcdConfigReady<T> {
    normal: String,
    anomalous: String,
    cpdag: Option<String>,
    csv: CsvConfig,
    brcd_config: BrcdConfig<T>,
}

impl<T> BrcdConfigReady<T> {
    /// Sets the optional CPDAG file path. Absent ⇒ BOSS learns the structure.
    pub fn with_cpdag_path(mut self, path: impl Into<String>) -> Self {
        self.cpdag = Some(path.into());
        self
    }

    /// Overrides the shared CSV parse options (defaults to `CsvConfig::default`).
    pub fn with_csv(mut self, csv: CsvConfig) -> Self {
        self.csv = csv;
        self
    }

    /// Builds the [`BrcdLoaderConfig`], verifying the referenced files exist.
    ///
    /// # Errors
    /// [`CdlError::ReadDataError`] if a dataset or CPDAG file is missing.
    pub fn build(self) -> Result<BrcdLoaderConfig<T>, CdlError> {
        check_file_exists(&self.normal)?;
        check_file_exists(&self.anomalous)?;
        if let Some(cpdag) = &self.cpdag {
            check_file_exists(cpdag)?;
        }
        Ok(BrcdLoaderConfig::new(
            self.normal,
            self.anomalous,
            self.cpdag,
            self.csv,
            self.brcd_config,
        ))
    }
}
