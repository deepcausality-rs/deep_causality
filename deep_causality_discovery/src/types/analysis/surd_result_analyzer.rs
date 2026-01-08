/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ProcessResultAnalyzer;
use crate::{AnalyzeConfig, AnalyzeError, ProcessAnalysis};
use deep_causality_algorithms::surd::SurdResult;

/// An implementation of `ProcessResultAnalyzer` for `SurdResult`.
///
/// This analyzer interprets the raw numerical output of the SURD algorithm,
/// comparing synergistic, unique, and redundant information values against
/// configured thresholds to generate a human-readable analysis report.
pub struct SurdResultAnalyzer;

impl ProcessResultAnalyzer for SurdResultAnalyzer {
    fn analyze(
        &self,
        surd_result: &SurdResult<f64>,
        config: &AnalyzeConfig,
    ) -> Result<ProcessAnalysis, AnalyzeError> {
        let mut messages = Vec::new();

        messages.push("--- Causal Analysis Report ---".to_string());

        // Information Leak
        let info_leak = surd_result.info_leak();
        messages.push(format!("\nInformation Leak: {:.3} bits", info_leak));
        if info_leak >= 0.5 {
            messages.push(
                "  (High information leak suggests significant unobserved factors or randomness.)"
                    .to_string(),
            );
        } else {
            messages.push(
                "  (Low information leak suggests observed factors explain most of the target's behavior.)"
                    .to_string(),
            );
        }

        // Synergistic Information
        messages.push("\n--- Synergistic Influences ---".to_string());
        let mut found_synergy = false;
        for (vars, &value) in surd_result.synergistic_info() {
            if value >= config.synergy_threshold() {
                messages.push(format!(
                    "  Strong synergy from {{{}}}: {:.3} bits.",
                    format_variables(vars),
                    value
                ));
                found_synergy = true;
            }
        }
        if !found_synergy {
            messages.push("  No strong synergistic influences found above threshold.".to_string());
        }

        // Unique Information
        messages.push("\n--- Unique Influences ---".to_string());
        let mut found_unique = false;
        for (vars, &value) in surd_result.mutual_info() {
            if vars.len() == 1 && value >= config.unique_threshold() {
                messages.push(format!(
                    "  Strong unique influence from {{{}}}: {:.3} bits.",
                    format_variables(vars),
                    value
                ));
                found_unique = true;
            }
        }
        if !found_unique {
            messages.push("  No strong unique influences found above threshold.".to_string());
        }

        // Redundant Information
        messages.push("\n--- Redundant Influences ---".to_string());
        let mut found_redundancy = false;
        for (vars, &value) in surd_result.redundant_info() {
            if value >= config.redundancy_threshold() {
                messages.push(format!(
                    "  Strong redundant influence from {{{}}}: {:.3} bits.",
                    format_variables(vars),
                    value
                ));
                found_redundancy = true;
            }
        }
        if !found_redundancy {
            messages.push("  No strong redundant influences found above threshold.".to_string());
        }

        Ok(ProcessAnalysis(messages))
    }
}

// Helper function to format variable indices for display
fn format_variables(vars: &[usize]) -> String {
    if vars.is_empty() {
        "Target Vars Empty".to_string() // Should not happen for source variables
    } else {
        vars.iter()
            .map(|&i| format!("S{}", i))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::format_variables;

    #[test]
    fn test_format_variables_empty() {
        assert_eq!(format_variables(&[]), "Target Vars Empty");
    }

    #[test]
    fn test_format_variables_non_empty() {
        assert_eq!(format_variables(&[0]), "S0");
        assert_eq!(format_variables(&[0, 1]), "S0, S1");
        assert_eq!(format_variables(&[2, 5, 1]), "S2, S5, S1");
    }
}
