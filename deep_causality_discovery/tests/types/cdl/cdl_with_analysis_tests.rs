/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::{CDL, CdlConfig, ProcessAnalysis, WithAnalysis};

fn create_cdl_with_analysis() -> CDL<WithAnalysis> {
    CDL {
        state: WithAnalysis {
            analysis: ProcessAnalysis(vec![]),
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
            records_count: 5,
        },
        config: CdlConfig::default(),
    }
}

#[test]
fn test_finalize_success() {
    let cdl = create_cdl_with_analysis();
    let res = cdl.finalize(); // No formatter arg based on source read

    assert!(res.inner.is_ok());
    let report = res.inner.unwrap();

    assert_eq!(report.records_processed, 5);
    // Path might be unknown since config is default
    assert!(report.dataset_path.contains("Unknown"));
}
