/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd;
use deep_causality_discovery::{
    AnalyzeConfig, CDL, CausalDiscoveryConfig, CdlConfig, ConsoleFormatter, CsvConfig,
    CsvDataLoader, DataLoaderConfig, FeatureSelectorConfig, MrmrConfig, MrmrFeatureSelector,
    SurdCausalDiscovery, SurdConfig, SurdResultAnalyzer,
};
use std::fs::File;
use std::io::Write;

#[test]
fn test_full_dsl_pipeline_csv() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Prepare test data
    let file_path = get_test_csv_file_path();
    // 2. Get the CDL configuration
    let cdl_config = get_cdl_config();

    // 3. Run the DSL pipeline
    let discovery_process = CDL::with_config(cdl_config)
        .start(CsvDataLoader, &file_path)?
        .feat_select(MrmrFeatureSelector)?
        .causal_discovery(SurdCausalDiscovery)?
        .analyze(SurdResultAnalyzer)?
        .finalize(ConsoleFormatter)?
        .build()
        .expect("Failed to build causal discovery process");

    let result = discovery_process.run();

    // 4. Assert success
    assert!(result.is_ok());

    // 5. Cleanup
    std::fs::remove_file(file_path).unwrap();

    Ok(())
}

fn get_cdl_config() -> CdlConfig {
    CdlConfig::new()
        // Define the data loader as CSV file loader and the corresponding CSV config
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(true, b',', 0, None)))
        // Define the feature selected as MRMR and set its parameters
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(2, 3)))
        // Define the causal discovery as SURD and set its parameters
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            surd::MaxOrder::Max,
        )))
        // Define the analysis of the SURD results and set its parameters
        .with_analyze_config(AnalyzeConfig::new(0.1, 0.1, 0.1))
}

fn get_test_csv_file_path() -> String {
    // 1. Create a dummy CSV file
    let csv_data =
        "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6";
    let file_path = "./test_data.csv";
    let mut file = File::create(file_path).unwrap();
    file.write_all(csv_data.as_bytes()).unwrap();

    file_path.to_string()
}
