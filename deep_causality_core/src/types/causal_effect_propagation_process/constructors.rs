/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::causality_error::CausalityError;
use crate::{CausalEffectPropagationProcess, CausalMonad, EffectLog, EffectValue};
use core::fmt::Debug;
use deep_causality_haft::MonadEffect5;

// These constructors are generic over State and Context and can be used by any alias.
impl<Value, State, Context>
    CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>
where
    Value: Default + Clone + Debug,
    State: Default + Clone + Debug,
    Context: Clone + Debug,
{
    /// Creates a new process that explicitly contains an error.
    /// The state is set to default.
    pub fn from_error(err: CausalityError) -> Self {
        Self {
            value: EffectValue::None,
            state: State::default(),
            context: None,
            error: Some(err),
            logs: EffectLog::new(),
        }
    }

    /// Creates a new process with `EffectValue::None`, default state, and no error.
    pub fn none() -> Self {
        Self {
            value: EffectValue::None,
            state: State::default(),
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    /// Lifts a pure value into a process with a default state.
    pub fn pure(value: Value) -> Self {
        CausalMonad::<State, Context>::pure(value)
    }

    /// Creates a new process from a given `EffectValue`.
    /// The state is set to default.
    pub fn from_effect_value(effect_value: EffectValue<Value>) -> Self {
        Self {
            value: effect_value,
            state: State::default(),
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    /// Creates a new process from a given `EffectValue` and `EffectLog`.
    /// The state is set to default.
    pub fn from_effect_value_with_log(value: EffectValue<Value>, logs: EffectLog) -> Self {
        Self {
            value,
            state: State::default(),
            context: None,
            error: None,
            logs,
        }
    }
}
