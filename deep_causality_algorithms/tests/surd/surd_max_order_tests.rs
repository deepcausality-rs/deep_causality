/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::causal_discovery::surd::MaxOrder;
use deep_causality_data_structures::CausalTensorError;
use std::fmt::Write;

#[test]
fn test_get_k_max_order_min() {
    let order = MaxOrder::Min;
    assert_eq!(order.get_k_max_order(5).unwrap(), 2);
    assert_eq!(order.get_k_max_order(10).unwrap(), 2);
}

#[test]
fn test_get_k_max_order_max() {
    let order = MaxOrder::Max;
    assert_eq!(order.get_k_max_order(5).unwrap(), 5);
    assert_eq!(order.get_k_max_order(10).unwrap(), 10);
}

#[test]
fn test_get_k_max_order_some_valid() {
    let n_vars = 10;
    // Valid case
    let order = MaxOrder::Some(5);
    assert_eq!(order.get_k_max_order(n_vars).unwrap(), 5);

    // Edge case: k = 2
    let order = MaxOrder::Some(2);
    assert_eq!(order.get_k_max_order(n_vars).unwrap(), 2);

    // Edge case: k = n_vars
    let order = MaxOrder::Some(10);
    assert_eq!(order.get_k_max_order(n_vars).unwrap(), 10);
}

#[test]
fn test_get_k_max_order_some_invalid_k_too_low() {
    let order = MaxOrder::Some(1);
    let result = order.get_k_max_order(10);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CausalTensorError::InvalidParameter(_)));
    assert_eq!(
        err.to_string(),
        "CausalTensorError: Invalid parameter: Max order k must be at least 2."
    );
}

#[test]
fn test_get_k_max_order_some_invalid_k_too_high() {
    let order = MaxOrder::Some(11);
    let result = order.get_k_max_order(10);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CausalTensorError::InvalidParameter(_)));
    assert_eq!(
        err.to_string(),
        "CausalTensorError: Invalid parameter: Max order k (11) cannot be greater than the number of source variables (10)."
    );
}

#[test]
fn test_display_implementation() {
    let mut s = String::new();
    write!(s, "{}", MaxOrder::Min).unwrap();
    assert_eq!(s, "Min");

    s.clear();
    write!(s, "{}", MaxOrder::Max).unwrap();
    assert_eq!(s, "Max");

    s.clear();
    write!(s, "{}", MaxOrder::Some(7)).unwrap();
    assert_eq!(s, "Some(7)");
}

#[test]
fn test_derived_traits() {
    // Test, Copy, PartialEq
    let order1 = MaxOrder::Some(5);
    let order2 = order1; // Test Copy
    assert_eq!(order1, order2);

    let order_min = MaxOrder::Min;
    let order_max = MaxOrder::Max;
    assert_ne!(order1, order_min);
    assert_ne!(order_min, order_max);

    // Test Ord, PartialOrd
    assert!(MaxOrder::Min < MaxOrder::Some(3));
    assert!(MaxOrder::Some(3) < MaxOrder::Some(4));
    assert!(MaxOrder::Some(4) < MaxOrder::Max);
}
