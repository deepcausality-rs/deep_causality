/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Functor, HKT};

/// Monad: Abstracts over the ability to sequence computations within a type constructor.
///
/// Generic over the HKT witness `F`.
pub trait Monad<F: HKT>: Functor<F> {
    /// Lifts a value into the minimal monadic context. (e.g., Some(value), Ok(value)).
    fn pure<T>(value: T) -> F::Type<T>;

    /// Chains a computation from an effectful value, flattening the result.
    /// This is the core sequencing operation.
    fn bind<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        // The function must return a new effectful type (F::Type<B>)
        Func: FnMut(A) -> F::Type<B>;
}
