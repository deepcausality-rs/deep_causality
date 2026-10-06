/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{MetricTensorError, MetricTensorErrorEnum};
use std::error::Error;

#[test]
fn test_each_constructor_builds_its_variant() {
    assert_eq!(
        MetricTensorError::NonFinite(0, 3),
        MetricTensorError::new(MetricTensorErrorEnum::NonFinite { row: 0, col: 3 })
    );
    assert_eq!(
        MetricTensorError::Asymmetric(1, 2),
        MetricTensorError(MetricTensorErrorEnum::Asymmetric { row: 1, col: 2 })
    );
    assert_eq!(
        MetricTensorError::Signature(2, 1, 1).0,
        MetricTensorErrorEnum::Signature {
            positive: 2,
            negative: 1,
            zero: 1
        }
    );
}

#[test]
fn test_the_fields_are_kept_apart() {
    // Swapped arguments name a different entry, and different counts a different inertia.
    assert_ne!(
        MetricTensorError::Asymmetric(1, 2),
        MetricTensorError::Asymmetric(2, 1)
    );
    assert_ne!(
        MetricTensorError::NonFinite(0, 3),
        MetricTensorError::Asymmetric(0, 3)
    );
    assert_ne!(
        MetricTensorError::Signature(2, 1, 1),
        MetricTensorError::Signature(1, 2, 1)
    );
}

#[test]
fn test_display_names_the_rule_and_its_data() {
    assert_eq!(
        MetricTensorError::NonFinite(0, 3).to_string(),
        "MetricTensorError: g[0][3] is not finite"
    );
    assert_eq!(
        MetricTensorError::Asymmetric(1, 2).to_string(),
        "MetricTensorError: the tensor is not symmetric: g[1][2] differs from g[2][1]"
    );
    assert_eq!(
        MetricTensorError::Signature(2, 1, 1).to_string(),
        "MetricTensorError: the tensor is not of signature (−, +, +, +): 2 positive, 1 negative \
         and 1 zero eigenvalues"
    );
}

#[test]
fn test_metric_tensor_error_is_error_trait_object() {
    let err = MetricTensorError::Signature(4, 0, 0);
    let trait_obj: &dyn Error = &err;
    assert!(trait_obj.is::<MetricTensorError>());
    assert!(trait_obj.source().is_none());
}

#[test]
fn test_metric_tensor_error_debug() {
    assert_eq!(
        format!("{:?}", MetricTensorError::NonFinite(2, 2)),
        "MetricTensorError(NonFinite { row: 2, col: 2 })"
    );
}
