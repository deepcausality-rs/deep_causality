/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::{AggregateLogic, CausableCollectionReasoning, PropagatingEffect};

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = test_utils::get_deterministic_test_causality_vec();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_deterministic(&effect_success, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_deterministic(&effect_fail, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}
