/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod traits;
mod types;

// re-exports
pub use deep_causality_algorithms::mrmr::mrmr_features_selector;
pub use deep_causality_algorithms::surd::MaxOrder;
pub use deep_causality_algorithms::surd::surd_states_cdl;

// Errors
pub use crate::errors::{
    AnalyzeError, CausalDiscoveryError, CdlError, DataCleaningError, DataLoadingError,
    FeatureSelectError, FinalizeError, PreprocessError,
};

// Traits
pub use crate::traits::causal_discovery::CausalDiscovery;
pub use crate::traits::data_cleaner::DataCleaner;
pub use crate::traits::data_loader::DataLoader;
pub use crate::traits::data_preprocessor::DataPreprocessor;
pub use crate::traits::feature_selector::FeatureSelector;
pub use crate::traits::process_result::{
    ProcessAnalysis, ProcessFormattedResult, ProcessResultAnalyzer, ProcessResultFormatter,
};

// Types
pub use crate::types::analysis::surd_result_analyzer::SurdResultAnalyzer;
pub use crate::types::causal_discovery::surd::SurdCausalDiscovery;
pub use crate::types::cdl::*;
pub use crate::types::config::*;
pub use crate::types::data_cleaner::option_none::OptionNoneDataCleaner;
pub use crate::types::data_loader::csv::CsvDataLoader;
pub use crate::types::data_loader::parquet::ParquetDataLoader;
pub use crate::types::data_preprocessor::data_discretizer::DataDiscretizer;
pub use crate::types::data_preprocessor::missing_value_imputer::MissingValueImputer;
pub use crate::types::feature_selector::mrmr::MrmrFeatureSelector;
pub use crate::types::formatter::console_formatter::ConsoleFormatter;

pub use crate::types::cdl_effect::*;
pub use crate::types::cdl_report::*;
pub use crate::types::cdl_warning::*;
