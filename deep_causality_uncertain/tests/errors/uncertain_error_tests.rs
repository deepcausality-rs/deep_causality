/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{
    BernoulliDistributionError, NormalDistributionError, UniformDistributionError,
};
use deep_causality_uncertain::UncertainError;
use rusty_fork::rusty_fork_test;
use std::error::Error;

rusty_fork_test! {
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
    fn test_from_uniform_error() {
        let err = UniformDistributionError::EmptyRange;
        let uncertain_error: UncertainError = err.into();
        match uncertain_error {
            UncertainError::UniformDistributionError(msg) => {
                assert!(msg.contains("low > high (or equal if exclusive)"));
            }
            _ => panic!("Expected UniformDistributionError"),
        }
    }

    #[test]
    fn test_from_bernoulli_error() {
        let err = BernoulliDistributionError::InvalidProbability;
        let uncertain_error: UncertainError = err.into();
        match uncertain_error {
            UncertainError::BernoulliDistributionError(msg) => {
                assert!(msg.contains("p is outside [0, 1] in Bernoulli distribution"));
            }
            _ => panic!("Expected BernoulliDistributionError"),
        }
    }

    #[test]
    fn test_from_normal_error() {
        let err = NormalDistributionError::MeanTooSmall;
        let uncertain_error: UncertainError = err.into();
        match uncertain_error {
            UncertainError::NormalDistributionError(msg) => {
                assert!(msg.contains("mean < 0 or NaN in log-normal"));
            }
            _ => panic!("Expected NormalDistributionError"),
        }
    }

}
