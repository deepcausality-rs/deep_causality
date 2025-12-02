/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalityError, CausalityErrorEnum, EffectLog, EffectValue, PropagatingEffect,
    PropagatingEffectWitness,
};
use deep_causality_haft::{Applicative, Functor, HKT, LogAddEntry, LogSize, Monad};

type TestPropagatingEffect<T> = PropagatingEffect<T>;
type TestWitness = PropagatingEffectWitness<CausalityError, EffectLog>;

fn setup_effect_with_value<T: Default + Clone>(value: T) -> TestPropagatingEffect<T> {
    PropagatingEffect {
        value: EffectValue::Value(value),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    }
}

fn setup_effect_with_error<T: Default + Clone>(
    error_enum: CausalityErrorEnum,
) -> TestPropagatingEffect<T> {
    PropagatingEffect {
        value: EffectValue::None,
        state: (),
        context: None,
        error: Some(CausalityError::new(error_enum)),
        logs: EffectLog::new(),
    }
}

fn setup_effect_with_none_value<T: Default + Clone>() -> TestPropagatingEffect<T> {
    PropagatingEffect {
        value: EffectValue::None,
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    }
}

#[test]
fn test_hkt_type_alias() {
    // This test ensures the HKT type alias resolves correctly.
    let effect: <TestWitness as HKT>::Type<i32> = setup_effect_with_value(10);
    assert_eq!(effect.value.as_value(), Some(&10));
}

// Functor Tests
#[test]
fn test_functor_fmap_with_value() {
    let m_a = setup_effect_with_value(5);
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert_eq!(m_b.value.as_value(), Some(&10));
    assert!(m_b.is_ok());
}

#[test]
fn test_functor_fmap_with_error() {
    let m_a = setup_effect_with_error::<i32>(CausalityErrorEnum::TypeConversionError);
    let f = |a: i32| a * 2; // This function should not be called
    let m_b = TestWitness::fmap(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error.unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value.is_none());
}

#[test]
fn test_functor_fmap_with_none_value() {
    let m_a = setup_effect_with_none_value::<i32>();
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_functor_fmap_logs_preserved() {
    let mut logs = EffectLog::new();
    logs.add_entry("Initial log");
    let m_a = PropagatingEffect {
        value: EffectValue::Value(5),
        state: (),
        context: None,
        error: None,
        logs,
    };
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert_eq!(m_b.logs.len(), 1);
}

// Applicative Tests
#[test]
fn test_applicative_pure() {
    let effect = TestWitness::pure(100);
    assert_eq!(effect.value.as_value(), Some(&100));
    assert!(effect.is_ok());
    assert!(effect.logs.is_empty());
}

#[test]
fn test_applicative_apply_with_values() {
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect {
        value: EffectValue::Value(f_ab_func),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert_eq!(m_b.value.as_value(), Some(&11));
    assert!(m_b.is_ok());
}

#[test]
fn test_applicative_apply_with_f_ab_error() {
    // Explicitly construct f_ab as its 'T' (a function) does not implement Default
    let f_ab = PropagatingEffect {
        value: EffectValue::<fn(i32) -> i32>::None, // Explicitly type None for the function
        state: (),
        context: None,
        error: Some(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error.unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value.is_none());
}

#[test]
fn test_applicative_apply_with_f_a_error() {
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect {
        value: EffectValue::Value(f_ab_func),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_error(CausalityErrorEnum::InternalLogicError);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_applicative_apply_with_both_errors() {
    // Explicitly construct f_ab as its 'T' (a function) does not implement Default
    let f_ab = PropagatingEffect {
        value: EffectValue::<fn(i32) -> i32>::None, // Explicitly type None for the function
        state: (),
        context: None,
        error: Some(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_error(CausalityErrorEnum::InternalLogicError);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error.unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value.is_none());
}

#[test]
fn test_applicative_apply_with_f_ab_none_value() {
    let f_ab = PropagatingEffect {
        value: EffectValue::<fn(i32) -> i32>::None, // Explicitly type None for the function
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_applicative_apply_with_f_a_none_value() {
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect {
        value: EffectValue::Value(f_ab_func),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let f_a = setup_effect_with_none_value::<i32>();
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_applicative_apply_logs_combined() {
    let mut logs1 = EffectLog::new();
    logs1.add_entry("Log 1");
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect {
        value: EffectValue::Value(f_ab_func),
        state: (),
        context: None,
        error: None,
        logs: logs1,
    };

    let mut logs2 = EffectLog::new();
    logs2.add_entry("Log 2");
    let f_a = PropagatingEffect {
        value: EffectValue::Value(10),
        state: (),
        context: None,
        error: None,
        logs: logs2,
    };

    let m_b = TestWitness::apply(f_ab, f_a);
    assert_eq!(m_b.logs.len(), 2);
}

// Monad Tests
#[test]
fn test_monad_bind_with_value() {
    let m_a = setup_effect_with_value(5);
    let f = |a: i32| setup_effect_with_value(a * 2);
    let m_b = TestWitness::bind(m_a, f);
    assert_eq!(m_b.value.as_value(), Some(&10));
    assert!(m_b.is_ok());
}

#[test]
fn test_monad_bind_with_error_in_m_a() {
    let m_a = setup_effect_with_error::<i32>(CausalityErrorEnum::TypeConversionError);
    let f = |a: i32| setup_effect_with_value(a * 2); // This function should not be called
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error.unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value.is_none());
}

#[test]
fn test_monad_bind_with_error_in_f_result() {
    let m_a = setup_effect_with_value(5);
    let f = |_a: i32| setup_effect_with_error::<i32>(CausalityErrorEnum::InternalLogicError); // Explicitly type T
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_monad_bind_with_none_value_in_m_a() {
    let m_a = setup_effect_with_none_value::<i32>();
    let f = |a: i32| setup_effect_with_value(a * 2);
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(m_b.error.unwrap().0, CausalityErrorEnum::InternalLogicError);
    assert!(m_b.value.is_none());
}

#[test]
fn test_monad_bind_logs_combined() {
    let mut logs1 = EffectLog::new();
    logs1.add_entry("Log 1 from m_a");
    let m_a = PropagatingEffect {
        value: EffectValue::Value(5),
        state: (),
        context: None,
        error: None,
        logs: logs1,
    };

    let f = |a: i32| {
        let mut logs2 = EffectLog::new();
        logs2.add_entry("Log 2 from f");
        PropagatingEffect {
            value: EffectValue::Value(a * 2),
            state: (),
            context: None,
            error: None,
            logs: logs2,
        }
    };

    let m_b = TestWitness::bind(m_a, f);
    assert_eq!(m_b.logs.len(), 2);
}
