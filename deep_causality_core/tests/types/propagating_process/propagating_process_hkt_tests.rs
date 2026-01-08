/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use deep_causality_core::{
    CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum, EffectLog, EffectValue,
    PropagatingProcessWitness,
};
use deep_causality_haft::{
    Applicative, Functor, HKT, HKT5, LogAddEntry, LogAppend, LogSize, Monad,
};

// --- Mock Types for Testing ---

#[derive(Clone, Default, Debug, PartialEq)]
struct TestState(u32);

#[derive(Clone, Debug, PartialEq)]
struct TestContext(String);

impl Default for TestContext {
    fn default() -> Self {
        TestContext("default_context".to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TestError {
    TestSpecificError,
    Causality(CausalityError),
}

impl From<CausalityError> for TestError {
    fn from(err: CausalityError) -> Self {
        TestError::Causality(err)
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
struct TestLog {
    entries: Vec<String>,
}

impl LogAddEntry for TestLog {
    fn add_entry(&mut self, message: &str) {
        self.entries.push(message.to_string());
    }
}

impl LogAppend for TestLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

// --- HKT Trait Tests ---

#[test]
fn test_hkt_type_association() {
    type Witness<S, C> = PropagatingProcessWitness<S, C>;
    type PropProcess<V, S, C> = CausalEffectPropagationProcess<V, S, C, CausalityError, EffectLog>;

    // Verify that the HKT's associated type correctly resolves to PropagatingProcess
    let _ = <Witness<TestState, TestContext> as HKT>::Type::<i32>::pure(10);
    // This compilation check is sufficient to ensure the type association is correct.
}

// --- Functor Trait Tests ---

#[test]
fn test_fmap_success() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Initial log");
            log
        },
    };

    let fmapped_process = <PropagatingProcessWitness<TestState, TestContext> as Functor<_>>::fmap(
        initial_process,
        |x| x * 2,
    );

    assert_eq!(fmapped_process.value, EffectValue::Value(10));
    assert_eq!(fmapped_process.state, TestState(1));
    assert_eq!(
        fmapped_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(fmapped_process.error, None);
    assert_eq!(fmapped_process.logs.len(), 1);
    assert!(format!("{}", fmapped_process.logs).contains("Initial log"));
}

#[test]
fn test_fmap_on_error() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: Some(CausalityError(CausalityErrorEnum::InternalLogicError)),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Error log");
            log
        },
    };

    let fmapped_process = <PropagatingProcessWitness<TestState, TestContext> as Functor<_>>::fmap(
        initial_process,
        |x| x * 2,
    );

    // Function should not be applied, error and logs preserved
    assert_eq!(fmapped_process.value, EffectValue::None);
    assert_eq!(fmapped_process.state, TestState(1));
    assert_eq!(
        fmapped_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(
        fmapped_process.error,
        Some(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    assert_eq!(fmapped_process.logs.len(), 1);
    assert!(format!("{}", fmapped_process.logs).contains("Error log"));
}

#[test]
fn test_fmap_on_none_value_produces_internal_logic_error() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: None, // No error initially
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("None value log");
            log
        },
    };

    let fmapped_process = <PropagatingProcessWitness<TestState, TestContext> as Functor<_>>::fmap(
        initial_process,
        |x| x * 2,
    );

    // Expecting an InternalLogicError because fmap tried to unwrap a None value
    assert_eq!(fmapped_process.value, EffectValue::None);
    assert_eq!(fmapped_process.state, TestState(1));
    assert_eq!(
        fmapped_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(
        fmapped_process.error,
        Some(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    assert_eq!(fmapped_process.logs.len(), 1);
    assert!(format!("{}", fmapped_process.logs).contains("None value log"));
}

// --- Applicative Trait Tests ---

#[test]
fn test_applicative_pure() {
    let process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess::pure(100);

    assert_eq!(process.value, EffectValue::Value(100));
    assert_eq!(process.state, TestState::default());
    assert_eq!(process.context, None);
    assert_eq!(process.error, None);
    assert_eq!(process.logs.len(), 0);
}

#[test]
fn test_applicative_apply_success() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(|x| x + 1),
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(20), // Should be ignored, f_ab's state takes precedence
        context: Some(TestContext("val_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::Value(6));
    assert_eq!(result_process.state, TestState(10)); // State from f_ab is carried over
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    ); // Context from f_ab
    assert_eq!(result_process.error, None);
    assert_eq!(result_process.logs.len(), 2);
    assert!(format!("{}", result_process.logs).contains("Func process log"));
    assert!(format!("{}", result_process.logs).contains("Value process log"));
}

#[test]
fn test_applicative_apply_with_func_error() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(|x| x + 1),
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: Some(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process error log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(20),
        context: Some(TestContext("val_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::None);
    assert_eq!(result_process.state, TestState(10));
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    );
    assert_eq!(
        result_process.error,
        Some(CausalityError::new(CausalityErrorEnum::TypeConversionError))
    );
    assert_eq!(result_process.logs.len(), 2);
    assert!(format!("{}", result_process.logs).contains("Func process error log"));
    assert!(format!("{}", result_process.logs).contains("Value process log"));
}

#[test]
fn test_applicative_apply_with_value_error() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(|x| x + 1),
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(20),
        context: Some(TestContext("val_ctx".to_string())),
        error: Some(CausalityError::new(CausalityErrorEnum::MaxStepsExceeded)),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process error log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::None);
    assert_eq!(result_process.state, TestState(10));
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    );
    assert_eq!(
        result_process.error,
        Some(CausalityError::new(CausalityErrorEnum::MaxStepsExceeded))
    );
    assert_eq!(result_process.logs.len(), 2);
    assert!(format!("{}", result_process.logs).contains("Func process log"));
    assert!(format!("{}", result_process.logs).contains("Value process error log"));
}

#[test]
fn test_applicative_apply_with_both_errors() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(|x| x + 1),
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: Some(CausalityError::new(CausalityErrorEnum::TypeConversionError)),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process error log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(20),
        context: Some(TestContext("val_ctx".to_string())),
        error: Some(CausalityError::new(CausalityErrorEnum::MaxStepsExceeded)),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process error log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::None);
    assert_eq!(result_process.state, TestState(10));
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    );
    // Should prioritize f_ab's error
    assert_eq!(
        result_process.error,
        Some(CausalityError::new(CausalityErrorEnum::TypeConversionError))
    );
    assert_eq!(result_process.logs.len(), 2);
    assert!(format!("{}", result_process.logs).contains("Func process error log"));
    assert!(format!("{}", result_process.logs).contains("Value process error log"));
}

#[test]
fn test_applicative_apply_with_func_none_value_produces_internal_logic_error() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::None, // No function
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(20),
        context: Some(TestContext("val_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::None);
    assert_eq!(result_process.state, TestState(10));
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    );
    assert_eq!(
        result_process.error,
        Some(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    assert_eq!(result_process.logs.len(), 2);
}

#[test]
fn test_applicative_apply_with_value_none_value_produces_internal_logic_error() {
    let f_process: CausalEffectPropagationProcess<
        fn(i32) -> i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(|x| x + 1),
        state: TestState(10),
        context: Some(TestContext("func_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Func process log");
            log
        },
    };

    let a_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::None, // No value
        state: TestState(20),
        context: Some(TestContext("val_ctx".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Value process log");
            log
        },
    };

    let result_process =
        <PropagatingProcessWitness<TestState, TestContext> as Applicative<_>>::apply(
            f_process, a_process,
        );

    assert_eq!(result_process.value, EffectValue::None);
    assert_eq!(result_process.state, TestState(10));
    assert_eq!(
        result_process.context,
        Some(TestContext("func_ctx".to_string()))
    );
    assert_eq!(
        result_process.error,
        Some(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    assert_eq!(result_process.logs.len(), 2);
}

// --- Monad Trait Tests ---

#[test]
fn test_monad_bind_success() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Initial log entry");
            log
        },
    };

    let bound_process = <PropagatingProcessWitness<TestState, TestContext> as Monad<_>>::bind(
        initial_process,
        |x| {
            let mut new_log = EffectLog::new();
            new_log.add_entry("Bound function log");
            CausalEffectPropagationProcess {
                value: EffectValue::Value(x * 10),
                state: TestState(2), // State from bind is ignored
                context: Some(TestContext("bound".to_string())), // Context from bind is ignored
                error: None,
                logs: new_log,
            }
        },
    );

    assert_eq!(bound_process.value, EffectValue::Value(50));
    assert_eq!(bound_process.state, TestState(1)); // State preserved from initial process
    assert_eq!(
        bound_process.context,
        Some(TestContext("initial".to_string()))
    ); // Context preserved from initial process
    assert_eq!(bound_process.error, None);
    assert_eq!(bound_process.logs.len(), 2);
    assert!(format!("{}", bound_process.logs).contains("Initial log entry"));
    assert!(format!("{}", bound_process.logs).contains("Bound function log"));
}

#[test]
fn test_monad_bind_on_error_process() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: Some(CausalityError::new(
            CausalityErrorEnum::StartNodeOutOfBounds,
        )),
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Initial error log");
            log
        },
    };

    let bound_process = <PropagatingProcessWitness<TestState, TestContext> as Monad<_>>::bind(
        initial_process,
        |x| {
            // This function should not be called
            let mut new_log = EffectLog::new();
            new_log.add_entry("Should not see this log");
            CausalEffectPropagationProcess {
                value: EffectValue::Value(x * 10),
                state: TestState(2),
                context: Some(TestContext("bound".to_string())),
                error: None,
                logs: new_log,
            }
        },
    );

    assert_eq!(bound_process.value, EffectValue::None); // Value is None on error
    assert_eq!(bound_process.state, TestState(1));
    assert_eq!(
        bound_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(
        bound_process.error,
        Some(CausalityError::new(
            CausalityErrorEnum::StartNodeOutOfBounds
        ))
    );
    assert_eq!(bound_process.logs.len(), 1);
    assert!(format!("{}", bound_process.logs).contains("Initial error log"));
    assert!(!format!("{}", bound_process.logs).contains("Should not see this log"));
}

#[test]
fn test_monad_bind_on_none_value_process_produces_internal_logic_error() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Initial none value log");
            log
        },
    };

    let bound_process = <PropagatingProcessWitness<TestState, TestContext> as Monad<_>>::bind(
        initial_process,
        |x| {
            // This function should not be called
            let mut new_log = EffectLog::new();
            new_log.add_entry("Should not see this log");
            CausalEffectPropagationProcess {
                value: EffectValue::Value(x * 10),
                state: TestState(2),
                context: Some(TestContext("bound".to_string())),
                error: None,
                logs: new_log,
            }
        },
    );

    assert_eq!(bound_process.value, EffectValue::None);
    assert_eq!(bound_process.state, TestState(1));
    assert_eq!(
        bound_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(
        bound_process.error,
        Some(CausalityError::new(CausalityErrorEnum::InternalLogicError))
    );
    assert_eq!(bound_process.logs.len(), 1);
    assert!(format!("{}", bound_process.logs).contains("Initial none value log"));
    assert!(!format!("{}", bound_process.logs).contains("Should not see this log"));
}

#[test]
fn test_monad_bind_propagates_error_from_bound_function() {
    let initial_process: CausalEffectPropagationProcess<
        i32,
        TestState,
        TestContext,
        CausalityError,
        EffectLog,
    > = CausalEffectPropagationProcess {
        value: EffectValue::Value(5),
        state: TestState(1),
        context: Some(TestContext("initial".to_string())),
        error: None,
        logs: {
            let mut log = EffectLog::new();
            log.add_entry("Initial log entry");
            log
        },
    };

    let bound_process = <PropagatingProcessWitness<TestState, TestContext> as Monad<_>>::bind(
        initial_process,
        |x| {
            let mut new_log = EffectLog::new();
            new_log.add_entry("Error from bound func log");
            CausalEffectPropagationProcess {
                value: EffectValue::Value(x * 10),
                state: TestState(2),
                context: Some(TestContext("bound".to_string())),
                error: Some(CausalityError::new(CausalityErrorEnum::MaxStepsExceeded)),
                logs: new_log,
            }
        },
    );

    assert_eq!(bound_process.value, EffectValue::Value(50)); // Value still produced
    assert_eq!(bound_process.state, TestState(1));
    assert_eq!(
        bound_process.context,
        Some(TestContext("initial".to_string()))
    );
    assert_eq!(
        bound_process.error,
        Some(CausalityError::new(CausalityErrorEnum::MaxStepsExceeded))
    );
    assert_eq!(bound_process.logs.len(), 2);
    assert!(format!("{}", bound_process.logs).contains("Initial log entry"));
    assert!(format!("{}", bound_process.logs).contains("Error from bound func log"));
}
