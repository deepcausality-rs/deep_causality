/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Effect3, Functor, Monad, MonadEffect3};
use deep_causality_haft::{HKT, HKT3, Placeholder};

// --- HKT Witnesses (re-declared for this test file's scope) ---

// Witness for Option
struct OptionWitness;

impl HKT for OptionWitness {
    type Type<T> = Option<T>;
}

// Witness for Result<T, E> where E is fixed
struct ResultWitness<E>(Placeholder, E);

impl<E> HKT for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

// --- Functor Tests ---

impl Functor<OptionWitness> for OptionWitness {
    fn fmap<A, B, Func>(m_a: Option<A>, f: Func) -> Option<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

#[test]
fn test_functor_option() {
    let opt_a = Some(5);
    let f = |x| x * 2;
    let opt_b = OptionWitness::fmap(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_mapped = OptionWitness::fmap(opt_none, f);
    assert_eq!(opt_none_mapped, None);
}

impl<E> Functor<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    fn fmap<A, B, Func>(m_a: Result<A, E>, f: Func) -> Result<B, E>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

#[test]
fn test_functor_result() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| x * 2;
    let res_b = ResultWitness::fmap(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_mapped = ResultWitness::fmap(res_err, f);
    assert_eq!(res_err_mapped, Err("Error".to_string()));
}

// --- Monad Tests ---

impl Monad<OptionWitness> for OptionWitness {
    fn pure<T>(value: T) -> Option<T> {
        Some(value)
    }

    fn bind<A, B, Func>(m_a: Option<A>, f: Func) -> Option<B>
    where
        Func: FnOnce(A) -> Option<B>,
    {
        m_a.and_then(f)
    }
}

#[test]
fn test_monad_option() {
    let opt_a = Some(5);
    let f = |x| Some(x * 2);
    let opt_b = OptionWitness::bind(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_bound = OptionWitness::bind(opt_none, f);
    assert_eq!(opt_none_bound, None);

    let opt_a_to_none = Some(5);
    let f_to_none = |_| -> Option<i32> { None };
    let opt_b_none = OptionWitness::bind(opt_a_to_none, f_to_none);
    assert_eq!(opt_b_none, None);

    let pure_val = OptionWitness::pure(100);
    assert_eq!(pure_val, Some(100));
}

impl<E> Monad<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    fn pure<T>(value: T) -> Result<T, E> {
        Ok(value)
    }

    fn bind<A, B, Func>(m_a: Result<A, E>, f: Func) -> Result<B, E>
    where
        Func: FnOnce(A) -> Result<B, E>,
    {
        m_a.and_then(f)
    }
}

#[test]
fn test_monad_result() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| Ok(x * 2);
    let res_b = ResultWitness::bind(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_bound = ResultWitness::bind(res_err, f);
    assert_eq!(res_err_bound, Err("Error".to_string()));

    let res_a_to_err: Result<i32, String> = Ok(5);
    let f_to_err = |_| -> Result<i32, String> { Err("Inner Error".to_string()) };
    let res_b_err = ResultWitness::bind(res_a_to_err, f_to_err);
    assert_eq!(res_b_err, Err("Inner Error".to_string()));

    let pure_val: Result<i32, String> = ResultWitness::pure(100);
    assert_eq!(pure_val, Ok(100));
}

// --- MonadEffect3 Tests ---

// A dummy type with three generic parameters to act as a witness
struct MyCustomEffectType<T, E, W> {
    value: T,
    error: Option<E>,
    warnings: Vec<W>,
}

// HKT3 Witness for MyCustomEffectType
struct MyEffectHktWitness<E, W>(Placeholder, E, W);

impl<E, W> HKT for MyEffectHktWitness<E, W> {
    type Type<T> = MyCustomEffectType<T, E, W>;
}

impl<E, W> HKT3<E, W> for MyEffectHktWitness<E, W> {
    type Type<T> = MyCustomEffectType<T, E, W>;
}

// Effect3 Witness
struct MyEffect;

impl Effect3 for MyEffect {
    type Fixed1 = String;
    type Fixed2 = String;
    type HktWitness = MyEffectHktWitness<Self::Fixed1, Self::Fixed2>;
}

// Functor for MyCustomEffectType
impl Functor<MyEffectHktWitness<String, String>> for MyEffectHktWitness<String, String> {
    fn fmap<A, B, Func>(
        m_a: MyCustomEffectType<A, String, String>,
        f: Func,
    ) -> MyCustomEffectType<B, String, String>
    where
        Func: FnOnce(A) -> B,
    {
        MyCustomEffectType {
            value: f(m_a.value),
            error: m_a.error,
            warnings: m_a.warnings,
        }
    }
}

// Monad for MyCustomEffectType
impl Monad<MyEffectHktWitness<String, String>> for MyEffectHktWitness<String, String> {
    fn pure<T>(value: T) -> MyCustomEffectType<T, String, String> {
        MyCustomEffectType {
            value,
            error: None,
            warnings: Vec::new(),
        }
    }

    fn bind<A, B, Func>(
        m_a: MyCustomEffectType<A, String, String>,
        f: Func,
    ) -> MyCustomEffectType<B, String, String>
    where
        Func: FnOnce(A) -> MyCustomEffectType<B, String, String>,
    {
        let mut next_effect = f(m_a.value);
        // Propagate errors and warnings
        if m_a.error.is_some() {
            next_effect.error = m_a.error;
        }
        let mut combined_warnings = m_a.warnings;
        combined_warnings.extend(next_effect.warnings);
        next_effect.warnings = combined_warnings;
        next_effect
    }
}

// MonadEffect3 for MyEffect
struct MyMonadEffect3;

impl MonadEffect3<MyEffect> for MyMonadEffect3 {
    fn pure<T>(
        value: T,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T> {
        <MyEffect as Effect3>::HktWitness::pure(value)
    }

    fn bind<T, U, Func>(
        effect: <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<T>,
        f: Func,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<U>
    where
        Func: FnOnce(
            T,
        ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<U>,
    {
        <MyEffect as Effect3>::HktWitness::bind(effect, f)
    }

    fn log<T>(
        mut effect: <<MyEffect as Effect3>::HktWitness as HKT3<
            <MyEffect as Effect3>::Fixed1,
            <MyEffect as Effect3>::Fixed2,
        >>::Type<T>,
        log: <MyEffect as Effect3>::Fixed2,
    ) -> <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T> {
        effect.warnings.push(log);
        effect
    }
}

#[test]
fn test_monad_effect3() {
    type MyEffectType<T> = <<MyEffect as Effect3>::HktWitness as HKT3<
        <MyEffect as Effect3>::Fixed1,
        <MyEffect as Effect3>::Fixed2,
    >>::Type<T>;

    let initial_effect: MyEffectType<i32> = MyMonadEffect3::pure(10);
    assert_eq!(initial_effect.value, 10);
    assert!(initial_effect.error.is_none());
    assert!(initial_effect.warnings.is_empty());

    let f1 = |x| MyEffectType {
        value: x * 2,
        error: None,
        warnings: vec!["Warning 1".to_string()],
    };

    let effect_after_bind = MyMonadEffect3::bind(initial_effect, f1);
    assert_eq!(effect_after_bind.value, 20);
    assert!(effect_after_bind.error.is_none());
    assert_eq!(effect_after_bind.warnings, vec!["Warning 1".to_string()]);

    let effect_with_log = MyMonadEffect3::log(effect_after_bind, "Warning 2".to_string());
    assert_eq!(effect_with_log.value, 20);
    assert!(effect_with_log.error.is_none());
    assert_eq!(
        effect_with_log.warnings,
        vec!["Warning 1".to_string(), "Warning 2".to_string()]
    );

    let f2 = |x| MyEffectType {
        value: x + 1,
        error: Some("Error occurred".to_string()),
        warnings: vec!["Warning 3".to_string()],
    };

    let final_effect = MyMonadEffect3::bind(effect_with_log, f2);
    assert_eq!(final_effect.value, 21);
    assert_eq!(final_effect.error, Some("Error occurred".to_string()));
    assert_eq!(
        final_effect.warnings,
        vec![
            "Warning 1".to_string(),
            "Warning 2".to_string(),
            "Warning 3".to_string()
        ]
    );
}
