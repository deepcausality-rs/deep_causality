/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{NumericalValue, ObservableReasoning, Observation};

pub fn get_test_obs_arr() -> [Observation; 5] {
    let o1 = Observation::new(0, 10.0, 1.0);
    let o2 = Observation::new(1, 10.0, 1.0);
    let o3 = Observation::new(2, 10.0, 1.0);
    let o4 = Observation::new(3, 12.0, 0.0);
    let o5 = Observation::new(4, 14.0, 0.0);
    [o1, o2, o3, o4, o5]
}

#[test]
fn test_number_observation() {
    let observations = get_test_obs_arr();
    assert!(!observations.is_empty());

    let target_threshold = 10.0 as NumericalValue;
    let target_effect = 1.0 as NumericalValue;
    let total_observation = observations.number_observation(target_threshold, target_effect);
    assert_eq!(3.0, total_observation);
}

#[test]
fn test_percent_observation() {
    let observations = get_test_obs_arr();
    assert!(!observations.is_empty());

    let target_threshold = 10.0;
    let target_effect = 1.0;
    let percent_observation = observations.percent_observation(target_threshold, target_effect);
    assert_eq!(0.6, percent_observation);
}

#[test]
fn test_number_non_observation() {
    let observations = get_test_obs_arr();
    assert!(!observations.is_empty());

    let target_threshold = 10.0 as NumericalValue;
    let target_effect = 1.0 as NumericalValue;
    let total_non_observation =
        observations.number_non_observation(target_threshold, target_effect);
    assert_eq!(2.0, total_non_observation);
}

#[test]
fn test_percent_non_observation() {
    let observations = get_test_obs_arr();
    assert!(!observations.is_empty());

    let target_threshold = 10.0;
    let target_effect = 1.0;
    let percent_non_observation =
        observations.percent_non_observation(target_threshold, target_effect);
    assert_eq!(0.4, percent_non_observation);
}

#[test]
fn test_get_all_items() {
    let observations = get_test_obs_arr();
    assert!(!observations.is_empty());

    let all_items = observations.get_all_items();

    let exp_len = observations.len();
    let act_len = all_items.len();
    assert_eq!(exp_len, act_len);
}

#[test]
fn test_len() {
    let col = get_test_obs_arr();
    assert!(!col.is_empty());

    assert_eq!(5, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_obs_arr();
    assert!(!col.is_empty());
}
