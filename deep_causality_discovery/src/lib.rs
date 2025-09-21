/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod traits;
mod types;
mod utils;

// Errors
pub use crate::errors::{
    AnalyzeError, CausalDiscoveryError, CdlError, DataError, FeatureSelectError, FinalizeError,
};

// Traits
pub use crate::traits::causal_discovery::CausalDiscovery;
pub use crate::traits::feature_selector::FeatureSelector;
pub use crate::traits::process_data_loader::ProcessDataLoader;
pub use crate::traits::process_result::{
    ProcessAnalysis, ProcessFormattedResult, ProcessResultAnalyzer, ProcessResultFormatter,
};

// Types
pub use crate::types::causal_discovery::surd::SurdCausalDiscovery;
pub use crate::types::cdl::*;
pub use crate::types::config::*;
pub use crate::types::data_loader::csv::CsvDataLoader;
pub use crate::types::data_loader::parquet::ParquetDataLoader;
pub use crate::types::feature_selector::mrmr::MrmrFeatureSelector;
pub use crate::types::formatter::console_formatter::ConsoleFormatter;
pub use types::analysis::surd_result_analyzer::SurdResultAnalyzer;
