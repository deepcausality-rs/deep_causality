/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Applicative, HKT};

/// The `Monad` trait extends `Applicative` by providing a `bind` operation
/// for sequencing computations that produce effectful values.
///
/// Monads are fundamental for managing side-effects and controlling the flow
/// of computations in a functional style, allowing for powerful abstractions
/// like error handling, logging, and state management.
///
/// This trait is generic over `F`, which is a Higher-Kinded Type (HKT) witness.
///
/// # Laws (Informal)
///
/// 1.  **Left Identity**: `pure(a).bind(f) == f(a)`
/// 2.  **Right Identity**: `m.bind(pure) == m`
/// 3.  **Associativity**: `m.bind(|x| f(x).bind(g)) == m.bind(f).bind(g)`
///
/// # Type Parameters
///
/// *   `F`: A Higher-Kinded Type (HKT) witness that represents the type constructor
///     (e.g., `OptionWitness`, `ResultWitness<E>`, `VecWitness`).
pub trait Monad<F: HKT>: Applicative<F> {
    /// Chains a computation from an effectful value, flattening the result.
    /// This is the core sequencing operation of a Monad.
    ///
    /// The `bind` method takes an effectful value (`m_a`) and a function `f`.
    /// The function `f` takes the inner value of `m_a` and returns a new effectful value.
    /// `bind` then flattens this nested structure into a single effectful value.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The initial effectful value (`F::Type<A>`).
    /// *   `f`: A function that takes the inner value of `m_a` and returns a new effectful value (`F::Type<B>`).
    ///
    /// # Returns
    ///
    /// A new effectful value (`F::Type<B>`) representing the chained and flattened computation.
    ///
    /// # Type Parameters
    ///
    /// *   `A`: The type of the value inside the initial effectful context.
    /// *   `B`: The type of the value inside the resulting effectful context.
    /// *   `Func`: The type of the binding function, which must be `FnMut(A) -> F::Type<B>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_haft::{Monad, OptionWitness, HKT};
    ///
    /// let opt_a: Option<<OptionWitness as HKT>::Type<i32>> = Some(Some(5));
    /// let f = |x| Some(x * 2);
    /// let opt_b = OptionWitness::bind(opt_a.unwrap(), f);
    /// assert_eq!(opt_b, Some(10));
    ///
    /// let opt_none: Option<<OptionWitness as HKT>::Type<i32>> = Some(None);
    /// let opt_none_bound = OptionWitness::bind(opt_none.unwrap(), f);
    /// assert_eq!(opt_none_bound, None);
    /// ```
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        // The function must return a new effectful type (F::Type<B>)
        Func: FnMut(A) -> F::Type<B>;
}
