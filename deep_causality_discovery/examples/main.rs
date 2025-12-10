/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_discovery::*;
use std::{fs::File, io::Write};

fn main() {
    // 1. Get (random test) data
    let file_path = get_test_csv_file_path();
    let target_index = 3; // Index of the target column

    // 2. Run the CDL pipeline (Monadic Flow)
    let result_effect = CdlBuilder::build()
        // Load Data with Inline Config. For a more detailed file config, use the load_data_with_config() constructor
        .bind(|cdl| cdl.load_data(&file_path, target_index, vec![]))
        // Clean Data using the OptionNoneDataCleaner
        .bind(|cdl| cdl.clean_data(OptionNoneDataCleaner))
        // Feature Selection using MRMR
        .bind(|cdl| {
            cdl.feature_select(|tensor| {
                //Select top 3 features, target at index 3
                mrmr_features_selector(tensor, 3, target_index)
            })
        })
        // Causal Discovery using SURD
        .bind(|cdl| {
            cdl.causal_discovery(|tensor| {
                // MaxOrder the maximum order of interactions to compute between all variables.
                surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)
            })
        })
        // Analyze Results
        .bind(|cdl| cdl.analyze())
        // Finalize and Format
        .bind(|cdl| cdl.finalize());

    // 3. Print all results
    result_effect.print_results();

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
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
