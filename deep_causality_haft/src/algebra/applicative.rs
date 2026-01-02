/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Functor, Satisfies, HKT};

/// The `Applicative` trait extends `Functor` by providing methods to apply a function
/// wrapped in a context to a value wrapped in a context, and to lift a pure value
/// into the minimal context.
///
/// This trait is generic over `F`, which is a Higher-Kinded Type (HKT) witness.
///
/// # Constraint Support
///
/// Both `pure` and `apply` now require types to satisfy the HKT's constraint.
/// This ensures type-safe lifting and application for constrained types.
///
/// # Laws (Informal)
///
/// 1.  **Identity**: `pure(id).apply(v) == v`
/// 2.  **Homomorphism**: `pure(f).apply(pure(x)) == pure(f(x))`
/// 3.  **Interchange**: `u.apply(pure(y)) == pure(|f| f(y)).apply(u)`
///
/// # Type Parameters
///
/// *   `F`: A Higher-Kinded Type (HKT) witness that represents the type constructor
///     (e.g., `OptionWitness`, `ResultWitness<E>`).
pub trait Applicative<F: HKT>: Functor<F> {
    /// Lifts a pure value into the minimal applicative context `F::Type<T>`.
    ///
    /// This is often used to introduce a value into an effectful computation
    /// without any side effects.
    ///
    /// # Arguments
    ///
    /// *   `value`: The pure value to lift.
    ///
    /// # Returns
    ///
    /// An instance of `F::Type<T>` containing the `value`.
    fn pure<T>(value: T) -> F::Type<T>
    where
        T: Satisfies<F::Constraint>;

    /// Applies a function wrapped in a context (`f_ab`) to a value wrapped in a context (`f_a`).
    ///
    /// This allows sequencing computations where both the function and its argument
    /// are within the same applicative context.
    ///
    /// # Arguments
    ///
    /// *   `f_ab`: An instance of `F::Type<Func>` containing the function to apply.
    /// *   `f_a`: An instance of `F::Type<A>` containing the argument for the function.
    ///
    /// # Returns
    ///
    /// An instance of `F::Type<B>` containing the result of the application,
    /// or an appropriate error/empty context if either `f_ab` or `f_a` is in an
    /// error/empty state.
    ///
    /// # Type Parameters
    ///
    /// *   `A`: The input type of the function.
    /// *   `B`: The output type of the function.
    /// *   `Func`: The type of the function, which must be `FnMut(A) -> B`.
    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        A: Satisfies<F::Constraint> + Clone,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> B;
}
