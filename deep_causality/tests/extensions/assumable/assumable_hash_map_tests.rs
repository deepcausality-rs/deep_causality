/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use deep_causality::prelude::{AssumableReasoning, Assumption};

use deep_causality::utils_test::test_utils::*;

fn get_test_assumption_map() -> HashMap<i8, Assumption> {
    let a1 = get_test_assumption();
    let a2 = get_test_assumption();
    let a3 = get_test_assumption();
    HashMap::from_iter([(1, a1), (2, a2), (3, a3)])
}

#[test]
fn test_add() {
    let mut map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let assumption = get_test_assumption();
    map.insert(4, assumption);
    assert_eq!(map.len(), 4);
}

#[test]
fn test_contains() {
    let mut map = get_test_assumption_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_assumption();
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove() {
    let mut map = get_test_assumption_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&3));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_all_assumptions_tested() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);
}

#[test]
fn test_all_assumptions_valid() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(all_valid);
}

#[test]
fn test_percent_assumption_valid() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 100.0);
}

#[test]
fn test_get_all_invalid_assumptions() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_invalid = map.get_all_invalid_assumptions();
    assert_eq!(all_invalid.len(), 0);
}

#[test]
fn test_get_all_valid_assumptions() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = map.get_all_valid_assumptions();
    assert_eq!(all_valid.len(), 3);
}

#[test]
fn test_get_all_tested_assumptions() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_tested_assumptions = map.get_all_tested_assumptions();
    assert_eq!(all_tested_assumptions.len(), 3);
}

#[test]
fn test_get_all_untested_assumptions() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_untested = map.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 3);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_untested = map.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 0);
}

#[test]
fn test_verify_all_assumptions() {
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert!(!all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(!all_valid);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert!(all_tested);

    let all_valid = map.all_assumptions_valid();
    assert!(all_valid);
}

#[test]
fn test_get_all_items() {
    let col = get_test_assumption_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_len() {
    let col = get_test_assumption_map();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_assumption_map();
    assert!(!col.is_empty());
}
