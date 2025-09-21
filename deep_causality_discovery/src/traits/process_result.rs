/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::AnalyzeConfig;
use crate::errors::{AnalyzeError, FinalizeError};
use deep_causality_algorithms::surd::SurdResult;

pub struct ProcessAnalysis;

// Placeholder
pub struct ProcessFormattedResult(pub String);

pub trait ProcessResultAnalyzer {
    fn analyze(
        &self,
        surd_result: &SurdResult<f64>,
        config: &AnalyzeConfig,
    ) -> Result<ProcessAnalysis, AnalyzeError>;
}

pub trait ProcessResultFormatter {
    fn format(&self, analysis: &ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError>;
}
