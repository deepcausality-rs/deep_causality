/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Scalar;
use core::marker::PhantomData;
use core::ops::{Add, Mul};
use deep_causality_haft::Arrow;

/// Forward-Euler endo-arrow: `s ↦ s + f(s)·dt`.
///
/// A first-order, value-level [`Arrow`] from a module-valued state `S` to itself, carrying the
/// step size `dt` and the rate field `f`. Iterate it with the
/// [`EndoArrow`](crate::EndoArrow) combinators to march, relax to a fixpoint, or run until an
/// event. Swap it for [`Rk4`](crate::Rk4) to raise accuracy with no change to the rate field.
pub struct Euler<S, R, F> {
    dt: R,
    rate: F,
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

impl<S, R, F> Arrow for Euler<S, R, F>
where
    S: Add<Output = S> + Mul<R, Output = S>,
    R: Scalar,
    F: Fn(&S) -> S,
{
    type In = S;
    type Out = S;

    #[inline]
    fn run(&self, state: S) -> S {
        let rate = (self.rate)(&state);
        state + rate * self.dt
    }
}
