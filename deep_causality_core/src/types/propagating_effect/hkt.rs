/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module provides Higher-Kinded Type (HKT) implementations for `PropagatingEffect`.
//!
//! By implementing `HKT`, `Functor`, `Applicative`, and `Monad` traits from the `deep_causality_haft` crate,
//! this module enables `PropagatingEffect` to be used in a functional programming style.
//! This allows for chaining operations, transforming values, and handling errors and logs
//! in a structured and composable manner, similar to how monads and functors work in other languages.
//!

use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;
use crate::{EffectLog, EffectValue};
use deep_causality_haft::{Applicative, Functor, HKT, HKT3, LogAppend, Monad, Placeholder};
use std::marker::PhantomData;

/// The `PropagatingEffectWitness` acts as a marker type to associate the `PropagatingEffect`
/// with these HKT traits, facilitating generic programming over different causal effect types.
pub struct PropagatingEffectWitness<E, L>(Placeholder, PhantomData<E>, PhantomData<L>);

/// Implements the `HKT` trait for `PropagatingEffectWitness`.
///
/// `PropagatingEffectWitness<E, L>` serves as a "witness" or "tag" type that
/// associates the `CausalEffectPropagationProcess<T, (), (), E, L>` with the Higher-Kinded Type
/// abstraction. This allows `PropagatingEffect` to be treated generically
/// by traits like `Functor`, `Applicative`, and `Monad`.
///
/// The `Type<T>` associated type specifies that `CausalEffectPropagationProcess<T, (), (), E, L>`
/// is the concrete type that this HKT represents, where `T` is the inner value type,
/// `E` is the error type, and `L` is the log type.
impl<E, L> HKT for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalEffectPropagationProcess<T, (), (), E, L>;
}

/// Implements the `HKT3` trait for `PropagatingEffectWitness`.
///
/// Similar to `HKT`, this implementation specifically caters to types that have
/// three type parameters, where `E` and `L` are fixed, and `T` is the varying parameter.
/// This provides a more specific HKT abstraction for `PropagatingEffect`.
impl<E, L> HKT3<E, L> for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalEffectPropagationProcess<T, (), (), E, L>;
}

/// Implements the `Functor` trait for `PropagatingEffectWitness`.
///
/// The `Functor` trait provides the `fmap` method, which allows transforming
/// the inner value of a `PropagatingEffect` without changing its structure,
/// error state, or associated logs.
///
/// # Arguments
/// * `m_a` - A `PropagatingEffect` instance containing a value of type `A`.
/// * `f` - A function that takes a value of type `A` and returns a value of type `B`.
///
/// # Returns
/// A new `PropagatingEffect` instance with the transformed value of type `B`,
/// preserving the original error and logs.
impl<E, L> Functor<PropagatingEffectWitness<E, L>> for PropagatingEffectWitness<E, L>
where
    E: 'static,
    L: 'static,
{
    fn fmap<A, B, Func>(
        m_a: <PropagatingEffectWitness<E, L> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingEffectWitness<E, L> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(f(m_a
                .value
                .into_value()
                .expect("Functor fmap on a non-error effect should contain a value"))),
            state: (),
            context: None,
            error: m_a.error,
            logs: m_a.logs,
        }
    }
}

/// Implements the `Applicative` trait for `PropagatingEffectWitness`.
///
/// The `Applicative` trait extends `Functor` by providing methods to lift
/// pure values into the `PropagatingEffect` context (`pure`) and to apply
/// a function wrapped in `PropagatingEffect` to a value wrapped in `PropagatingEffect` (`apply`).
/// This allows for combining independent `PropagatingEffect`s.
impl<E> Applicative<PropagatingEffectWitness<E, EffectLog>>
    for PropagatingEffectWitness<E, EffectLog>
where
    E: 'static + Clone,
{
    /// Lifts a pure value into a `PropagatingEffect` context.
    ///
    /// Creates a new `PropagatingEffect` with the given `value`,
    /// an empty error state (`None`), and an empty `CausalEffectLog`.
    ///
    /// # Arguments
    /// * `value` - The value to lift.
    ///
    /// # Returns
    /// A `PropagatingEffect` containing the lifted value.
    ///
    /// # Example
    /// ```
    /// use deep_causality_haft::{Applicative, HKT};
    /// use deep_causality_core::{PropagatingEffect, PropagatingEffectWitness, EffectLog, EffectValue};
    ///
    /// let effect = PropagatingEffectWitness::<&'static str, EffectLog>::pure(5);
    /// assert!(matches!(effect.value, EffectValue::Value(5)));
    /// ```
    fn pure<T>(value: T) -> <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<T> {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(value),
            state: (),
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    /// Applies a function wrapped in a `PropagatingEffect` to a value
    /// wrapped in a `PropagatingEffect`.
    ///
    /// This method combines two `PropagatingEffect` instances: one holding
    /// a function (`f_ab`) and another holding a value (`f_a`).
    ///
    /// # Error Handling
    /// If either `f_ab` or `f_a` contains an error, that error is propagated
    /// to the resulting `PropagatingEffect`. The function application
    /// still occurs, but the error state is preserved.
    ///
    /// # Log Aggregation
    /// The logs from both `f_ab` and `f_a` are combined (appended) into the
    /// resulting `PropagatingEffect`'s logs.
    ///
    /// # Arguments
    /// * `f_ab` - A `PropagatingEffect` containing a function `Func: FnMut(A) -> B`.
    /// * `f_a` - A `PropagatingEffect` containing a value of type `A`.
    ///
    /// # Returns
    /// A new `PropagatingEffect` with the result of the function application,
    /// propagated errors, and combined logs.
    fn apply<A, B, Func>(
        f_ab: <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<Func>,
        mut f_a: <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<A>,
    ) -> <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        // Always combine logs from both effects.
        let mut combined_logs = f_ab.logs;
        combined_logs.append(&mut f_a.logs);

        // If the function effect has an error, propagate it.
        // The value is still computed, but the error takes precedence.
        if f_ab.error.is_some() {
            return CausalEffectPropagationProcess {
                value: EffectValue::Value((f_ab.value.into_value().expect(
                    "Applicative apply (function) on a non-error effect should contain a value",
                ))(f_a.value.into_value().expect(
                    "Applicative apply (value) on a non-error effect should contain a value",
                ))),
                state: (),
                context: None,
                error: f_ab.error,
                logs: combined_logs,
            };
        }
        // If the value effect has an error, propagate it.
        // The value is still computed, but the error takes precedence.
        if f_a.error.is_some() {
            return CausalEffectPropagationProcess {
                value: EffectValue::Value((f_ab.value.into_value().expect(
                    "Applicative apply (function) on a non-error effect should contain a value",
                ))(f_a.value.into_value().expect(
                    "Applicative apply (value) on a non-error effect should contain a value",
                ))),
                state: (),
                context: None,
                error: f_a.error,
                logs: combined_logs,
            };
        }

        CausalEffectPropagationProcess {
            value: EffectValue::Value((f_ab.value.into_value().expect(
                "Applicative apply (function) on a non-error effect should contain a value",
            ))(
                f_a.value.into_value().expect(
                    "Applicative apply (value) on a non-error effect should contain a value",
                ),
            )),
            state: (),
            context: None,
            error: None,
            logs: combined_logs,
        }
    }
}

/// Implements the `Monad` trait for `PropagatingEffectWitness`.
///
/// The `Monad` trait provides the `bind` method, which is crucial for sequencing
/// computations that produce `PropagatingEffect`s. It allows for chaining
/// operations where each subsequent operation might depend on the result of the
/// previous one, while correctly handling errors and aggregating logs.
impl<E> Monad<PropagatingEffectWitness<E, EffectLog>> for PropagatingEffectWitness<E, EffectLog>
where
    E: 'static + Clone,
{
    /// Sequences a computation by applying a function that returns another
    /// `PropagatingEffect`.
    ///
    /// # Error Handling (Short-circuiting)
    /// If the initial `PropagatingEffect` (`m_a`) contains an error,
    /// the `bind` operation short-circuits. The function `f` is not applied,
    /// and a new `PropagatingEffect` is returned immediately with the
    /// original error and logs. This ensures that subsequent computations
    /// are not performed if an error has already occurred.
    ///
    /// # Log Aggregation
    /// If `m_a` does not contain an error, the function `f` is applied to
    /// `m_a.value` to produce `next_effect`. The logs from `m_a` are then
    /// combined (appended) with the logs from `next_effect`.
    ///
    /// # Arguments
    /// * `m_a` - The initial `PropagatingEffect` instance.
    /// * `f` - A function that takes the inner value of `m_a` and returns a new `PropagatingEffect`.
    ///
    /// # Returns
    /// A new `PropagatingEffect` representing the sequenced computation,
    /// with errors propagated and logs aggregated.
    fn bind<A, B, Func>(
        m_a: <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> <PropagatingEffectWitness<E, EffectLog> as HKT>::Type<B>,
    {
        // If there's an error in the initial effect, short-circuit and propagate it.
        // The function `f` is still called to get a default value for `B`,
        // but the error and logs from `m_a` are preserved.
        if m_a.error.is_some() {
            return CausalEffectPropagationProcess {
                value: EffectValue::Value(f(m_a.value.into_value().expect("Monad bind on a non-error effect should contain a value")).value.into_value().expect("Monad bind (resulting effect) on a non-error effect should contain a value")), // Call f to get a default value for B, but its error/logs are ignored.
                state: (),
                context: None,
                error: m_a.error,
                logs: m_a.logs,
            };
        }
        let mut next_effect = f(m_a
            .value
            .into_value()
            .expect("Monad bind on a non-error effect should contain a value"));
        let mut combined_logs = m_a.logs;
        combined_logs.append(&mut next_effect.logs);
        next_effect.logs = combined_logs;
        next_effect
    }
}
