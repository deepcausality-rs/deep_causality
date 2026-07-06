/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness, CausalityError,
    CausalityErrorEnum, EffectValue,
};
use deep_causality_haft::{Applicative, Functor, LogAppend, Pure};

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
    let process: TestProcess<i32> = CausalEffectPropagationProcess::new(
        Ok(EffectValue::Value(10)),
        1,
        Some("ctx".to_string()),
        TestLog(vec!["log1".to_string()]),
    );

    let mapped: TestProcess<String> = TestWitness::fmap(process, |x| x.to_string());

    let (outcome, state, context, logs) = mapped.into_parts();
    assert_eq!(unwrap_value(outcome.unwrap()), "10"); // Value transformed
    assert_eq!(state, 1); // State preserved
    assert_eq!(context, Some("ctx".to_string())); // Context preserved
    assert_eq!(logs.0, vec!["log1".to_string()]); // Logs preserved
}

#[test]
fn test_functor_fmap_short_circuits_on_error() {
    // The witness fmap is a left zero on errored carriers: `f` is not invoked and
    // the error propagates (no panic).
    let process: TestProcess<i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        1,
        Some("ctx".to_string()),
        TestLog(vec!["log1".to_string()]),
    );

    let mut called = false;
    let mapped: TestProcess<String> = TestWitness::fmap(process, |x| {
        called = true;
        x.to_string()
    });

    assert!(!called, "fmap must short-circuit on error and not call f");
    assert!(mapped.is_err());
    assert_eq!(
        mapped.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError
    );
    assert!(mapped.value().is_none()); // An errored carrier holds no value
    assert_eq!(*mapped.state(), 1); // State preserved
    assert_eq!(mapped.context(), &Some("ctx".to_string())); // Context preserved
    assert_eq!(mapped.logs().0, vec!["log1".to_string()]); // Logs preserved
}

#[test]
#[should_panic(expected = "Functor fmap on a non-error process should contain a value")]
fn test_functor_fmap_panics_on_value_less_ok() {
    // The generic witness cannot manufacture an error of type `E` for a value-less
    // Ok carrier, so it still panics via `expect`.
    let process: TestProcess<i32> =
        CausalEffectPropagationProcess::new(Ok(EffectValue::None), 1, None, TestLog::default());

    let _mapped: TestProcess<String> = TestWitness::fmap(process, |x| x.to_string());
}

#[test]
fn test_applicative_pure() {
    let process: TestProcess<i32> = TestWitness::pure(42);

    assert!(matches!(process.value(), Some(&42)));
    assert_eq!(*process.state(), 0); // Default state
    assert_eq!(process.context(), &None); // Default context (None)
    assert!(process.logs().0.is_empty());
}

#[test]
fn test_applicative_apply() {
    let func_process: TestProcess<fn(i32) -> i32> = CausalEffectPropagationProcess::new(
        Ok(EffectValue::Value(double)),
        10,
        Some("ctx1".to_string()),
        TestLog(vec!["func_log".to_string()]),
    );

    let arg_process: TestProcess<i32> = CausalEffectPropagationProcess::new(
        Ok(EffectValue::Value(5)),
        20,
        Some("ctx2".to_string()),
        TestLog(vec!["arg_log".to_string()]),
    );

    let result: TestProcess<i32> = TestWitness::apply(func_process, arg_process);

    let (outcome, state, context, logs) = result.into_parts();
    assert_eq!(unwrap_value(outcome.unwrap()), 10);
    assert_eq!(state, 10); // State from func_process
    assert_eq!(context, Some("ctx1".to_string())); // Context from func_process (or logic)
    assert_eq!(logs.0, vec!["func_log".to_string(), "arg_log".to_string()]);
}

#[test]
fn test_applicative_apply_short_circuits_on_error() {
    // When both sides are errored, the FIRST error encountered (f_ab's) propagates,
    // the function is never invoked, and logs from both sides are still combined.
    let func_process: TestProcess<fn(i32) -> i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
        10,
        Some("ctx1".to_string()),
        TestLog(vec!["func_log".to_string()]),
    );

    let arg_process: TestProcess<i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
        20,
        None,
        TestLog(vec!["arg_log".to_string()]),
    );

    let result: TestProcess<i32> = TestWitness::apply(func_process, arg_process);

    assert!(result.is_err());
    assert_eq!(
        result.error().unwrap().0,
        CausalityErrorEnum::InternalLogicError,
        "f_ab's error takes precedence"
    );
    assert!(result.value().is_none()); // An errored carrier holds no value
    assert_eq!(*result.state(), 10); // State from func_process
    assert_eq!(result.context(), &Some("ctx1".to_string()));
    assert_eq!(
        result.logs().0,
        vec!["func_log".to_string(), "arg_log".to_string()],
        "logs from both sides are combined even on error"
    );
}

#[test]
fn test_applicative_apply_propagates_argument_error() {
    let func_process: TestProcess<fn(i32) -> i32> = CausalEffectPropagationProcess::new(
        Ok(EffectValue::Value(double)),
        10,
        None,
        TestLog::default(),
    );

    let arg_process: TestProcess<i32> = CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::ValueNotAvailable)),
        20,
        Some("ctx2".to_string()),
        TestLog::default(),
    );

    let result: TestProcess<i32> = TestWitness::apply(func_process, arg_process);

    assert!(result.is_err());
    assert_eq!(
        result.error().unwrap().0,
        CausalityErrorEnum::ValueNotAvailable
    );
    assert_eq!(
        result.context(),
        &Some("ctx2".to_string()),
        "context falls back to f_a's when f_ab has none"
    );
}

// NOTE: the value-only `Monad::bind` on `CausalEffectPropagationProcessWitness` was removed because
// it could not thread the Markovian `State` channel (it froze state). State-threading bind behavior
// is covered by `types/causal_monad/causal_monad_tests.rs::test_bind_threads_and_updates_state`.
