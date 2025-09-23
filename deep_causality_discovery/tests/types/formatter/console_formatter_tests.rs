/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{ConsoleFormatter, ProcessAnalysis, ProcessResultFormatter};

#[test]
fn test_format_empty_analysis() {
    let formatter = ConsoleFormatter;
    let analysis = ProcessAnalysis(vec![]);

    let formatted_result = formatter.format(&analysis).unwrap();
    assert_eq!(formatted_result.0, "");
}

#[test]
fn test_format_single_line_analysis() {
    let formatter = ConsoleFormatter;
    let analysis = ProcessAnalysis(vec!["Line 1".to_string()]);

    let formatted_result = formatter.format(&analysis).unwrap();
    assert_eq!(formatted_result.0, "Line 1\n");
}

#[test]
fn test_format_multiple_line_analysis() {
    let formatter = ConsoleFormatter;
    let analysis = ProcessAnalysis(vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
        "Line 3".to_string(),
    ]);

    let formatted_result = formatter.format(&analysis).unwrap();
    assert_eq!(formatted_result.0, "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn test_format_with_special_characters() {
    let formatter = ConsoleFormatter;
    let analysis = ProcessAnalysis(vec![
        "Line with !@#$%^&*()".to_string(),
        "Line with \t tabs and \n newlines".to_string(),
    ]);

    let formatted_result = formatter.format(&analysis).unwrap();
    assert_eq!(
        formatted_result.0,
        "Line with !@#$%^&*()\nLine with \t tabs and \n newlines\n"
    );
}
