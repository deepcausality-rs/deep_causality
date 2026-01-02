/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness, CausalityError,
    EffectValue,
};
use deep_causality_haft::{Applicative, Functor, LogAppend, Monad, Pure};

#[derive(Debug, Clone, PartialEq, Default)]
struct TestLog(Vec<String>);

impl LogAppend for TestLog {
    fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
}

type TestProcess<T> = CausalEffectPropagationProcess<T, i32, String, CausalityError, TestLog>;
type TestWitness = CausalEffectPropagationProcessWitness<i32, String, CausalityError, TestLog>;

fn unwrap_value<T: std::fmt::Debug + Clone>(val: EffectValue<T>) -> T {
    if let EffectValue::Value(v) = val {
        v
    } else {
        panic!("Expected EffectValue::Value, got {:?}", val)
    }
}

fn double(x: i32) -> i32 {
    x * 2
}

#[test]
fn test_functor_fmap() {
    let process: TestProcess<i32> = CausalEffectPropagationProcess {
        value: EffectValue::Value(10),
        state: 1,
        context: Some("ctx".to_string()),
        error: None,
        logs: TestLog(vec!["log1".to_string()]),
    };

    let mapped: TestProcess<String> = TestWitness::fmap(process, |x| x.to_string());

    assert_eq!(unwrap_value(mapped.value), "10"); // Value transformed
    assert_eq!(mapped.state, 1); // State preserved
    assert_eq!(mapped.context, Some("ctx".to_string())); // Context preserved
    assert_eq!(mapped.logs.0, vec!["log1".to_string()]); // Logs preserved
}

#[test]
fn test_applicative_pure() {
    let process: TestProcess<i32> = TestWitness::pure(42);

    assert_eq!(unwrap_value(process.value), 42);
    assert_eq!(process.state, 0); // Default state
    assert_eq!(process.context, None); // Default context (None)
    assert!(process.logs.0.is_empty());
}

#[test]
fn test_applicative_apply() {
    let func_process: TestProcess<fn(i32) -> i32> = CausalEffectPropagationProcess {
        value: EffectValue::Value(double),
        state: 10,
        context: Some("ctx1".to_string()),
        error: None,
        logs: TestLog(vec!["func_log".to_string()]),
    };

    let arg_process: TestProcess<i32> = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: 20,
        context: Some("ctx2".to_string()),
        error: None,
        logs: TestLog(vec!["arg_log".to_string()]),
    };

    let result: TestProcess<i32> = TestWitness::apply(func_process, arg_process);

    assert_eq!(unwrap_value(result.value), 10);
    assert_eq!(result.state, 10); // State from func_process
    assert_eq!(result.context, Some("ctx1".to_string())); // Context from func_process (or logic)
    assert_eq!(
        result.logs.0,
        vec!["func_log".to_string(), "arg_log".to_string()]
    );
}

#[test]
fn test_monad_bind() {
    let process: TestProcess<i32> = CausalEffectPropagationProcess {
        value: EffectValue::Value(10),
        state: 1,
        context: Some("ctx".to_string()),
        error: None,
        logs: TestLog(vec!["start".to_string()]),
    };

    let bound: TestProcess<String> = TestWitness::bind(process, |x| {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(x.to_string()),
            state: 999, // Should be ignored/overwritten by bind implementation?
            // Bind impl:
            // state: m_a.state,     // State is passed through, not updated by f
            // context: m_a.context, // Context is passed through
            context: Some("ignored".to_string()),
            error: None,
            logs: TestLog(vec!["bound".to_string()]),
        }
    });

    assert_eq!(unwrap_value(bound.value), "10");
    assert_eq!(bound.state, 1); // From original process
    assert_eq!(bound.context, Some("ctx".to_string())); // From original process
    assert_eq!(bound.logs.0, vec!["start".to_string(), "bound".to_string()]);
}
