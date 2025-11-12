/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    CausalEffectLog, CausalMonad, CausalPropagatingEffect, CausalityError, EffectValue,
    PropagatingEffect,
};
use deep_causality_haft::{Applicative, Functor, Monad, MonadEffect3};

// A simple function to use in tests
fn add_one(effect: PropagatingEffect) -> PropagatingEffect {
    effect.bind(|value| {
        if let EffectValue::Numerical(num) = value {
            let mut new_effect = PropagatingEffect::from_numerical(num + 1.0);
            new_effect.logs.add_entry("Added one");
            new_effect
        } else {
            PropagatingEffect::from_error(CausalityError::new("Not a number".into()))
        }
    })
}

#[test]
fn test_bind_chaining() {
    let effect1 = PropagatingEffect::from_numerical(10.0);
    let effect2 = add_one(effect1);
    let effect3 = add_one(effect2);

    assert!(effect3.is_ok());
    assert_eq!(effect3.value, EffectValue::Numerical(12.0));
}

#[test]
fn test_bind_short_circuit() {
    let effect1 = PropagatingEffect::from_error(CausalityError::new("Initial error".into()));
    let effect2 = add_one(effect1);

    assert!(effect2.is_err());
    assert_eq!(
        effect2.error.unwrap().to_string(),
        "CausalityError: Initial error"
    );
}

#[test]
fn test_bind_log_aggregation() {
    let mut effect1 = PropagatingEffect::from_numerical(10.0);
    effect1.logs.add_entry("Initial log");

    let effect2 = add_one(effect1);

    assert!(effect2.has_log());
    let explanation = effect2.explain();
    assert!(explanation.contains("Initial log"));
    assert!(explanation.contains("Added one"));
}

#[test]
fn test_functor_fmap() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let effect = CausalMonad::pure(10);
    let new_effect = Witness::fmap(effect, |x| x.to_string());

    assert_eq!(new_effect.value, "10".to_string());
    assert!(new_effect.is_ok());
}

#[test]
fn test_functor_fmap_preserves_error() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let effect: CausalPropagatingEffect<i32, CausalityError, CausalEffectLog> =
        CausalPropagatingEffect {
            value: 0,
            error: Some(CausalityError::new("Test error".into())),
            logs: CausalEffectLog::new(),
        };

    let new_effect = Witness::fmap(effect, |x| x.to_string());

    assert!(new_effect.is_err());
    assert_eq!(
        new_effect.error.unwrap().to_string(),
        "CausalityError: Test error"
    );
}

#[test]
fn test_applicative_pure() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;
    let pure_effect = Witness::pure(42);

    assert_eq!(pure_effect.value, 42);
    assert!(pure_effect.is_ok());
    assert!(!pure_effect.has_log());
}

#[test]
fn test_applicative_apply() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let f = |x: i32| x + 1;
    let mut f_effect = Witness::pure(f);
    f_effect.logs.add_entry("Log from f");

    let mut v_effect = Witness::pure(1);
    v_effect.logs.add_entry("Log from v");

    let result_effect = Witness::apply(f_effect, v_effect);

    assert_eq!(result_effect.value, 2);
    assert!(result_effect.is_ok());
    assert!(result_effect.explain().contains("Log from f"));
    assert!(result_effect.explain().contains("Log from v"));
}

#[test]
fn test_applicative_apply_error_in_func() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let f = |x: i32| x + 1;
    let f_effect: CausalPropagatingEffect<_, _, _> = CausalPropagatingEffect {
        value: f,
        error: Some(CausalityError::new("Function error".into())),
        logs: CausalEffectLog::new(),
    };

    let v_effect = Witness::pure(1);
    let result_effect = Witness::apply(f_effect, v_effect);

    assert!(result_effect.is_err());
    assert_eq!(
        result_effect.error.unwrap().to_string(),
        "CausalityError: Function error"
    );
}

#[test]
fn test_applicative_apply_error_in_value() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let f = |x: i32| x + 1;
    let f_effect = Witness::pure(f);

    let v_effect: CausalPropagatingEffect<_, _, _> = CausalPropagatingEffect {
        value: 1,
        error: Some(CausalityError::new("Value error".into())),
        logs: CausalEffectLog::new(),
    };

    let result_effect = Witness::apply(f_effect, v_effect);

    assert!(result_effect.is_err());
    assert_eq!(
        result_effect.error.unwrap().to_string(),
        "CausalityError: Value error"
    );
}

#[test]
fn test_monad_bind() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let effect = Witness::pure(10);
    let new_effect = Witness::bind(effect, |x| Witness::pure(x + 5));

    assert_eq!(new_effect.value, 15);
    assert!(new_effect.is_ok());
}

#[test]
fn test_monad_bind_with_logs() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let mut effect = Witness::pure(10);
    effect.logs.add_entry("Initial");

    let new_effect = Witness::bind(effect, |x| {
        let mut next = Witness::pure(x + 5);
        next.logs.add_entry("Next");
        next
    });

    assert_eq!(new_effect.value, 15);
    assert!(new_effect.explain().contains("Initial"));
    assert!(new_effect.explain().contains("Next"));
}

#[test]
fn test_monad_bind_short_circuits() {
    type Witness = deep_causality::PropagatingEffectWitness<CausalityError, CausalEffectLog>;

    let effect: CausalPropagatingEffect<i32, _, _> = CausalPropagatingEffect {
        value: 0,
        error: Some(CausalityError::new("Monad error".into())),
        logs: CausalEffectLog::new(),
    };

    let new_effect = Witness::bind(effect, |x| Witness::pure(x + 5));

    assert!(new_effect.is_err());
    assert_eq!(
        new_effect.error.unwrap().to_string(),
        "CausalityError: Monad error"
    );
}
