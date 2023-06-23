/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::protocols::assumable::AssumableReasoning;
use deep_causality::utils::test_utils::{get_test_assumption, get_test_assumption_map, get_test_num_array};

#[test]
fn test_add()
{
    let mut map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let assumption = get_test_assumption();
    map.insert(4, assumption);
    assert_eq!(map.len(), 4);
}


#[test]
fn test_contains()
{
    let mut map = get_test_assumption_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_assumption();
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}


#[test]
fn test_remove()
{
    let mut map = get_test_assumption_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&3));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert_eq!(false, map.contains_key(&3));
}


#[test]
fn test_all_assumptions_tested()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);
}


#[test]
fn test_all_assumptions_valid()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, true);
}


#[test]
fn test_percent_assumption_valid()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, true);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 100.0);
}


#[test]
fn test_get_all_invalid_assumptions()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);
    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_invalid = map.get_all_invalid_assumptions();
    assert_eq!(all_invalid.len(), 0);
}

#[test]
fn test_get_all_valid_assumptions()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = map.get_all_valid_assumptions();
    assert_eq!(all_valid.len(), 3);
}

#[test]
fn test_get_all_tested_assumptions()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_tested_assumptions = map.get_all_tested_assumptions();
    assert_eq!(all_tested_assumptions.len(), 3);
}


#[test]
fn test_get_all_untested_assumptions()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_untested = map.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 3);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_untested = map.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 0);
}

#[test]
fn test_verify_all_assumptions()
{
    let map = get_test_assumption_map();
    assert_eq!(map.len(), 3);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = map.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    map.verify_all_assumptions(&data);

    let all_tested = map.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = map.all_assumptions_valid();
    assert_eq!(all_valid, true);
}