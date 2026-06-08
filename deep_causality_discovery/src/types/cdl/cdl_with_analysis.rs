/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::WithAnalysis;
use crate::{CDL, CdlBuilder, CdlEffect, CdlReport, Precision};

// After results are analyzed. Both lineages converge here, so `finalize` is
// implemented once and is algorithm-neutral: it packages whatever
// `DiscoveryOutcome` the analyze step produced into a `CdlReport`.
impl<T: Precision> CDL<WithAnalysis<T>> {
    /// Finalizes the pipeline and produces a `CdlReport`.
    pub fn finalize(self) -> CdlEffect<CdlReport<T>> {
        let report = CdlReport {
            dataset_path: self.state.dataset_path,
            records_processed: self.state.records_count,
            feature_selection: self.state.feature_selection,
            causal_analysis: self.state.outcome,
        };

        CdlBuilder::pure(report)
    }
}

// Fluent terminal method on the effect.
impl<T: Precision> CdlEffect<CDL<WithAnalysis<T>>> {
    /// See [`CDL::<WithAnalysis<T>>::finalize`].
    pub fn finalize(self) -> CdlEffect<CdlReport<T>> {
        self.and_then(|cdl| cdl.finalize())
    }
}
