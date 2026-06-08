/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AnalyzeError, BrcdAnalyzeConfig, Precision, ProcessAnalysis, ProcessResultAnalyzer};
use deep_causality_algorithms::brcd::BrcdResult;
use deep_causality_num::ToPrimitive;

/// An implementation of `ProcessResultAnalyzer` for `BrcdResult`.
///
/// This analyzer renders the top-ranked candidate root-cause sets and their
/// posterior weights into a human-readable analysis report.
pub struct BrcdResultAnalyzer;

impl<T: Precision + ToPrimitive> ProcessResultAnalyzer<T> for BrcdResultAnalyzer {
    type Input = BrcdResult<T>;
    type Config = BrcdAnalyzeConfig;

    fn analyze(
        &self,
        brcd_result: &BrcdResult<T>,
        config: &BrcdAnalyzeConfig,
    ) -> Result<ProcessAnalysis, AnalyzeError> {
        let mut messages = Vec::new();
        messages.push("--- BRCD Root-Cause Analysis ---".to_string());

        let ranks = brcd_result.ranks();
        let posterior = brcd_result.posterior();

        if ranks.is_empty() {
            messages.push("  No candidate root causes were ranked.".to_string());
            return Ok(ProcessAnalysis(messages));
        }

        // Bound by both lengths: a ranks/posterior length mismatch must not make
        // the rendered count disagree with the reported `Top {k}` header.
        let available = ranks.len().min(posterior.len());
        let top_k = config.top_k().min(available);
        messages.push(format!(
            "\nTop {} candidate root-cause set(s) by descending posterior:",
            top_k
        ));

        for (rank, (candidate, weight)) in
            ranks.iter().zip(posterior.iter()).take(top_k).enumerate()
        {
            messages.push(format!(
                "  {}. {{{}}}  posterior={:.4}",
                rank + 1,
                format_candidate(candidate),
                weight.to_f64().unwrap_or(f64::NAN)
            ));
        }

        Ok(ProcessAnalysis(messages))
    }
}

// Formats a candidate root-cause set (variable indices) for display.
fn format_candidate(vars: &[usize]) -> String {
    if vars.is_empty() {
        "∅".to_string()
    } else {
        vars.iter()
            .map(|&i| format!("V{}", i))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::format_candidate;

    #[test]
    fn test_format_candidate_empty() {
        assert_eq!(format_candidate(&[]), "∅");
    }

    #[test]
    fn test_format_candidate_non_empty() {
        assert_eq!(format_candidate(&[0]), "V0");
        assert_eq!(format_candidate(&[2, 5]), "V2, V5");
    }
}
