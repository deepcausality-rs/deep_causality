/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::mrmr::mrmr_utils;
use deep_causality_data_structures::CausalTensor;

#[test]
fn test_pearson_correlation() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 4.0, 3.0, 2.0, 1.0];
    let shape = vec![2, 5];
    let tensor = CausalTensor::new(data, shape).unwrap();

    let corr = mrmr_utils::pearson_correlation(&tensor, 0, 4).unwrap();
    assert!((corr - (-1.0)).abs() < 1e-9);
}

#[test]
fn test_f_statistic() {
    let data = vec![1.0, 2.0, 2.0, 4.0, 3.0, 6.0];
    let shape = vec![3, 2];
    let tensor = CausalTensor::new(data, shape).unwrap();

    // Correlation is 1.0, so F-statistic should be a large number.
    let f_stat = mrmr_utils::f_statistic(&tensor, 0, 1).unwrap();
    assert_eq!(f_stat, 1e12);
}
