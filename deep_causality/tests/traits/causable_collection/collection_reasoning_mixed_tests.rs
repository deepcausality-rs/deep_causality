/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::{AggregateLogic, CausableCollectionReasoning, PropagatingEffect};

#[test]
fn test_evaluate_mixed_propagation() {
    let col = test_utils::get_deterministic_test_causality_vec();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_mixed(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_mixed(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_mixed_uncertain_bool_propagation_all() {
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
}
