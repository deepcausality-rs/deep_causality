/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Dual, Real};
use core::ops::Div;

/// Forward-mode derivative `f'(x)` of a closed-form scalar function.
///
/// Seeds `Dual::variable(x)` (`x + 1·ε`), runs `f`, and reads the `ε` channel.
///
/// ```
/// use deep_causality_num::derivative;
///
/// // f(x) = x³ + 2x → f'(3) = 3·3² + 2 = 29
/// assert_eq!(derivative(|x| x * x * x + x + x, 3.0_f64), 29.0);
/// ```
#[inline]
pub fn derivative<R, F>(f: F, x: R) -> R
where
    R: Real + Div<Output = R>,
    F: Fn(Dual<R>) -> Dual<R>,
{
    f(Dual::variable(x)).derivative()
}

/// Forward-mode value and derivative `(f(x), f'(x))` from a single evaluation.
///
/// ```
/// use deep_causality_num::value_and_derivative;
///
/// let (v, d) = value_and_derivative(|x| x * x, 4.0_f64);
/// assert_eq!(v, 16.0); // f(4)
/// assert_eq!(d, 8.0); // f'(4) = 2·4
/// ```
#[inline]
pub fn value_and_derivative<R, F>(f: F, x: R) -> (R, R)
where
    R: Real + Div<Output = R>,
    F: Fn(Dual<R>) -> Dual<R>,
{
    let y = f(Dual::variable(x));
    (y.value(), y.derivative())
}

/// Forward-mode second derivative `f''(x)` via nested duals.
///
/// Runs `f` over `Dual<Dual<R>>` and reads the cross `ε` channel. The differentiand
/// is written over the nested dual type, exactly as `Dual<Dual<R>>` is a
/// [`Real`](crate::Real).
///
/// ```
/// use deep_causality_num::second_derivative;
///
/// // f(x) = x⁴ → f''(2) = 12·2² = 48
/// assert_eq!(second_derivative(|x| x * x * x * x, 2.0_f64), 48.0);
/// ```
#[inline]
pub fn second_derivative<R, F>(f: F, x: R) -> R
where
    R: Real + Div<Output = R>,
    F: Fn(Dual<Dual<R>>) -> Dual<Dual<R>>,
{
    f(Dual::variable(Dual::variable(x)))
        .derivative()
        .derivative()
}
