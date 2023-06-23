/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::protocols::assumable::Assumable;
use deep_causality::utils::test_utils::{get_test_assumption, get_test_num_array};

#[test]
fn test_assumption_tested() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert_eq!(tested, false);

    let data = get_test_num_array();
    assumption.verify_assumption(&data);

    let tested = assumption.assumption_tested();
    assert_eq!(tested, true);
}

#[test]
fn test_verify_assumption() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert_eq!(tested, false);

    let valid = assumption.assumption_tested();
    assert_eq!(valid, false);

    let data = get_test_num_array();
    let valid = assumption.verify_assumption(&data);
    assert_eq!(valid, true);
}

#[test]
fn test_assumption_valid() {
    let assumption = get_test_assumption();

    let tested = assumption.assumption_tested();
    assert_eq!(tested, false);

    let valid = assumption.assumption_tested();
    assert_eq!(valid, false);

    let data = get_test_num_array();
    let valid = assumption.verify_assumption(&data);
    assert_eq!(valid, true);

    let tested = assumption.assumption_tested();
    assert_eq!(tested, true);

    let valid = assumption.assumption_valid();
    assert_eq!(valid, true);
}