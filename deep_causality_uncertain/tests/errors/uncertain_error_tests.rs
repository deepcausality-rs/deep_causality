/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::UncertainError;
use std::error::Error;
// For the Error trait methods
use ultragraph::GraphError;
// For testing From impl

#[test]
fn test_graph_error_display() {
    let err = UncertainError::GraphError("test graph error".to_string());
    assert_eq!(
        err.to_string(),
        "Graph construction error: test graph error"
    );
    assert!(err.source().is_none());
}

#[test]
fn test_confidence_error_display() {
    let err = UncertainError::ConfidenceError("test confidence error".to_string());
    assert_eq!(err.to_string(), "Confidence error: test confidence error");
    assert!(err.source().is_none());
}

#[test]
fn test_unsupported_type_error_display() {
    let err = UncertainError::UnsupportedTypeError("test unsupported type".to_string());
    assert_eq!(err.to_string(), "Unsupported type: test unsupported type");
    assert!(err.source().is_none());
}

#[test]
fn test_bernoulli_distribution_error_display() {
    let err = UncertainError::BernoulliDistributionError("test bernoulli error".to_string());
    assert_eq!(
        err.to_string(),
        "Bernoulli distribution error: test bernoulli error"
    );
    assert!(err.source().is_none());
}

#[test]
fn test_normal_distribution_error_display() {
    let err = UncertainError::NormalDistributionError("test normal error".to_string());
    assert_eq!(
        err.to_string(),
        "Normal distribution error: test normal error"
    );
    assert!(err.source().is_none());
}

#[test]
fn test_uniform_distribution_error_display() {
    let err = UncertainError::UniformDistributionError("test uniform error".to_string());
    assert_eq!(
        err.to_string(),
        "Uniform distribution error: test uniform error"
    );
    assert!(err.source().is_none());
}

#[test]
fn test_sampling_error_display() {
    let err = UncertainError::SamplingError("test sampling error".to_string());
    assert_eq!(err.to_string(), "Sampling error: test sampling error");
    assert!(err.source().is_none());
}

#[test]
fn test_from_ultragraph_error() {
    let ug_err = GraphError::NodeNotFound(5);
    let err: UncertainError = ug_err.into(); // Use into() for From conversion
    assert_eq!(
        err.to_string(),
        "Graph construction error: Node with index 5 not found; it may be out of bounds or have been removed."
    );
    assert!(err.source().is_none());
}
