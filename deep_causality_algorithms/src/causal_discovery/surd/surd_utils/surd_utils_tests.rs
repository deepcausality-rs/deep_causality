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
fn test_entropy_nvars_cdl_all_none_is_empty_input() {
    // Behaviour change, decided with the migration: a marginal in which every entry is absent has
    // no distribution to take the entropy of, and is refused rather than answered with zero. Zero
    // is the entropy of a certain outcome, which is a different statement from having observed
    // nothing. Distinct from the low-mass case below, where values are present but sum under
    // epsilon, and which still returns zero.
    let data: Vec<Option<f64>> = vec![None, None, None, None];
    let p = CausalTensor::new(data, vec![2, 2]).unwrap();

    let err = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("empty"),
        "an all-absent marginal is EmptyInput, got: {err}"
    );
}

#[test]
fn test_entropy_nvars_cdl_all_zero_returns_zero() {
    let data: Vec<Option<f64>> = vec![Some(0.0), Some(0.0), Some(0.0), Some(0.0)];
    let p = CausalTensor::new(data, vec![2, 2]).unwrap();

    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
    assert_eq!(h, 0.0);
}

#[test]
fn test_entropy_nvars_cdl_discards_mass_below_epsilon_after_normalising() {
    // The CDL path is the one place in the workspace that uses `ZeroPolicy::SkipBelow(ε)` rather
    // than `SkipZero`, and the difference only shows on an entry that is positive but, once the
    // distribution is normalised, lands below `f64::EPSILON`. Nothing else in this suite has one,
    // so the choice between the two policies was unpinned.
    //
    // `(1, 1e-17)` normalises to about `(1, 1e-17)`. The first term contributes `−1·log₂1 = 0`, so
    // the whole entropy is the second — and under `SkipBelow(ε)` that term is numerical residue
    // rather than mass, so it is dropped and the answer is exactly zero. Under `SkipZero` it would
    // be counted, at `−1e-17·log₂(1e-17) ≈ 5.6e-16`.
    let data: Vec<Option<f64>> = vec![Some(1.0), Some(1e-17)];
    let p = CausalTensor::new(data, vec![2]).unwrap();

    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
    assert_eq!(
        h, 0.0,
        "normalised mass below epsilon is residue, not a symbol"
    );
}

#[test]
fn test_entropy_nvars_cdl_counts_mass_above_epsilon() {
    // The other side of the same threshold, so the test above cannot be satisfied by a policy that
    // simply discards every small entry. `(1, 1e-6)` normalises to about `(1, 1e-6)`, and
    // `−1e-6·log₂(1e-6) ≈ 1.99e-5` — small, above epsilon, and counted.
    let data: Vec<Option<f64>> = vec![Some(1.0), Some(1e-6)];
    let p = CausalTensor::new(data, vec![2]).unwrap();

    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
    assert!(h > 1e-5, "mass above epsilon must contribute, got {h}");
    assert!(h < 3e-5, "and it is the only contribution, got {h}");
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
#[test]
fn entropy_rejects_invalid_probabilities() {
    for value in [-0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let plain = deep_causality_tensor::CausalTensor::new(vec![value, 0.5], vec![2]).unwrap();
        assert!(matches!(
            super::entropy_nvars(&plain, &[0]),
            Err(deep_causality_tensor::CausalTensorError::InvalidParameter(
                _
            ))
        ));
        let optional =
            deep_causality_tensor::CausalTensor::new(vec![Some(value), Some(0.5)], vec![2])
                .unwrap();
        assert!(matches!(
            super::surd_utils_cdl::entropy_nvars_cdl(&optional, &[0]),
            Err(deep_causality_tensor::CausalTensorError::InvalidParameter(
                _
            ))
        ));
    }
}

#[test]
fn optional_entropy_strict_mass_boundary() {
    let eps = f64::EPSILON;
    for (mass, expected) in [(eps / 2.0, 0.0), (eps, 1.0), (eps * 2.0, 1.0)] {
        // Two equally likely outcomes have one bit, unless the caller's mass guard fires.
        let p = deep_causality_tensor::CausalTensor::new(
            vec![Some(mass / 2.0), None, Some(mass / 2.0)],
            vec![3],
        )
        .unwrap();
        let h = super::surd_utils_cdl::entropy_nvars_cdl(&p, &[0]).unwrap();
        assert!((h - expected).abs() < 1e-14);
    }
}
