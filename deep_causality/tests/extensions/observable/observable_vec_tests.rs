/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{NumericalValue, ObservableReasoning, Observation};

use deep_causality::utils_test::test_utils::*;

#[test]
fn test_add() {
    let mut col = get_test_obs_vec();
    assert_eq!(5, col.len());
    assert!(!col.is_empty());

    let o6 = get_test_observation();
    col.push(o6);
    assert_eq!(6, col.len());
}

#[test]
fn test_number_observation() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    let target_threshold = 10.0 as NumericalValue;
    let target_effect = 1.0 as NumericalValue;
    let total_observation = observations.number_observation(target_threshold, target_effect);
    assert_eq!(3.0, total_observation);
}

#[test]
fn test_percent_observation() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    let target_threshold = 10.0;
    let target_effect = 1.0;
    let percent_observation = observations.percent_observation(target_threshold, target_effect);
    assert_eq!(0.6, percent_observation);
}

#[test]
fn test_number_non_observation() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    let target_threshold = 10.0 as NumericalValue;
    let target_effect = 1.0 as NumericalValue;
    let total_non_observation =
        observations.number_non_observation(target_threshold, target_effect);
    assert_eq!(2.0, total_non_observation);
}

#[test]
fn test_percent_non_observation() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    let target_threshold = 10.0;
    let target_effect = 1.0;
    let percent_non_observation =
        observations.percent_non_observation(target_threshold, target_effect);
    assert_eq!(0.4, percent_non_observation);
}

#[test]
fn test_get_all_items() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    let all_items = observations.get_all_items();
    let exp_len = observations.len();
    let act_len = all_items.len();
    assert_eq!(exp_len, act_len);
}

#[test]
fn test_len() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    assert_eq!(5, observations.len());
}

#[test]
fn test_is_empty() {
    let observations = get_test_obs_vec();
    assert!(!observations.is_empty());

    assert!(!observations.is_empty());

    let empty: Vec<Observation> = Vec::new();
    assert!(ObservableReasoning::is_empty(&empty));
}
