/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::{
    AnalyzeConfig, CDL, CausalDiscoveryConfig, CdlConfig, ConsoleFormatter, CsvConfig,
    CsvDataLoader, DataLoaderConfig, FeatureSelectorConfig, MrmrConfig, MrmrFeatureSelector,
    SurdCausalDiscovery, SurdConfig, SurdResultAnalyzer,
};
use std::fs::File;
use std::io::Write;

fn main() {
    // 1. Prepare test data
    let file_path = get_test_csv_file_path();
    // 2. Get the CDL configuration
    let cdl_config = get_cdl_config();

    // 3. Build the CDL pipeline
    let discovery_process = CDL::with_config(cdl_config)
        .start(CsvDataLoader, &file_path)
        .expect("Failed to load file to start CDL process")
        .feat_select(MrmrFeatureSelector)
        .expect("Failed to select features")
        .causal_discovery(SurdCausalDiscovery)
        .expect("CausalDiscovery failed")
        .analyze(SurdResultAnalyzer)
        .expect("Analysis failed")
        .finalize(ConsoleFormatter)
        .expect("Finalization failed")
        .build()
        .expect("Failed to build causal discovery process");

    // 4. Run the CDL pipeline
    let result = discovery_process.run();
    dbg!(&result);

    if let Err(e) = &result {
        println!("Test failed with error: {}", e);
    }
    assert!(result.is_ok());

    // 5. Print the result
    let res = result.unwrap();
    println!("Result: {}", res);

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
}

fn get_cdl_config() -> CdlConfig {
    CdlConfig::new()
        // Define the data loader as CSV file loader and the corresponding CSV config
        .with_data_loader_config(DataLoaderConfig::Csv(CsvConfig::new(true, b',', 0, None)))
        // Define the feature selected as MRMR and set its parameters
        .with_feature_selector_config(FeatureSelectorConfig::Mrmr(MrmrConfig::new(2, 3)))
        // Define the causal discovery as SURD and set its parameters
        .with_causal_discovery_config(CausalDiscoveryConfig::Surd(SurdConfig::new(
            MaxOrder::Max,
            3,
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
