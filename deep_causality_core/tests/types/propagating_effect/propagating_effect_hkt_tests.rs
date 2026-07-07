/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffect, CausalityError, CausalityErrorEnum, EffectLog, PropagatingEffect,
    PropagatingEffectWitness,
};
use deep_causality_haft::{Applicative, Functor, HKT, LogAddEntry, LogSize, Monad, Pure};

type TestPropagatingEffect<T> = PropagatingEffect<T>;
type TestWitness = PropagatingEffectWitness<CausalityError, EffectLog>;

fn setup_effect_with_value<T: Default + Clone>(value: T) -> TestPropagatingEffect<T> {
    PropagatingEffect::new(Ok(CausalEffect::value(value)), (), None, EffectLog::new())
}

fn setup_effect_with_error<T: Default + Clone>(
    error_enum: CausalityErrorEnum,
) -> TestPropagatingEffect<T> {
    PropagatingEffect::new(
        Err(CausalityError::new(error_enum)),
        (),
        None,
        EffectLog::new(),
    )
}

fn setup_effect_with_none_value<T: Default + Clone>() -> TestPropagatingEffect<T> {
    PropagatingEffect::new(Ok(CausalEffect::none()), (), None, EffectLog::new())
}

#[test]
fn test_hkt_type_alias() {
    // This test ensures the HKT type alias resolves correctly.
    let effect: <TestWitness as HKT>::Type<i32> = setup_effect_with_value(10);
    assert_eq!(effect.value(), Some(&10));
}

// Functor Tests
#[test]
fn test_functor_fmap_with_value() {
    let m_a = setup_effect_with_value(5);
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert_eq!(m_b.value(), Some(&10));
    assert!(m_b.is_ok());
}

#[test]
fn test_functor_fmap_with_error() {
    let m_a = setup_effect_with_error::<i32>(CausalityErrorEnum::TypeConversionError);
    let f = |a: i32| a * 2; // This function should not be called
    let m_b = TestWitness::fmap(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_functor_fmap_with_none_value_passes_through() {
    // Totality invariant: `fmap` is total — a `None` effect passes through unchanged (no value, no
    // fabricated error). The continuation is not invoked.
    let m_a = setup_effect_with_none_value::<i32>();
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert!(m_b.is_ok());
    assert!(m_b.value().is_none());
    assert!(m_b.effect().unwrap().is_none());
}

#[test]
fn test_functor_fmap_logs_preserved() {
    let mut logs = EffectLog::new();
    logs.add_entry("Initial log");
    let m_a = PropagatingEffect::new(Ok(CausalEffect::value(5)), (), None, logs);
    let f = |a: i32| a * 2;
    let m_b = TestWitness::fmap(m_a, f);
    assert_eq!(m_b.logs().len(), 1);
}

// Applicative Tests
#[test]
fn test_applicative_pure() {
    let effect = TestWitness::pure(100);
    assert_eq!(effect.value(), Some(&100));
    assert!(effect.is_ok());
    assert!(effect.logs().is_empty());
}

#[test]
fn test_applicative_apply_with_values() {
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect::new(
        Ok(CausalEffect::value(f_ab_func)),
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert_eq!(m_b.value(), Some(&11));
    assert!(m_b.is_ok());
}

#[test]
fn test_applicative_apply_with_f_ab_error() {
    // Explicitly annotate f_ab as its 'T' (a function) cannot be inferred from an error
    let f_ab: PropagatingEffect<fn(i32) -> i32> = PropagatingEffect::new(
        Err(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_applicative_apply_with_f_a_error() {
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect::new(
        Ok(CausalEffect::value(f_ab_func)),
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_error(CausalityErrorEnum::InternalLogicError);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_applicative_apply_with_both_errors() {
    // Explicitly annotate f_ab as its 'T' (a function) cannot be inferred from an error
    let f_ab: PropagatingEffect<fn(i32) -> i32> = PropagatingEffect::new(
        Err(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_error(CausalityErrorEnum::InternalLogicError);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_err());
    // f_ab's error takes precedence
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_applicative_apply_with_f_ab_none_value_yields_none() {
    // Totality invariant: a value-less function operand yields absence (`none()`), not an error.
    let f_ab: PropagatingEffect<fn(i32) -> i32> = PropagatingEffect::new(
        Ok(CausalEffect::none()), // Explicitly type None for the function
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_value(10);
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_ok());
    assert!(m_b.value().is_none());
    assert!(m_b.effect().unwrap().is_none());
}

#[test]
fn test_applicative_apply_with_f_a_none_value_yields_none() {
    // Totality invariant: a value-less argument operand yields absence (`none()`), not an error.
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect::new(
        Ok(CausalEffect::value(f_ab_func)),
        (),
        None,
        EffectLog::new(),
    );
    let f_a = setup_effect_with_none_value::<i32>();
    let m_b = TestWitness::apply(f_ab, f_a);
    assert!(m_b.is_ok());
    assert!(m_b.value().is_none());
    assert!(m_b.effect().unwrap().is_none());
}

#[test]
fn test_applicative_apply_logs_combined() {
    let mut logs1 = EffectLog::new();
    logs1.add_entry("Log 1");
    let f_ab_func = |a: i32| a + 1;
    let f_ab = PropagatingEffect::new(Ok(CausalEffect::value(f_ab_func)), (), None, logs1);

    let mut logs2 = EffectLog::new();
    logs2.add_entry("Log 2");
    let f_a = PropagatingEffect::new(Ok(CausalEffect::value(10)), (), None, logs2);

    let m_b = TestWitness::apply(f_ab, f_a);
    assert_eq!(m_b.logs().len(), 2);
}

// Monad Tests
#[test]
fn test_monad_bind_with_value() {
    let m_a = setup_effect_with_value(5);
    let f = |a: i32| setup_effect_with_value(a * 2);
    let m_b = TestWitness::bind(m_a, f);
    assert_eq!(m_b.value(), Some(&10));
    assert!(m_b.is_ok());
}

#[test]
fn test_monad_bind_with_error_in_m_a() {
    let m_a = setup_effect_with_error::<i32>(CausalityErrorEnum::TypeConversionError);
    let f = |a: i32| setup_effect_with_value(a * 2); // This function should not be called
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::TypeConversionError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_monad_bind_with_error_in_f_result() {
    let m_a = setup_effect_with_value(5);
    let f = |_a: i32| setup_effect_with_error::<i32>(CausalityErrorEnum::InternalLogicError); // Explicitly type T
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_err());
    assert_eq!(
        m_b.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(m_b.value().is_none());
}

#[test]
fn test_monad_bind_with_none_value_in_m_a_passes_through() {
    // Lawful `Maybe`/free-monad short-circuit: `None >>= f = None` (right identity). The continuation
    // is not invoked and no error is fabricated.
    let m_a = setup_effect_with_none_value::<i32>();
    let f = |a: i32| setup_effect_with_value(a * 2);
    let m_b = TestWitness::bind(m_a, f);
    assert!(m_b.is_ok());
    assert!(m_b.value().is_none());
    assert!(m_b.effect().unwrap().is_none());
}

#[test]
fn test_monad_bind_logs_combined() {
    let mut logs1 = EffectLog::new();
    logs1.add_entry("Log 1 from m_a");
    let m_a = PropagatingEffect::new(Ok(CausalEffect::value(5)), (), None, logs1);

    let f = |a: i32| {
        let mut logs2 = EffectLog::new();
        logs2.add_entry("Log 2 from f");
        PropagatingEffect::new(Ok(CausalEffect::value(a * 2)), (), None, logs2)
    };

    let m_b = TestWitness::bind(m_a, f);
    assert_eq!(m_b.logs().len(), 2);
}
