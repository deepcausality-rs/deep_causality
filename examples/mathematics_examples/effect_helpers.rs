/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared helpers for the three `effect_*` examples.
//!
//! Each example wraps numerical state in a `CausalEffectPropagationProcess`
//! and chains transformations through `Monad::bind`. The aliases and helpers
//! here remove that ceremony so the examples can focus on the mathematics.
//!
//! ## Why the raw monad and a custom log
//!
//! These `effect_*` examples stay on the lower-level `CausalEffectPropagationProcess`
//! rather than the `CausalFlow` DSL, because they want a custom log type (`StepLog`).
//! `CausalFlow` fixes its log to the framework default, `EffectLog`; the raw monad leaves
//! the `Log` parameter open, so an example can supply whatever log serves it best. See
//! `StepLog` for why that matters here.

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness,
    CausalityError, CausalityErrorEnum,
};
use deep_causality_haft::LogAppend;

/// An ordered, human-readable transcript of a `bind` chain: one message per step,
/// concatenated in execution order and printed at the end by `print_log`.
///
/// ## Why `StepLog` rather than the default `EffectLog`
///
/// The framework default, `EffectLog` (the log the `CausalFlow` DSL uses), is a structured
/// causal-audit log. It records provenance such as wall-clock timestamps and intervention
/// markers, which is what a causal-reasoning pipeline wants. These composable-multi-math
/// examples want the opposite: a minimal, deterministic record of what each algebra or
/// tensor step did, with no timestamps, so the printed output is reproducible across runs
/// and the reader's attention stays on the mathematics instead of audit metadata.
///
/// The `Log` type is the only parameter that differs, so choosing `StepLog` is also what
/// keeps these examples on the raw `Process` / `ProcessWitness` monad instead of the
/// `CausalFlow` DSL, which pins its log to `EffectLog`.
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
/// canonical `CausalityError`; logs are the `StepLog` custom type.
pub type Process<T> = CausalEffectPropagationProcess<T, (), (), CausalityError, StepLog>;

/// Witness alias for the above process.
pub type ProcessWitness = CausalEffectPropagationProcessWitness<(), (), CausalityError, StepLog>;

/// Build a successful step result with an accompanying log line.
pub fn ok<T, S: Into<String>>(value: T, msg: S) -> Process<T> {
    CausalEffectPropagationProcess::new(Ok(CausalEffect::value(value)), (), None, StepLog::one(msg))
}

/// Build a failing step. The error short-circuits subsequent `bind` calls.
pub fn fail<T, S: Into<String>>(reason: S) -> Process<T> {
    CausalEffectPropagationProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::Custom(
            reason.into(),
        ))),
        (),
        None,
        StepLog::one("failed step"),
    )
}

/// Pretty-print the accumulated step log.
pub fn print_log(log: &StepLog) {
    println!("Chain log:");
    for (i, line) in log.0.iter().enumerate() {
        println!("  [{:>2}] {}", i, line);
    }
}
