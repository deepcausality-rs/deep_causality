/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::{
    AggregateLogic, BaseCausaloidVec, CausableCollectionReasoning, PropagatingEffect,
};

#[test]
fn test_evaluate_uncertain_propagation_err_empty() {
    let col: BaseCausaloidVec = Vec::new();

    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_err());
}

#[test]
fn test_evaluate_uncertain_propagation_err_invalid_effect() {
    let col = test_utils::get_deterministic_test_causality_vec();

    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_err());
}

#[test]
fn test_evaluate_uncertain_float_propagation_all() {
    let col = test_utils::get_uncertain_float_test_causality_vec();

    // Case 1: All succeed.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();

    let uncertain_bool = match res_success {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_success),
    };

    // Since all inputs are point(true), the result of AND should be deterministically true.
    let final_bool = uncertain_bool.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool);

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), the result of AND should be deterministically false.
    let final_bool_fail = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(!final_bool_fail);
}

#[test]
fn test_evaluate_uncertain_bool_propagation_all() {
    let col = test_utils::get_uncertain_bool_test_causality_vec();

    // Case 1: All succeed.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();

    let uncertain_bool = match res_success {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_success),
    };

    // Since all inputs are point(true), the result of AND should be deterministically true.
    let final_bool = uncertain_bool.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool);

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), the result of AND should be deterministically false.
    let final_bool_fail = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(!final_bool_fail);
}

#[test]
fn test_evaluate_uncertain_bool_propagation_any() {
    let col = test_utils::get_uncertain_bool_test_causality_vec();

    // Case 1: All succeed.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();

    let uncertain_bool = match res_success {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_success),
    };

    // Since all inputs are point(true), the result of AND should be deterministically true.
    let final_bool = uncertain_bool.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool);

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), the result of AND should be deterministically false.
    let final_bool_fail = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(!final_bool_fail);
}

#[test]
fn test_evaluate_uncertain_bool_propagation_some() {
    let col = test_utils::get_uncertain_bool_test_causality_vec();

    // Case 1: All succeed.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::Some(2), 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();

    let uncertain_bool = match res_success {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_success),
    };

    // Since all inputs are point(true), the result of AND should be deterministically true.
    let final_bool = uncertain_bool.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool);

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::Some(2), 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), the result of AND should be deterministically false.
    let final_bool_fail = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(!final_bool_fail);
}

#[test]
fn test_evaluate_uncertain_bool_propagation_none() {
    let col = test_utils::get_uncertain_bool_test_causality_vec();

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), then aggregation NONE evaluates to true.
    let final_bool_none = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool_none);
}
