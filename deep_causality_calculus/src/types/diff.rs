/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DifferentiableArrow, Scalar};
use core::marker::PhantomData;
use deep_causality_haft::Arrow;
use deep_causality_num::Dual;

/// The derivative-arrow view of a [`DifferentiableArrow`].
///
/// This is the tangent functor's action on a morphism, realized as a concrete value-level
/// [`Arrow`] from `Dual<R>` to `Dual<R>`. Because it is an ordinary `Arrow`, it composes with
/// the `arrow-strength` combinators (`compose` / `first` / `split` / `fanout`): the functor
/// *extends* the arrow algebra rather than replacing it. The base precision `R` is carried as
/// a phantom so the single scalar-generic model yields a concrete arrow per precision.
pub struct Diff<A, R>(A, PhantomData<R>);

impl<A, R> Diff<A, R> {
    /// Wraps a differentiable model as its derivative arrow over `Dual<R>`.
    #[inline]
    pub const fn new(arrow: A) -> Self {
        Diff(arrow, PhantomData)
    }
}

impl<A, R> Arrow for Diff<A, R>
where
    A: DifferentiableArrow,
    R: Scalar,
{
    type In = Dual<R>;
    type Out = Dual<R>;

    #[inline]
    fn run(&self, input: Dual<R>) -> Dual<R> {
        self.0.run(input)
    }
}
