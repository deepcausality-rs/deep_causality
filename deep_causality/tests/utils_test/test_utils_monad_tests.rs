/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_monad;
use deep_causality_core::EffectValue;
use deep_causality_haft::LogSize;

#[test]
fn test_smoking_logic() {
    // Case 1: Below threshold
    let input_low = EffectValue::from(0.1);
    let res_low = test_utils_monad::smoking_logic(input_low, (), None);
    assert!(res_low.is_ok());
    assert!(!res_low.value.into_value().unwrap()); // Expect false
    assert!(!res_low.logs.is_empty());

    // Case 2: Above threshold (0.6)
    let input_high = EffectValue::from(0.7);
    let res_high = test_utils_monad::smoking_logic(input_high, (), None);
    assert!(res_high.is_ok());
    assert!(res_high.value.into_value().unwrap()); // Expect true
    assert!(!res_high.logs.is_empty());

    // Case 3: Error/None input defaults to 0.0 -> false
    let input_none = EffectValue::None;
    let res_none = test_utils_monad::smoking_logic(input_none, (), None);
    assert!(res_none.is_ok());
    assert!(!res_none.value.into_value().unwrap());
}

#[test]
fn test_tar_logic() {
    // Case 1: True input
    let input_true = EffectValue::from(true);
    let res_true = test_utils_monad::tar_logic(input_true, (), None);
    assert!(res_true.is_ok());
    assert!(res_true.value.into_value().unwrap());
    assert!(!res_true.logs.is_empty());

    // Case 2: False input
    let input_false = EffectValue::from(false);
    let res_false = test_utils_monad::tar_logic(input_false, (), None);
    assert!(res_false.is_ok());
    assert!(!res_false.value.into_value().unwrap());

    // Case 3: None input defaults to false
    let input_none = EffectValue::None;
    let res_none = test_utils_monad::tar_logic(input_none, (), None);
    assert!(res_none.is_ok());
    assert!(!res_none.value.into_value().unwrap());
}

#[test]
fn test_error_logic() {
    let input = EffectValue::from(true);
    let res = test_utils_monad::error_logic(input, (), None);
    assert!(res.is_err());
    assert!(res.error.unwrap().to_string().contains("Simulated error"));
    assert!(!res.logs.is_empty());
}
