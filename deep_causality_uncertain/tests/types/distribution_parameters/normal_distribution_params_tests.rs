/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::NormalDistributionParams;

#[test]
fn test_normal_distribution_params_new() {
    let mean = 10.0;
    let std_dev = 2.5;
    let params = NormalDistributionParams::new(mean, std_dev);
    assert_eq!(params.mean, mean);
    assert_eq!(params.std_dev, std_dev);

    let mean_neg = -5.0;
    let std_dev_zero = 0.0;
    let params_neg_zero = NormalDistributionParams::new(mean_neg, std_dev_zero);
    assert_eq!(params_neg_zero.mean, mean_neg);
    assert_eq!(params_neg_zero.std_dev, std_dev_zero);
}

#[test]
fn test_normal_distribution_params_debug_clone_copy() {
    let params = NormalDistributionParams::new(100.0, 15.0);

    // Test Debug
    assert_eq!(
        format!("{:?}", params),
        "NormalDistributionParams { mean: 100.0, std_dev: 15.0 }"
    );

    // Test Clone
    let cloned_params = params;
    assert_eq!(cloned_params.mean, params.mean);
    assert_eq!(cloned_params.std_dev, params.std_dev);

    // Test Copy (by assignment)
    let copied_params = params;
    assert_eq!(copied_params.mean, params.mean);
    assert_eq!(copied_params.std_dev, params.std_dev);
}

#[test]
fn test_normal_distribution_params_display() {
    let params = NormalDistributionParams::new(12.34567, 0.98765);
    assert_eq!(
        format!("{}", params),
        "NormalDistributionParams { mean:  12.3457 , std_dev:  0.9877  }"
    );

    let params_zero = NormalDistributionParams::new(0.0, 0.0);
    assert_eq!(
        format!("{}", params_zero),
        "NormalDistributionParams { mean:  0.0000 , std_dev:  0.0000  }"
    );

    let params_neg = NormalDistributionParams::new(-1.23, 4.56);
    assert_eq!(
        format!("{}", params_neg),
        "NormalDistributionParams { mean:  -1.2300 , std_dev:  4.5600  }"
    );
}
