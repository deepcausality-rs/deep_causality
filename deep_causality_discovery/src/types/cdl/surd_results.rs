/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{SurdResults, WithAnalysis};
use crate::types::cdl_discovery_outcome::CdlDiscoveryOutcome;
use crate::{CDL, CdlBuilder, CdlEffect, Precision, ProcessResultAnalyzer, SurdResultAnalyzer};
use deep_causality_num::ToPrimitive;

// After SURD discovery is performed
impl<T: Precision + ToPrimitive> CDL<SurdResults<T>> {
    /// Analyzes the SURD result using the thresholds from the run config and
    /// converges onto the shared `WithAnalysis` state.
    pub fn surd_analyze(self) -> CdlEffect<CDL<WithAnalysis<T>>> {
        let analyzer = SurdResultAnalyzer;
        let config = self.state.config;
        let dataset_path = config.path().to_string();

        let analysis_res = analyzer.analyze(&self.state.surd_result, config.analyze());

        match analysis_res {
            Ok(analysis) => CdlBuilder::pure(CDL {
                state: WithAnalysis {
                    analysis,
                    outcome: CdlDiscoveryOutcome::Surd(Box::new(self.state.surd_result)),
                    feature_selection: Some(self.state.selection_result),
                    records_count: self.state.records_count,
                    dataset_path,
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
impl<T: Precision + ToPrimitive> CdlEffect<CDL<SurdResults<T>>> {
    /// See [`CDL::<SurdResults<T>>::surd_analyze`].
    pub fn surd_analyze(self) -> CdlEffect<CDL<WithAnalysis<T>>> {
        self.and_then(|cdl| cdl.surd_analyze())
    }
}
