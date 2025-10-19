/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Applicative, Foldable, Functor, HKT, HKT2, Monad, Placeholder};

/// `ResultWitness<E>` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `Result<T, E>` type constructor, where the error type `E` is fixed.
///
/// It allows `Result` to be used with generic functional programming traits like `Functor`,
/// `Applicative`, `Foldable`, and `Monad` by fixing one of its type parameters.
pub struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
    /// Specifies that `ResultWitness<E>` represents the `Result<T, E>` type constructor
    /// with a fixed error type `E`.
    type Type<T> = Result<T, E>;
}

impl<E> HKT for ResultWitness<E> {
    /// Specifies that `ResultWitness<E>` also acts as a single-parameter HKT,
    /// where the `E` parameter is considered part of the "witness" itself.
    type Type<T> = Result<T, E>;
}

// Implementation of Functor for ResultWitness
impl<E> Functor<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    /// Implements the `fmap` operation for `Result<T, E>`.
    ///
    /// If the `Result` is `Ok(value)`, the function `f` is applied to `value`,
    /// and the result is wrapped in `Ok`. If the `Result` is `Err(error)`, `Err(error)` is returned.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The `Result` to map over.
    /// *   `f`: The function to apply to the value inside the `Result`.
    ///
    /// # Returns
    ///
    /// A new `Result` with the function applied to its content, or the original `Err`.
    fn fmap<A, B, Func>(
        m_a: <ResultWitness<E> as HKT2<E>>::Type<A>,
        f: Func,
    ) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnOnce(A) -> B,
    {
        m_a.map(f)
    }
}

// Implementation of Applicative for ResultWitness
impl<E> Applicative<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static + Clone,
{
    /// Lifts a pure value into an `Ok` variant of `Result`.
    ///
    /// # Arguments
    ///
    /// *   `value`: The value to wrap in `Ok`.
    ///
    /// # Returns
    ///
    /// `Ok(value)`.
    fn pure<T>(value: T) -> <ResultWitness<E> as HKT2<E>>::Type<T> {
        Ok(value)
    }

    /// Applies a function wrapped in a `Result` (`f_ab`) to a value wrapped in a `Result` (`f_a`).
    ///
    /// If both `f_ab` and `f_a` are `Ok`, the function is applied to the value.
    /// If either is `Err`, the first encountered `Err` is propagated.
    ///
    /// # Arguments
    ///
    /// *   `f_ab`: A `Result` containing the function.
    /// *   `f_a`: A `Result` containing the argument.
    ///
    /// # Returns
    ///
    /// A `Result` containing the result of the application, or an `Err`.
    fn apply<A, B, Func>(
        f_ab: <ResultWitness<E> as HKT2<E>>::Type<Func>,
        f_a: <ResultWitness<E> as HKT2<E>>::Type<A>,
    ) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnMut(A) -> B,
    {
        match f_ab {
            Ok(mut f) => match f_a {
                Ok(a) => Ok(f(a)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

// Implementation of Foldable for ResultWitness
impl<E> Foldable<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static,
{
    /// Folds (reduces) a `Result` into a single value.
    ///
    /// If the `Result` is `Ok(value)`, the function `f` is applied with the initial
    /// accumulator and the `value`. If the `Result` is `Err`, the initial accumulator
    /// is returned.
    ///
    /// # Arguments
    ///
    /// *   `fa`: The `Result` to fold.
    /// *   `init`: The initial accumulator value.
    /// *   `f`: The folding function.
    ///
    /// # Returns
    ///
    /// The accumulated result.
    fn fold<A, B, Func>(fa: Result<A, E>, init: B, mut f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        match fa {
            Ok(a) => f(init, a),
            Err(_) => init,
        }
    }
}

// Implementation of Monad for ResultWitness
impl<E> Monad<ResultWitness<E>> for ResultWitness<E>
where
    E: 'static + Clone,
{
    /// Implements the `bind` (or `and_then`) operation for `Result<T, E>`.
    ///
    /// If the `Result` is `Ok(value)`, the function `f` is applied to `value`,
    /// which itself returns a `Result`. If the `Result` is `Err(error)`, `Err(error)` is returned.
    /// This effectively chains computations that might fail, propagating the first error.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The initial `Result`.
    /// *   `f`: A function that takes the inner value of `m_a` and returns a new `Result`.
    ///
    /// # Returns
    ///
    /// A new `Result` representing the chained computation.
    fn bind<A, B, Func>(
        m_a: <ResultWitness<E> as HKT2<E>>::Type<A>,
        f: Func,
    ) -> <ResultWitness<E> as HKT2<E>>::Type<B>
    where
        Func: FnOnce(A) -> <ResultWitness<E> as HKT2<E>>::Type<B>,
    {
        m_a.and_then(f)
    }
}
