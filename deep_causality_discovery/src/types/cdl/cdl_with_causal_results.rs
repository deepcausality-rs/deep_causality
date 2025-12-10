/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithAnalysis, WithCausalResults};
use crate::{AnalyzeConfig, CDL, CdlBuilder, CdlEffect, ProcessResultAnalyzer, SurdResultAnalyzer};

// After causal discovery is performed
impl CDL<WithCausalResults> {
    /// Analyzes the raw results.
    /// Uses default configuration or internal config if present.
    pub fn analyze(self) -> CdlEffect<CDL<WithAnalysis>> {
        // Use existing Analyzer logic
        let analyzer = SurdResultAnalyzer;

        // Use config from state or default
        let analyze_config = self
            .config
            .analyze_config()
            .clone()
            .unwrap_or_else(|| AnalyzeConfig::new(0.01, 0.01, 0.01));

        let analysis_res = analyzer.analyze(&self.state.surd_result, &analyze_config);

        match analysis_res {
            Ok(analysis) => CdlBuilder::pure(CDL {
                state: WithAnalysis {
                    analysis,
                    surd_result: self.state.surd_result,
                    selection_result: self.state.selection_result,
                    records_count: self.state.records_count,
                },
                config: self.config,
            }),
            Err(e) => CdlEffect {
                inner: Err(e.into()), // AnalyzeError -> CdlError
                warnings: Default::default(),
            },
        }
    }
}
