// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::utils::test_utils::get_test_observation;

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

    let expected = format!("Observation {{ id: {},observation: {},observed effect: {}}}", id,observation,observed_effect);
    let actual = o1.to_string();

    assert_eq!(actual, expected);
}