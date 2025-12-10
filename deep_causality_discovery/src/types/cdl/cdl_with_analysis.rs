/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::WithAnalysis;
use crate::{CDL, CdlBuilder, CdlEffect, CdlReport};

// After results are analyzed
impl CDL<WithAnalysis> {
    /// Finalizes the pipeline and produces a CdlReport.
    pub fn finalize(self) -> CdlEffect<CdlReport> {
        // Retrieve path from config
        let path = match self.config.data_loader_config().as_ref() {
            Some(crate::DataLoaderConfig::Csv(c)) => {
                c.file_path().cloned().unwrap_or("Unknown CSV".to_string())
            }
            Some(crate::DataLoaderConfig::Parquet(c)) => c
                .file_path()
                .cloned()
                .unwrap_or("Unknown Parquet".to_string()),
            None => "Unknown Data Source".to_string(),
        };

        let report = CdlReport {
            dataset_path: path,
            records_processed: self.state.records_count,
            feature_selection: self.state.selection_result,
            causal_analysis: self.state.surd_result,
        };

        CdlBuilder::pure(report)
    }
}
