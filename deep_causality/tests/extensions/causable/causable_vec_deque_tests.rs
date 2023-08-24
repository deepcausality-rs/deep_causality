// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::VecDeque;

use deep_causality::prelude::*;

use crate::utils::test_utils::*;

fn get_test_causality_vec_deque<'l>(
) -> VecDeque<Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>> {
    VecDeque::from_iter(get_test_causality_vec())
}

#[test]
fn test_add() {
    let mut col = get_test_causality_vec_deque();
    assert_eq!(3, col.len());

    let q = get_test_causaloid();
    col.push_back(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_all_active() {
    let col = get_test_causality_vec_deque();
    assert!(!col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(col.get_all_causes_true());
}

#[test]
fn test_number_active() {
    let col = get_test_causality_vec_deque();
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
    let col = get_test_causality_vec_deque();
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
    let col = get_test_causality_vec_deque();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_causality_vec_deque();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col = get_test_causality_vec_deque();
    assert_eq!(3, col.to_vec().len());
}
