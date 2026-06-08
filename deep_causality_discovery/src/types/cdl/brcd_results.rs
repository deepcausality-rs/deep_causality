/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::analysis::brcd_result_analyzer::BrcdResultAnalyzer;
use crate::types::cdl::{BrcdResults, WithAnalysis};
use crate::types::cdl_discovery_outcome::CdlDiscoveryOutcome;
use crate::{BrcdAnalyzeConfig, CDL, CdlBuilder, CdlEffect, Precision, ProcessResultAnalyzer};
use deep_causality_num::ToPrimitive;

// After BRCD discovery is performed
impl<T: Precision + ToPrimitive> CDL<BrcdResults<T>> {
    /// Analyzes the BRCD result and converges onto the shared `WithAnalysis` state.
    pub fn brcd_analyze(self) -> CdlEffect<CDL<WithAnalysis<T>>> {
        let analyzer = BrcdResultAnalyzer;
        let analyze_config = BrcdAnalyzeConfig::default();

        let analysis_res = analyzer.analyze(&self.state.brcd_result, &analyze_config);

        match analysis_res {
            Ok(analysis) => CdlBuilder::pure(CDL {
                state: WithAnalysis {
                    analysis,
                    outcome: CdlDiscoveryOutcome::Brcd(self.state.brcd_result),
                    feature_selection: None,
                    records_count: self.state.records_count,
                    dataset_path: "BRCD (normal + anomalous datasets)".to_string(),
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(e.into()),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision + ToPrimitive> CdlEffect<CDL<BrcdResults<T>>> {
    /// See [`CDL::<BrcdResults<T>>::brcd_analyze`].
    pub fn brcd_analyze(self) -> CdlEffect<CDL<WithAnalysis<T>>> {
        self.and_then(|cdl| cdl.brcd_analyze())
    }
}
