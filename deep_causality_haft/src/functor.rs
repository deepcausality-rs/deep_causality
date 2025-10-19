/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::HKT;

/// Functor: Abstracts over the ability to map a function over a type constructor.
///
/// Generic over the HKT witness `F`.
pub trait Functor<F: HKT> {
    /// Applies a function `f` to the value inside the container `m_a`.
    fn fmap<A, B, Func>(m_a: F::Type<A>, f: Func) -> F::Type<B>
    where
        Func: FnMut(A) -> B;
}
