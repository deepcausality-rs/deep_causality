/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;

/// Classical fourth-order Runge–Kutta endo-arrow:
/// `s ↦ s + (k₁ + 2k₂ + 2k₃ + k₄)·dt/6`, with the standard half-step stages.
///
/// A drop-in replacement for [`Euler`](crate::Euler): same state, same rate field, far higher
/// accuracy. A value-level [`Arrow`](deep_causality_haft::Arrow) from a module-valued state `S` to
/// itself; iterate it with the [`EndoArrow`](crate::EndoArrow) combinators.
pub struct Rk4<S, R, F> {
    pub(crate) dt: R,
    pub(crate) rate: F,
    _state: PhantomData<S>,
}

impl<S, R, F> Rk4<S, R, F> {
    /// Builds the RK4 endo-arrow for step size `dt` under rate field `rate`.
    #[inline]
    pub fn new(dt: R, rate: F) -> Self {
        Rk4 {
            dt,
            rate,
            _state: PhantomData,
        }
    }
}
