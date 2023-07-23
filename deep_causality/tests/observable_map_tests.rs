// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.



use deep_causality::protocols::observable::ObservableReasoning;
use deep_causality::types::alias_types::NumericalValue;
use deep_causality::utils::test_utils;

#[test]
fn test_add() {
    let mut col = test_utils::get_test_obs_map();
    assert_eq!(5, col.len());

    let o6 = test_utils::get_test_observation();
    col.insert(6, o6);
    assert_eq!(6, col.len());
}

#[test]
fn test_number_observation() {
    let observations = test_utils::get_test_obs_map();
    let target_threshold = 10.0 as NumericalValue;
    let target_effect = 1.0 as NumericalValue;
    let total_observation = observations.number_observation(target_threshold, target_effect);
    assert_eq!(3.0, total_observation);
}

#[test]
fn test_percent_observation() {
    let observations = test_utils::get_test_obs_map();
    let target_threshold = 10.0;
    let target_effect = 1.0;
    let percent_observation = observations.percent_observation(target_threshold, target_effect);
    assert_eq!(0.6, percent_observation);
}

#[test]
fn test_get_all_items() {
    let observations = test_utils::get_test_obs_map();
    let all_items = observations.get_all_items();

    let exp_len = observations.len();
    let act_len = all_items.len();
    assert_eq!(exp_len, act_len);
}

#[test]
fn test_len() {
    let col = test_utils::get_test_obs_map();
    assert_eq!(5, col.len());
}

#[test]
fn test_is_empty() {
    let col = test_utils::get_test_obs_map();
    assert!(!col.is_empty());
}
