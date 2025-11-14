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
//! ## Key Components
//!
//! - [`CausalEffectSystem`]: A marker struct that defines the fixed types for the
//!   monad's error and log components, making it a concrete instance of an `Effect3`
//!   from the `deep_causality_haft` crate.
//! - [`CausalMonad`]: The concrete implementation of `MonadEffect3` for the
//!   `CausalEffectSystem`, providing the `pure` and `bind` operations.
//!
//! ## Monadic Operations
//!
//! The monadic pattern, as implemented here, provides two fundamental operations:
//!
//! 1.  **`pure(value)`**: Lifts a plain value into the monadic context. In this case,
//!     it creates a `CausalPropagatingEffect` containing the value, with no error
//!     and an empty log. This is the entry point for introducing a value into the
//!     causal effect system.
//!
//! 2.  **`bind(effect, f)`**: Chains a computation (`f`) to an existing `effect`.
//!     - If the `effect` contains an error, `bind` short-circuits, propagating the
//!       error and the accumulated logs without executing `f`.
//!     - If the `effect` is successful, `bind` extracts its value, applies the
//!       function `f` (which itself returns a new effect), and then merges the
//!       logs from the original effect with the logs from the new effect.
//!       This ensures a continuous and complete log history.
//!
//! This design ensures that complex causal reasoning flows can be expressed clearly,
//! with built-in error handling and comprehensive logging, adhering to functional
//! programming principles within a performant Rust environment.
use crate::{CausalEffectLog, CausalPropagatingEffect, CausalityError, PropagatingEffectWitness};
use deep_causality_haft::{Effect3, Functor, HKT3, MonadEffect3};

/// `CausalEffectSystem` is a marker struct that serves as a concrete instance of the
/// `Effect3` trait from the `deep_causality_haft` crate.
///
/// It explicitly defines the fixed types for the error and log components that will be
/// carried alongside the primary value within the monadic context. This allows the
/// `CausalMonad` to operate on a consistent structure for error propagation and logging.
///
/// By implementing `Effect3`, `CausalEffectSystem` declares:
/// - `Fixed1 = CausalityError`: The type used for representing errors in the causal system.
///   When an operation fails, a `CausalityError` is propagated.
/// - `Fixed2 = CausalEffectLog`: The type used for accumulating a history of operations.
///   Every step in a monadic chain can add entries to this log.
/// - `HktWitness = PropagatingEffectWitness<Self::Fixed1, Self::Fixed2>`: A phantom type
///   that links this system to the `CausalPropagatingEffect` structure, which is the
///   actual container for the value, error, and logs.
///
/// This setup is crucial for enabling the Higher-Kinded Type (HKT) pattern, allowing
/// generic monadic operations over `CausalPropagatingEffect` instances.
pub struct CausalEffectSystem;

impl Effect3 for CausalEffectSystem {
    type Fixed1 = CausalityError;
    type Fixed2 = CausalEffectLog;
    type HktWitness = PropagatingEffectWitness<Self::Fixed1, Self::Fixed2>;
}

/// `CausalMonad` is the concrete implementation of the `MonadEffect3` trait for the
/// `CausalEffectSystem`. It provides the fundamental `pure` and `bind` operations
/// that enable monadic programming within the DeepCausality framework.
///
/// This monad is designed to manage the flow of `CausalPropagatingEffect`s, ensuring
/// that errors are handled gracefully and a comprehensive log of operations is maintained.
///
/// ## `pure` function
///
/// `pure(value)` lifts a raw value `T` into the monadic context, creating a new
/// `CausalPropagatingEffect`. This effect starts with:
/// - The provided `value`.
/// - No error (`None`).
/// - An empty `CausalEffectLog`.
///
/// This is the entry point for any value that needs to enter the causal effect system
/// and participate in monadic computations.
///
/// ## `bind` function
///
/// `bind(effect, f)` is the sequencing operator of the monad. It takes an existing
/// `CausalPropagatingEffect` and a function `f` (which itself returns a new effect),
/// and orchestrates their execution:
///
/// 1.  **Error Short-circuiting**: It first checks if the `incoming_effect` already
///     contains an error. If so, `f` is *not* executed, and a new `CausalPropagatingEffect`
///     is immediately returned, carrying forward the original error and logs. The `value`
///     of the new effect is set to `U::default()` as the computation was aborted.
///
/// 2.  **Function Application**: If the `incoming_effect` is successful (no error),
///     its `value` is extracted and passed to the function `f`. The function `f`
///     then performs its computation and returns a `next_effect`.
///
/// 3.  **Log Aggregation**: The `logs` from the `incoming_effect` are combined with
///     the `logs` generated by the `next_effect` (from function `f`). This ensures
///     that the entire history of operations is preserved in the resulting effect.
///
/// The `bind` operation is crucial for building complex, sequential causal reasoning
/// pipelines where each step's outcome (value, error, and logs) correctly influences
/// the subsequent steps.
pub struct CausalMonad;

impl MonadEffect3<CausalEffectSystem> for CausalMonad
where
    <CausalEffectSystem as Effect3>::HktWitness:
        Functor<<CausalEffectSystem as Effect3>::HktWitness> + Sized,
{
    fn pure<T>(
        value: T,
    ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
        <CausalEffectSystem as Effect3>::Fixed1,
        <CausalEffectSystem as Effect3>::Fixed2,
    >>::Type<T> {
        CausalPropagatingEffect {
            value,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    fn bind<T, U, Func>(
        effect: <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
            <CausalEffectSystem as Effect3>::Fixed1,
            <CausalEffectSystem as Effect3>::Fixed2,
        >>::Type<T>,
        mut f: Func,
    ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
        <CausalEffectSystem as Effect3>::Fixed1,
        <CausalEffectSystem as Effect3>::Fixed2,
    >>::Type<U>
    where
        Func: FnMut(
            T,
        ) -> <<CausalEffectSystem as Effect3>::HktWitness as HKT3<
            <CausalEffectSystem as Effect3>::Fixed1,
            <CausalEffectSystem as Effect3>::Fixed2,
        >>::Type<U>,
        U: Default,
    {
        if let Some(error) = effect.error {
            return CausalPropagatingEffect {
                value: U::default(),
                error: Some(error),
                logs: effect.logs,
            };
        }

        let mut next_effect = f(effect.value);
        let mut combined_logs = effect.logs;
        combined_logs.append(&mut next_effect.logs);
        next_effect.logs = combined_logs;
        next_effect
    }
}
