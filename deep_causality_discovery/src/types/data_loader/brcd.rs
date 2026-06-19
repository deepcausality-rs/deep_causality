/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Internal loader that turns a [`BrcdLoaderConfig`] into a [`BrcdInput`] bundle.
//!
//! This is `pub(crate)`: it is not a user-facing entry. The BRCD sub-pipeline's
//! `load_brcd_input` stage invokes it, so loading happens *inside* the CDL
//! pipeline (mirroring SURD's in-pipeline load), not as a separate call.

use crate::types::data_loader::cast::cast_loaded_tensor;
use crate::types::data_loader::cpdag_cache::resolve_cached_cpdag;
use crate::types::data_loader::cpdag_csv::load_cpdag_csv;
use crate::{
    BrcdInput, BrcdLoadError, BrcdLoaderConfig, CsvConfig, CsvDataLoader, DataLoader,
    DataLoaderConfig, Precision,
};
use deep_causality_num::ToPrimitive;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// Loads the two datasets and the optional CPDAG named by a [`BrcdLoaderConfig`]
/// into a single [`BrcdInput`] bundle.
pub(crate) struct BrcdDataLoader;

impl BrcdDataLoader {
    /// Builds a [`BrcdInput`] from the loader config.
    ///
    /// # Errors
    /// * [`BrcdLoadError::DataLoading`] if either dataset fails to load.
    /// * [`BrcdLoadError::DimensionMismatch`] if a dataset is not 2-D, the two
    ///   datasets disagree on variable count, or the CPDAG's vertex count differs.
    /// * [`BrcdLoadError::Cpdag`] if the CPDAG file fails to load or parse.
    /// * [`BrcdLoadError::Learning`] if learning or persisting a cached CPDAG fails.
    pub(crate) fn load<T: Precision + ToPrimitive>(
        config: &BrcdLoaderConfig<T>,
    ) -> Result<BrcdInput<T>, BrcdLoadError> {
        let normal = load_matrix::<T>(config.normal_path(), config.csv())?;
        let anomalous = load_matrix::<T>(config.anomalous_path(), config.csv())?;

        let num_vars = normal.shape()[1];
        let anom_vars = anomalous.shape()[1];
        if num_vars != anom_vars {
            return Err(BrcdLoadError::DimensionMismatch(format!(
                "normal has {} variables but anomalous has {}",
                num_vars, anom_vars
            )));
        }

        // CPDAG source resolution, in priority order:
        //   1. supplied `cpdag_path`  -> load the user-managed file (unchanged);
        //   2. else `cpdag_cache_path` -> keyed learn-once cache (hit loads, miss
        //      learns + persists); the learn step matches `brcd_run(None)` exactly
        //      so a warm (cached) run equals a cold (learned) run;
        //   3. else `None`            -> defer learning to `brcd_run` (unchanged).
        let cpdag: Option<MixedGraph<()>> = match config.cpdag_path() {
            Some(path) => Some(load_cpdag_csv(path)?),
            None => match config.cpdag_cache_path() {
                Some(cache_path) => Some(resolve_cached_cpdag::<T>(
                    &normal,
                    cache_path,
                    config.brcd_config().seed,
                )?),
                None => None,
            },
        };

        // The same dimension check applies whether the graph was supplied, cached,
        // or freshly learned.
        if let Some(graph) = &cpdag
            && graph.num_vertices() != num_vars
        {
            return Err(BrcdLoadError::DimensionMismatch(format!(
                "CPDAG has {} vertices but the datasets have {} variables",
                graph.num_vertices(),
                num_vars
            )));
        }

        Ok(BrcdInput::new(
            normal,
            anomalous,
            cpdag,
            config.brcd_config().clone(),
        ))
    }
}

/// Loads a single dense matrix file into the pipeline precision `T`, validating
/// that it is a 2-D matrix.
fn load_matrix<T: Precision>(
    path: &str,
    csv: &CsvConfig,
) -> Result<CausalTensor<T>, BrcdLoadError> {
    let cfg = DataLoaderConfig::Csv(csv.clone());
    let raw = CsvDataLoader.load(path, &cfg)?;
    if raw.shape().len() != 2 {
        return Err(BrcdLoadError::DimensionMismatch(format!(
            "dataset '{}' is not a 2-D matrix (shape {:?})",
            path,
            raw.shape()
        )));
    }
    Ok(cast_loaded_tensor::<T>(raw))
}
