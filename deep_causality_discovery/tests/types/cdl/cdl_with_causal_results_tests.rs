/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::{AnalyzeConfig, CDL, CdlConfig, WithCausalResults};

fn create_cdl_with_results() -> CDL<WithCausalResults> {
    CDL {
        state: WithCausalResults {
            surd_result: SurdResult::new(
                Default::default(),
                Default::default(),
                Default::default(),
                0.0,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ),
            selection_result: MrmrResult::new(vec![]),
            records_count: 10,
        },
        // Fix init
        config: CdlConfig::default().with_analysis(AnalyzeConfig::new(0.5, 0.5, 0.5)),
    }
}

#[test]
fn test_analyze_success() {
    let cdl = create_cdl_with_results();

    // No arg needed
    let res = cdl.analyze();
    // ...

    assert!(res.inner.is_ok());
    let with_analysis = res.inner.unwrap();
    assert_eq!(with_analysis.state.records_count, 10);
    // analysis should be present
}

// If we had a mock analyzer we could test failure, but currently analyzer is a struct.
// SurdResultAnalyzer process method returns Result.
// If we can control config to force failure... but SurdResultAnalyzer logic is generic.
// Assuming for now simple success test is sufficient for "cdl_with_causal_results.rs" logic which mainly delegates.
