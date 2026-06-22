/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// surd_utils are private and thus cannot be tested in the test folder.
// While a lot gets tested through the public API, these tests cover some rare corner cases.

use crate::causal_discovery::surd::surd_utils;
use crate::causal_discovery::surd::surd_utils::surd_utils_cdl;
use deep_causality_tensor::{CausalTensor, CausalTensorError};

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

// ---------------------------------------------------------------------------
// entropy_nvars: marginal-path zero-probability branch (mod.rs line ~142)
// ---------------------------------------------------------------------------

#[test]
fn test_entropy_nvars_marginal_path_with_zero_entry() {
    // 2-dim joint over (X, Y). Requesting entropy of axis 0 sums out axis 1,
    // taking the marginal path (axes_to_sum_out is non-empty). The marginal of
    // X has a zero entry, exercising the `else { acc }` skip branch in the
    // marginal-distribution entropy fold.
    // P(X=0,*) = 0, P(X=1,*) = 1.0 split across Y.
    let data = vec![0.0_f64, 0.0, 0.5, 0.5];
    let p = CausalTensor::new(data, vec![2, 2]).unwrap();

    // Marginal over axis 0 is [0.0, 1.0]; the 0.0 entry hits the skip branch.
    let h = surd_utils::entropy_nvars(&p, &[0]).unwrap();

    // Entropy of a deterministic marginal [0, 1] is 0.
    assert!(h.abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// entropy_nvars_cdl: all-None / all-zero marginal -> entropy 0 (surd_utils_cdl line ~158)
// ---------------------------------------------------------------------------

#[test]
fn test_entropy_nvars_cdl_all_none_returns_zero() {
    // A marginal whose Some values sum to (effectively) zero must short-circuit
    // to entropy 0 via the `sum_of_marginals.abs() < eps` guard.
    let data: Vec<Option<f64>> = vec![None, None, None, None];
    let p = CausalTensor::new(data, vec![2, 2]).unwrap();

    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_nvars_cdl_all_zero_returns_zero() {
    let data: Vec<Option<f64>> = vec![Some(0.0), Some(0.0), Some(0.0), Some(0.0)];
    let p = CausalTensor::new(data, vec![2, 2]).unwrap();

    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
    assert_eq!(h, 0.0);
}

// ---------------------------------------------------------------------------
// surd_utils_cdl: shape-mismatch error branches
// ---------------------------------------------------------------------------

#[test]
fn test_safe_div_cdl_shape_mismatch() {
    let num = CausalTensor::new(vec![Some(1.0_f64), Some(2.0)], vec![2]).unwrap();
    let den = CausalTensor::new(vec![Some(1.0_f64), Some(2.0), Some(3.0)], vec![3]).unwrap();

    let result = surd_utils_cdl::safe_div_cdl(&num, &den);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_mul_cdl_shape_mismatch() {
    let a = CausalTensor::new(vec![Some(1.0_f64), Some(2.0)], vec![2]).unwrap();
    let b = CausalTensor::new(vec![Some(1.0_f64), Some(2.0), Some(3.0)], vec![3]).unwrap();

    let result = surd_utils_cdl::mul_cdl(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_sub_cdl_shape_mismatch() {
    let a = CausalTensor::new(vec![Some(1.0_f64), Some(2.0)], vec![2]).unwrap();
    let b = CausalTensor::new(vec![Some(1.0_f64), Some(2.0), Some(3.0)], vec![3]).unwrap();

    let result = surd_utils_cdl::sub_cdl(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_broadcast_to_cdl_higher_rank_source_errors() {
    // Source tensor has higher rank than the target shape -> ShapeMismatch.
    let tensor = CausalTensor::new(
        vec![Some(1.0_f64), Some(2.0), Some(3.0), Some(4.0)],
        vec![2, 2],
    )
    .unwrap();
    let target_shape = vec![4];

    let result = surd_utils_cdl::broadcast_to_cdl(&tensor, &target_shape);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}
