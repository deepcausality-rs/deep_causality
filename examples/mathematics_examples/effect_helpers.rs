/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared helpers for the three `effect_*` examples.
//!
//! Each example wraps numerical state in a `CausalEffectPropagationProcess`
//! and chains transformations through `Monad::bind`. The aliases and helpers
//! here remove that ceremony so the examples can focus on the mathematics.

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness, CausalityError,
    CausalityErrorEnum, EffectValue,
};
use deep_causality_haft::LogAppend;

/// A simple log type that accumulates string messages across `bind` calls.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StepLog(pub Vec<String>);

impl LogAppend for StepLog {
    fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
}

impl StepLog {
    pub fn one<S: Into<String>>(msg: S) -> Self {
        Self(vec![msg.into()])
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

/// Process alias: `Value` is open; state and context are unit; error is the
/// canonical `CausalityError`; logs are `StepLog`.
pub type Process<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, StepLog>;

/// Witness alias for the above process.
pub type ProcessWitness = CausalEffectPropagationProcessWitness<(), (), CausalityError, StepLog>;

/// Build a successful step result with an accompanying log line.
pub fn ok<T, S: Into<String>>(value: T, msg: S) -> Process<T> {
    CausalEffectPropagationProcess {
        value: EffectValue::Value(value),
        state: (),
        context: None,
        error: None,
        logs: StepLog::one(msg),
    }
}

/// Build a failing step. The error short-circuits subsequent `bind` calls.
pub fn fail<T, S: Into<String>>(reason: S) -> Process<T> {
    CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: (),
        context: None,
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            reason.into(),
        ))),
        logs: StepLog::one("failed step"),
    }
}

/// Unwrap the final value for printing. Panics if the chain errored; the
/// examples check `process.error` before calling this.
pub fn expect_value<T: std::fmt::Debug + Clone>(v: &EffectValue<T>) -> T {
    match v {
        EffectValue::Value(t) => t.clone(),
        other => panic!("Expected EffectValue::Value, got {:?}", other),
    }
}

/// Pretty-print the accumulated step log.
pub fn print_log(log: &StepLog) {
    for (i, line) in log.0.iter().enumerate() {
        println!("  [{:>2}] {}", i, line);
    }
}
