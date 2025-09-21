/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AnalyzeConfig;
use crate::errors::{AnalyzeError, FinalizeError};
use deep_causality_algorithms::surd::SurdResult;
use std::fmt::Display;

pub struct ProcessAnalysis(pub Vec<String>);

// Placeholder
#[derive(Debug)]
pub struct ProcessFormattedResult(pub String);

impl Display for ProcessFormattedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
