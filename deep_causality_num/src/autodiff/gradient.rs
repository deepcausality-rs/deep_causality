/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Dual, Real};
use core::ops::Div;

/// Forward-mode gradient `∇f` of a scalar field of `N` inputs.
///
/// One forward pass per coordinate: coordinate `i` is seeded as `Dual::variable`,
/// the rest as `Dual::constant`. Allocation-free (stack-sized `[R; N]`).
///
/// ```
/// use deep_causality_num::gradient;
///
/// // f(x, y) = x² + y² → ∇f(3, 4) = [6, 8]
/// let g = gradient(|p| p[0] * p[0] + p[1] * p[1], &[3.0_f64, 4.0]);
/// assert_eq!(g, [6.0, 8.0]);
/// ```
#[inline]
pub fn gradient<R, F, const N: usize>(f: F, x: &[R; N]) -> [R; N]
where
    R: Real + Div<Output = R>,
    F: Fn(&[Dual<R>; N]) -> Dual<R>,
{
    core::array::from_fn(|i| {
        let seed: [Dual<R>; N] = core::array::from_fn(|j| {
            if j == i {
                Dual::variable(x[j])
            } else {
                Dual::constant(x[j])
            }
        });
        f(&seed).derivative()
    })
}

/// Forward-mode directional derivative `∇f(x) · dir` in a single pass.
///
/// Seeds coordinate `j` as `Dual::new(x[j], dir[j])`, so the `ε` channel carries the
/// directional combination directly.
///
/// ```
/// use deep_causality_num::directional_derivative;
///
/// // f(x, y) = x·y, at (2, 3) along (1, 1): ∇f·dir = y + x = 5
/// let d = directional_derivative(|p| p[0] * p[1], &[2.0_f64, 3.0], &[1.0, 1.0]);
/// assert_eq!(d, 5.0);
/// ```
#[inline]
pub fn directional_derivative<R, F, const N: usize>(f: F, x: &[R; N], dir: &[R; N]) -> R
where
    R: Real + Div<Output = R>,
    F: Fn(&[Dual<R>; N]) -> Dual<R>,
{
    let seed: [Dual<R>; N] = core::array::from_fn(|j| Dual::new(x[j], dir[j]));
    f(&seed).derivative()
}
