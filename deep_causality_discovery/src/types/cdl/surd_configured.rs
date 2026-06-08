/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{SurdConfigured, SurdData};
use crate::types::data_loader::cast::cast_loaded_tensor;
use crate::{
    CDL, CdlBuilder, CdlEffect, CdlError, CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig,
    Precision,
};
use deep_causality_tensor::CausalTensor;

// SURD entry state: load the dataset named by the carried run config.
impl<T: Precision> CDL<SurdConfigured<T>> {
    /// Loads the SURD dataset from the run config (path / target / exclude / CSV
    /// options), casting it to the pipeline precision `T`.
    pub fn surd_load_input(self) -> CdlEffect<CDL<SurdData<T>>> {
        let config = self.state.config;

        // Reuse the CSV loader with the config's parse options. The CSV config's
        // file_path is irrelevant here; the loader opens `config.path()` directly.
        let csv: CsvConfig = config.csv().clone();
        let csv = CsvConfig::new(
            csv.has_headers(),
            csv.delimiter(),
            csv.skip_rows(),
            csv.columns().clone(),
            Some(config.path().to_string()),
            Some(config.target_index()),
            config.exclude_indices().to_vec(),
        );
        let loader_config = DataLoaderConfig::Csv(csv);

        let load_result: Result<CausalTensor<f64>, CdlError> = CsvDataLoader
            .load(config.path(), &loader_config)
            .map_err(Into::into);

        match load_result {
            Ok(tensor) => {
                let records_count = tensor.shape()[0];
                CdlBuilder::pure(CDL {
                    state: SurdData {
                        tensor: cast_loaded_tensor::<T>(tensor),
                        records_count,
                        config,
                    },
                })
            }
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision> CdlEffect<CDL<SurdConfigured<T>>> {
    /// See [`CDL::<SurdConfigured<T>>::surd_load_input`].
    pub fn surd_load_input(self) -> CdlEffect<CDL<SurdData<T>>> {
        self.and_then(|cdl| cdl.surd_load_input())
    }
}
