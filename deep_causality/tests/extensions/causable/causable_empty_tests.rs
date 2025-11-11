/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    AggregateLogic, BaseCausaloid, CausaloidRegistry, MonadicCausableCollection, NumericalValue,
    PropagatingEffect,
};

#[test]
fn test_get_all_items_empty() {
    let col: [BaseCausaloid<NumericalValue, bool>; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);
}

#[test]
fn test_evaluate_deterministic_propagation_empty() {
    let col: [BaseCausaloid<NumericalValue, bool>; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);

    let registry = CausaloidRegistry::new();
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_collection(&registry, &effect_fail, &AggregateLogic::All, None);
    assert!(res.is_err());
}

#[test]
fn test_evaluate_probabilistic_propagation_empty() {
    let col: [BaseCausaloid<NumericalValue, f64>; 0] = [];
    let exp_len = col.len();
    assert_eq!(exp_len, 0);

    let registry = CausaloidRegistry::new();
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_collection(&registry, &effect_fail, &AggregateLogic::All, Some(0.5));
    assert!(res.is_err());
}
