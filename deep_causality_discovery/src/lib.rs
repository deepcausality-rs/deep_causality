/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Causal Discovery Language (CDL)
//!
//! CDL hosts two discovery algorithms — SURD and BRCD — as two compile-time-isolated
//! typestate lineages that converge on a shared analyze/finalize tail. A lineage is
//! seeded by [`CdlBuilder::build_surd`] / [`CdlBuilder::build_brcd`] from a run config
//! built by [`CdlConfigBuilder`].
//!
//! ## Compile-time lineage isolation
//!
//! Each algorithm's stage methods exist only on its own states, so crossing the
//! lineages does not compile. Calling a BRCD stage on a SURD state:
//!
//! ```compile_fail
//! use deep_causality_discovery::{CDL, SurdData};
//! fn cross(c: CDL<SurdData<f64>>) {
//!     let _ = c.brcd_discover(); // error[E0599]: no method named `brcd_discover`
//! }
//! ```
//!
//! Calling a SURD analyze on a BRCD state:
//!
//! ```compile_fail
//! use deep_causality_discovery::{CDL, BrcdLoaded};
//! fn cross(c: CDL<BrcdLoaded<f64>>) {
//!     let _ = c.surd_analyze(); // error[E0599]: no method named `surd_analyze`
//! }
//! ```
//!
//! Feature selection is unavailable to the BRCD lineage:
//!
//! ```compile_fail
//! use deep_causality_discovery::{CDL, BrcdConfigured};
//! fn cross(c: CDL<BrcdConfigured<f64>>) {
//!     let _ = c.feature_select(); // error[E0599]: no method named `feature_select`
//! }
//! ```

mod errors;
mod traits;
mod types;

// re-exports — reused feature-selection / SURD algorithm surface
pub use deep_causality_algorithms::mrmr::mrmr_features_selector;
pub use deep_causality_algorithms::surd::MaxOrder;
pub use deep_causality_algorithms::surd::surd_states_cdl;

// re-exports — reused BRCD algorithm surface (single source of truth)
pub use deep_causality_algorithms::brcd::{
    BrcdConfig, BrcdError, BrcdResult, FamilyKind, brcd_run,
};
// re-export — graph type used to build / carry a CPDAG
pub use deep_causality_topology::{Mark, MixedGraph};

// Errors
pub use crate::errors::{
    AnalyzeError, BrcdLoadError, CausalDiscoveryError, CdlError, CpdagError, DataCleaningError,
    DataLoadingError, FeatureSelectError, FinalizeError, PreprocessError,
};

// Traits
pub use crate::traits::causal_discovery::CausalDiscovery;
pub use crate::traits::data_cleaner::DataCleaner;
pub use crate::traits::data_loader::DataLoader;
pub use crate::traits::data_preprocessor::DataPreprocessor;
pub use crate::traits::feature_selector::FeatureSelector;
pub use crate::traits::precision::Precision;
pub use crate::traits::process_result::{
    ProcessAnalysis, ProcessFormattedResult, ProcessResultAnalyzer, ProcessResultFormatter,
};

// Types
pub use crate::types::analysis::brcd_result_analyzer::BrcdResultAnalyzer;
pub use crate::types::analysis::surd_result_analyzer::SurdResultAnalyzer;
pub use crate::types::brcd_input::BrcdInput;
pub use crate::types::causal_discovery::surd::SurdCausalDiscovery;
pub use crate::types::cdl::*;
pub use crate::types::cdl_discovery_outcome::CdlDiscoveryOutcome;
pub use crate::types::config::*;
pub use crate::types::data_cleaner::option_none::OptionNoneDataCleaner;
pub use crate::types::data_loader::cpdag_csv::{load_cpdag_csv, save_cpdag_csv};
pub use crate::types::data_loader::csv::CsvDataLoader;
pub use crate::types::data_loader::parquet::ParquetDataLoader;
pub use crate::types::data_preprocessor::data_discretizer::DataDiscretizer;
pub use crate::types::data_preprocessor::missing_value_imputer::MissingValueImputer;
pub use crate::types::feature_selector::mrmr::MrmrFeatureSelector;
pub use crate::types::formatter::console_formatter::ConsoleFormatter;

pub use crate::types::cdl_builder::*;
pub use crate::types::cdl_effect::*;
pub use crate::types::cdl_report::*;
pub use crate::types::cdl_warning::*;
