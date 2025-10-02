/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithAnalysis, WithCausalResults};
use crate::{CDL, CdlError, ProcessResultAnalyzer};

// After causal discovery is performed
impl CDL<WithCausalResults> {
    /// Analyzes the raw results from the discovery algorithm.
    ///
    /// # Arguments
    /// * `analyzer` - An implementation of `ProcessResultAnalyzer`.
    ///
    /// # Returns
    /// A `CDL` instance in the `WithAnalysis` state, or a `CdlError` if analysis fails.
    pub fn analyze<A>(self, analyzer: A) -> Result<CDL<WithAnalysis>, CdlError>
    where
        A: ProcessResultAnalyzer,
    {
        let analyze_config = self
            .config
            .analyze_config()
            .as_ref()
            .ok_or(CdlError::MissingAnalyzeConfig)?;

        let analysis = analyzer.analyze(&self.state.0, analyze_config)?;
        Ok(CDL {
            state: WithAnalysis(analysis),
            config: self.config,
        })
    }
}
