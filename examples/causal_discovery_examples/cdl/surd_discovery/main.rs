/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use causal_discovery_examples::cdl_data::write_surd_test_csv;
use deep_causality_discovery::*;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. The entire CDL pipeline runs at this precision.
pub type FloatType = deep_causality_num::Float106;

fn main() {
    // 1. Get (random test) data.
    let file_path = write_surd_test_csv();

    // 2. Build the SURD run config — the single source of truth. Every required
    //    parameter is explicit; `build()` checks the file exists and is compile-
    //    checked for completeness. Precision `T` is pinned here, once.
    let config = CdlConfigBuilder::build_surd_config::<FloatType>()
        .with_path(&file_path)
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()
        .expect("valid SURD config (file exists)");

    // 3. Run the SURD lineage. All parameters come from the config, so the DSL is
    //    free of inline knobs and reads as the pipeline it is.
    CdlBuilder::build_surd(&config)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner)
        .feature_select()
        .surd_discover()
        .surd_analyze()
        .finalize()
        .print_results();

    // Cleanup
    std::fs::remove_file(file_path).unwrap();
}
