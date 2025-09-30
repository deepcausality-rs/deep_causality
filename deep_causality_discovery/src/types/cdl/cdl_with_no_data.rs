/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::WithData;
use crate::{CDL, CdlConfig, CdlError, DataLoader, NoData};

// Initial state
impl Default for CDL<NoData> {
    fn default() -> Self {
        Self::new()
    }
}

impl CDL<NoData> {
    /// Creates a new CDL pipeline builder in its initial state with a default configuration.
    pub fn new() -> Self {
        CDL {
            state: NoData,
            config: CdlConfig::new(),
        }
    }

    /// Creates a new CDL pipeline builder in its initial state with a specific configuration.
    pub fn with_config(config: CdlConfig) -> Self {
        CDL {
            state: NoData,
            config,
        }
    }

    /// Starts the pipeline by loading data from the given path.
    ///
    /// # Arguments
    /// * `loader` - An implementation of `ProcessDataLoader` (e.g., `CsvDataLoader`).
    /// * `path` - The path to the data source file.
    ///
    /// # Returns
    /// A `CDL` instance in the `WithData` state, or a `CdlError` if loading fails.
    pub fn load_data<L>(self, loader: L, path: &str) -> Result<CDL<WithData>, CdlError>
    where
        L: DataLoader,
    {
        let loader_config = self
            .config
            .data_loader_config()
            .as_ref()
            .ok_or(CdlError::MissingDataLoaderConfig)?;

        let tensor = loader.load(path, loader_config)?;
        Ok(CDL {
            state: WithData(tensor),
            config: self.config,
        })
    }
}
