use deep_causality_core::{CausalityError, CausalityErrorEnum};
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalEffectPropagationProcess;
use deep_causality_core::EffectValue;

#[derive(Debug, Clone, PartialEq, Default)]
struct TestLog(Vec<String>);

impl deep_causality_haft::LogAppend for TestLog {
    fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
}

fn unwrap_value<T: Copy>(val: EffectValue<T>) -> T {
    if let EffectValue::Value(v) = val {
        v
    } else {
        panic!("Expected EffectValue::Value")
    }
}

#[test]
fn test_bind_propagation() {
    let initial_process = CausalEffectPropagationProcess {
        value: EffectValue::Value(10),
        state: 0,
        context: None::<()>,
        error: None::<String>,
        logs: TestLog::default(),
    };

    let next_process = initial_process.bind(|val, state, _ctx| {
        let new_val = unwrap_value(val) + 1;
        let new_state = state + 1;
        CausalEffectPropagationProcess {
            value: EffectValue::Value(new_val),
            state: new_state,
            context: None,
            error: None,
            logs: TestLog(vec!["step1".to_string()]),
        }
    });

    assert_eq!(unwrap_value(next_process.value), 11);
    assert_eq!(next_process.state, 1);
    assert_eq!(next_process.logs.0, vec!["step1".to_string()]);
}

#[test]
fn test_bind_error_propagation() {
    let error_process = CausalEffectPropagationProcess {
        value: EffectValue::Value(10),
        state: 0,
        context: None::<()>,
        error: Some(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        logs: TestLog::default(),
    };

    let next_process = error_process.bind(|val, state, _ctx| {
        // This closure should not be executed
        CausalEffectPropagationProcess {
            value: EffectValue::from(unwrap_value(val) + 1),
            state: state + 1,
            context: None,
            error: None,
            logs: TestLog::default(),
        }
    });

    assert!(next_process.error.is_some());
    assert_eq!(
        next_process.error.unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(matches!(next_process.value, EffectValue::None)); // Default value is None
}

#[test]
fn test_with_state() {
    let stateless_effect = CausalEffectPropagationProcess {
        value: EffectValue::Value(42),
        state: (),
        context: None::<()>,
        error: None::<String>,
        logs: TestLog(vec!["init".to_string()]),
    };

    let stateful_process = CausalEffectPropagationProcess::with_state(
        stateless_effect,
        100,
        Some("Context".to_string()),
    );

    assert_eq!(unwrap_value(stateful_process.value), 42);
    assert_eq!(stateful_process.state, 100);
    assert_eq!(stateful_process.context, Some("Context".to_string()));
    assert_eq!(stateful_process.logs.0, vec!["init".to_string()]);
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

    assert!(process.error.is_some());
    assert_eq!(
        process.error.unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(matches!(process.value, EffectValue::None));
    assert_eq!(process.state, 0);
    assert!(process.context.is_none());
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

    assert!(matches!(process.value, EffectValue::None));
    assert_eq!(process.state, 0);
    assert!(process.context.is_none());
    assert!(process.error.is_none());
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

    assert_eq!(unwrap_value(process.value), 99);
    assert_eq!(process.state, 0);
    assert!(process.context.is_none());
    assert!(process.error.is_none());
}

#[test]
fn test_from_effect_value() {
    let val = EffectValue::Value(123);
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::from_effect_value(val);

    assert_eq!(unwrap_value(process.value), 123);
    assert_eq!(process.state, 0);
    assert!(process.context.is_none());
    assert!(process.error.is_none());
}

#[test]
fn test_from_effect_value_with_log() {
    let val = EffectValue::Value(456);
    let logs = deep_causality_core::EffectLog::new();
    let process = CausalEffectPropagationProcess::<
        i32,
        i32,
        (),
        CausalityError,
        deep_causality_core::EffectLog,
    >::from_effect_value_with_log(val, logs);

    assert_eq!(unwrap_value(process.value), 456);
    assert_eq!(process.state, 0);
    assert!(process.context.is_none());
    assert!(process.error.is_none());
}
