/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::utils_tests::*;
use deep_causality_haft::{
    Applicative, Effect3, Effect4, Effect5, Functor, HKT, HKT3, HKT4, HKT5, Monad, MonadEffect3,
    MonadEffect4, MonadEffect5,
};

// --- MyCustomEffectType (Arity 3) Tests ---

#[test]
fn test_my_custom_effect_type_creation() {
    let effect: MyCustomEffectType<i32, Option<String>, Vec<String>> = MyCustomEffectType {
        value: 10,
        error: None,
        warnings: vec![],
    };
    assert_eq!(effect.value, 10);
    assert!(effect.error.is_none());
    assert!(effect.warnings.is_empty());
}

#[test]
fn test_my_custom_effect_type_with_error_and_warnings() {
    let effect = MyCustomEffectType {
        value: 20,
        error: Some("Test Error".to_string()),
        warnings: vec!["Warning 1".to_string()],
    };
    assert_eq!(effect.value, 20);
    assert_eq!(effect.error, Some("Test Error".to_string()));
    assert_eq!(effect.warnings, vec!["Warning 1".to_string()]);
}

// --- MyEffectHktWitness (Arity 3) HKT Tests ---

#[test]
fn test_my_effect_hkt_witness_hkt_type() {
    type WitnessedType<T> = <MyEffectHktWitness<String, String> as HKT>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType {
        value: 10,
        error: None,
        warnings: vec![],
    };
    assert_eq!(effect.value, 10);
}

#[test]
fn test_my_effect_hkt_witness_hkt3_type() {
    type WitnessedType<T> = <MyEffectHktWitness<String, String> as HKT3<String, String>>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType {
        value: 10,
        error: None,
        warnings: vec![],
    };
    assert_eq!(effect.value, 10);
}

// --- MyEffectHktWitness (Arity 3) Functor Implementation ---

#[test]
fn test_my_effect_hkt_witness_fmap_no_error() {
    let initial = MyCustomEffectType {
        value: 5,
        error: None,
        warnings: vec!["W1".to_string()],
    };
    let mapped = MyEffectHktWitness::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10);
    assert!(mapped.error.is_none());
    assert_eq!(mapped.warnings, vec!["W1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness_fmap_with_error() {
    let initial = MyCustomEffectType {
        value: 5,
        error: Some("E1".to_string()),
        warnings: vec!["W1".to_string()],
    };
    let mapped = MyEffectHktWitness::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10); // Value is still mapped
    assert_eq!(mapped.error, Some("E1".to_string()));
    assert_eq!(mapped.warnings, vec!["W1".to_string()]);
}

// --- MyEffectHktWitness (Arity 3) Applicative Implementation ---

#[test]
fn test_my_effect_hkt_witness_applicative_pure() {
    let pure_effect = MyEffectHktWitness::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.error.is_none());
    assert!(pure_effect.warnings.is_empty());
}

#[test]
fn test_my_effect_hkt_witness_applicative_apply_both_ok() {
    let f_effect = MyCustomEffectType {
        value: |x: i32| x * 2,
        error: None,
        warnings: vec!["F_W1".to_string()],
    };
    let a_effect = MyCustomEffectType {
        value: 5,
        error: None,
        warnings: vec!["A_W1".to_string()],
    };
    let applied = MyEffectHktWitness::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10);
    assert!(applied.error.is_none());
    assert_eq!(
        applied.warnings,
        vec!["F_W1".to_string(), "A_W1".to_string()]
    );
}

#[test]
fn test_my_effect_hkt_witness_applicative_apply_f_error() {
    let f_effect = MyCustomEffectType {
        value: |x: i32| x * 2,
        error: Some("F_E1".to_string()),
        warnings: vec!["F_W1".to_string()],
    };
    let a_effect = MyCustomEffectType {
        value: 5,
        error: None,
        warnings: vec!["A_W1".to_string()],
    };
    let applied = MyEffectHktWitness::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value, but should be B::default() if B: Default
    assert_eq!(applied.error, Some("F_E1".to_string()));
    assert_eq!(applied.warnings, vec!["F_W1".to_string()]); // Only f_ab warnings
}

#[test]
fn test_my_effect_hkt_witness_applicative_apply_a_error() {
    let f_effect = MyCustomEffectType {
        value: |x: i32| x * 2,
        error: None,
        warnings: vec!["F_W1".to_string()],
    };
    let a_effect = MyCustomEffectType {
        value: 5,
        error: Some("A_E1".to_string()),
        warnings: vec!["A_W1".to_string()],
    };
    let applied = MyEffectHktWitness::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value, but should be B::default() if B: Default
    assert_eq!(applied.error, Some("A_E1".to_string()));
    assert_eq!(applied.warnings, vec!["A_W1".to_string()]); // Only f_a warnings
}

#[test]
fn test_my_effect_hkt_witness_applicative_apply_both_error() {
    let f_effect = MyCustomEffectType {
        value: |x: i32| x * 2,
        error: Some("F_E1".to_string()),
        warnings: vec!["F_W1".to_string()],
    };
    let a_effect = MyCustomEffectType {
        value: 5,
        error: Some("A_E1".to_string()),
        warnings: vec!["A_W1".to_string()],
    };
    let applied = MyEffectHktWitness::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value, but should be B::default() if B: Default
    assert_eq!(applied.error, Some("F_E1".to_string())); // f_ab error takes precedence
    assert_eq!(applied.warnings, vec!["F_W1".to_string()]);
}

// --- MyEffectHktWitness (Arity 3) Monad Implementation ---

#[test]
fn test_my_effect_hkt_witness_monad_bind_no_error() {
    let initial = MyCustomEffectType {
        value: 5,
        error: None,
        warnings: vec!["W1".to_string()],
    };
    let bound = MyEffectHktWitness::bind(initial, |x| MyCustomEffectType {
        value: x + 1,
        error: None,
        warnings: vec!["W2".to_string()],
    });
    assert_eq!(bound.value, 6);
    assert!(bound.error.is_none());
    assert_eq!(bound.warnings, vec!["W1".to_string(), "W2".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness_monad_bind_initial_error() {
    let initial = MyCustomEffectType {
        value: 5,
        error: Some("E1".to_string()),
        warnings: vec!["W1".to_string()],
    };
    let bound = MyEffectHktWitness::bind(initial, |x| MyCustomEffectType {
        value: x + 1,
        error: None,
        warnings: vec!["W2".to_string()],
    });
    assert_eq!(bound.value, 6); // Value is still computed
    assert_eq!(bound.error, Some("E1".to_string()));
    assert_eq!(bound.warnings, vec!["W1".to_string()]); // Only initial warnings
}

#[test]
fn test_my_effect_hkt_witness_monad_bind_function_error() {
    let initial = MyCustomEffectType {
        value: 5,
        error: None,
        warnings: vec!["W1".to_string()],
    };
    let bound = MyEffectHktWitness::bind(initial, |x| MyCustomEffectType {
        value: x + 1,
        error: Some("E2".to_string()),
        warnings: vec!["W2".to_string()],
    });
    assert_eq!(bound.value, 6);
    assert_eq!(bound.error, Some("E2".to_string()));
    assert_eq!(bound.warnings, vec!["W1".to_string(), "W2".to_string()]);
}

// --- MyMonadEffect3 Implementation ---

#[test]
fn test_my_monad_effect3_pure() {
    let pure_effect = MyMonadEffect3::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.error.is_none());
    assert!(pure_effect.warnings.is_empty());
}

#[test]
fn test_my_monad_effect3_bind() {
    type MyEffectType<T> = <<MyEffect as Effect3>::HktWitness as HKT3<String, String>>::Type<T>;
    let initial = MyMonadEffect3::pure(5);
    let bound = MyMonadEffect3::bind(initial, |x| MyEffectType {
        value: x * 2,
        error: None,
        warnings: vec!["M3_W1".to_string()],
    });
    assert_eq!(bound.value, 10);
    assert!(bound.error.is_none());
    assert_eq!(bound.warnings, vec!["M3_W1".to_string()]);
}

// --- LoggableEffect3 Implementation ---

#[test]
fn test_loggable_effect3_log() {
    let initial = MyMonadEffect3::pure(10);
    let logged = MyMonadEffect3::log(initial, "Log message".to_string());
    assert_eq!(logged.value, 10);
    assert!(logged.error.is_none());
    assert_eq!(logged.warnings, vec!["Log message".to_string()]);
}

// --- MyCustomEffectType4 (Arity 4) Tests ---

#[test]
fn test_my_custom_effect_type4_creation() {
    let effect: MyCustomEffectType4<i32, Option<String>, Vec<String>, Vec<String>> =
        MyCustomEffectType4 {
            value: 10,
            f1: None,
            f2: vec![],
            f3: vec![],
        };
    assert_eq!(effect.value, 10);
    assert!(effect.f1.is_none());
    assert!(effect.f2.is_empty());
    assert!(effect.f3.is_empty());
}

#[test]
fn test_my_custom_effect_type4_with_all_fields() {
    let effect = MyCustomEffectType4 {
        value: 20,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![100],
    };
    assert_eq!(effect.value, 20);
    assert_eq!(effect.f1, Some("E1".to_string()));
    assert_eq!(effect.f2, vec!["L1".to_string()]);
    assert_eq!(effect.f3, vec![100]);
}

// --- MyEffectHktWitness4 (Arity 4) HKT Implementations ---

#[test]
fn test_my_effect_hkt_witness4_hkt_type() {
    type WitnessedType<T> = <MyEffectHktWitness4<String, String, u64> as HKT>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType4 {
        value: 10,
        f1: None,
        f2: vec![],
        f3: vec![],
    };
    assert_eq!(effect.value, 10);
}

#[test]
fn test_my_effect_hkt_witness4_hkt4_type() {
    type WitnessedType<T> =
        <MyEffectHktWitness4<String, String, u64> as HKT4<String, String, u64>>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType4 {
        value: 10,
        f1: None,
        f2: vec![],
        f3: vec![],
    };
    assert_eq!(effect.value, 10);
}

// --- MyEffectHktWitness4 (Arity 4) Functor Implementation ---

#[test]
fn test_my_effect_hkt_witness4_fmap_no_error() {
    let initial = MyCustomEffectType4 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
    };
    let mapped = MyEffectHktWitness4::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10);
    assert!(mapped.f1.is_none());
    assert_eq!(mapped.f2, vec!["L1".to_string()]);
    assert_eq!(mapped.f3, vec![10]);
}

#[test]
fn test_my_effect_hkt_witness4_fmap_with_error() {
    let initial = MyCustomEffectType4 {
        value: 5,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![10],
    };
    let mapped = MyEffectHktWitness4::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10);
    assert_eq!(mapped.f1, Some("E1".to_string()));
    assert_eq!(mapped.f2, vec!["L1".to_string()]);
    assert_eq!(mapped.f3, vec![10]);
}

// --- MyEffectHktWitness4 (Arity 4) Applicative Implementation ---

#[test]
fn test_my_effect_hkt_witness4_applicative_pure() {
    let pure_effect = MyEffectHktWitness4::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.f1.is_none());
    assert!(pure_effect.f2.is_empty());
    assert!(pure_effect.f3.is_empty());
}

#[test]
fn test_my_effect_hkt_witness4_applicative_apply_both_ok() {
    let f_effect = MyCustomEffectType4 {
        value: |x: i32| x * 2,
        f1: None,
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
    };
    let a_effect = MyCustomEffectType4 {
        value: 5,
        f1: None,
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
    };
    let applied = MyEffectHktWitness4::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10);
    assert!(applied.f1.is_none());
    assert_eq!(applied.f2, vec!["F_L1".to_string(), "A_L1".to_string()]);
    assert_eq!(applied.f3, vec![1, 2]);
}

#[test]
fn test_my_effect_hkt_witness4_applicative_apply_f_error() {
    let f_effect = MyCustomEffectType4 {
        value: |x: i32| x * 2,
        f1: Some("F_E1".to_string()),
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
    };
    let a_effect = MyCustomEffectType4 {
        value: 5,
        f1: None,
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
    };
    let applied = MyEffectHktWitness4::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("F_E1".to_string()));
    assert_eq!(applied.f2, vec!["F_L1".to_string()]);
    assert_eq!(applied.f3, vec![1]);
}

#[test]
fn test_my_effect_hkt_witness4_applicative_apply_a_error() {
    let f_effect = MyCustomEffectType4 {
        value: |x: i32| x * 2,
        f1: None,
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
    };
    let a_effect = MyCustomEffectType4 {
        value: 5,
        f1: Some("A_E1".to_string()),
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
    };
    let applied = MyEffectHktWitness4::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("A_E1".to_string()));
    assert_eq!(applied.f2, vec!["A_L1".to_string()]);
    assert_eq!(applied.f3, vec![2]);
}

#[test]
fn test_my_effect_hkt_witness4_applicative_apply_both_error() {
    let f_effect = MyCustomEffectType4 {
        value: |x: i32| x * 2,
        f1: Some("F_E1".to_string()),
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
    };
    let a_effect = MyCustomEffectType4 {
        value: 5,
        f1: Some("A_E1".to_string()),
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
    };
    let applied = MyEffectHktWitness4::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("F_E1".to_string()));
    assert_eq!(applied.f2, vec!["F_L1".to_string()]);
    assert_eq!(applied.f3, vec![1]);
}

// --- MyEffectHktWitness4 (Arity 4) Monad Implementation ---

#[test]
fn test_my_effect_hkt_witness4_monad_bind_no_error() {
    let initial = MyCustomEffectType4 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
    };
    let bound = MyEffectHktWitness4::bind(initial, |x| MyCustomEffectType4 {
        value: x + 1,
        f1: None,
        f2: vec!["L2".to_string()],
        f3: vec![20],
    });
    assert_eq!(bound.value, 6);
    assert!(bound.f1.is_none());
    assert_eq!(bound.f2, vec!["L1".to_string(), "L2".to_string()]);
    assert_eq!(bound.f3, vec![10, 20]);
}

#[test]
fn test_my_effect_hkt_witness4_monad_bind_initial_error() {
    let initial = MyCustomEffectType4 {
        value: 5,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![10],
    };
    let bound = MyEffectHktWitness4::bind(initial, |x| MyCustomEffectType4 {
        value: x + 1,
        f1: None,
        f2: vec!["L2".to_string()],
        f3: vec![20],
    });
    assert_eq!(bound.value, 6); // Value is still computed
    assert_eq!(bound.f1, Some("E1".to_string()));
    assert_eq!(bound.f2, vec!["L1".to_string()]);
    assert_eq!(bound.f3, vec![10]);
}

#[test]
fn test_my_effect_hkt_witness4_monad_bind_function_error() {
    let initial = MyCustomEffectType4 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
    };
    let bound = MyEffectHktWitness4::bind(initial, |x| MyCustomEffectType4 {
        value: x + 1,
        f1: Some("E2".to_string()),
        f2: vec!["L2".to_string()],
        f3: vec![20],
    });
    assert_eq!(bound.value, 6);
    assert_eq!(bound.f1, Some("E2".to_string()));
    assert_eq!(bound.f2, vec!["L1".to_string(), "L2".to_string()]);
    assert_eq!(bound.f3, vec![10, 20]);
}

// --- MyMonadEffect4 Implementation ---

#[test]
fn test_my_monad_effect4_pure() {
    let pure_effect = MyMonadEffect4::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.f1.is_none());
    assert!(pure_effect.f2.is_empty());
    assert!(pure_effect.f3.is_empty());
}

#[test]
fn test_my_monad_effect4_bind() {
    type MyEffectType<T> =
        <<MyEffect4 as Effect4>::HktWitness as HKT4<String, String, u64>>::Type<T>;
    let initial = MyMonadEffect4::pure(5);
    let bound = MyMonadEffect4::bind(initial, |x| MyEffectType {
        value: x * 2,
        f1: None,
        f2: vec!["M4_L1".to_string()],
        f3: vec![100],
    });
    assert_eq!(bound.value, 10);
    assert!(bound.f1.is_none());
    assert_eq!(bound.f2, vec!["M4_L1".to_string()]);
    assert_eq!(bound.f3, vec![100]);
}

// --- LoggableEffect4 Implementation ---

#[test]
fn test_loggable_effect4_log() {
    let initial = MyMonadEffect4::pure(10);
    let logged = MyMonadEffect4::log(initial, 500);
    assert_eq!(logged.value, 10);
    assert!(logged.f1.is_none());
    assert!(logged.f2.is_empty());
    assert_eq!(logged.f3, vec![500]);
}

// --- MyCustomEffectType5 (Arity 5) Tests ---

#[test]
fn test_my_custom_effect_type5_creation() {
    let effect: MyCustomEffectType5<i32, Option<String>, Vec<String>, Vec<String>, Vec<String>> =
        MyCustomEffectType5 {
            value: 10,
            f1: None,
            f2: vec![],
            f3: vec![],
            f4: vec![],
        };
    assert_eq!(effect.value, 10);
    assert!(effect.f1.is_none());
    assert!(effect.f2.is_empty());
    assert!(effect.f3.is_empty());
    assert!(effect.f4.is_empty());
}

#[test]
fn test_my_custom_effect_type5_with_all_fields() {
    let effect = MyCustomEffectType5 {
        value: 20,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![100],
        f4: vec!["T1".to_string()],
    };
    assert_eq!(effect.value, 20);
    assert_eq!(effect.f1, Some("E1".to_string()));
    assert_eq!(effect.f2, vec!["L1".to_string()]);
    assert_eq!(effect.f3, vec![100]);
    assert_eq!(effect.f4, vec!["T1".to_string()]);
}

// --- MyEffectHktWitness5 (Arity 5) HKT Implementations ---

#[test]
fn test_my_effect_hkt_witness5_hkt_type() {
    type WitnessedType<T> = <MyEffectHktWitness5<String, String, u64, String> as HKT>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType5 {
        value: 10,
        f1: None,
        f2: vec![],
        f3: vec![],
        f4: vec![],
    };
    assert_eq!(effect.value, 10);
}

#[test]
fn test_my_effect_hkt_witness5_hkt5_type() {
    type WitnessedType<T> = <MyEffectHktWitness5<String, String, u64, String> as HKT5<
        String,
        String,
        u64,
        String,
    >>::Type<T>;
    let effect: WitnessedType<i32> = MyCustomEffectType5 {
        value: 10,
        f1: None,
        f2: vec![],
        f3: vec![],
        f4: vec![],
    };
    assert_eq!(effect.value, 10);
}

// --- MyEffectHktWitness5 (Arity 5) Functor Implementation ---

#[test]
fn test_my_effect_hkt_witness5_fmap_no_error() {
    let initial = MyCustomEffectType5 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
        f4: vec!["T1".to_string()],
    };
    let mapped = MyEffectHktWitness5::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10);
    assert!(mapped.f1.is_none());
    assert_eq!(mapped.f2, vec!["L1".to_string()]);
    assert_eq!(mapped.f3, vec![10]);
    assert_eq!(mapped.f4, vec!["T1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_fmap_with_error() {
    let initial = MyCustomEffectType5 {
        value: 5,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![10],
        f4: vec!["T1".to_string()],
    };
    let mapped = MyEffectHktWitness5::fmap(initial, |x| x * 2);
    assert_eq!(mapped.value, 10);
    assert_eq!(mapped.f1, Some("E1".to_string()));
    assert_eq!(mapped.f2, vec!["L1".to_string()]);
    assert_eq!(mapped.f3, vec![10]);
    assert_eq!(mapped.f4, vec!["T1".to_string()]);
}

// --- MyEffectHktWitness5 (Arity 5) Applicative Implementation ---

#[test]
fn test_my_effect_hkt_witness5_applicative_pure() {
    let pure_effect = MyEffectHktWitness5::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.f1.is_none());
    assert!(pure_effect.f2.is_empty());
    assert!(pure_effect.f3.is_empty());
    assert!(pure_effect.f4.is_empty());
}

#[test]
fn test_my_effect_hkt_witness5_applicative_apply_both_ok() {
    let f_effect = MyCustomEffectType5 {
        value: |x: i32| x * 2,
        f1: None,
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
        f4: vec!["F_T1".to_string()],
    };
    let a_effect = MyCustomEffectType5 {
        value: 5,
        f1: None,
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
        f4: vec!["A_T1".to_string()],
    };
    let applied = MyEffectHktWitness5::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10);
    assert!(applied.f1.is_none());
    assert_eq!(applied.f2, vec!["F_L1".to_string(), "A_L1".to_string()]);
    assert_eq!(applied.f3, vec![1, 2]);
    assert_eq!(applied.f4, vec!["F_T1".to_string(), "A_T1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_applicative_apply_f_error() {
    let f_effect = MyCustomEffectType5 {
        value: |x: i32| x * 2,
        f1: Some("F_E1".to_string()),
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
        f4: vec!["F_T1".to_string()],
    };
    let a_effect = MyCustomEffectType5 {
        value: 5,
        f1: None,
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
        f4: vec!["A_T1".to_string()],
    };
    let applied = MyEffectHktWitness5::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("F_E1".to_string()));
    assert_eq!(applied.f2, vec!["F_L1".to_string()]);
    assert_eq!(applied.f3, vec![1]);
    assert_eq!(applied.f4, vec!["F_T1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_applicative_apply_a_error() {
    let f_effect = MyCustomEffectType5 {
        value: |x: i32| x * 2,
        f1: None,
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
        f4: vec!["F_T1".to_string()],
    };
    let a_effect = MyCustomEffectType5 {
        value: 5,
        f1: Some("A_E1".to_string()),
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
        f4: vec!["A_T1".to_string()],
    };
    let applied = MyEffectHktWitness5::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("A_E1".to_string()));
    assert_eq!(applied.f2, vec!["A_L1".to_string()]);
    assert_eq!(applied.f3, vec![2]);
    assert_eq!(applied.f4, vec!["A_T1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_applicative_apply_both_error() {
    let f_effect = MyCustomEffectType5 {
        value: |x: i32| x * 2,
        f1: Some("F_E1".to_string()),
        f2: vec!["F_L1".to_string()],
        f3: vec![1],
        f4: vec!["F_T1".to_string()],
    };
    let a_effect = MyCustomEffectType5 {
        value: 5,
        f1: Some("A_E1".to_string()),
        f2: vec!["A_L1".to_string()],
        f3: vec![2],
        f4: vec!["A_T1".to_string()],
    };
    let applied = MyEffectHktWitness5::apply(f_effect, a_effect);
    assert_eq!(applied.value, 10); // Dummy value
    assert_eq!(applied.f1, Some("F_E1".to_string())); // f_ab error takes precedence
    assert_eq!(applied.f2, vec!["F_L1".to_string()]);
    assert_eq!(applied.f3, vec![1]);
    assert_eq!(applied.f4, vec!["F_T1".to_string()]);
}

// --- MyEffectHktWitness5 (Arity 5) Monad Implementation ---

#[test]
fn test_my_effect_hkt_witness5_monad_bind_no_error() {
    let initial = MyCustomEffectType5 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
        f4: vec!["T1".to_string()],
    };
    let bound = MyEffectHktWitness5::bind(initial, |x| MyCustomEffectType5 {
        value: x + 1,
        f1: None,
        f2: vec!["L2".to_string()],
        f3: vec![20],
        f4: vec!["T2".to_string()],
    });
    assert_eq!(bound.value, 6);
    assert!(bound.f1.is_none());
    assert_eq!(bound.f2, vec!["L1".to_string(), "L2".to_string()]);
    assert_eq!(bound.f3, vec![10, 20]);
    assert_eq!(bound.f4, vec!["T1".to_string(), "T2".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_monad_bind_initial_error() {
    let initial = MyCustomEffectType5 {
        value: 5,
        f1: Some("E1".to_string()),
        f2: vec!["L1".to_string()],
        f3: vec![10],
        f4: vec!["T1".to_string()],
    };
    let bound = MyEffectHktWitness5::bind(initial, |x| MyCustomEffectType5 {
        value: x + 1,
        f1: None,
        f2: vec!["L2".to_string()],
        f3: vec![20],
        f4: vec!["T2".to_string()],
    });
    assert_eq!(bound.value, 6); // Value is still computed
    assert_eq!(bound.f1, Some("E1".to_string()));
    assert_eq!(bound.f2, vec!["L1".to_string()]);
    assert_eq!(bound.f3, vec![10]);
    assert_eq!(bound.f4, vec!["T1".to_string()]);
}

#[test]
fn test_my_effect_hkt_witness5_monad_bind_function_error() {
    let initial = MyCustomEffectType5 {
        value: 5,
        f1: None,
        f2: vec!["L1".to_string()],
        f3: vec![10],
        f4: vec!["T1".to_string()],
    };
    let bound = MyEffectHktWitness5::bind(initial, |x| MyCustomEffectType5 {
        value: x + 1,
        f1: Some("E2".to_string()),
        f2: vec!["L2".to_string()],
        f3: vec![20],
        f4: vec!["T2".to_string()],
    });
    assert_eq!(bound.value, 6);
    assert_eq!(bound.f1, Some("E2".to_string()));
    assert_eq!(bound.f2, vec!["L1".to_string(), "L2".to_string()]);
    assert_eq!(bound.f3, vec![10, 20]);
    assert_eq!(bound.f4, vec!["T1".to_string(), "T2".to_string()]);
}

// --- MyMonadEffect5 Implementation ---

#[test]
fn test_my_monad_effect5_pure() {
    let pure_effect = MyMonadEffect5::pure(100);
    assert_eq!(pure_effect.value, 100);
    assert!(pure_effect.f1.is_none());
    assert!(pure_effect.f2.is_empty());
    assert!(pure_effect.f3.is_empty());
    assert!(pure_effect.f4.is_empty());
}

#[test]
fn test_my_monad_effect5_bind() {
    type MyEffectType<T> =
        <<MyEffect5 as Effect5>::HktWitness as HKT5<String, String, u64, String>>::Type<T>;

    let initial = MyMonadEffect5::pure(5);
    let bound = MyMonadEffect5::bind(initial, |x| MyEffectType {
        value: x * 2,
        f1: None,
        f2: vec!["M5_L1".to_string()],
        f3: vec![100],
        f4: vec!["M5_T1".to_string()],
    });
    assert_eq!(bound.value, 10);
    assert!(bound.f1.is_none());
    assert_eq!(bound.f2, vec!["M5_L1".to_string()]);
    assert_eq!(bound.f3, vec![100]);
    assert_eq!(bound.f4, vec!["M5_T1".to_string()]);
}

// --- LoggableEffect5 Implementation ---

#[test]
fn test_loggable_effect5_log() {
    let initial = MyMonadEffect5::pure(10);
    let logged = MyMonadEffect5::log(initial, "Log message".to_string());
    assert_eq!(logged.value, 10);
    assert!(logged.f1.is_none());
    assert!(logged.f2.is_empty());
    assert!(logged.f3.is_empty());
    assert_eq!(logged.f4, vec!["Log message".to_string()]);
}
