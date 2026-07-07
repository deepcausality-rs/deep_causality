/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::AggregateLogic;
use deep_causality::monadic_collection_utils;
use deep_causality_core::CausalEffect;
use deep_causality_uncertain::Uncertain;

#[test]
fn test_aggregate_bool() {
    let true_val = CausalEffect::value(true);
    let false_val = CausalEffect::value(false);

    // All
    let inputs = vec![true_val.clone(), true_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(true));

    let inputs = vec![true_val.clone(), false_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(false));

    // Any
    let inputs = vec![false_val.clone(), false_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Any, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(false));

    let inputs = vec![false_val.clone(), true_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Any, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(true));

    // None
    let inputs = vec![false_val.clone(), false_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(true));

    let inputs = vec![true_val.clone(), false_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(false));

    // Some(k)
    let inputs = vec![true_val.clone(), true_val.clone(), false_val.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Some(2), None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(true));

    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Some(3), None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(false));
}

#[test]
fn test_aggregate_f64() {
    let p1 = CausalEffect::value(0.5);
    let p2 = CausalEffect::value(0.5);

    // All (Product)
    let inputs = vec![p1.clone(), p2.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_ok());
    // 0.5 * 0.5 = 0.25
    assert_eq!(res.unwrap(), CausalEffect::value(0.25));

    // Any (1 - product(1-p)) => 1 - (0.5 * 0.5) = 0.75
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Any, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(0.75));

    // None (1 - Any) => 1 - 0.75 = 0.25
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), CausalEffect::value(0.25));

    // Some(k) - threshold 0.5 forced in impl logic for counting
    // 0.5 is NOT > 0.5, so count is 0.
    let p_high = CausalEffect::value(0.9);
    let inputs = vec![p_high.clone(), p1.clone()]; // one > 0.5, one not
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Some(1), None);
    assert!(res.is_ok());
    // count is 1, k=1 -> 1.0
    assert_eq!(res.unwrap(), CausalEffect::value(1.0));

    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Some(2), None);
    assert!(res.is_ok());
    // count is 1, k=2 -> 0.0
    assert_eq!(res.unwrap(), CausalEffect::value(0.0));
}

#[test]
fn test_aggregate_uncertain_bool() {
    let ub_true = Uncertain::<bool>::point(true);
    let ub_false = Uncertain::<bool>::point(false);
    let ev_true = CausalEffect::value(ub_true);
    let ev_false = CausalEffect::value(ub_false);

    let threshold = Some(0.5);

    // All
    let inputs = vec![ev_true.clone(), ev_true.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, threshold);
    assert!(res.is_ok());
    // True & True -> True
    // We check via to_bool for simplicity or just success
    let val = res.unwrap().into_value().unwrap();
    // Assuming to_bool logic or point logic holds
    assert!(val.to_bool(0.5, 0.95, 0.05, 100).unwrap());

    // Any
    let inputs = vec![ev_false.clone(), ev_true.clone()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Any, threshold);
    assert!(res.is_ok());
    let val = res.unwrap().into_value().unwrap();
    assert!(val.to_bool(0.5, 0.95, 0.05, 100).unwrap());

    // None
    let inputs = vec![ev_false.clone(), ev_false.clone()];
    let res =
        monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::None, threshold);
    assert!(res.is_ok());
    let val = res.unwrap().into_value().unwrap();
    assert!(val.to_bool(0.5, 0.95, 0.05, 100).unwrap());

    // Some(k)
    let inputs = vec![ev_true.clone(), ev_true.clone(), ev_false.clone()];
    let res =
        monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::Some(2), threshold);
    assert!(res.is_ok());
    let val = res.unwrap().into_value().unwrap();
    assert!(val.to_bool(0.5, 0.95, 0.05, 100).unwrap());
}

#[test]
fn test_aggregate_uncertain_f64_error() {
    let uf = Uncertain::<f64>::point(0.5);
    let ev = CausalEffect::value(uf);
    let inputs = vec![ev];

    // Should return error as it is not supported
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("not supported"));
}

#[test]
fn test_aggregate_bool_non_value_errors() {
    // A non-`Value` variant (`None`) must trigger the "Expected Value(bool)" error path.
    let inputs: Vec<CausalEffect<bool>> = vec![CausalEffect::value(true), CausalEffect::none()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Expected Value(bool)")
    );
}

#[test]
fn test_aggregate_f64_non_value_errors() {
    let inputs: Vec<CausalEffect<f64>> = vec![CausalEffect::value(0.5), CausalEffect::none()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Expected Value(f64)"));
}

#[test]
fn test_aggregate_uncertain_bool_missing_threshold_errors() {
    let ub = Uncertain::<bool>::point(true);
    let inputs = vec![CausalEffect::value(ub)];
    // No threshold supplied -> must error.
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Threshold is required")
    );
}

#[test]
fn test_aggregate_uncertain_bool_non_value_errors() {
    let inputs: Vec<CausalEffect<deep_causality_uncertain::UncertainBool>> =
        vec![CausalEffect::none()];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, Some(0.5));
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Expected Value(UncertainBool)")
    );
}

#[test]
fn test_empty_collection_error() {
    let inputs: Vec<CausalEffect<bool>> = vec![];
    let res = monadic_collection_utils::aggregate_effects(&inputs, &AggregateLogic::All, None);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Cannot aggregate empty collection")
    );
}
