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
