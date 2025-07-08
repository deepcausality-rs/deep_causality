/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::traits::assumable::Assumable;
use deep_causality::{DescriptionValue, Identifiable};

use deep_causality::utils_test::test_utils::*;

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
fn test_assumption_debug() {
    let assumption = get_test_assumption();
    let id = 1;
    let description = "Test assumption that data are there";

    // 1. Test initial state (before verification)
    let expected_initial = format!(
        "Assumption: id: {}, description: {}, assumption_fn: fn(&[NumericalValue]) -> bool;, assumption_tested: {},assumption_valid: {}",
        id, description, false, false
    );

    // Test the Display trait implementation
    assert_eq!(format!("{assumption}"), expected_initial);
    // Test the Debug trait implementation
    assert_eq!(format!("{assumption:?}"), expected_initial);

    // 2. Verify the assumption to change its internal state
    let data = get_test_num_array();
    assumption.verify_assumption(&data); // This sets tested and valid to true

    // 3. Test final state (after verification)
    let expected_after_verify = format!(
        "Assumption: id: {}, description: {}, assumption_fn: fn(&[NumericalValue]) -> bool;, assumption_tested: {},assumption_valid: {}",
        id, description, true, true
    );

    // Test the Display trait again
    assert_eq!(format!("{assumption}"), expected_after_verify);
    // Test the Debug trait again
    assert_eq!(format!("{assumption:?}"), expected_after_verify);
}

#[test]
fn test_assumption_to_string() {
    let id = 1;
    let description: String = "Test assumption that data are there".to_string() as DescriptionValue;

    let assumption = get_test_assumption();
    assert_eq!(assumption.id(), id);
    assert_eq!(assumption.description(), description);

    let expected = format!(
        "Assumption: id: {}, description: {}, assumption_fn: fn(&[NumericalValue]) -> bool;, assumption_tested: {},assumption_valid: {}",
        id, description, false, false
    );
    let actual = assumption.to_string();
    assert_eq!(actual, expected);
}
