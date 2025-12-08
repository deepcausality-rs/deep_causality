/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionParameterValue, CsmEvaluable, UncertainParameter};
use deep_causality_uncertain::Uncertain;

#[test]
fn test_bool_evaluable() {
    let true_val = true;
    let false_val = false;

    // is_active
    assert!(true_val.is_active(None).unwrap());
    assert!(!false_val.is_active(None).unwrap());

    // to_action_param
    match true_val.to_action_param() {
        ActionParameterValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean parameter"),
    }
}

#[test]
fn test_uncertain_bool_evaluable() {
    let ub_true = Uncertain::<bool>::point(true);
    let ub_false = Uncertain::<bool>::point(false);

    // Implicit conditional (None params)
    assert!(ub_true.is_active(None).unwrap());
    assert!(!ub_false.is_active(None).unwrap());

    // With Parameters
    let params = UncertainParameter::new(0.5, 0.95, 0.05, 100);
    assert!(ub_true.is_active(Some(&params)).unwrap());
    assert!(!ub_false.is_active(Some(&params)).unwrap());

    // to_action_param
    match ub_true.to_action_param() {
        ActionParameterValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean parameter"),
    }
}

#[test]
fn test_uncertain_f64_evaluable() {
    let uf_high = Uncertain::<f64>::point(0.8);
    let uf_low = Uncertain::<f64>::point(0.2);

    // Error on None params
    let res = uf_high.is_active(None);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("UncertainFloat effect requires UncertainParameter")
    );

    // With Parameters (threshold 0.5)
    let params = UncertainParameter::new(0.5, 0.95, 0.05, 100);

    // High > Threshold => Active
    assert!(uf_high.is_active(Some(&params)).unwrap());

    // Low < Threshold => Inactive
    assert!(!uf_low.is_active(Some(&params)).unwrap());

    // to_action_param
    match uf_high.to_action_param() {
        ActionParameterValue::Number(n) => assert!((n - 0.8).abs() < 1e-6),
        _ => panic!("Expected Number parameter"),
    }
}
