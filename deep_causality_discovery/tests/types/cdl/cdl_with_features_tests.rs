/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::{CDL, CdlConfig, WithFeatures};
use deep_causality_tensor::CausalTensor;

fn create_cdl_with_features() -> CDL<WithFeatures> {
    let tensor = CausalTensor::new(vec![Some(1.0), Some(2.0)], vec![2, 1]).unwrap();
    CDL {
        state: WithFeatures {
            tensor,
            selection_result: MrmrResult::new(vec![(0, 0.5)]),
            records_count: 2,
        },
        config: CdlConfig::default(),
    }
}

fn create_dummy_surd_result() -> SurdResult<f64> {
    SurdResult::new(
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
    )
}

#[test]
fn test_causal_discovery_success() {
    let cdl = create_cdl_with_features();

    let res = cdl.causal_discovery(|_t| Ok(create_dummy_surd_result()));

    assert!(res.inner.is_ok());
    let with_res = res.inner.unwrap();
    assert_eq!(with_res.state.records_count, 2);
}

#[test]
fn test_causal_discovery_error() {
    use deep_causality_discovery::CausalDiscoveryError;
    use deep_causality_tensor::CausalTensorError;

    let cdl = create_cdl_with_features();

    let res = cdl.causal_discovery(|_| {
        Err(CausalDiscoveryError::TensorError(
            CausalTensorError::InvalidParameter("Mock Algo Failed".into()),
        ))
    });

    assert!(res.inner.is_err());
}
