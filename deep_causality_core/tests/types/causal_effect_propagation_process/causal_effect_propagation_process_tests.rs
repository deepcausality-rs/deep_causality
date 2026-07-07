/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalityError, CausalityErrorEnum};
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalEffect;
use deep_causality_core::CausalEffectPropagationProcess;

#[derive(Debug, Clone, PartialEq, Default)]
struct TestLog(Vec<String>);

impl deep_causality_haft::LogAppend for TestLog {
    fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
}

fn unwrap_value<T: Copy>(val: CausalEffect<T>) -> T {
    val.into_value().expect("expected a value effect")
}

#[test]
fn test_bind_propagation() {
    let initial_process: CausalEffectPropagationProcess<i32, i32, (), String, TestLog> =
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(10)),
            0,
            None,
            TestLog::default(),
        );

    let next_process = initial_process.bind(|val, state, _ctx| {
        let new_val = unwrap_value(val) + 1;
        let new_state = state + 1;
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(new_val)),
            new_state,
            None,
            TestLog(vec!["step1".to_string()]),
        )
    });

    let (outcome, state, _context, logs) = next_process.into_parts();
    assert_eq!(unwrap_value(outcome.unwrap()), 11);
    assert_eq!(state, 1);
    assert_eq!(logs.0, vec!["step1".to_string()]);
}

#[test]
fn test_bind_error_propagation() {
    // Value and error are one channel: an errored process is constructed as `Err`
    // and can no longer also carry a value.
    let error_process: CausalEffectPropagationProcess<i32, i32, (), CausalityError, TestLog> =
        CausalEffectPropagationProcess::new(
            Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
            0,
            None,
            TestLog::default(),
        );

    let next_process = error_process.bind(|val, state, _ctx| {
        // This closure should not be executed
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(unwrap_value(val) + 1)),
            state + 1,
            None,
            TestLog::default(),
        )
    });

    assert!(next_process.error().is_some());
    assert_eq!(
        next_process.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(next_process.value().is_none()); // An errored carrier holds no value
}

#[test]
fn test_with_state() {
    let stateless_effect: CausalEffectPropagationProcess<i32, (), (), String, TestLog> =
        CausalEffectPropagationProcess::new(
            Ok(CausalEffect::value(42)),
            (),
            None,
            TestLog(vec!["init".to_string()]),
        );

    let stateful_process = CausalEffectPropagationProcess::with_state(
        stateless_effect,
        100,
        Some("Context".to_string()),
    );

    let (outcome, state, context, logs) = stateful_process.into_parts();
    assert_eq!(unwrap_value(outcome.unwrap()), 42);
    assert_eq!(state, 100);
    assert_eq!(context, Some("Context".to_string()));
    assert_eq!(logs.0, vec!["init".to_string()]);
}

#[test]
fn test_from_error() {
    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::from_error(error);

    assert!(process.error().is_some());
    assert_eq!(
        process.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(process.value().is_none()); // An errored carrier holds no value
    assert_eq!(*process.state(), 0);
    assert!(process.context().is_none());
}

#[test]
fn test_none() {
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::none();

    assert!(process.effect().is_some_and(CausalEffect::is_none));
    assert_eq!(*process.state(), 0);
    assert!(process.context().is_none());
    assert!(process.error().is_none());
}

#[test]
fn test_pure() {
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::pure(99);

    assert!(matches!(process.value(), Some(&99)));
    assert_eq!(*process.state(), 0);
    assert!(process.context().is_none());
    assert!(process.error().is_none());
}

#[test]
fn test_from_effect() {
    let val = CausalEffect::value(123);
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::from_effect(val);

    assert!(matches!(process.value(), Some(&123)));
    assert_eq!(*process.state(), 0);
    assert!(process.context().is_none());
    assert!(process.error().is_none());
}

#[test]
fn test_from_effect_with_log() {
    let val = CausalEffect::value(456);
    let logs = deep_causality_core::EffectLog::new();
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::from_effect_with_log(val, logs);

    assert!(matches!(process.value(), Some(&456)));
    assert_eq!(*process.state(), 0);
    assert!(process.context().is_none());
    assert!(process.error().is_none());
}
