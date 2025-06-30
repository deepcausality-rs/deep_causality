/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

use deep_causality::utils_test::test_utils::*;

#[test]
fn test_effect_observed() {
    let o1 = get_test_observation();
    let target_threshold = 14.0;
    let false_effect = 0.0;
    let true_effect = 1.0;

    assert!(!o1.effect_observed(target_threshold, false_effect));
    assert!(o1.effect_observed(target_threshold, true_effect));
}

#[test]
fn test_effect_id() {
    let id = 0;
    let o1 = get_test_observation();

    assert_eq!(o1.id(), id);
}

#[test]
fn test_effect_to_string() {
    let id = 0;
    let o1 = get_test_observation();
    let observation = 14.0;
    let observed_effect = 1.0;

    let expected = format!(
        "Observation {{ id: {id},observation: {observation},observed effect: {observed_effect}}}"
    );
    let actual = o1.to_string();

    assert_eq!(actual, expected);
}
