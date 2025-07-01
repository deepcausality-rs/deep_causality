/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

use deep_causality::utils_test::test_utils::*;

#[test]
fn test_add() {
    let mut col = get_test_causality_vec();
    assert_eq!(3, col.len());

    let q = get_test_causaloid();
    col.push(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_all_active() {
    let col = get_test_causality_vec();
    assert!(!col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(col.get_all_causes_true());
}

#[test]
fn test_number_active() {
    let col = get_test_causality_vec();
    assert!(!col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(col.get_all_causes_true());
    assert_eq!(3.0, col.number_active());
}

#[test]
fn test_percent_active() {
    let col = get_test_causality_vec();
    assert!(!col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(col.get_all_causes_true());
    assert_eq!(3.0, col.number_active());
    assert_eq!(100.0, col.percent_active());
}

#[test]
fn test_size() {
    let col = get_test_causality_vec();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_causality_vec();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col = get_test_causality_vec();
    assert_eq!(3, col.to_vec().len());
}
