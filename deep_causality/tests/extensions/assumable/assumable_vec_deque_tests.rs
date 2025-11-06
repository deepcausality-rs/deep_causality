/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::VecDeque;

use deep_causality::{AssumableReasoning, Assumption, PropagatingEffect};

use deep_causality::utils_test::test_utils::*;

fn get_test_assumption_vec_deque() -> VecDeque<Assumption> {
    VecDeque::from(get_test_assumption_vec())
}

#[test]
fn test_add() {
    let mut col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let assumption = get_test_assumption();
    col.push_back(assumption);
    assert_eq!(col.len(), 4);
}

#[test]
fn test_all_assumptions_tested() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();

    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);
}

#[test]
fn test_all_assumptions_valid() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();
    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(all_valid);
}

#[test]
fn test_percent_assumption_valid() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();
    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 100.0);
}

#[test]
fn test_get_all_invalid_assumptions() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();

    col.verify_all_assumptions(&data).unwrap();
    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_invalid = col.get_all_invalid_assumptions();
    assert_eq!(all_invalid.len(), 0);
}

#[test]
fn test_get_all_valid_assumptions() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();

    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = col.get_all_valid_assumptions();
    assert_eq!(all_valid.len(), 3);
}

#[test]
fn test_get_all_tested_assumptions() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();

    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_tested_assumptions = col.get_all_tested_assumptions();
    assert_eq!(all_tested_assumptions.len(), 3);
}

#[test]
fn test_get_all_untested_assumptions() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_untested = col.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 3);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();

    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_untested = col.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 0);
}

#[test]
fn test_verify_all_assumptions() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = col.percent_assumption_valid().unwrap();
    assert_eq!(all_valid_percent, 0.0);

    let data: Vec<PropagatingEffect> = get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::from_numerical(x))
        .collect();
    col.verify_all_assumptions(&data).unwrap();

    let all_tested = col.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = col.all_assumptions_valid();
    assert!(all_valid);
}

#[test]
fn test_get_all_items() {
    let col = get_test_assumption_vec_deque();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_len() {
    let col = get_test_assumption_vec_deque();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_assumption_vec_deque();
    assert!(!col.is_empty());
}
