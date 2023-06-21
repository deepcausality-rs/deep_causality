/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::prelude::{AssumableReasoning, Assumption};
use deep_causality::utils::test_utils::{get_test_assumption, get_test_assumption_coll, get_test_num_array};


#[test]
fn test_add()
{
    let mut col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let assumption = get_test_assumption();
    col.push(assumption);
    assert_eq!(col.len(), 4);
}

#[test]
fn test_all_assumptions_tested()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);
}

#[test]
fn test_all_assumptions_valid()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);
    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, true);
}

#[test]
fn test_percent_assumption_valid()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);
    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, true);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 100.0);
}

#[test]
fn test_get_all_invalid_assumptions()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);
    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_invalid = col.get_all_invalid_assumptions();
    assert_eq!(all_invalid.len(), 0);
}

#[test]
fn test_get_all_valid_assumptions()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = col.get_all_valid_assumptions();
    assert_eq!(all_valid.len(), 3);
}

#[test]
fn test_get_all_tested_assumptions()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_tested_assumptions = col.get_all_tested_assumptions();
    assert_eq!(all_tested_assumptions.len(), 3);
}

#[test]
fn test_get_all_untested_assumptions()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_untested = col.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 3);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_untested = col.get_all_untested_assumptions();
    assert_eq!(all_untested.len(), 0);
}

#[test]
fn test_verify_all_assumptions()
{
    let col: Vec<Assumption> = get_test_assumption_coll();
    assert_eq!(col.len(), 3);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, false);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, false);

    let all_valid_percent = col.percent_assumption_valid();
    assert_eq!(all_valid_percent, 0.0);

    let data = get_test_num_array();
    col.verify_all_assumptions(&data);

    let all_tested = col.all_assumptions_tested();
    assert_eq!(all_tested, true);

    let all_valid = col.all_assumptions_valid();
    assert_eq!(all_valid, true);
}
