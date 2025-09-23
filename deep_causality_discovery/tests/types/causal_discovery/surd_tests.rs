/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::{
    CausalDiscovery, CausalDiscoveryConfig, CausalDiscoveryError, SurdCausalDiscovery, SurdConfig,
};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_surd_causal_discovery_success() {
    let data = vec![
        0.1, 0.2, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
        0.0, 0.2, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
        0.3, 0.0, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
        0.1, 0.1, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
    ];
    let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();

    let surd_config = SurdConfig::new(MaxOrder::Max, 0); // Target column 0
    let config = CausalDiscoveryConfig::Surd(surd_config);

    let discoverer = SurdCausalDiscovery;
    let result = discoverer.discover(p_raw, &config).unwrap();

    // Check that the main aggregate maps are not empty
    assert!(!result.synergistic_info().is_empty());
    assert!(!result.redundant_info().is_empty());
    assert!(!result.mutual_info().is_empty());

    // Check a specific calculated value (from deep_causality_algorithms tests)
    let synergy_12 = result.synergistic_info().get(&vec![1, 2]).unwrap();
    assert!(*synergy_12 > 0.0);

    let redundancy = result.redundant_info().get(&vec![1, 2]).unwrap();
    assert!(redundancy.is_finite());

    // Info leak should be non-zero
    assert!(result.info_leak() > 0.0);
}

#[test]
fn test_surd_causal_discovery_error_empty_tensor() {
    let p_raw = CausalTensor::new(vec![], vec![0]).unwrap();

    let surd_config = SurdConfig::new(MaxOrder::Max, 0);
    let config = CausalDiscoveryConfig::Surd(surd_config);

    let discoverer = SurdCausalDiscovery;
    let result = discoverer.discover(p_raw, &config);

    assert!(result.is_err());
    if let Err(CausalDiscoveryError::TensorError(e)) = result {
        assert_eq!(e, CausalTensorError::EmptyTensor);
    } else {
        panic!(
            "Expected CausalDiscoveryError::TensorError, got {:?}",
            result
        );
    }
}

#[test]
fn test_surd_causal_discovery_error_invalid_operation() {
    let p_raw = CausalTensor::new(vec![0.0; 8], vec![2, 2, 2]).unwrap();

    let surd_config = SurdConfig::new(MaxOrder::Max, 0);
    let config = CausalDiscoveryConfig::Surd(surd_config);

    let discoverer = SurdCausalDiscovery;
    let result = discoverer.discover(p_raw, &config);

    assert!(result.is_err());
    if let Err(CausalDiscoveryError::TensorError(e)) = result {
        assert_eq!(e, CausalTensorError::InvalidOperation);
    } else {
        panic!(
            "Expected CausalDiscoveryError::TensorError, got {:?}",
            result
        );
    }
}

#[test]
fn test_surd_causal_discovery_error_invalid_parameter() {
    let p_raw = CausalTensor::new(vec![0.125; 8], vec![2, 2, 2]).unwrap();

    // MaxOrder::Some(3) where n_vars = 2, should cause InvalidParameter
    let surd_config = SurdConfig::new(MaxOrder::Some(3), 0);
    let config = CausalDiscoveryConfig::Surd(surd_config);

    let discoverer = SurdCausalDiscovery;
    let result = discoverer.discover(p_raw, &config);

    assert!(result.is_err());
    if let Err(CausalDiscoveryError::TensorError(e)) = result {
        assert!(matches!(e, CausalTensorError::InvalidParameter(_)));
    } else {
        panic!(
            "Expected CausalDiscoveryError::TensorError, got {:?}",
            result
        );
    }
}
