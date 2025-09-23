/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AnalyzeConfig, AnalyzeError, FinalizeError};
use deep_causality_algorithms::surd::SurdResult;
use std::fmt::Display;

/// A wrapper struct holding the results of an analysis as a vector of strings.
///
/// Each string typically represents a human-readable line of the analysis report,
/// such as the interpretation of a specific causal influence.
#[derive(Debug)]
pub struct ProcessAnalysis(pub Vec<String>);

/// A wrapper struct holding the final, formatted output of the CDL pipeline.
///
/// This struct implements the `Display` trait, allowing it to be easily printed
/// to the console or written to a file.
#[derive(Debug)]
pub struct ProcessFormattedResult(pub String);

impl Display for ProcessFormattedResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Defines the contract for analyzing the raw results of a causal discovery algorithm.
///
/// Implementors of this trait translate the numerical results (e.g., from `SurdResult`)
/// into a structured, human-interpretable analysis based on a given configuration.
pub trait ProcessResultAnalyzer {
    /// Analyzes the raw causal discovery results.
    ///
    /// # Arguments
    ///
    /// * `surd_result` - A reference to the `SurdResult<f64>` output from the
    ///   discovery phase.
    /// * `config` - An `AnalyzeConfig` containing thresholds and settings that guide
    ///   the interpretation of the results (e.g., what constitutes a "strong" influence).
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ProcessAnalysis` struct, which holds the lines of the
    /// generated report.
    ///
    /// # Errors
    ///
    /// Returns an `AnalyzeError` if the analysis cannot be completed.
    fn analyze(
        &self,
        surd_result: &SurdResult<f64>,
        config: &AnalyzeConfig,
    ) -> Result<ProcessAnalysis, AnalyzeError>;
}

/// Defines the contract for formatting an analysis into a final output string.
///
/// Implementors of this trait take the structured analysis from a `ProcessResultAnalyzer`
/// and render it into a final presentable format, such as a console-friendly string.
pub trait ProcessResultFormatter {
    /// Formats the processed analysis into a final string representation.
    ///
    /// # Arguments
    ///
    /// * `analysis` - A reference to the `ProcessAnalysis` struct containing the
    ///   interpreted results.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `ProcessFormattedResult`, which wraps the final
    /// formatted string.
    ///
    /// # Errors
    ///
    /// Returns a `FinalizeError` if formatting fails.
    fn format(&self, analysis: &ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError>;
}
