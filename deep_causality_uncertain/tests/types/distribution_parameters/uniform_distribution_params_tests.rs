/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::UniformDistributionParams;

#[test]
fn test_uniform_distribution_params_new() {
    let low = 0.0;
    let high = 1.0;
    let params = UniformDistributionParams::new(low, high);
    assert_eq!(params.low, low);
    assert_eq!(params.high, high);

    let low_neg = -10.0;
    let high_pos = 5.0;
    let params_neg_pos = UniformDistributionParams::new(low_neg, high_pos);
    assert_eq!(params_neg_pos.low, low_neg);
    assert_eq!(params_neg_pos.high, high_pos);

    let equal_bounds = 7.0;
    let params_equal = UniformDistributionParams::new(equal_bounds, equal_bounds);
    assert_eq!(params_equal.low, equal_bounds);
    assert_eq!(params_equal.high, equal_bounds);
}

#[test]
fn test_uniform_distribution_params_debug_clone_copy() {
    let params = UniformDistributionParams::new(10.0, 20.0);

    // Test Debug
    assert_eq!(
        format!("{:?}", params),
        "UniformDistributionParams { low: 10.0, high: 20.0 }"
    );

    // Test Clone
    let cloned_params = params;
    assert_eq!(cloned_params.low, params.low);
    assert_eq!(cloned_params.high, params.high);

    // Test Copy (by assignment)
    let copied_params = params;
    assert_eq!(copied_params.low, params.low);
    assert_eq!(copied_params.high, params.high);
}

#[test]
fn test_uniform_distribution_params_display() {
    let params = UniformDistributionParams::new(1.23456, 7.89012);
    assert_eq!(
        format!("{}", params),
        "UniformDistributionParams { low: 1.2346 , high: 7.8901 }"
    );

    let params_zero = UniformDistributionParams::new(0.0000, 0.0);
    assert_eq!(
        format!("{}", params_zero),
        "UniformDistributionParams { low: 0.0000 , high: 0.0000 }"
    );

    let params_neg = UniformDistributionParams::new(-5.4321, -1.0);
    assert_eq!(
        format!("{}", params_neg),
        "UniformDistributionParams { low: -5.4321 , high: -1.0000 }"
    );
}
