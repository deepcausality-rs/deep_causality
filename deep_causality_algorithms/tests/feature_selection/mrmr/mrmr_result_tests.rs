/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::mrmr::MrmrResult;

#[test]
fn test_new_and_getters() {
    let features = vec![(0, 1.0), (2, 0.5)];
    let result = MrmrResult::new(features.clone());

    assert_eq!(result.len(), 2);
    assert!(!result.is_empty());
    assert_eq!(result.features(), &features);
}

#[test]
fn test_empty() {
    let features: Vec<(usize, f64)> = vec![];
    let result = MrmrResult::new(features);

    assert_eq!(result.len(), 0);
    assert!(result.is_empty());
    assert!(result.features().is_empty());
}

#[test]
fn test_iter() {
    let features = vec![(0, 1.0), (1, 0.8), (5, 0.2)];
    let result = MrmrResult::new(features.clone());

    let collected: Vec<(usize, f64)> = result.iter().copied().collect();
    assert_eq!(collected, features);
}

#[test]
fn test_into_iter_owned() {
    let features = vec![(10, 0.9)];
    let result = MrmrResult::new(features.clone());

    let mut iter = result.into_iter();
    assert_eq!(iter.next(), Some((10, 0.9)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_into_iter_ref() {
    let features = vec![(3, 0.3)];
    let result = MrmrResult::new(features.clone());
    let result_ref = &result;

    let mut iter = result_ref.into_iter();
    assert_eq!(iter.next(), Some(&(3, 0.3)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_display_formatting() {
    let features = vec![(0, 1.0), (1, 0.5)];
    let result = MrmrResult::new(features);
    let display_str = format!("{}", result);

    assert!(display_str.contains("mRMR Selected Features:"));
    assert!(display_str.contains("Index      | Score"));
    assert!(display_str.contains("0          | 1.0000"));
    assert!(display_str.contains("1          | 0.5000"));
}

#[test]
fn test_display_formatting_empty() {
    let features: Vec<(usize, f64)> = vec![];
    let result = MrmrResult::new(features);
    let display_str = format!("{}", result);

    assert!(display_str.contains("mRMR Selected Features:"));
    // Header should still be present
    assert!(display_str.contains("Index      | Score"));
}

#[test]
fn test_debug_derive() {
    let features = vec![(1, 0.1)];
    let result = MrmrResult::new(features);
    let debug_str = format!("{:?}", result);

    // Check that it's the standard derive debug format
    assert!(debug_str.contains("MrmrResult"));
    assert!(debug_str.contains("features"));
    assert!(debug_str.contains("(1, 0.1)"));
}

#[test]
fn test_clone_and_partial_eq() {
    let features = vec![(0, 1.0)];
    let result1 = MrmrResult::new(features.clone());
    #[allow(clippy::clone_on_copy)] // Just testing the Clone impl of the struct
    let result2 = result1.clone();

    assert_eq!(result1, result2);

    let result3 = MrmrResult::new(vec![(0, 0.9)]);
    assert_ne!(result1, result3);
}
