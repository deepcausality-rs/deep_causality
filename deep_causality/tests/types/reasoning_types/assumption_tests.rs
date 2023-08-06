// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{DescriptionValue, Identifiable};
use deep_causality::protocols::assumable::Assumable;
use deep_causality::utils::test_utils::{get_test_assumption, get_test_num_array};

#[test]
fn test_assumption_tested() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert!(!tested);

    let data = get_test_num_array();
    assumption.verify_assumption(&data);

    let tested = assumption.assumption_tested();
    assert!(tested);
}

#[test]
fn test_verify_assumption() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert!(!tested);

    let valid = assumption.assumption_tested();
    assert!(!valid);

    let data = get_test_num_array();
    let valid = assumption.verify_assumption(&data);
    assert!(valid);
}

#[test]
fn test_assumption_valid() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert!(!tested);

    let valid = assumption.assumption_tested();
    assert!(!valid);

    let data = get_test_num_array();
    let valid = assumption.verify_assumption(&data);
    assert!(valid);

    let tested = assumption.assumption_tested();
    assert!(tested);

    let valid = assumption.assumption_valid();
    assert!(valid);
}

#[test]
fn test_assumption_id() {
    let id = 1;
    let assumption = get_test_assumption();
    assert_eq!(assumption.id(), id);
}

#[test]
fn test_assumption_description() {
    let id = 1;
    let description: String = "Test assumption that data are there".to_string() as DescriptionValue;

    let assumption = get_test_assumption();
    assert_eq!(assumption.id(), id);
    assert_eq!(assumption.description(), description)
}

#[test]
fn test_assumption_to_string() {
    let id = 1;
    let description: String = "Test assumption that data are there".to_string() as DescriptionValue;

    let assumption = get_test_assumption();
    assert_eq!(assumption.id(), id);
    assert_eq!(assumption.description(), description);

    let expected = format!("Assumption: id: {}, description: {}, assumption_fn: fn(&[NumericalValue]) -> bool;, assumption_tested: {},assumption_valid: {}", id, description, false, false);
    let actual = assumption.to_string();
    assert_eq!(actual, expected);
}

