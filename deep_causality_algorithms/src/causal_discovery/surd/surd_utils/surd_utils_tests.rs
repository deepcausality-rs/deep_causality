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

#[test]
fn test_arg_sort_stable_orders_by_value() {
    // Well-separated values (gaps far larger than `tol`) sort ascending.
    let data = vec![3.0_f64, 1.0, 2.0];
    let order = surd_utils::arg_sort_stable(&data, 1e-9);
    assert_eq!(order, vec![1, 2, 0]);
}

#[test]
fn test_arg_sort_stable_sub_tolerance_keeps_original_order() {
    // Values differing by far less than `tol` fall in the same grid cell and are
    // treated as ties; the stable sort preserves their original index order,
    // independent of the sub-resolution differences (here the larger value comes
    // first in the input and must stay first).
    let data = vec![1.0_f64 + 1e-15, 1.0, 1.0 - 1e-15];
    let order = surd_utils::arg_sort_stable(&data, 1e-9);
    assert_eq!(order, vec![0, 1, 2]);
}

#[test]
fn test_arg_sort_stable_empty() {
    let data = Vec::<f64>::new();
    let order = surd_utils::arg_sort_stable(&data, 1e-9);
    assert!(order.is_empty());
}
