/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DifferentiableArrow, DifferentiableField, Scalar};
use deep_causality_num_dual::Dual;

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

    /// `∇²f(x)` — the Hessian, `H[i][j] = ∂²f / ∂xᵢ∂xⱼ`, from the same model instantiated at
    /// `Dual<Dual<R>>`.
    ///
    /// Entry `(i, j)` seeds `xᵢ` in the inner `ε` and `xⱼ` in the outer `ε` and reads the
    /// `ε₁ε₂` channel. Only the upper triangle is evaluated, `N(N+1)/2` passes, and each value is
    /// written to both `(i, j)` and `(j, i)`, so the result is exactly symmetric. Allocation-free.
    #[inline]
    fn hessian<R: Scalar>(&self, x: &[R; N]) -> [[R; N]; N] {
        let mut h = [[R::zero(); N]; N];
        (0..N)
            .flat_map(|i| (i..N).map(move |j| (i, j)))
            .for_each(|(i, j)| {
                let seed: [Dual<Dual<R>>; N] = core::array::from_fn(|k| {
                    let inner = if k == i {
                        Dual::variable(x[k])
                    } else {
                        Dual::constant(x[k])
                    };
                    let outer = if k == j { R::one() } else { R::zero() };
                    Dual::new(inner, Dual::constant(outer))
                });
                let hij = self.run(&seed).derivative().derivative();
                h[i][j] = hij;
                h[j][i] = hij;
            });
        h
    }
}

impl<const N: usize, A: DifferentiableField<N>> DifferentiateFieldExt<N> for A {}
