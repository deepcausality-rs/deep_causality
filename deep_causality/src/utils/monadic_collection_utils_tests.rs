/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::monadic_collection_utils::{
    aggregate_deterministic, aggregate_effects, aggregate_probabilistic, aggregate_uncertain,
};
use crate::{AggregateLogic, CausalityError, EffectValue};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{UncertainBool, UncertainF64};

#[test]
fn test_aggregate_effects_empty_collection() {
    let effects: Vec<EffectValue> = Vec::new();
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_effects(effects, &logic, threshold);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError("Cannot aggregate empty collection".to_string())
    );
}

#[test]
fn test_aggregate_effects_uncertain_strategy() {
    let effects = vec![
        EffectValue::UncertainBool(UncertainBool::point(true)),
        EffectValue::Boolean(false),
    ];
    let logic = AggregateLogic::Any;
    let threshold = Some(0.5f64);
    let result = aggregate_effects(effects, &logic, threshold);
    dbg!(&result);
    assert!(result.is_err());
}

#[test]
fn test_aggregate_effects_probabilistic_strategy() {
    let effects = vec![EffectValue::Probabilistic(0.8), EffectValue::Boolean(true)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_effects(effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::Probabilistic(_)));
}

#[test]
fn test_aggregate_effects_numerical_strategy() {
    let effects = vec![EffectValue::Numerical(0.7), EffectValue::Boolean(true)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_effects(effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::Probabilistic(_)));
}

#[test]
fn test_aggregate_effects_deterministic_strategy() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Boolean(false)];
    let logic = AggregateLogic::Any;
    let threshold = Some(0.5f64);
    let result = aggregate_effects(effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::Boolean(_)));
    assert_eq!(result, EffectValue::Boolean(true));
}

// --- Deterministic Aggregation Tests ---

#[test]
fn test_aggregate_deterministic_all_true() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Boolean(true)];
    let logic = AggregateLogic::All;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(true));
}

#[test]
fn test_aggregate_deterministic_all_false() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Boolean(false)];
    let logic = AggregateLogic::All;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(false));
}

#[test]
fn test_aggregate_deterministic_any_true() {
    let effects = vec![EffectValue::Boolean(false), EffectValue::Boolean(true)];
    let logic = AggregateLogic::Any;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(true));
}

#[test]
fn test_aggregate_deterministic_any_false() {
    let effects = vec![EffectValue::Boolean(false), EffectValue::Boolean(false)];
    let logic = AggregateLogic::Any;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(false));
}

#[test]
fn test_aggregate_deterministic_none_true() {
    let effects = vec![EffectValue::Boolean(false), EffectValue::Boolean(false)];
    let logic = AggregateLogic::None;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(true));
}

#[test]
fn test_aggregate_deterministic_none_false() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Boolean(false)];
    let logic = AggregateLogic::None;
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(false));
}

#[test]
fn test_aggregate_deterministic_some_true() {
    let effects = vec![
        EffectValue::Boolean(true),
        EffectValue::Boolean(false),
        EffectValue::Boolean(true),
    ];
    let logic = AggregateLogic::Some(2);
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(true));
}

#[test]
fn test_aggregate_deterministic_some_false() {
    let effects = vec![
        EffectValue::Boolean(true),
        EffectValue::Boolean(false),
        EffectValue::Boolean(false),
    ];
    let logic = AggregateLogic::Some(2);
    let result = aggregate_deterministic(&effects, &logic).unwrap();
    assert_eq!(result, EffectValue::Boolean(false));
}

#[test]
fn test_aggregate_deterministic_error_unsupported_type() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Probabilistic(0.5)];
    let logic = AggregateLogic::All;
    let result = aggregate_deterministic(&effects, &logic);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError(
            "Deterministic aggregation requires all effects to be Deterministic, but found Probabilistic(0.5)".to_string()
        )
    );
}

// --- Probabilistic Aggregation Tests ---

#[test]
fn test_aggregate_probabilistic_all() {
    let effects = vec![
        EffectValue::Probabilistic(0.5),
        EffectValue::Probabilistic(0.6),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.3)); // 0.5 * 0.6
}

#[test]
fn test_aggregate_probabilistic_any() {
    let effects = vec![
        EffectValue::Probabilistic(0.5),
        EffectValue::Probabilistic(0.6),
    ];
    let logic = AggregateLogic::Any;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.8)); // 1 - (1-0.5)*(1-0.6) = 1 - 0.5*0.4 = 1 - 0.2 = 0.8
}

#[test]
fn test_aggregate_probabilistic_none() {
    let effects = vec![
        EffectValue::Probabilistic(0.5),
        EffectValue::Probabilistic(0.6),
    ];
    let logic = AggregateLogic::None;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.19999999999999996)); // 1 - (1 - (1-0.5)*(1-0.6)) = 1 - 0.8 = 0.2
}

#[test]
fn test_aggregate_probabilistic_some_true() {
    let effects = vec![
        EffectValue::Probabilistic(0.6),
        EffectValue::Probabilistic(0.4),
        EffectValue::Probabilistic(0.7),
    ];
    let logic = AggregateLogic::Some(2);
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(1.0)); // 0.6 and 0.7 are > 0.5, so count is 2 >= 2
}

#[test]
fn test_aggregate_probabilistic_some_false() {
    let effects = vec![
        EffectValue::Probabilistic(0.6),
        EffectValue::Probabilistic(0.4),
        EffectValue::Probabilistic(0.3),
    ];
    let logic = AggregateLogic::Some(2);
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.0)); // Only 0.6 > 0.5, so count is 1 < 2
}

#[test]
fn test_aggregate_probabilistic_with_deterministic() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Probabilistic(0.5)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.5)); // 1.0 * 0.5
}

#[test]
fn test_aggregate_probabilistic_with_numerical() {
    let effects = vec![EffectValue::Numerical(0.7), EffectValue::Probabilistic(0.5)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    assert_eq!(result, EffectValue::Probabilistic(0.35)); // 0.7 * 0.5
}

#[test]
fn test_aggregate_probabilistic_with_uncertain_bool() {
    let effects = vec![
        EffectValue::UncertainBool(UncertainBool::bernoulli(0.7)),
        EffectValue::Probabilistic(0.5),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    // Bernoulli(0.7) estimate_probability(100) should be close to 0.7
    // So result should be close to 0.7 * 0.5 = 0.35
    if let EffectValue::Probabilistic(p) = result {
        assert!((p - 0.35).abs() < 0.1); // Allow some tolerance for sampling
    } else {
        panic!("Expected Probabilistic effect");
    }
}

#[test]
fn test_aggregate_probabilistic_with_uncertain_float() {
    let effects = vec![
        EffectValue::UncertainFloat(UncertainF64::normal(0.6, 0.1)),
        EffectValue::Probabilistic(0.5),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold).unwrap();
    // UncertainFloat::normal(0.6, 0.1) estimate_probability_exceeds(0.5, 100) should be > 0.5
    // So result should be close to (prob > 0.5) * 0.5
    if let EffectValue::Probabilistic(p) = result {
        // The probability of a normal distribution with mean 0.6 and std_dev 0.1
        // exceeding 0.5 is quite high.
        assert!(p > 0.25 && p < 0.5); // Should be roughly 0.5 * (prob > 0.5)
    } else {
        panic!("Expected Probabilistic effect");
    }
}

#[test]
fn test_aggregate_probabilistic_error_unsupported_type() {
    let effects = vec![
        EffectValue::Probabilistic(0.5),
        EffectValue::Tensor(CausalTensor::new(vec![1.0], vec![1]).unwrap()),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_probabilistic(&effects, &logic, threshold);
    assert!(result.is_err());
    dbg!(&result);
    assert_eq!(
        result.unwrap_err(),
        CausalityError(
            "Unsupported type for probabilistic aggregation: Tensor(CausalTensor { data: [1.0], shape: [1], strides: [1] })".to_string()
        )
    );
}

// --- Uncertain Aggregation Tests ---

#[test]
fn test_aggregate_uncertain_missing_threshold() {
    let effects = vec![EffectValue::UncertainBool(UncertainBool::point(true))];
    let logic = AggregateLogic::All;
    let threshold = None;
    let result = aggregate_uncertain(&effects, &logic, threshold);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError("Threshold is required for uncertain aggregation".to_string())
    );
}

#[test]
fn test_aggregate_uncertain_no_uncertain_compatible_effects() {
    let effects = vec![EffectValue::Boolean(true)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError("Unsupported type for uncertain aggregation: Boolean(true)".to_string())
    );
}

#[test]
fn test_aggregate_uncertain_all() {
    let ub1 = UncertainBool::bernoulli(0.8);
    let ub2 = UncertainBool::bernoulli(0.9);
    let effects = vec![
        EffectValue::UncertainBool(ub1.clone()),
        EffectValue::UncertainBool(ub2.clone()),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::UncertainBool(_)));
    if let EffectValue::UncertainBool(res_ub) = result {
        assert_eq!(res_ub, ub1 & ub2);
    }
}

#[test]
fn test_aggregate_uncertain_any() {
    let ub1 = UncertainBool::bernoulli(0.2);
    let ub2 = UncertainBool::bernoulli(0.3);
    let effects = vec![
        EffectValue::UncertainBool(ub1.clone()),
        EffectValue::UncertainBool(ub2.clone()),
    ];
    let logic = AggregateLogic::Any;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::UncertainBool(_)));
    if let EffectValue::UncertainBool(res_ub) = result {
        assert_eq!(res_ub, ub1 | ub2);
    }
}

#[test]
fn test_aggregate_uncertain_none() {
    let ub1 = UncertainBool::bernoulli(0.2);
    let ub2 = UncertainBool::bernoulli(0.3);
    let effects = vec![
        EffectValue::UncertainBool(ub1.clone()),
        EffectValue::UncertainBool(ub2.clone()),
    ];
    let logic = AggregateLogic::None;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::UncertainBool(_)));
    if let EffectValue::UncertainBool(res_ub) = result {
        assert_eq!(res_ub, !(ub1 | ub2));
    }
}

#[test]
fn test_aggregate_uncertain_some() {
    let ub1 = UncertainBool::bernoulli(0.8); // True
    let ub2 = UncertainBool::bernoulli(0.1); // False
    let ub3 = UncertainBool::bernoulli(0.9); // True
    let effects = vec![
        EffectValue::UncertainBool(ub1),
        EffectValue::UncertainBool(ub2),
        EffectValue::UncertainBool(ub3),
    ];
    let logic = AggregateLogic::Some(2);
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::UncertainBool(_)));
    if let EffectValue::UncertainBool(res_ub) = result {
        // With threshold 0.5, ub1 and ub3 are likely true, ub2 is likely false.
        // So 2 out of 3 are true, which satisfies Some(2).
        assert!(res_ub.to_bool(0.5, 0.95, 0.05, 1000).unwrap());
    }
}

#[test]
fn test_aggregate_uncertain_with_uncertain_float() {
    let uf1 = UncertainF64::normal(0.7, 0.1); // Likely > 0.5
    let uf2 = UncertainF64::normal(0.3, 0.1); // Likely < 0.5
    let effects = vec![
        EffectValue::UncertainFloat(uf1),
        EffectValue::UncertainFloat(uf2),
    ];
    let logic = AggregateLogic::Any;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold).unwrap();
    assert!(matches!(result, EffectValue::UncertainBool(_)));
    if let EffectValue::UncertainBool(res_ub) = result {
        // uf1.greater_than(0.5) is likely true
        // uf2.greater_than(0.5) is likely false
        // Any should be true
        assert!(res_ub.to_bool(0.5, 0.95, 0.05, 1000).unwrap());
    }
}

#[test]
fn test_aggregate_uncertain_error_unsupported_type() {
    let effects = vec![
        EffectValue::UncertainBool(UncertainBool::point(true)),
        EffectValue::Probabilistic(0.5),
    ];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError(
            "Unsupported type for uncertain aggregation: Probabilistic(0.5)".to_string()
        )
    );
}

#[test]
fn test_aggregate_uncertain_empty_u_bools_after_filtering() {
    let effects = vec![EffectValue::Boolean(true), EffectValue::Numerical(1.0)];
    let logic = AggregateLogic::All;
    let threshold = Some(0.5f64);
    let result = aggregate_uncertain(&effects, &logic, threshold);
    dbg!(&result);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        CausalityError("Unsupported type for uncertain aggregation: Boolean(true)".to_string())
    );
}
