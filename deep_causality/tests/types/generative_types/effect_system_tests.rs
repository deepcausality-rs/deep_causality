/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    AuditableGraphGenerator, GraphGeneratableEffect, GraphGeneratableEffectWitness,
    ModelValidationError, ModificationLog, ModificationLogEntry, OpStatus,
};
use deep_causality_haft::{Applicative, Functor, Monad, Pure};

// A simple test state
#[derive(Debug, Clone, PartialEq)]
struct MyState(u32);

// --- Functor Tests ---

#[test]
fn test_fmap_success() {
    let effect = GraphGeneratableEffect {
        value: Some(MyState(5)),
        error: None,
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Functor<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::fmap(effect, |state| MyState(state.0 * 2));

    assert_eq!(result.value, Some(MyState(10)));
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}

#[test]
fn test_fmap_with_error() {
    let effect = GraphGeneratableEffect {
        value: Some(MyState(5)),
        error: Some(ModelValidationError::TargetCausaloidNotFound { id: 1 }),
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Functor<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::fmap(effect, |state| MyState(state.0 * 2));

    // Value should be None, error should be preserved
    assert!(result.value.is_none());
    assert!(matches!(
        result.error,
        Some(ModelValidationError::TargetCausaloidNotFound { id: 1 })
    ));
    assert!(result.logs.is_empty());
}

#[test]
fn test_fmap_no_value_no_error() {
    let effect: AuditableGraphGenerator<MyState> = GraphGeneratableEffect {
        value: None,
        error: None,
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Functor<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::fmap(effect, |state| MyState(state.0 * 2));

    assert!(result.value.is_none());
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}

// --- Applicative Tests ---

#[test]
fn test_applicative_pure() {
    let result: AuditableGraphGenerator<u32> =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(10);

    assert_eq!(result.value, Some(10));
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}

#[test]
fn test_applicative_apply_success() {
    let f_ab: AuditableGraphGenerator<fn(u32) -> u32> = GraphGeneratableEffect {
        value: Some(|a| a * 2),
        error: None,
        logs: ModificationLog::new(),
    };
    let m_a: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: Some(5),
        error: None,
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Applicative<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::apply(f_ab, m_a);

    assert_eq!(result.value, Some(10));
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}

#[test]
fn test_applicative_apply_error_in_func() {
    let f_ab: AuditableGraphGenerator<fn(u32) -> u32> = GraphGeneratableEffect {
        value: None,
        error: Some(ModelValidationError::InterpreterError {
            reason: "function error".to_string(),
        }),
        logs: ModificationLog::new(),
    };
    let m_a: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: Some(5),
        error: None,
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Applicative<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::apply(f_ab, m_a);

    assert!(result.value.is_none());
    assert!(matches!(
        result.error,
        Some(ModelValidationError::InterpreterError { .. })
    ));
    assert!(result.logs.is_empty());
}

#[test]
fn test_applicative_apply_error_in_value() {
    let f_ab: AuditableGraphGenerator<fn(u32) -> u32> = GraphGeneratableEffect {
        value: Some(|a| a * 2),
        error: None,
        logs: ModificationLog::new(),
    };
    let m_a: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: None,
        error: Some(ModelValidationError::AddContextoidError {
            err: "value error".to_string(),
        }),
        logs: ModificationLog::new(),
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Applicative<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::apply(f_ab, m_a);

    assert!(result.value.is_none());
    assert!(matches!(
        result.error,
        Some(ModelValidationError::AddContextoidError { .. })
    ));
    assert!(result.logs.is_empty());
}

#[test]
fn test_applicative_apply_log_aggregation() {
    let mut logs1 = ModificationLog::new();
    logs1.add_entry(ModificationLogEntry::new(
        "log1".to_string(),
        "test_target".to_string(),
        OpStatus::Success,
        "test_message".to_string(),
    ));

    let mut logs2 = ModificationLog::new();
    logs2.add_entry(ModificationLogEntry::new(
        "log2".to_string(),
        "test_target".to_string(),
        OpStatus::Success,
        "test_message".to_string(),
    ));

    let f_ab: AuditableGraphGenerator<fn(u32) -> u32> = GraphGeneratableEffect {
        value: Some(|a| a * 2),
        error: None,
        logs: logs1,
    };
    let m_a: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: Some(5),
        error: None,
        logs: logs2,
    };

    let result =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Applicative<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::apply(f_ab, m_a);

    assert_eq!(result.value, Some(10));
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 2);
    assert!(
        result
            .logs
            .iter()
            .any(|entry| entry.operation_name == "log1")
    );
    assert!(
        result
            .logs
            .iter()
            .any(|entry| entry.operation_name == "log2")
    );
}

// --- Monad Tests ---

#[test]
fn test_monad_bind_success() {
    let effect: AuditableGraphGenerator<u32> =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(5);

    let result = <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Monad<
        GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
    >>::bind(effect, |val| {
        // This function returns a new effect
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(val * 2)
    });

    assert_eq!(result.value, Some(10));
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}

#[test]
fn test_monad_bind_error_propagation() {
    let effect: AuditableGraphGenerator<u32> =
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(5);

    let result = <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Monad<
        GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
    >>::bind(effect, |val| {
        // This function introduces an error
        GraphGeneratableEffect {
            value: Some(val * 2),
            error: Some(ModelValidationError::UnsupportedOperation {
                operation: "bind error".to_string(),
            }),
            logs: ModificationLog::new(),
        }
    });

    assert!(result.value.is_some()); // The value from the function should be propagated
    assert!(matches!(
        result.error,
        Some(ModelValidationError::UnsupportedOperation { .. })
    ));
    assert!(result.logs.is_empty());
}

#[test]
fn test_monad_bind_error_in_initial_effect() {
    let effect: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: Some(5),
        error: Some(ModelValidationError::UpdateNodeError {
            err: "initial error".to_string(),
        }),
        logs: ModificationLog::new(),
    };

    let result = <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Monad<
        GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
    >>::bind(effect, |val| {
        // This function would normally succeed, but the initial effect had an error
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(val * 2)
    });

    assert!(result.value.is_none());
    assert!(matches!(
        result.error,
        Some(ModelValidationError::UpdateNodeError { .. })
    ));
    assert!(result.logs.is_empty());
}

#[test]
fn test_monad_bind_log_aggregation() {
    let mut initial_logs = ModificationLog::new();
    initial_logs.add_entry(ModificationLogEntry::new(
        "initial_log".to_string(),
        "test_target".to_string(),
        OpStatus::Success,
        "test_message".to_string(),
    ));

    let effect: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: Some(5),
        error: None,
        logs: initial_logs,
    };

    let result = <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Monad<
        GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
    >>::bind(effect, |val| {
        let mut inner_logs = ModificationLog::new();
        inner_logs.add_entry(ModificationLogEntry::new(
            "inner_log".to_string(),
            "test_target".to_string(),
            OpStatus::Success,
            "test_message".to_string(),
        ));
        GraphGeneratableEffect {
            value: Some(val * 2),
            error: None,
            logs: inner_logs,
        }
    });

    assert_eq!(result.value, Some(10));
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 2);
    assert!(
        result
            .logs
            .iter()
            .any(|entry| entry.operation_name == "initial_log")
    );
    assert!(
        result
            .logs
            .iter()
            .any(|entry| entry.operation_name == "inner_log")
    );
}

#[test]
fn test_monad_bind_no_value_no_error() {
    let effect: AuditableGraphGenerator<u32> = GraphGeneratableEffect {
        value: None,
        error: None,
        logs: ModificationLog::new(),
    };

    let result = <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Monad<
        GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
    >>::bind(effect, |val| {
        <GraphGeneratableEffectWitness<ModelValidationError, ModificationLog> as Pure<
            GraphGeneratableEffectWitness<ModelValidationError, ModificationLog>,
        >>::pure(val * 2)
    });

    assert!(result.value.is_none());
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
}
