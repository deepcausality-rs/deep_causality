// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::utils::test_utils::{get_test_causality_map, get_test_causaloid};

#[test]
fn test_add()
{
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
}

#[test]
fn test_contains()
{
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove()
{
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_all_active()
{
    let map = get_test_causality_map();
    assert!(!map.get_all_causes_true());

    let obs = 0.99;
    for (_, cause) in &map {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(map.get_all_causes_true());
}

#[test]
fn test_number_active()
{
    let map = get_test_causality_map();
    assert!(!map.get_all_causes_true());
    assert_eq!(0.0, map.number_active());

    let obs = 0.99;
    for (_, cause) in &map {
        cause.verify_single_cause(&obs).expect("verify failed");
    }

    assert!(map.get_all_causes_true());
    assert_eq!(3.0, map.number_active());
}

#[test]
fn test_percent_active()
{
    let map = get_test_causality_map();
    assert!(!map.get_all_causes_true());

    let obs = 0.99;
    for (_, cause) in &map {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert!(map.get_all_causes_true());
    assert_eq!(3.0, map.number_active());
    assert_eq!(100.0, map.percent_active());
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_len()
{
    let map = get_test_causality_map();
    assert_eq!(3, map.len());
}

#[test]
fn test_is_empty()
{
    let map = get_test_causality_map();
    assert!(!map.is_empty());
}

#[test]
fn test_to_vec()
{
    let map = get_test_causality_map();
    assert_eq!(3, map.to_vec().len());
}