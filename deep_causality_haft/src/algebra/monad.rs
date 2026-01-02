/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Applicative, HKT, Satisfies};

/// The `Monad` trait extends `Applicative` by providing a `bind` operation
/// for sequencing computations that produce effectful values.
///
/// Monads are fundamental for managing side-effects and controlling the flow
/// of computations in a functional style, allowing for powerful abstractions
/// like error handling, logging, and state management.
///
/// This trait is generic over `F`, which is a Higher-Kinded Type (HKT) witness.
///
/// # Constraint Support
///
/// The `bind` function now requires both input `A` and output `B` types
/// to satisfy the HKT's constraint. This ensures type-safe chaining for
/// constrained types like `CausalTensor<T>` where `T: TensorData`.
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
    /// ```rust
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
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: FnMut(A) -> F::Type<B>;

    /// Flatten a nested structure into a single layer.
    ///
    /// Default implementation using `bind` with identity function.
    ///
    /// # Arguments
    ///
    /// *   `m_m_a`: A nested effectful value (`F::Type<F::Type<A>>`).
    ///
    /// # Returns
    ///
    /// A flattened effectful value (`F::Type<A>`).
    fn join<A>(m_m_a: F::Type<F::Type<A>>) -> F::Type<A>
    where
        A: Satisfies<F::Constraint>,
        F::Type<A>: Satisfies<F::Constraint>,
    {
        Self::bind(m_m_a, |x| x)
    }
}
