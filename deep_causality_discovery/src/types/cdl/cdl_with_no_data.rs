use crate::types::cdl::WithData;
use crate::types::cdl_effect::CdlBuilder;
use crate::{
    CDL, CdlEffect, CdlError, CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig,
    DataLoadingError, NoData, ParquetConfig, ParquetDataLoader,
};
use deep_causality_tensor::CausalTensor;
use std::path::Path;

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
            config: Default::default(),
        }
    }

    /// Starts the pipeline by loading data from the given path.
    pub fn load_data(
        self,
        path: &str,
        target_index: usize,
        exclude_indices: Vec<usize>,
    ) -> CdlEffect<CDL<WithData>> {
        // Simple dispatch based on extension
        let p = Path::new(path);
        let extension = p.extension().and_then(|s| s.to_str()).unwrap_or("");

        // Helper to construct config since DataLoaderConfig doesn't implement Default
        let mut loaded_config = self.config;

        let load_result: Result<CausalTensor<f64>, CdlError> = match extension {
            "csv" => {
                let loader = CsvDataLoader;
                let mut config = CsvConfig::default();
                config = CsvConfig::new(
                    config.has_headers(),
                    config.delimiter(),
                    config.skip_rows(),
                    config.columns().clone(),
                    Some(path.to_string()),
                    Some(target_index),
                    exclude_indices,
                );

                // Update the pipeline config to store the path
                loaded_config =
                    loaded_config.with_data_loader(DataLoaderConfig::Csv(config.clone()));

                loader
                    .load(path, &DataLoaderConfig::Csv(config))
                    .map_err(Into::into)
            }
            "parquet" => {
                let loader = ParquetDataLoader;
                let mut config = ParquetConfig::default();
                config = ParquetConfig::new(
                    config.columns().clone(),
                    config.batch_size(),
                    Some(path.to_string()),
                    Some(target_index),
                    exclude_indices,
                );

                // Update the pipeline config to store the path
                loaded_config =
                    loaded_config.with_data_loader(DataLoaderConfig::Parquet(config.clone()));

                loader
                    .load(path, &DataLoaderConfig::Parquet(config))
                    .map_err(Into::into)
            }
            _ => Err(CdlError::ReadDataError(DataLoadingError::FileNotFound(
                format!("Unsupported file extension: {}", extension),
            ))),
        };

        match load_result {
            Ok(tensor) => {
                let records_count = tensor.shape()[0];
                CdlBuilder::pure(CDL {
                    state: WithData {
                        tensor,
                        records_count,
                    },
                    config: loaded_config, // Use updated config
                })
            }
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: Default::default(),
            },
        }
    }

    /// Starts the pipeline by loading data using a specific `DataLoaderConfig`.
    pub fn load_data_with_config(self, config: DataLoaderConfig) -> CdlEffect<CDL<WithData>> {
        let mut loaded_config = self.config;
        loaded_config = loaded_config.with_data_loader(config.clone());

        let load_result: Result<CausalTensor<f64>, CdlError> = match &config {
            DataLoaderConfig::Csv(c) => {
                let path = c.file_path().cloned().unwrap_or_default();
                if path.is_empty() {
                    return CdlEffect {
                        inner: Err(CdlError::ReadDataError(DataLoadingError::FileNotFound(
                            "File path missing in config".into(),
                        ))),
                        warnings: Default::default(),
                    };
                }

                let loader = CsvDataLoader;
                loader.load(&path, &config).map_err(Into::into)
            }
            DataLoaderConfig::Parquet(c) => {
                let path = c.file_path().cloned().unwrap_or_default();
                if path.is_empty() {
                    return CdlEffect {
                        inner: Err(CdlError::ReadDataError(DataLoadingError::FileNotFound(
                            "File path missing in config".into(),
                        ))),
                        warnings: Default::default(),
                    };
                }

                let loader = ParquetDataLoader;
                loader.load(&path, &config).map_err(Into::into)
            }
        };

        match load_result {
            Ok(tensor) => {
                let records_count = tensor.shape()[0];
                CdlBuilder::pure(CDL {
                    state: WithData {
                        tensor,
                        records_count,
                    },
                    config: loaded_config,
                })
            }
            Err(e) => CdlEffect {
                inner: Err(e),
                warnings: Default::default(),
            },
        }
    }
}
