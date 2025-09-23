/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::{MaxOrder, SurdResult};
use deep_causality_discovery::{
    AnalyzeConfig, AnalyzeError, CDL, CausalDiscoveryConfig, CausalDiscoveryError, CdlConfig,
    CdlError, CsvConfig, DataError, DataLoaderConfig, FeatureSelectError, FeatureSelectorConfig,
    FinalizeError, MrmrConfig, PreprocessConfig, PreprocessError, ProcessAnalysis,
    ProcessFormattedResult, SurdConfig,
};
use deep_causality_discovery::{
    CausalDiscovery, DataPreprocessor, FeatureSelector, ProcessDataLoader, ProcessResultAnalyzer,
    ProcessResultFormatter,
};
use deep_causality_tensor::{CausalTensor, CausalTensorError};
use std::io::Write;
use tempfile::NamedTempFile;

// --- Mock Implementations for Trait Testing ---

struct MockDataLoader {
    success: bool,
}
impl ProcessDataLoader for MockDataLoader {
    fn load(
        &self,
        _path: &str,
        _config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataError> {
        if self.success {
            Ok(CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap())
        } else {
            Err(DataError::OsError("MockDataLoader failed".to_string()))
        }
    }
}

struct MockPreprocessor {
    success: bool,
}
impl DataPreprocessor for MockPreprocessor {
    fn process(
        &self,
        tensor: CausalTensor<f64>,
        _config: &PreprocessConfig,
    ) -> Result<CausalTensor<f64>, PreprocessError> {
        if self.success {
            Ok(tensor)
        } else {
            Err(PreprocessError::ConfigError(
                "MockPreprocessor failed".to_string(),
            ))
        }
    }
}

struct MockFeatureSelector {
    success: bool,
}
impl FeatureSelector for MockFeatureSelector {
    fn select(
        &self,
        tensor: CausalTensor<f64>,
        _config: &FeatureSelectorConfig,
    ) -> Result<CausalTensor<f64>, FeatureSelectError> {
        if self.success {
            Ok(tensor)
        } else {
            Err(FeatureSelectError::TooFewFeatures(1, 0))
        }
    }
}

struct MockCausalDiscovery {
    success: bool,
}
impl CausalDiscovery for MockCausalDiscovery {
    fn discover(
        &self,
        _tensor: CausalTensor<f64>,
        _config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        if self.success {
            // Create a dummy SurdResult
            Ok(SurdResult::new(
                Default::default(),
                Default::default(),
                Default::default(),
                0.5,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ))
        } else {
            Err(CausalDiscoveryError::TensorError(
                CausalTensorError::EmptyTensor,
            ))
        }
    }
}

struct MockResultAnalyzer {
    success: bool,
}
impl ProcessResultAnalyzer for MockResultAnalyzer {
    fn analyze(
        &self,
        _surd_result: &SurdResult<f64>,
        _config: &AnalyzeConfig,
    ) -> Result<ProcessAnalysis, AnalyzeError> {
        if self.success {
            Ok(ProcessAnalysis(vec!["Analysis result".to_string()]))
        } else {
            Err(AnalyzeError::AnalysisFailed(
                "MockResultAnalyzer failed".to_string(),
            ))
        }
    }
}

struct MockResultFormatter {
    success: bool,
}
impl ProcessResultFormatter for MockResultFormatter {
    fn format(&self, _analysis: &ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError> {
        if self.success {
            Ok(ProcessFormattedResult("Formatted result".to_string()))
        } else {
            Err(FinalizeError::FormattingError(
                "MockResultFormatter failed".to_string(),
            ))
        }
    }
}

// --- Helper for creating dummy CSV file ---
fn create_test_csv_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

// --- Tests for CDL<NoData> ---

#[test]
fn test_cdl_new_and_default() {
    let cdl = CDL::new();
    assert!(cdl.config().data_loader_config().is_none());

    let default_cdl: CDL<_> = Default::default();
    assert!(default_cdl.config().data_loader_config().is_none());
}

#[test]
fn test_cdl_with_config() {
    let custom_config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)));
    let cdl = CDL::with_config(custom_config.clone());
    assert!(cdl.config().data_loader_config().is_some());
    assert_eq!(
        cdl.config()
            .data_loader_config()
            .as_ref()
            .unwrap()
            .to_string(),
        custom_config
            .data_loader_config()
            .as_ref()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_cdl_start_success() {
    let file = create_test_csv_file("1.0,2.0\n3.0,4.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    // Check if state transitioned to WithData
    let _ = cdl.state();
}

#[test]
fn test_cdl_start_error_missing_config() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let cdl = CDL::new(); // No data loader config
    let result = cdl.start(MockDataLoader { success: true }, file_path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CdlError::MissingDataLoaderConfig);
}

#[test]
fn test_cdl_start_error_loader_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)));
    let cdl = CDL::with_config(config).start(MockDataLoader { success: false }, file_path);
    assert!(cdl.is_err());
    assert_eq!(
        cdl.unwrap_err(),
        CdlError::ReadDataError(DataError::OsError("MockDataLoader failed".to_string()))
    );
}

// --- Tests for CDL<WithData> ---

#[test]
fn test_cdl_preprocess_success() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_preprocess_config(PreprocessConfig::new(
            deep_causality_discovery::BinningStrategy::EqualWidth,
            2,
            deep_causality_discovery::ColumnSelector::All,
        ));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl.preprocess(MockPreprocessor { success: true }).unwrap();
    // Check if state remained WithData
    let _ = cdl.state();
}

#[test]
fn test_cdl_preprocess_skipped_no_config() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None))); // No preprocess config
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl.preprocess(MockPreprocessor { success: true }).unwrap();
    // Check if state remained WithData
    let _ = cdl.state();
}

#[test]
fn test_cdl_preprocess_error_preprocessor_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_preprocess_config(PreprocessConfig::new(
            deep_causality_discovery::BinningStrategy::EqualWidth,
            2,
            deep_causality_discovery::ColumnSelector::All,
        ));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let result = cdl.preprocess(MockPreprocessor { success: false });
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CdlError::PreprocessError(PreprocessError::ConfigError(
            "MockPreprocessor failed".to_string()
        ))
    );
}

#[test]
fn test_cdl_feat_select_success() {
    let file = create_test_csv_file("1.0,2.0,3.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    // Check if state transitioned to WithFeatures
    let _ = cdl.state();
}

#[test]
fn test_cdl_feat_select_error_missing_config() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None))); // No feature selector config
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let result = cdl.feat_select(MockFeatureSelector { success: true });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CdlError::MissingFeatureSelectorConfig);
}

#[test]
fn test_cdl_feat_select_error_selector_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let result = cdl.feat_select(MockFeatureSelector { success: false });
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CdlError::FeatSelectError(FeatureSelectError::TooFewFeatures(1, 0))
    );
}

// --- Tests for CDL<WithFeatures> ---

#[test]
fn test_cdl_causal_discovery_success() {
    let file = create_test_csv_file("1.0,2.0,3.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    // Check if state transitioned to WithCausalResults
    let _ = cdl.state();
}

#[test]
fn test_cdl_causal_discovery_error_missing_config() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0))); // No causal discovery config
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let result = cdl.causal_discovery(MockCausalDiscovery { success: true });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CdlError::MissingCausalDiscoveryConfig);
}

#[test]
fn test_cdl_causal_discovery_error_discovery_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let result = cdl.causal_discovery(MockCausalDiscovery { success: false });
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CdlError::CausalDiscoveryError(CausalDiscoveryError::TensorError(
            CausalTensorError::EmptyTensor
        ))
    );
}

// --- Tests for CDL<WithCausalResults> ---

#[test]
fn test_cdl_analyze_success() {
    let file = create_test_csv_file("1.0,2.0,3.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )))
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let cdl = cdl.analyze(MockResultAnalyzer { success: true }).unwrap();
    // Check if state transitioned to WithAnalysis
    let _ = cdl.state();
}

#[test]
fn test_cdl_analyze_error_missing_config() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        ))); // No analyze config
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let result = cdl.analyze(MockResultAnalyzer { success: true });
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CdlError::MissingAnalyzeConfig);
}

#[test]
fn test_cdl_analyze_error_analyzer_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )))
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let result = cdl.analyze(MockResultAnalyzer { success: false });
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CdlError::AnalyzeError(AnalyzeError::AnalysisFailed(
            "MockResultAnalyzer failed".to_string()
        ))
    );
}

// --- Tests for CDL<WithAnalysis> ---

#[test]
fn test_cdl_finalize_success() {
    let file = create_test_csv_file("1.0,2.0,3.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )))
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let cdl = cdl.analyze(MockResultAnalyzer { success: true }).unwrap();
    let cdl = cdl.finalize(MockResultFormatter { success: true }).unwrap();
    // Check if state transitioned to Finalized
    let _ = cdl.state();
}

#[test]
fn test_cdl_finalize_error_formatter_failure() {
    let file = create_test_csv_file("1.0,2.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )))
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let cdl = cdl.analyze(MockResultAnalyzer { success: true }).unwrap();
    let result = cdl.finalize(MockResultFormatter { success: false });
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CdlError::FinalizeError(FinalizeError::FormattingError(
            "MockResultFormatter failed".to_string()
        ))
    );
}

// --- Tests for CDL<Finalized> and CQDRunner ---

#[test]
fn test_cdl_build_and_run_success() {
    let file = create_test_csv_file("1.0,2.0,3.0");
    let file_path = file.path().to_str().unwrap();
    let config = CdlConfig::new()
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(false, b',', 0, None)))
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            0,
        )))
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1));
    let cdl = CDL::with_config(config)
        .start(MockDataLoader { success: true }, file_path)
        .unwrap();
    let cdl = cdl
        .feat_select(MockFeatureSelector { success: true })
        .unwrap();
    let cdl = cdl
        .causal_discovery(MockCausalDiscovery { success: true })
        .unwrap();
    let cdl = cdl.analyze(MockResultAnalyzer { success: true }).unwrap();
    let runner = cdl
        .finalize(MockResultFormatter { success: true })
        .unwrap()
        .build()
        .unwrap();
    let result = runner.run().unwrap();
    assert_eq!(result.to_string(), "Formatted result");
}
