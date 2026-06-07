/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;

/// Forward-Euler endo-arrow: `s ↦ s + f(s)·dt`.
///
/// A first-order, value-level [`Arrow`](deep_causality_haft::Arrow) from a module-valued state `S`
/// to itself, carrying the step size `dt` and the rate field `f`. Iterate it with the
/// [`EndoArrow`](crate::EndoArrow) combinators to march, relax to a fixpoint, or run until an
/// event. Swap it for [`Rk4`](crate::Rk4) to raise accuracy with no change to the rate field.
pub struct Euler<S, R, F> {
    pub(crate) dt: R,
    pub(crate) rate: F,
    _state: PhantomData<S>,
}

impl<S, R, F> Euler<S, R, F> {
    /// Builds the Euler endo-arrow for step size `dt` under rate field `rate`.
    #[inline]
    pub fn new(dt: R, rate: F) -> Self {
        Euler {
            dt,
            rate,
            _state: PhantomData,
        }
    }
}
