/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectValue;
use deep_causality_haft::LogAppend;

pub mod constructors;
mod display;
mod explain;
pub mod hkt;
mod predicates;

#[derive(Debug, PartialEq, Clone)]
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    pub value: EffectValue<Value>,
    pub state: State,
    pub context: Option<Context>,
    pub error: Option<Error>,
    pub logs: Log,
}

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: LogAppend + Default,
    State: Clone,
    Context: Clone,
    Error: Clone,
{
    /// Chains a stateful, context-aware computation.
    ///
    /// This is the primary method for building Markovian process chains, as the
    /// function `f` receives the value, state, and context from the previous step.
    pub fn bind<F, NewValue>(
        self,
        f: F,
    ) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
    where
        F: FnOnce(
            EffectValue<Value>,
            State,
            Option<Context>,
        ) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>,
        NewValue: Default,
    {
        if let Some(error) = self.error {
            return CausalEffectPropagationProcess {
                value: EffectValue::default(),
                state: self.state,
                context: self.context,
                error: Some(error),
                logs: self.logs,
            };
        }

        let mut next_process = f(self.value, self.state, self.context);

        let mut combined_logs = self.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}

impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: Clone,
    Error: Clone,
{
    /// Lifts a stateless effect into a stateful process by providing an initial state and context.
    ///
    /// This is the primary entry point for starting a stateful computation chain from a
    /// simple, pre-existing effect.
    ///
    /// # Arguments
    /// * `effect`: The stateless `PropagatingEffect` (where State and Context are `()`).
    /// * `initial_state`: The starting state for the new process.
    /// * `initial_context`: The optional starting context for the new process.
    ///
    /// # Returns
    /// A new `CausalEffectPropagationProcess` ready for stateful operations.
    pub fn with_state(
        effect: CausalEffectPropagationProcess<Value, (), (), Error, Log>,
        initial_state: State,
        initial_context: Option<Context>,
    ) -> Self {
        Self {
            value: effect.value,
            state: initial_state,
            context: initial_context,
            error: effect.error,
            logs: effect.logs,
        }
    }
}
