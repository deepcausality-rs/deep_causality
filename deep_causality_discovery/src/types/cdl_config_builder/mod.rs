/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The single source of truth for constructing CDL run configurations.
//!
//! [`CdlConfigBuilder`] starts a **staged** typestate builder for either lineage.
//! Each required field is its own stage, so `build()` is only reachable once every
//! required field is set — omitting one is a compile error, not a runtime check.
//! `build()` additionally verifies that the referenced files exist, failing fast
//! before the pipeline runs. The product configs ([`crate::SurdLoaderConfig`],
//! [`crate::BrcdLoaderConfig`]) have no public constructor, so this builder is the
//! only way to make one.

pub mod brcd_builder;
pub mod surd_builder;

use crate::{CdlError, DataLoadingError};
use brcd_builder::BrcdConfigNeedsNormal;
use surd_builder::SurdConfigNeedsPath;

/// Entry point for the staged CDL config builders.
pub struct CdlConfigBuilder;

impl CdlConfigBuilder {
    /// Starts building a BRCD run configuration. Required, in order:
    /// `with_normal_path`, `with_anomalous_path`, `with_brcd_config`; then optional
    /// `with_cpdag_path` / `with_csv`; then `build()`.
    pub fn build_brcd_config() -> BrcdConfigNeedsNormal {
        BrcdConfigNeedsNormal::new()
    }

    /// Starts building a SURD run configuration at precision `T`. Required, in
    /// order: `with_path`, `with_target_index`, `with_num_features`,
    /// `with_max_order`, `with_analyze`; then optional `with_exclude_indices` /
    /// `with_csv`; then `build()`.
    pub fn build_surd_config<T>() -> SurdConfigNeedsPath<T> {
        SurdConfigNeedsPath::new()
    }
}

/// Fails fast if a referenced file does not exist on disk.
pub(crate) fn check_file_exists(path: &str) -> Result<(), CdlError> {
    if std::path::Path::new(path).exists() {
        Ok(())
    } else {
        Err(CdlError::ReadDataError(DataLoadingError::FileNotFound(
            path.to_string(),
        )))
    }
}
