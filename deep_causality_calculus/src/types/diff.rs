/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;

/// The derivative-arrow view of a [`DifferentiableArrow`](crate::DifferentiableArrow).
///
/// This is the tangent functor's action on a morphism, realized as a concrete value-level
/// [`Arrow`](deep_causality_haft::Arrow) from `Dual<R>` to `Dual<R>`. Because it is an ordinary
/// `Arrow`, it composes with the `arrow-strength` combinators (`compose` / `first` / `split` /
/// `fanout`): the functor *extends* the arrow algebra rather than replacing it. The base precision
/// `R` is carried as a phantom so the single scalar-generic model yields a concrete arrow per
/// precision.
pub struct Diff<A, R>(pub(crate) A, PhantomData<R>);

impl<A, R> Diff<A, R> {
    /// Wraps a differentiable model as its derivative arrow over `Dual<R>`.
    #[inline]
    pub const fn new(arrow: A) -> Self {
        Diff(arrow, PhantomData)
    }
}
