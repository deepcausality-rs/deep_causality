/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DifferentiableArrow, DifferentiableField, Scalar};
use deep_causality_num::Dual;

/// The tangent functor as a fluent type extension on any [`DifferentiableArrow`].
///
/// This is the `…Ext` convention used across the math crates (`CausalTensorMathExt`, …):
/// a blanket-implemented extension trait that adds methods to every model, so differentiation
/// reads `model.derivative(x)` rather than `derivative(&model, x)`. Each method seeds `Dual`
/// internally, runs the scalar-generic model over it, and reads the `ε` channel — the caller
/// never names `Dual`. Never implemented by hand.
pub trait DifferentiateExt: DifferentiableArrow {
    /// `f'(x)` at base precision `R`.
    #[inline]
    fn derivative<R: Scalar>(&self, x: R) -> R {
        self.run(Dual::<R>::variable(x)).derivative()
    }

    /// `(f(x), f'(x))` from a single evaluation.
    #[inline]
    fn value_and_derivative<R: Scalar>(&self, x: R) -> (R, R) {
        let y = self.run(Dual::<R>::variable(x));
        (y.value(), y.derivative())
    }

    /// `f''(x)` — the functor instantiated at `Dual<Dual<R>>`, same model.
    #[inline]
    fn second_derivative<R: Scalar>(&self, x: R) -> R {
        self.run(Dual::<Dual<R>>::variable(Dual::<R>::variable(x)))
            .derivative()
            .derivative()
    }
}

impl<A: DifferentiableArrow> DifferentiateExt for A {}

/// The multi-input tangent functor as a fluent extension on any [`DifferentiableField`].
pub trait DifferentiateFieldExt<const N: usize>: DifferentiableField<N> {
    /// `∇f(x)` — one seeded coordinate per pass, allocation-free.
    #[inline]
    fn gradient<R: Scalar>(&self, x: &[R; N]) -> [R; N] {
        core::array::from_fn(|i| {
            let seed: [Dual<R>; N] = core::array::from_fn(|j| {
                if j == i {
                    Dual::variable(x[j])
                } else {
                    Dual::constant(x[j])
                }
            });
            self.run(&seed).derivative()
        })
    }

    /// `∇f(x) · dir` in a single pass (seed coordinate `j` as `x[j] + dir[j]·ε`).
    #[inline]
    fn directional_derivative<R: Scalar>(&self, x: &[R; N], dir: &[R; N]) -> R {
        let seed: [Dual<R>; N] = core::array::from_fn(|j| Dual::new(x[j], dir[j]));
        self.run(&seed).derivative()
    }
}

impl<const N: usize, A: DifferentiableField<N>> DifferentiateFieldExt<N> for A {}
