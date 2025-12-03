/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Causal Monad
//!
//! This module defines the core monadic structure for handling causal effects within the
//! DeepCausality framework. It leverages Higher-Kinded Types (HKT) and the monadic pattern
//! to provide a robust, composable, and error-aware mechanism for propagating effects
//! through causal graphs and collections.
//!
//! The primary goal is to ensure that causal computations are:
//! - **Composable**: Effects can be chained together, with the output of one becoming
//!   the input of the next.
//! - **Error-aware**: Errors are propagated automatically, short-circuiting further
//!   computation in a controlled manner.
//! - **Log-preserving**: A complete history of operations and their outcomes is
//!   maintained for auditability and explanation.
//! - **Type-safe**: Leveraging Rust's type system to enforce correct usage and
//!   prevent common pitfalls in complex data flows.
//!
//! ## Monadic Operations
//!
//! The monadic pattern, as implemented here, provides two fundamental operations:
//!
//! 1.  **`pure(value)`**: Lifts a plain value into the monadic context. In this case,
//!     it creates a `CausalEffectPropagationProcess` containing the value, with no error
//!     and an empty log. This is the entry point for introducing a value into the
//!     causal effect system.
//!
//! 2.  **`bind(effect, f)`**: Chains a computation (`f`) to an existing `effect`.
//!     - If the `effect` contains an error, `bind` short-circuiting, propagating the
//!       error and the accumulated logs without executing `f`.
//!     - If the `effect` is successful, `bind` extracts its value, applies the
//!       function `f` (which itself returns a new effect), and then merges the
//!       logs from the original effect with the logs from the new effect.
//!       This ensures a continuous and complete log history.
//!
//! This design ensures that complex causal reasoning flows can be expressed clearly,
//! with built-in error handling and comprehensive logging, adhering to functional
//! programming principles within a performant Rust environment.

use crate::Intervenable;
use crate::types::causal_system::CausalSystem;
use crate::{
    CausalEffectPropagationProcess, CausalityError, CausalityErrorEnum, EffectLog, EffectValue,
};
use core::marker::PhantomData;
use deep_causality_haft::{Effect5, Functor, LogAddEntry, LogAppend, MonadEffect5};

/// `CausalMonad` is the concrete implementation of the `MonadEffect5` trait for the
/// `CausalSystem`. It provides the fundamental `pure` and `bind` operations
/// that enable monadic programming within the DeepCausality framework.
///
/// This monad is designed to manage the flow of `CausalEffectPropagationProcess`s, ensuring
/// that errors are handled gracefully and a comprehensive log of operations is maintained.
pub struct CausalMonad<S = (), C = ()>(PhantomData<(S, C)>);

impl<S, C> MonadEffect5<CausalSystem<S, C>> for CausalMonad<S, C>
where
    S: Clone + Default,
    C: Clone,
    <CausalSystem<S, C> as Effect5>::HktWitness:
        Functor<<CausalSystem<S, C> as Effect5>::HktWitness> + Sized,
{
    fn pure<T>(value: T) -> CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog> {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(value),
            state: S::default(),
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    fn bind<T, U, Func>(
        process: CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>,
        mut f: Func,
    ) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>
    where
        Func: FnMut(T) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>,
        U: Default,
    {
        if let Some(error) = process.error {
            return CausalEffectPropagationProcess {
                value: EffectValue::Value(U::default()),
                state: process.state,
                context: process.context,
                error: Some(error),
                logs: process.logs,
            };
        }

        let value = match process.value.into_value() {
            Some(v) => v,
            None => {
                return CausalEffectPropagationProcess {
                    value: EffectValue::Value(U::default()),
                    state: process.state,
                    context: process.context,
                    error: Some(CausalityError::new(CausalityErrorEnum::InternalLogicError)),
                    logs: process.logs,
                };
            }
        };
        let mut next_process = f(value);

        next_process.state = process.state;
        next_process.context = process.context;

        let mut combined_logs = process.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}

impl<S, C> Intervenable<CausalSystem<S, C>> for CausalMonad<S, C>
where
    S: Clone + Default,
    C: Clone,
{
    fn intervene<T>(
        effect: CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>,
        new_value: T,
    ) -> CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog> {
        // 1. Preserve the incoming logs and add a new entry for the intervention.
        let mut new_logs = effect.logs;
        new_logs.add_entry("Intervention occurred");

        // 2. Construct the new effect.
        CausalEffectPropagationProcess {
            // The value is replaced with the intervention value.
            value: EffectValue::Value(new_value),
            // The state is preserved.
            state: effect.state,
            // The context is preserved.
            context: effect.context,
            // The error state is preserved.
            error: effect.error,
            // The updated logs are carried forward.
            logs: new_logs,
        }
    }
}
