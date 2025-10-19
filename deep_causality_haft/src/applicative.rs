/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Functor, HKT};

/// Applicative: Abstracts over the ability to apply a function wrapped in a context
/// to a value wrapped in a context.
///
/// Generic over the HKT witness `F`.
pub trait Applicative<F: HKT>: Functor<F> {
    /// Lifts a pure value into the minimal applicative context.
    fn pure<T>(value: T) -> F::Type<T>;

    /// Applies a function wrapped in a context to a value wrapped in a context.
    fn apply<A, B, Func>(f_ab: F::Type<Func>, f_a: F::Type<A>) -> F::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone;
}
