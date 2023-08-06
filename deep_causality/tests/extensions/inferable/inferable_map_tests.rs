// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::utils::test_utils::{get_test_inf_map, get_test_inferable};

#[test]
fn test_add() {
    let mut col = get_test_inf_map();
    assert_eq!(2, col.len());

    let f3 = get_test_inferable(3, true);
    col.insert(3,f3);
    assert_eq!(3, col.len());
}

#[test]
fn test_all_inferable() {
    let f = get_test_inferable(3, false);
    let col: Vec<Inference> = Vec::from_iter([f]);
    assert!(col.all_inferable());
}

#[test]
fn test_all_inverse_inferable() {
    let f = get_test_inferable(3, true);
    let col: Vec<Inference> = Vec::from_iter([f]);
    assert!(col.all_inverse_inferable());
}

#[test]
fn test_all_non_inferable() {
    let col = get_test_inf_map();
    assert!(!col.all_non_inferable());
}

#[test]
fn test_conjoint_delta() {
    let col = get_test_inf_map();
    // in the synthetic test data,
    // the conjoint delta is 0.0% because all causes explain the observed effects.
    assert_eq!(0.0, col.conjoint_delta());
}

#[test]
fn test_number_inferable() {
    let col = get_test_inf_map();
    assert_eq!(1.0, col.number_inferable());
}

#[test]
fn test_number_inverse_inferable() {
    let col = get_test_inf_map();
    assert_eq!(1.0, col.number_inverse_inferable());
}

#[test]
fn test_number_non_inferable() {
    let col = get_test_inf_map();
    assert_eq!(0.0, col.number_non_inferable());
}

#[test]
fn test_percent_inferable() {
    let col = get_test_inf_map();
    assert_eq!(50.0, col.percent_inferable())
}

#[test]
fn test_percent_inverse_inferable() {
    let col = get_test_inf_map();
    assert_eq!(50.0, col.percent_inverse_inferable())
}

#[test]
fn test_percent_non_inferable() {
    let col = get_test_inf_map();
    assert_eq!(0.0, col.percent_non_inferable())
}

#[test]
fn test_get_all_inferable() {
    let mut col = get_test_inf_map();
    let f3 = get_test_inferable(3, false);
    col.insert(3, f3);
    let all_inf = col.get_all_inferable();
    assert_eq!(2, all_inf.len());
}

#[test]
fn test_get_all_inverse_inferable() {
    let col = get_test_inf_map();
    let all_inv_inf = col.get_all_inverse_inferable();
    assert_eq!(1, all_inv_inf.len());
}

#[test]
fn test_get_all_non_inferable() {
    let col = get_test_inf_map();
    let all_non_inf = col.get_all_non_inferable();
    assert_eq!(0, all_non_inf.len());
}

#[test]
fn test_get_all_items() {
    let col = get_test_inf_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_len() {
    let col = get_test_inf_map();
    assert_eq!(2, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_inf_map();
    assert!(!col.is_empty());
}
