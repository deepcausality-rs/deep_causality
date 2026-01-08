/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    AnalyzeError, CausalDiscoveryError, CdlError, DataCleaningError, DataLoadingError,
    FeatureSelectError, FinalizeError, PreprocessError,
};
use deep_causality_tensor::CausalTensorError;
use std::error::Error;

#[test]
fn test_display() {
    // Variants wrapping other errors
    let data_err = DataLoadingError::FileNotFound("path/to/file.csv".to_string());
    let err = CdlError::ReadDataError(data_err);
    assert_eq!(
        err.to_string(),
        "Step [Data Loading] failed: File not found at path: path/to/file.csv"
    );

    let preprocess_err = PreprocessError::BinningError("binning failed".to_string());
    let err = CdlError::PreprocessError(preprocess_err);
    assert_eq!(
        err.to_string(),
        "Step [Preprocessing] failed: Binning error: binning failed"
    );

    let feat_select_err = FeatureSelectError::TooFewFeatures(5, 2);
    let err = CdlError::FeatSelectError(feat_select_err);
    assert_eq!(
        err.to_string(),
        "Step [Feature Selection] failed: Too few features available. Need at least 5, but found 2."
    );

    let causal_discovery_err = CausalDiscoveryError::TensorError(CausalTensorError::ShapeMismatch);
    let err = CdlError::CausalDiscoveryError(causal_discovery_err);
    assert_eq!(
        err.to_string(),
        "Step [Causal Discovery] failed: Tensor error during SURD: CausalTensorError: Shape mismatch error"
    );

    let analyze_err = AnalyzeError::EmptyResult;
    let err = CdlError::AnalyzeError(analyze_err);
    assert_eq!(
        err.to_string(),
        "Step [Analysis] failed: The causal discovery result is empty."
    );

    let finalize_err = FinalizeError::FormattingError("format failed".to_string());
    let err = CdlError::FinalizeError(finalize_err);
    assert_eq!(
        err.to_string(),
        "Step [Finalization] failed: Formatting error: format failed"
    );

    let clean_data_err = DataCleaningError::TensorError(CausalTensorError::InvalidOperation);
    let err = CdlError::CleanDataError(clean_data_err);
    assert_eq!(
        err.to_string(),
        "Step [Cleaning] failed: DataCleaningError: Tensor Error: CausalTensorError: Invalid operation error"
    );

    // Missing config variants
    let err = CdlError::MissingDataLoaderConfig;
    assert_eq!(
        err.to_string(),
        "Missing data loader configuration. Please provide a DataLoaderConfig."
    );

    let err = CdlError::MissingFeatureSelectorConfig;
    assert_eq!(
        err.to_string(),
        "Missing feature selector configuration. Please provide a FeatureSelectorConfig."
    );

    let err = CdlError::MissingCausalDiscoveryConfig;
    assert_eq!(
        err.to_string(),
        "Missing causal discovery configuration. Please provide a CausalDiscoveryConfig."
    );

    let err = CdlError::MissingAnalyzeConfig;
    assert_eq!(
        err.to_string(),
        "Missing analysis configuration. Please provide an AnalyzeConfig."
    );

    let err = CdlError::MissingFinalizeConfig;
    assert_eq!(
        err.to_string(),
        "Missing finalization configuration. Please provide a FinalizeConfig."
    );
}

#[test]
fn test_source() {
    // Variants wrapping other errors
    let data_err = DataLoadingError::FileNotFound("path".to_string());
    let err = CdlError::ReadDataError(data_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "File not found at path: path"
    );

    let preprocess_err = PreprocessError::BinningError("binning failed".to_string());
    let err = CdlError::PreprocessError(preprocess_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "Binning error: binning failed"
    );

    let feat_select_err = FeatureSelectError::TooFewFeatures(5, 2);
    let err = CdlError::FeatSelectError(feat_select_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "Too few features available. Need at least 5, but found 2."
    );

    let causal_discovery_err = CausalDiscoveryError::TensorError(CausalTensorError::ShapeMismatch);
    let err = CdlError::CausalDiscoveryError(causal_discovery_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "Tensor error during SURD: CausalTensorError: Shape mismatch error"
    );

    let analyze_err = AnalyzeError::EmptyResult;
    let err = CdlError::AnalyzeError(analyze_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "The causal discovery result is empty."
    );

    let finalize_err = FinalizeError::FormattingError("format failed".to_string());
    let err = CdlError::FinalizeError(finalize_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "Formatting error: format failed"
    );

    let clean_data_err = DataCleaningError::TensorError(CausalTensorError::InvalidOperation);
    let err = CdlError::CleanDataError(clean_data_err);
    assert!(err.source().is_some());
    assert_eq!(
        err.source().unwrap().to_string(),
        "DataCleaningError: Tensor Error: CausalTensorError: Invalid operation error"
    );

    // Missing config variants (should return None)
    let err = CdlError::MissingDataLoaderConfig;
    assert!(err.source().is_none());
    let err = CdlError::MissingFeatureSelectorConfig;
    assert!(err.source().is_none());
    let err = CdlError::MissingCausalDiscoveryConfig;
    assert!(err.source().is_none());
    let err = CdlError::MissingAnalyzeConfig;
    assert!(err.source().is_none());
    let err = CdlError::MissingFinalizeConfig;
    assert!(err.source().is_none());
}

#[test]
fn test_from_impls() {
    // From<DataError>
    let data_err = DataLoadingError::FileNotFound("path".to_string());
    let err = CdlError::from(data_err);
    if let CdlError::ReadDataError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for DataError");
    }

    // From<PreprocessError>
    let preprocess_err = PreprocessError::ConfigError("config error".to_string());
    let err = CdlError::from(preprocess_err);
    if let CdlError::PreprocessError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for PreprocessError");
    }

    // From<FeatureSelectError>
    let feat_select_err = FeatureSelectError::MrmrError(
        deep_causality_algorithms::mrmr::MrmrError::InvalidInput("failed".to_string()),
    );
    let err = CdlError::from(feat_select_err);
    if let CdlError::FeatSelectError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for FeatureSelectError");
    }

    // From<CausalDiscoveryError>
    let causal_discovery_err =
        CausalDiscoveryError::TensorError(CausalTensorError::InvalidOperation);
    let err = CdlError::from(causal_discovery_err);
    if let CdlError::CausalDiscoveryError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for CausalDiscoveryError");
    }

    // From<AnalyzeError>
    let analyze_err = AnalyzeError::AnalysisFailed("failed".to_string());
    let err = CdlError::from(analyze_err);
    if let CdlError::AnalyzeError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for AnalyzeError");
    }

    // From<FinalizeError>
    let finalize_err = FinalizeError::FormattingError("failed".to_string());
    let err = CdlError::from(finalize_err);
    if let CdlError::FinalizeError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for FinalizeError");
    }

    // From<DataCleaningError>
    let clean_data_err = DataCleaningError::TensorError(CausalTensorError::InvalidOperation);
    let err = CdlError::from(clean_data_err);
    if let CdlError::CleanDataError(_) = err {
        // Test passed
    } else {
        panic!("Incorrect error variant for DataCleaningError");
    }
}
