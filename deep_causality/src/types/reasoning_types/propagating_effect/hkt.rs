/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module provides Higher-Kinded Type (HKT) implementations for `CausalPropagatingEffect`.
//!
//! By implementing `HKT`, `Functor`, `Applicative`, and `Monad` traits from the `deep_causality_haft` crate,
//! this module enables `CausalPropagatingEffect` to be used in a functional programming style.
//! This allows for chaining operations, transforming values, and handling errors and logs
//! in a structured and composable manner, similar to how monads and functors work in other languages.
//!
//! The `PropagatingEffectWitness` acts as a marker type to associate the `CausalPropagatingEffect`
//! with these HKT traits, facilitating generic programming over different causal effect types.
use crate::{CausalEffectLog, CausalPropagatingEffect, PropagatingEffectWitness};
use deep_causality_haft::{Applicative, Functor, HKT, HKT3, Monad};

/// Implements the `HKT` trait for `PropagatingEffectWitness`.
///
/// `PropagatingEffectWitness<E, L>` serves as a "witness" or "tag" type that
/// associates the `CausalPropagatingEffect<T, E, L>` with the Higher-Kinded Type
/// abstraction. This allows `CausalPropagatingEffect` to be treated generically
/// by traits like `Functor`, `Applicative`, and `Monad`.
///
/// The `Type<T>` associated type specifies that `CausalPropagatingEffect<T, E, L>`
/// is the concrete type that this HKT represents, where `T` is the inner value type,
/// `E` is the error type, and `L` is the log type.
impl<E, L> HKT for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalPropagatingEffect<T, E, L>;
}

/// Implements the `HKT3` trait for `PropagatingEffectWitness`.
///
/// Similar to `HKT`, this implementation specifically caters to types that have
/// three type parameters, where `E` and `L` are fixed, and `T` is the varying parameter.
/// This provides a more specific HKT abstraction for `CausalPropagatingEffect`.
impl<E, L> HKT3<E, L> for PropagatingEffectWitness<E, L> {
    type Type<T> = CausalPropagatingEffect<T, E, L>;
}

/// Implements the `Functor` trait for `PropagatingEffectWitness`.
///
/// The `Functor` trait provides the `fmap` method, which allows transforming
/// the inner value of a `CausalPropagatingEffect` without changing its structure,
/// error state, or associated logs.
///
/// # Arguments
/// * `m_a` - A `CausalPropagatingEffect` instance containing a value of type `A`.
/// * `f` - A function that takes a value of type `A` and returns a value of type `B`.
///
/// # Returns
/// A new `CausalPropagatingEffect` instance with the transformed value of type `B`,
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
        CausalPropagatingEffect {
            value: f(m_a.value),
            error: m_a.error,
            logs: m_a.logs,
        }
    }
}

/// Implements the `Applicative` trait for `PropagatingEffectWitness`.
///
/// The `Applicative` trait extends `Functor` by providing methods to lift
/// pure values into the `CausalPropagatingEffect` context (`pure`) and to apply
/// a function wrapped in `CausalPropagatingEffect` to a value wrapped in `CausalPropagatingEffect` (`apply`).
/// This allows for combining independent `CausalPropagatingEffect`s.
impl<E> Applicative<PropagatingEffectWitness<E, CausalEffectLog>>
    for PropagatingEffectWitness<E, CausalEffectLog>
where
    E: 'static + Clone,
{
    /// Lifts a pure value into a `CausalPropagatingEffect` context.
    ///
    /// Creates a new `CausalPropagatingEffect` with the given `value`,
    /// an empty error state (`None`), and an empty `CausalEffectLog`.
    ///
    /// # Arguments
    /// * `value` - The value to lift.
    ///
    /// # Returns
    /// A `CausalPropagatingEffect` containing the lifted value.
    ///
    fn pure<T>(value: T) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<T> {
        CausalPropagatingEffect {
            value,
            error: None,
            logs: CausalEffectLog::new(),
        }
    }

    /// Applies a function wrapped in a `CausalPropagatingEffect` to a value
    /// wrapped in a `CausalPropagatingEffect`.
    ///
    /// This method combines two `CausalPropagatingEffect` instances: one holding
    /// a function (`f_ab`) and another holding a value (`f_a`).
    ///
    /// # Error Handling
    /// If either `f_ab` or `f_a` contains an error, that error is propagated
    /// to the resulting `CausalPropagatingEffect`. The function application
    /// still occurs, but the error state is preserved.
    ///
    /// # Log Aggregation
    /// The logs from both `f_ab` and `f_a` are combined (appended) into the
    /// resulting `CausalPropagatingEffect`'s logs.
    ///
    /// # Arguments
    /// * `f_ab` - A `CausalPropagatingEffect` containing a function `Func: FnMut(A) -> B`.
    /// * `f_a` - A `CausalPropagatingEffect` containing a value of type `A`.
    ///
    /// # Returns
    /// A new `CausalPropagatingEffect` with the result of the function application,
    /// propagated errors, and combined logs.
    ///
    /// # Example
    /// ```
    /// use deep_causality_haft::{Applicative, HKT};
    /// use deep_causality::{CausalPropagatingEffect, PropagatingEffectWitness, CausalEffectLog};
    ///
    /// let add_one = |x: i32| x + 1;
    /// let effect_func = PropagatingEffectWitness::<&'static str, CausalEffectLog>::pure(add_one);
    /// let effect_val = PropagatingEffectWitness::<&'static str, CausalEffectLog>::pure(5);
    ///
    /// let result_effect = PropagatingEffectWitness::apply(effect_func, effect_val);
    /// assert_eq!(result_effect.value, 6);
    /// assert!(result_effect.error.is_none());
    /// ```
    fn apply<A, B, Func>(
        mut f_ab: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<Func>,
        mut f_a: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<A>,
    ) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>
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
            return CausalPropagatingEffect {
                value: (f_ab.value)(f_a.value),
                error: f_ab.error,
                logs: combined_logs,
            };
        }
        // If the value effect has an error, propagate it.
        // The value is still computed, but the error takes precedence.
        if f_a.error.is_some() {
            return CausalPropagatingEffect {
                value: (f_ab.value)(f_a.value),
                error: f_a.error,
                logs: combined_logs,
            };
        }

        CausalPropagatingEffect {
            value: (f_ab.value)(f_a.value),
            error: None,
            logs: combined_logs,
        }
    }
}

/// Implements the `Monad` trait for `PropagatingEffectWitness`.
///
/// The `Monad` trait provides the `bind` method, which is crucial for sequencing
/// computations that produce `CausalPropagatingEffect`s. It allows for chaining
/// operations where each subsequent operation might depend on the result of the
/// previous one, while correctly handling errors and aggregating logs.
impl<E> Monad<PropagatingEffectWitness<E, CausalEffectLog>>
    for PropagatingEffectWitness<E, CausalEffectLog>
where
    E: 'static + Clone,
{
    /// Sequences a computation by applying a function that returns another
    /// `CausalPropagatingEffect`.
    ///
    /// # Error Handling (Short-circuiting)
    /// If the initial `CausalPropagatingEffect` (`m_a`) contains an error,
    /// the `bind` operation short-circuits. The function `f` is not applied,
    /// and a new `CausalPropagatingEffect` is returned immediately with the
    /// original error and logs. This ensures that subsequent computations
    /// are not performed if an error has already occurred.
    ///
    /// # Log Aggregation
    /// If `m_a` does not contain an error, the function `f` is applied to
    /// `m_a.value` to produce `next_effect`. The logs from `m_a` are then
    /// combined (appended) with the logs from `next_effect`.
    ///
    /// # Arguments
    /// * `m_a` - The initial `CausalPropagatingEffect` instance.
    /// * `f` - A function that takes the inner value of `m_a` and returns a new `CausalPropagatingEffect`.
    ///
    /// # Returns
    /// A new `CausalPropagatingEffect` representing the sequenced computation,
    /// with errors propagated and logs aggregated.
    fn bind<A, B, Func>(
        m_a: <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<A>,
        f: Func,
    ) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>
    where
        Func: FnOnce(A) -> <PropagatingEffectWitness<E, CausalEffectLog> as HKT>::Type<B>,
    {
        // If there's an error in the initial effect, short-circuit and propagate it.
        // The function `f` is still called to get a default value for `B`,
        // but the error and logs from `m_a` are preserved.
        if m_a.error.is_some() {
            return CausalPropagatingEffect {
                value: f(m_a.value).value, // Call f to get a default value for B, but its error/logs are ignored.
                error: m_a.error,
                logs: m_a.logs,
            };
        }
        let mut next_effect = f(m_a.value);
        let mut combined_logs = m_a.logs;
        combined_logs.append(&mut next_effect.logs);
        next_effect.logs = combined_logs;
        next_effect
    }
}
