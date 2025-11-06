/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    AggregateLogic, BaseCausaloid, CausableCollectionExplaining, CausableCollectionReasoning,
    PropagatingEffect,
};

#[test]
fn test_get_all_items_empty() {
    let col: [BaseCausaloid; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);
}

#[test]
fn test_explain_deterministic_propagation_empty() {
    let col: [BaseCausaloid; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);
    let res = col.explain();
    assert!(res.is_err());
}

#[test]
fn test_evaluate_deterministic_propagation_empty() {
    let col: [BaseCausaloid; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);

    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_deterministic(&effect_fail, &AggregateLogic::All);
    assert!(res.is_err());
}

#[test]
fn test_evaluate_probabilistic_propagation_empty() {
    let col: [BaseCausaloid; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);

    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_probabilistic(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_err());
}

#[test]
fn test_evaluate_probabilistic_mixed_empty() {
    let col: [BaseCausaloid; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);

    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_mixed(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_err());
}
