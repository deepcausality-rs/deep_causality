/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// surd_utils are private and thus cannot be tested in the test folder.
// While a lot gets tested through the public API, these tests cover some rare corner cases.

use crate::causal_discovery::surd::surd_utils;
use crate::causal_discovery::surd::surd_utils::surd_utils_cdl;
use deep_causality_tensor::CausalTensorError;

#[test]
fn test_diff_empty() {
    let data = Vec::<f64>::new();

    let diff = surd_utils::diff(data.as_slice());

    assert!(diff.is_empty());
}

#[test]
fn test_combinations_r_empty() {
    let data = Vec::<f64>::new();
    let r = 0;

    let result = surd_utils::combinations(data.as_slice(), r);

    assert_eq!(result.len(), 1);
    assert!(result[0].is_empty());
}

#[test]
#[should_panic]
fn test_combinations_r_exceeds_pool() {
    let data = Vec::<f64>::new();
    let r = 3;
    // Triggers panic: Cannot choose r elements from a pool smaller than r.
    surd_utils::combinations(data.as_slice(), r);
}

#[test]
fn test_ravel_index_from_coords_dimension_mismatch() {
    let coords = &[1, 2];
    let shape = &[3, 3, 3]; // Mismatched dimensions
    let result = surd_utils_cdl::ravel_index_from_coords_option(coords, shape);
    assert!(matches!(result, Err(CausalTensorError::DimensionMismatch)));
}

#[test]
fn test_ravel_index_from_coords_axis_out_of_bounds() {
    let coords = &[1, 5];
    let shape = &[3, 3]; // 5 is out of bounds for second axis
    let result = surd_utils_cdl::ravel_index_from_coords_option(coords, shape);
    assert!(matches!(result, Err(CausalTensorError::AxisOutOfBounds)));
}
