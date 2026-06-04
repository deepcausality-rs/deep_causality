/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Scalar;

/// A scalar-generic arrow: a model whose evaluation is parametric in the working scalar.
///
/// This is the construct that hosts the tangent functor. A concrete value-level
/// `Arrow<In = f64, Out = f64>` cannot be lifted over `Dual` — its `run` only accepts `f64`.
/// By making `run` generic over the scalar, the same model evaluates at `f64` (the value) and
/// at `Dual` (the derivative), so [`Diff`](crate::Diff) and the
/// [`DifferentiateExt`](crate::DifferentiateExt) methods can instantiate it at `Dual<…>`.
///
/// A model is a named type (usually zero-sized), not a closure, because a closure cannot
/// carry a generic call signature.
pub trait DifferentiableArrow {
    /// Evaluate the model at working scalar `S`.
    fn run<S: Scalar>(&self, x: S) -> S;
}

/// The multi-input form: a scalar field of `N` inputs, `Rᴺ → R`. Differentiated by the
/// [`DifferentiateFieldExt`](crate::DifferentiateFieldExt) methods (`gradient`,
/// `directional_derivative`).
pub trait DifferentiableField<const N: usize> {
    /// Evaluate the field at working scalar `S`.
    fn run<S: Scalar>(&self, x: &[S; N]) -> S;
}
