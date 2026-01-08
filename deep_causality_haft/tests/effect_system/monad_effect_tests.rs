/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::utils_tests::*;
use deep_causality_haft::{
    Effect3, Effect4, Effect5, HKT3, HKT4, HKT5, MonadEffect3, MonadEffect4, MonadEffect5,
};

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

#[test]
fn test_monad_effect4() {
    type MyEffectType<T> = <<MyEffect4 as Effect4>::HktWitness as HKT4<
        <MyEffect4 as Effect4>::Fixed1,
        <MyEffect4 as Effect4>::Fixed2,
        <MyEffect4 as Effect4>::Fixed3,
    >>::Type<T>;

    let initial_effect: MyEffectType<i32> = MyMonadEffect4::pure(10);
    assert_eq!(
        initial_effect,
        MyCustomEffectType4 {
            value: 10,
            f1: None,
            f2: vec![],
            f3: vec![]
        }
    );

    let f1 = |x| MyEffectType {
        value: x * 2,
        f1: None,
        f2: vec!["Log 1".to_string()],
        f3: vec![100],
    };

    let effect1 = MyMonadEffect4::bind(initial_effect, f1);
    assert_eq!(
        effect1,
        MyCustomEffectType4 {
            value: 20,
            f1: None,
            f2: vec!["Log 1".to_string()],
            f3: vec![100]
        }
    );

    let effect2 = MyMonadEffect4::log(effect1, 200);
    assert_eq!(
        effect2,
        MyCustomEffectType4 {
            value: 20,
            f1: None,
            f2: vec!["Log 1".to_string()],
            f3: vec![100, 200]
        }
    );

    let f2 = |x| MyEffectType {
        value: x + 5,
        f1: Some("Error Occurred".to_string()),
        f2: vec!["Log 2".to_string()],
        f3: vec![300],
    };

    let final_effect = MyMonadEffect4::bind(effect2, f2);
    assert_eq!(
        final_effect,
        MyCustomEffectType4 {
            value: 25,
            f1: Some("Error Occurred".to_string()),
            f2: vec!["Log 1".to_string(), "Log 2".to_string()],
            f3: vec![100, 200, 300],
        }
    );
}

#[test]
fn test_monad_effect5() {
    type MyEffectType<T> = <<MyEffect5 as Effect5>::HktWitness as HKT5<
        <MyEffect5 as Effect5>::Fixed1,
        <MyEffect5 as Effect5>::Fixed2,
        <MyEffect5 as Effect5>::Fixed3,
        <MyEffect5 as Effect5>::Fixed4,
    >>::Type<T>;

    let initial_effect: MyEffectType<i32> = MyMonadEffect5::pure(10);
    assert_eq!(
        initial_effect,
        MyCustomEffectType5 {
            value: 10,
            f1: None,
            f2: vec![],
            f3: vec![],
            f4: vec![]
        }
    );

    let f1 = |x| MyEffectType {
        value: x * 2,
        f1: None,
        f2: vec!["Log 1".to_string()],
        f3: vec![100],
        f4: vec!["Trace 1".to_string()],
    };

    let effect1 = MyMonadEffect5::bind(initial_effect, f1);
    assert_eq!(
        effect1,
        MyCustomEffectType5 {
            value: 20,
            f1: None,
            f2: vec!["Log 1".to_string()],
            f3: vec![100],
            f4: vec!["Trace 1".to_string()]
        }
    );

    let effect2 = MyMonadEffect5::log(effect1, "Trace 2".to_string());
    assert_eq!(
        effect2,
        MyCustomEffectType5 {
            value: 20,
            f1: None,
            f2: vec!["Log 1".to_string()],
            f3: vec![100],
            f4: vec!["Trace 1".to_string(), "Trace 2".to_string()]
        }
    );
}
