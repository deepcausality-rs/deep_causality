/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::ProcessResultAnalyzer;
use deep_causality_discovery::{AnalyzeConfig, SurdResultAnalyzer};
use std::collections::HashMap;

// Helper function to create a default SurdResult for testing
fn create_test_surd_result(
    info_leak: f64,
    synergistic_data: HashMap<Vec<usize>, f64>,
    mutual_data: HashMap<Vec<usize>, f64>,
    redundant_data: HashMap<Vec<usize>, f64>,
) -> SurdResult<f64> {
    SurdResult::new(
        redundant_data,
        synergistic_data,
        mutual_data,
        info_leak,
        HashMap::new(), // causal_redundant_states
        HashMap::new(), // causal_unique_states
        HashMap::new(), // causal_synergistic_states
        HashMap::new(), // non_causal_redundant_states
        HashMap::new(), // non_causal_unique_states
        HashMap::new(), // non_causal_synergistic_states
    )
}

// Helper function to format variable indices for display (copied from source for testing)
fn format_variables(vars: &[usize]) -> String {
    if vars.is_empty() {
        "Target".to_string() // Should not happen for source variables
    } else {
        vars.iter()
            .map(|&i| format!("S{}", i))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[test]
fn test_format_variables_empty() {
    assert_eq!(format_variables(&[]), "Target");
}

#[test]
fn test_format_variables_single() {
    assert_eq!(format_variables(&[0]), "S0");
}

#[test]
fn test_format_variables_multiple() {
    assert_eq!(format_variables(&[0, 1, 2]), "S0, S1, S2");
}

#[test]
fn test_analyze_high_info_leak_no_influences() {
    let surd_result = create_test_surd_result(
        0.6, // High info leak
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    let config = AnalyzeConfig::new(0.5, 0.5, 0.5);
    let analyzer = SurdResultAnalyzer;

    let analysis = analyzer.analyze(&surd_result, &config).unwrap();
    let output = analysis.0.join("\n");
    dbg!(&output);

    assert!(output.contains("--- Causal Analysis Report ---"));
    assert!(output.contains("Information Leak: 0.600 bits"));
    assert!(output.contains(
        "  (High information leak suggests significant unobserved factors or randomness.)"
    ));
    assert!(output.contains("No strong synergistic influences found above threshold."));
    assert!(output.contains("No strong unique influences found above threshold."));
    assert!(output.contains("No strong redundant influences found above threshold."));
}

#[test]
fn test_analyze_low_info_leak_no_influences() {
    let surd_result = create_test_surd_result(
        0.3, // Low info leak
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    let config = AnalyzeConfig::new(0.5, 0.5, 0.5);
    let analyzer = SurdResultAnalyzer;

    let analysis = analyzer.analyze(&surd_result, &config).unwrap();
    let output = analysis.0.join("\n");

    // Note: Information Leak output was removed from SurdResultAnalyzer
    assert!(output.contains("--- Causal Analysis Report ---"));
    assert!(output.contains("No strong synergistic influences found above threshold."));
    assert!(output.contains("No strong unique influences found above threshold."));
    assert!(output.contains("No strong redundant influences found above threshold."));
}

#[test]
fn test_analyze_with_all_strong_influences() {
    let mut synergistic_data = HashMap::new();
    synergistic_data.insert(vec![0, 1], 0.6);

    let mut mutual_data = HashMap::new();
    mutual_data.insert(vec![0], 0.7);
    mutual_data.insert(vec![1], 0.4);
    mutual_data.insert(vec![0, 1], 0.8);

    let mut redundant_data = HashMap::new();
    redundant_data.insert(vec![0, 1], 0.6);

    let surd_result = create_test_surd_result(
        0.4, // Low info leak
        synergistic_data,
        mutual_data,
        redundant_data,
    );
    let config = AnalyzeConfig::new(0.5, 0.5, 0.5);
    let analyzer = SurdResultAnalyzer;

    let analysis = analyzer.analyze(&surd_result, &config).unwrap();
    let output = analysis.0.join("\n");
    dbg!(&output);

    assert!(output.contains("Information Leak: 0.400 bits"));
    assert!(output.contains(
        "  (Low information leak suggests observed factors explain most of the target's behavior.)"
    ));

    assert!(output.contains("Strong synergy from {S0, S1}: 0.600 bits."));
    assert!(!output.contains("Strong synergy from {S0, S2}: 0.300 bits."));

    assert!(output.contains("Strong unique influence from {S0}: 0.700 bits."));
    assert!(!output.contains("Strong unique influence from {S1}: 0.400 bits."));
    assert!(!output.contains("Strong unique influence from {S0, S1}: 0.800 bits."));

    assert!(output.contains("Strong redundant influence from {S0, S1}: 0.600 bits."));
    assert!(!output.contains("Strong redundant influence from {S0, S2}: 0.300 bits."));
}

#[test]
fn test_analyze_with_mixed_influences_and_thresholds() {
    let mut synergistic_data = HashMap::new();
    synergistic_data.insert(vec![0, 1], 0.8);
    synergistic_data.insert(vec![0, 2], 0.6);

    let mut mutual_data = HashMap::new();
    mutual_data.insert(vec![0], 0.9);
    mutual_data.insert(vec![1], 0.7);

    let mut redundant_data = HashMap::new();
    redundant_data.insert(vec![0, 1], 0.8);
    redundant_data.insert(vec![0, 2], 0.6);

    let surd_result = create_test_surd_result(
        0.7, // High info leak
        synergistic_data,
        mutual_data,
        redundant_data,
    );
    let config = AnalyzeConfig::new(0.75, 0.85, 0.75); // Higher thresholds
    let analyzer = SurdResultAnalyzer;

    let analysis = analyzer.analyze(&surd_result, &config).unwrap();
    let output = analysis.0.join("\n");
    dbg!(&output);

    assert!(output.contains("Information Leak: 0.700 bits"));
    assert!(output.contains(
        "  (High information leak suggests significant unobserved factors or randomness.)"
    ));

    assert!(output.contains("Strong synergy from {S0, S1}: 0.800 bits."));
    assert!(!output.contains("No strong synergistic influences found above threshold.")); // Because there is one strong synergy

    assert!(output.contains("Strong unique influence from {S0}: 0.900 bits."));
    assert!(!output.contains("No strong unique influences found above threshold."));

    assert!(output.contains("Strong redundant influence from {S0, S1}: 0.800 bits."));
    assert!(!output.contains("No strong redundant influences found above threshold.")); // Because there is one strong redundancy
}
